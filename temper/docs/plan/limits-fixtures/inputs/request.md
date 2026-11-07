# limits-fixtures: what temper's many small tests cost before they check anything

temper's focused suite has a limit of 30 CPU s and reads about 336, in 10,197 tests (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1). A test's process costs 2 to 3 ms before it checks anything. These tests cost many times that, because of what each one builds first. This plan is 8.1's second question for them: their place is right, so make them cheaper at the cause. Other plans take the whole-deployment worlds and the other heavy tests.

Medians of three gate runs, CPU s, with a binary's floor as the 5th percentile of its tests:

| Binary | Module | Tests | CPU s | Median ms | Binary's floor, ms |
|---|---|---:|---:|---:|---:|
| `temper-engine-domain-world::suite` | `forge_routes` | 425 | 31.2 | 55 | 14 |
| | `root_routes` | 125 | 3.4 | 25 | |
| | all | 893 | 52.9 | 33 | |
| `temper-service` | `setup` | 238 | 24.9 | 77 | 6 |
| | `tests::stock` | 74 | 3.1 | 19 | |
| | all | 1,111 | 46.3 | 18 | |
| `temper-client-world::suite` | all | 384 | 24.3 | 49 | 23 |
| `temper-client-view` | all but `tests::limits` | 1,427 | 22.7 | 13 | 8 |
| `temper::suite` | `startup` | 89 | 5.3 | 26 | 6 |
| | `forge` | 31 | 2.1 | 44 | |

For comparison, `temper-client-domain`'s 1,036 tests read a 2.6 ms floor, and `temper-engine-domain`'s 2.4 ms.

## What's wanted

**1. For each row, what's built before the first assertion?** Profile the cheapest test of that module (the floor), and a median one. Where a floor is well above a process's 2 to 3 ms, something runs for every test whatever it checks: a stock configuration parsed, a charter or schema decoded, stylesheets, a fake peer started, a store created and written, a large static. **Fix it there:** "a fixture many tests share is cheap to build, or a constant", and "a test is cheaper by its algorithm first" (section 8). A cost in the fixture may be a cost in the product, and a person's box pays it too, so prefer the product fix.

**2. What each `forge_routes` and `setup` test does beyond its fixture.** They read 55 and 77 ms where the strategy's guide for an in-memory world test is 200 ms but the suite's limit needs far less. Find what most of them repeat, and fix that. `forge_routes` and `setup` are the two largest single sets in the suite.

**3. Report before and after, per row, in instruction counts** for the tests profiled, with the cause of each and whether it's fixed. Causes found and not fixed get a line, with what fixing them would take.

Keep tests as they are unless a test's own setup is the cause: rows of table tests are another plan's. No test leaves its suite in this plan. Largest first, and stop when the next cause's gain is under about 1% of a binary. At most about eight tasks. `walk = false`.

**Overlaps:** other plans add tests to these crates all the time. Expect catch-ups. Keep a fixture's change in one task.

## Sources

Read by section, too large to be sources: `skein/docs/foundation/testing-strategy.md`, 8 and 8.1.

- `temper/tests/engine/domain/Cargo.toml`
- `temper/crates/temper-service/Cargo.toml`
- `temper/tests/client/Cargo.toml`
- `temper/crates/temper-client-view/Cargo.toml`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories. No walk, render check or throwaway driver. A new story in a slow tier is written and compiled, not run.
- **8.1's order, always** (`skein/docs/foundation/testing-strategy.md`, 8.1): does the test belong in this suite; if it does, can it be made faster, at the cause; if neither, the measurements go in the report and the test stays. What a test checks decides its place, never its cost. Moving a test to escape a budget is raising it.
- **Find the cause with a profile before changing anything** (section 8, "A slow test is understood, never hidden"). `valgrind --tool=callgrind` on one test by name gives instruction counts that a loaded machine doesn't move; quote them before and after. CPU seconds on these machines swing by a third between runs.
- **No budget, timeout, deadline or build setting changes.** Test builds stay unoptimized. If a profile shows a cost no test can avoid, say so in the report, and leave it.
- **Nothing is lost.** A test that goes names what holds its behaviour now, by test name, in `plan.md`, and that holder exists and passes first. A test past its guide isn't merged with another or named an exception to make room (`temper/docs/design/domain/testing.md`, 6.1).
- **A product change is fine where the profile puts the cause,** at the size the cause needs, with its own focused test, and no change of behaviour. A cause in skein, smith or jig is fixed there if small and clearly theirs; otherwise report it.
- **Decide, don't ask.** A stored format or protocol may change freely, with no compatibility. A small edit just outside a task's file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
