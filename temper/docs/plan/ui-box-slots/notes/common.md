# ui-box-slots: what every task keeps

These are the request's rules for every task of this plan (`temper/docs/plan/ui-box-slots/inputs/request.md`). They win over anything in a spec.

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan. Don't ask for them or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A story in those tiers that your change reaches is edited and compiled (`heavy cargo check -p PKG --tests`), not run; say so in your report.
- **Tests stay cheap.** A new case of one behaviour is a row of an existing table test, not a new test (`skein/docs/foundation/testing-strategy.md`, 8). temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost is rejected there.
- **Decide, don't ask.** The user is asleep and takes this plan's judgement. A small edit your task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in your report, not parked. A `Question:` is only for what truly blocks the task.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task, by their own tool. `temper/crates/temper/src/tests/configuration.rs` and `limits.rs` write theirs with `SKEIN_CODEGEN_WRITE=1`.
- **Goldens and prints** conflict across plans: on a catch-up, regenerate rather than hand-merge.
- **Help pages** a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in your report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If the base passes and your branch fails, it's yours.
- **`docs/design/` and `docs/help/` depart from your text in one way:** your task names the status lines it closes and the help lines it changes. Change those, nothing else there. The design sections it cites are already written.
- **Keep hunks small.** Other drafts are changing files beside yours (`temper/docs/plan/ui-box-slots/plan.md`, *Overlaps*). A new word or row goes beside its siblings, never at a file's end.
- **`walk = false`:** nothing here adds a walk or a QA rung.
