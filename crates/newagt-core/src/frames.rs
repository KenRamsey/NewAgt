//! Trainer-facing frame index: pair `SenUpd` / `TgtUpd` timelines from a parsed document.

use serde::Serialize;

use crate::ast::{
    AgtItem, Document, Field, SenSectItem, SenUpd, SenUpdItem, Tgt, TgtItem, TgtSectItem, TgtUpd,
    TgtUpdItem, UnknownStatement,
};
use crate::token::TokenKind;
use crate::keyword::Keyword;
use crate::span::Span;
use crate::value::{FieldValue, Fov, PixBox, PixLoc, PixRange, Time};

/// How sensor/target updates were aligned into frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PairingProvenance {
    /// Index *i* in collected `SenUpd` / `TgtUpd` lists (default).
    #[default]
    Order,
    /// AGTJ-inspired `Keyword` / `Comment` frame hints (non-authoritative).
    HeuristicAgTJ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FrameIndexOptions {
    /// When true, attempt frame-hint pairing before falling back to list order.
    pub use_agtj_heuristics: bool,
    /// User-supplied timeline length when set (e.g. from ARF metadata in a future reader).
    ///
    /// When `Some(n)`, the frame index has exactly **n** slots (indices `0..n-1`). Each slot
    /// uses the *i*-th collected `SenUpd` / `TgtUpd` when present; missing sides are `None`
    /// with per-frame warnings. Updates beyond index `n-1` are ignored (with a warning). Index-level
    /// warnings are emitted when the collected `SenUpd` or `TgtUpd` count differs from `n`.
    ///
    /// When `None`, pairing uses `max(sen_upd, tgt_upd)` list-order alignment (current default).
    /// ARF files are not read yet; this value is optional external authority for indexing only.
    pub expected_frame_count: Option<u32>,
}

