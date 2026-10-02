//! Core library for parsing Adventure Game Toolkit (AGT) game bundles.
//!
//! Milestone M0: crate scaffold only; DA1 and record readers come in later milestones.

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
