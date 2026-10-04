# Bounding box output — NewAgt design

**Audience:** Ken (imagery ground-truth AGT → trainer bboxes)  
**Status:** Implemented (M8b) in `newagt-core::bbox` and `newagt bboxes`.

---

## Summary

Most corpus AGT files do **not** include `PixBox`. Trainers still need axis-aligned pixel rectangles. NewAgt should expose a **two-path** resolver:

| Path | When | Source |
|------|------|--------|
| **Authoritative** | `PixBox` on `Tgt` | Four integers: upper-left `(x1,y1)`, lower-right `(x2,y2)` per [agt-format-spec-1992.md §2.1.2.6](agt-format-spec-1992.md) |
| **Computed** | No usable `PixBox` | Physical size from **`tgt.dat`** + `Aspect`, `Range`, sensor **`Fov`**, image size, `PixLoc` center |

Legacy references:

- **Target DB + linear size helper:** `/home/atlas/Devel/agt/tgtdb.py`, `/home/atlas/Devel/agt/tgt.dat`
- **Normative scoring geometry (atan, centering, PixBox precedence):** `NewC_r529/test/Abuse_Tgt.c`, documented in `NewC_r529/test/Score.c`
- **Parse-only Python (no bbox pass):** `/home/atlas/Devel/agt/agt_parser.py` — parses `PixBox`, `Fov`, `Aspect`, `Range` but does not compute boxes

---

## Path 1 — Authoritative `PixBox`

When a `Tgt` block contains:

```text
PixBox      62 317 82 327
```

Interpret as inclusive pixel indices (AGTJ / eval convention):

- `upper_left = (62, 317)`
- `lower_right = (82, 327)`
- Width / height: `boxw = x2 - x1 + 1`, `boxh = y2 - y1 + 1`

**Policy (Ken):** If `PixBox` is present, it **wins** over computed geometry. Do not re-derive unless an explicit override flag is set (eval `Abuse_Tgt` used `-usepixbox`-style behavior; default for trainers should match Score: PixBox when present).

**NewAgt:** `PixBox`, `Aspect`, and `Range` are copied onto `TargetEntry` in the M8 frame index and used by `resolve_bboxes`.

---

## Path 2 — Computed from geometry

### Inputs

| Input | Typical source | NewAgt parse status |
|-------|------------------|---------------------|
| `TgtType` | `Tgt` | ✓ frame index (`tgt_type`) |
| `Aspect` | `Tgt` (degrees, clockwise from sensor-facing = 0°) | Parsed on `Tgt`; **not** on `TargetEntry` yet |
| `Range` | `Tgt` (meters to CFOV) | Parsed on `Tgt`; **not** on `TargetEntry` yet |
| `PixLoc` | `Tgt` (x, y) | ✓ frame index |
| `PixRange` | `Tgt` / `SenUpd` (pixel + range) | ✓ lists on frame sides; range fallback TBD |
| `Fov` (hor, vert °) | `SenSect`, `SenUpd`, rarely `Tgt` | ✓ on sensor frame (`SenUpd`); section-level `Fov` parsed but not merged in M8 |
| Length, width, height (m) | **`tgt.dat`** lookup by `TgtType` | **Not in NewAgt** |
| Image width / height (px) | **External** (ARF / dataset metadata) | **Not in AGT** |

### `tgt.dat` format

Whitespace-delimited **text** (352 lines in legacy tree; ~tens of KB — do not commit full file).

**Default paths (legacy):**

- `$CONFIG_DIR/tgt.dat` (`tgtdb.py`)
- `/pkg/eval/etc/tgt.dat` (`Score.c`, `Abuse_Tgt.c`)

**Columns** (from `Score.c`; matches `TgtDB` loading in `tgtdb.py`):

| Col | Field | Example (`M1` line) |
|-----|--------|---------------------|
| 1 | `TgtType` key (verbatim match to AGT) | `M1` |
| 2 | Length (m) | `7.72` |
| 3 | Width (m) | `3.66` |
| 4 | Height (m) | `2.34` |
| 5 | Recognition type | `TANK` |
| 6 | Classification | `TRACKED` |
| 7 | Label | `M1` |
| 8 | Origin | `USA` |
| 9 | Short class code | `T` |
| 10 | Numeric `old_recog_val` | `20` |

