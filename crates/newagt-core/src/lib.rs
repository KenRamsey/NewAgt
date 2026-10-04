//! Core library for parsing imagery ground truth AGT containers.
//!
//! Milestone M0: crate scaffold only; format readers come after `Agt-1992.pdf` alignment.

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
