# NewAgt

Rust tooling for **imagery ground truth AGT** — read, validate, and index AGT containers used in **image-stream / training** pipelines (PyTorch and Python consumers planned).

**AGT here is not the Adventure Game Toolkit** (text adventures, `*.DA1`, AGiliTy). See [docs/agt-domain-correction.md](docs/agt-domain-correction.md).

The on-disk format is documented starting from **`Agt-1992.pdf`** (legacy tree `NewC_r529/AGTJ`) and Ken’s real training files. Many files are **non-compliant** with the written spec yet still **loadable**; **`COMMENT`** and **`KEYWORD`** often carry extra ground truth (see [docs/agt-real-world-format.md](docs/agt-real-world-format.md)). Legacy **AGTJ** code under `NewC_r529` is a loose hint only — not authoritative.

## Status

Early development through **M6** (`newagt validate` reports loadability, warnings, and optional `--format json`).

- Format summary (from `Agt-1992.pdf`): [docs/agt-format-spec-1992.md](docs/agt-format-spec-1992.md)
- Implementation milestones: [docs/agt-implementation-plan.md](docs/agt-implementation-plan.md)
- Domain: [docs/agt-domain-correction.md](docs/agt-domain-correction.md)
- Wild-format notes: [docs/agt-real-world-format.md](docs/agt-real-world-format.md)
- Old adventure-game plan (obsolete): [docs/agt-parser-plan.md](docs/agt-parser-plan.md)

## Quick start

```bash
cargo build --release
cargo run --release -- validate /path/to/file.agt
cargo run --release -- validate --format json /path/to/file.agt
```

| Command | Purpose |
|---------|---------|
| `validate` | Parse file and print a text or JSON validation report (exit 0 if loadable) |
| `info` | Summarize discovered AGT inputs (stub) |
| `dump` | Human-readable dump of parsed structure (stub) |
| `to-json` | Export structured JSON (stub) |

Stubs exit with code `2` and print `not yet implemented`.

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## Fixtures

Use Ken’s imagery AGT corpus locally for integration tests (not bundled in this repo).

## License

MIT — see [LICENSE](LICENSE).
