# jig focused suite: where the time goes, and what gets the median to 12 s

Date: 2026-10-18. jig `origin/main` 08430b94, detached scratch worktree (removed afterwards). Measured on debian-16gb-hel1-1 (8 vCPU AMD EPYC Genoa), NOT on the gate machine: carbon-forge is an i5-8265U, 4 cores / 8 threads (jig test-budgets findings/measurements.md), and the Gate-Result times below come from it. Every cargo run went in the gate lane unless said.

## 1. How gate time is measured

The merge evidence uses nextest's own `Summary [ N s]` of each `cargo nextest run` in jig's `scripts/check.sh`: focused, fuzzy, browser, then the whole gate. So the focused figure is nextest wall time only (no compile), default profile, `NEXTEST_DOUBLE_SPAWN=0`, test threads = logical CPUs. Budget: `.config/nextest.toml` `[profile.default] global-timeout` (30 s since jig 20d9501e / ruling 275; 15 s before). The `measure` profile has no budget: `cargo nextest run --workspace --profile measure -j 1` (jig docs/development/workflow.md; client-api README, "Test time"). The 12 s aim is the median of three gate-lane runs.

```
$ git -C jig log origin/main --format='%h %(trailers:key=Gate-Result,valueonly,separator=)' | grep passed | head -20 | sed -E 's/ scripts.check.sh passed: ([0-9]+) tests \/ ([0-9.]+) s;.*/ \1 \2/' | paste - -
08430b94 4266 14.289	d6afc638 4264 7.920
48a6fe9a 4264 15.108	8bf9d290 4263 13.931
534f14e3 4262 14.406	80b25e95 4261 15.470
59a8efec 4260 13.590	fa8a3735 4252 13.658
338fcb40 4252 14.595	4e1ab92b 4250 15.770
c023da40 4248 15.645	b9defaf2 4236 14.407
38ae1ba1 4225 18.703	7d156bd8 4220 14.788
21f9d4a8 4212 15.183	899bd8e6 4212 18.585
8ce834ff 4209 15.456	7d3becac 4194 17.607
e0c7b18b 4188 14.819	1c041ac7 4185 15.084
```

Last 20 code landings (hash, tests, focused s): median 14.95 s, min 7.92, max 18.70; last 10: median 14.35 s. The same code swings 13.6 to 18.7 s (load from other sessions; one 7.92 s outlier). At the client-api base the median was 10.33 s for 3568 tests (client-api findings/measurements.md): +698 tests cost +4.1 s on carbon-forge, about 6 ms of wall each against a 3.4 ms suite average, so the new tests (client view and world) are heavier than the rest.

## 2. The focused suite as the gate runs it, and measured serially

```
$ cargo nextest run --workspace      (x3, scratch worktree)
     Summary [   6.663s] 4266 tests run: 4266 passed (1 slow), 0 skipped     (cold cache)
     Summary [   5.305s] 4266 tests run: 4266 passed, 0 skipped
     Summary [   5.941s] 4266 tests run: 4266 passed, 0 skipped
$ cargo nextest run --workspace --profile measure -j 1
     Summary [  52.628s] 4266 tests run: 4266 passed, 0 skipped      (log tmp/logs/20261018T220649-3795.log)
```

Median 5.94 s here against about 14.4 s on carbon-forge (2.4 times). Summed per-test time at `-j 1` is 52.1 s, mean 12.2 ms, median 6 ms. Starting 4266 tests across 248 binaries (43 skipped by the default filter): 61 unit-test binaries (lib/bin) hold 3034 tests, 27.9 s summed; 164 integration binaries hold 1232 tests, 24.3 s.

Distribution of one test's time (`-j 1`): up to 3 ms 820 tests (2.1 s); up to 5 ms 1241 (5.5 s); up to 10 ms 1003 (7.3 s); up to 20 ms 580 (9.0 s); up to 50 ms 487 (14.3 s); up to 100 ms 80 (5.7 s); up to 200 ms 47 (6.3 s); over 200 ms 8 (1.9 s). The tail is short: no test takes over 0.27 s, so the suite is throughput-bound, not tail-bound. With 8 threads, wall is 5 to 6 s for 52 s of work.

