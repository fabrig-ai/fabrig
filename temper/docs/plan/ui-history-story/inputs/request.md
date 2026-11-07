# ui-history-story: the Chats history story holds its filter, or says why not

A red's repair, `red-63`, opened 10:09Z at `77544586ee` with no owner found; it's temper/ui's area. `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` failed on the run and its rerun at `temper/tests/client-browser/tests/browser_all/browser_pages.rs`, near line 156:

    history: person 0, step 74: keep selector main .temper-filters > button[aria-pressed='true']:nth-child(3) attribute aria-pressed: no element

The page was "Talk to an agent about playground/widget. · temper". Only docs changed since the last green run at `a749ec09c7`, so this is timing, or something the story reads that moves, and not a change's fault. The same story failed once at `e929a7d195` (`red-62`), with another message, and closed absent.

## One task

1. Run the story by name, alone, through the build wrapper, as a red's repair may: enough times to see whether it fails, and on which step.
2. **Read whether Chats really loses its filter across a reload or Back, which would be a person's fault,** or whether the story reads too soon or by position. The step selects the filter by its place among its siblings, `:nth-child(3)`, which `temper/docs/design/client/journeys.md`, 7 says a selector never does.
3. Fix what it shows: a real loss of the filter in the client, with a focused test of the restored state first; or the story's selector, by the filter's name and pressed state, waiting as its sibling steps do. Keep every check the story makes.

No product change if the client is right. No new browser test, and no deadline widened. No walk or render check, by the user's rule.

## Sources

- `temper/tests/client-browser/tests/browser_all/browser_pages.rs`

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
