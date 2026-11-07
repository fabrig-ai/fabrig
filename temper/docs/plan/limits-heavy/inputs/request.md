# limits-heavy: temper's other heavy focused tests

temper's focused suite has a limit of 30 CPU s and reads about 336 (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1). 119 tests outside the whole-deployment worlds read at least 0.25 CPU s, about 89 CPU s together. This plan is 8.1's repair for them, in its order. Other plans take the whole-deployment worlds and the fixtures of the many small tests.

Medians of three gate runs, CPU s:

| Where | Tests | CPU s | Largest |
|---|---:|---:|---|
| `temper::suite` `box_listeners` | 8 | 17.6 | `lifetime::a_hangup_stops_a_box_and_leaves_nothing_running` 8.1, `lifetime::a_killed_box_takes_its_steps_with_it` 7.8 |
| `temper-client-gallery` `tests` | 30 | 12.1 | `print_writes_every_page_and_the_bundle` 4.2; four others at 1.8 each |
| `temper` `fake_backends::tests` | 50 | 11.5 | `retained_peer_edits_are_checked_without_replacement` 2.2, `missing_required_retained_documents_refuse_without_writes` 1.5 |
| `temper-client-view` `tests::limits` | 6 | 9.1 | `a_window_full_of_small_lines_builds` 2.7, `kept_words_at_the_threads_bytes_build` 2.5, `a_window_full_of_long_words_builds` 1.9 |
| `temper::suite` `forge_land` | 44 | 4.0 | `the_launcher_gives_each_landing_story_its_own_store` 1.1 |
| `temper-engine-protocol-forge-world::suite` `rate` | 3 | 3.5 | two at 1.6 |
| `temper-engine-domain-world::suite` `fabrig_scale` | 2 | 3.1 | `a_goal_tree_takes_384_tasks` 2.7 |
| `temper-box` `tests::restart` | 8 | 2.6 | |
| `temper-engine-protocol-forge-world::suite` `github_land` | 8 | 2.4 | |

Also: `temper-client-tools` `tests::jig::turn::a_document_sized_turn_prints_escaped_tool_names_with_the_shipped_limits` 1.1 and `reads::a_document_at_its_byte_bound_prints_its_maximal_json_escapes` 0.6, `temper-service` `setup::tests::lifecycle_tests::setup_malformed_queries_are_bad_requests` 1.1 and `accounts::tests::expiry_and_native_paste_race_take_at_most_one_redirect` 0.9.

## What's wanted, per row, largest first

**1. Does it belong in the focused suite?** `box_listeners::lifetime` starts real boxes, with 11 s of wall to 8 CPU s. If what it checks is the process tier's (`temper/docs/design/domain/testing.md`, 2.6), it moves there, with the tier's budget as it stands: that tier is near its figure, so say what the move costs it, in `plan.md`. If it belongs, it stays. Don't move any test for its cost. `fake_backends::tests` runs a fake machine's preparation: read whether each belongs where it is too.

**2. If it belongs, why is it slow?** For each: profile, name the cause and fix it. Starting points, from names alone and not findings: does a gallery test render every page because its check needs them, or because its fixture does? A fixture built once per process is built once per test, so tests with one fixture and checks that cost little beside it may be rows of one test (section 8, "A test is a process"), or may read the fixture's output from one place. Does a limit test build at a bound in linear time, and is what it builds used? Does a `rate` test run an hour of simulated requests where fewer show the rate?

**3. A table in `plan.md`,** one row per test touched, and a report of before and after in instruction counts, with causes found and not fixed.

Stop when the next cause's gain is under about 0.2 CPU s. At most about eight tasks. `walk = false`.

## Sources

Read by section, too large to be sources: `temper/docs/design/domain/testing.md`, 2.6 and 6.1; `skein/docs/foundation/testing-strategy.md`, 8 and 8.1.

- `temper/crates/temper/tests/suite/box_listeners.rs`
- `temper/crates/temper-client-view/src/tests/limits.rs`
- `temper/tools/gallery/Cargo.toml`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories. No walk, render check or throwaway driver. A new story in a slow tier is written and compiled, not run.
- **8.1's order, always** (`skein/docs/foundation/testing-strategy.md`, 8.1): does the test belong in this suite; if it does, can it be made faster, at the cause; if neither, the measurements go in the report and the test stays. What a test checks decides its place, never its cost. Moving a test to escape a budget is raising it.
- **Find the cause with a profile before changing anything** (section 8, "A slow test is understood, never hidden"). `valgrind --tool=callgrind` on one test by name gives instruction counts that a loaded machine doesn't move; quote them before and after. CPU seconds on these machines swing by a third between runs.
- **No budget, timeout, deadline or build setting changes.** Test builds stay unoptimized. If a profile shows a cost no test can avoid, say so in the report, and leave it.
- **Nothing is lost.** A test that goes names what holds its behaviour now, by test name, in `plan.md`, and that holder exists and passes first. A test past its guide isn't merged with another or named an exception to make room (`temper/docs/design/domain/testing.md`, 6.1).
- **A product change is fine where the profile puts the cause,** at the size the cause needs, with its own focused test, and no change of behaviour. A cause in skein, smith or jig is fixed there if small and clearly theirs; otherwise report it.
- **Decide, don't ask.** A stored format or protocol may change freely, with no compatibility. A small edit just outside a task's file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
