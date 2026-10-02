# NewAgt — AGT Parser Implementation Plan

**Status:** Research complete, implementation not started (updated with Ken’s AGTJ context)  
**Repo:** `/workspace` (NewAgt) — empty except git (`main`, single commit `85d15a1` “Initialize project”, no tracked files)  
**Date:** 2026-10-02

---

## 0. Reference materials hierarchy

NewAgt has **no single authoritative binary spec**. Treat every written or legacy implementation source as a **hypothesis** until checked against real compiled games and (where possible) **`agtout`** / AGiliTy load behavior.

| Tier | Material | Role | Trust level |
|------|----------|------|-------------|
| **A — Empirical ground truth** | IF Archive AGT game bundles; **`agtout`** dumps; AGiliTy 1.1.2 load path | What parsers must match in practice | **Highest** for acceptance tests |
| **B — De facto reverse-engineered spec** | AGiliTy / Magx C sources (`agtread.c`, `agility.h`, …) | Best public description of on-disk layout across `aver` variants | **High**, but GPL reference only — clean-room reimplementation |
| **C — Legacy project hints (Ken)** | **`AGTJ`** in `NewC_r529` — old library source + **original AGT spec document (~1992)** | Historical intent, field names, early layout assumptions | **Starting point only — not 100% accurate** |
| **D — Community / ancillary** | intfiction.org threads, `agt.lse`, Pascal AGT 1.7 sources, Babel `agt.c` | Context, source language, AGX IFIDs | **Supplemental** |

**AGTJ usage rules**

- Use the **1992 spec** and **AGTJ library code** to accelerate field naming, document structure, and “expected” record layouts.
- **Do not** treat AGTJ or the 1992 doc as normative when they disagree with AGiliTy or with bytes in a real `*.DA*` / `*.D$$` file.
- When AGTJ, the 1992 spec, AGiliTy, and **`agtout`** disagree, **record all variants** in `docs/format-notes.md` and let **corpus tests** pick the behavior NewAgt ships (prefer matching tier A unless Ken directs otherwise).

**Parser design consequence:** readers should be **tolerant** (recover / partial parse where safe), attach **provenance** (which `aver` probe or size table applied), and surface **validation findings** (pointer out of range, record size mismatch, spec-vs-bytes notes) instead of assuming one frozen layout.

### Availability on this cloud VM

| Location | Result (2026-10-02) |
|----------|---------------------|
| Ken’s machine: `/home/ken/Repos/NewC_r529/AGTJ` | **Not present** on the cloud agent VM (expected — user-local path only) |
| `/workspace` search (`AGTJ`, `NewC_r529`) | **No copy** in the NewAgt repo or workspace |

**To use AGTJ in automated research or CI:** Ken should **clone or copy AGTJ into the workspace** (e.g. `/workspace/vendor/AGTJ` or a sibling repo in the Project environment), **or** run format-mining tasks on a **self-hosted worker** that mounts `NewC_r529`. Until then, plan milestones rely on tier **A + B** only; tier **C** is manual input from Ken’s tree.

---

## 1. What AGT Is

The **Adventure Game Toolkit (AGT)** is a late-1980s text-adventure authoring system (David Malmberg & Mark Welch). Authors write games in an AGT **source language** (rooms, nouns/objects, creatures, commands/metacommands, messages). The original toolchain was **Turbo Pascal** on DOS (`COMPILE.EXE` → data files, `RUN.EXE` / `MRUN.EXE` → play).

