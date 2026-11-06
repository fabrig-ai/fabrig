# limits-heavy: temper's other heavy focused tests

The request: `inputs/request.md`. The repair of `skein/docs/foundation/testing-strategy.md`, 8.1, in its order, for the rows outside the whole-deployment worlds (about 89 CPU s of a focused suite that reads about 336 against 30). No budget, timeout or build setting changes. `walk = false`.

## Parts and tasks

| Part | Task | Needs | Rows | CPU s |
|---|---|---|---|---:|
| `box` | `tasks/box-lifetime.md` | | `temper::suite` `box_listeners` (two tests move), `forge_land` | 17.6, 4.0 |
| `fake` | `tasks/fake-backends.md` | | `temper` `fake_backends::tests` | 11.5 |
| `limits` | `tasks/view-limits.md` | | `temper-client-view` `tests::limits` | 9.1 |
| `rate` | `tasks/forge-rate.md` | | `temper-engine-protocol-forge-world::suite` `rate`, `github_land` | 3.5, 2.4 |

The rest of the plan is still being cut: `temper-client-gallery`'s tests (12.1), `fabrig_scale` and `temper-box`'s `tests::restart` (3.1, 2.6), the single tests in `temper-client-tools` and `temper-service` (3.7), and a closing task that builds this file's table and the report from each task's figures.

**Critical path, as it stands:** none of these tasks needs another, so the path is the longest one of them, then the close.

## What this plan decided

- **Each task writes its own figures file** (`figures/<task>.md`, in `notes/common.md`'s form), so that no two branches meet in a table. The close builds the table from them.
- **The stop rule is in CPU s, the measure in instructions.** A cause's share of a test's instructions times that test's CPU s in the request's table gives its gain, against the request's 0.2.
- **`forge_land` goes with `box_listeners`:** one binary, one writer, and a task fewer against the request's about eight.

## Belonging

`temper/docs/design/domain/testing.md`, 2.6, and `skein/docs/foundation/testing-strategy.md`, 2.9 and 8.1, are the test. What a test checks decides its place, never its cost. This is a cut's reading of the design and the tests, not a run.

- **`box_listeners::lifetime::a_hangup_stops_a_box_and_leaves_nothing_running` moves to the process tier.** It starts the playground's steps through `temper ctl forge setup --playground`, signals the box, and restarts it. 2.6's *the playground and the set up* puts stories like it there: "they need the box's processes and their deaths". Lower worlds hold the order and the decision (`temper-box`'s `a_hangup_stops_in_the_usual_order_and_a_second_changes_nothing`, `a_termination_after_a_hangup_closes_the_steps`); only processes show that nothing is left.
- **`box_listeners::lifetime::a_killed_box_takes_its_steps_with_it` moves too,** for the same reason: a parent-death signal, with real steps. Its nearest neighbour is the process tier's `a_story_box_and_its_steps_die_when_the_test_process_is_killed`, which kills the test and watches the box and its descendants die on a box with no playground steps asked for. Neither replaces the other at this reading, and they aren't merged: 6.1 forbids merging to make room. `box-lifetime` reports how much they overlap, for a later choice.
- **`box_listeners::lifetime::a_box_ignoring_hangups_runs_on` stays.** It asks for no step: it is one path through `main`, the signal mask left as inherited. Strategy 2.9 keeps that in the focused suite ("one per path through `main`, not one per behaviour").
- **`box_listeners`' other five stay,** for the same reason: `a_fresh_box_holds_serves_clients_port`, `a_later_box_refuses_a_taken_port_naming_the_port_and_document`, `a_fresh_box_prints_its_block_and_nothing_else`, `a_box_without_openssl_logs_sign_in_unavailable_once_and_stays_up` and `the_first_link_opened_at_once_waits_and_signs_in` are each a path through `temper box`'s `main` with its group dormant. 2.6's *focused wiring* names one such test for `temper serve`. Whether a box's first-run paths should be so few is a fair question, but it isn't this plan's: they're cheap, and cost must not decide it either way.
- **`fake_backends::tests` stay.** Reading the names and `rg 'Command::new|paths::binary'`: none spawns the binary, and they call `prepare` and its neighbours directly on a scratch directory. They hold decisions of that code, with `current_exe` only as a fixture path. So they're no path through `main` and no process's. `fake-backends` confirms it from the bodies.
- **`temper-client-view`'s `tests::limits` stay.** They hold `temper/docs/design/client/README.md`, 5.4, the derived page bounds, on a tree with no process.
- **`rate` and `github_land` stay.** 6.1 names rate scenarios for the focused suite: "their behaviour defines their tier". `github_land` is a protocol world's story (2.2).
- **`forge_land` stays,** at this reading: its launcher tests run `scripts/proof-live.sh` against a stubbed `cargo`, which holds the launcher's choices without a slow tier.

## What the move costs the process tier

`a_hangup_stops_a_box_and_leaves_nothing_running` 8.1 and `a_killed_box_takes_its_steps_with_it` 7.8 add about **15.9 CPU s**, by the gate's medians, against `tools/tiers/budgets.toml`'s `process-temper`: 360 for the suite, 60 for a test, and a wall hard limit of 1080. The request calls that tier near its figure; its latest reading isn't in this worktree, so the margin is unknown here. Each is well within 60 alone, and at about 11 s of wall they add little past the limit. Neither figure changes. If the tier goes over, after-landing brings it back as a repair under 8.1, and the two stories' overlap above is the first place to look, with the user's say. They aren't run in this plan.

## Overlaps this plan could not avoid

`notes/common.md`, *Beside this plan*, gives each task the list. `fake-backends` and `box-lifetime` may reach `temper/crates/temper/tests/support/**`, `temper-init/template/*`, `temper-fake/src/**` and `temper-service/src/startup*` (`model-catalogue-2`) only if a profile puts a cause there. `view-limits` may reach `temper-client-view/src/tests/fixtures.rs` (`limits-fixtures`). `forge-rate` may reach `temper-engine-domain-forge` (`goal-delivery`, `look-round-13`). This first batch's owned files are this plan's alone at the cut.

## The table

Written by the closing task: one row per test touched, before and after in instruction counts, with causes found and not fixed.
