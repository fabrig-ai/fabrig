# ui-public-queue: the public story reads a queued change on Landings

A red's repair, `red-56`, from `ui-public-history`'s words. `browser_public::public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` fails at `temper/tests/client-browser/tests/browser_all/browser_public.rs`, near line 229, on the first run and its rerun at `aa6e377c73`:

    public landings page: temperLandingsQueuetask 18c22892 · Goal 18c22892 · Ready just nowNo landings to show yet.

**The page is right, and the story is too narrow.** It takes "No landings to show yet." or one landed line, but a stock queue's change may be read while it waits, so the page shows its Queue and then no landing. Which it reads depends on timing, so the story has passed on other runs.

## One task

Make the `Page::Landings` check accept a queued member too: "Queue", one member in the form the page draws (`task` and eight digits, `Goal` and eight digits, a phase in words, a time), then either form it takes now. Every other check stays, and its parsing stays as strict as it is now: a line of any other form still fails. Run that story by name, alone, through the build wrapper with `TEMPER_BUNDLE=none`, once or twice, as a red's repair may. No browser beyond that, no walk, no render check. No product change.

## Sources

- `temper/tests/client-browser/tests/browser_all/browser_public.rs`
- `temper/docs/design/client/browser.md`

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
