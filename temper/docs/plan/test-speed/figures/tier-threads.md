# tier-threads: measurements

2026-11-06. Slot 6, machine `debian-32gb-hel1-14`. The fake-box port fix stands; `process-temper` stays at one thread under the task's flake fallback.

## History and changes

`git log -S 'test-threads = 1' -- .config/nextest.toml` traces the setting to `070bf648b5`, which restored tree-scoped profiles without explaining this setting. Later matching commits remove other profiles without changing process-temper. `temper/docs/design/domain/testing.md`, 2.6 now specifies three concurrent stories and gives no reason for one.

A fresh fake sandbox now passes `[0; 5]` to init. Ordinary init and a person's box retain their preferences. An imported init-shaped source and a retained sandbox keep their addresses. The help example names the chosen `PORT`; its page is 11,959 bytes against 12,288.

`temper-sign-in-callback`, max-threads 1, groups the three `fake_backends::walk::sign_in::` tests: ChatGPT's registered redirect binds fixed port 1455. The group remains with the concurrency fallback. The listener/path audit found no other group to add: ordinary process fixtures use `support::free_addresses` or `support::own_listener_files`, scratch paths are their own, and `cache_forgejo_binaries` copies the read-only pinned sources into each scratch's cache. The sharing-cache story shares only inside its own scratch.

The five-number search found ordinary-init preferences, deterministic configuration/rendering fixtures and synthetic printed links in the other readers. Those do not depend on a fresh fake box's chosen ports. No occurrence was found under `temper/tests/client-browser`. Committed journey recordings were left intact.

## Method and runs

Each whole run used `heavy python3 -I -B tools/tiers/run --cpu DIR temper process`, then `python3 -I -B tools/tiers/cpu-summary DIR --budgets tools/tiers/budgets.toml --pair process-temper --json`. Builds and tests ran sequentially through heavy, with `TEMPER_BUNDLE=none`. The table uses cpu-summary's test-tree CPU and tier wall, excluding prerequisite builds. No retries, ignores, budgets, global-timeout or wall limits were changed: 360 CPU s/suite, 60 CPU s/test, 1080 s wall hard limit, 25 min global-timeout.

- Base: `70c981f961377cc5373dc068683b32c87e8d3c42`, one thread, before any edits.
- Trial: that base plus the task's uncommitted port/test/group/help/status changes, three threads, before final rustfmt. This is the exploratory trial, not a proving run of a committed head.
- Final: `d44d39c7970851af0ad1de1a3a2a21d92b596897`, one thread, including the fallback and rustfmt. The following figures commit changes this document alone.

| Run | Threads | Tests passed/failed | Wall s | CPU s | CPU budget | First sampled load (1/5/15 min) | Peak sampled 1-min load |
| --- | ---: | --- | ---: | ---: | --- | --- | ---: |
| Base | 1 | 51/0 | 503.889284 | 390.484907 | over | 11.84/8.62/8.44 | 24.77 |
| Loaded trial | 3 | 49/2 | 203.702393 | 349.679284 | within | 2.97/6.28/8.33 | 31.63 |
| Final | 1 | 51/0 | 506.755667 | 367.841645 | over | 6.26/12.58/11.83 | 56.15 |

All three runs selected 51 tests and skipped the same two ignored tests. Each had zero starved tests, starved runs, unobserved attempts, leaked descendants and incomplete records. CPU-summary's "within budget" for the trial does not turn its two test failures into a pass. The base was already 30.485 CPU s over budget; the final run is 7.842 CPU s over. The inherited suite-budget red remains open.

Load was sampled about once per second using `os.getloadavg()`, alongside process names/PIDs; no foreign environments or worktrees were read. Peak here means the maximum observed one-minute load average, not instantaneous utilization. The base sampler started 26.089 s after the first test; its initial load and any earlier peak are unknown. The other two samplers cover their runs from within one second of the first test.

