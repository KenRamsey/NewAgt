//! Core library for parsing imagery ground truth AGT containers.

pub mod ast;
pub mod keyword;
pub mod lex;
pub mod parse;
pub mod span;
pub mod token;
pub mod value;

pub use ast::Document;
pub use keyword::Keyword;
pub use lex::{lex, lex_tokens, LexError, LexTokenIter};
pub use parse::{parse, Found, ParseError};
pub use span::Span;
pub use token::{Token, TokenKind};
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
