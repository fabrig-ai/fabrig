# ui-rename-focus: the project menu's rename form, and a stray focus ring

The user asked for these two today, seen on a box from main at `359cb69106` (through fabrig/temper). They come after `ui-final-walk`'s `fix-look`, which touches the same stylesheet. Causes below are a reader's, not reproduced: reproduce first. One or two tasks; each fix has a test red first; proof at 1440, 1000 and 420 beside the mock.


1. The project menu's rename form should match the mock (greenfield/app.js 229-240 and its Escape at 1136; mock.css 291-296): while renaming, the open project's row itself becomes the form: a name field filling the row (min-height 34px) and a primary small Save. "Rename NAME" leaves the menu's acts while the form is open. Enter saves, Escape cancels, an empty name keeps the old one, no visible Cancel. Today: temper-client-view/src/frame.rs rename_form, and temper.css 119-121 .temper-project-rename-form, the old look's rules; remove what this replaces.
2. A dark focus ring on main after a focused control disappears (the rail's divider looks like a black line). Repro: Home, project menu, Rename Hello, Cancel: skein-ui-web page.rs (~360-376) moves focus to Role::Main with tabindex=-1, and Chrome draws focus-visible; #app > main.temper-session has no rule. temper-client-shell-web lib.rs finish can do the same after keyboard use. Wanted: (a) closing the rename form by Save or Escape returns focus to the control that opened it, or the menu's summary, not main; (b) a programmatic focus target (main, its h1, anything given tabindex=-1 only to take focus) draws no outline, in skein's sheet if it belongs there. Real controls keep their focus ring.

## Size

One cut is at most 20 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/crates/temper-client-view/src/frame.rs`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
