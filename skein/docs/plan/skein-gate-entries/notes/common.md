# skein-gate-entries: what every task keeps

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything else in a spec.
- **Decide, don't ask.** A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **Nothing is measured in this plan.** The check is of the file's shape, so skein's suites need no run for these entries.
- **Whether 30 and 120 stay as written or become CPU budgets by the user's word stays open.** Leave `skein/docs/foundation/testing-strategy.md` exactly as it is. Write nowhere that the user set these figures.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1). The only figures this plan adds are the 30 and 120 its request names.
