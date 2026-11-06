# ui-repo-link-2: what every task holds

From the request (`inputs/request.md`, *For every task of this plan*), which each task obeys:

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. A new story for those tiers is written and compiled, not run. A throwaway driver's walk isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`. A throwaway render script is allowed, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver. Nothing committed names a lab path.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that a change makes stale are its task's to regenerate; don't park on them.
- **Help pages:** a `docs/help/` page a change grows past its bound is that task's to shorten, losing no facts, and never raises the bound. A task here may edit a help page it names (this departs from the implementer's text).
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- No `Look-request` trailer. The review's own file lands separately: this plan doesn't touch `temper/docs/development/ui-repo-link-review.md`.

Beside the request:

- **Failing tests first,** at the lowest tier that holds the rule.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **The design is written:** each task's `## Inputs` names its sections. A task never edits them, but for dropping the **Not yet** entries it closes in the status files beside them (this departs from the implementer's text), and a line the code or the binary taught it, named in its report.
