# CORRECTION: AGT is not Adventure Game Toolkit

**Date:** 2026-10-04 (Ken Ramsey)  
**Impact:** Earlier Project research and `docs/agt-parser-plan.md` in the repo assumed the wrong format.

## What went wrong

Initial agents interpreted **“AGT”** as the **Adventure Game Toolkit** (Malmberg/Welch text adventures: `*.DA1`, `*.DA2`, AGiliTy, IF Archive games). That is **unrelated** to this project.

## What AGT actually is (this project)

- **Imagery ground truth format** — containers for **image-stream / training** ground truth, not interactive fiction.
- **Authoritative starting point (non-binding):** `Agt-1992.pdf` in the legacy tree  
  **`/home/atlas/Repos/NewC_r529/AGTJ/Agt-1992.pdf`**  
  (Earlier notes used `/home/ken/Repos/...`; use whichever path exists on the active machine.)
- The acronym **“AGT”** may **not be expanded** in the original spec — do not assume “Adventure Game Toolkit.”
- **AGTJ (C):** under **`/home/atlas/Repos/NewC_r529`** — sporadically updated to track **actual usage**; accommodations add value but are **not absolute truth** (same tier as spec: triangulate with corpus).
- **Real files:** many are **non-compliant** with the written spec; **`COMMENT`** and **`KEYWORD`** are often used as overflow data slots (see [AGT in the wild](agt-real-world-format.md)).

## What to stop using as spec

- AGiliTy, `agtout`, `*.DA1`–`*.DA6`, IF Archive AGT games, Magx, AGX interchange — **out of scope** unless Ken says otherwise.
- GitHub `NewAgt` repo **`docs/agt-parser-plan.md`** copied from early research — **treat as wrong** until rewritten from `Agt-1992.pdf` + corpus.

## Correct reference hierarchy (revised)

| Tier | Source | Role |
|------|--------|------|
| A | Real **imagery AGT** files Ken uses for training | Empirical ground truth |
| B | **`Agt-1992.pdf`** | Starting skeleton (not 100% accurate) |
| C | **NewC_r529 / AGTJ** code | Legacy behavior hints |
| D | Corpus pattern analysis | COMMENT/KEYWORD and extensions |

## Next work

1. Read **`Agt-1992.pdf`** and produce a **new** format summary (file layout, record types, image stream semantics).
2. Align **NewAgt** Rust parser milestones to **that** format — not DA1/AGiliTy milestones.
3. Fix **NewAgt** repo docs on GitHub after spec read (M0 scaffold code can stay; plan text must change).

## Consumers (unchanged)

PyTorch / Python trainers — fast native Rust core, thin Python API later.
