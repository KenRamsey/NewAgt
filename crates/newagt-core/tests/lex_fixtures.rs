use newagt_core::keyword::Keyword;
use newagt_core::{lex, TokenKind};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(path).expect("read fixture")
}

fn kw(tokens: &[newagt_core::Token], idx: usize) -> Keyword {
    match &tokens[idx].kind {
        TokenKind::Keyword(k) => *k,
        other => panic!("expected keyword at {idx}, got {other:?}"),
    }
}

#[test]
fn example1_field_values() {
    let tokens = lex(&fixture("example1_tgt_fields.agt")).unwrap();
    assert_eq!(kw(&tokens, 0), Keyword::TgtType);
    assert_eq!(kw(&tokens, 1), Keyword::PixLoc);
    assert_eq!(kw(&tokens, 2), Keyword::Aspect);
    assert_eq!(kw(&tokens, 3), Keyword::Range);
    assert!(matches!(&tokens[4].kind, TokenKind::String(s) if s == "M1A1"));
    assert!(matches!(tokens[5].kind, TokenKind::Integer(200)));
    assert!(matches!(tokens[6].kind, TokenKind::Integer(230)));
    assert!(matches!(tokens[7].kind, TokenKind::Real(r) if (r - 45.0).abs() < 1e-6));
    assert!(matches!(tokens[8].kind, TokenKind::Real(r) if (r - 1500.0).abs() < 1e-6));
}

#[test]
fn example2_two_tgt_headers_and_values() {
    let tokens = lex(&fixture("example2_two_tgts.agt")).unwrap();
    assert_eq!(kw(&tokens, 0), Keyword::Tgt);
    assert!(matches!(tokens[1].kind, TokenKind::LBrace));
    assert!(matches!(tokens[6].kind, TokenKind::RBrace));
    assert_eq!(kw(&tokens, 7), Keyword::Tgt);
    assert!(matches!(&tokens[14].kind, TokenKind::String(s) if s == "M1A1"));
    assert!(matches!(&tokens[19].kind, TokenKind::String(s) if s == "M2"));
}

#[test]
fn example3_prj_section_keywords() {
    let tokens = lex(&fixture("example3_prj_header.agt")).unwrap();
    assert_eq!(kw(&tokens, 0), Keyword::PrjSect);
    assert_eq!(kw(&tokens, 2), Keyword::Name);
    assert!(matches!(&tokens[3].kind, TokenKind::String(s) if s == "vision7"));
    assert_eq!(kw(&tokens, 4), Keyword::Scenario);
    assert_eq!(kw(&tokens, 6), Keyword::Site);
    assert_eq!(kw(&tokens, 8), Keyword::Time);
    assert_eq!(kw(&tokens, 15), Keyword::LatLong);
    let comments = tokens
        .iter()
        .filter(|t| matches!(t.kind, TokenKind::Keyword(Keyword::Comment)))
        .count();
    assert_eq!(comments, 2);
}

#[test]
fn prototype_has_keyword_extension() {
    let tokens = lex(&fixture("prototype_prj_header.agt")).unwrap();
    assert_eq!(kw(&tokens, 0), Keyword::Agt);
    assert_eq!(kw(&tokens, 2), Keyword::PrjSect);
    assert!(
        tokens
            .iter()
            .any(|t| matches!(t.kind, TokenKind::Keyword(Keyword::Keyword)))
    );
    assert!(matches!(
        &tokens.iter().find(|t| {
            matches!(&t.kind, TokenKind::String(s) if s.contains("Keyword 1"))
        })
        .expect("keyword string")
        .kind,
        TokenKind::String(_)
    ));
}

#[test]
fn spans_are_one_based() {
    let tokens = lex("Agt\n{").unwrap();
    assert_eq!(tokens[0].span.line, 1);
    assert_eq!(tokens[0].span.column, 1);
    assert_eq!(tokens[1].span.line, 2);
    assert_eq!(tokens[1].span.column, 1);
}

#[test]
fn crlf_fixture_lexes() {
    let tokens = lex(&fixture("crlf_header.agt")).unwrap();
    assert_eq!(kw(&tokens, 0), Keyword::Agt);
    assert!(matches!(tokens[1].kind, TokenKind::LBrace));
    assert!(matches!(tokens[2].kind, TokenKind::RBrace));
}

#[test]
fn underscore_unknown_keyword_fixture() {
    let tokens = lex(&fixture("underscore_field.agt")).unwrap();
    assert!(
        tokens.iter().any(|t| matches!(
            &t.kind,
            TokenKind::UnknownKeyword(s) if s == "PLATFORM_LATITUDE"
        )),
        "expected PLATFORM_LATITUDE token"
    );
}
