//! Typed composite field values (PDF §2.1.2) and scalar coercions.

use crate::keyword::Keyword;
use crate::profile::ParseProfile;
use crate::span::Span;
use crate::token::{Token, TokenKind};

#[derive(Debug, Clone, PartialEq)]
pub struct Time {
    pub year: i64,
    pub julian_day: i64,
    pub hour: i64,
    pub min: i64,
    pub sec: i64,
    pub ms: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixLoc {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LatLong {
    pub lat_deg: i64,
    pub lat_min: i64,
    pub lat_sec: f64,
    pub lat_dir: String,
    pub long_deg: i64,
    pub long_min: i64,
    pub long_sec: f64,
    pub long_dir: String,
}

/// PDF-minimal UTM (3 numbers) or AGTJ-style 5-field with grid/datum strings.
#[derive(Debug, Clone, PartialEq)]
pub struct Utm {
    pub easting: i64,
    pub northing: i64,
    pub elevation: f64,
    /// Present when a 5-token AGTJ-style `Utm` includes grid/datum strings after elevation.
    pub grid: Option<String>,
    pub datum: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fov {
    pub horizontal: f64,
    pub vertical: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixRange {
    pub x: i64,
    pub y: i64,
    pub range_m: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PixBox {
    pub upper_left: PixLoc,
    pub lower_right: PixLoc,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Time(Time),
    PixLoc(PixLoc),
    LatLong(LatLong),
    Utm(Utm),
    Fov(Fov),
    PixRange(PixRange),
    PixBox(PixBox),
    /// Single quoted string (`Comment`, `Name`, …).
    String(String),
    Float(f64),
    Integer(i64),
    /// Value tokens not matching the expected composite or scalar shape.
    Raw(Vec<Token>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueParseError {
    pub span: Span,
    pub keyword: Keyword,
    pub message: String,
}

impl std::fmt::Display for ValueParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "parse error at line {}, column {}: {} `{}`: {}",
            self.span.line, self.span.column, self.keyword, self.keyword.as_str(), self.message
        )
    }
}

impl std::error::Error for ValueParseError {}

/// Attach typed `FieldValue` to a known keyword's value tokens.
pub fn parse_field_value(
    profile: ParseProfile,
    keyword: Keyword,
    tokens: Vec<Token>,
) -> Result<FieldValue, ValueParseError> {
    let span = value_span(&tokens).unwrap_or(Span::new(1, 1, 0));
    match keyword {
        Keyword::Time => parse_time(tokens, keyword, span),
        Keyword::PixLoc => parse_pix_loc(tokens, keyword, span),
        Keyword::LatLong => parse_lat_long(tokens, keyword, span),
        Keyword::Utm => parse_utm(profile, tokens, keyword, span),
        Keyword::Fov => parse_fov(tokens, keyword, span),
        Keyword::PixRange => parse_pix_range(tokens, keyword, span),
        Keyword::PixBox => parse_pix_box(tokens, keyword, span),
        Keyword::Comment
        | Keyword::Name
        | Keyword::Scenario
        | Keyword::Site
        | Keyword::TgtType
        | Keyword::PlyId
        | Keyword::Keyword => parse_one_string(tokens, keyword, span),
        Keyword::Aspect
        | Keyword::Azimuth
        | Keyword::Elevation
        | Keyword::Pitch
        | Keyword::Roll
        | Keyword::Range
        | Keyword::Obscuration => parse_one_float(tokens, keyword, span),
        Keyword::Stake => parse_one_integer(tokens, keyword, span),
        _ => Ok(FieldValue::Raw(tokens)),
    }
}

fn value_span(tokens: &[Token]) -> Option<Span> {
    let first = tokens.first()?;
    let last = tokens.last()?;
    Some(Span::merge(first.span, last.span))
}

fn incomplete(keyword: Keyword, span: Span, expected: &str, got: usize) -> ValueParseError {
    ValueParseError {
        span,
        keyword,
        message: format!("expected {expected}, got {got} value token(s)"),
    }
}

fn as_integer(tok: &Token) -> Option<i64> {
    match &tok.kind {
        TokenKind::Integer(n) => Some(*n),
        _ => None,
    }
}

fn as_float(tok: &Token) -> Option<f64> {
    match &tok.kind {
        TokenKind::Real(n) => Some(*n),
        TokenKind::Integer(n) => Some(*n as f64),
        _ => None,
    }
}

fn as_long(tok: &Token) -> Option<i64> {
    as_integer(tok)
}

fn as_string(tok: &Token) -> Option<String> {
    match &tok.kind {
        TokenKind::String(s) => Some(s.clone()),
        _ => None,
    }
}

fn parse_time(tokens: Vec<Token>, keyword: Keyword, span: Span) -> Result<FieldValue, ValueParseError> {
    const N: usize = 6;
    if tokens.len() != N {
        return Err(incomplete(keyword, span, "6 integers (Time)", tokens.len()));
    }
    let mut ints = tokens.iter().map(as_integer);
    Ok(FieldValue::Time(Time {
        year: ints.next().flatten().ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
        julian_day: ints.next().flatten().ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
        hour: ints.next().flatten().ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
        min: ints.next().flatten().ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
        sec: ints.next().flatten().ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
        ms: ints.next().flatten().ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
    }))
}

fn parse_pix_loc(tokens: Vec<Token>, keyword: Keyword, span: Span) -> Result<FieldValue, ValueParseError> {
    const N: usize = 2;
    if tokens.len() != N {
        return Err(incomplete(keyword, span, "2 integers (PixLoc)", tokens.len()));
    }
    Ok(FieldValue::PixLoc(PixLoc {
        x: as_integer(&tokens[0]).ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
        y: as_integer(&tokens[1]).ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
    }))
}

fn parse_lat_long(tokens: Vec<Token>, keyword: Keyword, span: Span) -> Result<FieldValue, ValueParseError> {
    const N: usize = 8;
    if tokens.len() != N {
        return Err(incomplete(
            keyword,
            span,
            "8 tokens (LatLong DMS + directions)",
            tokens.len(),
        ));
    }
    let lat_deg = as_integer(&tokens[0]);
    let lat_min = as_integer(&tokens[1]);
    let lat_sec = as_float(&tokens[2]);
    let lat_dir = as_string(&tokens[3]);
    let long_deg = as_integer(&tokens[4]);
    let long_min = as_integer(&tokens[5]);
    let long_sec = as_float(&tokens[6]);
    let long_dir = as_string(&tokens[7]);
    if let (Some(lat_deg), Some(lat_min), Some(lat_sec), Some(lat_dir), Some(long_deg), Some(long_min), Some(long_sec), Some(long_dir)) =
        (lat_deg, lat_min, lat_sec, lat_dir, long_deg, long_min, long_sec, long_dir)
    {
        return Ok(FieldValue::LatLong(LatLong {
            lat_deg,
            lat_min,
            lat_sec,
            lat_dir,
            long_deg,
            long_min,
            long_sec,
            long_dir,
        }));
    }
    Ok(FieldValue::Raw(tokens))
}

fn parse_utm(
    profile: ParseProfile,
    tokens: Vec<Token>,
    keyword: Keyword,
    span: Span,
) -> Result<FieldValue, ValueParseError> {
    // PDF: easting, northing, elevation. AGTJ may append grid + datum strings (5 fields total).
    match tokens.len() {
        3 => {
            let easting = as_long(&tokens[0]).ok_or_else(|| type_mismatch(keyword, span, "long"))?;
            let northing = as_long(&tokens[1]).ok_or_else(|| type_mismatch(keyword, span, "long"))?;
            let elevation = as_float(&tokens[2]).ok_or_else(|| type_mismatch(keyword, span, "float"))?;
            Ok(FieldValue::Utm(Utm {
                easting,
                northing,
                elevation,
                grid: None,
                datum: None,
            }))
        }
        5 if profile == ParseProfile::Agtj => {
            let easting = as_long(&tokens[0]).ok_or_else(|| type_mismatch(keyword, span, "long"))?;
            let northing = as_long(&tokens[1]).ok_or_else(|| type_mismatch(keyword, span, "long"))?;
            let elevation = as_float(&tokens[2]).ok_or_else(|| type_mismatch(keyword, span, "float"))?;
            let grid = as_string(&tokens[3]);
            let datum = as_string(&tokens[4]);
            if let (Some(grid), Some(datum)) = (grid, datum) {
                return Ok(FieldValue::Utm(Utm {
                    easting,
                    northing,
                    elevation,
                    grid: Some(grid),
                    datum: Some(datum),
                }));
            }
            Ok(FieldValue::Raw(tokens))
        }
        5 => Err(incomplete(
            keyword,
            span,
            "3 numbers (Utm) under profile pdf1999; use profile agtj for grid/datum strings",
            5,
        )),
        n => Err(incomplete(
            keyword,
            span,
            if profile == ParseProfile::Agtj {
                "3 numbers (Utm) or 5 with grid/datum strings (AGTJ)"
            } else {
                "3 numbers (Utm)"
            },
            n,
        )),
    }
}

fn parse_fov(tokens: Vec<Token>, keyword: Keyword, span: Span) -> Result<FieldValue, ValueParseError> {
    const N: usize = 2;
    if tokens.len() != N {
        return Err(incomplete(keyword, span, "2 floats (Fov)", tokens.len()));
    }
    Ok(FieldValue::Fov(Fov {
        horizontal: as_float(&tokens[0]).ok_or_else(|| type_mismatch(keyword, span, "float"))?,
        vertical: as_float(&tokens[1]).ok_or_else(|| type_mismatch(keyword, span, "float"))?,
    }))
}

fn parse_pix_range(tokens: Vec<Token>, keyword: Keyword, span: Span) -> Result<FieldValue, ValueParseError> {
    const N: usize = 3;
    if tokens.len() != N {
        return Err(incomplete(keyword, span, "2 integers + float (PixRange)", tokens.len()));
    }
    Ok(FieldValue::PixRange(PixRange {
        x: as_integer(&tokens[0]).ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
        y: as_integer(&tokens[1]).ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
        range_m: as_float(&tokens[2]).ok_or_else(|| type_mismatch(keyword, span, "float"))?,
    }))
}

fn parse_pix_box(tokens: Vec<Token>, keyword: Keyword, span: Span) -> Result<FieldValue, ValueParseError> {
    const N: usize = 4;
    if tokens.len() != N {
        return Err(incomplete(keyword, span, "4 integers (PixBox: two PixLoc)", tokens.len()));
    }
    Ok(FieldValue::PixBox(PixBox {
        upper_left: PixLoc {
            x: as_integer(&tokens[0]).ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
            y: as_integer(&tokens[1]).ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
        },
        lower_right: PixLoc {
            x: as_integer(&tokens[2]).ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
            y: as_integer(&tokens[3]).ok_or_else(|| type_mismatch(keyword, span, "integer"))?,
        },
    }))
}

fn parse_one_string(tokens: Vec<Token>, keyword: Keyword, span: Span) -> Result<FieldValue, ValueParseError> {
    if tokens.len() != 1 {
        return Err(incomplete(keyword, span, "1 string", tokens.len()));
    }
    if let Some(s) = as_string(&tokens[0]) {
        return Ok(FieldValue::String(s));
    }
    Ok(FieldValue::Raw(tokens))
}

fn parse_one_float(tokens: Vec<Token>, keyword: Keyword, span: Span) -> Result<FieldValue, ValueParseError> {
    if tokens.len() != 1 {
        return Err(incomplete(keyword, span, "1 float", tokens.len()));
    }
    if let Some(n) = as_float(&tokens[0]) {
        return Ok(FieldValue::Float(n));
    }
    Ok(FieldValue::Raw(tokens))
}

fn parse_one_integer(tokens: Vec<Token>, keyword: Keyword, span: Span) -> Result<FieldValue, ValueParseError> {
    if tokens.len() != 1 {
        return Err(incomplete(keyword, span, "1 integer", tokens.len()));
    }
    if let Some(n) = as_integer(&tokens[0]) {
        return Ok(FieldValue::Integer(n));
    }
    Ok(FieldValue::Raw(tokens))
}

fn type_mismatch(keyword: Keyword, span: Span, expected: &str) -> ValueParseError {
    ValueParseError {
        span,
        keyword,
        message: format!("expected {expected}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Token;

    fn int(n: i64) -> Token {
        Token::new(TokenKind::Integer(n), Span::new(1, 1, 0))
    }

    fn real(n: f64) -> Token {
        Token::new(TokenKind::Real(n), Span::new(1, 1, 0))
    }

    fn str(s: &str) -> Token {
        Token::new(TokenKind::String(s.to_string()), Span::new(1, 1, 0))
    }

    #[test]
    fn time_valid() {
        let v = parse_field_value(
            ParseProfile::Agtj,
            Keyword::Time,
            vec![int(1992), int(140), int(16), int(34), int(15), int(341)],
        )
        .unwrap();
        assert_eq!(
            v,
            FieldValue::Time(Time {
                year: 1992,
                julian_day: 140,
                hour: 16,
                min: 34,
                sec: 15,
                ms: 341,
            })
        );
    }

    #[test]
    fn time_incomplete() {
        let err = parse_field_value(ParseProfile::Agtj, Keyword::Time, vec![int(1992), int(140)]).unwrap_err();
        assert!(err.message.contains("6 integers"));
    }

    #[test]
    fn lat_long_valid() {
        let v = parse_field_value(
            ParseProfile::Agtj,
            Keyword::LatLong,
            vec![
                int(11),
                int(27),
                real(34.323),
                str("N"),
                int(47),
                int(57),
                real(58.021),
                str("E"),
            ],
        )
        .unwrap();
        assert!(matches!(v, FieldValue::LatLong(_)));
    }

    #[test]
    fn lat_long_incomplete_seven_tokens() {
        let err = parse_field_value(
            ParseProfile::Agtj,
            Keyword::LatLong,
            vec![int(1), int(2), real(3.0), str("N"), int(4), int(5), real(6.0)],
        )
        .unwrap_err();
        assert!(err.message.contains("8 tokens"));
    }

    #[test]
    fn utm_pdf_three_fields() {
        let v = parse_field_value(
            ParseProfile::Agtj,
            Keyword::Utm,
            vec![int(500_000), int(4_000_000), real(100.5)],
        )
        .unwrap();
        let FieldValue::Utm(u) = v else {
            panic!("expected Utm");
        };
        assert_eq!(u.easting, 500_000);
        assert!(u.grid.is_none());
    }

    #[test]
    fn utm_agtj_five_fields() {
        let v = parse_field_value(
            ParseProfile::Agtj,
            Keyword::Utm,
            vec![int(1), int(2), real(3.0), str("18N"), str("WGS84")],
        )
        .unwrap();
        let FieldValue::Utm(u) = v else {
            panic!("expected Utm");
        };
        assert_eq!(u.grid.as_deref(), Some("18N"));
        assert_eq!(u.datum.as_deref(), Some("WGS84"));
    }

    #[test]
    fn utm_incomplete_two_fields() {
        let err = parse_field_value(ParseProfile::Agtj, Keyword::Utm, vec![int(1), int(2)]).unwrap_err();
        assert!(err.message.contains("3 numbers"));
    }

    #[test]
    fn pix_loc_and_range_and_fov() {
        let pl = parse_field_value(ParseProfile::Agtj, Keyword::PixLoc, vec![int(200), int(230)]).unwrap();
        assert_eq!(
            pl,
            FieldValue::PixLoc(PixLoc { x: 200, y: 230 })
        );
        let pr =
            parse_field_value(ParseProfile::Agtj, Keyword::PixRange, vec![int(10), int(20), real(1500.0)]).unwrap();
        assert!(matches!(pr, FieldValue::PixRange(_)));
        let fov = parse_field_value(ParseProfile::Agtj, Keyword::Fov, vec![real(30.0), real(20.0)]).unwrap();
        assert!(matches!(fov, FieldValue::Fov(_)));
    }

    #[test]
    fn pix_range_incomplete() {
        let err = parse_field_value(ParseProfile::Agtj, Keyword::PixRange, vec![int(1), int(2)]).unwrap_err();
        assert!(err.message.contains("2 integers + float"));
    }

    #[test]
    fn pix_box_four_integers() {
        let v = parse_field_value(
            ParseProfile::Agtj,
            Keyword::PixBox,
            vec![int(0), int(0), int(100), int(100)],
        )
        .unwrap();
        assert!(matches!(v, FieldValue::PixBox(_)));
    }
}
