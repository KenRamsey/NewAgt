//! Bounding-box resolution for trainer output (M8b).

mod bbox_score;
mod bbox_tgtdb;
mod resolve;
mod tgt_dat;

pub use bbox_score::{
    center_inclusive_box, score_box_dimensions, TargetDimensions, NATO_HEIGHT_M, NATO_LENGTH_M,
    NATO_WIDTH_M,
};
pub use bbox_tgtdb::{tgtdb_get_rect, tgtdb_inclusive_box};
pub use resolve::{
    resolve_bboxes, BboxError, BboxMethod, BboxProvenance, BBoxIndex, BBoxOptions, TargetBBox,
    TargetBBoxRecord,
};
pub use tgt_dat::{TgtDatDb, TgtDatEntry, TgtDatError, DEFAULT_TEST_TGT_DAT_SNIPPET};
