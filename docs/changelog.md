# Changelog (milestones)

High-level project history. Not a per-commit log. No customer paths or corpus filenames belong in this file.

## M0–M10 (core product)

| Milestone | Summary |
|-----------|---------|
| **M0** | Rust workspace, stub CLI, domain docs |
| **M1** | Lexer, spans, keyword table |
| **M2** | PDF-shaped parser and AST |
| **M3** | Typed composite values |
| **M4** | Wild-format bucket (`Keyword`, unknown statements, warnings) |
| **M5** | Parse profiles `agtj` (default) and `pdf1999` |
| **M6** | `validate` with text/JSON reports |
| **M7** | `dump`, `to-json` (`newagt.schema.v1`) |
| **M8** | Frame index (`frames`, heuristic pairing) |
| **M8b** | Bounding boxes (`bboxes`, PixBox + Score/tgtdb compute) |
| **M9** | Python PyO3 module and `trainers.py` helpers |
| **M10** | Directory batch `info`, packaging docs, CI on fixtures |

Detail per milestone: [agt-implementation-plan.md](agt-implementation-plan.md).

## Post-M10

| Theme | Shipped |
|-------|---------|
| **Authority frame count** | Global `--frame-count` / Python `frame_count=`; padding and cap semantics — [arf-frame-authority.md](arf-frame-authority.md) |
| **Classic dataset pairing** | `newagt pairs --dataset-root` with optional `--missing agt\|arf\|both` |
| **Corpus batch summary** | Directory `info` stderr one-line stats (parse ok/fail, frames, warnings) — [corpus-testing.md](corpus-testing.md) |
| **Lexer tolerance** | CRLF line endings; underscores in unknown keywords; signed integer literals (e.g. negative `PixLoc` components) |
| **Signed / wild PixLoc** | Corpus-aligned integer parsing for trainer geometry |

## Not yet

- ARF container reader (frame count from imagery metadata)
- Full-tree corpus benchmarking in CI (private data stays local)
- Deep PyTorch `Dataset` in-repo (sketch only in [user-guide.md](user-guide.md))
