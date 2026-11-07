# ui-live-cheap-2: the live Claude Code story's settings are accepted

A red's repair, of my `ui-live-cheap` (landed 2026-11-07 18:48Z), on `red-70`'s story. Since then, after-landing's `temper::live` `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` fails before it reaches Claude. At tip `3b01f60f08`:

    settings apply: This model cannot be chosen: claude-haiku-5-5

That's `temper ctl settings apply <store>/forge-land-claude/first/settings.json --against 1 --key live-settings`, which exits 2 (`temper/crates/temper/tests/live.rs`, near line 110). The step before it, `account list --json`, exits 0. `ui-live-cheap`'s `live_settings` (`temper/crates/temper/tests/support/forge_land.rs`) adds `claude-haiku-5-5` as a catalogue item at number 64, copied from stock row 3 with effort `low`, then puts that model whole in both charters, all in one apply. The refusal's reason (`jig-client-domain`'s `LlmWhy`: unconfigured, effort, charter, authority, ...) isn't printed.

## One task

1. **Find the `LlmWhy` the core gives for that apply,** with a focused test that applies the fixture's own `live_settings` output to a service holding a Claude Code program account on the fake backends. Use temper-client-tools' settings tests or the service's settings tests, wherever such an apply already runs. It must be red first, with the same refusal.
2. **Make the live story's settings accepted by the path a person takes** to choose a Claude subscription's model by name since `model-catalogue-3` (Settings, Models): the catalogue item first, then the charters, in separate applies if the core checks a charter against the catalogue before the apply. Keep `claude-haiku-5-5` at `low` effort. Only if the core can't let a Claude Code program account choose a model outside stock, use stock row 2, `claude-sonnet-5-5`, at `low`, and say why in `plan.md`. Fable 5.1 isn't an option.
3. **The focused test turns green on the fixture's real output.** The live story itself is compiled, not run: no worker holds its login, and after-landing runs it hourly while it is red.

If the fault is the core's, a model a person adds by name being refused for a charter, fix it there instead, at its lowest place: that's a person's path too.

## Sources

- `temper/crates/temper/tests/support/forge_land.rs`
- `temper/crates/temper/tests/live.rs`
- `jig/crates/jig-client-domain/src/contract/llm.rs`
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
