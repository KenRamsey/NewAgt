//! Resolve per-target bounding boxes from a parsed document + frame index.

use std::path::PathBuf;

use serde::Serialize;

use crate::ast::{AgtItem, Document, SenSectItem};
use crate::frames::{build_frame_index, FrameIndexOptions, TargetEntry};
use crate::keyword::Keyword;
use crate::value::{FieldValue, Fov, PixBox};

use super::bbox_score::{center_inclusive_box, score_box_dimensions, TargetDimensions};
use super::bbox_tgtdb::tgtdb_inclusive_box;
use super::tgt_dat::{TgtDatDb, TgtDatError};

/// How computed boxes are derived when `PixBox` is absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BboxMethod {
    /// `Score.c` / `Abuse_Tgt.c` atan path (trainer normative default).
    #[default]
    Score,
    /// `tgtdb.py` `getRect` linear approximation (comparison / legacy tooling).
    Tgtdb,
}

/// Provenance tag for the returned rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BboxProvenance {
    PixBox,
    ScoreGeometry,
    TgtdbLinear,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TargetBBox {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub provenance: BboxProvenance,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BBoxOptions {
    pub tgt_dat_path: Option<PathBuf>,
    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub fov_h_deg: Option<f64>,
    pub fov_v_deg: Option<f64>,
    pub default_fov: Option<Fov>,
    pub method: BboxMethod,
    pub ignore_pix_box: bool,
    pub frame_index: FrameIndexOptions,
}

impl Default for BBoxOptions {
    fn default() -> Self {
        Self {
            tgt_dat_path: None,
            image_width: None,
            image_height: None,
            fov_h_deg: None,
            fov_v_deg: None,
            default_fov: None,
            method: BboxMethod::Score,
            ignore_pix_box: false,
            frame_index: FrameIndexOptions::DEFAULT,
        }
    }
}

#[derive(Debug)]
pub enum BboxError {
    TgtDat(TgtDatError),
    MissingTgtDat,
}

impl std::fmt::Display for BboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TgtDat(e) => write!(f, "tgt.dat: {e}"),
            Self::MissingTgtDat => write!(f, "tgt.dat path required for computed bounding boxes"),
        }
    }
}

impl std::error::Error for BboxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TgtDat(e) => Some(e),
            Self::MissingTgtDat => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TargetBBoxRecord {
    pub frame_index: u32,
    pub tgt_index: u32,
    pub tgt_type: Option<String>,
    pub pix_loc_x: Option<i64>,
    pub pix_loc_y: Option<i64>,
    pub bbox: Option<TargetBBox>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BBoxIndex {
    pub method: BboxMethod,
    pub records: Vec<TargetBBoxRecord>,
    pub warnings: Vec<String>,
}

pub fn resolve_bboxes(doc: &Document, options: &BBoxOptions) -> Result<BBoxIndex, BboxError> {
    let tgt_db = load_tgt_db(options)?;
    let sen_sect_fov = collect_sen_sect_fov(doc);
    let frame_index = build_frame_index(doc, options.frame_index);

    let mut records = Vec::new();
    let mut warnings = frame_index.warnings.clone();

    let (imw, imh) = match (options.image_width, options.image_height) {
        (Some(w), Some(h)) => (Some(w), Some(h)),
        (None, None) => (None, None),
        _ => {
            warnings.push("image width and height must both be set for computed boxes".into());
            (None, None)
        }
    };

    for frame in &frame_index.frames {
        let Some(target_side) = frame.target.as_ref() else {
            continue;
        };
        let sensor_fov = frame
            .sensor
            .as_ref()
            .and_then(|s| s.fov.as_ref())
            .cloned();
        let sect_fov = frame
            .sensor
            .as_ref()
            .map(|s| s.reference.sen_sect_index)
            .and_then(|i| sen_sect_fov.get(&i).cloned());

        for tgt in &target_side.targets {
            let mut rec_warnings = Vec::new();
            let bbox = resolve_one_target(
                tgt,
                options,
                tgt_db.as_ref(),
                imw,
                imh,
                sensor_fov.as_ref(),
                sect_fov.as_ref(),
                &mut rec_warnings,
            );
            warnings.extend(rec_warnings);
            records.push(TargetBBoxRecord {
                frame_index: frame.index,
                tgt_index: tgt.tgt_index,
                tgt_type: tgt.tgt_type.clone(),
                pix_loc_x: tgt.pix_loc.as_ref().map(|p| p.x),
                pix_loc_y: tgt.pix_loc.as_ref().map(|p| p.y),
                bbox,
            });
        }
    }

    Ok(BBoxIndex {
        method: options.method,
        records,
        warnings,
    })
}

fn load_tgt_db(options: &BBoxOptions) -> Result<Option<TgtDatDb>, BboxError> {
    match &options.tgt_dat_path {
        Some(path) => TgtDatDb::load(path).map(Some).map_err(BboxError::TgtDat),
        None => Ok(None),
    }
}

fn collect_sen_sect_fov(doc: &Document) -> std::collections::HashMap<u32, Fov> {
    let mut map = std::collections::HashMap::new();
    let mut sect_i = 0u32;
    for item in &doc.root.items {
        if let AgtItem::SenSect(sect) = item {
            for si in &sect.items {
                if let SenSectItem::Field(f) = si {
                    if f.keyword == Keyword::Fov {
                        if let FieldValue::Fov(v) = &f.value {
                            map.insert(sect_i, v.clone());
                        }
                    }
                }
            }
            sect_i += 1;
        }
    }
    map
}

fn resolve_fov<'a>(
    target: &'a TargetEntry,
    sensor: Option<&'a Fov>,
    sect: Option<&'a Fov>,
    options: &'a BBoxOptions,
) -> Option<(f64, f64)> {
    if let Some(f) = target.fov.as_ref() {
        return Some((f.horizontal, f.vertical));
    }
    if let Some(f) = sensor {
        return Some((f.horizontal, f.vertical));
    }
    if let Some(f) = sect {
        return Some((f.horizontal, f.vertical));
    }
    if let (Some(h), Some(v)) = (options.fov_h_deg, options.fov_v_deg) {
        return Some((h, v));
    }
    options
        .default_fov
        .as_ref()
        .map(|f| (f.horizontal, f.vertical))
}

