# debt-26: the plan

From `inputs/request.md`: pay debt items, twenty-sixth round, in four independent parts. The trunk is `debt-26/integration`. No temper client view, no `temper.css`, no `jig-client-view`: fabrig/ui has priority.

## Parts and tasks

| # | Task | Request's part | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| 1 | `records` | 1 | | medium, 10 | D004 `kept`; the plaintext shim's Departs entry says the user left it |
| 2 | `forgejo-kinds` | 1 | | high, 70 | the rule in `forge.status.md`; `_ =>` under a named expect; the five wrappers and their `Option` parameters gone |
| 3 | `world-kits-item` | 2 | | medium, 20 | jig's new item; `testing.md`'s "in any tree" says the gate's reach |

The rest of parts 2, 3 and 4 follow in later batches.

## Critical path

As it stands: `forgejo-kinds`, ~70 implementer-minutes.

## What the cut settled

- **The plaintext shim's Departs entry already exists** in `skein/docs/design/link.status.md`; it said the user had yet to decide. `records` records that they did.
- **D004 goes `kept`:** what's left aligns the two roots or waits on the same decision, so the user's deferral is all that's left.
- **`reading.rs`'s `Some(_) | None`** follows guards on two optional times and picks no kind: exempted, not changed.
- **`testing.md`'s sentence** is true of its words: the gate does read every tree's edges onto `skein/tests/`. What misleads is that it reads as the whole rule, with about 93 edges onto smith's and temper's `tests/` it never reads. The correction says so.

## Overlaps with drafts beside this one

None for this batch.
