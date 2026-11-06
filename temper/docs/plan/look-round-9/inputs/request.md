# look-round-9: the welcome tries a model

At the user's request (2026-11-06): land the look request `model-check` (`0bfb77711b`, on `origin/look-requests`; the branch moves, so if that hash is gone, find it by its `Look-request: model-check` trailer). The welcome runs one real turn, its steps live in Forgejo's status box, and a failure offers "Try again", the fitting act, and "Continue anyway".

**The procedure is `temper/docs/design/draft/look-requests.md`: every step of sections 3 to 5, and its account.** Read the commit and its whole message. Every commit carries `Look-request: model-check` directly above `Co-Authored-By:`. Requests on the branch above and below it are other plans'.

## Beside it

fabrig/temper builds the behaviour, `temper/docs/design/draft/model-check.md`, after `first-usable-model`, and takes this look by catch-up. So this plan lands the view's and stylesheet's part. Where the view has no fact yet, draw what can honestly be drawn, keep one function for it, and name it in the account as deferred to that draft. Don't build the behaviour. `look-round-7` (`sign-in-new-tab`) changes the same Model step, and is landing now: chain behind it by catch-up, and keep its words.

## How the cut shapes it

One or two tasks. Screenshots beside the request's pages at 1440, 1000 and 420 wide in `~/src/fabrig/look-round-9/` on the builder, outside any worktree, the report saying which builder. New rules go under `/* ==== ui: welcome ==== */`.

## Size

One cut is at most 30 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/draft/look-requests.md`
- `temper/docs/design/draft/model-check.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
