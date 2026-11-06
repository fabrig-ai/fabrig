# process-ports: what every task holds

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". What changes in those tiers is written and compiled (`heavy cargo check --tests`), not run. This overrides anything else in a spec.
- **Decide, don't ask.** The user is asleep and takes the lead's judgement. A public interface may change freely: no compatibility, no `Question:`; note it in the report. A small edit a task needs just outside its file list (a re-export, a `mod` line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **Keep `test-threads = 1`.** Raising it is a later, separate change. `.config/nextest.toml` isn't touched here, and the `temper-sign-in-callback` group stays.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1).
