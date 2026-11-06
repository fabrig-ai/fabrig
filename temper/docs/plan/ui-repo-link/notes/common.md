# ui-repo-link: what every task holds

From the request (`inputs/request.md`, *For every task of this plan*), which each task obeys:

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** new rules go under this plan's marker comment, `/* ==== ui: switcher ==== */` and `/* ==== ui: settings ==== */` as the area says, never at the end of the file. Replaced rules are removed where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** on a catch-up, regenerate rather than hand-merge; `names.txt` stays sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. A throwaway render script is allowed, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver. Screenshots go to `~/src/fabrig/ui-repo-link/` on the builder, outside any worktree.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that a change makes stale are its task's to regenerate; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound. (This departs from the implementer's text: a task here may edit a help page it names.)

Beside the request:

- **Failing tests first,** at the lowest tier that holds the rule.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **The design is written:** each task's `## Inputs` names its sections. A task never edits them, but for dropping the **Not yet** entry it closes in the status file beside them (this departs from the implementer's text), and a line the code taught it, named in its report.
