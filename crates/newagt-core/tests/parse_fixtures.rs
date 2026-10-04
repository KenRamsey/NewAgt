use newagt_core::ast::{AgtItem, Field, PrjItem, TgtItem, TgtSectItem, TgtUpdItem};
use newagt_core::keyword::Keyword;
use newagt_core::{parse, ParseError, TokenKind};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).expect("read fixture")
}

#[test]
fn parse_minimal_agt_tgt_chain() {
    let doc = parse(&fixture("minimal_agt_tgt.agt")).expect("parse minimal");
    assert_eq!(doc.root.items.len(), 1);
    let AgtItem::TgtSect(tgt_sect) = &doc.root.items[0] else {
        panic!("expected TgtSect");
    };
    assert_eq!(tgt_sect.items.len(), 1);
    let TgtSectItem::TgtUpd(upd) = &tgt_sect.items[0] else {
        panic!("expected TgtUpd");
    };
    let TgtUpdItem::Tgt(tgt) = &upd.items[0] else {
        panic!("expected Tgt");
    };
    assert_eq!(tgt.items.len(), 4);
    let TgtItem::Field(Field {
        keyword: Keyword::TgtType,
        values,
        ..
    }) = &tgt.items[0]
    else {
        panic!("expected TgtType field");
    };
    assert!(matches!(&values[0].kind, TokenKind::String(s) if s == "M1A1"));
}

#[test]
fn parse_example3_prj_in_agt() {
    let doc = parse(&fixture("example3_in_agt.agt")).expect("parse example3");
    let AgtItem::PrjSect(prj) = &doc.root.items[0] else {
        panic!("expected PrjSect");
    };
    assert_eq!(prj.items.len(), 7);
    let PrjItem::Field(Field {
        keyword: Keyword::Name,
        values,
        ..
    }) = &prj.items[0]
    else {
        panic!("Name first");
    };
    assert!(matches!(&values[0].kind, TokenKind::String(s) if s == "vision7"));

    let time_field = prj
        .items
        .iter()
        .find_map(|item| match item {
            PrjItem::Field(f) if f.keyword == Keyword::Time => Some(f),
            _ => None,
        })
        .expect("Time field");
    assert_eq!(time_field.values.len(), 6);
    assert!(matches!(time_field.values[0].kind, TokenKind::Integer(1992)));

    let comments: Vec<_> = prj
        .items
        .iter()
        .filter(|item| matches!(item, PrjItem::Field(f) if f.keyword == Keyword::Comment))
        .collect();
    assert_eq!(comments.len(), 2);
}

#[test]
fn parse_error_tgt_under_agt() {
    let err = parse("Agt { Tgt { } }").unwrap_err();
    assert!(matches!(err, ParseError::Unexpected { .. }));
    let msg = err.to_string();
    assert!(msg.contains("PrjSect") || msg.contains("SenSect") || msg.contains("TgtSect"));
}

#[test]
fn parse_error_missing_closing_brace() {
    let err = parse("Agt { PrjSect { Name \"x\"").unwrap_err();
    assert!(matches!(err, ParseError::Unexpected { .. }));
    assert!(err.to_string().contains("`}`"));
}
