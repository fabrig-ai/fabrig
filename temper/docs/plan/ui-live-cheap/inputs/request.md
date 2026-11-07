# ui-live-cheap: the live Claude Code story runs on Haiku again, at low effort

The user's word, 11-01: live tiers use cheap models. `model-catalogue-2`'s fixture change (`2bb9867c71`, "Use stock catalogue rows in slow-tier fixtures") moved `LiveModel::ClaudeCode` from `claude-haiku-5-5` to stock row 3, `claude-fable-5-1`, the dearest row in the stock catalogue. So `temper::live`'s `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` now runs Claude Code as `--model claude-fable-5-1 --effort high` at every after-landing run. The ChatGPT story stays on `gpt-6-luna`, stock row 7.

The stock catalogue's Claude rows are Opus 5.5, Sonnet 5.5 and Fable 5.1, with no Haiku. Since `model-catalogue-3`, a Claude subscription takes its models by name, as it has no list to read.

## One task

1. **Run the Claude Code story on `claude-haiku-5-5` at `low` effort,** taking it by name the way a person adds a Claude subscription's model since `model-catalogue-3`. Select it in every charter the story selects, as it does now for the stock row (`temper/crates/temper/tests/support/forge_land.rs`, `LiveModel` and the reads after startup). The story ran on Haiku until `2bb9867c71`.
2. **Only if a model taken by name can't be selected through ctl,** use stock row 2, `claude-sonnet-5-5`, at `low` effort, and say why in `plan.md`. Don't add a row to the stock catalogue.
3. **Hold what the story checks:** the selected model's name in every charter, and the same Claude Code launch arguments but `--model` and `--effort`. `LiveModel::ChatGpt` stays as it is. A focused test of the fixture, where one exists, takes the new name as a row. The live story is compiled (`cargo check --tests`), not run: no worker holds its login, and after-landing runs it at every landing.

## Sources

- `temper/crates/temper/tests/support/forge_land.rs`
- `temper/crates/temper/tests/support/forge_land/rung.rs`
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
