# look-round-25: How work runs as the final mockup, and the ways page's last misses

A look plan following P3a (`ui-p3a-how`, landed 2026-11-08 19:29Z). P3a was cut before the mockups' last changes landed: `370f1be9ab`, then the audit's `d8f321422b`. So it draws an earlier `how.html`. This plan brings How work runs and the ways page to the mockups on main, `temper/docs/design/client/web/whole-story/how.html` with its `README.md`, and to `temper/docs/design/draft/stock-ways.md` 4 and `surfaces.md` 7 as amended there. The rule of every whole-story milestone holds (`whole-story.execution.md`, section 2): draw what the engine holds, and nothing a later milestone adds. Follow `temper/docs/design/draft/look-requests.md`, sections 3 and 5.

## Tasks

1. **The path as amended:**
   - who acts by mark: a person's ring orange, an agent's star ink, code grey, and no blue;
   - "update" folds into the cards, with no stop of its own;
   - "conflict" and "resolve" are where a goal's landing meets main in the queue, and "catch up" is where a goal's branch takes main, with a repair and a conflict told apart;
   - vsr's path drawn as `how.html` has it.

   The one engine-side derivation changes, and every client draws what it gives.
2. **Roles on How work runs** (`surfaces.md` 7): every agent the project runs, the foreman and the status agent among them. Each one's "can" line: "never merge or update" becomes "never merge". "Finish alone" is reworded for Ask temper and the status agent. The status agent's line reads "When you look at Home after landings", as `status.rs` does.
3. **The ways page's last misses,** from its render check after `look-round-24`:
   - two small card positions, at 1000 and on a phone;
   - the CI card naming the repository's CI workflows;
   - the line painted as the request's continuous gradient;
   - with a fourth "Yours" tab, the tabs fitting under 760 px, as "Agents review" doesn't fit a quarter of a phone today.

A browser case is a row of an existing table test, within the cap of 12. The gallery's prints are regenerated as the common notes say. Words saved with a setting (P3e) and a goal's "design" and "close-out" stops (P3f) are milestones of their own, not this plan's.

## Sources

- `temper/docs/design/draft/stock-ways.md`
- `temper/docs/design/draft/surfaces.md`
- `temper/docs/design/client/web/whole-story/README.md`


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
