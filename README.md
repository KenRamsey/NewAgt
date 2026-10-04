# NewAgt

Rust tooling for **imagery ground truth AGT** — read, validate, and index AGT containers used in **image-stream / training** pipelines (Rust core, CLI, and Python bindings).

**AGT here is not the Adventure Game Toolkit** (text adventures, `*.DA1`, AGiliTy). See [docs/agt-domain-correction.md](docs/agt-domain-correction.md).

The on-disk format is documented starting from **`Agt-1992.pdf`** (legacy tree `NewC_r529/AGTJ`) and real training files. Many files are **non-compliant** with the written spec yet still **loadable**; **`COMMENT`** and **`KEYWORD`** often carry extra ground truth (see [docs/agt-real-world-format.md](docs/agt-real-world-format.md)).

## Documentation

**Start here → [docs/user-guide.md](docs/user-guide.md)** (install, CLI, Python API, examples with placeholders only).

| Doc | Topic |
|-----|--------|
| [docs/README.md](docs/README.md) | Full documentation index |
| [docs/user-guide.md](docs/user-guide.md) | CLI and Python user guide |
| [docs/changelog.md](docs/changelog.md) | Milestones M0–M10 and post-M10 |
| [docs/agt-implementation-plan.md](docs/agt-implementation-plan.md) | Implementation status |
| [docs/agt-format-spec-1992.md](docs/agt-format-spec-1992.md) | PDF format summary |
| [docs/corpus-testing.md](docs/corpus-testing.md) | Private corpus scans |
| [docs/arf-frame-authority.md](docs/arf-frame-authority.md) | Frame count **N**, dataset layout |
| [docs/bounding-boxes.md](docs/bounding-boxes.md) | Bbox design |

## Status

**M0–M10 complete**, plus post-M10: authority `--frame-count`, `pairs`, corpus batch summaries, lexer fixes (CRLF, underscores, signed integers). See [docs/changelog.md](docs/changelog.md).

Surfaces:

- **CLI:** `info`, `validate`, `dump`, `to-json`, `frames`, `bboxes`, `pairs`
- **Python:** `parse`, `validate`, `frames`, `bboxes`, `to_json` via PyO3
- **CI:** fixture tests only (no private corpus in git)

## Quick start

```bash
cargo install --locked --path crates/newagt-cli

export AGT_FILE='/path/to/example.agt'   # set locally; do not commit

newagt validate "$AGT_FILE"
newagt info "$AGT_FILE"
newagt to-json "$AGT_FILE" --spans -o /tmp/out.json
newagt frames "$AGT_FILE" --heuristic
```

Parse profile: `--profile agtj` (default) or `--profile pdf1999` on any subcommand.

More commands and Python examples: [docs/user-guide.md](docs/user-guide.md).

## Python

```bash
maturin develop --release
python -m pytest tests/test_newagt_py.py
```

```python
import newagt

doc = newagt.parse("/path/to/file.agt")
report = newagt.validate("/path/to/file.agt")
```

See [docs/user-guide.md](docs/user-guide.md) for `frames`, `bboxes`, `frame_count=`, and PyTorch sketches.

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## Fixtures

Integration fixtures live under `crates/newagt-core/tests/fixtures/`. For bulk real-world testing, use a local corpus root — see [docs/corpus-testing.md](docs/corpus-testing.md).

## License

MIT — see [LICENSE](LICENSE).