| Run | First test UTC | Last test ends UTC | First load sample UTC | Samples |
| --- | --- | --- | --- | ---: |
| Base | 2026-11-06T10:33:22.857588+00:00 | 2026-11-06T10:41:46.746872+00:00 | 2026-11-06T10:33:48.946297+00:00 | 449 |
| Loaded trial | 2026-11-06T10:43:50.570179+00:00 | 2026-11-06T10:47:14.272572+00:00 | 2026-11-06T10:43:51.545726+00:00 | 195 |
| Final | 2026-11-06T10:53:02.090449+00:00 | 2026-11-06T11:01:28.846116+00:00 | 2026-11-06T10:53:02.996792+00:00 | 480 |

**Loaded condition:** another process-temper nextest invocation, PID 356760, was running when the three-thread trial began (own nextest PID 358803). The process-name sample at 10:43:51 UTC contains both; a `ps` command-line observation at 10:43:52 UTC identified the other invocation's process-temper profile. Other compiler work joined during the trial. The base also overlapped other cargo/rustc/clippy work; the final run started beside another nextest invocation and rustc work and reached a sampled peak load of 56.15. No extra build was started by this task to manufacture load.

The trial was 300.187 s shorter than the base, but failed twice. Aggregate CPU was 40.806 s lower despite the extra concurrency; differing builder load prevents assigning that change to threads. Relative to the final one-thread run, 12 tests used more than 25% extra CPU in this single trial, including shared-cache restart (14.797 to 20.861 s), a worker killed mid-run (2.142 to 6.744 s), and the failed turn-landing story (23.440 to 30.594 s). Tiny tests also show large percentages for millisecond differences. The final one-thread run took 2.866 s more wall and 22.643 s less CPU than the base. These single samples show both beneficial overlap and worse individual costs, not a repeatability result.

## Failures and isolated checks

The following are retained excerpts from the captured logs. The temporary collectors and full transcripts were removed before pushing. Both isolated reruns used the same candidate code and `heavy cargo nextest run --profile process-temper -p temper --test process -E 'test(=NAME)'`, with `TEMPER_TEST_CLAUDE` naming the built fake-claude sibling.

### Held-login guard restart

Test: `a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path`. Trial log excerpts:

```text
FAIL [  31.275s] temper::process a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path
process.rs:2256:53: the guard returned the record: Os { code: 2, kind: NotFound, message: "No such file or directory" }
box.err: temper box: .../held-login-guard/config/temper.json: port 45949 (127.0.0.1:45949): cannot listen: AddressInUse
call-0.err: temper ctl: the engine at http://127.0.0.1:41273 did not answer within 30 s
```

The public deployment confirms 45949 was its retained webhooks listener. The owner of the conflicting socket was not established. Reuse by another listener or outgoing socket during the stopped interval is an inference, not a diagnosed cause. This is not the fresh fake sandbox's 8367–8371 preference.

Isolated log:

```text
PASS [  32.558s] temper::process a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path
Summary [  32.559s] 1 test run: 1 passed, 52 skipped
```

It also passed in the final whole one-thread run. The flake was not repeated unchanged again.

### Turn landing and restarts

Test: `a_turn_landing_is_found_after_a_runner_cancellation_and_restarts`. Trial log excerpts:

```text
FAIL [  78.157s] temper::process a_turn_landing_is_found_after_a_runner_cancellation_and_restarts
stalled update: style=rebase
stalled tasks: ... update pull request: uncertain: it may or may not have been made; resource acme/widget #1 ...
support/turn_restart.rs:211:9:
turn restart stalled: native step 34, finished=true, setup=Some([Some(Code(0)), Some(Code(0)), Some(Code(0)), Some(Code(0)), Some(Code(0)), Some(Code(0))]), forge=1 pulls, merged=false, branches=["main", "temper/2/c3"], main edited=false; the restarted turn's progress: patience 60000000000 ns; elapsed 60000223586 ns; thread runnable wait 2189548125 ns; thread CPU 4384766611 ns; tree runnable wait 858214000 ns
```

The update was held as uncertain and the landing did not progress. Its scheduler figures do not meet the starvation threshold; it remains a failure. No port or shared-cache cause was demonstrated. Repair needs a lower-tier reproduction, beyond a quick port-isolation change.

Isolated log:

```text
PASS [  36.178s] temper::process a_turn_landing_is_found_after_a_runner_cancellation_and_restarts
Summary [  36.179s] 1 test run: 1 passed, 52 skipped
```

