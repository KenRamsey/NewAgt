//! Lexical analyzer for ASCII AGT (PDF §6.1 lex rules).

use crate::keyword::Keyword;
use crate::span::Span;
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    UnexpectedCharacter {
        ch: char,
        span: Span,
    },
    IntegerOverflow {
        span: Span,
    },
    InvalidReal {
        span: Span,
    },
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedCharacter { ch, span } => {
                write!(
                    f,
                    "unexpected character {:?} at line {}, column {}",
                    ch, span.line, span.column
                )
            }
            Self::IntegerOverflow { span } => {
                write!(
                    f,
                    "integer literal out of range at line {}, column {}",
                    span.line, span.column
                )
            }
            Self::InvalidReal { span } => {
                write!(
                    f,
                    "invalid real literal at line {}, column {}",
                    span.line, span.column
                )
            }
        }
    }
}

impl std::error::Error for LexError {}

struct Lexer<'src> {
    source: &'src str,
    bytes: &'src [u8],
    pos: usize,
    line: u32,
    column: u32,
}

impl<'src> Lexer<'src> {
    fn new(source: &'src str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    fn current_span(&self) -> Span {
        Span::new(self.line, self.column, self.pos)
    }

    fn bump(&mut self) -> Option<u8> {
        if self.at_end() {
            return None;
        }
        let b = self.bytes[self.pos];
        self.pos += 1;
        if b == b'\r' {
            if self.peek() == Some(b'\n') {
                self.pos += 1;
            }
            self.line += 1;
            self.column = 1;
        } else if b == b'\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(b)
    }

    fn peek(&self) -> Option<u8> {
        if self.at_end() {
            None
        } else {
            Some(self.bytes[self.pos])
        }
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.bump();
        }
    }

    fn is_ident_continue(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'_'
    }

    fn slice(&self, start: usize, end: usize) -> &'src str {
        &self.source[start..end]
    }

    fn lex_qstring(&mut self) -> Token {
        let start = self.pos;
        let span_start = self.current_span();
        debug_assert_eq!(self.peek(), Some(b'"'));
        self.bump(); // opening quote

        while let Some(b) = self.peek() {
            if b == b'"' {
                self.bump();
                let text = self.slice(start + 1, self.pos - 1);
                return Token::new(TokenKind::String(text.to_string()), span_start);
            }
            if b == b'\n' {
                self.bump();
                let text = self.slice(start + 1, self.pos - 1);
                return Token::new(TokenKind::String(text.to_string()), span_start);
            }
            self.bump();
        }

        // EOF without closing quote — content after opening quote
        let text = self.slice(start + 1, self.pos);
        Token::new(TokenKind::String(text.to_string()), span_start)
    }

    /// PDF real: `([+-]*[0-9]*"."[0-9]+)|([0-9]*"."[0-9]+[eE][+-]?[0-9]+)`
    fn try_lex_real(&mut self) -> Option<Result<Token, LexError>> {
        let start = self.pos;
        let span_start = self.current_span();

        let try_pattern1 = |bytes: &[u8], i: &mut usize| -> bool {
            let mut j = *i;
            while j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
                j += 1;
            }
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j >= bytes.len() || bytes[j] != b'.' {
                return false;
            }
            j += 1;
            let mut digits_after = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                digits_after += 1;
                j += 1;
            }
            if digits_after == 0 {
                return false;
            }
            *i = j;
            true
        };

        let try_pattern2 = |bytes: &[u8], i: &mut usize| -> bool {
            let mut j = *i;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                j += 1;
            }
            if j >= bytes.len() || bytes[j] != b'.' {
                return false;
            }
            j += 1;
            let mut digits_after = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                digits_after += 1;
                j += 1;
            }
            if digits_after == 0 {
                return false;
            }
            if j >= bytes.len() || (bytes[j] != b'e' && bytes[j] != b'E') {
                return false;
            }
            j += 1;
            if j < bytes.len() && (bytes[j] == b'+' || bytes[j] == b'-') {
                j += 1;
            }
            let mut exp_digits = 0;
            while j < bytes.len() && bytes[j].is_ascii_digit() {
                exp_digits += 1;
                j += 1;
            }
            if exp_digits == 0 {
                return false;
            }
            *i = j;
            true
        };

        let bytes = self.bytes;
        let mut i2 = start;
        if try_pattern1(bytes, &mut i2) {
            let text = self.slice(start, i2);
            let len = i2 - start;
            self.pos = i2;
            self.column = span_start.column + len as u32;
            match text.parse::<f64>() {
                Ok(v) => return Some(Ok(Token::new(TokenKind::Real(v), span_start))),
                Err(_) => return Some(Err(LexError::InvalidReal { span: span_start })),
            }
        }

        let mut i3 = start;
        if try_pattern2(bytes, &mut i3) {
            let text = self.slice(start, i3);
            let len = i3 - start;
            self.pos = i3;
            self.column = span_start.column + len as u32;
            match text.parse::<f64>() {
                Ok(v) => return Some(Ok(Token::new(TokenKind::Real(v), span_start))),
                Err(_) => return Some(Err(LexError::InvalidReal { span: span_start })),
            }
        }

        None
    }

    fn try_lex_integer(&mut self) -> Option<Result<Token, LexError>> {
        let span_start = self.current_span();
        let start = self.pos;
        let bytes = self.bytes;
        let mut i = start;
        if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        if i >= bytes.len() || !bytes[i].is_ascii_digit() {
            return None;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        let text = self.slice(start, i);
        self.pos = i;
        self.column = span_start.column + (i - start) as u32;
        match text.parse::<i64>() {
            Ok(v) => Some(Ok(Token::new(TokenKind::Integer(v), span_start))),
            Err(_) => Some(Err(LexError::IntegerOverflow { span: span_start })),
        }
    }

    fn lex_keyword(&mut self) -> Option<Token> {
        if !self.peek().is_some_and(|b| b.is_ascii_alphabetic()) {
            return None;
        }
        let start = self.pos;
        let span_start = self.current_span();
        while self
            .peek()
            .is_some_and(|b| Self::is_ident_continue(b))
        {
            self.bump();
        }
        let text = self.slice(start, self.pos);
        if text.is_empty() {
            return None;
        }
        let kind = if let Some(kw) = Keyword::from_ident(text) {
            TokenKind::Keyword(kw)
        } else {
            TokenKind::UnknownKeyword(text.to_string())
        };
        Some(Token::new(kind, span_start))
    }

    fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        self.skip_ws();
        if self.at_end() {
            return Ok(None);
        }

        let span_start = self.current_span();
        match self.peek() {
            Some(b'{') => {
                self.bump();
                Ok(Some(Token::new(TokenKind::LBrace, span_start)))
            }
            Some(b'}') => {
                self.bump();
                Ok(Some(Token::new(TokenKind::RBrace, span_start)))
            }
            Some(b'"') => Ok(Some(self.lex_qstring())),
            Some(b) if b.is_ascii_digit() || b == b'+' || b == b'-' || b == b'.' => {
                if let Some(res) = self.try_lex_real() {
                    return res.map(Some);
                }
                if let Some(res) = self.try_lex_integer() {
                    return res.map(Some);
                }
                if b == b'+' || b == b'-' {
                    // lone sign — fall through to error
                } else if b == b'.' {
                    // incomplete real
                }
                Err(LexError::UnexpectedCharacter {
                    ch: self.source[self.pos..].chars().next().unwrap(),
                    span: span_start,
                })
            }
            Some(b) if b.is_ascii_alphabetic() => Ok(self.lex_keyword()),
            Some(_) => {
                let ch = self.source[self.pos..].chars().next().unwrap();
                Err(LexError::UnexpectedCharacter {
                    ch,
                    span: span_start,
                })
            }
            None => Ok(None),
        }
    }
}

