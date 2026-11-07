# limits-fuzzy: temper's fuzzy suite under its limit

temper's fuzzy suite has a limit of 120 CPU s (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1) and reads 125 to 128 in 81 tests, on main after `limits-worlds`. It needs to hold under the limit on a machine whose figures swing, so the aim is about 15 CPU s out, and more if it's there. This plan is 8.1's second question for it. Once it holds, the limit fails merges.

Medians of the latest gate runs, CPU s:

| World | Tests | CPU s | Looked at by `limits-worlds`? |
|---|---:|---:|---|
| `temper-real-loop-world` | 18 | 57 | Yes: its figures, retired with that plan, list the parser (7 to 9%), string measuring and byte searches as what's left |
| `temper-engine-domain-world` | 26 | 27.5 | **No** |
| `temper-conformance-world` | 15 | 21 | Yes: cuts now fork at each commit |
| `temper-one-process-world` | 4 | 11 | Partly: only focused stories |
| `temper-codecs-world` | 5 | 5 | **No** |

Largest not looked at: `fuzzy::one_process::drawn_kills_keep_the_conformance_rules_and_answer_adoptions_...` 4.9, `fuzzy::one_process::drawn_power_losses_keep_the_conformance_rules` 4.7, `fuzzy::public_wire::public_records_decoders_never_panic_and_reencode_the_same` 3.9, `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut` 3.5, and `fuzzy::walking::fuzzy_walking_shard_0` to `_7`, 18 together.

## What's wanted, largest gain first

**1. Profile what hasn't been profiled:** `temper-engine-domain-world`'s walking shards and escalation sweep, the two `one_process` sweeps and `public_wire`. Name each cause, fix the largest, and stop when the next is under about 2% of a sweep. Setup rebuilt per drawn world, a restart replaying what a snapshot could hold, a check that scans the whole state per step, and encoding done again are the kinds of cause to look for. These are starting points, not findings.

**2. A sweep keeps what it checks.** The number of worlds or seeds a sweep draws isn't this plan's to lower, nor is any crash, invariant or ending dropped. A sweep's rate of worlds per CPU second is what goes up.

**3. Whatever the real loop's remaining causes offer cheaply,** from `limits-worlds`' figures, in the retired plan's `figures/close.md`, if a change is small and takes more than about 2 CPU s out of the suite.

**4. A report:** each sweep touched, before and after in instruction counts, and the causes not fixed.

At most about five tasks. `walk = false`.

## Sources

Read by section, too large to be sources: `skein/docs/foundation/testing-strategy.md`, 8 and 8.1; `temper/docs/design/domain/testing.md`, 6.1.

- `temper/tests/one-process/tests/suite/fuzzy/one_process.rs`
- `temper/tests/engine/domain/tests/suite/fuzzy/escalation.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories. No walk, render check or throwaway driver. A new story in a slow tier is written and compiled, not run.
- **8.1's order, always** (`skein/docs/foundation/testing-strategy.md`, 8.1): does the test belong in this suite; if it does, can it be made faster, at the cause; if neither, the measurements go in the report and the test stays. What a test checks decides its place, never its cost. Moving a test to escape a budget is raising it.
- **Find the cause with a profile before changing anything** (section 8, "A slow test is understood, never hidden"). `valgrind --tool=callgrind` on one test by name gives instruction counts that a loaded machine doesn't move; quote them before and after. CPU seconds on these machines swing by a third between runs.
- **No budget, timeout, deadline or build setting changes.** Test builds stay unoptimized. If a profile shows a cost no test can avoid, say so in the report, and leave it.
- **Nothing is lost.** A test that goes names what holds its behaviour now, by test name, in `plan.md`, and that holder exists and passes first. A test past its guide isn't merged with another or named an exception to make room (`temper/docs/design/domain/testing.md`, 6.1).
- **A product change is fine where the profile puts the cause,** at the size the cause needs, with its own focused test, and no change of behaviour. A cause in skein, smith or jig is fixed there if small and clearly theirs; otherwise report it.
- **Decide, don't ask.** A stored format or protocol may change freely, with no compatibility. A small edit just outside a task's file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
