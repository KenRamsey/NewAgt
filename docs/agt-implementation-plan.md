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

## M2 — AST and PDF grammar parser

**Deliverables**

- Container AST: `Agt { prj, sen, tgt }` with nested `*Upd`, `Tgt`, `TgtSenRel`, `TgtAbs`.
- Parser driven by PDF yacc structure (§6.2): placement rules for `Tgt` under `TgtUpd`, etc.
- Scalar fields attached to owning container with **occurrence order** preserved.

**Tests:** Parse minimal `Agt { TgtSect { TgtUpd { Tgt { … } } } } }`; parse full Example 3 from PDF text fixture.

**Exit:** `parse(&str) -> Result<Document, ParseError>` with structured errors (line, expected token).

---

## M3 — Composite value parsing

**Deliverables**

- Typed values: `Time`, `PixLoc`, `LatLong`, `Utm`, `Fov`, `PixRange` (and optional `PixBox` if seen).
- Repeatable fields: multiple `Comment`, `PixRange`, `Tgt`, `TgtUpd`, `SenUpd`.

**Tests:** Field order independence where grammar allows; reject only **structurally** incomplete composites (e.g. `LatLong` with 7 tokens).

---

## M4 — Extension bucket (wild format)

**Deliverables**

- First-class **`Keyword`** string records (parallel to `Comment`) at every container level PDF allows comments.
- **`UnknownStatement`**: unrecognized keyword + raw argument tokens until next sibling keyword.
- **`ExtensionRecord`**: optional typed hooks once corpus patterns are known.

**Policy:** Default **load success** with warnings for spec mismatches ([wild format](agt-real-world-format.md)).

**Tests:** AGTJ `Prototype.agt` snippet — multiple Comment/Keyword lines without truncation at 20 (Rust vectors, not AGTJ caps).

---

## M5 — AGTJ-aligned extensions (optional profile)

**Deliverables**

- Parse **5-field `Utm`** (easting, northing, elevation, grid, datum) when present.
- **`Fov` inside `SenUpd`** (per prototypes).
- **`PixBox` on `Tgt`** when present.
- Document as **`profile: agtj`** in `format-notes.md`; strict PDF-only mode flag `--profile pdf1999`.

**Tests:** Diff `Prototype_SenSect.agt` / `Prototype_TgtSect.agt` against AGTJ list output where available.

---

## M6 — `validate` subcommand

**Deliverables**

- Severity: **error** = unbalanced braces, unterminated string, unrecoverable parse; **warning** = unknown keyword, PDF placement oddities, duplicate singletons; **info** = missing optional sections.
- JSON/text report; exit code 0 if loadable, non-zero only on structural failure.

**Exit:** `newagt validate file.agt` useful on Ken’s corpus.

---

## M7 — `dump` and `to-json`

**Deliverables**

- `dump`: indented tree, comments/keywords verbatim.
- `to-json`: stable schema version `newagt.schema.v1` — containers, scalars, extensions[], spans optional.

**Exit:** Round-trip read → JSON → (future write) not required yet.

---

## M8 — Update / frame index (trainer-facing)

**Deliverables**

- Align paired `SenUpd` / `TgtUpd` lists by order and/or **`Keyword`/`Comment` frame hints** (AGTJ `get_next_Upd.c` heuristics as **optional** index pass, provenance tagged).
- API: `frames()` iterator with sensor pose, target list, timestamps, pix refs.

**Depends:** Corpus validation; not PDF-specified.

---

## M9 — Python / PyO3 (later)

**Deliverables**

- `pip install`able module exposing parse, validate, frame index, columnar export for trainers.
- Map messy COMMENT/KEYWORD to canonical trainer columns in Python, not by lossy Rust normalization.

---

## M10 — Performance and packaging

**Deliverables**

- Batch directory walk for `info` (find `.agt`, summarize sections/updates/target counts).
- `cargo install` release binaries; CI on Ken’s worker with corpus smoke tests (fixtures not in public repo).

---

## CLI mapping (target)

| Command | Milestone |
|---------|-----------|
| `info` | M8/M10 — counts, paths, profile guess |
| `dump` | M7 |
| `validate` | M6 |
| `to-json` | M7 |

---

## Out of scope (unless Ken redirects)

- Adventure Game Toolkit (`*.DA1`, AGiliTy, `agtout`).
- Writing ARF imagery or transcoding rasters.
- Requiring strict PDF compliance for load success.

---

## Immediate next step after this doc

**Start M1** in `newagt-core`: module layout `lex.rs`, `token.rs`, `keyword.rs`, fixtures under `crates/newagt-core/tests/data/` from PDF examples + one AGTJ prototype header.
