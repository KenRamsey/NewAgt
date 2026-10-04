//! M5: AGTJ parse profile and PDF-strict mode.

use newagt_core::ast::{
    AgtItem, SenSectItem, SenUpdItem, TgtItem, TgtSectItem, TgtUpdItem,
};
use newagt_core::{
    parse_with_options, FieldValue, ParseOptions, ParseProfile, ParseWarningKind,
};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).expect("read fixture")
}

#[test]
fn prototype_sen_sect_parses_under_agtj_profile() {
    let src = fixture("prototype_sen_sect_snippet.agt");
    let result = parse_with_options(
        &src,
        ParseOptions {
            profile: ParseProfile::Agtj,
        },
    )
    .expect("parse agtj");

    assert!(result.warnings.is_empty());

    let AgtItem::SenSect(sen) = &result.document.root.items[0] else {
        panic!("SenSect");
    };
    assert_eq!(sen.items.len(), 3);
    let SenSectItem::SenUpd(upd) = &sen.items[2] else {
        panic!("SenUpd");
    };

    let mut saw_fov_in_upd = false;
    let mut saw_utm = false;
    for item in &upd.items {
        let SenUpdItem::Field(f) = item else {
            continue;
        };
        if let FieldValue::Fov(fov) = &f.value {
            saw_fov_in_upd = true;
            assert!((fov.horizontal - 40.0).abs() < f64::EPSILON);
            assert!((fov.vertical - 30.0).abs() < f64::EPSILON);
        }
        if let FieldValue::Utm(u) = &f.value {
            saw_utm = true;
            assert_eq!(u.easting, 323_339);
            assert_eq!(u.northing, 4_305_586);
            assert!((u.elevation - 26.0).abs() < f64::EPSILON);
            assert!(u.grid.is_none());
        }
    }
    assert!(saw_fov_in_upd, "Fov expected inside SenUpd under agtj");
    assert!(saw_utm, "Utm expected in SenUpd");
}

#[test]
fn prototype_tgt_sect_pixbox_under_agtj_profile() {
    let src = fixture("prototype_tgt_sect_snippet.agt");
    let result = parse_with_options(
        &src,
        ParseOptions {
            profile: ParseProfile::Agtj,
        },
    )
    .expect("parse agtj");

    let AgtItem::TgtSect(tgt_sect) = &result.document.root.items[0] else {
        panic!("TgtSect");
    };
    let TgtSectItem::TgtUpd(upd) = &tgt_sect.items[0] else {
        panic!("TgtUpd");
    };
    let TgtUpdItem::Tgt(tgt) = &upd.items[2] else {
        panic!("Tgt");
    };

    let mut saw_pix_box = false;
    for item in &tgt.items {
        let TgtItem::Field(f) = item else {
            continue;
        };
        if let FieldValue::PixBox(b) = &f.value {
            saw_pix_box = true;
            assert_eq!(b.upper_left.x, 62);
            assert_eq!(b.upper_left.y, 317);
            assert_eq!(b.lower_right.x, 82);
            assert_eq!(b.lower_right.y, 327);
        }
    }
    assert!(saw_pix_box, "PixBox on Tgt under agtj");
}

#[test]
fn pdf1999_profile_warns_on_fov_in_sen_upd() {
    let src = r#"Agt {
  SenSect {
    SenUpd {
      Fov 40.0 30.0
      Time 2005 256 9 9 56 637
    }
  }
}"#;

    let agtj = parse_with_options(
        src,
        ParseOptions {
            profile: ParseProfile::Agtj,
        },
    )
    .expect("agtj parse");
    assert!(agtj.warnings.is_empty());
    assert!(matches!(
        &agtj.document.root.items[0],
        AgtItem::SenSect(_)
    ));

    let pdf = parse_with_options(
        src,
        ParseOptions {
            profile: ParseProfile::Pdf1999,
        },
    )
    .expect("pdf1999 still loadable");
    assert_eq!(pdf.warnings.len(), 1);
    assert_eq!(
        pdf.warnings[0].kind,
        ParseWarningKind::ProfileExtension
    );
    assert!(pdf.warnings[0].message.contains("Fov"));

    let AgtItem::SenSect(sen) = &pdf.document.root.items[0] else {
        panic!("SenSect");
    };
    let SenSectItem::SenUpd(upd) = &sen.items[0] else {
        panic!("SenUpd");
    };
    assert!(matches!(&upd.items[0], SenUpdItem::Unknown(_)));
}

#[test]
fn utm_five_fields_agtj_only() {
    use newagt_core::value::parse_field_value;
    use newagt_core::{Keyword, Span, Token, TokenKind};

    fn int(n: i64) -> Token {
        Token::new(TokenKind::Integer(n), Span::new(1, 1, 0))
    }
    fn real(n: f64) -> Token {
        Token::new(TokenKind::Real(n), Span::new(1, 1, 0))
    }
    fn str(s: &str) -> Token {
        Token::new(TokenKind::String(s.to_string()), Span::new(1, 1, 0))
    }

    let tokens = vec![
        int(1),
        int(2),
        real(3.0),
        str("18N"),
        str("WGS84"),
    ];

    parse_field_value(ParseProfile::Agtj, Keyword::Utm, tokens.clone()).expect("agtj utm");
    let err = parse_field_value(ParseProfile::Pdf1999, Keyword::Utm, tokens).unwrap_err();
    assert!(err.message.contains("pdf1999"));
}
