# ui-trial-account: an account just connected is the one tried

The user, from a journey on a Claude account, 2026-11-07: "this looks and works great, what if we perform the same "it works" check also for chatgpt? for consistency and to double check". A first account has the check already (`temper/docs/design/domain/trial.md`; `model_check.rs`). What they've agreed is missing is the double check: **a later account's check runs on that account.**

Today, by reading and not run: a trial names no `llm`, so it selects as a new chat does. So a second account's trial may answer as another account, with `took` naming which, and that account's row holding the line (`temper/docs/design/draft/model-check.status.md`, the departure on section 2; `trial.md`, 2, 6.2 and 8). Someone who adds ChatGPT beside Claude, or Claude beside ChatGPT, can get no proof of the account they just connected.

## What this plan makes true

1. **The account just connected is the one tried,** whichever provider comes first, in either order, and also for a second account of one provider. Its own row shows the steps, the answer or the failure, and "Try again" retries that account.
2. **The first usable account keeps today's trial, naming no `llm`,** for its reason: a trial that named its model could pass while chats still hung.
3. **How a later trial is held to its account is the cut's:** naming its account's first offered model, or an account preference on the trial's task (jig's `domain/accounts.md`, 4.3), whichever is smaller and leaves the first trial as it is. Say which in `plan.md`, and in `trial.md` and `model-check.status.md`, whose departure on section 2 goes or narrows.
4. **A story for both orders, ChatGPT then Claude and Claude then ChatGPT, and a second account of one provider,** written and compiled, not run if it's in a slow tier, as a row or step of an existing one. No new browser test.

## Limits

One or two tasks. Focused and fuzzy alone: no browser, walk or render check, by the user's rule that a plan is complete when it lands. A stored format may change in place; name it in `plan.md`, as temper/migration's plans share the engine. `ui-forge-identity` and `model-catalogue-3` are on accounts beside it: whichever lands second catches up.

## Sources

- `temper/docs/design/domain/trial.md`
- `temper/docs/design/draft/model-check.status.md`

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
