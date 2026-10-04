//! Document summary counts for the `info` CLI.

use crate::ast::{
    Agt, AgtItem, Document, SenSectItem, TgtSectItem, TgtUpdItem,
};
use crate::frames::{FrameIndex, PairingProvenance};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentSummary {
    pub prj_sect: u32,
    pub sen_sect: u32,
    pub tgt_sect: u32,
    pub sen_upd: u32,
    pub tgt_upd: u32,
    pub tgt: u32,
}

/// Count top-level sections and nested update/target containers.
pub fn summarize_document(doc: &Document) -> DocumentSummary {
    summarize_agt(&doc.root)
}

fn summarize_agt(agt: &Agt) -> DocumentSummary {
    let mut s = DocumentSummary::default();
    for item in &agt.items {
        match item {
            AgtItem::PrjSect(_) => s.prj_sect += 1,
            AgtItem::SenSect(sect) => {
                s.sen_sect += 1;
                for i in &sect.items {
                    if let SenSectItem::SenUpd(_) = i {
                        s.sen_upd += 1;
                    }
                }
            }
            AgtItem::TgtSect(sect) => {
                s.tgt_sect += 1;
                for i in &sect.items {
                    if let TgtSectItem::TgtUpd(upd) = i {
                        s.tgt_upd += 1;
                        for u in &upd.items {
                            if matches!(u, TgtUpdItem::Tgt(_)) {
                                s.tgt += 1;
                            }
                        }
                    }
                }
            }
            AgtItem::Unknown(_) => {}
        }
    }
    s
}

fn pairing_mode_label(mode: PairingProvenance) -> &'static str {
    match mode {
        PairingProvenance::Order => "order",
        PairingProvenance::HeuristicAgTJ => "heuristic_agtj",
    }
}

/// Comma-separated section names that appear at least once.
pub fn sections_present(summary: &DocumentSummary) -> String {
    let mut parts = Vec::new();
    if summary.prj_sect > 0 {
        parts.push("PrjSect");
    }
    if summary.sen_sect > 0 {
        parts.push("SenSect");
    }
    if summary.tgt_sect > 0 {
        parts.push("TgtSect");
    }
    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join(",")
    }
}

/// One-line summary for batch `info` over a directory (tab-separated fields).
pub fn format_info_batch_line(
    path: &str,
    frame_count: Option<usize>,
    sections: &str,
    parse_ok: bool,
) -> String {
    let frames = frame_count
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());
    let status = if parse_ok { "ok" } else { "fail" };
    format!("path: {path}\tframes: {frames}\tsections: {sections}\tparse: {status}\n")
}

/// Human-readable summary lines.
pub fn format_info(
    path: &str,
    profile: &str,
    summary: &DocumentSummary,
    frames: &FrameIndex,
) -> String {
    let authority = frames
        .authority_frame_count
        .map(|n| format!("\nframes: authority_count={n}"))
        .unwrap_or_default();
    format!(
        "path: {path}\nprofile: {profile}\nsections: PrjSect={} SenSect={} TgtSect={}\nupdates: SenUpd={} TgtUpd={}\ntargets: Tgt={}\nframes: count={} pairing={}{authority}\n",
        summary.prj_sect,
        summary.sen_sect,
        summary.tgt_sect,
        summary.sen_upd,
        summary.tgt_upd,
        summary.tgt,
        frames.len(),
        pairing_mode_label(frames.pairing_mode),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn summarize_minimal_tgt() {
        let src = include_str!("../tests/fixtures/minimal_agt_tgt.agt");
        let doc = parse(src).expect("parse");
        let s = summarize_document(&doc);
        assert_eq!(s.tgt_sect, 1);
        assert_eq!(s.tgt_upd, 1);
        assert_eq!(s.tgt, 1);
        assert_eq!(s.prj_sect, 0);
    }
}
