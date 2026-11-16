# mechanical-changes: execution

2026-11-15. How [mechanical-changes.md](mechanical-changes.md) gets built. It designs nothing: the milestone writes its own design from the draft as it runs, and lands whole, with its own tests. It is a goal the user approves. Plans use the focused and fuzzy suites alone; walks come after landing.

## 1. The milestones

| Milestone | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| M1, a change opened from a pushed branch: the `change open` act, its usage and its refusals; its request on the client's wire and the root's expansion of it into the person's own change, with their authority and budget; the source parameter; the taking state that selects the branch's head once, keeps it and makes the change's own branch at it, across restarts and cancels; the change then going as a change whose producer pushed it, under `turn` and `queue`; the stored defaults and the client wire's version | 1 to 10 | none | after landing, on dev.fabrig.ai once redeployed with it: a docs branch opened with the act joins main's next landing by turn and lands, its pushed branch untouched | landed (`../domain/forge/change/opened.md`); its walk follows the redeploy |

## 2. Order

M1 is the draft whole. Section 11's items are no milestone's: each is a later step, drafted with the user.
