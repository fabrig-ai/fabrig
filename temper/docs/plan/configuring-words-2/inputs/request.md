# configuring-words-2: what a real walk of the configuring chat still found

`configuring-words` landed 2026-11-06; temper/qa's walk of it (`rp3`) found its words hold, but these remain. Causes are from reading a run, not reproduced: reproduce first in the lowest tier. The design is `temper/docs/design/draft/configuring.md`. Stock words can only be tested for what's mechanical; a walk tests the rest.

1. **"Nothing has changed" still ends 3 of 5 and 4 of 5 replies on two walks, and comes before any proposal** (the user asked for once, where a person could think something did). Read why the instruction didn't hold: stock words saying to report it somewhere else, or a tool's result saying so, which the agent repeats. Fix the source, and say what you found.
2. **Told to withdraw a proposal, its authority refuses it.** A proposal an agent made, it can't withdraw when the person asks. Either it may, by `temper/docs/design/protocol/configuration.md`'s rules for a proposal, or its reply says plainly how a person withdraws it. Settle by the design.
3. **Small slips in its words:** "the stock settings", "the deployment operator", and the literal "How PROJECT works" with the placeholder unfilled. After a proposal's accepted, it tells a person how to add a teammate but drops adopting again. If stock's words have a placeholder a run can't fill, fix the stock.
4. **The admin's words don't hold across runs.** One walk named temper's admin before any refusal; the next said adopting is "done on the operator's side", though it had read the landing rule's sentence and `Write; forge user 3, login temper`. The stock configuring instructions' "add the person on the forge, adopt again" is the likely source, inferred. Say plainly in the stock who does it and how, so a run can't drop it. A refusal already gets a person past it.

Left out: adopting again, the first proposal's role and a proposal checked whole, which fab's own repairs of the walk hold: read how they landed.

Failing tests first where mechanical; no test reaches a real host. One or two tasks, temper. Beside it, finding-400 (the configuring chat can't run after its proposal is accepted) is on the loop's path, with a foreman since 12:34Z, in the same stock configuring words and charter: read how it lands, take it by catch-up, and don't fix it here. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1). `walk = false`: temper/qa rewalks `rp3` after it lands. Started by temper/ui, which owns the configuring chat: decide, and note it in `plan.md`.


## Sources

- `temper/docs/design/draft/configuring.md`
- `temper/crates/temper-service/src/stock/configuring.txt`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
