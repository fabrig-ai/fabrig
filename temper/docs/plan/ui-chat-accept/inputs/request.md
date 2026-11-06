# ui-chat-accept: accepting a settings proposal in the chat that made it

**Configure temper's main path is broken.** `ui-walk-3`'s finding B1: on a fresh fake box (ChatGPT first run, Settings, Start, the exact proposal cue, before any settings change), stay in the chat and press the proposal card's Accept. Its sheet opens; the primary Accept answers "No visible object with that name", leaves the sheet open, and Needs you stays 1, at 1440 and 420. The same proposal accepted from Needs you or Home works. `temper/docs/design/client/README.md`, 5.13: the card's Accept sends the proposal's decision; 3.6: a successful primary completes.

The walk's report and reproduction: on hel8, `/home/free/src/fabrig/coord/ui/walk-driver/walk3-findings.md` (B1) and `walk3-report.md`; its driver is beside them (`README.md`, `walk.py`, `additions.py`, `cdp.py`, `again.py`). Evidence: `~/src/fabrig/ui-walk-3/walk-again-chat/` on hetzner8 (`free@2.29.110.236`).

## The task

1. **Red first, at the lowest tier.** Cause not established. Start from what differs between the chat's card and Needs you's: which request and scope each sends, and what party and project each decides under. `ui-walk-large` gave a proposal `projects`, with `project` staying 0, and `ui-walk-3`'s M3 changed the rejection path: read whether this ever worked, by walking the history before assuming it's a regression. The words come from a refusal of the engine's for what a party can't see.
2. **Fix the cause,** and both of the card's acts, Accept and Reject: "the unavailable direct Reject control" in the same evidence is the same family. Accepting from the chat then updates the card, clears the count, and the rail changes, as from Needs you.
3. **Prove it in a browser:** the configuring chat's whole path on a fresh fake box, at 1440 and 420: the cue, the card, Accept, the rail changing, and the chat still answering afterwards.

## Beside it

**finding-400,** "the configuring chat can't run after its proposal is accepted", is on the loop's path with a foreman since 12:34Z, its cause unknown. Read `coord/qa`'s or the loop's account (`fab/runs/configuring-walks/finding-400*`) first, and don't duplicate it: if it's landed, build on it; if the same cause explains both, say so. `configuring-words-2` changes the same chat's stock words. `ui-pause-close` changes the chat's bar: expect catch-ups.

## How the cut shapes it

One or two tasks. Design documents move with it. A protocol or stored-format change needs no `Question:`, but tell temper/forge in `plan.md` and through your report. Screenshots in `~/src/fabrig/ui-chat-accept/` on the builder, outside any worktree.

## Size

One cut is at most 20 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/crates/temper-client-domain/src/pages/chat.rs`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
