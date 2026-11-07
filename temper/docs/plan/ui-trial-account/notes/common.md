# ui-trial-account: the rules every task names

Read this note, then your spec. Where this note and the implementer's text differ, this note wins. The request is `temper/docs/plan/ui-trial-account/inputs/request.md`; the shapes that cross tasks are `temper/docs/plan/ui-trial-account/notes/contracts.md`.

## Departures from the implementer's text

- **Decide, don't ask (the user, 2026-11-06, asleep).** A public interface may change freely: no compatibility, no `Question:`. A `Question:` is only for what truly blocks the task.
- **A small edit your task needs just outside its file list** (a re-export, a `mod` line, a golden) is made and named in the report, not parked.
- **Status lines.** The cut wrote this plan's design. Each task removes the **Not yet** entry its spec names from its status file when its acceptance holds, and nothing else in `docs/design/`; what the code taught goes in the report.
- **A required field crosses trees.** A branch stays in its tree, but it carries the other trees' lines that name a field its change requires, each edited only for that. Run `heavy cargo check -p <dependent> --tests` for each dependent the compiler names.

## What holds in every task

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, a `<tier>-<tree>` profile, `scripts/browser.sh`, or a tier's CPU figure. A story for those tiers is written and compiled (`heavy cargo check -p PKG --tests`), not run; say so in the report. No walk, render check, screenshot or throwaway driver: they follow landing. This overrides anything else.
- **Tests stay cheap.** A new case of one behaviour is a row of an existing table test, not a new test (`skein/docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost is rejected there. Report the wall time nextest gives each test you add or grow, before and after.
- **No new browser test,** and no provider's real name in a fake: a fake world's providers are numbers. Rows that stand for Claude and ChatGPT say so in a comment.
- **Goldens, prints and journey recordings** conflict across plans: on a catch-up, regenerate rather than hand-merge.
- **`temper-client-journeys again` is generation, not a walk:** it regenerates what the gate's focused replay checks read, so a task whose change makes a recording stale runs it, through `heavy`. A re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so.
- **A test's oracle or helper your change's correct behaviour trips** is yours to repair in the same task, keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- **Beside this plan:** `model-catalogue-3` narrows the offer per account and touches `jig-domain-core`'s `runs/llm.rs`, `documents/catalogue.rs` and `vocabulary.rs`; `limits-fuzzy` touches `temper-engine-domain` and `temper/tests/engine/domain`. Whichever lands second catches up. Keep your edits in those files to the lines your spec names.
