# ui-public-2: public reading's spend, look and sense

temper/qa walked `rung-public` (loopback, nothing public) and it failed. Public reading stays off regardless of this plan: going public is the user's word. This plan fixes what the walk found that's temper/ui's to fix. Isolation held: another project, emails, logins, models, cookies and unknown identifiers all behaved.

**Titles are not this plan's.** The walk also found that agent-written titles carry words from a chat to a stranger. That titles cross is the user's decision of 2026-11-03 (`jig/docs/design/domain/people.md`, 13.3), and it's with them now. Don't change what a title is, whether it crosses, or the words about it. Nothing here should make either answer harder.

## 2. Goals' spend alone

**What the walk found:** every line shows "126 spent · of 505246", tasks too. `browser.md` 8.3 lists a goal's spend alone; `people.md` 13.3 and `contract.md` 12 list both.

**Ruling:** a goal's line shows its spend. A task's doesn't, and no line shows a budget. Remove them from the public records too, and make the three documents agree.

## 3. The look

**What the walk found:** on the board and task pages, phase, time, spend and priority overprint each other at 1440 and 420, in a lopsided table. There's no mock for these pages and they've had no look pass.

**Ruling:** a plain single column, in the client's type and colours (`temper.css`'s tokens, the Chats page's rows as the model, not a table, no monospace), the same at both widths:
- the project's name as the title, and one line under it: "What temper is doing on this project, as it happens. Read-only."
- each goal a row: "Goal 7169df3c" and its title (`browser.md` 8.3), then its phase in words, for how long, its spend, muted, on one line that wraps cleanly; its tasks indented below it in the same form, without spend.
- Landings below the goals, only when there's a queue; "Nothing has landed yet." when it's empty.
- empty pages say so: "No goals yet."

Nothing overlaps at any width from 320 to 1440; check with a throwaway render, and report it with screenshots at 1440 and 420 of a board with live, landed and failed goals.

## 4. Sense to a stranger

- Failed and landed goals both read "ended". **Ruling:** "landed", "failed" and "ended" (for one closed with neither), whatever the public record can carry to say so.
- "its change on the forge789ada..." has lost its spacing and reads badly: "Its change: 789ada1", a short commit.
- Commit links are padded with 24 zeros: a 40-digit id widened to 64 somewhere. Fix it where it happens, with a focused test.
- Forge links answer 404 to a stranger when the repository is private, and a box's is. **Ruling:** link only where the forge's repository is known public; otherwise the short commit is plain text. If the index can't know, drop the links.

## Left out

Landed work from before a project went public never shows; a restart empties the history; a stock queue's Landings stay empty. Say in `browser.status.md` that these remain.

Only focused and fuzzy run in the plan, as below; update any browser story this changes, compiled, not run. Two tasks at most, `jig` first.

## Sources

- `temper/docs/design/client/browser.md`

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