It also passed in the final whole one-thread run (39.391 s).

## Decision and validation

Under the task's explicit flake fallback, keep the kernel-port fix, leave test-threads at 1, and retain the reason in testing.status.md. Two threads were not measured. Three proving runs at three threads and a committed-head loaded proving run were not attempted after these two flakes. The loaded exploratory trial above is reported in full. The criterion "more than 25% worse on all three runs" is consequently untested; no CPU non-regression claim is made.

The new port test reads all five listener fields from a fresh fake sandbox's written public deployment. Immediately before the red run, binds confirmed all five preferred ports were free. With the old preference it failed in 0.108 s: `/application/clients/listener/socket: a fake box wrote preferred port 127.0.0.1:8367`. After `[0; 5]` it passed in 0.102 s; after rustfmt it passed in 0.095 s. This is the broken rule and observed failure for the new test. An initial `--bin temper` selection ran zero tests (exit 4, bin test=false); it was corrected to `--lib` and is not counted as evidence.

`cargo fmt`, temper's old-form ratchet, and `heavy cargo clippy -p temper -p temper-docs --all-targets -- -D warnings` passed. The help-page rules test passed (1 selected). Focused tests passed 4,473/4,473 across 17 binaries in 18.850 s (28 skipped), for temper, temper-docs, temper-fake, temper-playground, temper-box and the direct dependents temper-service, temper-engine-smith, temper-client-tools, temper-client-view and temper-real-loop-world. No public API signature changed, no cross-tree package became a consumer, and no generated fixture or reference needed regeneration.

## Every selected test

CPU and wall seconds are cpu-summary's figures, including the test's accounted descendants. All rows are the same named test across the three runs. The last column compares the single trial with the final one-thread run; it does not apply the unrun three-replicate rule.