25 slowest tests (`-j 1`, s, binary, test):
```
0.269 jig-client-protocol-world::protocol signed_out_with_requests_pending_then_signed_in_again
0.257 jig-channel-world::channel a_resumed_run_is_sent_only_its_current_window
0.250 jig-client-protocol-world::protocol words_to_a_stopped_task_release_it_in_one_compound_request
0.247 jig-ops-world::client_stories p_accepts_a_restart_from_the_incidents_page_with_a_second_seed
0.247 jig-client-protocol-world::protocol actual_busy_not_ready_restart_and_dropped_watch_retry_the_same_request
0.220 jig-one-process-world::one_process a_person_s_chat_is_answered_by_the_in_process_worker
0.219 jig-client-protocol-world::protocol a_request_whose_answer_was_lost_is_made_once
0.208 jig-one-process-world::one_process a_power_loss_mid_run_restarts_and_adopts_nothing_from_the_dead_worker
0.192 jig-client-protocol-world::protocol conversations::a_person_s_chats_list_live_then_ended_by_last_activity
0.187 jig-client-protocol-world::protocol a_reload_restores_a_pending_request_from_the_small_store_s_bytes
0.180 jig-conformance-world::scenarios every_shared_adversarial_story_runs_on_the_testing_application
0.175 jig-ops-world::client_stories p_accepts_a_restart_from_the_incidents_page
0.171 jig-ops-world joint::bytes::tests::a_linked_ops_root_starts_on_the_common_world_clock
0.170 jig-one-process-world::one_process the_next_start_settles_the_worker_s_claims_at_once
0.164 jig-one-process-world::one_process a_kill_before_a_commit_loses_nothing_a_person_saw
0.164 jig-client-protocol-world::protocol a_person_s_words_reach_a_running_run_through_bytes
0.163 jig-conformance-world::snapshot standing_task_snapshots_match_complete_fresh_state
0.159 jig-ops-world::client_stories q_asks_for_an_environment_in_a_chat_and_accepts_what_it_costs
0.157 jig-client-protocol-world::protocol a_slow_reader_gets_fewer_deliveries_and_others_are_answered
0.156 jig-client-protocol-world::protocol goals::moving_a_goal_up_reorders_the_board_in_one_compound_request
0.150 jig-channel-world::channel a_message_refused_by_the_run_is_committed_with_its_next_turn
0.144 jig-ops-world::client_stories scripted_client_starts_within_the_shell_heap_envelope
0.138 jig-conformance-world::snapshot shrinking_pool_snapshots_match_complete_fresh_state
0.138 jig-client-protocol-world::protocol an_engine_outside_the_client_s_versions_shows_out_of_date
0.138 jig-client-protocol-world::client closing_a_chat_by_shapes_moves_its_delegate_then_cancels_it_after_its_grace
```

Top 15 crates (tests, summed s, ms each); 80 crates in all:
```
jig-client-view 568 9.10 16.0 | jig-client-world 359 7.83 21.8 | jig-core-world 359 7.38 20.6
jig-client-protocol-world 58 3.68 63.4 | jig-channel-world 31 2.39 77.2 | jig-conformance-world 51 2.03 39.9
jig-ops-world 61 1.71 28.1 | jig-domain-core 253 1.62 6.4 | jig-protocol-store 296 1.57 5.3
jig-test-service 101 1.39 13.8 | jig-boundary 74 1.29 17.4 | jig-tasks-world 155 1.06 6.8
jig-client-test-view 42 0.93 22.2 | jig-one-process-world 5 0.90 179.0 | jig-client-scripted-engine 138 0.63 4.6
```
The top three are 24.3 s of 52.1 (47%) and 1286 tests. Inside jig-core-world nearly every test binary has a floor of 12 to 20 ms per test (even binaries of 1 to 3 tests; `referee` is 2 to 3 ms), so the cost is the world each test builds, not the process.

## 3. Measurements behind the levers

Process start (direct runs of test binaries, 100 each, `NEXTEST=1`, bash fork included; `/bin/true` is 2.0 ms):
```
$ ./referee-* --exact nonexistent_test            3.0 ms   (49 MB binary)
$ ./jig_domain_core-* --exact nonexistent_test    4.0 ms   (37 MB)
$ ./jig_client_view-* --exact nonexistent_test    4.6 ms   (41 MB)
$ ./stories-* (jig-core-world) --exact nonexistent_test   5.1 ms (81 MB);  with one real test 17.8 ms
$ LD_DEBUG=statistics ./stories-* ...   dynamic loader 626k cycles (0.2 ms), 70% of it relocation
```
So the floor of a test is about 2 to 3 ms of mostly kernel exec/page-fault time (sys 0.3 s of 0.4 s per 100 runs); the loader and relocations are negligible. 2061 tests take 5 ms or less (7.6 s summed): they are nearly all floor.

Unit tests in one process (libtest) instead of one process each, all 61 unit binaries run directly, none failing (jig-boundary needs its package directory as cwd):
```
$ taskset -c 0-7.../libs.sh 8     direct libtest 8 threads: 2.59 s    vs nextest kind(lib or bin): 2.82 s
$ taskset -c 0-3.../libs.sh 8     direct libtest 8 threads: 3.28, 3.48 s   vs nextest: 4.84, 4.85 s
$.../libs.sh 1 (one thread)       12.9 s    vs nextest -j 1 summed 27.9 s
```
Opt-level of dependencies (config only, no file touched: `--config 'profile.dev.package."*".opt-level=2'`, own `--target-dir`, build 4m39s against 2m54s). jig's Cargo.toml has `opt-level = 3` only for `jig-boundary` and `skein-codegen`. A/B interleaved, three runs each, `taskset` to model fewer cores (`-j 8`):
```
$ taskset -c 0-3 cargo nextest run --workspace -j 8 [--config...]
4 vCPU: base 10.51 8.66 8.84 (median 8.84)    deps opt 2: 8.05 7.35 7.46 (median 7.46)   -15.6%
8 vCPU: base  4.82 4.86 4.66 (median 4.82)    deps opt 2: 3.94 4.10 4.07 (median 4.07)   -15.6%
$ ... --profile measure -j 1 (deps opt 2)   Summary [ 45.104s]   (52.628 s base: -14.4%)
```
Per crate at `-j 1`, base to deps opt 2: jig-channel-world 2.39 to 1.53 s (-36%), jig-ops-world -28%, jig-client-protocol-world -26%, jig-client-world -20%, jig-client-view 9.10 to 7.41 s (-19%), jig-core-world 7.38 to 7.02 s (-5%), jig-domain-core and jig-protocol-store about 0.
4 vCPU with `-j 8` takes 8.8 s here against 14.4 s on carbon-forge (1.6 times); the suite is CPU-bound, so I scale savings by the same share.

