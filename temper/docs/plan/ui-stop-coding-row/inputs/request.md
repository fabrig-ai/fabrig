# ui-stop-coding-row: a box's stop costs a coding run nothing

A follow-up to `ui-box-children-2` (`red-70`'s repair), asked for by the owner of coding runs. Since that plan, serve's first signal cancels every run once the engine has drained (`worker.stop()`, the worker domain's `Event::Shutdown`, `host::Reason::Shutdown`). Each agent can then end its Claude Code program before the worker closes. That plan's report says the engine's closing worker-link phase discards what the worker sends after the drain, so no cancel's outcome reaches the journal. No test holds that for a coding run.

## One task

1. **A row first, beside `a_signal_mid_run_drains_and_the_next_start_settles_the_claims`** (`temper/tests/one-process/tests/suite/one_process.rs`, `world::drain_mid_run`), or as a row of `temper/tests/real-loop/tests/suite/lifecycle.rs`'s stop tests if only that world hosts a coding run's agent. A coding run's task has its agent mid-turn at the first signal. After the drain, and a restart on the same store:
   - the task is due again, as after a crash;
   - no attempt is counted against it, and its change's tries are as before the stop;
   - its saved work is kept, as `temper/docs/design/domain/worker.md` keeps it across a restart;
   - its slot and checkout are as the restart expects.
2. **If the row fails because the Shutdown cancel reaches the journal** as a cancelled run, a failed attempt or a spent try, fix it at its lowest place. Keep the outcome out of the journal once the engine has drained, or record the run as interrupted. If it fails for another reason, the slot or the checkout, fix that the same way. Say in `plan.md` which it was.
3. **If neither world can restart a coding run on one store cheaply,** check instead that nothing the cancel sends after the drain reaches the journal, so the run stays in flight as after a crash, and say so. Run each row alone.

## Sources

- `temper/tests/one-process/tests/suite/one_process.rs`
- `temper/tests/one-process/src/world.rs`
- `temper/tests/real-loop/tests/suite/lifecycle.rs`
- `temper/crates/temper-worker-domain/src/route.rs`
- `temper/docs/design/domain/worker.md`


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
