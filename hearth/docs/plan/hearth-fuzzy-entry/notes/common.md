# hearth-fuzzy-entry: what every task keeps

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything else in a spec.
- **Decide, don't ask.** A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **Nothing is measured in this plan.** The check is of the file's shape, so hearth's fuzzy suite needs no run for this entry.
- **Whether the 120 stays wall time or becomes a CPU budget is an open question, and it stays open.** Leave `hearth/docs/design/domain/testing.md`, 9.1 and 11, and `testing.status.md` exactly as they are. Write nowhere that the user set this figure.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1). The only figure this plan adds is the 120 its request names.
