//! Token model for the AGT lexer.

use crate::keyword::Keyword;
use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    LBrace,
    RBrace,
    Integer(i64),
    Real(f64),
    String(String),
    Keyword(Keyword),
    UnknownKeyword(String),
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}
