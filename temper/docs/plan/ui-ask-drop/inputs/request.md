# ui-ask-drop: the configuring chat goes, as Ask temper takes its place

The user's word, 2026-11-07, on Ask temper (`temper/docs/design/draft/ask-temper.md`, sections 1 and 9 as `c0ac935335` left them): "just drop the old one". Ask temper is temper's own AI, in the rail, and nothing keeps the configuring chat beside it. `ui-ask-m1` built Ask temper, moved and renamed the client's Configure page into it, and left the engine's side for this plan.

## What goes

- **Template 3 and charter 3** leave the stock, with every rule, delegation, policy, default and count that names them. Charter 11 and template 5 take their places where a person's setup needs a configuring agent.
- **What reaches them:** `Configure`'s act and offer, and anything left in the client, ctl or the welcome that starts template 3. Help pages and their reference say Ask temper.
- **`temper ctl`'s paths** that start or name template 3 or charter 3 go in this plan too, at temper/migration's word. Its help and words keep their present style, and `plan.md` lists each ctl path changed, for temper/migration to judge after.
- **The stock's executors after:** `plan.md` states the new list of charters a stock policy grants as executors (today 1, 2, 3, 4, 6 and 9), as generated settings elsewhere grant them by number.
- **No stored or wire compatibility:** numbers may change in place. Note what changed in `plan.md`.

## What stays, moved to charter 11

The configuring acceptance stays whole. `configuring.md`, section 8, and rung D ask Ask temper, and the gate's scripted runs of charter 3 become charter 11's, with what they check unchanged. A test that only checks charter 3 exists goes with it. A test of the configuring mechanics that now live in `propose_settings` moves to charter 11's tools.

## Its tests

Focused and fuzzy only. Journeys whose recordings the change makes stale are regenerated with `temper-client-journeys again`, as generation. Slow-tier stories that name charter 3 are changed and compiled, not run.

## Sources

- `temper/docs/design/draft/ask-temper.md`
- `temper/docs/design/draft/configuring.md`
- `temper/docs/design/draft/configuring.execution.md`
- `temper/crates/temper-service/src/stock.rs`


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
