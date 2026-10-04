use newagt_core::ast::{AgtItem, Field, PrjItem, TgtItem, TgtSectItem, TgtUpdItem};
use newagt_core::keyword::Keyword;
use newagt_core::{parse, FieldValue, LatLong, ParseError, PixLoc, Time};

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
        value: FieldValue::String(s),
        ..
    }) = &tgt.items[0]
    else {
        panic!("expected TgtType field");
    };
    assert_eq!(s, "M1A1");

    let TgtItem::Field(Field {
        keyword: Keyword::PixLoc,
        value: FieldValue::PixLoc(PixLoc { x: 200, y: 230 }),
        ..
    }) = &tgt.items[1]
    else {
        panic!("expected typed PixLoc");
    };
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
        value: FieldValue::String(s),
        ..
    }) = &prj.items[0]
    else {
        panic!("Name first");
    };
    assert_eq!(s, "vision7");

    let time_field = prj
        .items
        .iter()
        .find_map(|item| match item {
            PrjItem::Field(f) if f.keyword == Keyword::Time => Some(f),
            _ => None,
        })
        .expect("Time field");
    assert_eq!(
        time_field.value,
        FieldValue::Time(Time {
            year: 1992,
            julian_day: 140,
            hour: 16,
            min: 34,
            sec: 15,
            ms: 341,
        })
    );

    let lat_field = prj
        .items
        .iter()
        .find_map(|item| match item {
            PrjItem::Field(f) if f.keyword == Keyword::LatLong => Some(f),
            _ => None,
        })
        .expect("LatLong field");
    let FieldValue::LatLong(LatLong {
        lat_deg,
        lat_dir,
        long_dir,
        ..
    }) = &lat_field.value
    else {
        panic!("typed LatLong");
    };
    assert_eq!(*lat_deg, 11);
    assert_eq!(lat_dir, "N");
    assert_eq!(long_dir, "E");

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

#[test]
fn parse_error_incomplete_lat_long_in_document() {
    let src = r#"Agt {
  PrjSect {
    LatLong 1 2 3.0 "N" 4 5 6.0
  }
}"#;
    let err = parse(src).unwrap_err();
    assert!(matches!(err, ParseError::Value(_)));
    assert!(err.to_string().contains("LatLong"));
}

#[test]
fn tgt_field_order_independence() {
    let canonical = r#"Agt {
  TgtSect {
    TgtUpd {
      Tgt {
        TgtType "M1A1"
        PixLoc 200 230
        Aspect 45.0
        Range 1500.0
      }
    }
  }
}"#;
    let reordered = r#"Agt {
  TgtSect {
    TgtUpd {
      Tgt {
        Range 1500.0
        PixLoc 200 230
        TgtType "M1A1"
        Aspect 45.0
      }
    }
  }
}"#;
    let a = parse(canonical).expect("canonical");
    let b = parse(reordered).expect("reordered");

    fn extract_tgt(doc: &newagt_core::ast::Document) -> &Field {
        let AgtItem::TgtSect(ts) = &doc.root.items[0] else {
            panic!("TgtSect");
        };
        let TgtSectItem::TgtUpd(upd) = &ts.items[0] else {
            panic!("TgtUpd");
        };
        let TgtUpdItem::Tgt(tgt) = &upd.items[0] else {
            panic!("Tgt");
        };
        let by_kw = |kw: Keyword| {
            tgt.items.iter().find_map(|item| match item {
                TgtItem::Field(f) if f.keyword == kw => Some(f),
                _ => None,
            })
        };
        by_kw(Keyword::PixLoc).expect("PixLoc")
    }

    let pl_a = extract_tgt(&a);
    let pl_b = extract_tgt(&b);
    assert_eq!(pl_a.value, pl_b.value);
    assert_eq!(
        pl_a.value,
        FieldValue::PixLoc(PixLoc { x: 200, y: 230 })
    );
}

#[test]
fn repeatable_comments_and_pix_ranges() {
    let src = r#"Agt {
  SenSect {
    Comment "a"
    Comment "b"
    SenUpd {
      PixRange 10 20 100.0
      PixRange 30 40 200.0
      Comment "inside"
    }
  }
}"#;
    let doc = parse(src).expect("parse");
    let AgtItem::SenSect(sen) = &doc.root.items[0] else {
        panic!("SenSect");
    };
    let comment_count = sen
        .items
        .iter()
        .filter(|i| matches!(i, newagt_core::ast::SenSectItem::Field(f) if f.keyword == Keyword::Comment))
        .count();
    assert_eq!(comment_count, 2);
    let upd = match &sen.items[2] {
        newagt_core::ast::SenSectItem::SenUpd(u) => u,
        _ => panic!("SenUpd"),
    };
    let pix_ranges: Vec<_> = upd
        .items
        .iter()
        .filter_map(|i| match i {
            newagt_core::ast::SenUpdItem::Field(f) if f.keyword == Keyword::PixRange => {
                Some(&f.value)
            }
            _ => None,
        })
        .collect();
    assert_eq!(pix_ranges.len(), 2);
    assert!(matches!(pix_ranges[0], FieldValue::PixRange(_)));
}
