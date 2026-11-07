# limits-heavy: temper's other heavy focused tests

The request: `inputs/request.md`. The repair of `skein/docs/foundation/testing-strategy.md`, 8.1, in its order, for the rows outside the whole-deployment worlds (about 89 CPU s of a focused suite that reads about 336 against 30). No budget, timeout or build setting changes. `walk = false`.

The implementation and closing account are complete; landing completes this plan. The suite's CPU reading follows landing.

## Parts and tasks

| Part | Task | Needs | Rows | CPU s |
|---|---|---|---|---:|
| `box` | `tasks/box-lifetime.md` | | `temper::suite` `box_listeners` (two tests move), `forge_land` | 17.6, 4.0 |
| `fake` | `tasks/fake-backends.md` | | `temper` `fake_backends::tests` | 11.5 |
| `limits` | `tasks/view-limits.md` | | `temper-client-view` `tests::limits` | 9.1 |
| `rate` | `tasks/forge-rate.md` | | `temper-engine-protocol-forge-world::suite` `rate`, `github_land` | 3.5, 2.4 |
| `gallery` | `tasks/gallery-print.md` | | `temper-client-gallery` `tests` | 12.1 |
| `scale` | `tasks/scale-restart.md` | | `temper-engine-domain-world::suite` `fabrig_scale`, `temper-box` `tests::restart` | 3.1, 2.6 |
| `rows` | `tasks/small-rows.md` | | Four single tests in `temper-client-tools` and `temper-service` | 3.7 |
| `close` | `tasks/heavy-close.md` | all of the above | This file's table and `temper/docs/development/limits-heavy.md` | |

Eight tasks, every row of the request's table and its four named tests. `temper-client-gallery`'s row is the second largest but went out in the second batch, to keep the first off files `look-round-13` is changing.

**Critical path:** any of the seven row tasks, then `heavy-close`. None of the seven needs another. The longest is unknown until the profiles say; `box-lifetime` carries the most, with its move and two rows.

## What this plan decided

