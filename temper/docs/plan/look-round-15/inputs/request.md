# look-round-15: a chat's heading is its first sentence, short

At the user's go, 2026-11-07, from journey 9: land the look request `chat-title` (`27b6052708` on `origin/look-requests` now; the branch moves, so find it by its `Look-request: chat-title` trailer). Page `temper/docs/design/client/journeys/chat-title/01.html`, one rule in `overlay.css`; every state is in its message.

**The procedure is `temper/docs/design/draft/look-requests.md`: sections 3 and 5, and its account.** Section 4's render check follows landing, temper/ui's. Read the commit and its whole message. **The two commits above it, `chat-terminal` and `chat-terminal-font`, carry `Look-hold`: they aren't this plan's, so leave them.**

## What it is

A chat's heading is its first prompt, whole today, and a long one fills the bar's first row and pushes the chat's controls down to a second.

- **The heading shows the prompt's first sentence, or its first line if that's shorter.** Past 72 characters it's cut on a word, with an ellipsis.
- **The whole prompt stays in the heading's label and shows on hover.** The person's message and the task card keep it whole, and the stored title stays the whole prompt.
- **The crumbs yield to the controls:** `.temper-crumbs { flex: 1 1 320px }`. The row still wraps below that, so a phone keeps today's two rows.
- **Not the look's:** the short form is worked out in the view, where `pages/chat.rs` passes the chip's title to `Heading::Chat`. Keep it in one place, a function with a focused table test: a sentence, a line, a long word, nothing past 72 but on a word, an ellipsis. The user took this "for now", and a model-written title may replace it later as its own request.
- The terminal's heading takes the same short form.

## Limits

One task. Focused and fuzzy alone, as below; prints regenerate by `temper-client-journeys again` where a journey's heading changes. No browser story and no render check in the plan. New cases are rows of an existing table test.

## Sources

- `temper/docs/design/draft/look-requests.md`
- `temper/crates/temper-client-view/src/pages/chat.rs`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **`temper-client-journeys again` is generation, not a walk:** it regenerates what the gate's focused replay checks read, so a task whose change makes a recording stale runs it, though it drives a browser.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
