# debt-28: the plan

From `inputs/request.md`: pay debt items, twenty-eighth round, one task each. The trunk is `debt-28/integration`. No temper client view, no `temper.css`, no `jig-client-view`: fabrig/ui has priority.

## Parts and tasks

| # | Task | Item | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|---|
| 1 | `inner-allow` | jig D020 | | medium, 15 | `inner_allow` an old form at 0 |
| 2 | `role-sets` | skein D047 | | medium, 50 | 20 named predicates behind one exhaustive table; the slices gone |

The other items follow in later batches.

## Critical path

So far no task needs another: `role-sets`, ~50 implementer-minutes.

## What the cut settled

- **jig's D041 and D043 can't be paid in this plan, and stay open.** Re-measured: 27 and 2, every site in `jig-client-view`, which the request closes twice. The request's own rule covers it: "if the honest shape isn't reachable, keep the item open". Neither gets a task, and each item already says why.
- **skein's D047 is wider than its measure.** 8 more slices sit across lines, where the measure can't see them, and `==` chains test the same sets. **Paid when** says "each set", so they go too. It can't end with fewer lines, since an exhaustive match outweighs the short sites it replaces: its cap is on what it adds. One table behind all the predicates keeps that to about 150 lines where a match per predicate would be about 900.
- **skein's D047's drifted copies stay drifted.** `focusable` holds `Main` and `mapping.rs`'s `tabindex` set doesn't; merging them changes behaviour, so `role-sets` names both for fabrig/debt.

## Overlaps with drafts beside this one

- `role-sets` and `chat-reading-c` (`log-surface`, `text-reading`, `page-reading`, `log-busy`): `skein-ui/src/tree.rs` and `builder.rs`, `skein-ui-text/src/render.rs`, `skein-ui-web/src/page.rs`. Unavoidable: the item lives there. That trunk adds no new set but keeps those it moves, and a new role it brings takes a row in the table.