- **Each task writes its own figures file** (`figures/<task>.md`, in `notes/common.md`'s form), so that no two branches meet in a table. The close builds the table from them.
- **The stop rule is in CPU s, the measure in instructions.** A cause's share of a test's instructions times that test's CPU s in the request's table gives its gain, against the request's 0.2.
- **`forge_land` goes with `box_listeners`:** one binary, one writer, and a task fewer against the request's about eight. So do `fabrig_scale` with `restart`, and the four single tests together: small independent loops, each under its own stop rule.
- **Rows only where the profile shows the fixture is the cost.** The request allows rows of one test and also forbids merging to make room. `notes/common.md`, *Rows*, tells them apart: where a test's own check costs little beside a shared fixture, it may be a row of one test; where the check is the cost, it stays a test. A test that goes is named with its holder.
- **Every bound and every order stays.** A limit test still reaches its bound, `a_goal_tree_takes_384_tasks` still takes 384, and the restart tests still run all 720 orders. Only the building gets cheaper.
- **The report goes in `temper/docs/development/limits-heavy.md`,** with the table here, since a plan's files leave main when it's done.

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
- **`temper-client-gallery`'s tests stay.** They check the states the gallery draws, through the view code the client ships, on a tree or a printed file (`temper/docs/design/client/browser.md`, 7), with no browser and no process of temper's.
- **`fabrig_scale` stays.** It's the engine domain's world at the stock limits, in memory, and `temper/docs/design/protocol/setup.status.md`, 8.3, leans on it for a fresh box's scale.
- **`temper-box`'s `tests::restart` stay.** They're step tests of the box on the simulator, and they deliver every ending order, which no process could arrange.
- **The four single tests stay.** `temper-client-tools`' two print at the shipped limits with no process, and `temper-service`'s two drive the service's step functions: a test of each behaviour where it lives.

## What the move costs the process tier

`a_hangup_stops_a_box_and_leaves_nothing_running` 8.1 and `a_killed_box_takes_its_steps_with_it` 7.8 add about **15.9 CPU s**, by the gate's medians, against `tools/tiers/budgets.toml`'s `process-temper`: 360 for the suite, 60 for a test, and a wall hard limit of 1080. The request calls that tier near its figure; its latest reading isn't in this worktree, so the margin is unknown here. Each is well within 60 alone, and at about 11 s of wall they add little past the limit. Neither figure changes. If the tier goes over, after-landing brings it back as a repair under 8.1, and the two stories' overlap above is the first place to look, with the user's say. They aren't run in this plan.

The `box-lifetime` figures confirm the overlap: both killed-owner stories observe descendant death, but `a_story_box_and_its_steps_die_when_the_test_process_is_killed` kills the owning test without asking for playground steps. The moved `a_killed_box_takes_its_steps_with_it` signs in, starts the playground, kills the box itself, and observes every retained step exiting. They exercise different ownership links and different running descendants; neither replaces the other. The new process stories were compiled, not run, in this plan. Their process-tier runtime remains unobserved.

## Overlaps this plan could not avoid

`notes/common.md`, *Beside this plan*, gives each task the list.

- **`gallery-print` and `look-round-13`:** every open task of `look-round-13` that draws a page has `temper/tools/gallery/**`. This is the one overlap in owned files. The spec keeps hunks small, leaves `samples.rs` and `worlds.rs` unless a profile names them, and has every check run over whatever states exist then. Its row-merging in `tests.rs` is the likeliest place for a merge to meet them.
- **`small-rows` and `limits-fixtures`:** both reach `temper-service`'s tests. `small-rows` owns its two tests and leaves the shared `service` and `SignInWorld` fixtures to `limits-fixtures` unless the profile puts more than 0.2 CPU s on them.
- **`scale-restart` and `limits-fixtures`:** both reach `temper/tests/engine/domain/src/**`, the same way. A cause in `jig-domain-tasks` or `jig-domain-core` would meet `ui-model-next-turn` and `model-catalogue-2`.
- **Only if a profile puts a cause there:** `fake-backends` and `box-lifetime` in `temper/crates/temper/tests/support/**`, `temper-init/template/*`, `temper-fake/src/**` and `temper-service/src/startup*` (`model-catalogue-2`); `view-limits` in `temper-client-view/src/tests/fixtures.rs` (`limits-fixtures`); `forge-rate` in `temper-engine-domain-forge` (`goal-delivery`, `look-round-13`) and with `limits-worlds` where it shares a fake forge; `small-rows` in `temper-client-tools/src/jig/**` (`model-catalogue-2/configuring-learns`).

`limits-worlds` and `limits-fixtures` were still being cut when this was written, with no tasks on their trunks, so these come from their requests and may move.

## The table

All 121 rows from the seven figures files follow, including unprofiled tests and increases. Counts are instructions (Ir); `unknown` means no individual profile, and gallery's zero means the separate test process went away while its assertions stayed in the named holder. Child-trace conventions and the CPU fallback remain in the source figures. The permanent account is `temper/docs/development/limits-heavy.md`, including every finding left unfixed.

| Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---:|---:|---|---|
| `temper::suite::box_listeners::lifetime::a_hangup_stops_a_box_and_leaves_nothing_running` | unknown | unknown | Running playground steps require the process tier | moved whole to `process::box_lifetime::a_hangup_stops_a_box_and_leaves_nothing_running`; compiled, not run there |
| `temper::suite::box_listeners::lifetime::a_killed_box_takes_its_steps_with_it` | unknown | unknown | Running playground steps require the process tier | moved whole to `process::box_lifetime::a_killed_box_takes_its_steps_with_it`; compiled, not run there |
| `temper::suite::box_listeners::a_fresh_box_holds_serves_clients_port` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::box_listeners::a_later_box_refuses_a_taken_port_naming_the_port_and_document` | 975,595,748 | 5,413,110,171 | RSA-3072 generation dominates and varies with the primes; configuration parsing follows | stays: product unchanged; command helper shared with process stories |
| `temper::suite::box_listeners::a_fresh_box_prints_its_block_and_nothing_else` | unknown | unknown | Callgrind cannot signal pidfds; CPU fallback: 0.320 s before, 0.322 s after | stays: product and assertions unchanged |
| `temper::suite::box_listeners::a_box_without_openssl_logs_sign_in_unavailable_once_and_stays_up` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::box_listeners::the_first_link_opened_at_once_waits_and_signs_in` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::box_listeners::lifetime::a_box_ignoring_hangups_runs_on` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_fresh_sittings_close_leaves_no_live_root` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_kept_store_keeps_an_earlier_sittings_live_chat` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_sitting_never_closes_an_earlier_root` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_roots_read_must_be_whole` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_readme_ignores_trailing_empty_lines` | 12,156,740 | 12,159,810 | Dynamic-loader relocations: 11,653,064 Ir (95.9%) before | stays: distinct behaviour; shared executable startup is not repaired by build settings |
| `temper::suite::forge_land::a_readme_keeps_its_last_nonempty_line_exactly` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::an_empty_park_stops_the_story` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::active_work_and_intentional_waits_continue` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_terminal_chat_is_not_an_empty_park` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::missing_chat_facts_report_the_shape_error` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::the_claude_launcher_refuses_invalid_held_directories` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_claude_configuration_starts_on_inits_limits` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_claude_configuration_replaces_openai_and_keeps_the_demo` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::stock_charters_choose_claude_before_the_demo_model` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_claude_plan_declares_without_import_or_sign_in` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::the_launcher_returns_the_login_after_build_failure` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::the_launcher_returns_the_login_after_a_signal` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::the_launcher_refuses_changed_login_metadata_on_exit` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::the_launcher_runs_only_the_claude_story_then_returns` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::the_launcher_gives_each_landing_story_its_own_store` | 79,217,172,205 | 211,172,655 | One shell and external readlink per process in each /proc snapshot | batch working-directory reads in one Python process per snapshot |
| `temper::suite::forge_land::the_launcher_runs_its_builds_through_the_wrapper_it_is_given` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_claude_source_is_verified_by_its_declared_account` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_claude_account_needs_its_login_usable_on_worker` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_claude_plan_refuses_missing_or_unusable_accounts` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_claude_plan_selects_provider_two_beside_demo_accounts` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_held_login_returns_when_validation_panics_between_boxes` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_held_login_returns_on_restart_and_failures_in_either_box` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::terminal_login_cleanup_excludes_a_restart_import` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::the_launcher_returns_the_ordinary_held_login_after_build_failure` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::the_launcher_runs_the_ordinary_return_hook_before_stopping_after_a_signal` | unknown | unknown | Launcher process snapshots; no individual instruction profile | batched snapshot; also checks scratch and store descendants stop after the return hook |
| `temper::suite::forge_land::the_launcher_refuses_changed_ordinary_held_login_metadata` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::the_launcher_records_a_failed_ordinary_return_hook` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_held_chatgpt_record_is_admitted_only_as_a_private_regular_file_never_named` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::each_landing_story_takes_its_own_store_under_one_live_store` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::the_held_records_import_and_give_back_name_the_sittings_client_directory` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::an_imported_records_path_is_printed_as_file` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::ended_delegates_are_read_whole_by_their_cursor` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::an_ended_delegate_read_refuses_live_rows_a_cut_without_a_cursor_and_a_33rd_page` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::setup_accepts_a_project_free_first_repository_row` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::setup_resolves_the_adopted_project_on_fresh_and_seeded_deployments` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::setup_refuses_missing_or_ambiguous_adopted_projects` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_landing_referee_reads_committed_tools_beyond_the_conversation` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_landing_referee_keeps_a_successful_raw_delegate_in_its_evidence` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper::suite::forge_land::a_landing_referee_requires_whole_bounded_activity` | unknown | unknown | No individual instruction profile; aggregate CPU timing only | stays: assertions unchanged |
| `temper-client-gallery::lib::tests::print_writes_every_page_and_the_bundle` | 31,170,701,091 | 8,292,374,426 | Two complete prints; repeated English phrasebook loading and bounds derivation | Prepare words and bounds once per print; print once onto an existing gallery, retaining every page, index and bundle assertion; hold whole-directory replacement separately |
| `temper-client-gallery::lib::tests::each_state_holds_what_its_name_says` | 12,608,794,546 | 5,534,470,586 | View construction is 93.28% before; words and bounds are rebuilt for each state | One pass shares prepared words and bounds and holds all four additional rules below |
| `temper-client-gallery::lib::tests::navigation_links_have_addresses_and_install_action_links_keep_their_binding` | 12,675,806,998 | 0 | View construction is 92.77% before | Row of `each_state_holds_what_its_name_says`; its helper costs 204,188,704 Ir, included in that holder |
| `temper-client-gallery::lib::tests::no_named_control_draws_a_text_child_in_any_state` | 12,470,818,651 | 0 | View construction is 94.30% before | Row of `each_state_holds_what_its_name_says`; its helper costs 5,264,990 Ir, included in that holder |
| `temper-client-gallery::lib::tests::no_state_reads_of_zero` | 12,573,446,064 | 0 | View construction is 93.93% before | Row of `each_state_holds_what_its_name_says`; its helper costs 104,150,536 Ir, included in that holder |
| `temper-client-gallery::lib::tests::each_chat_reads_its_allotment` | 1,966,208,085 | 0 | View construction is 93.73% before | Row of `each_state_holds_what_its_name_says`; its helper costs 1,096,040 Ir, included in that holder; its existing chat predicate and spend rows stay |
| `temper-client-gallery::lib::tests::the_browser_draws_each_model_name_once` | 54,346,700 | 54,341,265 | Single-sample floor: English loading is 25,355,847 Ir, bounds 6,787,821 Ir; samples array only 1,220,765 Ir | Stays: one sample needs its own view and page; public single-page construction moves its words without cloning them |
| `temper-client-gallery::lib::tests::replacing_a_gallery_removes_the_whole_previous_print` | unknown | 2,533,331 | New test, no before profile; the second complete rendering previously paid for directory replacement | Calls the same private replacement function as `print`, first for a missing directory, then over an index, stale page and nested old bundle; checks the replacement is an empty directory |
| `temper::lib::fake_backends::tests::chosen_ports_refuses_all_but_nine_nonzero_ports` | 6,219,759 | 6,221,247 | Process/loader floor; no preparation fixture | stays: all five invalid-port cases |
| `temper::lib::fake_backends::tests::fresh_preparation_is_private_and_isolates_imported_resources` | 3,123,457,580 | 879,454,153 | Optional RSA provider-key generation and repeated source setup checks | Explicit fixture environment; reuse admitted bundle limits |
| `temper::lib::fake_backends::tests::retained_peer_edits_are_checked_without_replacement` | 19,892,711,440 | 2,481,957,851 | Six complete preparations, six RSA keys, repeated source checks | One prepared fixture, restore each peer document; explicit environment; reuse limits |
| `temper::lib::fake_backends::tests::missing_required_retained_documents_refuse_without_writes` | 19,819,608,616 | 1,525,518,072 | Six complete preparations, six RSA keys, repeated source checks | One prepared fixture, restore each required document; explicit environment; reuse limits |
| `temper::lib::fake_backends::tests::oversized_source_bundle_refuses_before_sandbox_creation` | unknown | 172,875,213 | New regression test; source admission and early bundle refusal | Holds the admitted source bundle bound and refusal before sandbox writes |
| `temper::lib::fake_backends::tests::preparation_does_not_discover_optional_programs_outside_its_environment` | unknown | 497,843,702 | New regression test; fresh preparation without optional executables | Holds explicit optional-program facts and absence of an unneeded provider key |
| `temper::lib::fake_backends::tests::peer_reservations_transfer_once_and_close_with_preparation` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::explicit_bundle_initializes_without_creating_absent_source_configuration` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::explicit_bundle_leaves_existing_empty_source_configuration_untouched` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::explicit_bundle_overrides_an_unavailable_source_bundle_without_rewriting_source` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::explicit_bundle_imports_init_configuration_without_a_source_bundle` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::empty_configuration_refuses_before_creating_data_or_cache` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::initialized_source_without_bundle_refuses_before_sandbox_mutation` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::partial_sandbox_refuses_with_explicit_reset_instructions` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::reuse_keeps_effective_edits_peer_addresses_and_data` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::reservations_avoid_every_port_the_peer_check_refuses` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::reuse_atomically_replaces_bundle_and_preserves_every_other_edit` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::repeating_the_same_bundle_preserves_document_bytes_and_inode` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::invalid_replacement_bundle_preserves_retained_document` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::invalid_retained_configuration_refuses_a_valid_replacement_without_mutation` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::staging_file_collision_leaves_the_retained_document_untouched` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::reuse_refuses_a_missing_effective_bundle_and_preserves_edits` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::current_init_import_ignores_charter_files_and_rebases_paths` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::unsupported_imports_refuse_with_the_public_member_named` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::effective_commands_reopen_the_retained_directories` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::missing_provisioned_directory_refuses_before_data_or_cache_mutation` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::oversized_explicit_bundle_refuses_before_creating_data_or_cache` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::unsupported_provisioned_file_refuses_before_sandbox_creation` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::linked_provisioned_page_refuses_before_reading_its_target` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::source_public_asset_links_are_refused_before_reading_them` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::import_preserves_disabled_git_and_adds_the_fake_program` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::fake_github_machine_passes_ordinary_admission_without_document_forges` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::foreign_retained_adapter_refuses_without_resetting_or_rewriting_it` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::source_adapter_is_a_bounded_fact_without_path_discovery` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::own_app_key_is_the_seeded_apps_private_pem` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::changed_source_bytes_or_directory_require_explicit_reset_without_writes` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::a_new_sandbox_takes_the_nine_ports_its_test_named_and_a_restart_keeps_them` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper::lib::fake_backends::tests::a_named_port_held_elsewhere_refuses_a_new_sandbox_naming_it` | unknown | unknown | Shared preparation fixture; not individually profiled | Explicit absent optional executables; existing assertions unchanged |
| `temper-client-view::lib::tests::limits::a_window_full_of_small_lines_builds` | 23,174,371,947 | 493,358,090 | Page ordering: 22,711,430,720 Ir, 98.00%; pairwise comparisons and whole decoded-line clones | Unique positions use a bounded ordered map and move their lines; ordering now costs 30,407,393 Ir |
| `temper-client-view::lib::tests::limits::kept_words_at_the_threads_bytes_build` | 12,148,031,421 | 11,991,007,201 | Drawing and freeing the full newline tree; byte-at-a-time input construction costs another 158,888,502 Ir | Fill byte runs in bulk: input construction is 1,821,468 Ir; `draw_words` stays at 11,286,705,917 Ir |
| `temper-client-view::lib::tests::limits::a_window_full_of_long_words_builds` | 9,458,177,324 | 9,415,975,227 | Whole-word drawing and tree destruction; byte-at-a-time input construction costs 37,231,370 Ir | Move ordered lines and fill byte runs in bulk: input construction is 531,284 Ir; both byte patterns and shells stay |
| `temper-client-view::lib::tests::limits::a_window_full_of_markdown_builds` | 6,288,570,002 | 5,592,630,919 | Markdown reading/drawing: 4,679,171,085 Ir; page ordering: 703,019,960 Ir | Move ordered lines; ordering is 6,806,931 Ir after; all three Markdown patterns stay |
| `temper-client-view::lib::tests::limits::a_window_full_of_calls_builds` | 6,084,229,642 | 963,736,342 | Page ordering: 5,130,307,131 Ir, 84.32%; repeatedly cloning the calls and their inputs | Move ordered lines; ordering is 9,763,891 Ir after; the same calls and window remain |
| `temper-client-view::lib::tests::limits::a_request_and_a_composer_at_long_text_build` | 1,479,366,130 | 1,461,058,898 | Drawing the request and composer; byte-at-a-time input construction costs 18,612,583 Ir | Fill byte runs in bulk: input construction is 262,540 Ir; `draw_words` stays at 1,390,929,414 Ir |
| `jig-client::lib::thread::order::tests::ordering_keeps_whole_lines_and_their_room` | unknown | unknown | Not profiled: new focused regression | Checks descending order, unchanged source allocations, every line and retained capacity |
| `jig-client::lib::thread::order::tests::repeated_positions_keep_every_line_in_pairwise_order` | unknown | unknown | Not profiled: new focused regression | Checks that repeated positions keep every line and the same pairwise order |
| `temper-engine-protocol-forge-world::suite::rate::iterating_changes_installation_requests_per_hour` | 8,256,333,753 | 7,943,865,123 | Readiness and deadline scans repeated between HTTP fragments although the domain had received no input | Cache the domain schedule between answers and domain steps; 3.8% fewer instructions |
| `temper-engine-protocol-forge-world::suite::rate::moving_main_installation_requests_per_hour` | 8,321,536,270 | 8,008,904,224 | The same unchanged-domain scans, including while independently moving main | The same schedule cache; 3.8% fewer instructions |
| `temper-engine-protocol-forge-world::suite::rate::pending_changes_installation_requests_per_hour` | 1,603,071,588 | 1,552,089,088 | The same scans during read-only pending CI | The same schedule cache; 3.2% fewer instructions |
| `temper-engine-protocol-forge-world::suite::github_land::direct_landing_finish_and_two_writes` | 224,238,073 | 218,804,684 | The same scans while opening and confirming the merge | The same schedule cache; 2.4% fewer instructions |
| `temper-engine-protocol-forge-world::suite::github_land::pending_ci_across_token_hour_completes_on_fresh_token` | 5,476,039,300 | 5,234,863,065 | The same scans across pending CI, credential renewal and landing | The same schedule cache; 4.4% fewer instructions |
| `temper-engine-domain-world::suite::fabrig_scale::a_goal_tree_takes_384_tasks` | 17,457,265,311 | 9,362,561,515 | Every saved task rebuilt the tracked projection's whole plan, deeply cloning all retained task words and phases | Update an existing projection task in place; move retained tasks when appending. All 384 tasks, the capacity refusal and the restart oracle stay |
| `temper-engine-domain-world::suite::fabrig_scale::eighty_chats_are_funded_at_once` | 2,922,851,330 | 2,923,111,661 | Eighty durable admissions, cancellation and the restart/read oracle; no tracked projection in this story | Stays: the required eighty funded chats and eighty-first refusal are unchanged; no single avoidable cause reaches 0.2 CPU s |
| `temper-box::lib::tests::restart::an_aborted_serve_restarts_in_every_ending_order` | 2,465,836,366 | 2,465,838,814 | 720 complete simulator lifecycles: startup, six endpoint deliveries, replacement and teardown | Stays: all 720 orders and every assertion; sharing the startup snapshot requires broader support than a small clone |
| `temper-box::lib::tests::restart::early_reply_pipe_failure_and_busy_logs_allow_restarting` | 3,834,834,688 | 3,834,837,463 | The same 720 lifecycles, plus the early failed reply and a held log write | Stays: all 720 orders; additional turns drive the required pipe and log activity |
| `jig-domain-core::lib::tests::effects::saving_a_projection_preserves_neighbours_when_updating_or_appending` | unknown | 4,013,388 | New focused regression; no before test exists | Checks update, append, unchanged saves, emitted durable rows and retention of neighbours' word allocations |
| `temper-client-tools::lib::tests::jig::turn::a_document_sized_turn_prints_escaped_tool_names_with_the_shipped_limits` | 6,995,481,808 | 6,832,067,781 | Input construction puts each NUL separately; most remaining work validates the printed JSON | Allocate the same zero bytes once; 2.3% fewer instructions |
| `temper-client-tools::lib::tests::jig::reads::a_document_at_its_byte_bound_prints_its_maximal_json_escapes` | 5,180,449,245 | 3,363,194,720 | Rendering hexadecimal paths builds a nibble array and calls the sized writer per digit, on both measuring and writing passes, including the digest's passes | Fill exact two-byte pairs in one final-length buffer; construct the same NUL input in bulk; 35.1% fewer instructions |
| `temper-service::lib::setup::tests::lifecycle_tests::setup_malformed_queries_are_bad_requests` | 1,033,151,661 | 140,501,808 | Sixteen service constructions cost 947,830,537 Ir | Build one service; retain all sixteen requests, their 400 and page/header checks, with the target in failures; 86.4% fewer instructions |
| `temper-service::lib::accounts::tests::expiry_and_native_paste_race_take_at_most_one_redirect` | 3,361,893,352 | 1,097,274,466 | Each schedule starts and restarts a forge service although the host only uses its store and values; settling runs 512 turns even after quiescence | Construct a durable store and values directly, and stop settling when the host and store queues are quiet; retain all 54 schedules; 67.4% fewer instructions |
| `temper-client-tools::lib::tests::jig::reads::binary_paths_print_fixed_width_lowercase_hex` | unknown | unknown | New focused regression, not profiled | Exact output and measured length for empty paths, every hex digit, leading zeroes and binary bytes |

## Tests that went

Four separate gallery tests became rules in `temper-client-gallery::lib::tests::each_state_holds_what_its_name_says`. From `figures/gallery-print.md`:

- `tests::navigation_links_have_addresses_and_install_action_links_keep_their_binding`:
  held by `tests::each_state_holds_what_its_name_says`, which passed at `bbf0fb6cd3`.
- `tests::no_named_control_draws_a_text_child_in_any_state`:
  held by `tests::each_state_holds_what_its_name_says`, which passed at `bbf0fb6cd3`.
- `tests::no_state_reads_of_zero`:
  held by `tests::each_state_holds_what_its_name_says`, which passed at `bbf0fb6cd3`.
- `tests::each_chat_reads_its_allotment`:
  held by `tests::each_state_holds_what_its_name_says`, which passed at `bbf0fb6cd3`.

All other figures files say none. The two lifetime stories moved to the process tier and are not tests that went.
