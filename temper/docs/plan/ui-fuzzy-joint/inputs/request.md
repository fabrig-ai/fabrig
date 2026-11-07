# ui-fuzzy-joint: jig's fuzzy suite back under its 120 CPU seconds

A budget red, and this plan's owner took it over from `model-catalogue-1`, which landed it. Over budget is red (skein's `docs/foundation/testing-strategy.md`, 8.1). Repair it by its questions, in order: does the test belong, and can it be made faster. A tier move or a raise isn't a repair, and budgets are the user's.

## What's known, from the queue's full batches on 2026-11-06

- jig's fuzzy suite read 67.2 to 70.9 CPU s up to 20:20Z, then 152.1 to 158.0 in all 7 full batches from `model-catalogue-1`'s landing at 20:54Z (`8f75c514e2`) to 23:06Z. The test count stayed at 131. The only other code changes between the last low batch's landing (`7106dfea9c`) and that plan's parent (`e7bb41d364`) are deleted plan files, so it's that plan.
- It's all in `jig-ops-world::suite fuzzy::joint`. Wall per test, before and after: `p_story_survives_seeded_commit_crashes_and_random_presses` 2.1 to 15.6 s, `q_story_survives_seeded_commit_crashes_reload_and_random_presses` 1.1 to 11.5 s, `q_asks_for_an_environment_in_a_chat_and_accepts_what_it_costs` about 0.45 to 4.9 s. That binary's 17 fuzzy tests read 103 of the 153.
- `fuzzy::client::*`, in the same binary, didn't move, so it's the joint world, not the client as a whole.
- A callgrind profile of `q_asks_for_an_environment...` at `7ffd748466`, 34.8G instructions: `Bytes::pump` 56%, called with `Net::advance` 186,412 times (26%); `World::build` 41%, 882 view builds at about 16M instructions each, mostly `tree::sections::inbox` and `ObjectsBorrowed::get` cloning (18%).

## What isn't known

The cause. There's no profile from before that landing, so whether builds, pumps or a frame's size grew isn't known: take one, at `e7bb41d364`, to compare. Suspects, read only: `Clients`' new `server_finished` path in `jig/testing/jig-test-service/src/clients.rs`; `picker: true` in `jig/testing/jig-client-kit/src/joint/component.rs`; and whether a Frame now changes more often in this world. **If a Frame does change more often, real clients may pay for it too, and that's the fix to make,** in the code under test rather than in the test.

## What's wanted

1. Find the cause by comparing the two profiles, and say it in the report with the figures.
2. Fix that cause, at its lowest place. Each story keeps every check, its seeds and its number of presses: a story made to do less isn't a repair, unless what it stopped doing is shown repeated work.
3. jig's fuzzy suite reads under 120 CPU s, and close to the 67 to 71 it read before if the cause allows, measured by the fuzzy profile alone: `cargo nextest run --profile fuzzy -p jig-ops-world`. Run it alone: `-E 'test(fuzzy::joint::q_asks_for_an_environment)'` reproduces it. Report figures before and after, on the same machine.
4. If it can't get under 120 without changing what a story checks, stop with a `Question:` carrying the measurements, for the user.

## Limits

One or two tasks. Focused and fuzzy alone. `model-catalogue-2` is touching the catalogue and offer: whichever lands second catches up, and don't take on its work. New cases are rows of an existing table test.

## Sources

- `jig/testing/jig-test-service/src/clients.rs`
- `jig/testing/jig-client-kit/src/joint/component.rs`

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
