//! Recursive-descent parser for PDF §6.2 container grammar.

use crate::ast::{
    Agt, AgtItem, Document, Field, PrjItem, PrjSect, SenSect, SenSectItem, SenUpd, SenUpdItem,
    Tgt, TgtAbs, TgtAbsItem, TgtItem, TgtSect, TgtSectItem, TgtSenRel, TgtSenRelItem, TgtUpd,
    TgtUpdItem, UnknownField,
};
use crate::keyword::Keyword;
use crate::lex::lex;
use crate::span::Span;
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    Lex(crate::lex::LexError),
    Unexpected {
        span: Span,
        expected: &'static str,
        found: Found,
    },
    ExtraInput {
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Found {
    End,
    Token(TokenKind),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lex(e) => write!(f, "lex error: {e}"),
            Self::Unexpected {
                span,
                expected,
                found,
            } => write!(
                f,
                "parse error at line {}, column {}: expected {}, found {}",
                span.line, span.column, expected, found
            ),
            Self::ExtraInput { span } => write!(
                f,
                "parse error at line {}, column {}: unexpected tokens after document",
                span.line, span.column
            ),
        }
    }
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for Found {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::End => f.write_str("end of input"),
            Self::Token(kind) => write!(f, "{kind:?}"),
        }
    }
}

