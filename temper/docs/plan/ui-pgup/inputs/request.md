# ui-pgup: PgUp while the agent writes, and after its turn

The user's journey 23, from their laptop: PgUp must scroll a chat's conversation while the agent is writing, and after a turn has ended, with the focus in the composer, and the view must not snap back. temper/forge's `chat-reading-c` is on main and brought PgUp and PgDn from the composer, the head reading earlier lines as it nears, the window by size, and "Latest". It didn't prove these two streaming cases. They're the point of this plan.

The design is already decided: `temper/docs/design/draft/chat-reading.md`, section 5 ("**New words while reading back** do not move the view"; "Latest ↓"; "Keys"), `temper/docs/design/client/browser.md`'s "Reading the conversation", and skein's `skein/docs/design/ui-web.md`, section 7, "Keys".

## What a person does, and must see

On a chat with enough words to scroll, on a fake box whose model streams slowly enough to watch, focus in the composer throughout:

1. **While the agent writes:** press PgUp. The view moves up a screen less a line and stays there while words keep arriving. It doesn't snap back at the next delivery, the next call row or the next line. "Latest ↓" shows. Press PgUp again: further up. Press PgDn until the foot: the view follows the live words again and "Latest ↓" goes. Press "Latest ↓" from further up: to the end, following.
2. **After the turn ends:** the same, starting with PgUp at the moment the turn ends and just after. The ending redraws the turn as committed, which is a likely place for a snap back: press PgUp while it's writing, let the turn end, and the words being read stay where they are on the screen. A turn's ending doesn't bring the view to the foot. Sending does, as the design says.
3. **Focus** stays in the composer across all of it, with typed words kept: type a few words first, and they're still there, the caret too.

Also check the same with a wheel scroll in place of PgUp in step 1, since a person does both. If the key path and the wheel path differ, the key path is the one to fix so they match.

## How

First reproduce it with a throwaway driver, as below, and report what each step does today, before changing anything. If a step already holds, say so and only add its test.

The likely place is how the page decides the person has scrolled away (`skein-ui-web`'s page part: a key's scroll may not count as a person's scroll where a wheel does, or a delivery may restore the foot before the scroll is seen), and how a committed turn's redraw is keyed in temper's view. That's a guess: read it.

A fix in skein stays generic, in its own tests, with `ui-web.md` updated. Focused tests in `skein-ui-web` for each rule the fix turns on: a PageUp during following stops following; a delivery after it leaves the position; a PageDown to the foot follows again. In temper's view or client domain, if a turn's ending is the cause, a test of what the committed redraw keeps.

Don't add a browser story: the tier is at its budget. The throwaway driver's report is the proof, at 1440 and at 420, with the scroll offsets at each step and a screenshot of the view holding while words arrive.

Budgets are the user's, and they stay within budget.

## Sources

- `temper/docs/design/draft/chat-reading.md`
- `skein/docs/design/ui-web.md`

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
