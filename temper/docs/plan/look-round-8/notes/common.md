# look-round-8: standing rules

For every task of this plan. It lands the look request `configure-temper-own-way`; `temper/docs/design/draft/look-requests.md`, sections 3 to 5, is its procedure.

## The request

- **Read it:** `git show 3651707d4c`, on `origin/look-requests` (fetch that branch): its whole message, then the hunks of `temper/docs/design/client/journeys/overlay.css` and `temper/docs/design/client/journeys/configure-own-way/{01,02}.html`. The plan's request names `c980127128`, the same commit before a refresh; `3651707d4c` is the one on the branch now. Its `Look-hold` no longer applies: the user released it on 2026-11-06.
- **Never landed:** no file of the request's own comes in, neither `overlay.css` nor `configure-own-way/`. Read them with `git show`, or a throwaway copy outside the worktree. `main`'s overlay stays as it is.
- **Every commit carries** `Look-request: configure-temper-own-way`, directly above `Co-Authored-By:` where the commit has one, the last line otherwise.
- **Where the task and the request differ,** the task holds. `notes/contracts.md` says what differs, and why. The figures, the table's cells and the picture are decided there: do not use the request's numbers.

## Decide, don't ask

The user is asleep and takes fabrig/ui's judgement.

- A stored format, a protocol or a public interface may change freely, with no compatibility and no `Question:`; say so in the report.
- A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, never parked.
- A `Question:` is only for what truly blocks the task.

## Files shared with other plans

- **`temper.css`:** new rules go under `/* ==== ui: settings ==== */`, never at the end of the file, including the document pane's rules for this request. Remove a replaced rule where it stands.
- **`look.rs` and `words.rs`:** take the next free token and word number. Another plan may take the same one: on a catch-up, renumber yours.
- **Goldens, prints and the gallery's `names.txt` and `tests.rs`:** they conflict across plans. On a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) your change makes stale are yours to regenerate in the same task.
- **Help pages:** a page your change grows past its bound is yours to shorten, losing no fact. The bound never rises.
- **Not yours:** `skein/` (its Markdown reader and flows stay as they are), `temper-client-view/src/pages/chat.rs` beyond a call's new argument, and a project's stored settings.

## The numbered list

`chat-reading-d/numbered-lists`, on another trunk, changes skein's reader: today `1. x` is a bullet list's item whose words begin "1. "; then it is a real ordered list, `<ol start>`, with no marker in the words. Either may land first, so:

- A test reads the terminal's print, where both give `1. Design draft`, and never a list item's words.
- The rules hold both forms (`notes/contracts.md`, 5).

## Render proofs

- Any task may use a throwaway render script, untracked and gone before the push.
- Chromium 147's new headless won't go below 500 px by `--window-size`: a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`.
- Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- The mock is `temper/docs/design/client/greenfield/`. It has no such card: the request's pages are the look here.
