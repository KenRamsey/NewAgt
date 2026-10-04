# NewAgt implementation plan — imagery ground truth AGT

**Aligned to:** [agt-format-spec-1992.md](agt-format-spec-1992.md) (`Agt-1992.pdf`) — **not** DA1/AGiliTy.  
**Wild format:** [agt-real-world-format.md](agt-real-world-format.md)  
**Repo:** `/home/atlas/Repos/NewAgt` (Rust workspace: `newagt-core`, `newagt-cli`)

---

## Goals

1. **Read** ASCII `.agt` containers into a structured in-memory model (and JSON export).
2. **Preserve** all `Comment` / `Keyword` (and unknown keyword) payloads with source locations.
3. **Validate** with **reporting**, not gatekeeping — match Ken’s corpus loadability.
4. Later: **index** updates/frames for PyTorch trainers (thin Python API on native core).

---

## Reference tiers (unchanged)

| Tier | Role |
|------|------|
| A | Real training AGT files |
| B | `Agt-1992.pdf` / format summary |
| C | AGTJ (`NewC_r529/AGTJ`) — evolving C parser; accommodations valuable, not authoritative |
| D | COMMENT/KEYWORD pattern inventory |

---

## Current state — M0 (done)

| Item | Status |
|------|--------|
| Cargo workspace + MIT license | ✓ |
| `newagt-core` with `VERSION` | ✓ |
| `newagt-cli` stubs: `info`, `dump`, `validate`, `to-json` | ✓ (exit 2, not implemented) |
| Domain docs in repo | ✓ (`agt-domain-correction.md`, etc.) |
| Obsolete adventure-game plan | Marked withdrawn in `agt-parser-plan.md` |

**M0 exit criteria:** `cargo test`, `cargo clippy -- -D warnings` clean; CLI prints stub message. **Met.**

---

## M1 — Lexer and token model (done)

**Deliverables**

- `newagt-core::lex`: whitespace, newlines, `{`/`}`, integers, reals, quoted strings (PDF lex behavior).
- Keyword table matching PDF §6.1 (+ register **`Keyword`** as extension token for corpus).
- Source span (line, column, byte offset) on every token.

**Tests:** Tokenize PDF **Example 1–3** fragments; golden token streams.

**Exit:** No parsing yet; unknown alphabetic tokens classified as `UnknownKeyword` (not silent single-char yacc behavior).

---

## M2 — AST and PDF grammar parser (done)

**Deliverables**

- Container AST: `Agt { prj, sen, tgt }` with nested `*Upd`, `Tgt`, `TgtSenRel`, `TgtAbs`.
- Parser driven by PDF yacc structure (§6.2): placement rules for `Tgt` under `TgtUpd`, etc.
- Scalar fields attached to owning container with **occurrence order** preserved.

**Tests:** Parse minimal `Agt { TgtSect { TgtUpd { Tgt { … } } } } }`; parse full Example 3 from PDF text fixture.

**Exit:** `parse(&str) -> Result<Document, ParseError>` with structured errors (line, expected token). **Met.**

---

## M3 — Composite value parsing (done)

**Deliverables**

- Typed values: `Time`, `PixLoc`, `LatLong`, `Utm`, `Fov`, `PixRange` (and optional `PixBox` if seen).
- Repeatable fields: multiple `Comment`, `PixRange`, `Tgt`, `TgtUpd`, `SenUpd`.

**Tests:** Field order independence where grammar allows; reject only **structurally** incomplete composites (e.g. `LatLong` with 7 tokens).

**M3 exit criteria:** `Field.value` typed via `newagt-core::value`; AGTJ 5-field `Utm` accepted when arity matches; `cargo test`, `cargo clippy -- -D warnings` clean. **Met.**

---

## M4 — Extension bucket (wild format) (done)

**Deliverables**

- First-class **`Keyword`** string records (parallel to `Comment`) at every container level PDF allows comments.
- **`UnknownStatement`**: unrecognized keyword + raw argument tokens until next sibling keyword.
- **`ExtensionRecord`**: optional typed hooks once corpus patterns are known.

**Policy:** Default **load success** with warnings for spec mismatches ([wild format](agt-real-world-format.md)).

**Tests:** AGTJ `Prototype.agt` snippet — multiple Comment/Keyword lines without truncation at 20 (Rust vectors, not AGTJ caps).

**Exit:** `parse_with_warnings`, `UnknownStatement` in AST, fixture `prototype_comment_keyword_burst.agt`. **Met.**

---

## M5 — AGTJ-aligned extensions (optional profile) (done)

**Deliverables**

- Parse **5-field `Utm`** (easting, northing, elevation, grid, datum) when present.
- **`Fov` inside `SenUpd`** (per prototypes).
- **`PixBox` on `Tgt`** when present.
- Document as **`profile: agtj`** in `format-notes.md`; strict PDF-only mode flag `--profile pdf1999`.

**Tests:** Diff `Prototype_SenSect.agt` / `Prototype_TgtSect.agt` against AGTJ list output where available.

**Exit:** `ParseProfile` + `parse_with_options`; CLI global `--profile`; fixtures `prototype_*_snippet.agt`. **Met.**

---

## M6 — `validate` subcommand (done)

**Deliverables**

- Severity: **error** = unbalanced braces, unterminated string, unrecoverable parse; **warning** = unknown keyword, PDF placement oddities, duplicate singletons; **info** = missing optional sections.
- JSON/text report; exit code 0 if loadable, non-zero only on structural failure.

**Exit:** `newagt validate file.agt` useful on Ken’s corpus. **Met.**

---

## M7 — `dump` and `to-json` (done)

**Deliverables**

