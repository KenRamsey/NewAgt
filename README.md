# NewAgt

Rust tooling for **imagery ground truth AGT** — read, validate, and index AGT containers used in **image-stream / training** pipelines (PyTorch and Python consumers planned).

**AGT here is not the Adventure Game Toolkit** (text adventures, `*.DA1`, AGiliTy). See [docs/agt-domain-correction.md](docs/agt-domain-correction.md).

The on-disk format is documented starting from **`Agt-1992.pdf`** (legacy tree `NewC_r529/AGTJ`) and Ken’s real training files. Many files are **non-compliant** with the written spec yet still **loadable**; **`COMMENT`** and **`KEYWORD`** often carry extra ground truth (see [docs/agt-real-world-format.md](docs/agt-real-world-format.md)). Legacy **AGTJ** code under `NewC_r529` is a loose hint only — not authoritative.

## Status

Early development through **M10** (Rust CLI + **`newagt` Python module** via PyO3: parse, validate, frames, bboxes; batch `info` on directories; GitHub Actions CI; plus `newagt dump`, `to-json`, `info`, `frames`, `bboxes` CLI and `Document::frames` / `Document::bboxes` in `newagt-core`).

- Format summary (from `Agt-1992.pdf`): [docs/agt-format-spec-1992.md](docs/agt-format-spec-1992.md)
- Implementation milestones: [docs/agt-implementation-plan.md](docs/agt-implementation-plan.md)
- Domain: [docs/agt-domain-correction.md](docs/agt-domain-correction.md)
- Wild-format notes: [docs/agt-real-world-format.md](docs/agt-real-world-format.md)
- Old adventure-game plan (obsolete): [docs/agt-parser-plan.md](docs/agt-parser-plan.md)

## Installing

Build and install the `newagt` CLI from this repository (requires Rust ≥ 1.74):

```bash
# from repo root — installs the workspace binary crate
cargo install --locked --path crates/newagt-cli

# or build a release binary without installing
cargo build --release -p newagt-cli
# binary: target/release/newagt
```

## Quick start

```bash
cargo build --release
cargo run --release -- validate /path/to/file.agt
cargo run --release -- dump /path/to/file.agt
cargo run --release -- to-json /path/to/file.agt
cargo run --release -- to-json --spans /path/to/file.agt -o out.json
cargo run --release -- info /path/to/file.agt
cargo run --release -- info /path/to/agt/corpus/
```

| Command | Purpose |
|---------|---------|
| `validate` | Parse file and print a text or JSON validation report (exit 0 if loadable) |
| `dump` | Indented tree of containers and fields (exit 0 on successful parse) |
| `to-json` | Export `newagt.schema.v1` JSON; optional `--spans` for source locations |
| `info` | Single file: path, profile, section/update/target counts; directory: one tab-separated summary line per `.agt` (walks subdirectories by default; `--limit N` caps batch size) |

Use `--profile agtj` (default) or `--profile pdf1999` on any subcommand.

## Python (PyO3 / maturin)

Install [maturin](https://www.maturin.rs/) and a Python ≥3.9 venv, then from the repo root:

```bash
# editable install (debug, fast iteration)
maturin develop

# release build (matches trainer performance expectations)
maturin develop --release
# or: maturin build --release
```

On Python 3.14+ you may need `PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1` (the crate builds with `abi3-py39`).

```bash
PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1 maturin develop --release
```

```python
import newagt

doc = newagt.parse("/path/to/file.agt")  # or inline AGT source string
print(doc.summary())
report = newagt.validate("/path/to/file.agt")
frames = newagt.frames("/path/to/file.agt", heuristic=False)
boxes = newagt.bboxes(
    "/path/to/file.agt",
    image_width=640,
    image_height=480,
    fov_h=30.0,
    fov_v=20.0,
    tgt_dat="/path/to/tgt.dat",
    method="score",
)
```

Columnar list helpers (no PyTorch dependency): `python/newagt/trainers.py`.

```bash
python -m pytest tests/test_newagt_py.py
# or: python -m unittest tests.test_newagt_py
```

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## Fixtures

Use Ken’s imagery AGT corpus locally for integration tests (not bundled in this repo).

### Testing against your corpus

For stress-testing directory walks and real-world parse tolerance, point the CLI at your own bulk-storage tree (set `$CORPUS_ROOT` locally—never commit mount paths). See [docs/corpus-testing.md](docs/corpus-testing.md) for access checks, `newagt info` batch mode, `--limit` samples, and privacy rules.


## License

MIT — see [LICENSE](LICENSE).
