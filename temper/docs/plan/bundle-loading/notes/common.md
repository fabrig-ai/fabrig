# bundle-loading: what every task holds to

The request is `temper/docs/plan/bundle-loading/inputs/request.md`; these are its standing rules, and the plan's.

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in your report. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **Failing tests first, in the lowest tier.** Write the tests your task names before the code they hold, see each fail for the reason it names, then make it pass.
- **Design status is yours where your task says so.** A departure from the implementer's text: each task deletes from the status files it names the **Not yet** lines it closes, and only those. Otherwise `docs/design/` is not yours; what the code taught goes in the report. Whatever its section says stays as the plan's cut wrote it.
- **Proofs live outside the worktree:** `~/src/fabrig/bundle-loading/` on your builder. Your report names the builder; nothing committed does (`tests/boundary/tests/outward.rs`), other than inside this plan's directory, and even there it isn't needed.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **`temper.css`:** this plan adds no rule to it. A rule it replaces is removed where it stands.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** on a catch-up, regenerate rather than hand-merge; keep `names.txt` sorted as it is.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task, by their own tool.
- **Help pages:** a `docs/help/` page your change grows past its bound is yours to shorten, losing no facts, and never raise the bound.
- **Budgets are the user's.** Over budget is red (`skein/docs/foundation/testing-strategy.md`, 8.1): does the test belong here; can it be made faster; otherwise report the measurements.
- **Drafts beside this one:** `sign-in-new-tab` and `model-check` are `temper/forge`'s and don't touch this. Keep off `temper/tests/client-browser/src/lib.rs`, `temper/crates/temper-client-view/**` and `temper/crates/temper-client-domain/**` unless your task names them.
