# ui-public-history: standing rules for every task

These come from `temper/docs/plan/ui-public-history/inputs/request.md`, *For every task of this plan*. Read it; these are
the rows this plan's two tasks can meet.

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a
  tier's CPU figure. A story in those tiers is written and compiled (`heavy cargo check -p PKG --tests`), not run; say so
  in the report. This overrides anything else here.
- **Decide, don't ask.** The user is asleep. A small edit a task needs just outside its file list (a re-export, a `mod`
  line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Generated files** your change makes stale (prints, goldens, counted baselines) are yours to regenerate by their own
  tool in the same task; on a catch-up, regenerate rather than hand-merge.
- **A test's oracle or helper your change's correct behaviour trips** is yours to repair in the same task, keeping what it
  checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and
  your branch fails, it's yours.
- **Out of this plan:** titles and any title rule, going public, temper/qa's new edition of `rung-public`, and
  `browser_public`'s geometry story (`public_board_and_task_lines_are_readable_at_desktop_and_phone_widths`), which
  `red-50` is repairing on the same file: leave its measuring alone.
- **The Projects page stays as it is:** "No public projects yet." does not change.
- **No wire change:** `public-v1.schema`, `public-v1.fields` and the codec stay as they are (`contracts.md`).
