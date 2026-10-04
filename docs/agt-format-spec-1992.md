# AGT ground truth format — summary from `Agt-1992.pdf`

**Source:** Mark Smith, E-OIR Measurements / NVEOD — *ARF Ground Truth Specification*, **Appendix C: AGT Specification** (PDF dated **May 4, 1999**; file name `Agt-1992.pdf`).  
**Status:** **Starting skeleton only** — Ken’s corpus and AGTJ diverge; do not treat as normative.  
**Related:** [domain correction](agt-domain-correction.md), [AGT in the wild](agt-real-world-format.md)

---

## What “AGT” is (per PDF)

- **Role:** ASCII **ground truth** container format, designed to complement **ARF** (*Another Raster Format*). ARF holds raster imagery (XDR, random access); AGT holds **human-readable** metadata about tests, sensors, targets, and **image-plane geometry**.
- **Use cases:** Field tests, terrain board, synthetic imagery; static/moving targets and sensors.
- **Acronym:** The PDF **does not expand “AGT”** as a phrase (only “AGT Specification” and the top-level object name **`Agt`**). It is **not** Adventure Game Toolkit.
- **Design philosophy:** Optional fields; software **queries** the file for what is present (“no data is better than bad data”). Format described as a **small language** (lex/yacc-friendly).

---

## Physical file model

| Aspect | PDF rule |
|--------|----------|
| Encoding | **ASCII text** |
| Structure | **Keywords** + **containers** `{` … `}` + **data values** |
| Separators | At least one **tab or space** between tokens |
| Top level | Exactly one root container: keyword **`Agt`** |
| Strings | Double-quoted (`"..."`); lexer allows `\"`-less content until quote/newline |
| Numbers | Integer (`[0-9]+`), real (decimal and scientific) |
| Identifiers | Keywords match `[a-zA-Z]*` (case-sensitive in reference grammar) |

**Typical extension:** `.agt` (AGTJ samples: `Prototype.agt`, etc.) — **not specified in PDF**.

There is **no embedded image payload** in the PDF spec; image linkage is via **PixLoc**, **PixRange**, **Fov**, and project/scenario IDs that tie ground truth to **ARF** or other sensor data.

---

## Container hierarchy

```
Agt
├── PrjSect   (optional) — project / scenario grouping
├── SenSect   (optional) — sensor static + updates
└── TgtSect   (optional) — target updates
```

**Placement rules (PDF):**

- `Agt` may contain zero or more of `PrjSect`, `SenSect`, `TgtSect` (all optional).
- `Tgt` **must not** hang directly under `Agt`; path is `Agt → TgtSect → TgtUpd → Tgt`.
- **Container list objects** (may repeat under one parent): `TgtUpd`, `Tgt`, `SenUpd`.

**Syntax pattern:**

```text
ContainerKeyword
{
  … daughter objects and data keywords …
}
```

---

## Container contents (spec tables)

### `Agt`

| Daughter | Purpose |
|----------|---------|
| `TgtSect` | Target-related containers |
| `SenSect` | Sensor-related containers |
| `PrjSect` | Group files into a project |

### `TgtSect`

| Child | Purpose |
|-------|---------|
| `Comment` | Free text (repeatable in examples) |
| `TgtUpd` | One update per sensor update or automated ground-truth pass; one update suffices for static scenarios |

### `TgtUpd`

| Child | Purpose |
|-------|---------|
| `Comment` | |
| `Time` | Timestamp for target location info |
| `Tgt` | One or more targets / target-like objects / false alarms |

### `Tgt`

| Child | Purpose |
|-------|---------|
| `Comment` | Subjective notes; target-like / false-alarm identification |
| `TgtType` | Valid target type for ATR scoring (optional if not a scored target) |
| `PlyId` | Target serial number |
| `Aspect` | Target aspect relative to sensor (degrees, clockwise from sensor-facing = 0°) |
| `Range` | Sensor-to-target range (meters) |
| `PixLoc` | Target in **image coordinates** (pixels) |
| `TgtSenRel` | Sensor-relative target container |
| `TgtAbs` | Absolute (world) target container |

