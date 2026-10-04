# NewAgt

Rust tooling for **imagery ground truth AGT** — read, validate, and index AGT containers used in **image-stream / training** pipelines (PyTorch and Python consumers planned).

**AGT here is not the Adventure Game Toolkit** (text adventures, `*.DA1`, AGiliTy). See [docs/agt-domain-correction.md](docs/agt-domain-correction.md).

The on-disk format is documented starting from **`Agt-1992.pdf`** (legacy tree `NewC_r529/AGTJ`) and Ken’s real training files. Many files are **non-compliant** with the written spec yet still **loadable**; **`COMMENT`** and **`KEYWORD`** often carry extra ground truth (see [docs/agt-real-world-format.md](docs/agt-real-world-format.md)). Legacy **AGTJ** code under `NewC_r529` is a loose hint only — not authoritative.

## Status

Early development (M1 lexer in `newagt-core::lex`; parser M2 not started).

- Format summary (from `Agt-1992.pdf`): [docs/agt-format-spec-1992.md](docs/agt-format-spec-1992.md)
- Implementation milestones: [docs/agt-implementation-plan.md](docs/agt-implementation-plan.md)
- Domain: [docs/agt-domain-correction.md](docs/agt-domain-correction.md)
- Wild-format notes: [docs/agt-real-world-format.md](docs/agt-real-world-format.md)
- Old adventure-game plan (obsolete): [docs/agt-parser-plan.md](docs/agt-parser-plan.md)

## Quick start

```bash
cargo build --release
cargo run --release -- info /path/to/agt/container-or-dir
```

CLI subcommands (stubs for now):

| Command | Purpose |
|---------|---------|
| `info` | Summarize discovered AGT inputs and layout hints |
| `dump` | Human-readable dump of parsed structure |
| `validate` | Report spec mismatches and structural issues |
| `to-json` | Export structured JSON |

Each stub exits with code `2` and prints `not yet implemented`.

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## Fixtures

Use Ken’s imagery AGT corpus locally for integration tests (not bundled in this repo).

## License

MIT — see [LICENSE](LICENSE).
