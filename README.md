# NewAgt

Modern tooling for **Adventure Game Toolkit (AGT)** game files — read, validate, and convert legacy AGT data without DOS-era utilities.

AGT games ship as a bundle of files (`*.DA1`, `*.DA2`, `*.DA3`, optional `*.DA4`–`*.DA6`, and encrypted messages in `*.D$$`). NewAgt parses these into a structured model for preservation, analysis, and conversion (JSON and, eventually, AGX). Real-world files are often non-compliant yet loadable; the parser is designed for **tolerant reading** where safe (see [docs/agt-real-world-format.md](docs/agt-real-world-format.md)).

This project is a **clean-room** implementation informed by the community’s [AGiliTy](https://github.com/DavidKinder/Windows-AGiliTy) interpreter behavior and validated against real game files. Legacy AGTJ / 1992 documentation is used only as non-authoritative hints. It is not a fork of AGiliTy or AGTJ.

## Status

Early development (M0 scaffold). Parsing is not implemented yet.

- Plan: [docs/agt-parser-plan.md](docs/agt-parser-plan.md)
- Wild-format notes: [docs/agt-real-world-format.md](docs/agt-real-world-format.md)

## Quick start

```bash
cargo build --release
cargo run --release -- info /path/to/game/ELECTRA
```

CLI subcommands (stubs for now):

| Command | Purpose |
|---------|---------|
| `info` | Summarize discovered files and version hints |
| `dump` | Human-readable dump (agtout-style) |
| `validate` | Cross-check ranges, pointers, record sizes |
| `to-json` | Export structured JSON |

Each stub exits with code `2` and prints `not yet implemented`.

## Development

```bash
cargo test
cargo clippy -- -D warnings
```

## Fixtures

Download sample AGT games from the [IF Archive AGT collection](https://www.ifarchive.org/indexes/if-archive/games/agt/) for local testing (not bundled in this repo yet).

## License

MIT — see [LICENSE](LICENSE).