### `TgtSenRel`

| Field keyword | Units (PDF) |
|---------------|-------------|
| `Azimuth`, `Elevation`, `Pitch`, `Roll` | degrees |
| `Obscuration` | 0–100% |

### `TgtAbs`

| Field keyword | Notes |
|---------------|-------|
| `Utm` | Easting, northing, elevation |
| `LatLong` | DMS + N/S, E/W |
| `Stake` | Numbered test-area location |
| `Azimuth`, `Elevation`, `Roll` | vs true north / tilt |

### `SenSect`

| Child | Purpose |
|-------|---------|
| `Comment` | |
| `Name` | Sensor name |
| `Fov` | Field of view |
| `SenUpd` | Sensor pose/update (one enough if static) |

### `SenUpd`

| Child | Purpose |
|-------|---------|
| `Comment` | |
| `Time` | Sensor pointing timestamp |
| `Azimuth`, `Elevation`, `Roll` | Sensor pointing / focal plane |
| `LatLong`, `Utm` | Sensor position |
| `Range` | Range to CFOV |
| `PixRange` | **Repeatable:** image point + range (meters) |

### `PrjSect`

| Child | Purpose |
|-------|---------|
| `Name` | Project / field test name |
| `Scenario` | Numeric id linking ground truth, digital sensor, other data |
| `Site` | Collection location |
| `Time` | Scenario start (local) |
| `LatLong` | Collection coordinates |
| `Comment` | |

---

## Composite data shapes (inline fields)

Keywords introduce a **fixed sequence** of values (not nested `{` blocks in the PDF grammar).

| Keyword | Fields (order) | Types |
|---------|----------------|-------|
| `Time` | year, julian_day, hour, min, sec, ms | 6× integer |
| `PixLoc` | x, y | 2× integer (pixels: left, top) |
| `Utm` | easting, northing, elevation | long, long, float |
| `LatLong` | lat_deg, lat_min, lat_sec, lat_dir, long_deg, long_min, long_sec, long_dir | int, int, float, string, … |
| `Fov` | hor, vert | 2× float (degrees) |
| `PixBox` | upper_left (PixLoc), lower_right (PixLoc) | **Typo in PDF:** `PiixLoc` for lower_right |
| `PixRange` | pix_loc (x,y), range | 2× int + float (meters) |

### Scalar data keywords (PDF §2.1.3)

| Keyword | Type | Notes |
|---------|------|-------|
| `Comment` | string | |
| `Aspect`, `Azimuth`, `Elevation`, `Pitch`, `Roll`, `Range`, `Obscuration` | float | |
| `Name`, `PlyId`, `TgtType`, `Scenario`, `Site` | string | |
| `Stake` | long | |

### Base types

- **AgtFloat:** decimal or scientific (e.g. `123.4`, `-122.0`, `2.456E5`)
- **AgtLong:** integer (PDF mentions 32-bit range)
- **AgtString:** quoted string

---

## Image / stream-related semantics (PDF)

| Concept | Meaning |
|---------|---------|
| `PixLoc` | Target centroid (or interest point) in the **image plane** — origin top-left per examples (640×480 diagram) |
| `PixRange` | Samples **range vs image coordinates** along a ray fan (multiple entries per `SenUpd`) |
| `Fov` | Sensor horizontal/vertical FOV (degrees) — ties geometry to raster extent |
| `Scenario` | Correlates ground truth with **digital sensor** and other collected data (stream/scenario id) |
| `Time` + repeated `SenUpd` / `TgtUpd` | **Time series** of pose and labels for moving sensors/targets |

The PDF does **not** define frame indices, file paths to ARF, or video stream chunking — those appear in **legacy/tooling** layers (see below).

---

## Reference grammar (PDF §6)

The appendix embeds **lex** and **yacc** sources. Recognized **keywords** in the PDF tables:

