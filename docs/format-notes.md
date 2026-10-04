# AGT format notes

Purpose: **spec vs corpus notes** — deltas between `Agt-1992.pdf`, AGTJ, and Ken’s imagery AGT training files.

As milestones land, record tables here: *1992 spec says* / *AGTJ code does* / *corpus sample*.

See also:

- [agt-format-spec-1992.md](agt-format-spec-1992.md) — PDF summary  
- [agt-implementation-plan.md](agt-implementation-plan.md) — NewAgt milestones  
- [agt-real-world-format.md](agt-real-world-format.md) — COMMENT/KEYWORD expectations  

## M4 — extension bucket (stub)

| Item | NewAgt |
|------|--------|
| `Keyword` string field | First-class `Field` with `Keyword::Keyword` at every level that accepts `Comment` |
| Unknown corpus keywords | `UnknownStatement` + `ParseWarningKind::UnknownKeyword` |
| Misplaced PDF keywords | Preserved as `UnknownStatement` + `ParseWarningKind::OddPlacement` |
| Typed corpus hooks | `ExtensionRecord::Uninterpreted` on `ParseResult.extensions` (empty until inventory) |

Load API: `parse()` (warnings discarded) and `parse_with_warnings()` ([wild format](agt-real-world-format.md)).
