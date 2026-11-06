# ui-composer-focus: the composer keeps the focus after sending

The user, journey 16 (2026-11-06): "after sending a prompt, the focus is not anymore in the prompt entry form, and I have to click on it again to type more".

In that recording the send was a chat's first message: `StartChat` from Configure, then `/chats/1`. `composer_focus` moves only on `Act::FocusComposer` (`temper-client-domain/src/pages/chat.rs`, about line 934), so arriving at the chat puts the focus on the heading (`temper-client-view/src/frame.rs`, about line 303). Reader's cause, not run: reproduce first.

## Wanted

- **A chat you've just started by sending opens with its composer focused,** from every start that sends and moves: a new chat, Configure, a starter, Start goal's chat if it does.
- **After `Act::Send`, the focus is in the composer, however it was sent.** Enter should keep it, but pressing Send probably leaves the focus on a button that goes disabled as the words clear: check it. A disabled button losing the focus is what hands it to the fallback.
- **Opening an existing chat from a list keeps the heading's focus:** a person reading with a screen reader or the keyboard lands on what the page is. No change.
- On a phone, don't raise the keyboard by moving focus after a press where a person didn't ask: if focusing after a send on a narrow width reopens or holds the keyboard, keep what a phone's messaging app does, the keyboard stays up after sending; opening a chat doesn't raise it. Say what you found.

## How the cut shapes it

One task. Tests red first, at the domain tier for which act moves the focus, and in the browser tier for what only a browser shows: real focus after pressing Send and after Enter (a story in `temper/tests/client-browser`, naming that, at the browser floor's cost; report its CPU and wall). `ui-pause-close` changes the chat's bar, `ui-rename-focus` set the focus rules for removed controls: follow them; expect catch-ups.

## Size

One cut is at most 15 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

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
