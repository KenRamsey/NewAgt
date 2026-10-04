//! Core library for parsing imagery ground truth AGT containers.

pub mod ast;
pub mod dump;
pub mod extension;
pub mod info;
pub mod json_export;
pub mod keyword;
pub mod lex;
pub mod parse;
pub mod profile;
pub mod span;
pub mod token;
pub mod validate;
pub mod value;

pub use ast::{Document, UnknownStatement};
pub use dump::dump_document;
pub use extension::{
    ExtensionRecord, ParseResult, ParseWarning, ParseWarningKind,
};
pub use info::{format_info, summarize_document, DocumentSummary};
pub use json_export::{
    export_document, export_parse_result, JsonExportOptions, JsonExportRoot, SCHEMA_V1,
};
pub use keyword::Keyword;
pub use lex::{lex, lex_tokens, LexError, LexTokenIter};
pub use parse::{parse, parse_with_options, parse_with_warnings, Found, ParseError, ParseOptions};
pub use profile::ParseProfile;
pub use span::Span;
pub use token::{Token, TokenKind};
pub use validate::{
    format_report_json, format_report_text, validate, ValidationCode, ValidationEntry,
    ValidationReport, ValidationSeverity,
};
pub use value::{
    FieldValue, Fov, LatLong, PixBox, PixLoc, PixRange, Time, Utm, ValueParseError,
};

/// Library version (matches `Cargo.toml`).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_non_empty() {
        assert!(!VERSION.is_empty());
    }
}
