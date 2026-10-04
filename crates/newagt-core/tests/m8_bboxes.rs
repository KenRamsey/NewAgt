//! M8b bounding-box resolution tests (Score vs tgtdb linear).

use std::path::PathBuf;
use std::process::Command;

use newagt_core::{
    parse, resolve_bboxes, tgtdb_get_rect, BboxMethod, BboxProvenance, BBoxOptions,
    FrameIndexOptions,
};

fn snippet_tgt_dat() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tgt.dat.snippet")
}

#[test]
fn pixbox_fixture_prototype_tgt_sect() {
    let src = include_str!("fixtures/prototype_tgt_sect_snippet.agt");
    let doc = parse(src).expect("parse");
    let options = BBoxOptions {
        tgt_dat_path: Some(snippet_tgt_dat()),
        image_width: Some(640),
        image_height: Some(480),
        fov_h_deg: Some(30.0),
        fov_v_deg: Some(20.0),
        ..Default::default()
    };
    let idx = resolve_bboxes(&doc, &options).expect("resolve");
    let b = idx.records[0].bbox.as_ref().expect("bbox");
    assert_eq!(b.provenance, BboxProvenance::PixBox);
    assert_eq!((b.x1, b.y1, b.x2, b.y2), (62, 317, 82, 327));
}

#[test]
fn score_golden_matches_c_truncation() {
    let src = r#"Agt {
  SenSect { SenUpd { Fov 30.0 20.0 Time 2000 1 0 0 0 0 } }
  TgtSect {
    TgtUpd {
      Tgt {
        TgtType "M1"
        Aspect 0.0
        Range 500.0
        PixLoc 320 240
      }
    }
  }
}"#;
    let doc = parse(src).expect("parse");
    let options = BBoxOptions {
        tgt_dat_path: Some(snippet_tgt_dat()),
        image_width: Some(640),
        image_height: Some(480),
        method: BboxMethod::Score,
        ..Default::default()
    };
    let idx = resolve_bboxes(&doc, &options).expect("resolve");
    let b = idx.records[0].bbox.as_ref().expect("bbox");
    assert_eq!(b.provenance, BboxProvenance::ScoreGeometry);
    assert_eq!((b.x1, b.y1, b.x2, b.y2), (317, 238, 324, 243));
}

#[test]
fn tgtdb_linear_floats_match_python_harness() {
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/scripts/tgtdb_get_rect.py");
    let out = Command::new("python3")
        .arg(&script)
        .args(["7.72", "3.66", "2.34", "0", "500", "30", "20", "640", "480"])
        .output()
        .expect("run python harness");
    assert!(
        out.status.success(),
        "python failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8_lossy(&out.stdout);
    let mut parts = text.split_whitespace();
    let py_w: f64 = parts.next().unwrap().parse().unwrap();
    let py_h: f64 = parts.next().unwrap().parse().unwrap();

    let (rw, rh) = tgtdb_get_rect(7.72, 3.66, 2.34, 0.0, 500.0, 30.0, 20.0, 640, 480);
    assert!((rw - py_w).abs() < 1e-9, "width rust={rw} py={py_w}");
    assert!((rh - py_h).abs() < 1e-9, "height rust={rh} py={py_h}");
}

#[test]
fn tgtdb_method_provenance_distinct_from_score() {
    let src = r#"Agt {
  SenSect { SenUpd { Fov 30.0 20.0 } }
  TgtSect {
    TgtUpd {
      Tgt { TgtType "M1" Aspect 0.0 Range 500.0 PixLoc 320 240 }
    }
  }
}"#;
    let doc = parse(src).expect("parse");
    let base = BBoxOptions {
        tgt_dat_path: Some(snippet_tgt_dat()),
        image_width: Some(640),
        image_height: Some(480),
        frame_index: FrameIndexOptions::DEFAULT,
        ..Default::default()
    };
    let score = resolve_bboxes(
        &doc,
        &BBoxOptions {
            method: BboxMethod::Score,
            ..base.clone()
        },
    )
    .expect("score");
    let tgtdb = resolve_bboxes(
        &doc,
        &BBoxOptions {
            method: BboxMethod::Tgtdb,
            ..base
        },
    )
    .expect("tgtdb");
    assert_eq!(
        score.records[0].bbox.as_ref().unwrap().provenance,
        BboxProvenance::ScoreGeometry
    );
    assert_eq!(
        tgtdb.records[0].bbox.as_ref().unwrap().provenance,
        BboxProvenance::TgtdbLinear
    );
    // Linear width is ~8.95 → truncates to 8 wide (same int box as Score for this case).
    assert_eq!(
        score.records[0].bbox.as_ref().unwrap().x1,
        tgtdb.records[0].bbox.as_ref().unwrap().x1
    );
}

#[test]
fn missing_range_emits_no_box_and_warning() {
    let src = r#"Agt {
  TgtSect { TgtUpd { Tgt { TgtType "M1" PixLoc 1 2 } } }
}"#;
    let doc = parse(src).expect("parse");
    let options = BBoxOptions {
        tgt_dat_path: Some(snippet_tgt_dat()),
        image_width: Some(640),
        image_height: Some(480),
        fov_h_deg: Some(30.0),
        fov_v_deg: Some(20.0),
        ..Default::default()
    };
    let idx = resolve_bboxes(&doc, &options).expect("resolve");
    assert!(idx.records[0].bbox.is_none());
}

#[test]
fn missing_tgt_dat_row_warns_no_panic() {
    let src = r#"Agt {
  TgtSect {
    TgtUpd {
      Tgt { TgtType "NOT_IN_SNIPPET" Range 500.0 PixLoc 10 10 Aspect 0.0 }
    }
  }
}"#;
    let doc = parse(src).expect("parse");
    let options = BBoxOptions {
        tgt_dat_path: Some(snippet_tgt_dat()),
        image_width: Some(640),
        image_height: Some(480),
        fov_h_deg: Some(30.0),
        fov_v_deg: Some(20.0),
        ..Default::default()
    };
    let idx = resolve_bboxes(&doc, &options).expect("resolve");
    assert!(idx.records[0].bbox.is_none());
    assert!(idx.warnings.iter().any(|w| w.contains("no tgt.dat row")));
}
