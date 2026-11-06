# ui-qa-small: Home's false empties and three lines of words

From temper/qa's walk on a real model (`f89ecece19`). Each part is small and has no look decision.

1. **Home says "No chats yet" and "Nothing needs you. Work carries on…" while Behind, with T1 live.** Seen intermittently, with tabs backgrounded, at 1440 and 420. Whatever provokes it, the words are wrong.
   **Ruling:** no page claims empty until its read has answered. While a read hasn't answered, whether first load or after the link comes back, Home keeps what it last knew, and with nothing known it shows the section's title with no line under it. Check `Word::NoChats` and `Word::HomeNeedsEmpty` on Home, and the same words on Chats and Needs you (`Word::InboxNeedsEmpty`), and apply the rule wherever the same claim is made. A focused test for each, red without the fix: Behind with an unanswered read draws no empty claim; an answered empty read still says it.
2. **A used sign-in link's words name `temper serve` to someone who ran `temper box`.** `words.rs`, "Signed out. To sign in, open a sign-in link temper serve printed." becomes "Signed out. To sign in, open a sign-in link temper printed."
3. **The configuring chat's model wrote `127.0.0.1:38507`, port and all, into charter instructions.** A box's port changes, so those instructions go stale. In `temper-service/src/stock/configuring.txt`, add one sentence in its style: instructions and reasons name a repository by its name, never by an address or port. Regenerate whatever the charter's text feeds.
4. **"Proposal t1/4 awaits an owner" reached the owner in the chat.** That wording isn't in temper's words as far as I could grep, so find where it comes from. If it's a tool's answer, have the answer say "waits for an owner's Accept" without a key, and if it's the model paraphrasing, add one sentence to the same stock: tell the person the proposal waits in this chat for their Accept, without its key. Say which in the report.

One or two tasks. No browser proof is needed for 2 to 4; for 1, a view or joint-world test is enough. No `Look-request` trailer.

## Sources

- `temper/crates/temper-service/src/stock/configuring.txt`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
