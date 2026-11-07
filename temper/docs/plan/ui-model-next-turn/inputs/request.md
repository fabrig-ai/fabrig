# ui-model-next-turn: changing a started chat's model takes effect on its next turn

`look-round-11`'s proof found this on a fresh `temper box --fake-backends`. On a started chat, the picker applied and persisted GPT-6 Astra · low, across a reload too, and the note says "A change applies from the next turn." The following message's request to the provider still named `gpt-6.1-sol`. The page says one thing and the run does another, on the choice the user asked for.

No root cause is claimed. By the proof's reading only, `smith/crates/smith-domain/src/route.rs`, `resumed_model` near line 1061, selects a saved window's model when main changes, and is the place to look first. It could also be anywhere from the client's change to what the worker's opening carries.

## What this plan makes true

1. **A started chat's change of model or effort is what its next turn's request uses,** as the page says. Reproduce it first, in a failing focused test at the lowest tier that shows it, then fix the cause there.
2. **Effort too:** check that the request's effort follows, whether or not the model changes.
3. **A change across providers, or to a window too small for the conversation so far:** read what happens, and leave it if it already does what `temper/docs/design/draft/model-catalogue.md`, section 3, and `temper/docs/design/client/README.md`, 3.5, say. If they don't say, decide the plain thing, record it in the design's status, and say so in the report.
4. A task keeps the model it has, until a person changes it (`model-catalogue.md`, section 4).

## Beside it

`model-catalogue-2/smith-limits` changes how smith reads a saved window's model, in the same code, and `look-round-11` is landing the picker. Whichever lands second catches up: don't wait for them, and don't take on their work.

## Limits

One or two tasks. Focused and fuzzy alone: no browser, walk or render check, by the user's rule that a plan is complete when it lands. No stored format changes unless the cause needs it.

## Sources

- `smith/crates/smith-domain/src/route.rs`
- `temper/docs/design/draft/model-catalogue.md`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
