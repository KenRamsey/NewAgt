//! PDF §6.1 keyword table plus corpus extension `Keyword`.

use std::fmt;

/// Recognized AGT keyword spellings (PDF yacc + extension).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    Agt,
    TgtSect,
    TgtUpd,
    Tgt,
    TgtAbs,
    TgtSenRel,
    SenSect,
    SenUpd,
    PrjSect,
    Comment,
    PixRange,
    Aspect,
    Azimuth,
    Elevation,
    Fov,
    LatLong,
    Name,
    Obscuration,
    PlyId,
    Pitch,
    PixLoc,
    PixBox,
    Range,
    Roll,
    Scenario,
    Site,
    Stake,
    TgtType,
    Time,
    Utm,
    /// Extension token used in AGTJ / training corpus (not in PDF yacc list).
    Keyword,
}

impl Keyword {
    pub fn from_ident(text: &str) -> Option<Self> {
        Some(match text {
            "Agt" => Self::Agt,
            "TgtSect" => Self::TgtSect,
            "TgtUpd" => Self::TgtUpd,
            "Tgt" => Self::Tgt,
            "TgtAbs" => Self::TgtAbs,
            "TgtSenRel" => Self::TgtSenRel,
            "SenSect" => Self::SenSect,
            "SenUpd" => Self::SenUpd,
            "PrjSect" => Self::PrjSect,
            "Comment" => Self::Comment,
            "PixRange" => Self::PixRange,
            "Aspect" => Self::Aspect,
            "Azimuth" => Self::Azimuth,
            "Elevation" => Self::Elevation,
            "Fov" => Self::Fov,
            "LatLong" => Self::LatLong,
            "Name" => Self::Name,
            "Obscuration" => Self::Obscuration,
            "PlyId" => Self::PlyId,
            "Pitch" => Self::Pitch,
            "PixLoc" => Self::PixLoc,
            "PixBox" => Self::PixBox,
            "Range" => Self::Range,
            "Roll" => Self::Roll,
            "Scenario" => Self::Scenario,
            "Site" => Self::Site,
            "Stake" => Self::Stake,
            "TgtType" => Self::TgtType,
            "Time" => Self::Time,
            "Utm" => Self::Utm,
            "Keyword" => Self::Keyword,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Agt => "Agt",
            Self::TgtSect => "TgtSect",
            Self::TgtUpd => "TgtUpd",
            Self::Tgt => "Tgt",
            Self::TgtAbs => "TgtAbs",
            Self::TgtSenRel => "TgtSenRel",
            Self::SenSect => "SenSect",
            Self::SenUpd => "SenUpd",
            Self::PrjSect => "PrjSect",
            Self::Comment => "Comment",
            Self::PixRange => "PixRange",
            Self::Aspect => "Aspect",
            Self::Azimuth => "Azimuth",
            Self::Elevation => "Elevation",
            Self::Fov => "Fov",
            Self::LatLong => "LatLong",
            Self::Name => "Name",
            Self::Obscuration => "Obscuration",
            Self::PlyId => "PlyId",
            Self::Pitch => "Pitch",
            Self::PixLoc => "PixLoc",
            Self::PixBox => "PixBox",
            Self::Range => "Range",
            Self::Roll => "Roll",
            Self::Scenario => "Scenario",
            Self::Site => "Site",
            Self::Stake => "Stake",
            Self::TgtType => "TgtType",
            Self::Time => "Time",
            Self::Utm => "Utm",
            Self::Keyword => "Keyword",
        }
    }
}

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
