# ui-folds-away: skein's menu stories stop flaking

A red's repair, `red-65`, handed to temper/ui by the director: the stories and the behaviour they check, page menus closing on an outside press, came from temper/ui's `look-round-6` (`f83d202cc9`). `skein-browser-world::browser_real`, `folds_away::a_press_outside_closes_a_menu_and_goes_on`, failed on its run and its rerun at `2a158edd47`, at `skein/tests/browser/tests/suite/folds_away.rs:159`, `assertion left == right`, with only docs changed since a green run. A flake that fails a gate is on the happy path.

**The family:** `folds_away::*` has flaked in about 4 of every 10 after-landing runs since 2026-11-06, in three different stories: `a_press_outside_closes_a_menu_and_goes_on`, `a_panel_folds_away_without_taking_the_outside_press_or_a_forms_escape`, `removing_a_focused_form_returns_to_its_disclosures_summary`. The last's initial lookup was repaired under `ui-goal-asks/red-57` (landed 04:04Z). Reruns on hetzner8 are `browser-skein-N.log` in `~/src/fabrig/fab/after-landing/<tip>-*/`.

## One task

1. Run the three stories by name, alone, through the build wrapper, as a red's repair may, enough times to see them fail, and read what each failure says. `red-57`'s repair is the precedent: it was a lookup before the page had settled.
2. Find the cause, whether a step that presses or reads before the page has settled, an outside press landing where the story didn't mean, or a real fault in skein-ui's folding: a menu that sometimes doesn't close on an outside press is a person's fault, and a focused test of it comes first, in skein's lowest tier that shows it.
3. Fix that cause in all three where it's shared. Each story keeps every check it makes, and no deadline widens. If it's a skein-ui fault, say so in the report, as skein/tiers wants to hear.

No new browser test, no walk or render check. New cases are rows of an existing table test.

## Sources

- `skein/tests/browser/tests/suite/folds_away.rs`

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