/// Tokenize all of `source` into a vector (PDF §6.1 lex behavior).
pub fn lex(source: &str) -> Result<Vec<Token>, LexError> {
    let mut lexer = Lexer::new(source);
    let mut tokens = Vec::new();
    while let Some(tok) = lexer.next_token()? {
        tokens.push(tok);
    }
    Ok(tokens)
}

/// Iterator over tokens from `source`.
pub fn lex_tokens(source: &str) -> LexTokenIter<'_> {
    LexTokenIter {
        lexer: Lexer::new(source),
        error: None,
    }
}

pub struct LexTokenIter<'src> {
    lexer: Lexer<'src>,
    error: Option<LexError>,
}

impl<'src> Iterator for LexTokenIter<'src> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(err) = self.error.take() {
            return Some(Err(err));
        }
        match self.lexer.next_token() {
            Ok(Some(tok)) => Some(Ok(tok)),
            Ok(None) => None,
            Err(e) => {
                self.error = Some(e);
                self.next()
            }
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::keyword::Keyword;

    #[test]
    fn unknown_keyword_is_not_single_char() {
        let tokens = lex("Foo").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(
            tokens[0].kind,
            TokenKind::UnknownKeyword("Foo".to_string())
        );
    }

    #[test]
    fn real_and_integer_distinction() {
        let tokens = lex("200 45.000000").unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Integer(200)));
        assert!(matches!(tokens[1].kind, TokenKind::Real(r) if (r - 45.0).abs() < f64::EPSILON));
    }

    #[test]
    fn signed_integer_literals() {
        let tokens = lex("-3 197 +42").unwrap();
        assert!(matches!(tokens[0].kind, TokenKind::Integer(-3)));
        assert!(matches!(tokens[1].kind, TokenKind::Integer(197)));
        assert!(matches!(tokens[2].kind, TokenKind::Integer(42)));
    }

    #[test]
    fn extension_keyword_token() {
        let tokens = lex("Keyword").unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Keyword));
    }

    #[test]
    fn underscore_in_unknown_keyword() {
        let tokens = lex("PLATFORM_LATITUDE 63.835").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens[0].kind,
            TokenKind::UnknownKeyword("PLATFORM_LATITUDE".to_string())
        );
        assert!(matches!(tokens[1].kind, TokenKind::Real(_)));
    }

    #[test]
    fn crlf_line_endings() {
        let tokens = lex("Agt\r\n{\r\n}").unwrap();
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Agt));
        assert_eq!(tokens[1].kind, TokenKind::LBrace);
        assert_eq!(tokens[2].kind, TokenKind::RBrace);
        assert_eq!(tokens[1].span.line, 2);
        assert_eq!(tokens[2].span.line, 3);
    }
}
