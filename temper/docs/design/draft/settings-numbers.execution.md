# Settings numbers: execution

2026-11-18. How [settings-numbers.md](settings-numbers.md) gets built. It designs nothing: the milestone writes its own design from the draft as it runs, and lands whole, with its own tests and fixtures. It's a goal the user approves. Plans use the focused and fuzzy suites alone; walks come after landing.

## 1. The milestone

| Milestone | Draft parts | Needs | dev.fabrig.ai | State |
|---|---|---|---|---|
| R1, the ranges: jig's settings check refuses an item below 33 that the stock doesn't have, or one past 64, in templates, charters, policies and models, with the range in its words, at a set, at a proposal's decision and at a start; templates' and charters' numbers 1 to 64; temper's template-goal and charter-words kinds 1 to 64; `upgrade --check` naming each such stored item; help's template and charter pages, and Ask temper's next free number from 33. Tests: for each kind, an item at 32 refused and at 33 taken, a change of a stock item at its number taken, a removal below 33 taken, a proposal judged at its decision, a start and the upgrade check refusing a stored template 9 and charter 17 by name | 3, 4, 5, 6 | — | steps, before the first redeploy with R1, on the build dev runs, by `settings-numbers.md`, section 7: template 9 to 33 and 16 to 34, and charter 19 to 33 and 17 to 34, so that template 33 runs on charter 33 and template 34 on charter 34, as 9 ran on 19 and 16 on 17. Their template-goals 9 and 16 move to 33 and 34. The rules' delegations to 19 and 17 (one each) and policy 1's (four each), and project 2's policy copy if it names them, point at 33 and 34. Then the old numbers are removed. No live task needs to end first. `upgrade --check` on the new build shows none left | not started |

## 2. Order

- R1 starts once this draft has landed. No priority mark.
- dev.fabrig.ai's steps can be taken any time before its first redeploy with R1, each step on the user's yes as every change to dev's settings is.
