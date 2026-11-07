# browser-cap: what every task holds to

**Only focused and fuzzy run in this plan (the user, 2026-11-06).** Never `tools/tiers/run`, a `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or live tier, one of their stories even by name, or a tier's CPU figure. This overrides anything below or in a spec. A merged browser test is written and compiled: `heavy cargo check -p temper-client-browser-world --tests`. Your report says it wasn't run. The tier's unit tests run: `heavy cargo nextest run -p temper-client-browser-world --lib`.

**Nothing is lost.** Each story's assertions stay as its steps, in its order, moved as they stand, unless your spec names a change. A step may move only where the spec says. No assertion is dropped or weakened to fit.

**No deadline widens.** `WITHIN`, `ABSENT`, a step's `within`, the harness's forty seconds, nextest's profile and `tools/tiers/budgets.toml` stay as they are. If a merged test can't fit a deadline, stop and report it. It doesn't get a longer one.

**Shared setup, not new machinery.** No change to `skein/` or to the browser harness. The tier's helpers in `temper/tests/client-browser/src/` are yours to extend as your spec says.

**Stay out of product crates.** No `temper-client-*`, `temper-service` or `jig-*` source changes. If a merge shows a product bug, report it and don't fix it.

**Decide, don't ask.** A small edit your task needs just outside its file list (a re-export, a `mod` line, a manifest line) is made and named in the report. A `Question:` is only for what truly blocks the task.

**A test's helper that the merge makes unused goes in the same task.**

**Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.

**`temper/docs/design/` is already written.** This plan's cut wrote `temper/docs/design/client/browser.md`, 4.2, 4.3, 4.6 and section 5, and their status. Don't edit them. What the code teaches goes in your report.

**Other drafts touch these files.** `look-round-13/sheet-story` has `browser_accounts.rs`, `browser_pages.rs` and `src/lib.rs`; `model-catalogue-2/process-fixtures` has `temper/tests/client-browser/src/**`; `look-round-13/model-cards` and `model-end` have `temper/tests/client/tests/suite/connect_model*.rs`. Keep your edits to what your spec names, and expect a catch-up.
