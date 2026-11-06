# ui-pause-close: one vocabulary for stopping work: Pause, Resume, Close

The user, walking a chat (journey 13, 2026-11-06): "I clicked on "Stop" chat, but I'd see a way to resume it. Also, do we really want "Stop" ? It's confusing to me (but maybe it's just me), I wouldn't quite grasp the difference between "Stop" and "Close" as a new user I think. Something like "Pause" or "Park" or whatever?"

They're right on both counts. A chat's "Stop" is jig's `Stop`: the run cancelled and the task held, which a goal's page calls "Pause" and answers with "Resume" (`temper-client-view/src/pages/goal.rs`, about line 228). A chat has no way back: `bar_actions` in `pages/chat.rs` draws no `ActKind::Release`. And "Stop" means the opposite on a goal: `Word::Stop` is `Act::Close`, the end of the work (`pages/goal.rs`, about line 232). This departs from the mock on purpose.

## Decided (temper/ui, on the laptop session's recommendation), browser and terminal alike

- **Pause, Resume, Close.** "Stop" goes as a person's word everywhere. Act names and `ActKind`s stay. `temper ctl`'s own commands keep jig's names: they're temper/forge's. The welcome's checks are out of scope.
- **A chat's bar:** "Pause" where `Stop` is allowed, "Resume" where `Release` is allowed, then "Close".
- **Its status:** "Paused" when the hold is the person's own pause. "Held" stays for a hold with another reason (the reasoning cap and so on), which has its own card and why. Resume is offered for the person's own pause; a hold card keeps its acts. If the client can't tell a person's pause from another hold, find what it's given, and add the fact if it's small: stored formats and protocols may change; tell temper/forge first if one does.
- **A goal's and an agent task's end:** "Stop", "Stop goal", "Stop task", "Stop this task?" and "Stop this goal?" become "Close", "Close goal", "Close task", "Close this task?" and "Close this goal?", what they say the same as a chat's close sheet ("You can still read it, but it can't be reopened.") where that holds, using `look-round-10`'s one sheet.
- **The state's words follow:** a goal's "stopped", "Stopped by you", "Stopped before it landed" and the like become closed's words, on rows, pages, Home and filters. Ended goals keep their meaning. Check a goal's Pause toast: it says running tasks stop at once, which is true, and may keep the verb.
- **Sending a message to a chat you paused also resumes it,** the composer's hint saying so while paused: "Sending resumes this chat." Today the composer stays open and a message sits with no run. **Read first what jig does with a `Message` to a held task** (`jig/docs/design/client/domain/contract.md`, its domain): send `Release` right after it, or whatever makes one act of it, in one domain step, tested. If a refused message must not release, say how that's kept.
- **"Applied" under the conversation after a Pause or Resume goes:** the status says it. Journey 13 shows it standing under the person's message; it's the same stray word as on a closed chat (`client/README.md`, 5.5).

## How the cut shapes it

The design first: `client/README.md`, 5.5, 5.8 and 5.11, including the departures list's "no new word replaces it"; `client/greenfield/README.md` gets a line that this departs from the mock on purpose. Failing tests first, at the lowest tier: the domain's step for the message and Resume, and view tests of each state in both shells. Gallery states for a paused chat, with Resume, its hint and its status. A real browser check on a fake box: start a chat, Pause, Resume, Pause, send a message, see it answered; proof at 1440 and 420 in `~/src/fabrig/ui-pause-close/` on the builder, outside any worktree, the report saying which builder. Regenerate prints, re-record with `temper-client-journeys again` where stale, retrying a moving-page mark. No `Look-request` trailer. The real-loop and `temper ctl` paths keep working.

## Beside it

`ui-walk-3` lands first; `chat-reading-c` is temper/forge's and touches a chat's page: expect catch-ups.

## Size

One or two tasks. One cut is at most 20 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/crates/temper-client-view/src/pages/chat.rs`
- `temper/crates/temper-client-view/src/pages/goal.rs`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
