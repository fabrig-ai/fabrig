# ui-walk-2: an answer that didn't send, and a command that doesn't run

Two findings the second whole walk moved to fabrig/ui. Findings and reproductions: `temper/docs/plan/ui-final-walk/findings.md` (A4) and `real-checks.md` (R1), on `origin/ui-final-walk/integration` or main once it lands. Evidence: `~/src/fabrig/ui-final-walk/` on hetzner11. There is no supported deployment: formats and protocols change freely; decide, and note it in `plan.md`.

## A4: a choice pressed straight after Undo didn't answer

On Needs you at 1440: answer a question, press Undo, press a choice at once. Six seconds later the question was still open, the count still 1, and ctl showed it unanswered. A fresh repeat passed at 1440 and 420, so it's intermittent, or depends on timing. **A person's answer that silently doesn't send is the worst thing this page can do.** Find it. Start from the 5-second hold (`ui-needs`): the timer Undo cancels, and whether the next press can be cancelled or dropped with it; a stale request, or a re-render swapping the button under a click, as `ui-browser-red` found for Sign out (a click lost between mouse down and up). Reproduce it at the lowest tier, deterministically if the race allows, with a test red first. Fix it. If it can't be reproduced, say what was ruled out, and make the failure visible: a held answer that didn't go must show as not sent, never as answered or as nothing.

## R1: the Claude way's command refuses on a box with custom directories

On the Model step, `temper program-login claude-owner`, run exactly as shown, refuses when the box runs with non-default config, data or cache directories: it looks under the default config. With them supplied it works, and the real fetch is proved. A plain `temper box` with default directories isn't affected. `fix-flow` found that the client has no directory facts, since the deployment's answer carries only `welcome` and `owned`.

**Decided (fabrig/ui):** the deployment's answer carries the facts, only to a person who can see the command (the owner, during set up): whether directories are non-default, and those three paths. The page shows `TEMPER_CONFIG=… TEMPER_DATA=… TEMPER_CACHE=… temper program-login NAME`, shell-quoted, when they aren't the defaults, and stays short otherwise. If a smaller way exists (for example, the box answers with the exact command as one string, composed where the facts are), prefer it, and say why. Tests for both cases, offline; then the real command on a box with custom directories, up to the sign-in prompt, signing in to nothing.

## How the cut shapes it

Two tasks, not chained if they can avoid it. Keep every assertion. New rules, if any, under the marker of their area.

## Size

One cut is at most 20 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/crates/temper-client-domain/src/pages/inbox.rs`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
