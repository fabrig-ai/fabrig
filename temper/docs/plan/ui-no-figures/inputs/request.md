# ui-no-figures: the reference has no figures

The user (2026-11-06), of "How temper builds temper"'s section 6, "In numbers": "just remove those, we'll use the site or people can go to the public facing version once in place".

Remove the section: `temper/docs/help/how-temper-builds-temper.md`, "## 6. In numbers", its three lines and "Figures to come.", and what renders it, with any rule, word or test that exists only for it (`temper-client-view/src/tests/reference.rs` holds its lines). No link, no placeholder, nothing in its place. The reference's other sections keep their numbers.

Bring the design with it: `temper/docs/design/draft/own-way.md`: its one-page line on figures, section 3's list, section 5 and section 7's open question go; say in its status that figures live on the site or the public-facing version, the user's word, 2026-11-06. Keep section numbers cited elsewhere sound: the queue's citation gate rejects a broken citation (`tools/citations --all`). Check `temper/crates/temper-service/src/stock/configuring.txt` and the look's account for the figures too.

One task. Regenerate prints and generated help. A browser check that the pane still opens at 1440 and 420, by a gallery state if that's enough. No `Look-request` trailer. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/help/how-temper-builds-temper.md`
- `temper/docs/design/draft/own-way.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
