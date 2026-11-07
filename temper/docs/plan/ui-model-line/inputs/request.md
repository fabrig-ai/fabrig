# ui-model-line: a chat says the model and effort it really uses

The user, 2026-11-06, of a closed configuring chat with no choice of its own, on their box at `d893a93cc1`, reading "T2 · Using the default model": "it should tell the actual model and effort instead". That chat read "T2 · ChatGPT plan · gpt-6-astra" while it ran, on an older build.

By reading, not running: `model_words` in `temper/crates/temper-client-view/src/choice.rs` returns `None` for a task with no choice of its own, so `using_words` says `W::UsingDefault`, and the default it would resolve is only looked up for a new draft.

## What this plan makes true

1. **A chat that has run** says what its latest run used, model and effort. **First, say whether a run's record keeps both, and whether the client is given them.** If not, carrying them is this plan's: stored formats change in place, with no compatibility. Tell temper/migration's seam by naming it in `plan.md`, as both of us write `temper-engine-domain`.
2. **A chat that hasn't run** says its template's `resolved_default`, as a new chat's composer does.
3. **Effort always shows when it's known.** Today's line drops it when the choice names none.
4. **"Using the default model" stays only where temper truly can't know.** `temper/docs/design/client/README.md`, near line 4199, changes with it. The terminal client takes the same words.

## Where the line goes

`look-round-11` moved the choice under the composer (`model-picker`, page 06), and a closed chat has no composer. So a chat with a composer keeps the picker's line as it is; a closed chat says it in the header, where it was: "T2 · GPT-6 Astra · high", in the shown names the picker uses, and the bar's muted form. An agent's or a goal's page with the same line follows the same rule.

## Limits

One or two tasks. Focused and fuzzy alone: no browser, walk or render check, by the user's rule that a plan is complete when it lands. `ui-model-next-turn` is fixing what the next request sends, in the runtime; this is what the page says, so meet at catch-up.

## Sources

- `temper/crates/temper-client-view/src/choice.rs`
- `temper/docs/design/draft/model-catalogue.md`

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
