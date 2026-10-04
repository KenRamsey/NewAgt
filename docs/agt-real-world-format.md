# AGT in the wild — spec vs reality

**Source:** Ken Ramsey (project context, 2026-10-02)  
**Audience:** NewAgt parser design — especially Rust core + trainer indexing

## Tortured history (summary)

AGT as used today **did not evolve as a clean implementation of one frozen spec**. Authors often **did not know the spec** or **could not express** what they needed in the “official” fields. They still stored ground-truth data inside AGT containers because those files were the **system of record** (today: **ground truth for image streams** and training pipelines).

The written spec is a **starting skeleton**, not a strict schema. A large fraction of files in the wild are **non-compliant** yet **loadable**, **useful**, and **must not be rejected** wholesale.

## COMMENT and KEYWORD — the main escape hatches

Inside AGT containers, **`COMMENT`** and **`KEYWORD`** are spec-defined hooks that became **general-purpose payload slots**:

| Spec intent (idealized) | What happened in practice |
|-------------------------|---------------------------|
| Human comments | Arbitrary text, **metadata**, labels, notes, sometimes **structured or binary-ish blobs** |
| Keywords / tags | **Innovation channel** — things that are **neither** comments **nor** keywords in the spec sense |

Users **jammed data** that “really belonged elsewhere” in the spec into COMMENT/KEYWORD because it was **available and worked** for their toolchain.

**Parser pitfall:** Treating COMMENT/KEYWORD as ignorable decoration or as a rigid grammar ** loses trainer-critical ground truth**.

## Design consequences for NewAgt

### 1. Parse model — preserve, don’t normalize away

- **Capture raw COMMENT/KEYWORD occurrences** (position, container context, payload bytes/text) even when semantics are unknown.
- **Structured fields first** where the spec or legacy AGTJ layout is clear; **extension bucket** for everything else.
- **Do not fail** the whole file on “invalid” COMMENT/KEYWORD usage unless the container is truly unreadable.

### 2. Validation is reporting, not gatekeeping

- **`validate` mode:** classify findings — spec mismatch, unknown extension pattern, suspicious overlap with other fields — **severity levels** (info / warning / error for structural corruption only).
- Distinguish **“non-compliant but loadable”** (common) from **“truncated / corrupt”** (rare).

### 3. Corpus-driven semantics (ongoing)

- Expect to **analyze many real AGT files** to infer **common usage patterns** (frequency, payload shapes, co-occurrence with image streams).
- Patterns become **documented heuristics** in `format-notes.md` and optional **named profiles** — not hard-coded as the 1992 spec alone.
- **Adaptation:** new files will invent new COMMENT/KEYWORD payloads; parser should **extend** (unknown-type registry, versioned sidecar index) rather than require a spec amendment.

### 4. Trainers (PyTorch)

- Image-stream ground truth may live in **non-spec COMMENT/KEYWORD** slots — indexing must expose **raw + best-effort interpreted** views (e.g. frame list from heuristics, with provenance: “pattern v3 from corpus 2026-02”).
- Training pipelines need **stable column names** in Python even when on-disk layout is messy — map messy → canonical in the **Python layer**, fed by rich native index.

### 5. Reference hierarchy

**Corpus** = highest. **`Agt-1992.pdf`** = skeleton. **AGTJ (C)** = sporadically updated to track real usage — accommodations **valuable, not absolute truth**. COMMENT/KEYWORD innovation = **first-class data**, not errors.

## Open work (later milestones)

- [ ] Inventory COMMENT/KEYWORD patterns in Ken’s corpus (and public samples where licensed).
- [ ] Define `newagt` extension record schema (unknown payload types, hashes, optional JSON sidecar for trainers).
- [ ] Compare AGTJ vs on-disk bytes for contested slots; log in `format-notes.md`.

## Related docs

- [Domain correction](agt-domain-correction.md) — imagery AGT vs wrong “Adventure Game Toolkit” research  
- [AGT parser plan](agt-parser-plan.md) — **obsolete** adventure-game plan (historical only)  
- [Format notes](format-notes.md) — triangulated layout deltas
