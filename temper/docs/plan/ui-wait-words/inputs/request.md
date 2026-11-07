# ui-wait-words: a run waiting for a host says why, with a command that exists

temper/qa's `proof-2` at `f67c3f0a7a`, a box run, medium. Evidence on hetzner8 under `~/src/fabrig/fab/walks/proof-2/`: `goal-replay-initial.txt`, `deployment-after-setup.txt`, `help-deployment.txt`.

- **The words point at nothing:** a goal waiting for a host shows "no host has room, or none is connected: deployment show", and `temper ctl` answers "no group deployment"; `temper help deployment` says there's no such ctl group.
- **They hide the cause:** the goal's charter asked 100,000 turns against a worker's 800, so no host could ever take it. Only reading the source showed that.

## What this plan makes true

1. **Every waiting line names a command that exists,** in the page, the terminal and ctl's answers alike. Check each such pointer in the client's and ctl's words against ctl's real groups, with a focused test that fails on a group that doesn't exist, as a row of an existing table if there is one.
2. **A run no host can ever take says so, and which bound keeps it out,** in a person's words: "No worker can take this: its charter asks 100,000 turns, and the largest worker takes 800." That covers the other bounds admission compares as well, each in the same form. A run that's only waiting for room keeps "Waiting for other work to finish. It starts by itself." on a box (`ui-box-slots`), and its words on `temper serve`.
3. Read where the engine decides that no host can ever take a run, as against none having room now, and carry that difference to the client if it doesn't reach it. It's engine admission, temper/ui's by seam 5. If carrying it changes a stored format or protocol, name it in `plan.md`.

Whether a charter should ask 100,000 turns is the charter owner's, and isn't changed here.

## Limits

One or two tasks. Focused and fuzzy alone: no browser, walk or render check, by the user's rule that a plan is complete when it lands. New cases are rows of an existing table test. No new browser test.

## Sources

- `temper/crates/temper-client-view/src/words.rs`

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
