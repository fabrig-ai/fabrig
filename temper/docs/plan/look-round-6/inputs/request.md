# look-round-6: how a chat's calls and its answer read

At the user's request (2026-11-06, of these pages: "Exactly what I'd like to see"): land the look request `chat-work` (`3f707babb9`, on `origin/look-requests`).

**The procedure is `temper/docs/design/draft/look-requests.md`: every step of sections 3 to 5, and its account.** Read the commit and its whole message: it lists the states not drawn and what isn't the look's. Every commit carries `Look-request: chat-work` directly above `Co-Authored-By:`. Its pages are hand-made, `journeys/chat-work/01.html` to `04.html`; its `README.md` says so.

`configure-temper-own-way` (`8ad410e969`), above it on the branch, is held for the user: don't land it.

## Beside it

fabrig/temper is building `temper/docs/design/draft/chat-reading.md` around this, in the user's order: (a) section 3's whole live text and section 4's turn budget, starting now; (b) section 2's line, on top of (a) and this look; (c) section 5's window and scrolling; (d) section 6's numbered lists in skein's Markdown. Read that draft first. This plan lands the view and stylesheet of the request: the line, the answer's look, "Latest ↓", "Loading earlier messages…". What needs the engine or the window is theirs. Where the view has no fact yet, draw what can honestly be drawn, keep one function for it, and name it in the account as deferred to `chat-reading`. Don't build (a), (c) or (d) here. Take their commits by catch-up.

`ui-final-walk` and `ui-rename-focus` are landing `temper.css` changes too; `ui-rename-focus` adds `position: relative` to the conversation's scroller. Expect catch-ups. New rules go under `/* ==== ui: chats ==== */`.

## How the cut shapes it

Two to four tasks, in order: the call line and its open state; the answer; reading's pill and message. Screenshots come back to the user beside the request's pages at 1440, 1000 and 420 wide, with the line open, in `~/src/fabrig/look-round-6/` on the builder, outside any worktree, the report saying which builder. Journeys that print a chat are reprinted, and re-recorded if stale. The terminal's prints keep their meaning.

## Size

One cut is at most 45 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/draft/look-requests.md`
- `temper/docs/design/draft/chat-reading.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
