# ui-walk-3: the whole client, walked again, once everything has landed

All of fabrig/ui's plans are on main: the pages, the walk's fixes, proposals in Needs you, Home's brief on a fresh project, the rename form, Sign out in the Owner menu, the lost-answer fix, `chat-work`, `sign-in-new-tab`, `model-check`, `close-chat` and temper's own way. Each proved itself against a moving main. This walk checks that they still hold together, as `ui-final-walk`'s two walks did.

## The walk

The driver is on hel8 at `/home/free/src/fabrig/coord/ui/walk-driver/` (`README.md`, `cdp.py`, `walk.py`), with the second walk's `findings-walk-2.md`, `report-walk-2.md` and `real-checks.md`. The tasks copy what they need: it stays untracked, or goes under this plan's directory, as before. The same walk: fresh `temper box --fake-backends`, at 1440 and 420, mock beside client, with decisions (`/home/free/src/fabrig/coord/ui/report/decisions.md`, `rulings.md`) not counted as findings. Add these:

- **W3:** Configure temper's proposal in the project's Needs you, the rail's count, Accept, the rail changing.
- **W6:** a landing, 31 minutes away, Home's brief; on a real sign-in's personal account, as the walk had. Then an unfunded path: the computed sentence stands.
- **A4:** the exact press sequence, repeated at least ten times: Later, Reply, send, Undo, press the choice. Every answer sends.
- **R1:** the shown command on a box with custom directories, up to the sign-in prompt, no credentials.
- **The rename form, focus and the Owner menu:** Enter, Escape, focus's return, Sign out from the menu, reload, staying out; the Accounts tab on a phone.
- **The two scroll bars:** a long chat at 1280x720 and 1000x600, older messages loaded.
- **`chat-work`:** the one-line calls open and shut, the growing answer, "Latest ↓". What `chat-reading` hasn't landed yet is named, not a finding.
- **`close-chat`:** each confirmation the sheet serves.
- **`sign-in-new-tab` and `model-check`:** the looks as landed; the behaviour that fabrig/temper's drafts still owe is named, not a finding.
- **Temper's own way:** the card, See how it works, Start from it, a fresh project.

## The result

`report.md` and `findings.md`, in the same form as before, comparing with the second walk: what's fixed, what's new, what regressed. Fix what's small, with a test red first. List what's large for fabrig/ui. Screenshots under `~/src/fabrig/ui-walk-3/` on the builder, outside any worktree, the report naming it. Copy nothing to the repository but the report.

## How the cut shapes it

The walk; small fixes; the walk once more only over what the fixes touched. Decide, don't ask.

## Size

One cut is at most 20 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/client/journeys.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
