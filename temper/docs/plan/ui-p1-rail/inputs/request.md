# ui-p1-rail: the rail, Work, and Deployment

Milestone P1 of `temper/docs/design/draft/whole-story.execution.md`: `temper/docs/design/draft/surfaces.md`, sections 2, 5 and 8. The user's go of 2026-11-08, about 16:25Z. Its look is the mockups on main, `temper/docs/design/client/web/whole-story/` (`work.html`, the rail as each mockup draws it, and their `README.md` for precedence over the greenfield mock). Nothing in the drafts is reopened.

## What it executes

As the execution's P1 to P6 entry says:
- **The design first:** `client/README.md`'s rail and pages, `client/browser.md` where the browser differs, and the greenfield mock's README for what it now defers to.
- **Then the build:** the rail as `surfaces.md` 2 draws it; Work's one list (section 5); and Deployment (section 8). Each comes from the engine's reads, with what a read lacks named in the page's lag.
- **Places the new rail drops** keep a way in, where `surfaces.md` gives one. Ask temper's "What temper remembers about you" (its M4, not built yet) gets a place in the person's menu, beside Deployment, laptop/web's pick: leave a named slot there.

Home and Needs you are `ui-p2-home`'s, running beside this plan. Keep off their pages' files and meet them in the rail's shared parts, as one rail.

## Its tests

Focused and fuzzy only, as the execution's entry says:
- the gallery's states for the rail, Work and Deployment at a laptop's and a phone's width, day and night;
- the client's joint world for Work across two projects;
- a fuzz test of the rail's and Work's parts against `surfaces.md` section 12's rules, across random settings and their moves.

Journeys the change makes stale are regenerated with `temper-client-journeys again`, as generation. New journeys and the walk come after landing.

## Who hears first

`plan.md` names every change to `temper-client-domain` for the TUI's owner. It names every change to `temper-engine-domain` and `forge_route.rs` for the owner of coding runs.

## Sources

- `temper/docs/design/draft/surfaces.md`
- `temper/docs/design/draft/whole-story.execution.md`
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
