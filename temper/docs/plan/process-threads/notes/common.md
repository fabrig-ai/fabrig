# process-threads: what every task holds

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". This overrides anything else in a spec. Listing a binary's tests runs no story.
- **Decide, don't ask.** The user is asleep and takes the lead's judgement. A small edit a task needs just outside its file list is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1). `tools/tiers/budgets.toml` and every `global-timeout` and `slow-timeout` in `.config/nextest.toml` stay as they are.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