- `dump`: indented tree, comments/keywords verbatim.
- `to-json`: stable schema version `newagt.schema.v1` — containers, scalars, extensions[], spans optional.

**Exit:** Round-trip read → JSON → (future write) not required yet. **Met** (`dump.rs`, `json_export.rs`, minimal `info`, CLI tests).

---

## M8 — Update / frame index (trainer-facing) (done)

**Deliverables**

- Align paired `SenUpd` / `TgtUpd` lists by order and/or **`Keyword`/`Comment` frame hints** (AGTJ `get_next_Upd.c` heuristics as **optional** index pass, provenance tagged).
- API: `frames()` iterator with sensor pose, target list, timestamps, pix refs.

**Depends:** Corpus validation; not PDF-specified.

**Exit:** `newagt-core::frames`, `document.frames(options)`, `newagt info` frame count, `newagt frames`. **Met.**

---

## M8b — Bounding boxes (trainer output) ✓

**Design:** [bounding-boxes.md](bounding-boxes.md) (PixBox authoritative vs `tgt.dat` + geometry).

**Deliverables** (done)

- `newagt-core::bbox`: `TgtDatDb`, `resolve_bboxes(doc, BBoxOptions) -> BBoxIndex` with provenance `PixBox` | `ScoreGeometry` | `TgtdbLinear`.
- **Authoritative path:** `PixBox` on `TargetEntry` when present (unless `--ignore-pix-box`).
- **Computed path:** Score/`Abuse_Tgt.c` atan sizing (default `--method score`); alternate `--method tgtdb` matches `tgtdb.py` `getRect` linear sizing.
- FOV cascade: target `Fov` → paired `SenUpd.Fov` → `SenSect.Fov` → `--fov-h` / `--fov-v`.
- CLI: `newagt bboxes --tgt-dat PATH --image-width W --image-height H [--fov-h H] [--fov-v V] [--method score|tgtdb]`.
- Fixture: `crates/newagt-core/tests/fixtures/tgt.dat.snippet`.

**Tests**

- PixBox fixture `prototype_tgt_sect_snippet.agt`; Score golden vs C truncation; tgtdb vs Python harness in `tests/scripts/tgtdb_get_rect.py`.

**Exit:** Trainers can emit inclusive `[x1,y1,x2,y2]` per target per frame from Rust.

---

## M9 — Python / PyO3 (done)

**Deliverables**

- `newagt` PyO3 module (`crates/newagt-py`, maturin / `pyproject.toml`): `parse`, `validate`, `frames`, `bboxes`, `to_json`.
- `python/newagt/trainers.py` columnar list helpers for PyTorch pipelines (no `torch` hard dep).
- COMMENT/KEYWORD → trainer columns stays in Python helpers; Rust preserves raw extension text.

**Exit:** `maturin develop --release`, `pytest tests/test_newagt_py.py` (or unittest). **Met.**

---

## M10 — Performance and packaging (done)

**Deliverables**

- Batch directory walk for `info` (find `.agt`, one summary line per file: path, frame count, sections present, parse ok/fail; single-file multi-line output unchanged).
- `cargo install --path crates/newagt-cli` documented in README; workspace publish metadata for `newagt-cli`.
- GitHub Actions CI (ubuntu): `cargo test`, `clippy -D warnings`, release CLI build, fixture smoke tests (no private corpus).

**Exit:** **Met.**

---

## CLI mapping

| Command | Milestone |
|---------|-----------|
| `info` | M7 / M8 frames / M10 batch ✓ |
| `frames` | M8 ✓ |
| `bboxes` | M8b ✓ |
| `dump` | M7 ✓ |
| `validate` | M6 ✓ |
| `to-json` | M7 ✓ |
| `pairs` | Post-M10 ✓ |

---

## Out of scope (unless Ken redirects)

- Adventure Game Toolkit (`*.DA1`, AGiliTy, `agtout`).
- Writing ARF imagery or transcoding rasters.
- Requiring strict PDF compliance for load success.
- Checking private corpus paths or scan artifacts into the public repo.

---

## Post-M10 (shipped and documented)

| Item | Status |
|------|--------|
| Authority frame count **N** | ✓ Global CLI `--frame-count`; Python `frame_count=`; Rust `FrameIndexOptions::expected_frame_count`. [arf-frame-authority.md](arf-frame-authority.md) |
| Classic ARF/AGT pairing scan | ✓ `newagt pairs --dataset-root` + `--missing agt\|arf\|both`; layout in [arf-frame-authority.md](arf-frame-authority.md) |
| Corpus directory `info` summary | ✓ Stderr batch line (`InfoBatchStats`); `--progress`, `--limit`; [corpus-testing.md](corpus-testing.md), [user-guide.md](user-guide.md) |
| Lexer / literal tolerance | ✓ CRLF fixtures; underscores in unknown keywords; signed integer literals for corpus `PixLoc` |
| User-facing docs | ✓ [user-guide.md](user-guide.md), [changelog.md](changelog.md), [README.md](README.md) index |

## Post-M10 (optional / future)

| Item | Notes |
|------|--------|
| ARF reader → frame count | Populate **N** from paired `.arf` when a reader exists; precedence TBD |
| Corpus benchmarking (private mount) | Offline only; `newagt info` with `--limit` or local TSV redirects — not CI |
| Full-tree timing / failure histograms | Scripts over local `info`/`validate` output |
| Trainer integration | PyO3 + [user-guide.md](user-guide.md) PyTorch sketch; in-repo `Dataset` only if needed |

---

## Immediate next step after this doc

M0–M10 and the post-M10 items above are **done**. Optional follow-on: ARF metadata integration, corpus-scale profiling on private data ([corpus-testing.md](corpus-testing.md)), and trainer-specific PyTorch wiring.