**Loader behavior:**

- Index by column 1 (`TgtType`). Duplicate keys: last line wins (`TgtDB` dict).
- Lines with non-numeric L/W/H (e.g. `???`) are **skipped** in Python (`tgtdb.py` try/except).
- `RcgValDB` alternate index by column 10 — only needed for legacy recog-value pipelines.

**Fixture:** `crates/newagt-core/tests/fixtures/tgt.dat.snippet` (used in `m8_bboxes` tests).

### Normative algorithm (eval / Score)

From `Abuse_Tgt.c` (when `Range` is set and `PixBox` is not authoritative):

1. **Resolve dimensions** `(L, W, H)` from `tgt.dat` (`L`=col2, `W`=col3, `H`=col4). If `TgtType` missing: NATO defaults **W=H=2.3 m**, **L=6.4 m** (`Score.c`).
2. **Apparent horizontal extent** (meters, aspect `Asp` in radians):
   - `td = |W·cos(Asp)| + |L·sin(Asp)|`
   - `θ_w = 2·atan(0.5 · td / Range)`
   - `boxw = round or truncate to int` via: `boxw = imw · θ_w / (FovH_rad)` where `FovH` is horizontal FOV in **degrees** and `FovH_rad = FovH · π/180`.
3. **Vertical:**
   - `θ_h = 2·atan(0.5 · H / Range)`
   - `boxh = imh · θ_h / (FovV_rad)`
4. **Center on `PixLoc`** (integer inclusive box, AGTJ style):
   - `x1 = PixLocX - (boxw - 1) / 2`
   - `x2 = x1 + boxw - 1`
   - `y1 = PixLocY - (boxh - 1) / 2`
   - `y2 = y1 + boxh - 1`

**FOV resolution order (proposed for Rust):** paired frame’s `SenUpd.Fov` → `SenSect.Fov` → `BBoxOptions.default_fov`.  

**Range gate:** eval code only computes when `Range` is non-zero. Without `Range`, skip target or emit warning (no box).

### Legacy Python linear helper (`tgtdb.Tgt.getRect`)

Separate small-angle approximation (used in old tooling, **not** identical to Score height):

```python
tgt_width = |L·sin(Asp)| + |W·cos(Asp)|
pix_width = tgt_width * image_width / (range * hfov_rad)
pix_height = H * image_height / (range * vfov_rad)
```

Does **not** define centering; callers must align with `PixLoc`. M8b should implement **atan + inclusive integer box** first; optional `--linear-legacy` if parity with a specific script is required.

### Edge cases

| Case | Behavior |
|------|----------|
| `PixBox` present | Use as authoritative (Ken) |
| Unknown / missing `TgtType` | NATO defaults or skip with warning |
| No `tgt.dat` row for `TgtType` | Skip or warn; no silent zero-size box |
| Missing `Range` | No computed box (eval skips) |
| Missing `Aspect` | Treat as 0° (cos=1, sin=0) unless corpus proves otherwise |
| Missing `PixLoc` | Cannot center; warn / omit computed box |
| Missing `Fov` | Require `BBoxOptions.default_fov` or fail soft with warning |
| Missing image size | Required external option |
| `PixRange` vs `Range` | Prefer `Tgt.Range`; define per-target min range from `PixRange` list if needed |
| FOV vs raster mismatch | Score assumes image dimensions match AGT FOV; document in CLI |
| Multiple `Tgt` per frame | One bbox per `TargetEntry` |
| Duplicate `TgtType` in `tgt.dat` | Last row wins (document) |

---

## NewAgt parser cross-check (M3 / M5 / M8)

| Feature | Spec | Parsed | Frame index M8 |
|---------|------|--------|----------------|
| `PixBox` | §2.1.2.6 | ✓ `PixBox { upper_left, lower_right }` (agtj) | ✓ on `TargetEntry` |
| `PixLoc` | §2.1.2 | ✓ | ✓ |
| `Fov` | composite | ✓ | ✓ sensor `SenUpd` only |
| `Aspect`, `Range` | scalars | ✓ on `Tgt` | ✓ on `TargetEntry` |
| `SenSect.Fov` | PDF | ✓ parse | ✓ bbox FOV cascade (not frame index) |
| `tgt.dat` | external | — | ✓ `TgtDatDb::load` |