fn bbox_from_pix_box(pix: &PixBox) -> TargetBBox {
    TargetBBox {
        x1: pix.upper_left.x as i32,
        y1: pix.upper_left.y as i32,
        x2: pix.lower_right.x as i32,
        y2: pix.lower_right.y as i32,
        provenance: BboxProvenance::PixBox,
        warnings: Vec::new(),
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_one_target(
    tgt: &TargetEntry,
    options: &BBoxOptions,
    tgt_db: Option<&TgtDatDb>,
    imw: Option<u32>,
    imh: Option<u32>,
    sensor_fov: Option<&Fov>,
    sect_fov: Option<&Fov>,
    warnings: &mut Vec<String>,
) -> Option<TargetBBox> {
    if !options.ignore_pix_box {
        if let Some(pb) = tgt.pix_box.as_ref() {
            return Some(bbox_from_pix_box(pb));
        }
    }

    let range_m = tgt.range_m.filter(|r| *r != 0.0)?;
    let (pix_x, pix_y) = match tgt.pix_loc.as_ref() {
        Some(p) => (p.x as i32, p.y as i32),
        None => {
            warnings.push(format!(
                "frame target {}: missing PixLoc; cannot center computed box",
                tgt.tgt_index
            ));
            return None;
        }
    };

    let (imw, imh) = match (imw, imh) {
        (Some(w), Some(h)) => (w, h),
        _ => {
            warnings.push(format!(
                "frame target {}: image dimensions required for computed box",
                tgt.tgt_index
            ));
            return None;
        }
    };

    let (fov_h, fov_v) = match resolve_fov(tgt, sensor_fov, sect_fov, options) {
        Some(v) => v,
        None => {
            warnings.push(format!(
                "frame target {}: missing FOV (SenUpd/SenSect/CLI)",
                tgt.tgt_index
            ));
            return None;
        }
    };

    let aspect_deg = tgt.aspect_deg.unwrap_or(0.0);

    let dims = match tgt.tgt_type.as_deref() {
        None | Some("") => TargetDimensions::NATO,
        Some(tt) => match tgt_db {
            Some(db) => match db.get(tt) {
                Some(e) => TargetDimensions {
                    length_m: e.length_m,
                    width_m: e.width_m,
                    height_m: e.height_m,
                },
                None => {
                    warnings.push(format!(
                        "frame target {}: no tgt.dat row for TgtType `{tt}`",
                        tgt.tgt_index
                    ));
                    return None;
                }
            },
            None => {
                warnings.push(format!(
                    "frame target {}: tgt.dat required for TgtType `{tt}`",
                    tgt.tgt_index
                ));
                return None;
            }
        },
    };

    match options.method {
        BboxMethod::Score => {
            let (boxw, boxh) = score_box_dimensions(
                dims, aspect_deg, range_m, fov_h, fov_v, imw, imh,
            );
            if boxw <= 0 || boxh <= 0 {
                warnings.push(format!(
                    "frame target {}: Score geometry produced non-positive box ({boxw}x{boxh})",
                    tgt.tgt_index
                ));
                return None;
            }
            let (x1, y1, x2, y2) = center_inclusive_box(boxw, boxh, pix_x, pix_y);
            Some(TargetBBox {
                x1,
                y1,
                x2,
                y2,
                provenance: BboxProvenance::ScoreGeometry,
                warnings: Vec::new(),
            })
        }
        BboxMethod::Tgtdb => {
            let (x1, y1, x2, y2) = tgtdb_inclusive_box(
                dims.length_m,
                dims.width_m,
                dims.height_m,
                aspect_deg,
                range_m,
                fov_h,
                fov_v,
                imw,
                imh,
                pix_x,
                pix_y,
            );
            if x2 < x1 || y2 < y1 {
                warnings.push(format!(
                    "frame target {}: tgtdb linear produced invalid box",
                    tgt.tgt_index
                ));
                return None;
            }
            Some(TargetBBox {
                x1,
                y1,
                x2,
                y2,
                provenance: BboxProvenance::TgtdbLinear,
                warnings: Vec::new(),
            })
        }
    }
}

impl Document {
    pub fn bboxes(&self, options: &BBoxOptions) -> Result<BBoxIndex, BboxError> {
        resolve_bboxes(self, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn snippet_tgt_dat_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/tgt.dat.snippet")
    }

    #[test]
    fn pixbox_authoritative_over_computed() {
        let src = include_str!("../../tests/fixtures/prototype_tgt_sect_snippet.agt");
        let doc = parse(src).expect("parse");
        let options = BBoxOptions {
            tgt_dat_path: Some(snippet_tgt_dat_path()),
            image_width: Some(640),
            image_height: Some(480),
            fov_h_deg: Some(30.0),
            fov_v_deg: Some(20.0),
            ..Default::default()
        };
        let idx = resolve_bboxes(&doc, &options).expect("resolve");
        let rec = idx.records.first().expect("one target");
        let b = rec.bbox.as_ref().expect("bbox");
        assert_eq!(b.provenance, BboxProvenance::PixBox);
        assert_eq!((b.x1, b.y1, b.x2, b.y2), (62, 317, 82, 327));
    }
}
