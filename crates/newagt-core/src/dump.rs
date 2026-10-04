//! Human-readable indented tree dump of a parsed AGT document.

use crate::ast::{
    Agt, AgtItem, Document, Field, PrjItem, PrjSect, SenSect, SenSectItem, SenUpd, SenUpdItem,
    Tgt, TgtAbs, TgtAbsItem, TgtItem, TgtSenRel, TgtSenRelItem, TgtSect, TgtSectItem, TgtUpd,
    TgtUpdItem, UnknownStatement,
};
use crate::token::{Token, TokenKind};
use crate::value::FieldValue;

/// Render `doc` as an indented tree (containers, fields, unknown statements).
pub fn dump_document(doc: &Document) -> String {
    let mut out = String::new();
    dump_agt(&doc.root, 0, &mut out);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn indent_level(depth: usize) -> String {
    "  ".repeat(depth)
}

fn dump_agt(agt: &Agt, depth: usize, out: &mut String) {
    let pad = indent_level(depth);
    out.push_str(&pad);
    out.push_str("Agt {\n");
    for item in &agt.items {
        match item {
            AgtItem::PrjSect(s) => dump_prj_sect(s, depth + 1, out),
            AgtItem::SenSect(s) => dump_sen_sect(s, depth + 1, out),
            AgtItem::TgtSect(s) => dump_tgt_sect(s, depth + 1, out),
            AgtItem::Unknown(u) => dump_unknown(u, depth + 1, out),
        }
    }
    out.push_str(&pad);
    out.push_str("}\n");
}

fn dump_prj_sect(sect: &PrjSect, depth: usize, out: &mut String) {
    dump_container("PrjSect", depth, out, |d, o| {
        for item in &sect.items {
            match item {
                PrjItem::Field(f) => dump_field(f, d, o),
                PrjItem::Unknown(u) => dump_unknown(u, d, o),
            }
        }
    });
}

fn dump_sen_sect(sect: &SenSect, depth: usize, out: &mut String) {
    dump_container("SenSect", depth, out, |d, o| {
        for item in &sect.items {
            match item {
                SenSectItem::Field(f) => dump_field(f, d, o),
                SenSectItem::SenUpd(u) => dump_sen_upd(u, d, o),
                SenSectItem::Unknown(u) => dump_unknown(u, d, o),
            }
        }
    });
}

fn dump_sen_upd(upd: &SenUpd, depth: usize, out: &mut String) {
    dump_container("SenUpd", depth, out, |d, o| {
        for item in &upd.items {
            match item {
                SenUpdItem::Field(f) => dump_field(f, d, o),
                SenUpdItem::Unknown(u) => dump_unknown(u, d, o),
            }
        }
    });
}

fn dump_tgt_sect(sect: &TgtSect, depth: usize, out: &mut String) {
    dump_container("TgtSect", depth, out, |d, o| {
        for item in &sect.items {
            match item {
                TgtSectItem::Field(f) => dump_field(f, d, o),
                TgtSectItem::TgtUpd(u) => dump_tgt_upd(u, d, o),
                TgtSectItem::Unknown(u) => dump_unknown(u, d, o),
            }
        }
    });
}

fn dump_tgt_upd(upd: &TgtUpd, depth: usize, out: &mut String) {
    dump_container("TgtUpd", depth, out, |d, o| {
        for item in &upd.items {
            match item {
                TgtUpdItem::Field(f) => dump_field(f, d, o),
                TgtUpdItem::Tgt(t) => dump_tgt(t, d, o),
                TgtUpdItem::Unknown(u) => dump_unknown(u, d, o),
            }
        }
    });
}

fn dump_tgt(tgt: &Tgt, depth: usize, out: &mut String) {
    dump_container("Tgt", depth, out, |d, o| {
        for item in &tgt.items {
            match item {
                TgtItem::Field(f) => dump_field(f, d, o),
                TgtItem::TgtSenRel(r) => dump_tgt_sen_rel(r, d, o),
                TgtItem::TgtAbs(a) => dump_tgt_abs(a, d, o),
                TgtItem::Unknown(u) => dump_unknown(u, d, o),
            }
        }
    });
}

fn dump_tgt_sen_rel(rel: &TgtSenRel, depth: usize, out: &mut String) {
    dump_container("TgtSenRel", depth, out, |d, o| {
        for item in &rel.items {
            match item {
                TgtSenRelItem::Field(f) => dump_field(f, d, o),
                TgtSenRelItem::Unknown(u) => dump_unknown(u, d, o),
            }
        }
    });
}

fn dump_tgt_abs(abs: &TgtAbs, depth: usize, out: &mut String) {
    dump_container("TgtAbs", depth, out, |d, o| {
        for item in &abs.items {
            match item {
                TgtAbsItem::Field(f) => dump_field(f, d, o),
                TgtAbsItem::Unknown(u) => dump_unknown(u, d, o),
            }
        }
    });
}

fn dump_container(
    name: &str,
    depth: usize,
    out: &mut String,
    body: impl FnOnce(usize, &mut String),
) {
    let pad = indent_level(depth);
    out.push_str(&pad);
    out.push_str(name);
    out.push_str(" {\n");
    body(depth + 1, out);
    out.push_str(&pad);
    out.push_str("}\n");
}

fn dump_field(field: &Field, depth: usize, out: &mut String) {
    let pad = indent_level(depth);
    out.push_str(&pad);
    out.push_str(field.keyword.as_str());
    let suffix = format_field_value(&field.value);
    if !suffix.is_empty() {
        out.push(' ');
        out.push_str(&suffix);
    }
    out.push('\n');
}

fn dump_unknown(stmt: &UnknownStatement, depth: usize, out: &mut String) {
    let pad = indent_level(depth);
    out.push_str(&pad);
    out.push_str("UnknownStatement ");
    out.push_str(&stmt.name);
    let vals = format_tokens(&stmt.values);
    if !vals.is_empty() {
        out.push(' ');
        out.push_str(&vals);
    }
    out.push('\n');
}

fn format_field_value(value: &FieldValue) -> String {
    match value {
        FieldValue::String(s) => format_quoted(s),
        FieldValue::Integer(n) => n.to_string(),
        FieldValue::Float(f) => format_float(*f),
        FieldValue::Time(t) => format!(
            "{} {} {} {} {} {}",
            t.year, t.julian_day, t.hour, t.min, t.sec, t.ms
        ),
        FieldValue::PixLoc(p) => format!("{} {}", p.x, p.y),
        FieldValue::LatLong(l) => format!(
            "{} {} {} {} {} {} {} {}",
            l.lat_deg,
            l.lat_min,
            format_float(l.lat_sec),
            format_quoted(&l.lat_dir),
            l.long_deg,
            l.long_min,
            format_float(l.long_sec),
            format_quoted(&l.long_dir)
        ),
        FieldValue::Utm(u) => {
            let mut parts = vec![
                u.easting.to_string(),
                u.northing.to_string(),
                format_float(u.elevation),
            ];
            if let Some(g) = &u.grid {
                parts.push(format_quoted(g));
            }
            if let Some(d) = &u.datum {
                parts.push(format_quoted(d));
            }
            parts.join(" ")
        }
        FieldValue::Fov(f) => format!("{} {}", format_float(f.horizontal), format_float(f.vertical)),
        FieldValue::PixRange(p) => format!("{} {} {}", p.x, p.y, format_float(p.range_m)),
        FieldValue::PixBox(b) => format!(
            "{} {} {} {}",
            b.upper_left.x, b.upper_left.y, b.lower_right.x, b.lower_right.y
        ),
        FieldValue::Raw(raw) => format_tokens(raw),
    }
}

fn format_tokens(tokens: &[Token]) -> String {
    tokens
        .iter()
        .map(format_token)
        .collect::<Vec<_>>()
        .join(" ")
}

fn format_token(tok: &Token) -> String {
    match &tok.kind {
        TokenKind::LBrace => "{".to_string(),
        TokenKind::RBrace => "}".to_string(),
        TokenKind::Integer(n) => n.to_string(),
        TokenKind::Real(f) => format_float(*f),
        TokenKind::String(s) => format_quoted(s),
        TokenKind::Keyword(k) => k.as_str().to_string(),
        TokenKind::UnknownKeyword(s) => s.clone(),
    }
}

fn format_quoted(s: &str) -> String {
    let mut out = String::from('"');
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn format_float(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < 1e15 {
        format!("{f:.1}")
    } else {
        format!("{f}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    fn fixture(name: &str) -> String {
        let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
        std::fs::read_to_string(path).expect("read fixture")
    }

    #[test]
    fn dump_minimal_tgt_contains_fields() {
        let doc = parse(&fixture("minimal_agt_tgt.agt")).expect("parse");
        let text = dump_document(&doc);
        assert!(text.contains("Agt {"));
        assert!(text.contains("TgtSect {"));
        assert!(text.contains("TgtUpd {"));
        assert!(text.contains("Tgt {"));
        assert!(text.contains("TgtType \"M1A1\""));
        assert!(text.contains("PixLoc 200 230"));
        assert!(text.contains("Aspect 45.0"));
        assert!(text.contains("Range 1500.0"));
    }

    #[test]
    fn dump_comment_keyword_verbatim() {
        let doc = parse(&fixture("prototype_comment_keyword_burst.agt")).expect("parse");
        let text = dump_document(&doc);
        assert!(text.contains("Comment \"This is demo PrjSect Comment 1\""));
        assert!(text.contains("Keyword \"This is demo PrjSect Keyword 1\""));
    }

    #[test]
    fn dump_unknown_statement() {
        let src = r#"Agt { PrjSect { CustomTrainerTag "corpus-sidecar" 99 } }"#;
        let doc = parse(src).expect("parse");
        let text = dump_document(&doc);
        assert!(text.contains("UnknownStatement CustomTrainerTag"));
        assert!(text.contains("\"corpus-sidecar\""));
        assert!(text.contains("99"));
    }
}
