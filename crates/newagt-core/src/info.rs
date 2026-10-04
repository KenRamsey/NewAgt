//! Document summary counts for the `info` CLI.

use crate::ast::{
    Agt, AgtItem, Document, SenSectItem, TgtSectItem, TgtUpdItem,
};

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

/// Human-readable summary lines.
pub fn format_info(path: &str, profile: &str, summary: &DocumentSummary) -> String {
    format!(
        "path: {path}\nprofile: {profile}\nsections: PrjSect={} SenSect={} TgtSect={}\nupdates: SenUpd={} TgtUpd={}\ntargets: Tgt={}\n",
        summary.prj_sect,
        summary.sen_sect,
        summary.tgt_sect,
        summary.sen_upd,
        summary.tgt_upd,
        summary.tgt,
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