AGT was widely used on CompuServe; notable games include *Shades of Gray*, *The Multi-dimensional Thief*, and many others catalogued on [IFDB](https://ifdb.org/) and the [IF Archive AGT games index](https://www.ifarchive.org/indexes/if-archive/games/agt/).

AGT is **not** the unrelated QuickBASIC **ADVGEN** system (`.ADV` binaries). NewAgt targets **Malmberg/Welch AGT** only.

### AGT language variants (compatibility tiers)

Magx and AGiliTy treat these as one language family with version-specific quirks:

| Era | Names |
|-----|--------|
| Early | GAGS, “Classic” AGT, AGT 1.0 |
| Common | AGT 1.2–1.3.5, AGT 1.5 (“Hotel”), AGT 1.6, AGT 1.8.2–1.8.3 |
| Advanced | AGT Master’s Edition (ME) 1.0–1.7 |

There is **no single version byte** in the binary files. Interpreters **infer** `aver` (compatibility profile) and `ver` (size class: small / large / master’s) by probing file layout — see AGiliTy `try_read_da1()` in `agtread.c`. The **1992 AGT spec** (in Ken’s AGTJ tree) may describe an idealized layout; **AGTJ code** may reflect an older subset — both can diverge from games built with later AGT / ME builds (see §0).

---

## 2. File Types Overview

A compiled AGT game is a **multi-file bundle** sharing a basename (e.g. `ELECTRA.*`). AGiliTy documents these components:

### 2.1 Compiled binary / data files (primary parser targets)

| Extension | Role | Encoding |
|-----------|------|----------|
| **`.DA1`** | General game metadata: start room, score, object ID ranges, dictionary hooks, pointers into descriptions, questions, subsystem sizes | **Line-oriented text** (`TRUE`/`FALSE`, decimal numbers, `PTR` start/len pairs, inline text lines) |
| **`.DA2`** | Room records (exits, flags, light, points, picture refs, …) | **Fixed-size binary records** (220 bytes/room in AGiliTy `FRS_ROOM`) |
| **`.DA3`** | Noun/object records | **Fixed-size binary records** (~310 bytes; size varies by `aver`) |
| **`.DA4`** | Creature records (optional if game has no creatures) | Binary records |
| **`.DA5`** | Command / metacommand bytecode (optional) | Binary records |
| **`.DA6`** | Master’s Edition assets: picture/sound/font names, PIX data, extended opcodes (optional) | Binary |
| **`.D$$`** | Descriptions and messages (room text, object text, intro, errors, …) | **Binary**, often **encrypted** Pascal-style length-prefixed strings; encryption detected heuristically |

### 2.2 Auxiliary text files (secondary parse targets)

| Extension | Role |
|-----------|------|
| **`.TTL`** | Title screen: game name, author, copyright (plain text) |
| **`.INS`** | Instruction / help text shown to player |
| **`.VOC`** | Menu vocabulary (built-in verb lists) |
| **`.OPT`** | Optional game options |
| **`.CFG`** | Interpreter configuration (`NO_*` / purity flags, bold mode, etc.) |
| **`.HNT`** | Hints (some games) |

### 2.3 Unified format: `.AGX`

**AGX** (“Adventure Game eXecutable”) is Robert Masenten’s **single-file** packaging of all AGT components, produced by **`agt2agx`** or **Magx**. It is the de facto modern interchange format for AGiliTy.

- **Magic (bytes 0–3):** `58 C7 C1 51`
- **Version bytes (4–7):** owner + version + extension owner + extension (vanilla AGX: `'R', 2, 'R', 1`)
- **IF Archive Babel** module (`iftechfoundation/babel-tool` `agt.c`) identifies AGX for Treaty of Babel IFIDs (`AGT-%05d-%08X` from description block at offset read from word at +32)

AGiliTy explicitly warns against casual AGX format changes (`agxfile.c` header comment).

### 2.4 Authoring source (out of scope for v1 unless requested)

Original AGT **source** is not one extension; sample game **CAVE** uses multiple AGT source files compiled by `COMPILE CAVE`. Syntax is documented in **AGT-DOC.TXT** (Master’s Edition manual) and approximated in the **LSE/ELSE** template `agt.lse` ([IF Archive](http://ftp.funet.fi/pub/misc/ifarchive/programming/editors/agt.lse)). **Magx** compiles AGT-family source directly to **AGX** ([Magx on IF Archive](https://www.ifarchive.org/if-archive/programming/agt/magx/)).

---

## 3. Format Realities (Why This Is Hard)

1. **No single published binary spec that matches all games.** The [intfiction.org thread (2024)](https://intfiction.org/t/specification-for-agt-binary-file-formats/68278) confirms community interest but no maintained authoritative document. Ken’s **~1992 AGT spec (AGTJ)** and **AGTJ library** are valuable **hints** but are **not guaranteed accurate** for every variant. The best public **behavioral** reference remains **AGiliTy 1.1.2** ([DavidKinder/Windows-AGiliTy](https://github.com/DavidKinder/Windows-AGiliTy), [IF Archive `agil112src.zip`](https://www.ifarchive.org/if-archive/programming/agt/agility/)). NewAgt resolves conflicts via **§0** (empirical corpus + `agtout`).

2. **Spec / code / bytes can disagree.** Examples to expect: record sizes in old docs vs AGiliTy `FRS_*` constants; fields added in Master’s Edition; encryption heuristics for `D$$`. The parser should implement **variant-aware decoding** and log **discrepancy reports** (e.g. “1992 spec field X at offset Y vs AGiliTy layout vs actual file length”).

3. **Version detection is heuristic.** `ver` ≈ size class (0 unknown, 1 small, 2 big, 4 master’s); `aver` ≈ compatibility profile (AGiliTy defines `AGT10` … `AGTME16` in `agility.h`). `agtout` prints a disassembly after guessing. AGTJ may assume fewer variants — do not collapse detection to a single spec-era model.

4. **Mixed encodings.** DA1 is textual; DA2–DA6 and D$$ are binary; D$$ strings use AGT-specific encryption (`convert_agt_descr()` in `agtread.c`).

5. **Platform / port variance.** Games were built on PC, Mac, Atari ST, Amiga; cross-platform compatibility is a known concern (same thread). Endianness is mostly **little-endian 16-bit integers** in AGiliTy’s readers; validate against corpus.

6. **Command opcodes** vary by `aver`; AGiliTy normalizes via `fixcmd()` / opcode tables. Cross-check opcode tables against AGTJ only where AGTJ actually implements command reading.

---

## 4. Existing Tools (Legacy Baseline)

| Tool | Role | Notes |
|------|------|-------|
| **AGT COMPILE / RUN** (DOS) | Original compiler & interpreter | Turbo Pascal; version-specific |
| **AGiliTy** | Universal C interpreter | Reads multi-file AGT + AGX; maintained by David Kinder (1.1.2) |
| **`agtout`** | Disassembler / inspector | Ships with AGiliTy; golden reference for dumps |
| **`agt2agx`** | AGT bundle → AGX | Conversion reference |
| **Magx** | Source → AGX compiler | C source on IF Archive |
| **Babel `agt.c`** | AGX IFID for Treaty of Babel | Minimal read of AGX header/metadata |
| **`agt.lse`** | Text editor syntax for AGT source | Not binary format |
| **AGTJ (`NewC_r529/AGTJ`)** | Ken’s legacy library + ~1992 spec | **Non-authoritative** reference; triangulate with AGiliTy + corpus |

**Replacement goal:** A **library-first, testable parser** that exposes structured game data (and validation errors) without running a full interpreter — optionally emitting JSON, AGX, or human-readable reports comparable to `agtout`.

---

## 5. Parser Goals (Recommended Scope)

### Phase A — Read & model (MVP)

- [ ] Accept a **game directory** or basename; discover required files (DA1, DA2, DA3, D$$ minimum).
- [ ] Parse **DA1** into a typed header model (rooms/nouns/creatures ranges, flags, pointers).
- [ ] Parse **DA2/DA3/DA4/DA5** records into structures aligned with AGiliTy’s `room_rec`, `noun_rec`, etc.
- [ ] Parse **D$$**: decrypt when needed, extract description blocks referenced by pointers.
- [ ] Parse auxiliary **TTL, INS, VOC, OPT, CFG** as optional attachments.
- [ ] Report **`ver` / `aver` guess** with confidence / which probe path matched (mirror `try_read_da1` retry logic conceptually).

### Phase B — Validate

- [ ] Cross-check DA1 ranges vs actual record counts and file sizes.
- [ ] Validate pointer ranges against D$$ size.
- [ ] Flag unknown opcodes in DA5 (non-fatal warnings).
- [ ] **`validate` mode:** emit structured findings for spec/code mismatches (AGTJ or 1992 doc vs AGiliTy layout vs bytes) without failing the whole parse when the game is still playable under AGiliTy.
- [ ] Optional: diff structural hash against AGiliTy load (integration test).
- [ ] When AGTJ is available in workspace: optional **`newagt diff-spec`** (or test-only harness) comparing AGTJ struct assumptions to AGiliTy constants — output markdown for `docs/format-notes.md`.

### Phase C — Convert & emit

- [ ] **JSON** export (stable schema versioned as `newagt.schema.v1`) for tools/IF research.
- [ ] **AGX writer** (stretch): port logic from `agt2agx.c` / `agxfile.c` — high value for portability.
- [ ] CLI **`dump`** subcommand mirroring **`agtout`** summary sections for regression.

### Non-goals (initially)

- Executing metacommands / playable interpreter (AGiliTy remains the reference runtime).
- Magx-compatible **source** parser (large separate grammar project).
- GUI (unless later requested).

---

## 6. Recommended Stack

**Primary recommendation: Rust**

| Concern | Choice |
|---------|--------|
| Language | **Rust** (2021 edition) |
| CLI | `clap` (derive) |
| Errors | `thiserror` + `mreport`/`color-eyre` for CLI |
| Serialization | `serde` + `serde_json` for JSON export |
| Binary I/O | `byteorder` + explicit readers (or `nom` only where it helps) |
| Text DA1 | Line-based parser with borrowed `str` (no regex needed) |
| Tests | `insta` snapshots + corpus under `tests/fixtures/` |
| Fuzzing (later) | `cargo fuzz` on record readers |

**Why Rust:** Strong typing for many record layouts, easy CLI/library crate split (`newagt-core` + `newagt-cli`), good fit for IF preservation tooling, straightforward WASM path if a browser inspector is ever wanted.

**Alternative:** **TypeScript (Node)** — faster iteration and JSON-native, weaker for binary correctness unless very careful; good if the user prefers npm ecosystem.

**Alternative:** **Python 3** — excellent for research scripts; weaker for performance and AGX binary writer safety.

**Reference implementation language:** AGiliTy is **ANSI C** — treat it as the **de facto behavioral spec**, not as code to embed (GPL-2). **AGTJ** and the **1992 document** are **additional hints** (tier C in §0), not a license to copy blindly. NewAgt should be a **clean-room reader** informed by AGiliTy + empirical games, with **corpus tests** rather than copying C or AGTJ verbatim.

---

## 7. Proposed Repository Layout

```
NewAgt/
├── README.md
├── LICENSE
├── Cargo.toml                 # workspace
├── crates/
│   ├── newagt-core/           # library: parse, model, validate
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── game.rs        # GameBundle, GameMetadata
│   │   │   ├── version.rs     # ver/aver detection
│   │   │   ├── da1.rs
│   │   │   ├── da2.rs … da6.rs
│   │   │   ├── dss.rs         # D$$ descriptions + crypto
│   │   │   ├── aux.rs         # TTL, INS, VOC, OPT, CFG
│   │   │   ├── agx.rs         # phase C
│   │   │   ├── opcodes.rs     # command decoding tables
│   │   │   └── json_export.rs
│   │   └── tests/
│   │       ├── corpus/        # tiny games + IF Archive samples (git-lfs or fetch script)
│   │       └── agtout_snapshots/
│   └── newagt-cli/
│       └── src/main.rs        # newagt info|dump|validate|to-json|to-agx
├── docs/
│   ├── format-notes.md        # triangulated notes: corpus + AGiliTy + AGTJ/spec deltas
│   └── agt-parser-plan.md     # this document (copy or link from Project store)
├── vendor/                    # optional; not committed unless Ken adds submodule
│   └── AGTJ/                  # clone of NewC_r529/AGTJ for spec/code mining
└── scripts/
    └── fetch-ifarchive-fixtures.sh
```

---

## 8. Milestones

| # | Milestone | Deliverable | Success criteria |
|---|-----------|-------------|------------------|
| M0 | Scaffold | Cargo workspace, CI (`cargo test`, `clippy`), README | Builds on Linux/macOS/Windows |
| M1 | DA1 + discovery | Load header, object ID ranges | Parses DA1 from CAVE sample + 2 IF Archive games |
| M2 | DA2 + DA3 | Rooms & nouns | Record counts match DA1; fields match `agtout` dump |
| M3 | D$$ | Decrypted descriptions | Intro + room 1 text matches AGiliTy |
| M4 | DA4 + DA5 | Creatures + commands | Opcode decode for one classic + one ME game |
| M5 | Aux files | TTL/INS/VOC/CFG | Round-trip read; optional JSON section |
| M6 | Validation CLI | `newagt validate` | Actionable errors on truncated/corrupt fixtures |
| M7 | JSON export | Stable schema | Documented schema; snapshot tests |
| M8 | AGX (optional) | `to-agx` | Byte-identical or functionally equivalent to `agt2agx` on corpus |
| M9 | DA6 / ME assets | Picture/sound name extraction | Matches AGiliTy for *Shades of Gray* or similar |

---

## 9. Test Strategy

1. **Corpus (tier A):** Start with IF Archive games tagged AGT (small classics first). Store **hashes**, not necessarily full games in git (scripted download + license respect).
2. **Golden output:** Run upstream **`agtout`** (build AGiliTy from source in CI or pin release) → compare normalized dumps to `newagt dump`.
3. **Unit tests:** Fixed byte arrays for individual record types — primary transcription from **AGiliTy** (`FRS_ROOM = 220`, `FRS_NOUN = 310`, …); where **AGTJ** or the **1992 spec** differs, add **explicit test cases** documenting both interpretations and which one the corpus requires.
4. **Version matrix:** At least one game per major `aver` bucket (AGT135, AGT15, AGTME10, AGTME15).
5. **Triangulation (when AGTJ available):** For each file type (DA1…D$$), maintain a short delta table in `format-notes.md`: *1992 spec says* / *AGTJ code does* / *AGiliTy does* / *verified game(s)*.

---

## 10. Key Implementation References

| Resource | URL / path |
|----------|------------|
| **AGTJ** (legacy library + ~1992 AGT spec) | Ken: `/home/ken/Repos/NewC_r529/AGTJ` — **not on cloud VM**; clone to `/workspace/vendor/AGTJ` or use self-hosted worker |
| AGiliTy source (Windows port, Generic/) | https://github.com/DavidKinder/Windows-AGiliTy/tree/master/Generic |
| AGiliTy IF Archive | https://www.ifarchive.org/if-archive/programming/agt/agility/ |
| Magx compiler | https://www.ifarchive.org/if-archive/programming/agt/magx/ |
| AGT games | https://www.ifarchive.org/indexes/if-archive/games/agt/ |
| Binary format discussion | https://intfiction.org/t/specification-for-agt-binary-file-formats/68278 |
| Babel AGX IFID | https://github.com/iftechfoundation/babel-tool/blob/main/agt.c |
| AGT 1.7 Pascal source (compiler logic, not binary layout) | https://pascal.sources.ru/gamestxt/agtsrc.htm |
| AGT source editor grammar | http://ftp.funet.fi/pub/misc/ifarchive/programming/editors/agt.lse |

**Primary spec files to port conceptually (tier B):** `agtread.c`, `auxfile.c`, `agxfile.c`, `agt2agx.c`, `agility.h`.  
**Secondary hints (tier C):** AGTJ sources + 1992 spec document — mine for naming and layout ideas; verify every struct offset against tier A/B.

---

## 11. Open Questions for Ken

0. **AGTJ access:** Can you add AGTJ as a git submodule, tarball, or copy under `vendor/AGTJ` in NewAgt (or grant a self-hosted path)? Which filename is the 1992 spec inside AGTJ?
1. **Primary output:** JSON only, AGX conversion, both, or something else (e.g. Glulx-adjacent IR, Inform-style AST)?
2. **Language lock-in:** Is **Rust** acceptable, or prefer TypeScript/Python for your workflow?
3. **Scope of “parser”:** Compiled **multi-file AGT only**, or also **AGX**, or eventually **AGT source** (.TTL-style authoring files)?
4. **License:** OK with **MIT/Apache-2.0** for NewAgt (keeping AGiliTy as GPL reference only, no code paste)?
5. **Corpus:** Can we depend on downloading IF Archive games in CI, or will you provide a fixed fixture set locally?
6. **Compatibility target:** Must output match **`agtout`** exactly, or is semantic equivalence enough?
7. **Master’s Edition / DA6:** Required for v1, or defer graphics/sound metadata?
8. **CLI name:** `newagt`, `agtparse`, or match a legacy tool name (`agtout` compatibility mode)?

---

## 12. Proposed README (repo is empty — not committed)

When scaffolding begins, replace the empty repo with something like:

```markdown
# NewAgt

Modern tooling for **Adventure Game Toolkit (AGT)** game files — read, validate, and convert legacy AGT data without DOS-era utilities.

AGT games ship as a bundle of files (`*.DA1`, `*.DA2`, `*.DA3`, optional `*.DA4`–`*.DA6`, and encrypted messages in `*.D$$`). NewAgt parses these into a structured model for preservation, analysis, and conversion (JSON and, eventually, AGX).

This project is a clean-room implementation informed by the community’s [AGiliTy](https://github.com/DavidKinder/Windows-AGiliTy) interpreter sources and validated against real game files; legacy AGTJ / 1992 documentation is used only as non-authoritative hints. It is not a fork of AGiliTy or AGTJ.

## Status

Early development — see [docs/agt-parser-plan.md](docs/agt-parser-plan.md).

## Quick start (once implemented)

```bash
cargo build --release
newagt info /path/to/game/ELECTRA
newagt dump /path/to/game/ELECTRA
newagt validate /path/to/game/ELECTRA
newagt to-json /path/to/game/ELECTRA -o electra.json
```

## Fixtures

Download sample AGT games from the [IF Archive AGT collection](https://www.ifarchive.org/indexes/if-archive/games/agt/) for local testing.

## License

TBD (see open questions).
```

---

## 13. Repo Inspection Summary

| Item | State |
|------|--------|
| Branch | `main`, clean, tracks `origin/main` |
| Commits | 1 — “Initialize project” (no files in tree) |
| README | **Absent** in working tree |
| Implementation | **None** |

**Next coordinator step:** Confirm stack and scope (Section 11), then M0 scaffold on branch `cursor/agt-parser-core-1f50` with DA1 reader (M1).
