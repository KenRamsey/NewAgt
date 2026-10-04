//! M4: Comment/Keyword growth, UnknownStatement, parse warnings.

use newagt_core::ast::{
    AgtItem, PrjItem, SenSectItem, SenUpdItem, TgtAbsItem, TgtItem, TgtSenRelItem, TgtSectItem,
    TgtUpdItem,
};
use newagt_core::keyword::Keyword;
use newagt_core::{parse_with_warnings, FieldValue, ParseWarningKind};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).expect("read fixture")
}

fn count_comment_keyword_fields(doc: &newagt_core::Document) -> usize {
    fn walk_field(keyword: Keyword) -> bool {
        matches!(keyword, Keyword::Comment | Keyword::Keyword)
    }

    let mut n = 0usize;
    let root = &doc.root;
    for agt_item in &root.items {
        match agt_item {
            AgtItem::PrjSect(prj) => {
                for item in &prj.items {
                    match item {
                        PrjItem::Field(f) if walk_field(f.keyword) => n += 1,
                        _ => {}
                    }
                }
            }
            AgtItem::SenSect(sen) => {
                for item in &sen.items {
                    match item {
                        SenSectItem::Field(f) if walk_field(f.keyword) => n += 1,
                        SenSectItem::SenUpd(upd) => {
                            for u in &upd.items {
                                if let SenUpdItem::Field(f) = u {
                                    if walk_field(f.keyword) {
                                        n += 1;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            AgtItem::TgtSect(ts) => {
                for item in &ts.items {
                    match item {
                        TgtSectItem::Field(f) if walk_field(f.keyword) => n += 1,
                        TgtSectItem::TgtUpd(upd) => {
                            for u in &upd.items {
                                match u {
                                    TgtUpdItem::Field(f) if walk_field(f.keyword) => n += 1,
                                    TgtUpdItem::Tgt(tgt) => {
                                        for t in &tgt.items {
                                            match t {
                                                TgtItem::Field(f) if walk_field(f.keyword) => {
                                                    n += 1;
                                                }
                                                TgtItem::TgtSenRel(rel) => {
                                                    for r in &rel.items {
                                                        if let TgtSenRelItem::Field(f) = r {
                                                            if walk_field(f.keyword) {
                                                                n += 1;
                                                            }
                                                        }
                                                    }
                                                }
                                                TgtItem::TgtAbs(abs) => {
                                                    for a in &abs.items {
                                                        if let TgtAbsItem::Field(f) = a {
                                                            if walk_field(f.keyword) {
                                                                n += 1;
                                                            }
                                                        }
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            AgtItem::Unknown(_) => {}
        }
    }
    n
}

#[test]
fn prototype_burst_comment_keyword_exceeds_agtj_cap() {
    let result =
        parse_with_warnings(&fixture("prototype_comment_keyword_burst.agt")).expect("load burst");
    let count = count_comment_keyword_fields(&result.document);
    assert!(
        count > 20,
        "expected >20 Comment/Keyword field records, got {count}"
    );
}

#[test]
fn unknown_keyword_in_burst_fixture() {
    let result =
        parse_with_warnings(&fixture("prototype_comment_keyword_burst.agt")).expect("load burst");
    let unknown = result
        .warnings
        .iter()
        .any(|w| w.kind == ParseWarningKind::UnknownKeyword);
    assert!(unknown, "expected unknown-keyword warning");
    let AgtItem::PrjSect(prj) = &result.document.root.items[0] else {
        panic!("PrjSect first");
    };
    let custom = prj.items.iter().find_map(|item| match item {
        PrjItem::Unknown(stmt) if stmt.name == "CustomTrainerTag" => Some(stmt),
        _ => None,
    });
    let stmt = custom.expect("CustomTrainerTag UnknownStatement");
    assert_eq!(stmt.values.len(), 2);
    assert!(matches!(
        stmt.values[0].kind,
        newagt_core::TokenKind::String(_)
    ));
}

#[test]
fn keyword_string_field_distinct_from_comment() {
    let result = parse_with_warnings(
        r#"Agt { PrjSect { Keyword "frame-hint" Comment "human note" } }"#,
    )
    .expect("parse");
    assert!(result.warnings.is_empty());
    let AgtItem::PrjSect(prj) = &result.document.root.items[0] else {
        panic!("PrjSect");
    };
    let PrjItem::Field(kw) = &prj.items[0] else {
        panic!("Keyword field");
    };
    assert_eq!(kw.keyword, Keyword::Keyword);
    assert_eq!(kw.value, FieldValue::String("frame-hint".into()));
    let PrjItem::Field(c) = &prj.items[1] else {
        panic!("Comment field");
    };
    assert_eq!(c.keyword, Keyword::Comment);
}
