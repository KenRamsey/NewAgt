//! Container AST for imagery AGT (PDF §6.2 placement rules).

use crate::keyword::Keyword;
use crate::span::Span;
use crate::token::Token;
use crate::value::FieldValue;

/// Parsed document: exactly one root `Agt` container.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub root: Agt,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Agt {
    pub span: Span,
    pub items: Vec<AgtItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgtItem {
    PrjSect(PrjSect),
    SenSect(SenSect),
    TgtSect(TgtSect),
    Unknown(UnknownStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrjSect {
    pub span: Span,
    pub items: Vec<PrjItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrjItem {
    Field(Field),
    Unknown(UnknownStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SenSect {
    pub span: Span,
    pub items: Vec<SenSectItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SenSectItem {
    Field(Field),
    SenUpd(SenUpd),
    Unknown(UnknownStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SenUpd {
    pub span: Span,
    pub items: Vec<SenUpdItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SenUpdItem {
    Field(Field),
    Unknown(UnknownStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TgtSect {
    pub span: Span,
    pub items: Vec<TgtSectItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TgtSectItem {
    Field(Field),
    TgtUpd(TgtUpd),
    Unknown(UnknownStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TgtUpd {
    pub span: Span,
    pub items: Vec<TgtUpdItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TgtUpdItem {
    Field(Field),
    Tgt(Tgt),
    Unknown(UnknownStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tgt {
    pub span: Span,
    pub items: Vec<TgtItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TgtItem {
    Field(Field),
    TgtSenRel(TgtSenRel),
    TgtAbs(TgtAbs),
    Unknown(UnknownStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TgtSenRel {
    pub span: Span,
    pub items: Vec<TgtSenRelItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TgtSenRelItem {
    Field(Field),
    Unknown(UnknownStatement),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TgtAbs {
    pub span: Span,
    pub items: Vec<TgtAbsItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TgtAbsItem {
    Field(Field),
    Unknown(UnknownStatement),
}

/// Scalar or composite field: keyword plus typed value (raw token fallback when needed).
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub keyword: Keyword,
    pub keyword_span: Span,
    pub value: FieldValue,
    /// Lexed value tokens (preserves source spans for export).
    pub tokens: Vec<Token>,
}

/// Unrecognized keyword and trailing value tokens until the next sibling keyword.
#[derive(Debug, Clone, PartialEq)]
pub struct UnknownStatement {
    pub name: String,
    pub keyword_span: Span,
    pub values: Vec<Token>,
}
