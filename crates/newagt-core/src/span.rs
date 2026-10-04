//! Source locations for tokens (1-based line and column).

/// Byte offset and human-readable position in the source file.
///
/// `line` and `column` are **1-based** (first character is line 1, column 1).
/// `offset` is a **0-based** index into the original UTF-8 source string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

impl Span {
    pub const fn new(line: u32, column: u32, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }

    /// Span covering `start` through `end` (uses `start` for line/column, `end` for extent).
    pub fn merge(start: Self, _end: Self) -> Self {
        Self {
            line: start.line,
            column: start.column,
            offset: start.offset,
        }
    }
}
