# NewAgt user guide

NewAgt reads **imagery ground truth AGT** — ASCII container files used in sensor/target training pipelines. Real corpora are often **wild**: they load successfully while diverging from the 1992 PDF yacc spec; `Comment` and `Keyword` lines frequently carry extra labels and frame hints.

The project ships three surfaces on one **Rust core** (`newagt-core`):

| Surface | Role |
|---------|------|
| **`newagt` CLI** | Inspect, validate, export JSON, frame index, bboxes, dataset pairing |
| **`newagt` Python package** | PyO3 bindings: `parse`, `validate`, `frames`, `bboxes`, `to_json` |
| **Library** | Same logic for custom Rust tools |

AGT here is **not** the Adventure Game Toolkit (`*.DA1`, AGiliTy). See [agt-domain-correction.md](agt-domain-correction.md).

---

## Install

### CLI (Rust)

Requires Rust ≥ 1.74. From a clone of this repository:

```bash
cargo install --locked --path crates/newagt-cli
```

Or build without installing:

```bash
cargo build --release -p newagt-cli
# binary: target/release/newagt
```

### Python (maturin)

Install [maturin](https://www.maturin.rs/) and use Python ≥ 3.9. From the repo root:

```bash
python -m venv .venv
source .venv/bin/activate
pip install maturin

# editable debug build (fast iteration)
maturin develop

# release build (closer to trainer performance)
maturin develop --release
```

On Python 3.14+, you may need:

```bash
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --release
```

Install the package from the repo (not PyPI yet):

```bash
pip install .
# or after maturin develop, `import newagt` works in that venv
```

Run Python smoke tests:

```bash
python -m pytest tests/test_newagt_py.py
# or: python -m unittest tests.test_newagt_py
```

Optional helpers (no PyTorch dependency): `python/newagt/trainers.py`.

---

## Parse profiles

Global flag on every CLI subcommand: **`--profile`**.

| Profile | Value | Use when |
|---------|-------|----------|
| **AGTJ-aligned** (default) | `agtj` | Ken’s training files and AGTJ-shaped extensions (`Fov` in `SenUpd`, `PixBox` on `Tgt`, 5-field `Utm`) |
| **PDF strict** | `pdf1999` | yacc-only placements from `Agt-1992.pdf`; AGTJ-only constructs become unknown statements with warnings |

```bash
newagt validate "$AGT_FILE" --profile pdf1999
```

Python: `newagt.parse(..., profile="agtj")`, `newagt.validate(..., profile="pdf1999")`, `newagt.to_json(..., profile="agtj")`.

Details: [format-notes.md](format-notes.md), [agt-format-spec-1992.md](agt-format-spec-1992.md).

---

## Optional authority frame count

Trainer timelines sometimes need a fixed length **N** even when `SenUpd` / `TgtUpd` lists differ in length. NewAgt does **not** read ARF imagery yet; you supply **N** externally.

**CLI:** global **`--frame-count N`** on `info`, `frames`, `validate`, and `bboxes`.

**Python:** keyword **`frame_count=None`** on `validate`, `frames`, and `bboxes`.

Full behavior (padding, capping, warnings): [arf-frame-authority.md](arf-frame-authority.md).

```bash
newagt frames "$AGT_FILE" --heuristic --frame-count 120
newagt validate "$AGT_FILE" --frame-count 120
```

---

## CLI reference

Set placeholders in your shell (examples only — do not commit real mount paths):

```bash
export AGT_FILE='/path/to/example.agt'
export CORPUS_ROOT='/path/to/agt/tree'
export DATASET_ROOT='/path/to/classic/dataset'
export TGT_DAT='/path/to/tgt.dat'
```

### `info` — summary for one file or a directory batch

**Single file:** multi-line summary (path, profile, section/update/target counts, frame count).

**Directory:** one **tab-separated line per `.agt`** on stdout; after the batch, a **summary line on stderr** (files scanned, parse ok/fail, total frames, warning totals when non-zero).

| Flag | Applies to | Meaning |
|------|------------|---------|
| `--recursive` | directory | Walk subdirectories (default: true) |
| `--limit N` | directory | Process at most N files after sorted walk |
| `--progress [N]` | directory | Progress on stderr every N files (default N=100 if flag given alone) |
| `--frame-count N` | both | Authority frame count for frame totals (see above) |

```bash
# one file
newagt info "$AGT_FILE"

# corpus sample (see corpus-testing.md)
newagt info "$CORPUS_ROOT" --limit 50

# long scan: TSV on stdout, progress + batch summary on stderr
newagt info "$CORPUS_ROOT" --progress > ~/agt-scan.tsv 2> ~/agt-scan.log

# top level only
newagt info "$CORPUS_ROOT/subtree" --recursive=false
```

See [corpus-testing.md](corpus-testing.md) for privacy rules and offline filtering.

### `validate` — structural loadability report

Exit **0** if the file is **loadable** (warnings allowed); **non-zero** on hard parse/structure failure.

```bash
newagt validate "$AGT_FILE"
newagt validate "$AGT_FILE" --format json
newagt validate "$AGT_FILE" --frame-count 100 --format json
```

### `dump` — human-readable parse tree

```bash
newagt dump "$AGT_FILE"
```

### `to-json` — `newagt.schema.v1` export

```bash
newagt to-json "$AGT_FILE"
newagt to-json "$AGT_FILE" --spans -o /tmp/out.json
```

### `frames` — trainer frame index (one line per frame)

Pair `SenUpd` / `TgtUpd` by list order; optional AGTJ-style hints from `Keyword` / `Comment`.

```bash
newagt frames "$AGT_FILE"
newagt frames "$AGT_FILE" --heuristic
newagt frames "$AGT_FILE" --heuristic --frame-count 64
```

Warnings (if any) print before frame lines.

### `bboxes` — per-target pixel boxes (JSON on stdout)

Uses **PixBox** on the target when present; otherwise computes from geometry, image size, FOV, and optional **`tgt.dat`**.

| Flag | Meaning |
|------|---------|
| `--tgt-dat PATH` | Target dimension database |
| `--image-width`, `--image-height` | Raster size in pixels |
| `--fov-h`, `--fov-v` | Horizontal/vertical FOV in degrees when not in AGT |
| `--method score\|tgtdb` | Computed box math (default: `score`) |
| `--ignore-pix-box` | Recompute even when `PixBox` exists |
| `--heuristic` | AGTJ frame pairing for index |
| `--frame-count N` | Authority timeline length |

```bash
newagt bboxes "$AGT_FILE" \
  --tgt-dat "$TGT_DAT" \
  --image-width 640 --image-height 480 \
  --fov-h 30 --fov-v 20 \
  --method score \
  --heuristic
```

Design and provenance fields: [bounding-boxes.md](bounding-boxes.md).

### `pairs` — classic ARF/AGT basename inventory

Expects **one subdirectory per sensor**, each with `arf/` and `agt/` folders; pairs by **stem** (filename without extension). Does not parse file contents.

```bash
newagt pairs --dataset-root "$DATASET_ROOT"
newagt pairs --dataset-root "$DATASET_ROOT" --missing both
```

Output columns (tab-separated): `sensor`, `arf_path`, `agt_path`, `stem`. Paths are **your machine’s runtime output** — keep them out of git.

Layout details: [arf-frame-authority.md](arf-frame-authority.md#dataset-layout-classic-arfagt-pairing).

---

## Python API

```python
import os
import newagt

agt_path = os.environ["AGT_FILE"]  # export AGT_FILE locally first
tgt_dat = os.environ.get("TGT_DAT")

# parse: file path OR inline AGT source string
doc = newagt.parse(agt_path)
print(doc.profile, doc.path)
summary = doc.summary()  # dict: sections, updates, targets counts

# validate / frames / bboxes / to_json: filesystem paths only
report = newagt.validate(agt_path, profile="agtj", frame_count=None)
frames = newagt.frames(agt_path, heuristic=True, frame_count=64)
boxes = newagt.bboxes(
    agt_path,
    image_width=640,
    image_height=480,
    fov_h=30.0,
    fov_v=20.0,
    tgt_dat=tgt_dat,
    method="score",
    heuristic=False,
    frame_count=None,
)
payload = newagt.to_json(agt_path, profile="agtj", spans=False)
```

### Return shapes (brief)

**`doc.summary()`** — dict with `profile`, optional `path`, nested `sections` (`prj_sect`, `sen_sect`, `tgt_sect`), `updates` (`sen_upd`, `tgt_upd`), `targets` (count).

**`validate(...)`** — dict mirroring the CLI JSON report: at least `loadable` (bool) and `entries` (list of severity/message records).

**`frames(...)`** — list of frame dicts: `index`, `pairing`, optional `sensor` / `target` objects with pose, times, and nested target lists (includes `PixLoc`, types, etc., when present).

**`bboxes(...)`** — list of records with `frame_index`, target metadata, and nested `bbox` (`x1`, `y1`, `x2`, `y2`, `provenance` such as `pix_box`, `score_geometry`, or `tgtdb_linear`).

**`to_json(...)`** — parsed **`newagt.schema.v1`** tree (containers, typed fields, extensions); optional span metadata when `spans=True`.

### PyTorch loader sketch (no extra deps in NewAgt)

NewAgt stops at lists and dicts; wire your own `Dataset`:

```python
import os
from pathlib import Path
import newagt
from newagt.trainers import flatten_frames, flatten_bboxes

agt_path = Path(os.environ["AGT_FILE"])
tgt_dat = os.environ["TGT_DAT"]
frame_rows = flatten_frames(newagt.frames(str(agt_path), heuristic=True))
bbox_rows = flatten_bboxes(
    newagt.bboxes(
        str(agt_path),
        640,
        480,
        30.0,
        20.0,
        tgt_dat=tgt_dat,
        method="score",
    )
)

# In user code:
# import torch
# class AgtFrameDataset(torch.utils.data.Dataset):
#     def __init__(self, agt_paths, ...):
#         ...
#     def __getitem__(self, i):
#         frames = newagt.frames(str(self.agt_paths[i]), heuristic=True)
#         return flatten_frames(frames)
```

Pair with ARF imagery using paths from `newagt pairs` locally; ARF reading is not implemented in NewAgt yet.

---

## Related documentation

| Doc | Topic |
|-----|--------|
| [agt-format-spec-1992.md](agt-format-spec-1992.md) | PDF format summary |
| [agt-real-world-format.md](agt-real-world-format.md) | Wild-format / COMMENT-KEYWORD patterns |
| [arf-frame-authority.md](arf-frame-authority.md) | Frame count **N**, classic dataset layout |
| [corpus-testing.md](corpus-testing.md) | Private corpus scans with `info` |
| [bounding-boxes.md](bounding-boxes.md) | PixBox vs computed geometry |
| [agt-implementation-plan.md](agt-implementation-plan.md) | Milestone status |
| [changelog.md](changelog.md) | High-level release milestones |

---

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

Fixtures for CI live under `crates/newagt-core/tests/fixtures/`. Large corpora stay on your storage; see [corpus-testing.md](corpus-testing.md).
