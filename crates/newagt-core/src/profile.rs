//! Named parse profiles: PDF-strict vs AGTJ-aligned accommodations.

/// Which dialect rules apply when parsing and typing field values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParseProfile {
    /// AGTJ-aligned: `Fov` in `SenUpd`, `PixBox` on `Tgt`, 5-field `Utm` (default).
    #[default]
    Agtj,
    /// `Agt-1992.pdf` yacc placements and composite shapes only.
    Pdf1999,
}

impl ParseProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Agtj => "agtj",
            Self::Pdf1999 => "pdf1999",
        }
    }

    pub fn parse_name(s: &str) -> Option<Self> {
        match s {
            "agtj" => Some(Self::Agtj),
            "pdf1999" => Some(Self::Pdf1999),
            _ => None,
        }
    }
}

impl std::str::FromStr for ParseProfile {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_name(s).ok_or_else(|| format!("unknown parse profile `{s}` (expected `agtj` or `pdf1999`)"))
    }
}

impl std::fmt::Display for ParseProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
