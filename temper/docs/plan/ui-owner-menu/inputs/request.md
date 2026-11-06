# ui-owner-menu: Sign out from the rail's Owner row

At the user's word (2026-11-06, through the director, taking fabrig/ui's recommendation): Sign out is too buried at the foot of the Accounts tab. The rail's Owner row, plain text in the mock, opens a small menu with Sign out. A small, decided departure from the mock.

## The change

- The rail's foot row, the person's name, becomes a button that opens a small menu above it, in the look of the project switcher's menu (`frame.rs`, the `#app > .temper-rail` rules of `temper.css`). Its entries: "Accounts", to Settings' Accounts tab, and "Sign out", the same act and the same failure line as today's. Escape and a click outside close it, and focus returns to the row.
- Sign out leaves the Accounts tab's foot. A project-less Accounts page, with no rail, keeps it there.
- **On a phone** there's no rail row: Sign out stays reachable on Settings' Accounts tab. Keep it there at the phone's width.
- The terminal keeps its acts and their meaning.
- `ui-rename-focus` lands focus rules for menus and removed controls first: take them by catch-up, and follow them. The Sign out work of `ui-browser-red` holds: the button keeps its identity while accounts arrive, so a click isn't lost; keep the browser test's assertions, adapting only its way to the button.

## How the cut shapes it

One or two tasks. Tests red first; a browser story that signs out from the menu, reloads and stays out. The proof at 1440, 1000 and 420, open and closed. Update `client/README.md` and the mock's note of departures. New rules go under `/* ==== ui: settings ==== */`.

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
