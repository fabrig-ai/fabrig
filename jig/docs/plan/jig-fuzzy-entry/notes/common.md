# jig-fuzzy-entry: what every task keeps

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything else in a spec.
- **Decide, don't ask.** A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **Nothing is measured in this plan.** The check is of the file's shape, so jig's fuzzy suite needs no run for this entry.
- **Whether 120 is jig's figure stays open.** The entry's comment says so, and the entry does not settle it. Write nowhere that the user set this figure.
- **No `[gate.jig.default]`.** The focused suite is over its figure, and its entry waits on the user's word.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1). The only figure this plan adds is the 120 its request names.