| Test | Base 1 CPU | Base 1 wall | Trial 3 CPU | Trial 3 wall | Final 1 CPU | Final 1 wall | Trial/final CPU change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart` | 40.249675 | 51.877948 | 48.520587 | 47.207807 | 51.682856 | 47.019341 | -6.1% |
| `a_forgejo_elsewhere_connected_as_its_admin_makes_the_first_repository_lands_a_chats_change_and_is_found_after_a_restart` | 22.203100 | 28.602677 | 21.557301 | 22.977197 | 22.467552 | 28.762167 | -4.1% |
| `a_killed_serve_keeps_its_port_and_prints_a_fresh_link` | 0.585682 | 2.014228 | 0.623453 | 2.085288 | 0.616299 | 2.050855 | +1.2% |
| `a_mixed_document_exits_1_naming_its_path` | 0.049241 | 0.069349 | 0.046997 | 0.045006 | 0.033015 | 0.071233 | +42.4% |
| `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` | 18.632883 | 30.201863 | 20.477934 | 29.165070 | 23.585771 | 31.206626 | -13.2% |
| `a_persons_plan_change_lands_on_the_owned_forge_and_is_found_after_a_restart` | 23.463830 | 41.856622 | 27.265330 | 39.371598 | 32.951896 | 37.349472 | -17.3% |
| `a_prepared_forge_runs_forgejo_and_its_runner_once_each` | 3.568571 | 2.351081 | 2.395343 | 2.167820 | 4.411276 | 2.471573 | -45.7% |
| `a_push_is_delivered_signed_with_the_secret_reconcile_drew` | 7.549772 | 7.294334 | 6.588425 | 7.397311 | 7.661042 | 7.311313 | -14.0% |
| `a_restart_over_an_imported_then_exported_account_answers_ctl` | 1.361413 | 1.424285 | 1.418223 | 1.561732 | 1.149732 | 1.613615 | +23.4% |
| `a_restart_with_another_box_sharing_its_cache_answers_ctl` | 13.004637 | 14.330939 | 20.861474 | 16.911967 | 14.797323 | 24.589468 | +41.0% |
| `a_story_box_and_its_steps_die_when_the_test_process_is_killed` | 0.064238 | 0.066744 | 0.117464 | 0.112877 | 0.061914 | 0.068482 | +89.7% |
| `a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path` | 0.955843 | 31.222939 | 1.273516 | 31.273931 | 1.059551 | 31.843449 | +20.2% |
| `a_task_runs_on_each_worker` | 1.162574 | 1.456197 | 1.935209 | 1.407282 | 1.166270 | 2.276220 | +65.9% |
| `a_turn_landing_is_found_after_a_runner_cancellation_and_restarts` | 26.385159 | 38.565439 | 30.594256 | 78.155251 | 23.440474 | 39.390330 | +30.5% |
| `a_worker_alone_with_the_owned_examples_worker_links_and_drains_on_a_signal` | 0.172088 | 0.311833 | 0.297596 | 0.351103 | 0.168439 | 0.544822 | +76.7% |
| `a_worker_hung_past_the_grace_is_fenced_and_links_again` | 12.159888 | 8.723452 | 12.373334 | 8.477438 | 6.224501 | 11.158164 | +98.8% |
| `a_worker_killed_mid_run_is_lost_and_its_run_answered_once_on_another` | 7.026706 | 4.910248 | 6.744353 | 4.820744 | 2.141786 | 7.347788 | +214.9% |
| `a_worker_restarted_on_its_directories_links_at_its_position` | 2.126106 | 1.508302 | 2.081122 | 1.497102 | 1.311628 | 4.178115 | +58.7% |
| `a_worker_started_later_takes_the_waiting_task` | 1.776077 | 1.330639 | 2.009127 | 1.455029 | 1.428098 | 3.724188 | +40.7% |
| `an_adopt_cut_by_a_signal_is_made_once` | 8.154416 | 5.802670 | 7.665739 | 5.389220 | 4.762142 | 6.341821 | +61.0% |
| `an_engine_alone_prints_its_lines_and_drains_on_a_signal` | 0.162470 | 0.259555 | 0.142617 | 0.180049 | 0.122435 | 0.377653 | +16.5% |
| `an_older_recorded_forgejo_is_dumped_then_upgraded` | 2.977325 | 1.979619 | 2.676812 | 1.927704 | 2.206683 | 2.397011 | +21.3% |
| `an_older_recorded_runner_has_its_files_prepared_again_without_a_dump` | 2.370567 | 1.622288 | 2.181397 | 1.512924 | 1.628664 | 1.761079 | +33.9% |
| `an_owned_forgejo_signs_its_member_in_through_temper` | 8.031432 | 5.218077 | 4.916508 | 4.691111 | 4.643523 | 4.669333 | +5.9% |
| `an_unexpected_member_exit_is_reported_after_its_client_settles` | 0.786778 | 0.677445 | 0.428691 | 0.606827 | 0.534360 | 1.014449 | -19.8% |
| `box_kinds::a_box_starts_an_owned_slot_on_start_kind_and_removes_it` | 4.238388 | 4.053631 | 2.682471 | 3.591877 | 2.801897 | 3.779811 | -4.3% |
| `fake_backends::walk::away::a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off` | 5.010863 | 25.274007 | 3.585543 | 26.129557 | 4.012163 | 24.297857 | -10.6% |
| `fake_backends::walk::chat_document::a_chat_document_is_written_and_revised_on_the_fake_box` | 1.903255 | 1.463701 | 1.141668 | 1.623509 | 1.290509 | 1.418773 | -11.5% |
| `fake_backends::walk::copy::a_copy_is_made_and_adopted_through_ctl_on_the_fake_box` | 0.866080 | 0.811476 | 0.656102 | 1.219717 | 0.692314 | 0.843239 | -5.2% |
| `fake_backends::walk::copy::readys_two_copies_are_adopted_in_order_with_the_first_as_home` | 1.155425 | 1.110491 | 0.821434 | 1.518652 | 0.829921 | 1.009639 | -1.0% |
| `fake_backends::walk::goal::a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list` | 4.060563 | 11.820708 | 2.556010 | 12.272134 | 2.386380 | 11.490294 | +7.1% |
| `fake_backends::walk::here_key::forgejo_here_and_a_key_reach_a_first_chat_on_the_fake_box` | 10.859707 | 9.030335 | 7.562861 | 12.640617 | 6.942482 | 8.811162 | +8.9% |
| `fake_backends::walk::note_parts::a_long_document_is_built_in_parts_on_the_fake_box` | 4.503134 | 3.581960 | 2.986623 | 4.928932 | 2.836403 | 2.819189 | +5.3% |
| `fake_backends::walk::sign_in::a_chatgpt_browser_return_on_the_fake_box_leaves_ready_able_to_create` | 9.228541 | 7.236275 | 6.003665 | 9.980180 | 5.733856 | 6.611752 | +4.7% |
| `fake_backends::walk::sign_in::a_chatgpt_sign_in_beside_a_browsers_idle_issuer_sockets_leaves_ready_able_to_create` | 9.035085 | 7.203248 | 7.597320 | 7.671391 | 8.258982 | 7.089203 | -8.0% |
| `fake_backends::walk::sign_in::a_chatgpt_sign_in_on_the_fake_box_leaves_a_usable_model_and_ready_can_create` | 9.472722 | 7.385595 | 7.438209 | 6.730111 | 9.354432 | 7.845634 | -20.5% |
| `fake_backends::walk::status::a_plans_project_has_its_brief_on_a_return_past_the_sitting` | 41.800580 | 53.044667 | 26.223999 | 61.723412 | 29.309456 | 41.834176 | -10.5% |
| `fake_backends::walk::status::home_on_the_fake_box_keeps_a_landing_count_after_box_and_client_restarts` | 8.262764 | 26.803759 | 5.545930 | 27.469729 | 6.203962 | 27.456663 | -10.6% |
| `forge_prepare_makes_an_owned_forge_then_remakes_nothing` | 2.429913 | 1.749718 | 1.753395 | 1.832163 | 2.584863 | 1.716235 | -32.2% |
| `playground_setup_adopts_widget_and_the_worker_reads_its_source_on_the_owned_forge` | 17.972884 | 19.862174 | 12.611247 | 24.918411 | 19.372258 | 20.116654 | -34.9% |
| `startover::owned_setup_cuts_start_over_and_a_removal_cut_finishes_on_restart` | 34.654389 | 23.293940 | 23.093365 | 37.315588 | 34.164024 | 23.234952 | -32.4% |
| `support::forge_land::tests::a_kept_deployment_with_no_merged_pulls_can_find_its_project` | 0.009546 | 0.010329 | 0.004743 | 0.005032 | 0.009268 | 0.009730 | -48.8% |
| `support::forge_land::tests::a_kept_project_matches_the_owned_adoptions_identity` | 0.007292 | 0.007729 | 0.004860 | 0.005330 | 0.009199 | 0.009570 | -47.2% |
| `support::forge_land::tests::a_kept_project_requires_complete_visible_unambiguous_adoptions` | 0.006708 | 0.007244 | 0.004773 | 0.005204 | 0.009240 | 0.009625 | -48.3% |
| `support::owned::agent::tests::an_absolute_program_document_goes_to_smith_byte_for_byte` | 0.008970 | 0.009354 | 0.004515 | 0.004896 | 0.009450 | 0.009838 | -52.2% |
| `support::owned::agent::tests::fetched_names_the_launcher_beside_the_programs_directory` | 0.005709 | 0.006140 | 0.004678 | 0.011735 | 0.009989 | 0.010548 | -53.2% |
| `support::owned::agent::tests::fetched_requires_a_programs_directory_with_a_parent` | 0.009220 | 0.009621 | 0.005158 | 0.005475 | 0.009529 | 0.009926 | -45.9% |
| `temper_box_prints_its_link_and_stops_its_processes` | 4.998863 | 3.921059 | 2.939675 | 8.207437 | 4.768257 | 3.930071 | -38.3% |
| `temper_box_restarts_a_killed_forgejo_while_the_engine_serves` | 6.876069 | 5.295331 | 4.120746 | 11.406748 | 7.671055 | 5.697891 | -46.3% |
| `temper_open_adopts_a_repository_by_commands` | 7.853316 | 6.821911 | 4.951454 | 7.366237 | 8.065643 | 6.761678 | -38.6% |
| `the_cluster_stops_in_reverse_each_exiting_0` | 0.274450 | 0.316975 | 0.186032 | 0.418654 | 0.248883 | 0.312607 | -25.3% |
