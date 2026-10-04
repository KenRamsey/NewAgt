use newagt_core::{
    build_frame_index, parse, FrameIndexOptions, PairingProvenance,
};

#[test]
fn two_pair_fixture_order_mode() {
    let src = include_str!("fixtures/frames_two_pair.agt");
    let doc = parse(src).expect("parse");
    let idx = build_frame_index(&doc, FrameIndexOptions::DEFAULT);
    assert_eq!(idx.pairing_mode, PairingProvenance::Order);
    assert_eq!(idx.len(), 2);
    assert!(idx.frames[0].sensor.is_some());
    assert!(idx.frames[0].target.is_some());
}

#[test]
fn two_pair_heuristic_mode() {
    let src = include_str!("fixtures/frames_two_pair.agt");
    let doc = parse(src).expect("parse");
    let idx = build_frame_index(
        &doc,
        FrameIndexOptions {
            use_agtj_heuristics: true,
            ..FrameIndexOptions::DEFAULT
        },
    );
    assert_eq!(idx.pairing_mode, PairingProvenance::HeuristicAgTJ);
    assert_eq!(idx.len(), 2);
}

#[test]
fn document_frames_api() {
    let src = include_str!("fixtures/frames_combined_prototype.agt");
    let doc = parse(src).expect("parse");
    let idx = doc.frames(FrameIndexOptions::DEFAULT);
    assert_eq!(idx.len(), 1);
}
