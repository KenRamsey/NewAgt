//! JSON export with schema `newagt.schema.v1`.

use serde::Serialize;

use crate::ast::{
    Agt, AgtItem, Document, Field, PrjItem, PrjSect, SenSect, SenSectItem, SenUpd, SenUpdItem,
    Tgt, TgtAbs, TgtAbsItem, TgtItem, TgtSenRel, TgtSenRelItem, TgtSect, TgtSectItem, TgtUpd,
    TgtUpdItem, UnknownStatement,
};
use crate::extension::{ExtensionRecord, ParseResult};
use crate::profile::ParseProfile;
use crate::span::Span;
use crate::token::{Token, TokenKind};
use crate::value::FieldValue;

/// Stable export schema identifier.
pub const SCHEMA_V1: &str = "newagt.schema.v1";

#[derive(Debug, Clone, Copy, Default)]
pub struct JsonExportOptions {
    pub include_spans: bool,
}

#[derive(Debug, Serialize)]
pub struct JsonExportRoot {
    pub schema: &'static str,
    pub profile: String,
    pub document: JsonNode,
    pub extensions: Vec<JsonExtension>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum JsonNode {
    Container {
        kind: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<JsonSpan>,
        items: Vec<JsonNode>,
    },
    Field {
        keyword: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<JsonSpan>,
        value: JsonFieldValue,
    },
    Unknown {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        span: Option<JsonSpan>,
        tokens: Vec<JsonToken>,
    },
}

#[derive(Debug, Serialize)]
pub struct JsonSpan {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

#[derive(Debug, Serialize)]
pub struct JsonToken {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span: Option<JsonSpan>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JsonFieldValue {
    String { value: String },
    Integer { value: i64 },
    Float { value: f64 },
    Time {
        year: i64,
        julian_day: i64,
        hour: i64,
        min: i64,
        sec: i64,
        ms: i64,
    },
    PixLoc { x: i64, y: i64 },
    LatLong {
        lat_deg: i64,
        lat_min: i64,
        lat_sec: f64,
        lat_dir: String,
        long_deg: i64,
        long_min: i64,
        long_sec: f64,
        long_dir: String,
    },
    Utm {
        easting: i64,
        northing: i64,
        elevation: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        grid: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        datum: Option<String>,
    },
    Fov {
        horizontal: f64,
        vertical: f64,
    },
    PixRange {
        x: i64,
        y: i64,
        range_m: f64,
    },
    PixBox {
        upper_left: JsonPixLoc,
        lower_right: JsonPixLoc,
    },
    Raw { tokens: Vec<JsonToken> },
}

#[derive(Debug, Serialize)]
pub struct JsonPixLoc {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JsonExtension {
    Uninterpreted,
}

/// Build the export tree from a parsed document.
pub fn export_document(
    doc: &Document,
    profile: ParseProfile,
    extensions: &[ExtensionRecord],
    options: JsonExportOptions,
) -> JsonExportRoot {
    JsonExportRoot {
        schema: SCHEMA_V1,
        profile: profile.as_str().to_string(),
        document: export_agt(&doc.root, options),
        extensions: extensions.iter().map(export_extension).collect(),
    }
}

/// Serialize a successful parse result to pretty JSON.
pub fn export_parse_result(result: &ParseResult, profile: ParseProfile, options: JsonExportOptions) -> String {
    let root = export_document(&result.document, profile, &result.extensions, options);
    serde_json::to_string_pretty(&root).expect("json serialize")
}

fn export_extension(ext: &ExtensionRecord) -> JsonExtension {
    match ext {
        ExtensionRecord::Uninterpreted => JsonExtension::Uninterpreted,
    }
}

fn maybe_span(span: Span, options: JsonExportOptions) -> Option<JsonSpan> {
    if options.include_spans {
        Some(JsonSpan {
            line: span.line,
            column: span.column,
            offset: span.offset,
        })
    } else {
        None
    }
}

fn export_agt(agt: &Agt, options: JsonExportOptions) -> JsonNode {
    JsonNode::Container {
        kind: "Agt".to_string(),
        span: maybe_span(agt.span, options),
        items: agt
            .items
            .iter()
            .map(|i| match i {
                AgtItem::PrjSect(s) => export_prj_sect(s, options),
                AgtItem::SenSect(s) => export_sen_sect(s, options),
                AgtItem::TgtSect(s) => export_tgt_sect(s, options),
                AgtItem::Unknown(u) => export_unknown(u, options),
            })
            .collect(),
    }
}

fn export_prj_sect(sect: &PrjSect, options: JsonExportOptions) -> JsonNode {
    container("PrjSect", sect.span, options, sect.items.iter().map(|i| match i {
        PrjItem::Field(f) => export_field(f, options),
        PrjItem::Unknown(u) => export_unknown(u, options),
    }))
}

fn export_sen_sect(sect: &SenSect, options: JsonExportOptions) -> JsonNode {
    container("SenSect", sect.span, options, sect.items.iter().flat_map(|i| match i {
        SenSectItem::Field(f) => vec![export_field(f, options)],
        SenSectItem::SenUpd(u) => vec![export_sen_upd(u, options)],
        SenSectItem::Unknown(u) => vec![export_unknown(u, options)],
    }))
}

fn export_sen_upd(upd: &SenUpd, options: JsonExportOptions) -> JsonNode {
    container("SenUpd", upd.span, options, upd.items.iter().map(|i| match i {
        SenUpdItem::Field(f) => export_field(f, options),
        SenUpdItem::Unknown(u) => export_unknown(u, options),
    }))
}

fn export_tgt_sect(sect: &TgtSect, options: JsonExportOptions) -> JsonNode {
    container("TgtSect", sect.span, options, sect.items.iter().flat_map(|i| match i {
        TgtSectItem::Field(f) => vec![export_field(f, options)],
        TgtSectItem::TgtUpd(u) => vec![export_tgt_upd(u, options)],
        TgtSectItem::Unknown(u) => vec![export_unknown(u, options)],
    }))
}

fn export_tgt_upd(upd: &TgtUpd, options: JsonExportOptions) -> JsonNode {
    container("TgtUpd", upd.span, options, upd.items.iter().flat_map(|i| match i {
        TgtUpdItem::Field(f) => vec![export_field(f, options)],
        TgtUpdItem::Tgt(t) => vec![export_tgt(t, options)],
        TgtUpdItem::Unknown(u) => vec![export_unknown(u, options)],
    }))
}

fn export_tgt(tgt: &Tgt, options: JsonExportOptions) -> JsonNode {
    container("Tgt", tgt.span, options, tgt.items.iter().flat_map(|i| match i {
        TgtItem::Field(f) => vec![export_field(f, options)],
        TgtItem::TgtSenRel(r) => vec![export_tgt_sen_rel(r, options)],
        TgtItem::TgtAbs(a) => vec![export_tgt_abs(a, options)],
        TgtItem::Unknown(u) => vec![export_unknown(u, options)],
    }))
}

fn export_tgt_sen_rel(rel: &TgtSenRel, options: JsonExportOptions) -> JsonNode {
    container("TgtSenRel", rel.span, options, rel.items.iter().map(|i| match i {
        TgtSenRelItem::Field(f) => export_field(f, options),
        TgtSenRelItem::Unknown(u) => export_unknown(u, options),
    }))
}

fn export_tgt_abs(abs: &TgtAbs, options: JsonExportOptions) -> JsonNode {
    container("TgtAbs", abs.span, options, abs.items.iter().map(|i| match i {
        TgtAbsItem::Field(f) => export_field(f, options),
        TgtAbsItem::Unknown(u) => export_unknown(u, options),
    }))
}

fn container(
    kind: &str,
    span: Span,
    options: JsonExportOptions,
    items: impl Iterator<Item = JsonNode>,
) -> JsonNode {
    JsonNode::Container {
        kind: kind.to_string(),
        span: maybe_span(span, options),
        items: items.collect(),
    }
}

fn export_field(field: &Field, options: JsonExportOptions) -> JsonNode {
    JsonNode::Field {
        keyword: field.keyword.as_str().to_string(),
        span: maybe_span(field.keyword_span, options),
        value: export_field_value(&field.value, options),
    }
}

fn export_unknown(stmt: &UnknownStatement, options: JsonExportOptions) -> JsonNode {
    JsonNode::Unknown {
        name: stmt.name.clone(),
        span: maybe_span(stmt.keyword_span, options),
        tokens: stmt
            .values
            .iter()
            .map(|t| export_token(t, options))
            .collect(),
    }
}

fn export_field_value(value: &FieldValue, options: JsonExportOptions) -> JsonFieldValue {
    match value {
        FieldValue::String(s) => JsonFieldValue::String { value: s.clone() },
        FieldValue::Integer(n) => JsonFieldValue::Integer { value: *n },
        FieldValue::Float(f) => JsonFieldValue::Float { value: *f },
        FieldValue::Time(t) => JsonFieldValue::Time {
            year: t.year,
            julian_day: t.julian_day,
            hour: t.hour,
            min: t.min,
            sec: t.sec,
            ms: t.ms,
        },
        FieldValue::PixLoc(p) => JsonFieldValue::PixLoc { x: p.x, y: p.y },
        FieldValue::LatLong(l) => JsonFieldValue::LatLong {
            lat_deg: l.lat_deg,
            lat_min: l.lat_min,
            lat_sec: l.lat_sec,
            lat_dir: l.lat_dir.clone(),
            long_deg: l.long_deg,
            long_min: l.long_min,
            long_sec: l.long_sec,
            long_dir: l.long_dir.clone(),
        },
        FieldValue::Utm(u) => JsonFieldValue::Utm {
            easting: u.easting,
            northing: u.northing,
            elevation: u.elevation,
            grid: u.grid.clone(),
            datum: u.datum.clone(),
        },
        FieldValue::Fov(f) => JsonFieldValue::Fov {
            horizontal: f.horizontal,
            vertical: f.vertical,
        },
        FieldValue::PixRange(p) => JsonFieldValue::PixRange {
            x: p.x,
            y: p.y,
            range_m: p.range_m,
        },
        FieldValue::PixBox(b) => JsonFieldValue::PixBox {
            upper_left: JsonPixLoc {
                x: b.upper_left.x,
                y: b.upper_left.y,
            },
            lower_right: JsonPixLoc {
                x: b.lower_right.x,
                y: b.lower_right.y,
            },
        },
        FieldValue::Raw(tokens) => JsonFieldValue::Raw {
            tokens: tokens.iter().map(|t| export_token(t, options)).collect(),
        },
    }
}

fn export_token(tok: &Token, options: JsonExportOptions) -> JsonToken {
    let (kind, text, number) = match &tok.kind {
        TokenKind::LBrace => ("lbrace".to_string(), None, None),
        TokenKind::RBrace => ("rbrace".to_string(), None, None),
        TokenKind::Integer(n) => ("integer".to_string(), None, Some(*n as f64)),
        TokenKind::Real(f) => ("real".to_string(), None, Some(*f)),
        TokenKind::String(s) => ("string".to_string(), Some(s.clone()), None),
        TokenKind::Keyword(k) => ("keyword".to_string(), Some(k.as_str().to_string()), None),
        TokenKind::UnknownKeyword(s) => ("unknown_keyword".to_string(), Some(s.clone()), None),
    };
    JsonToken {
        kind,
        text,
        number,
        span: maybe_span(tok.span, options),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::{parse_with_options, ParseOptions};
    use serde_json::Value;

    fn fixture(name: &str) -> String {
        let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(path).expect("read fixture")
    }

    #[test]
    fn export_has_schema_version() {
        let result = parse_with_options(&fixture("minimal_agt_tgt.agt"), ParseOptions::default())
            .expect("parse");
        let json = export_parse_result(&result, ParseProfile::Agtj, JsonExportOptions::default());
        let v: Value = serde_json::from_str(&json).expect("parse json");
        assert_eq!(v["schema"], SCHEMA_V1);
        assert_eq!(v["profile"], "agtj");
    }

    #[test]
    fn export_round_trip_structure() {
        let result = parse_with_options(&fixture("minimal_agt_tgt.agt"), ParseOptions::default())
            .expect("parse");
        let json = export_parse_result(&result, ParseProfile::Agtj, JsonExportOptions::default());
        let v: Value = serde_json::from_str(&json).expect("parse json");
        assert_eq!(v["document"]["type"], "container");
        assert_eq!(v["document"]["kind"], "Agt");
        let items = v["document"]["items"].as_array().expect("items");
        assert_eq!(items[0]["kind"], "TgtSect");
    }

    #[test]
    fn export_spans_when_requested() {
        let result = parse_with_options(&fixture("minimal_agt_tgt.agt"), ParseOptions::default())
            .expect("parse");
        let json = export_parse_result(
            &result,
            ParseProfile::Agtj,
            JsonExportOptions {
                include_spans: true,
            },
        );
        let v: Value = serde_json::from_str(&json).expect("parse json");
        assert!(v["document"]["span"]["line"].is_number());
    }

    #[test]
    fn export_omits_spans_by_default() {
        let result = parse_with_options(&fixture("minimal_agt_tgt.agt"), ParseOptions::default())
            .expect("parse");
        let json = export_parse_result(&result, ParseProfile::Agtj, JsonExportOptions::default());
        let v: Value = serde_json::from_str(&json).expect("parse json");
        assert!(v["document"].get("span").is_none());
    }
}
