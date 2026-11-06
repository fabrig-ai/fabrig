# ui-qa-small: what every task holds to

From the request (`temper/docs/plan/ui-qa-small/inputs/request.md`), and they override anything else in a spec.

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories. A file of those tiers that a change reaches is edited and compiled (`heavy cargo check -p PKG --tests`), not run.
- **Decide, don't ask.** A small edit a task needs just outside its file list (a re-export, a `mod` line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task. A test's table of counted bytes is such a baseline: take the new figure from the failure, after reading why it moved.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`** conflict across plans: on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair,** keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **No `Look-request` trailer,** and no browser proof: a view or joint-world test is the proof here.
- **This plan's status entries are the task's to close.** Departing from the implementer's text, which closes `docs/design/` to you: when your acceptance holds, delete the **Not yet** entries your spec names from their status files, and change nothing else under `docs/design/`. What the code taught goes in the report.
