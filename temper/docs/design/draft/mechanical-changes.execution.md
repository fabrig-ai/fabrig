# mechanical-changes: execution

2026-11-15. How [mechanical-changes.md](mechanical-changes.md) gets built. It designs nothing: the milestone writes its own design from the draft as it runs, and lands whole, with its own tests. It is a goal the user approves. Plans use the focused and fuzzy suites alone; walks come after landing.

## 1. The milestones

| Milestone | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| M1, a change opened from a pushed branch: the `change open` act and its refusals; the change made in the person's own batch, what the forge fills in for it, and its authority; the new parameter, the branch to take with its commit; the first step that reads the branch's head, checks it and makes the change's own branch at it; the change then going as a change whose producer pushed it, under `turn` and `queue`; the new parameter's store and its upgrade | 1 to 10 | none | after landing, on dev.fabrig.ai once redeployed with it: a docs branch opened with the act joins main's next landing by turn and lands, its pushed branch untouched | not started |

## 2. Order

M1 is the draft whole. Section 11's items are no milestone's: each is a later step, drafted with the user.
