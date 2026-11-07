# ui-forge-reviewer: for every task

From the request (`temper/docs/plan/ui-forge-reviewer/inputs/request.md`), the rules that bite here:

- **A plan is complete when it lands.** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A slow-tier story that reads what changes is changed with it and compiled (`heavy cargo check -p PKG --tests`), not run; say so in the report.
- **Tests stay cheap:** a new case of one behaviour is a row of a table test, and a new table only where the spec names one. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The stored format changes in place, with no migration and no compatibility: there's no supported deployment. A small edit the task needs just outside its file list (a re-export, a `mod` line, a golden) is made and named in the report.
- **Generated files** (prints, goldens, counted baselines) your change makes stale are yours to regenerate, by their own tool, in the same task.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.

And for this plan:

- **What this plan doesn't build:** the draft's link (`person` on a forge's record), `Reconcile`'s login, `AppMade`, the approval's card, and eligibility per repository's forge (`temper/docs/design/draft/forge-identity.md`, sections 4, 7, 8 and 9).
- **A branch stays in temper's tree.** jig's people stay as they are: `role_for_identity`, `identity` and `connector_adopt_parties` are enough, and jig's own tests keep their subjects.
- **Beside this plan,** `goal-delivery` writes `forge_route.rs`, and `limits-fixtures` the world fixtures this sweep edits a line of. Keep each edit to the expression it changes, so that whichever lands second catches up easily.
