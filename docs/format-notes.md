# AGT format notes

Purpose: **spec vs corpus notes** — deltas between `Agt-1992.pdf`, AGTJ, and Ken’s imagery AGT training files.

As milestones land, record tables here: *1992 spec says* / *AGTJ code does* / *corpus sample*.

See also:

- [agt-format-spec-1992.md](agt-format-spec-1992.md) — PDF summary  
- [agt-implementation-plan.md](agt-implementation-plan.md) — NewAgt milestones  
- [agt-real-world-format.md](agt-real-world-format.md) — COMMENT/KEYWORD expectations  

## M4 — extension bucket

| Item | NewAgt |
|------|--------|
| `Keyword` string field | First-class `Field` with `Keyword::Keyword` at every level that accepts `Comment` |
| Unknown corpus keywords | `UnknownStatement` + `ParseWarningKind::UnknownKeyword` |
| Misplaced PDF keywords | Preserved as `UnknownStatement` + `ParseWarningKind::OddPlacement` |
| Typed corpus hooks | `ExtensionRecord::Uninterpreted` on `ParseResult.extensions` (empty until inventory) |

Load API: `parse()` (warnings discarded), `parse_with_warnings()`, and `parse_with_options()` ([wild format](agt-real-world-format.md)).

## M5 — parse profiles

| Profile | CLI `--profile` | Behavior |
|---------|-----------------|----------|
| **`agtj`** (default) | `agtj` | AGTJ-aligned placements: `Fov` in `SenUpd`, `PixBox` on `Tgt`, 5-field `Utm` (grid + datum strings after elevation). |
| **`pdf1999`** | `pdf1999` | PDF yacc §6.2 placements only: AGTJ-only constructs become `UnknownStatement` with `ProfileExtension` warnings; `Utm` accepts exactly 3 numeric fields. |

| Item | 1992 PDF yacc | AGTJ / corpus | NewAgt default (`agtj`) |
|------|---------------|---------------|-------------------------|
| `Fov` in `SenUpd` | Not in `sen_upd_item` | `Prototype_SenSect.agt` | Typed `Field` |
| `PixBox` on `Tgt` | §2.1.2 but not in `tgt_item` | `Prototype_TgtSect.agt` | Typed `Field` |
| `Utm` | 3 numbers | Optional grid/datum strings (5 tokens) | 3 or 5 tokens |
