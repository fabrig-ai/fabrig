# model-catalogue-2: the rules every task names

Read this note, then your spec. Where this note and the implementer's text differ, this note wins. The request is `temper/docs/plan/model-catalogue-2/inputs/request.md`; the draft is `temper/docs/design/draft/model-catalogue.md`.

## Departures from the implementer's text

- **Decide, don't ask (the user, 2026-11-06, asleep).** A stored format, a protocol or a public interface may change freely: no compatibility, no version bump, no migration, no `Question:`. Each family's schema, generated code and one set of golden bytes change in place (`jig/docs/design/protocol/store.md`, 5). A `Question:` is only for what truly blocks the task.
- **A small edit your task needs just outside its file list** (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked.
- **Design text.** A task edits the `docs/design/` lines its `Owns and may change` lists, and any single design line your work proves wrong; the rest of `docs/design/` stays as this plan's cut wrote it. What the code taught goes in the report.

## What holds in every task

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, a `<tier>-<tree>` profile, `scripts/browser.sh`, or a tier's CPU figure. A new story for those tiers is written and compiled (`heavy cargo check -p PKG --tests`), not run; say so in the report. This overrides anything else. A look's render check by a throwaway driver isn't a tier.
- **`temper.css`:** new rules go under the marker `/* ==== ui: settings ==== */`, never at the end of the file. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`** conflict across plans: on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) your change makes stale are yours to regenerate in the same task, by their own tool.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** a `docs/help/` page a change grows past its bound is that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper your change's correct behaviour trips** is yours to repair in the same task, keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **The look request:** a commit landing part of page 07 (`models-tab`, `models-switch`, `models-add`, `models-proof`) carries the trailer `Look-request: model-picker`, directly above the other trailers (`temper/docs/design/draft/look-requests.md`, 3).
- **Neighbours on their way to main:** `look-round-11` (the picker at the composer: `temper-client-view/src/choice.rs`, `choice/picker.rs`, `pages/session.rs`, `new_chat.rs`, `configure.rs`) and `model-check` (`chat-draft`'s prints). Keep your hunks in their files small. A print conflict at catch-up is regenerated, not merged.