`Agt`, `TgtSect`, `TgtUpd`, `Tgt`, `TgtAbs`, `TgtSenRel`, `SenSect`, `SenUpd`, `PrjSect`, `Comment`, `PixRange`, `Aspect`, `Azimuth`, `Elevation`, `Fov`, `LatLong`, `Name`, `Obscuration`, `PlyId`, `Pitch`, `PixLoc`, `Range`, `Roll`, `Scenario`, `Site`, `Stake`, `TgtType`, `Time`, `Utm`

**Unknown keywords:** lexer warns and returns a single-character token (lenient / fragile).

**Not in PDF keyword list:** **`Keyword`** (capital K) — see legacy hints.

---

## `TgtType` vocabulary (PDF §3)

Enumerated **example** target types (U.S. and C.I.S. vehicle/aircraft names) for ATR scoring — illustrative, not a closed enum in the grammar (values are strings).

---

## Gaps, ambiguities, and PDF vs reality

| Issue | Detail |
|-------|--------|
| **Non-authoritative** | Ken: modern files often **non-compliant** yet loadable; spec is a skeleton |
| **`Keyword` vs `Comment`** | PDF grammar has **`Comment` only**; field **`Keyword`** is widely used in AGTJ/corpus as extra string slots ([wild format](agt-real-world-format.md)) |
| **Case** | PDF tables use `Comment`; yacc uses `"Comment"`. Prototypes use `Comment` / `Keyword` — confirm corpus conventions |
| **PixBox** | Defined in §2.1.2.6 but **not** wired into `Tgt` item list in yacc §6.2 |
| **TGT_SEN_SECT** | Declared in yacc `%token` line but **unused** in shown grammar |
| **Utm** | PDF: 3 numbers; AGTJ often adds **grid** and **datum** strings (5 fields) |
| **Fov in `SenUpd`** | Not in PDF `sen_upd_item` list; present in AGTJ `Prototype.agt` |
| **Filename vs date** | `Agt-1992.pdf` vs document footer **1999** |
| **Sections 4–5** | “Guidelines” header only; examples carry most normative weight |
| **Validation** | No schema version field; no checksum |

---

## Legacy hints (AGTJ — not spec)

Skim of `/home/atlas/Repos/NewC_r529/AGTJ` for names that **extend** the PDF. **Hints only.**

| PDF / grammar | AGTJ (`agtJ_types.h`, readers) |
|---------------|--------------------------------|
| `PrjSect`, `SenSect`, `TgtSect`, `TgtSt`, `PixRange`, `Agt` | Same section/target naming |
| `Comment` | `Comment[NComment]` arrays (NComment=20) at Prj/Sen/Tgt/TgtUpd/SenUpd |
| — | `Keyword[NKeyword]` arrays (NKeyword=20) — **not in PDF yacc** |
| `Tgt` fields | Adds `PixBox`, `ObscPart`, `Grid`/`Datum` on UTM, `StakeLoc`, `tgtdat[][]` |
| Updates | `AgtUpdate` linked list: **frame**, `SenUpd`/`TgtUpd` pairing, file offsets |
| Keywords in logic | e.g. `Frame#`, `STAKE_LOC_*`, `UTMGrid` parsed from Keyword strings (`get_next_Upd.c`) |
| Sample files | `Prototype*.agt`, `Pprint.agt` |

NewAgt should **parse PDF-shaped core** plus **preserve unknown / extension tokens** rather than assuming AGTJ limits (20 comments, etc.).

---

## Reference hierarchy for NewAgt

| Tier | Source |
|------|--------|
| A | Ken’s imagery AGT training corpus |
| B | This summary ← `Agt-1992.pdf` |
| C | AGTJ code + prototypes |
| D | Corpus pattern analysis (COMMENT/KEYWORD payloads) |

---

## Document provenance

Extracted via `pdftotext` from `Agt-1992.pdf` (~19.5 KB text). Full yacc/lex reproduced in PDF pages 18–26.