---

## Rust API sketch (M8b)

New module: `newagt_core::bbox` (names tentative).

```rust
/// Provenance of the rectangle.
pub enum BboxSource {
    PixBox,
    Computed,
}

/// Inclusive pixel rectangle (AGTJ / Score convention).
pub struct TargetBBox {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
    pub source: BboxSource,
    pub warnings: Vec<String>,
}

pub struct BBoxOptions {
    pub tgt_dat_path: PathBuf,
    pub image_width: u32,
    pub image_height: u32,
    /// Used when FOV not found on sensor section/update.
    pub default_fov: Option<Fov>,
    /// If true, recompute even when PixBox exists (default: false).
    pub ignore_pix_box: bool,
    pub frame_index: FrameIndexOptions, // optional `expected_frame_count` (see arf-frame-authority.md)
}

pub struct TargetBBoxRecord {
    pub frame_index: u32,
    pub tgt_index: u32,
    pub tgt_type: Option<String>,
    pub pix_loc: Option<PixLoc>,
    pub bbox: Option<TargetBBox>,
}

pub struct BBoxIndex {
    pub records: Vec<TargetBBoxRecord>,
    pub warnings: Vec<String>,
}

/// Build per-frame target boxes from a parsed document.
pub fn resolve_bboxes(doc: &Document, options: &BBoxOptions) -> Result<BBoxIndex, BboxError>;

impl Document {
    pub fn bboxes(&self, options: &BBoxOptions) -> Result<BBoxIndex, BboxError> {
        resolve_bboxes(self, options)
    }
}
```

Supporting types:

```rust
pub struct TgtDatEntry {
    pub tgt_type: String,
    pub length_m: f64,
    pub width_m: f64,
    pub height_m: f64,
    // optional: recognition metadata cols 5–10
}

pub struct TgtDatDb {
    pub fn load(path: &Path) -> Result<Self, TgtDatError>;
    pub fn get(&self, tgt_type: &str) -> Option<&TgtDatEntry>;
}
```

---

## CLI sketch

```text
newagt bboxes [OPTIONS] <file.agt>

Options:
  --tgt-dat PATH          Target dimension DB (default: $CONFIG_DIR/tgt.dat or none → error on compute)
  --image-size W H        Raster dimensions (required for computed path)
  --default-fov H V       Horizontal / vertical degrees if absent from AGT
  --frame-heuristics      Same as `newagt frames --heuristic-agtj`
  -o, --output FORMAT     json (default) | csv
```

JSON shape: align with `newagt.schema.v1` frames + `bboxes[]` per target, or separate `newagt.schema.bboxes.v1`.

---

## Python layer (M9)

PyO3 should expose `Document.bboxes(options)` returning a list of dicts or a **pandas**-friendly columnar export:

- `frame_idx`, `tgt_idx`, `tgt_type`, `x1`, `y1`, `x2`, `y2`, `source`, `warning`

Keep **`tgt.dat` path and image size in Python config** for trainer repos; Rust performs math for speed and test parity with eval.

---

## Verification strategy

1. **PixBox fixture:** `prototype_tgt_sect_snippet.agt` — `m8_bboxes::pixbox_fixture_prototype_tgt_sect`.
2. **Score golden:** synthetic `M1`, `Aspect 0`, `Range 500`, `Fov 30×20`, `640×480`, `PixLoc 320,240` — inclusive box `(317,238)–(324,243)`. Integer `boxw`/`boxh` match C `IS32` truncation of the double expression in `Score.c` (not banker's rounding).
3. **tgtdb linear:** Rust `tgtdb_get_rect` vs `tests/scripts/tgtdb_get_rect.py` (same formula as `tgtdb.py` `getRect`); full boxes use provenance `TgtdbLinear`, never mixed silently with Score.
4. **Missing inputs:** `missing_range_*` / `missing_tgt_dat_row_*` integration tests — warnings, no panic.
5. **tgt.dat snippet:** `TgtDatDb` unit test loads `M1` L/W/H.

---

## Related docs

- Repo: [agt-implementation-plan.md](agt-implementation-plan.md) — **M8b** milestone
- Repo: [format-notes.md](format-notes.md) — `PixBox` / `Fov` placement
- Project: [agt-format-spec-1992.md](agt-format-spec-1992.md)
