//! Wild-format extensions: warnings, future corpus hooks (M4).

use crate::ast::Document;
use crate::span::Span;

/// Typed corpus-sidecar patterns (stub until corpus inventory lands).
///
/// Future variants may carry decoded COMMENT/KEYWORD payloads, frame hints,
/// or trainer column mappings without changing the container AST shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionRecord {
    /// Placeholder — no typed extensions decoded yet.
    Uninterpreted,
}

/// Non-fatal parse finding (spec mismatch, unknown keyword, odd placement).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseWarningKind {
    UnknownKeyword,
    OddPlacement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseWarning {
    pub kind: ParseWarningKind,
    pub span: Span,
    pub message: String,
}

/// Successful load with collected warnings and optional extension records.
#[derive(Debug, Clone, PartialEq)]
pub struct ParseResult {
    pub document: Document,
    pub warnings: Vec<ParseWarning>,
    pub extensions: Vec<ExtensionRecord>,
}
