//! Core library for parsing imagery ground truth AGT containers.

pub mod keyword;
pub mod lex;
pub mod span;
pub mod token;

pub use keyword::Keyword;
pub use lex::{lex, lex_tokens, LexError, LexTokenIter};
pub use span::Span;
pub use token::{Token, TokenKind};

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
