# ui-reaches: what reaches you, its three classes and Away

Milestone N1 of `temper/docs/design/draft/whole-story.execution.md`: `temper/docs/design/draft/what-reaches-you.md`, sections 2 to 6, as written. That's the three classes, Away as a modifier, who decides in your place, why a thing reached you and Stop asking, and Decided for any task. The user's word of 2026-11-08: "we want all this by v0.1.0 ... move fast and parallelize". Nothing in the draft is reopened.

## What it executes

- **The design first,** as the execution's N1 entry says:
  - `temper/docs/design/domain/forge.md`, 5.8, widened from goals to any task;
  - the classes as settings in `temper/docs/design/protocol/configuration.md`;
  - their status files, and the draft's status for N1.
- **Then the code:** each section's behaviour as the draft gives it, through the engine and both clients' existing pages. The pages' new look is P1 to P5's, so draw what each needs within today's pages. Each need's class, why it reached the person, and Stop asking show wherever a need does.
- **A standing role's case** (section 4) waits for the owner of coding runs' R1. Leave it as the draft's lag, naming R1.

## Its tests

Focused and fuzzy only, from the execution's N1 to N3 entry, the parts N1 executes:
- in temper's worlds, each class's need asked, decided and listed;
- Away moving a class, and not "always waits";
- "why it reached you", and Stop asking changing what reaches the person next;
- Decided for any task, not only a goal.

## Who hears first

`plan.md` names every change to `temper-engine-domain` and `forge_route.rs`, which the owner of coding runs writes too. It also names every change to `temper-client-domain`, for the TUI's owner. Both hear before the plan lands.

## Sources

- `temper/docs/design/draft/what-reaches-you.md`
- `temper/docs/design/draft/whole-story.execution.md`
- `temper/docs/design/domain/forge.md`


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