Thread count: the best is `-j` equal to the CPUs, nothing to gain:
```
4 vCPU: -j4 8.95/9.00  -j6 9.24/9.17  -j8 8.60/8.80  -j12 8.78/9.95
8 vCPU: -j6 6.27/6.28  -j8 5.06/5.12  -j12 5.29/5.19  -j16 5.37/5.06
```
Phrasebook `english()`: no view test builds it through the shared fixture (`Fixture::build` in jig-client-view's tests/fixtures/client.rs does not). Counting by source: about 42 `#[test]` functions call a builder directly (jig-client-view 19, jig-client-test-view 16, jig-client 3, others 4), and roughly 100 more reach it through helpers (jig-client-view tests `owner_chip.rs`: 47 call sites; client world `harness/referee.rs` `Referee::new`: 24; the ops and shell-web fixtures). Those modules' median is 17 to 25 ms, against 8 ms for jig-client-view `tests::domain`; the cheapest one in each is 6 to 27 ms. A `OnceLock` (testing/jig-client-test-view/src/tests.rs) does nothing under process-per-test.

## Levers

Savings are wall time on carbon-forge (14.4 s median now), scaled from the 4 vCPU model.
1. **Dependencies at opt-level 2** (`[profile.dev.package."*"] opt-level = 2` in jig/Cargo.toml). About -2.3 s (-16%). Evidence: A/B above, 3 runs each at two core counts, identical -15.6%. Costs: a cold first build 1.5 to 2 minutes longer, the wasm32 dev builds of the browser tier get the same setting (not measured). Size S.
2. **Unit tests in one process** (the fallback of test-budgets ruling 5: `<repo>-tests.one-process`; the designer changes skein testing-strategy.md, section 8, "A test is a process"; check.sh runs `cargo test --workspace --lib --bins` beside `cargo nextest run -E 'kind(test)'`). About -2.0 to -2.3 s (-14 to -16%; only -0.2 s at 8 vCPU). Evidence: 61 unit binaries pass in-process; 3034 tests, 15 s of 27.9 s serial CPU is process cost. Loses nextest's SLOW flag and global-timeout for those tests. Size M.
3. **Cut the world fixtures' floor** in jig-core-world (15 ms per test x 359), jig-client-world (`harness/world.rs`, `harness/referee.rs`) and jig-client-view `Fixture::build` (about 1300 tests): halve the floor and save -1 to -2 s. Evidence: floors above; deps opt 2 moves jig-core-world only 5%, so the cost is in jig's own crates or the test body; where exactly is unmeasured (no perf on this machine). Size M to L.
4. **`opt-level = 1` for jig's own crates** (`[profile.dev] opt-level = 1`, config only): expected from the 5% in jig-core-world, unknown, maybe -1 s; costs compile time on every increment. Size S to try, not tried.
5. **Merge tiny tests into table tests**: the 2061 tests at 5 ms or less hold about 4.5 s of floor (-1 s wall at most per 1000 merged); lever 2 takes the same cost with no edits. Size L, skip.
6. **Phrasebook `english()`, thread count, loader relocations**: under 0.5 s each or nothing (see above). Skip.

Together 1 and 2 give 14.4 s to about 10.5 to 11.5 s (-25 to -30% if they add; not measured together). Twelve seconds as the median of three looks reachable in 3 increments (lever 1; lever 2 with the designer's one-line change; lever 3 only if the gate machine's runs still sit above 11.5 s), plus one to restore `global-timeout = "15s"` and the docs.

## Not verified

- Nothing ran on carbon-forge: the 1.6-times scaling from 4 vCPU is a model (its linearity rests on 8-to-4 vCPU runs scaling 1.8 times); its `~/.cargo/config.toml` (`debug`, linker) may differ from this machine's (`line-tables-only`, mold); first check, three gate-lane runs there.
- Levers 1 and 2 were never measured together, nor lever 1 on the wasm32 builds or on compile time of an incremental change.
- Where jig-core-world's and the client fixtures' floor goes (the world build, allocations, or the test bodies): no profiler here. The cost of `english()` is bounded from the module floors, not measured on its own.
- Gate-Result noise (13.6 to 18.7 s for the same code) is not explained; the 7.92 s landing d6afc638 may have run on another machine.
- One `cargo nextest list --workspace --message-format json` ran (nothing built) to find the test binaries; the 100-run loops ran the test binaries directly, not cargo.
