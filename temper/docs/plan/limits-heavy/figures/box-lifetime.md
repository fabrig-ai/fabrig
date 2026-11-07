# box-lifetime: figures

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

## Measurement and checks

The unchanged base was `0548af58892589416dd169ae2a09b4a95738294b`.
Both moved tests passed by their exact names in `suite` there: killed-box
10.065 s wall; hangup 12.675 s wall when run alone. The first combined
run's hangup failed with `Set up failed: fetch-1 exited 1`; its isolated
run passed. All assertions moved unchanged. After the move, the `process`
binary compiled with `cargo nextest run -p temper --test process --no-run`;
the moved stories were not run there. The six remaining listener tests
and all 44 forge-landing tests passed before and after the move.

Profiles use unoptimized tests, one exact test through nextest, with
`NEXTEST_DOUBLE_SPAWN=0`. Listing profiles are excluded. Counts for child
traces sum the test and descendant files. They include forked shell
profiles, and the launcher counts depend on how many processes the
machine has; they are measurements of this run, not fixed expectations.
Before and after used the same machine and commands.

The launcher profile used Callgrind's `--trace-children=yes` and
`--trace-children-skip=/usr/bin/readlink` with the fuzzy profile and
`--ignore-default-filter`. A full child trace first reached the unchanged
600-second limit and is incomplete; it is not the table's before count.
The comparable trace passed before (185.704 s wall) and after (21.684 s).
Its shell profiles fell from 3,853 / 79,148,169,212 Ir to 9 / 17,302,726 Ir.
External readlink executables are excluded on both sides. Python now
accounts for 165,089,256 Ir across four executions, including the existing
Claude admission. Only working-directory links and entry names are read.
Kept-store processes remain excluded from cleanup.

The refused-port trace includes children without exclusions. Its OpenSSL
RSA-3072 generation alone was 704,660,593 Ir before and 5,142,084,149 Ir
after. The unchanged temper init/box processes together were 253,648,112
and 253,716,953 Ir. This is random key-generation variation, not a product
regression. Configuration parsing in the refused box cost 165,005,873 Ir,
within setup checking's 191,518,590 Ir.

The fresh-box child trace exhausted its 60-second real-clock patience
while stopping. Callgrind reported unsupported syscall 424
(`pidfd_send_signal`), and cleanup then panicked too; that incomplete run
is not an instruction measurement. It passed normally before and after.
The task's CPU fallback uses the median of three separate nextest runs,
less the median overhead of the trivial README test in the same binary.
The shell's `time` builtin measured user plus system CPU, including waited
children (`/usr/bin/time` was absent); no measurement driver was added.
Runs were serial with `-j 1` and `NEXTEST_DOUBLE_SPAWN=0`.

| Selection | Before raw CPU s (three runs) | After raw CPU s (three runs) | Before adjusted median | After adjusted median |
|---|---|---|---:|---:|
| Trivial README calibration | 0.867, 0.936, 0.908 | 0.925, 0.898, 0.841 | 0 | 0 |
| Six remaining listeners | 3.500, 3.216, 3.338 | 3.283, 3.185, 3.184 | 2.430 | 2.287 |
| All 44 forge-landing tests | 15.404, 14.495, 13.800 | 2.776, 3.264, 3.125 | 13.587 | 2.227 |
| Fresh-box block alone | 1.228, 1.290, 1.204 | 1.582, 1.183, 1.220 | 0.320 | 0.322 |

Calibration medians were 0.908 s before and 0.898 s after. The six have no
product change and their difference is measurement/key-generation noise.
The forge row saves 11.360 CPU s (83.6%) on these measurements. No budget,
timeout or build setting changed.

The added cleanup assertion was checked by hand: suppressing the
snapshot's PID output left both scratch/store descendants alive and the
exact signal/return-hook test failed naming both survivors. Restoring
output made that test pass. No test was removed or merged.

## Found and not fixed

- RSA-3072 key creation is required for each fresh sign-in provider. The
  refused-port profiles above show its variable cost. Sharing private
  keys across real boxes, shortening the key or changing the algorithm
  would change the product's behaviour; none is done.
- Setup configuration parsing: 165,005,873 Ir in the refused box, with
  191,518,590 Ir in setup checking. A person's box pays this too. Reusing
  one parsed document across startup validation and setup would require
  restructuring that boundary; the whole parsing share is about 0.29 CPU s
  when conservatively projected over the request's six-test 1.7 s, and
  eliminating all parsing is impossible. It remains recorded rather than
  hidden behind a fixture or build setting.
- Shared executable relocation: 11,653,064 Ir, 95.9% of the median README
  test. Distinct behaviours remain separate tests. Changing build settings
  is forbidden; separating the binary is outside this task's fixed layout.
- The launcher still starts Python for each snapshot (165,089,256 Ir across
  four Python executions including admission in the profiled story), plus
  its existing notice follower and evidence tools. A native snapshot helper
  could remove interpreter startup but would introduce another shipped
  launcher component. These remaining costs are recorded; no budget is
  raised to absorb them.

## Tests that went

None. Two stories moved, rather than being removed. The request's medians
transfer about 15.9 CPU s to the process tier. The existing
`a_story_box_and_its_steps_die_when_the_test_process_is_killed` kills the
owning test and observes its box and descendants, without requesting
playground steps. The moved killed-box story signs in, starts the
playground, kills the box itself, and observes every retained step exiting.
They overlap in descendant death after owner death, but exercise different
ownership links and different running descendants; neither replaces the
other. Process-tier runtime and the macOS build remain unobserved here.
