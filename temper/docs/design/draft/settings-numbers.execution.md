# Settings numbers: execution

2026-11-18. How [settings-numbers.md](settings-numbers.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole, with its own tests and fixtures. Each is a goal the user approves. Plans use the focused and fuzzy suites alone; walks come after landing.

## 1. The milestones

| Milestone | Draft parts | Needs | dev.fabrig.ai | State |
|---|---|---|---|---|
| R1, the ranges: jig's settings check refuses an addition below 33 or past 64 in templates, charters, policies and models, with the range in its words; templates' and charters' numbers 1 to 64; temper's template-goal and charter-words kinds 1 to 64; a stored addition below 33 still restored at a start, and marked on the settings page and in `temper ctl settings show`; help's template and charter pages, and Ask temper's next free number from 33. Tests: for each kind, an addition at 32 refused and at 33 taken, a change of a stock item at its number taken, a proposal judged at its decision, a stored addition at 9 restored and marked | 3, 4, 5 | — | compatible: its additions below 33 (templates 9, 16, charters 17, 19) stay and get marked | not started |
| R2, collisions: a start's new stock items met by a stored change at their number, as a list; `temper-engine upgrade --check` and `temper-box upgrade --check` refusing with each collision's line; a start logging each, and the settings page and `temper ctl settings show` showing the hidden stock item beside the owner's; help's recipe for moving an item. Tests: a build adding a stock template 9 over a stored template 9 is refused by the check and started with the stored one in force and the stock's shown hidden; a change made over a stock item that was already there is no collision | 6, 7 | — | compatible: nothing changes until a build brings a stock item onto one of its numbers | not started |

## 2. Order

- **Eager.** R1 and R2 start at once, each its own goal, in parallel. They meet only in the settings page's and ctl's marks, which each adds beside the other's.
- **No priority mark.**
- **dev.fabrig.ai needs no step.** Moving its template 9 to 33 or above before a build adds a stock template 9 is advised, by the recipe R2 writes.
