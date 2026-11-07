# ui-box-slots: a box runs more than one agent, and says so when it's waiting

From the user's journeys on their own box, relayed from their laptop, 2026-11-06. With a configuring chat writing, they started a second chat and briefly saw `WaitCapacity`, "No host has room for this, or none is connected." Their words: "Maybe we're capping smith runs to 1 in temper box?" Yes: `workers[0].slots` is 1 in `temper/crates/temper-init/template/temper.json` and `bare.json`, and `temper/docs/design/protocol/configuration.md`, section 13, and `setup.md`, 8.1 write it down as "the worker's one slot".

## 1. A box defaults to 4 slots

So that two chats and a goal's agents run side by side. The pick is the laptop's and temper/ui's, for the user to reverse. By `temper/docs/design/domain/worker.md`, 6, `hub.slots` and `fleet.slots` (both 2 now) and `checkout.workspaces` (3) must rise with it, and `git_component.operations` (3) with the workspaces. Move them together, keeping the relations that section states.

**Check first what 4 costs, and pick the nearest sound number if 4 doesn't fit:** the start's fit check (`configuration.md`, section 6), memory and disk on a person's laptop, and whether a stock limit quietly assumes one run. Say what you found in the report and `plan.md`. Change `configuration.md`, 13, and `setup.md`, 8.1, with it, and the help where it says one. Limits for coding runs aren't changed by this: only how many run at once.

Stories that read one slot, process-tier ones included, change with it: written and compiled, not run. `process-fixtures` in `model-catalogue-2` is moving the same documents: whichever lands second catches up.

## 2. Its words on a box

A box's worker is in its own process, so "none is connected" isn't true there, and a person who has only run `temper box` doesn't know "host". On a box: "Waiting for other work to finish. It starts by itself." `temper serve` keeps today's, if the view can tell them apart. If it can't from what it's told, say so, and give it the smallest fact that lets it: a box's own client knows it's a box in at least one place, so look before adding one.

## 3. A failed call a person reads

On "How does temper work?", the configuring agent called `settings` with `{"setting":{"kind":"landing-rule"}}`. The chat shows "1 failed" and "failed: host call member missing: number" (`temper-engine-smith`'s `tools.rs`, near 57; `settings.rs`, the `Missing(b"number")`). It called again with `number` 1 at once, so nothing was lost, and the user noticed the red line. The schema invites it (`calls.rs`, near 185): `number` says "Only numbered kinds take number", never which, and `project` says "optional for landing-rule (the deployment's when absent)". The kinds beside it, `rules` and `project-ways`, take none.

- `number`'s description names the kinds that need it.
- The answer says what to do, as the commit identifier's does, for both reader kinds: "landing-rule needs number", over "host call member missing".
- Failed calls keep showing: they're honest.

## Limits

Two tasks at most. Focused and fuzzy alone: no browser, walk or render check, by the user's rule that a plan is complete when it lands. New cases are rows of an existing table test.

## Sources

- `temper/docs/design/domain/worker.md`
- `temper/docs/design/protocol/setup.md`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