impl FrameIndexOptions {
    pub const DEFAULT: Self = Self {
        use_agtj_heuristics: false,
        expected_frame_count: None,
    };
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SenUpdRef {
    pub sen_sect_index: u32,
    pub upd_index: u32,
    pub span: JsonSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TgtUpdRef {
    pub tgt_sect_index: u32,
    pub upd_index: u32,
    pub span: JsonSpan,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JsonSpan {
    pub line: u32,
    pub column: u32,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SensorFrameSide {
    pub reference: SenUpdRef,
    pub time: Option<Time>,
    pub fov: Option<Fov>,
    pub pix_ranges: Vec<PixRange>,
    pub azimuth: Option<f64>,
    pub elevation: Option<f64>,
    pub roll: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TargetEntry {
    pub tgt_index: u32,
    pub name: Option<String>,
    pub tgt_type: Option<String>,
    pub pix_loc: Option<PixLoc>,
    pub pix_box: Option<PixBox>,
    pub aspect_deg: Option<f64>,
    pub range_m: Option<f64>,
    pub pix_ranges: Vec<PixRange>,
    pub fov: Option<Fov>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TargetFrameSide {
    pub reference: TgtUpdRef,
    pub time: Option<Time>,
    pub targets: Vec<TargetEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Frame {
    pub index: u32,
    pub pairing: PairingProvenance,
    pub warnings: Vec<String>,
    pub sensor: Option<SensorFrameSide>,
    pub target: Option<TargetFrameSide>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FrameIndex {
    pub pairing_mode: PairingProvenance,
    /// Echo of [`FrameIndexOptions::expected_frame_count`] when the index was built.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority_frame_count: Option<u32>,
    pub warnings: Vec<String>,
    pub frames: Vec<Frame>,
}

impl FrameIndex {
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Frame> {
        self.frames.iter()
    }
}

impl<'a> IntoIterator for &'a FrameIndex {
    type Item = &'a Frame;
    type IntoIter = std::slice::Iter<'a, Frame>;

    fn into_iter(self) -> Self::IntoIter {
        self.frames.iter()
    }
}

#[derive(Clone)]
struct LocatedSenUpd {
    sen_sect_index: u32,
    upd_index: u32,
    upd: SenUpd,
}

#[derive(Clone)]
struct LocatedTgtUpd {
    tgt_sect_index: u32,
    upd_index: u32,
    upd: TgtUpd,
}

/// Build a frame timeline from a parsed document.
pub fn build_frame_index(doc: &Document, options: FrameIndexOptions) -> FrameIndex {
    let sen_upds_full = collect_sen_upds(doc);
    let tgt_upds_full = collect_tgt_upds(doc);
    let sen_count = sen_upds_full.len();
    let tgt_count = tgt_upds_full.len();

    let mut index_warnings = Vec::new();
    if let Some(n) = options.expected_frame_count {
        warn_authority_counts(sen_count, tgt_count, n, &mut index_warnings);
    }

    let (sen_upds, tgt_upds) = if let Some(n) = options.expected_frame_count {
        (
            cap_located_updates(&sen_upds_full, n),
            cap_located_updates(&tgt_upds_full, n),
        )
    } else {
        (sen_upds_full, tgt_upds_full)
    };

    let (mut pairs, pairing_mode, mut pair_warnings) = if options.use_agtj_heuristics {
        match try_heuristic_pairs(&sen_upds, &tgt_upds) {
            Some(p) => (p, PairingProvenance::HeuristicAgTJ, Vec::new()),
            None => {
                let mut w = vec![
                    "AGTJ frame hints incomplete or ambiguous; paired SenUpd/TgtUpd by list order"
                        .to_string(),
                ];
                let p = order_pairs(&sen_upds, &tgt_upds, options.expected_frame_count, &mut w);
                (p, PairingProvenance::Order, w)
            }
        }
    } else {
        let mut w = Vec::new();
        let p = order_pairs(&sen_upds, &tgt_upds, options.expected_frame_count, &mut w);
        (p, PairingProvenance::Order, w)
    };
    index_warnings.append(&mut pair_warnings);

    if let Some(n) = options.expected_frame_count {
        normalize_pairs_to_authority(n, &mut pairs, &mut index_warnings);
    }

    let per_frame_prov = pairing_mode;
    let frames = pairs
        .into_iter()
        .enumerate()
        .map(|(i, pair)| {
            let warnings = pair.warnings;
            let sensor = pair.sen.map(|s| extract_sensor_side(&s));
            let target = pair.tgt.map(|t| extract_target_side(&t));
            Frame {
                index: i as u32,
                pairing: per_frame_prov,
                warnings,
                sensor,
                target,
            }
        })
        .collect();

    FrameIndex {
        pairing_mode,
        authority_frame_count: options.expected_frame_count,
        warnings: index_warnings,
        frames,
    }
}

impl Document {
    /// Build a frame index with the given pairing options.
    pub fn frames(&self, options: FrameIndexOptions) -> FrameIndex {
        build_frame_index(self, options)
    }
}

struct PairSlot {
    sen: Option<LocatedSenUpd>,
    tgt: Option<LocatedTgtUpd>,
    warnings: Vec<String>,
}

/// Index-level warnings when an authority frame count is supplied (shared with `validate`).
pub fn authority_frame_count_messages(sen_count: usize, tgt_count: usize, n: u32) -> Vec<String> {
    let mut warnings = Vec::new();
    warn_authority_counts(sen_count, tgt_count, n, &mut warnings);
    warnings
}

fn warn_authority_counts(sen_count: usize, tgt_count: usize, n: u32, warnings: &mut Vec<String>) {
    let n_usize = n as usize;
    if sen_count > n_usize {
        warnings.push(format!(
            "SenUpd count ({sen_count}) exceeds authority frame count ({n}); ignoring updates beyond index {}",
            n.saturating_sub(1)
        ));
    }
    if tgt_count > n_usize {
        warnings.push(format!(
            "TgtUpd count ({tgt_count}) exceeds authority frame count ({n}); ignoring updates beyond index {}",
            n.saturating_sub(1)
        ));
    }
    if sen_count != n_usize {
        warnings.push(format!(
            "SenUpd count ({sen_count}) != authority frame count ({n})"
        ));
    }
    if tgt_count != n_usize {
        warnings.push(format!(
            "TgtUpd count ({tgt_count}) != authority frame count ({n})"
        ));
    }
}

fn cap_located_updates<T: Clone>(updates: &[T], n: u32) -> Vec<T> {
    updates.iter().take(n as usize).cloned().collect()
}

fn normalize_pairs_to_authority(n: u32, pairs: &mut Vec<PairSlot>, warnings: &mut Vec<String>) {
    let n_usize = n as usize;
    if pairs.len() > n_usize {
        warnings.push(format!(
            "paired timeline length ({}) exceeds authority frame count ({n}); truncating to {n} frames",
            pairs.len()
        ));
        pairs.truncate(n_usize);
    }
    while pairs.len() < n_usize {
        pairs.push(PairSlot {
            sen: None,
            tgt: None,
            warnings: vec!["empty frame (authority padding; no SenUpd/TgtUpd at this index)".into()],
        });
    }
}

fn order_pairs(
    sen_upds: &[LocatedSenUpd],
    tgt_upds: &[LocatedTgtUpd],
    authority_frame_count: Option<u32>,
    index_warnings: &mut Vec<String>,
) -> Vec<PairSlot> {
    let n = authority_frame_count
        .map(|c| c as usize)
        .unwrap_or_else(|| sen_upds.len().max(tgt_upds.len()));
    if authority_frame_count.is_none() && sen_upds.len() != tgt_upds.len() {
        index_warnings.push(format!(
            "SenUpd count ({}) != TgtUpd count ({}); extra updates appear in frames without a counterpart",
            sen_upds.len(),
            tgt_upds.len()
        ));
    }
    (0..n)
        .map(|i| {
            let mut warnings = Vec::new();
            let sen = sen_upds.get(i).cloned();
            let tgt = tgt_upds.get(i).cloned();
            if sen.is_some() && tgt.is_none() {
                warnings.push("sensor-only frame (no TgtUpd at this index)".into());
            }
            if sen.is_none() && tgt.is_some() {
                warnings.push("target-only frame (no SenUpd at this index)".into());
            }
            PairSlot {
                sen,
                tgt,
                warnings,
            }
        })
        .collect()
}

fn try_heuristic_pairs(
    sen_upds: &[LocatedSenUpd],
    tgt_upds: &[LocatedTgtUpd],
) -> Option<Vec<PairSlot>> {
    if sen_upds.is_empty() && tgt_upds.is_empty() {
        return Some(Vec::new());
    }

    let sen_hints: Vec<Option<u32>> = sen_upds
        .iter()
        .map(|u| frame_hint_from_upd_sen(&u.upd))
        .collect();
    let tgt_hints: Vec<Option<u32>> = tgt_upds
        .iter()
        .map(|u| frame_hint_from_upd_tgt(&u.upd))
        .collect();

    if sen_hints.iter().any(|h| h.is_none()) || tgt_hints.iter().any(|h| h.is_none()) {
        return None;
    }
    let sen_ids: Vec<u32> = sen_hints.iter().map(|h| h.unwrap()).collect();
    let tgt_ids: Vec<u32> = tgt_hints.iter().map(|h| h.unwrap()).collect();

    if sen_ids.len() != unique_count(&sen_ids) || tgt_ids.len() != unique_count(&tgt_ids) {
        return None;
    }
    let mut sen_sorted = sen_ids.clone();
    sen_sorted.sort_unstable();
    let mut tgt_sorted = tgt_ids.clone();
    tgt_sorted.sort_unstable();
    if sen_sorted != tgt_sorted {
        return None;
    }

    let mut by_id: std::collections::BTreeMap<u32, PairSlot> = std::collections::BTreeMap::new();
    for (i, id) in sen_ids.iter().enumerate() {
        by_id.entry(*id).or_insert(PairSlot {
            sen: None,
            tgt: None,
            warnings: Vec::new(),
        });
        by_id.get_mut(id).unwrap().sen = Some(sen_upds[i].clone());
    }
    for (i, id) in tgt_ids.iter().enumerate() {
        by_id.entry(*id).or_insert(PairSlot {
            sen: None,
            tgt: None,
            warnings: Vec::new(),
        });
        by_id.get_mut(id).unwrap().tgt = Some(tgt_upds[i].clone());
    }

    Some(by_id.into_values().collect())
}

fn unique_count(ids: &[u32]) -> usize {
    let mut v = ids.to_vec();
    v.sort_unstable();
    v.dedup();
    v.len()
}

fn collect_sen_upds(doc: &Document) -> Vec<LocatedSenUpd> {
    let mut out = Vec::new();
    let mut sect_i = 0u32;
    for item in &doc.root.items {
        if let AgtItem::SenSect(sect) = item {
            let mut upd_i = 0u32;
            for si in &sect.items {
                if let SenSectItem::SenUpd(upd) = si {
                    out.push(LocatedSenUpd {
                        sen_sect_index: sect_i,
                        upd_index: upd_i,
                        upd: upd.clone(),
                    });
                    upd_i += 1;
                }
            }
            sect_i += 1;
        }
    }
    out
}

fn collect_tgt_upds(doc: &Document) -> Vec<LocatedTgtUpd> {
    let mut out = Vec::new();
    let mut sect_i = 0u32;
    for item in &doc.root.items {
        if let AgtItem::TgtSect(sect) = item {
            let mut upd_i = 0u32;
            for ti in &sect.items {
                if let TgtSectItem::TgtUpd(upd) = ti {
                    out.push(LocatedTgtUpd {
                        tgt_sect_index: sect_i,
                        upd_index: upd_i,
                        upd: upd.clone(),
                    });
                    upd_i += 1;
                }
            }
            sect_i += 1;
        }
    }
    out
}

fn frame_hint_from_upd_sen(upd: &SenUpd) -> Option<u32> {
    frame_hint_from_items(upd.items.iter().filter_map(|i| match i {
        SenUpdItem::Field(f) => Some(f),
        _ => None,
    }))
}

fn frame_hint_from_upd_tgt(upd: &TgtUpd) -> Option<u32> {
    frame_hint_from_items(upd.items.iter().filter_map(|i| match i {
        TgtUpdItem::Field(f) => Some(f),
        _ => None,
    }))
}

fn frame_hint_from_items<'a>(fields: impl Iterator<Item = &'a Field>) -> Option<u32> {
    for f in fields {
        if f.keyword == Keyword::Keyword || f.keyword == Keyword::Comment {
            if let FieldValue::String(s) = &f.value {
                if let Some(n) = parse_frame_number(s) {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Parse AGTJ-style frame hints from `Keyword` / `Comment` text (e.g. `Frame# 1`).
pub fn parse_frame_number(text: &str) -> Option<u32> {
    let lower = text.to_ascii_lowercase();
    let pos = lower.find("frame")?;
    let mut rest = lower[pos + 5..].chars().peekable();
    while rest.peek().is_some_and(|c| c.is_whitespace()) {
        rest.next();
    }
    if rest.peek().is_some_and(|c| *c == '#') {
        rest.next();
        while rest.peek().is_some_and(|c| c.is_whitespace()) {
            rest.next();
        }
    }
    let mut digits = String::new();
    for c in rest {
        if c.is_ascii_digit() {
            digits.push(c);
        } else if !digits.is_empty() {
            break;
        }
    }
    if digits.is_empty() {
        return None;
    }
    digits.parse().ok()
}

fn extract_sensor_side(s: &LocatedSenUpd) -> SensorFrameSide {
    let mut time = None;
    let mut fov = None;
    let mut pix_ranges = Vec::new();
    let mut azimuth = None;
    let mut elevation = None;
    let mut roll = None;

    for item in &s.upd.items {
        if let SenUpdItem::Field(f) = item {
            match f.keyword {
                Keyword::Time => {
                    if let FieldValue::Time(t) = &f.value {
                        time = Some(t.clone());
                    }
                }
                Keyword::Fov => {
                    if let FieldValue::Fov(v) = &f.value {
                        fov = Some(v.clone());
                    }
                }
                Keyword::PixRange => {
                    if let FieldValue::PixRange(r) = &f.value {
                        pix_ranges.push(r.clone());
                    }
                }
                Keyword::Azimuth => azimuth = float_field(f),
                Keyword::Elevation => elevation = float_field(f),
                Keyword::Roll => roll = float_field(f),
                _ => {}
            }
        }
    }

    SensorFrameSide {
        reference: SenUpdRef {
            sen_sect_index: s.sen_sect_index,
            upd_index: s.upd_index,
            span: json_span(s.upd.span),
        },
        time,
        fov,
        pix_ranges,
        azimuth,
        elevation,
        roll,
    }
}

fn extract_target_side(t: &LocatedTgtUpd) -> TargetFrameSide {
    let mut time = None;
    let mut targets = Vec::new();
    let mut tgt_index = 0u32;

    for item in &t.upd.items {
        match item {
            TgtUpdItem::Field(f) => {
                if f.keyword == Keyword::Time {
                    if let FieldValue::Time(v) = &f.value {
                        time = Some(v.clone());
                    }
                }
            }
            TgtUpdItem::Tgt(tgt) => {
                targets.push(extract_tgt_entry(tgt, tgt_index));
                tgt_index += 1;
            }
            TgtUpdItem::Unknown(_) => {}
        }
    }

    TargetFrameSide {
        reference: TgtUpdRef {
            tgt_sect_index: t.tgt_sect_index,
            upd_index: t.upd_index,
            span: json_span(t.upd.span),
        },
        time,
        targets,
    }
}

fn extract_tgt_entry(tgt: &Tgt, tgt_index: u32) -> TargetEntry {
    let mut name = None;
    let mut tgt_type = None;
    let mut pix_loc = None;
    let mut pix_box = None;
    let mut aspect_deg = None;
    let mut range_m = None;
    let mut pix_ranges = Vec::new();
    let mut fov = None;

    for item in &tgt.items {
        match item {
            TgtItem::Field(f) => match f.keyword {
                Keyword::Name => {
                    if let FieldValue::String(s) = &f.value {
                        name = Some(s.clone());
                    }
                }
                Keyword::TgtType => {
                    if let FieldValue::String(s) = &f.value {
                        tgt_type = Some(s.clone());
                    }
                }
                Keyword::PixLoc => {
                    if let FieldValue::PixLoc(p) = &f.value {
                        pix_loc = Some(p.clone());
                    }
                }
                Keyword::PixBox => {
                    if let FieldValue::PixBox(b) = &f.value {
                        pix_box = Some(b.clone());
                    }
                }
                Keyword::Aspect => {
                    aspect_deg = float_field(f);
                }
                Keyword::Range => {
                    range_m = float_field(f);
                }
                Keyword::PixRange => {
                    if let FieldValue::PixRange(r) = &f.value {
                        pix_ranges.push(r.clone());
                    }
                }
                Keyword::Fov => {
                    if let FieldValue::Fov(v) = &f.value {
                        fov = Some(v.clone());
                    }
                }
                _ => {}
            },
            TgtItem::Unknown(u) if u.name == "Name" && name.is_none() => {
                name = unknown_first_string(u);
            }
            _ => {}
        }
    }

    TargetEntry {
        tgt_index,
        name,
        tgt_type,
        pix_loc,
        pix_box,
        aspect_deg,
        range_m,
        pix_ranges,
        fov,
    }
}

fn unknown_first_string(u: &UnknownStatement) -> Option<String> {
    u.values.iter().find_map(|t| {
        if let TokenKind::String(s) = &t.kind {
            Some(s.clone())
        } else {
            None
        }
    })
}

fn float_field(f: &Field) -> Option<f64> {
    match &f.value {
        FieldValue::Float(v) => Some(*v),
        FieldValue::Integer(v) => Some(*v as f64),
        _ => None,
    }
}

fn json_span(span: Span) -> JsonSpan {
    JsonSpan {
        line: span.line,
        column: span.column,
        offset: span.offset,
    }
}

/// One-line summary for CLI listing.
pub fn format_frame_line(frame: &Frame) -> String {
    let sen = frame.sensor.is_some();
    let tgt = frame.target.is_some();
    let tgt_n = frame
        .target
        .as_ref()
        .map(|t| t.targets.len())
        .unwrap_or(0);
    let time_s = frame
        .sensor
        .as_ref()
        .and_then(|s| s.time.as_ref())
        .or_else(|| frame.target.as_ref().and_then(|t| t.time.as_ref()))
        .map(format_time_short)
        .unwrap_or_else(|| "-".into());
    let warn = if frame.warnings.is_empty() {
        String::new()
    } else {
        format!(" warnings={}", frame.warnings.len())
    };
    format!(
        "frame {}: sen={sen} tgt={tgt} targets={tgt_n} time={time_s} pairing={pairing}{warn}",
        frame.index,
        pairing = match frame.pairing {
            PairingProvenance::Order => "order",
            PairingProvenance::HeuristicAgTJ => "heuristic_agtj",
        }
    )
}

fn format_time_short(t: &Time) -> String {
    format!(
        "{}:{:03}:{:02}:{:02}.{:03}",
        t.year, t.julian_day, t.hour, t.min, t.ms
    )
}

// Serialize Time and composite values for frame JSON snapshots.
impl Serialize for Time {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Time", 6)?;
        s.serialize_field("year", &self.year)?;
        s.serialize_field("julian_day", &self.julian_day)?;
        s.serialize_field("hour", &self.hour)?;
        s.serialize_field("min", &self.min)?;
        s.serialize_field("sec", &self.sec)?;
        s.serialize_field("ms", &self.ms)?;
        s.end()
    }
}

impl Serialize for PixLoc {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("PixLoc", 2)?;
        s.serialize_field("x", &self.x)?;
        s.serialize_field("y", &self.y)?;
        s.end()
    }
}

impl Serialize for PixBox {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("PixBox", 2)?;
        s.serialize_field("upper_left", &self.upper_left)?;
        s.serialize_field("lower_right", &self.lower_right)?;
        s.end()
    }
}

impl Serialize for PixRange {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("PixRange", 3)?;
        s.serialize_field("x", &self.x)?;
        s.serialize_field("y", &self.y)?;
        s.serialize_field("range_m", &self.range_m)?;
        s.end()
    }
}

impl Serialize for Fov {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Fov", 2)?;
        s.serialize_field("horizontal", &self.horizontal)?;
        s.serialize_field("vertical", &self.vertical)?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn parse_frame_number_variants() {
        assert_eq!(parse_frame_number("Frame# 1"), Some(1));
        assert_eq!(parse_frame_number("keyword frame 12 end"), Some(12));
        assert_eq!(parse_frame_number("no hint"), None);
    }

    #[test]
    fn order_pairing_prototype_snippets() {
        let src = include_str!("../tests/fixtures/frames_combined_prototype.agt");
        let doc = parse(src).expect("parse combined");
        let idx = build_frame_index(&doc, FrameIndexOptions::DEFAULT);
        assert_eq!(idx.len(), 1);
        assert_eq!(idx.pairing_mode, PairingProvenance::Order);
        let f = &idx.frames[0];
        assert!(f.sensor.is_some());
        assert!(f.target.is_some());
        assert_eq!(f.target.as_ref().unwrap().targets.len(), 1);
        assert_eq!(
            f.target.as_ref().unwrap().targets[0].pix_loc,
            Some(PixLoc { x: 72, y: 322 })
        );
    }

    #[test]
    fn heuristic_pairing_by_frame_keyword() {
        let src = r#"Agt {
  SenSect {
    SenUpd { Keyword "Frame# 2" Time 2005 256 9 9 56 637 }
    SenUpd { Keyword "Frame# 1" Time 2005 256 9 9 56 638 }
  }
  TgtSect {
    TgtUpd { Keyword "Frame# 1" Tgt { TgtType "a" PixLoc 1 2 } }
    TgtUpd { Keyword "Frame# 2" Tgt { TgtType "b" PixLoc 3 4 } }
  }
}"#;
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
        assert_eq!(
            idx.frames[0]
                .target
                .as_ref()
                .unwrap()
                .targets[0]
                .tgt_type
                .as_deref(),
            Some("a")
        );
        assert_eq!(
            idx.frames[1]
                .target
                .as_ref()
                .unwrap()
                .targets[0]
                .tgt_type
                .as_deref(),
            Some("b")
        );
    }

    #[test]
    fn unequal_lengths_warn() {
        let src = r#"Agt {
  SenSect { SenUpd { Time 2000 1 0 0 0 0 } SenUpd { Time 2000 1 0 0 0 1 } }
  TgtSect { TgtUpd { Tgt { Name "only" } } }
}"#;
        let doc = parse(src).expect("parse");
        let idx = build_frame_index(&doc, FrameIndexOptions::DEFAULT);
        assert_eq!(idx.len(), 2);
        assert!(!idx.warnings.is_empty());
        assert!(!idx.frames[1].warnings.is_empty());
    }

    #[test]
    fn authority_frame_count_pads_and_warns() {
        let src = r#"Agt {
  SenSect { SenUpd { Time 2000 1 0 0 0 0 } SenUpd { Time 2000 1 0 0 0 1 } }
  TgtSect { TgtUpd { Tgt { Name "only" } } }
}"#;
        let doc = parse(src).expect("parse");
        let idx = build_frame_index(
            &doc,
            FrameIndexOptions {
                expected_frame_count: Some(5),
                ..FrameIndexOptions::DEFAULT
            },
        );
        assert_eq!(idx.len(), 5);
        assert_eq!(idx.authority_frame_count, Some(5));
        assert!(idx.warnings.iter().any(|w| w.contains("SenUpd count (2)")));
        assert!(idx.warnings.iter().any(|w| w.contains("TgtUpd count (1)")));
        assert!(idx.frames[0].sensor.is_some());
        assert!(idx.frames[0].target.is_some());
        assert!(idx.frames[1].sensor.is_some());
        assert!(idx.frames[1].target.is_none());
        assert!(idx.frames[4].sensor.is_none());
        assert!(idx.frames[4].target.is_none());
    }

    #[test]
    fn authority_frame_count_unset_unchanged() {
        let src = r#"Agt {
  SenSect { SenUpd { Time 2000 1 0 0 0 0 } SenUpd { Time 2000 1 0 0 0 1 } }
  TgtSect { TgtUpd { Tgt { Name "only" } } }
}"#;
        let doc = parse(src).expect("parse");
        let idx = build_frame_index(&doc, FrameIndexOptions::DEFAULT);
        assert_eq!(idx.len(), 2);
        assert!(idx.authority_frame_count.is_none());
    }

    #[test]
    fn authority_frame_count_caps_excess_sen() {
        let src = r#"Agt {
  SenSect {
    SenUpd { Time 2000 1 0 0 0 0 }
    SenUpd { Time 2000 1 0 0 0 1 }
    SenUpd { Time 2000 1 0 0 0 2 }
  }
  TgtSect { TgtUpd { Tgt { Name "t" } } }
}"#;
        let doc = parse(src).expect("parse");
        let idx = build_frame_index(
            &doc,
            FrameIndexOptions {
                expected_frame_count: Some(1),
                ..FrameIndexOptions::DEFAULT
            },
        );
        assert_eq!(idx.len(), 1);
        assert!(idx.warnings.iter().any(|w| w.contains("exceeds authority")));
        assert!(idx.frames[0].sensor.is_some());
    }

    #[test]
    fn frame_index_json_snapshot() {
        let src = include_str!("../tests/fixtures/frames_two_pair.agt");
        let doc = parse(src).expect("parse");
        let idx = build_frame_index(&doc, FrameIndexOptions::DEFAULT);
        let json = serde_json::to_string_pretty(&idx).expect("serde");
        assert!(json.contains("\"pairing_mode\": \"order\""));
        let v: serde_json::Value = serde_json::from_str(&json).expect("json");
        assert_eq!(v["frames"].as_array().unwrap().len(), 2);
        assert_eq!(
            v["frames"][0]["target"]["targets"][0]["pix_loc"]["x"],
            10
        );
    }
}
