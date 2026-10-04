# Optional authority frame count (N)

Trainer pipelines often need a **fixed-length frame timeline** even when an AGT file’s `SenUpd` / `TgtUpd` lists are shorter, longer, or uneven. Today, **NewAgt does not read ARF imagery containers**; frame count authority is supplied externally (CLI, Python, or future ARF integration).

## Behavior

| `expected_frame_count` | Timeline length | Pairing |
|------------------------|-----------------|---------|
| `None` (default) | `max(sen_upd, tgt_upd)` | List-order (or AGTJ heuristics when enabled) |
| `Some(n)` | Exactly **n** frames (`0 .. n-1`) | Same pairing pass, then pad or cap to **n** |

When **N is set**:

- Index **i** uses the *i*-th collected `SenUpd` and *i*-th `TgtUpd` when present; missing sides are empty with per-frame warnings.
- If either list has **more than N** updates, extras beyond index `N-1` are **ignored** (warning).
- If either list length **≠ N**, an index-level **warning** is emitted (both sides can disagree with N independently).
- Heuristic (`Keyword` / `Comment` frame hints) pairing still runs when enabled; the result is **normalized** to length N (truncate if longer, pad empty frames if shorter).

When **N is unset**, behavior matches the original M8 frame index (`max(sen, tgt)` order pairing).

## Surfaces

- **Rust:** `FrameIndexOptions::expected_frame_count`, `build_frame_index`, `Document::frames`.
- **CLI:** global `--frame-count N` on `newagt frames`, `info`, `bboxes`, and `validate` (count checks as validation warnings).
- **Python:** `newagt.frames(..., frame_count=None)`, `validate(..., frame_count=None)`, `bboxes(..., frame_count=None)`.

`FrameIndex::authority_frame_count` echoes the option used to build the index (for reporting).

## Future: ARF reader

ARF metadata may eventually provide the canonical frame count. Until an ARF reader exists, **user-supplied N is optional authority** for indexing only; AGT list contents remain the source of update payloads. When ARF support lands, the same `expected_frame_count` field can be populated from ARF instead of (or in addition to) CLI flags, with documented precedence.

## Example (conceptual)

Given 2 `SenUpd` and 1 `TgtUpd` and `--frame-count 5`:

- Five frames are emitted.
- Frame 0: sensor + target.
- Frame 1: sensor only.
- Frames 2–4: empty (padding).
- Warnings: `SenUpd count (2) != authority frame count (5)`, `TgtUpd count (1) != authority frame count (5)`.

No corpus paths or private scan artifacts belong in this repository; exercise authority mode on your local AGT files.