/// Parse a complete AGT document (`Agt { … }` root).
pub fn parse(source: &str) -> Result<Document, ParseError> {
    let tokens = lex(source).map_err(ParseError::Lex)?;
    let mut parser = Parser::new(tokens);
    let root = parser.parse_agt()?;
    if parser.peek().is_some() {
        let span = parser.peek().unwrap().span;
        return Err(ParseError::ExtraInput { span });
    }
    Ok(Document { root })
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn bump(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }

    fn found(&self) -> Found {
        self.peek()
            .map(|t| Found::Token(t.kind.clone()))
            .unwrap_or(Found::End)
    }

    fn unexpected(&self, expected: &'static str) -> ParseError {
        ParseError::Unexpected {
            span: self.peek().map(|t| t.span).unwrap_or(Span::new(1, 1, 0)),
            expected,
            found: self.found(),
        }
    }

    fn expect_lbrace(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Some(t) if matches!(t.kind, TokenKind::LBrace) => {
                self.bump();
                Ok(())
            }
            _ => Err(self.unexpected("`{`")),
        }
    }

    fn expect_rbrace(&mut self) -> Result<(), ParseError> {
        match self.peek() {
            Some(t) if matches!(t.kind, TokenKind::RBrace) => {
                self.bump();
                Ok(())
            }
            _ => Err(self.unexpected("`}`")),
        }
    }

    fn consume_value_tokens(&mut self) -> Vec<Token> {
        let mut vals = Vec::new();
        while let Some(tok) = self.peek() {
            match &tok.kind {
                TokenKind::Integer(_) | TokenKind::Real(_) | TokenKind::String(_) => {
                    vals.push(self.bump());
                }
                _ => break,
            }
        }
        vals
    }

    fn parse_field(&mut self, keyword: Keyword, keyword_span: Span) -> Field {
        let values = self.consume_value_tokens();
        Field {
            keyword,
            keyword_span,
            values,
        }
    }

    fn parse_unknown_field(&mut self, name: String, keyword_span: Span) -> UnknownField {
        let values = self.consume_value_tokens();
        UnknownField {
            name,
            keyword_span,
            values,
        }
    }

    fn parse_agt(&mut self) -> Result<Agt, ParseError> {
        let start = self.peek().ok_or_else(|| self.unexpected("`Agt`"))?;
        if !matches!(start.kind, TokenKind::Keyword(Keyword::Agt)) {
            return Err(self.unexpected("`Agt`"));
        }
        let span = start.span;
        self.bump();
        self.expect_lbrace()?;

        let mut items = Vec::new();
        while !matches!(self.peek(), Some(t) if matches!(t.kind, TokenKind::RBrace)) {
            if self.at_end() {
                return Err(self.unexpected("`}`"));
            }
            items.push(self.parse_agt_item()?);
        }
        self.expect_rbrace()?;

        Ok(Agt { span, items })
    }

    fn parse_agt_item(&mut self) -> Result<AgtItem, ParseError> {
        let tok = self.peek().ok_or_else(|| self.unexpected("section keyword"))?;
        match &tok.kind {
            TokenKind::Keyword(Keyword::PrjSect) => {
                Ok(AgtItem::PrjSect(self.parse_prj_sect()?))
            }
            TokenKind::Keyword(Keyword::SenSect) => {
                Ok(AgtItem::SenSect(self.parse_sen_sect()?))
            }
            TokenKind::Keyword(Keyword::TgtSect) => {
                Ok(AgtItem::TgtSect(self.parse_tgt_sect()?))
            }
            TokenKind::Keyword(Keyword::Tgt) => Err(ParseError::Unexpected {
                span: tok.span,
                expected: "`PrjSect`, `SenSect`, or `TgtSect`",
                found: Found::Token(TokenKind::Keyword(Keyword::Tgt)),
            }),
            TokenKind::Keyword(kw) => Err(ParseError::Unexpected {
                span: tok.span,
                expected: "`PrjSect`, `SenSect`, or `TgtSect`",
                found: Found::Token(TokenKind::Keyword(*kw)),
            }),
            TokenKind::UnknownKeyword(name) => Err(ParseError::Unexpected {
                span: tok.span,
                expected: "`PrjSect`, `SenSect`, or `TgtSect`",
                found: Found::Token(TokenKind::UnknownKeyword(name.clone())),
            }),
            _ => Err(self.unexpected("section keyword")),
        }
    }

    fn parse_prj_sect(&mut self) -> Result<PrjSect, ParseError> {
        let start = self.bump();
        let span = start.span;
        debug_assert!(matches!(start.kind, TokenKind::Keyword(Keyword::PrjSect)));
        self.expect_lbrace()?;

        let mut items = Vec::new();
        while !matches!(self.peek(), Some(t) if matches!(t.kind, TokenKind::RBrace)) {
            if self.at_end() {
                return Err(self.unexpected("`}`"));
            }
            items.push(self.parse_prj_item()?);
        }
        self.expect_rbrace()?;
        Ok(PrjSect { span, items })
    }

    fn parse_prj_item(&mut self) -> Result<PrjItem, ParseError> {
        let tok = self.bump();
        match tok.kind {
            TokenKind::Keyword(
                kw @ (Keyword::Name
                | Keyword::Scenario
                | Keyword::Site
                | Keyword::Time
                | Keyword::LatLong
                | Keyword::Comment
                | Keyword::Keyword),
            ) => Ok(PrjItem::Field(self.parse_field(kw, tok.span))),
            TokenKind::UnknownKeyword(name) => {
                Ok(PrjItem::Unknown(self.parse_unknown_field(name, tok.span)))
            }
            other => Err(ParseError::Unexpected {
                span: tok.span,
                expected: "project field keyword",
                found: Found::Token(other),
            }),
        }
    }

    fn parse_sen_sect(&mut self) -> Result<SenSect, ParseError> {
        let start = self.bump();
        let span = start.span;
        debug_assert!(matches!(start.kind, TokenKind::Keyword(Keyword::SenSect)));
        self.expect_lbrace()?;

        let mut items = Vec::new();
        while !matches!(self.peek(), Some(t) if matches!(t.kind, TokenKind::RBrace)) {
            if self.at_end() {
                return Err(self.unexpected("`}`"));
            }
            items.push(self.parse_sen_sect_item()?);
        }
        self.expect_rbrace()?;
        Ok(SenSect { span, items })
    }

    fn parse_sen_sect_item(&mut self) -> Result<SenSectItem, ParseError> {
        if matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::Keyword(Keyword::SenUpd))
        ) {
            let span = self.bump().span;
            return Ok(SenSectItem::SenUpd(self.parse_sen_upd_body(span)?));
        }
        let tok = self.bump();
        match tok.kind {
            TokenKind::Keyword(kw @ (Keyword::Comment | Keyword::Name | Keyword::Fov)) => {
                Ok(SenSectItem::Field(self.parse_field(kw, tok.span)))
            }
            TokenKind::Keyword(Keyword::Keyword) => {
                Ok(SenSectItem::Field(self.parse_field(Keyword::Keyword, tok.span)))
            }
            TokenKind::UnknownKeyword(name) => {
                Ok(SenSectItem::Unknown(self.parse_unknown_field(name, tok.span)))
            }
            other => Err(ParseError::Unexpected {
                span: tok.span,
                expected: "sensor section field or `SenUpd`",
                found: Found::Token(other),
            }),
        }
    }

    fn parse_sen_upd_body(&mut self, span: Span) -> Result<SenUpd, ParseError> {
        self.expect_lbrace()?;
        let mut items = Vec::new();
        while !matches!(self.peek(), Some(t) if matches!(t.kind, TokenKind::RBrace)) {
            if self.at_end() {
                return Err(self.unexpected("`}`"));
            }
            items.push(self.parse_sen_upd_item()?);
        }
        self.expect_rbrace()?;
        Ok(SenUpd { span, items })
    }

    fn parse_sen_upd_item(&mut self) -> Result<SenUpdItem, ParseError> {
        let tok = self.bump();
        match tok.kind {
            TokenKind::Keyword(
                kw @ (Keyword::Comment
                | Keyword::Time
                | Keyword::Azimuth
                | Keyword::Elevation
                | Keyword::Roll
                | Keyword::LatLong
                | Keyword::Utm
                | Keyword::Range
                | Keyword::PixRange
                | Keyword::Keyword),
            ) => Ok(SenUpdItem::Field(self.parse_field(kw, tok.span))),
            TokenKind::UnknownKeyword(name) => {
                Ok(SenUpdItem::Unknown(self.parse_unknown_field(name, tok.span)))
            }
            other => Err(ParseError::Unexpected {
                span: tok.span,
                expected: "sensor update field keyword",
                found: Found::Token(other),
            }),
        }
    }

    fn parse_tgt_sect(&mut self) -> Result<TgtSect, ParseError> {
        let start = self.bump();
        let span = start.span;
        debug_assert!(matches!(start.kind, TokenKind::Keyword(Keyword::TgtSect)));
        self.expect_lbrace()?;

        let mut items = Vec::new();
        while !matches!(self.peek(), Some(t) if matches!(t.kind, TokenKind::RBrace)) {
            if self.at_end() {
                return Err(self.unexpected("`}`"));
            }
            items.push(self.parse_tgt_sect_item()?);
        }
        self.expect_rbrace()?;
        Ok(TgtSect { span, items })
    }

    fn parse_tgt_sect_item(&mut self) -> Result<TgtSectItem, ParseError> {
        if matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::Keyword(Keyword::TgtUpd))
        ) {
            let span = self.bump().span;
            return Ok(TgtSectItem::TgtUpd(self.parse_tgt_upd_body(span)?));
        }
        let tok = self.bump();
        match tok.kind {
            TokenKind::Keyword(kw @ (Keyword::Comment | Keyword::Keyword)) => {
                Ok(TgtSectItem::Field(self.parse_field(kw, tok.span)))
            }
            TokenKind::UnknownKeyword(name) => {
                Ok(TgtSectItem::Unknown(self.parse_unknown_field(name, tok.span)))
            }
            other => Err(ParseError::Unexpected {
                span: tok.span,
                expected: "`Comment` or `TgtUpd`",
                found: Found::Token(other),
            }),
        }
    }

    fn parse_tgt_upd_body(&mut self, span: Span) -> Result<TgtUpd, ParseError> {
        self.expect_lbrace()?;
        let mut items = Vec::new();
        while !matches!(self.peek(), Some(t) if matches!(t.kind, TokenKind::RBrace)) {
            if self.at_end() {
                return Err(self.unexpected("`}`"));
            }
            items.push(self.parse_tgt_upd_item()?);
        }
        self.expect_rbrace()?;
        Ok(TgtUpd { span, items })
    }

    fn parse_tgt_upd_item(&mut self) -> Result<TgtUpdItem, ParseError> {
        if matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::Keyword(Keyword::Tgt))
        ) {
            let span = self.bump().span;
            return Ok(TgtUpdItem::Tgt(self.parse_tgt_body(span)?));
        }
        let tok = self.bump();
        match tok.kind {
            TokenKind::Keyword(kw @ (Keyword::Comment | Keyword::Time | Keyword::Keyword)) => {
                Ok(TgtUpdItem::Field(self.parse_field(kw, tok.span)))
            }
            TokenKind::UnknownKeyword(name) => {
                Ok(TgtUpdItem::Unknown(self.parse_unknown_field(name, tok.span)))
            }
            other => Err(ParseError::Unexpected {
                span: tok.span,
                expected: "`Comment`, `Time`, or `Tgt`",
                found: Found::Token(other),
            }),
        }
    }

    fn parse_tgt_body(&mut self, span: Span) -> Result<Tgt, ParseError> {
        self.expect_lbrace()?;
        let mut items = Vec::new();
        while !matches!(self.peek(), Some(t) if matches!(t.kind, TokenKind::RBrace)) {
            if self.at_end() {
                return Err(self.unexpected("`}`"));
            }
            items.push(self.parse_tgt_item()?);
        }
        self.expect_rbrace()?;
        Ok(Tgt { span, items })
    }

    fn parse_tgt_item(&mut self) -> Result<TgtItem, ParseError> {
        if matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::Keyword(Keyword::TgtSenRel))
        ) {
            let span = self.bump().span;
            return Ok(TgtItem::TgtSenRel(self.parse_tgt_sen_rel_body(span)?));
        }
        if matches!(
            self.peek().map(|t| &t.kind),
            Some(TokenKind::Keyword(Keyword::TgtAbs))
        ) {
            let span = self.bump().span;
            return Ok(TgtItem::TgtAbs(self.parse_tgt_abs_body(span)?));
        }
        let tok = self.bump();
        match tok.kind {
            TokenKind::Keyword(
                kw @ (Keyword::Comment
                | Keyword::TgtType
                | Keyword::PlyId
                | Keyword::Aspect
                | Keyword::Range
                | Keyword::PixLoc
                | Keyword::Keyword),
            ) => Ok(TgtItem::Field(self.parse_field(kw, tok.span))),
            TokenKind::UnknownKeyword(name) => {
                Ok(TgtItem::Unknown(self.parse_unknown_field(name, tok.span)))
            }
            other => Err(ParseError::Unexpected {
                span: tok.span,
                expected: "target field or nested container",
                found: Found::Token(other),
            }),
        }
    }

    fn parse_tgt_sen_rel_body(&mut self, span: Span) -> Result<TgtSenRel, ParseError> {
        self.expect_lbrace()?;
        let mut items = Vec::new();
        while !matches!(self.peek(), Some(t) if matches!(t.kind, TokenKind::RBrace)) {
            if self.at_end() {
                return Err(self.unexpected("`}`"));
            }
            items.push(self.parse_tgt_sen_rel_item()?);
        }
        self.expect_rbrace()?;
        Ok(TgtSenRel { span, items })
    }

    fn parse_tgt_sen_rel_item(&mut self) -> Result<TgtSenRelItem, ParseError> {
        let tok = self.bump();
        match tok.kind {
            TokenKind::Keyword(
                kw @ (Keyword::Azimuth
                | Keyword::Elevation
                | Keyword::Pitch
                | Keyword::Roll
                | Keyword::Obscuration
                | Keyword::Comment
                | Keyword::Keyword),
            ) => Ok(TgtSenRelItem::Field(self.parse_field(kw, tok.span))),
            TokenKind::UnknownKeyword(name) => Ok(TgtSenRelItem::Unknown(self.parse_unknown_field(
                name,
                tok.span,
            ))),
            other => Err(ParseError::Unexpected {
                span: tok.span,
                expected: "TgtSenRel field keyword",
                found: Found::Token(other),
            }),
        }
    }

    fn parse_tgt_abs_body(&mut self, span: Span) -> Result<TgtAbs, ParseError> {
        self.expect_lbrace()?;
        let mut items = Vec::new();
        while !matches!(self.peek(), Some(t) if matches!(t.kind, TokenKind::RBrace)) {
            if self.at_end() {
                return Err(self.unexpected("`}`"));
            }
            items.push(self.parse_tgt_abs_item()?);
        }
        self.expect_rbrace()?;
        Ok(TgtAbs { span, items })
    }

    fn parse_tgt_abs_item(&mut self) -> Result<TgtAbsItem, ParseError> {
        let tok = self.bump();
        match tok.kind {
            TokenKind::Keyword(
                kw @ (Keyword::Utm
                | Keyword::LatLong
                | Keyword::Stake
                | Keyword::Azimuth
                | Keyword::Elevation
                | Keyword::Roll
                | Keyword::Comment
                | Keyword::Keyword),
            ) => Ok(TgtAbsItem::Field(self.parse_field(kw, tok.span))),
            TokenKind::UnknownKeyword(name) => {
                Ok(TgtAbsItem::Unknown(self.parse_unknown_field(name, tok.span)))
            }
            other => Err(ParseError::Unexpected {
                span: tok.span,
                expected: "TgtAbs field keyword",
                found: Found::Token(other),
            }),
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use crate::ast::{TgtSectItem, TgtUpdItem};

    #[test]
    fn minimal_tgt_nesting() {
        let src = r#"Agt {
  TgtSect {
    TgtUpd {
      Tgt {
        TgtType "M1A1"
        PixLoc 200 230
        Aspect 45.0
        Range 1500.0
      }
    }
  }
}"#;
        let doc = parse(src).expect("parse");
        assert_eq!(doc.root.items.len(), 1);
        let AgtItem::TgtSect(tgt_sect) = &doc.root.items[0] else {
            panic!("expected TgtSect");
        };
        let TgtSectItem::TgtUpd(upd) = &tgt_sect.items[0] else {
            panic!("expected TgtUpd");
        };
        let TgtUpdItem::Tgt(tgt) = &upd.items[0] else {
            panic!("expected Tgt");
        };
        assert_eq!(tgt.items.len(), 4);
    }

    #[test]
    fn tgt_directly_under_agt_is_error() {
        let src = "Agt { Tgt { } }";
        let err = parse(src).unwrap_err();
        assert!(matches!(err, ParseError::Unexpected { .. }));
        assert!(err.to_string().contains("Tgt"));
    }

    #[test]
    fn missing_rbrace() {
        let src = "Agt { PrjSect { Name \"x\"";
        let err = parse(src).unwrap_err();
        assert!(matches!(err, ParseError::Unexpected { .. }));
    }
}
