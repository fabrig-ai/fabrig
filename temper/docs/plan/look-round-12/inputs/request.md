# look-round-12: the model's check runs in the side pane

At the user's request (2026-11-06, of these pages: "go for it"): land the look request `model-check-pane` (`ba044d7413`, on `origin/look-requests`; the branch moves, so if that hash is gone, find it by its `Look-request: model-check-pane` trailer). Pages `journeys/model-check-pane/01.html` to `04.html`; every state is in its message.

**The procedure is `temper/docs/design/draft/look-requests.md`: every step of sections 3 to 5, and its account.** Read the commit and its whole message. Every commit carries `Look-request: model-check-pane` directly above `Co-Authored-By:`. Other requests on the branch are other plans': `model-picker` (below it) waits for `model-catalogue`; a newer one above it isn't released.

**It reverses part of `look-round-9`,** on main: the status box in the panel's place goes, Ready's checklist goes to the side pane, and "Continue" and "Continue anyway" go. Remove what it replaces, rules and view both. Keep what `sign-in-new-tab` (`look-round-7`) and `ui-welcome` landed that it doesn't touch.

## Beside it

fabrig/temper builds `temper/docs/design/draft/model-check.md`'s behaviour once `first-usable-model` lands, and takes this look by catch-up. So: land the view's and stylesheet's part. Where the view has no fact yet (the try's steps, its outcome), draw what can honestly be drawn, keep one function for it, and name it in the account as deferred to `model-check`. Don't build the trial. A person must still be able to go through Ready to Create on a usable account, with no check running.

## How the cut shapes it

One or two tasks. Screenshots beside the request's pages at 1440, 1280, 1000 and 420 wide, "Each step" open, in `~/src/fabrig/look-round-12/` on the builder, outside any worktree, the report saying which builder. Journeys printing Ready are reprinted, and re-recorded if stale. New rules go under `/* ==== ui: welcome ==== */`.

## Size

One cut is at most 20 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

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
