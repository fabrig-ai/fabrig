# look-round-10: closing a chat, and every confirmation

At the user's request (2026-11-06; asleep, taking recommendations): land the look request `close-chat` (``, on `origin/look-requests`; the branch moves, so if that hash is gone, find it by its `Look-request: close-chat` trailer). The close dialog becomes a sheet ("Close this chat?", "Keep it", "Close chat") saying only what closing changes, and a closed chat ends on one quiet line. "Dangerous action", the zero counts, "Decision made" and "Applied" go. The same sheet serves every confirmation in `confirm.rs`. No behaviour changes.

**The procedure is `temper/docs/design/draft/look-requests.md`: every step of sections 3 to 5, and its account.** Read the commit and its whole message. Every commit carries `Look-request: close-chat` directly above `Co-Authored-By:`. Requests above and below it on the branch are other plans'.

## Beside it

`look-round-6` (`chat-work`) changes how a chat reads, and `ui-walk-2` is checking presses lost to re-renders on Needs you: a confirmation's button keeps its identity while the page updates. `ui-goal-page`'s Pause and Stop, and `ui-needs`' Accept, go through `confirm.rs`: walk each confirmation it serves. Expect catch-ups. New rules go under `/* ==== ui: chats ==== */`.

## How the cut shapes it

One or two tasks. Screenshots beside the request's pages at 1440, 1000 and 420 wide in `~/src/fabrig/look-round-10/` on the builder, outside any worktree, the report saying which builder. The terminal's prints keep their meaning.

## Size

One cut is at most 20 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/crates/temper-client-view/src/confirm.rs`
- `temper/docs/design/draft/look-requests.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
