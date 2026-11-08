# ui-history-red: the history story passes every time

A red's repair, `red-78`. The director routed it here by area, after the foreman cleared its one suspect. It's `temper-client-browser-world::browser_all`'s `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back`, failing at `temper/tests/client-browser/tests/browser_all/browser_pages.rs` near line 188:

    history: person 0, step 107: keep selector main.temper-needs attribute class: no element

Step 107 is just after `Step::Person(PersonStep::Go { address: "/nowhere" })`, `see(Find::exactly_named(Role::Main, b""))` and `needs_page()`. That's the Needs page reached from an unknown address.

**It's intermittent.** In after-landing on 2026-11-08 it failed and then passed alone at tips `8d2cdcdcc6` (14:35Z), `da3a06bf55` (14:44Z) and `703cbe11ae` (15:24Z). It passed at `423669a32a` and `c049c710fd`, and failed twice at `0f6c343237` (15:15Z). It began around when `look-round-22` (14:37Z) changed the outside pages. That plan shows the top bar alone while `me` is read, and "No project yet" in place of whatever an address drew. So a page body that comes and goes is a likely race: either the story sees an empty `main` before the Needs page draws, or the client draws one page, then another.

## One task

1. **Find the race,** by running the story by name, alone, through the build wrapper, as a red's repair may, as often as needed, under a load you make if it won't fail unloaded. Record which page draws at step 107 and when.
2. **Fix it at its lowest place.**
   - If a person could see the wrong page flash at an unknown address, it's the product's: under a failing focused test, the client draws one page for an address once `me` is read, never another first.
   - If only the story is early, it waits for the readiness a person's client gives, not for any `main`.

   The story's checks stay as they are.
3. Say in `plan.md` which it was, and how many runs passed after the fix.

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
