# Temper's four CPU layers after the 11-17 repairs

Measured 2026-11-17 on the goal's tree at `7abbe33109ec2ffc8d69f4b9295cbb2c666cc9da`, including all eight repairs below. The forge-route repair was supplied in the starting tree before its pull request merged; the measured code includes it. This is a measurement report, not a budget change or a claim about main's gate.

## Method and limits

The measurement builder was **hel1-8**, an eight-vCPU AMD EPYC-Genoa machine, not the CI builder. Focused and fuzzy selections used **eight lanes**; the real-loop follow-ups also used one and two lanes, by restricting CPU affinity. Both bundle settings were `none`. No slow tier ran, and no code, assertion, seed, cut, patience, reference or budget changed.

The runner was `tools/tiers/one-process`, following `tools/tiers/design.md`, 11 and 11.9. CPU is the sum of each suite process's user and system time, waited-for descendants included. A long binary can be split across processes. Per-test times below are the run's **wall seconds**, not CPU shares; they cannot be summed into a CPU total (`tools/tiers/design.md`, 11.8). Figures are rounded only for presentation.

**Main's ledgers were unavailable:** `TIERS_LEDGER` was unset. No `--ledger` was supplied; no new/flagged-test reference processes were selected, and no current per-test alone CPU is inferred. The ledgers written by these runs contain this selection's measurements, not main's measurements, and were not substituted for main's. All binaries placed in each requested layer were selected, so the complete domain and protocol measurements need no unmeasured-binary contribution from main. Real-loop completeness is discussed separately below.

Run IDs below are directory names under the measuring checkout's `target/`. Each keeps `run.json`, `results.json`, `command-output.txt`, `junit.xml`, `plan.json` and its selection ledger; these are local evidence, not repository fixtures. Layer placement and process CPU were read from `results.json`, and totals verified against the collector's printed summary.

| Run | Profile and selection | Lanes | Observation |
|---|---|---:|---|
| `cpu-1117-focused` | default, all packages below | 8 | 4,741 passed, eight starved, one failed; all domain/protocol tests passed |
| `cpu-1117-fuzzy` | fuzzy, the eight domain packages below | 8 | All 62 passed; none starved or unobserved |
| `cpu-1117-real-loop` | default, `temper-real-loop-world` | 8 | 227 passed, three starved |
| `cpu-1117-real-loop-one-lane` | same package, restricted affinity | 1 | 228 passed, one starved, one failed |
| `cpu-1117-real-loop-two-lanes` | same package, restricted affinity | 2 | 223 passed, six starved, one failed; failed lock story passed the collector's alone diagnostic |

The focused command's package arguments were:

```text
-p temper-client-domain -p temper-client-view -p temper-conformance-world
-p temper-engine-domain-world -p temper-engine-forge-world -p temper-twin-tool
-p temper-worker-checkout-world -p temper-worker-domain-world
-p temper-engine-protocol-forge-world -p temper-box-kit -p temper-box-machine
-p temper-forge-setup-shell -p temper-qa-tool -p temper-real-loop-world
-p temper-redeploy-tool
```

The fuzzy command used the first eight packages. Package selection also runs their selected step tests; those are not counted in the requested layers. In fuzzy, client-domain, client-view and twin-tool had no selected domain tests, and the checkout's real-git binary had none. Every placed binary was accounted for, including these zero selections.

## Layer figures

The before columns are CPU seconds from the CI builder, in this order: run **1206** (11-16 22:40Z), **1237** (11-17 03:20Z), **1248** (05:42Z), **1274** (11:39Z). They are not same-machine baselines for the measurements here. A twentieth under means **CPU <= 0.95 × ceiling**, without rounding that mark down.

The checkout split moved 1.5 focused seconds and 2.8 fuzzy seconds from real-loop to domain. Thus the effective ceilings are 137.5 / 9 / 81.5 / 80.8 seconds, not the pre-split 136 / 9 / 83 / 78. See `tools/tiers/budgets.md`, 4, 7, 8 and 10, and `skein/docs/foundation/testing-strategy.md`, 8.1. With no main ledger, the held budgets here equal the effective ceilings; main's ratcheted held budgets are unknown.

### Focused domain

| Ceiling, CPU s | Twentieth-under mark | Four gates before, CPU s | Now, CPU s | At least a twentieth under? |
|---:|---:|---|---:|---|
| 137.5 | 130.625 | 150.1 / 152.8 / 148.2 / 152.5 | **161.271829** | **No**; 30.646829 s above the mark |

### Focused protocol

| Ceiling, CPU s | Twentieth-under mark | Four gates before, CPU s | Now, CPU s | At least a twentieth under? |
|---:|---:|---|---:|---|
| 9 | 8.55 | 10.0 / 10.1 / 10.1 / 10.5 | **11.391424** | **No**; 2.841424 s above the mark |

### Focused real-loop

| Ceiling, CPU s | Twentieth-under mark | Four gates before, CPU s | Now, CPU s | At least a twentieth under? |
|---:|---:|---|---:|---|
| 81.5 | 77.425 | 83.5 / 83.3 / 82.0 / 90.0 | **66.407326, incomplete** | **Not established**; eight starved and one failed in the full selection |

### Fuzzy domain

| Ceiling, CPU s | Twentieth-under mark | Four gates before, CPU s | Now, CPU s | At least a twentieth under? |
|---:|---:|---|---:|---|
| 80.8 | 76.76 | 89.0 / 91.1 / 88.9 / 92.8 | **100.521904** | **No**; 23.761904 s above the mark |

### What the checks said

These are the collector's warning lines, verbatim, not the delivery check's verdicts:

```text
warning: layer over budget: temper default domain: 161.27 CPU s, budget 137.5 s
warning: layer over budget: temper default protocol: 11.39 CPU s, budget 9 s
warning: layer over budget: temper fuzzy domain: 100.52 CPU s, budget 80.8 s
```

Without `--gate`, these are warnings. The first focused run's aggregate verdict was `starved`, not a pass; its one failed real-loop test made the command exit 100. Fuzzy's aggregate verdict was `no budget set`, exit 0: this is the non-gate verdict, **not** evidence that its layer was under budget (`tools/tiers/budgets.md`, 4).

This report is undeclared-reader Markdown under `docs/development/`. Its prose-only delivery selects outward names, citations and size checks, no focused/fuzzy run (`tools/pre-pr/design.md`, 2, *Prose only*). Consequently it supplies **no numeric delivery CPU verdict** and no main-ledger judgement; the measurements above must not be presented as either. Main's gate must judge again on its own builder and ledgers.

## Binaries and the largest tests still to consider

Every CPU entry is a measured binary sum, excluding reference/empty processes. Every parenthesized test time is **wall**, from that same run, not alone CPU. Names in each row belong to that binary. No main-ledger-selected alone CPU exists here; the collector's lock-failure diagnostic and the implementation reports' alone observations below are separate evidence.

### Focused domain: 161.271829 CPU seconds

| Binary | Tests | CPU s | Largest tests, wall s |
|---|---:|---:|---|
| `temper-engine-domain-world::suite` | 1,248 | 144.807676 | `fabrig_scale::a_goal_tree_takes_384_tasks` (38.185); `forge_routes::b2_tree::every_row_of_b2s_tree_reads_whole_on_standard` (6.780); `forge_routes::status::failed_and_empty_reports_clear_the_run_and_save_nothing` (5.767) |
| `temper-engine-forge-world::suite` | 344 | 8.305353 | `landing_proofs::restarts_in_proving_probing_and_turn_keep_every_identity_once` (3.447); `landing_batches::searching_restarts_after_probes_leaves_cut_open_and_merge_without_repeating_writes` (2.386) |
| `temper-conformance-world::suite` | 30 | 5.621187 | `stories::a_forked_cut_ends_as_a_fresh_cut_does` (3.732); `simulation::a_changed_judge_keeps_its_promises_at_focused_crash_cuts` (3.623) |
| `temper-worker-checkout-world::fakes` | 81 | 1.495047 | `simulation::facts_change_nothing_the_checkout_does` (1.896); `memory::a_domain_with_every_workspace_held_at_its_limits_stays_within_its_heap_ceiling` (0.939) |
| `temper-client-view::suite` | 21 | 0.648671 | `change_page::tests::queue_ordinals_are_rendered_in_the_steps_and_the_line` (0.294); `change_page::tests::bounds_show_zero_below_at_past_and_the_largest_wire_numbers` (0.122) |
| `temper-worker-domain-world::suite` | 63 | 0.218231 | `runtime::a_worker_shutting_down_past_the_grace_abandons_an_unacknowledged_answer_with_its_turns` (0.187); `close::a_close_job_commits_what_its_command_changed_and_pushes_it_with_one_parent` (0.162) |
| `temper-twin-tool::suite` | 33 | 0.147326 | `frames::a_bad_checksum_is_refused_at_the_frame_offset` (0.046); `frames::round_trip::text_and_missing_tokens_stay_normalized_through_a_round_trip` (0.028) |
| `temper-client-domain::suite` | 14 | 0.028338 | `change::tests::a_cut_ended_page_is_kept_without_following_its_cursor` (0.107); `change::tests::an_old_ended_read_answer_cannot_attach_to_the_next_task` (0.040) |

The engine-domain binary is 89.8% of this layer. Its scale story still drives all 384 tasks through delegation, assignment and finish, then reads the oracle, restarts over retained rows and compares the readback. `fabrig_scale.rs::assignment` still searches accumulated assignments backwards on every wait; `close` still performs its full restart/readback. Those are remaining mechanisms, not a measured attribution of the 38.185 wall seconds. The B2 story builds its tree and checks each task's document, stored point, history and ended row. The status story builds four independent worlds for its four failure/empty-report cases. No check was removed to improve these figures.

The forge-route report says its idle fence remains conservative for externally blocked work, legacy modifier paths can still rebuild, and `with_box_setup` still has a specialized bootstrap. The direct-driver report did not claim a scale-story improvement. A further phase profile, separating active root/store work, setup and oracle/restart, is still needed; aggregate binary CPU does not establish which dominates.

For forge-world, the history/referee scans were small in its diagnostic sample; its report identified 58,666 client-event steps, taking 1.220 diagnostic wall seconds, as a larger remaining cost. Conformance still executes each cut's recovery and suffix with every observation/readback. Neither report isolated the remaining CPU by phase. The other five binaries total only 2.537613 CPU seconds; they have no newly identified inefficiency here. Their largest walls above are candidates for reference checking when main's ledger is available, not proven per-test CPU breaches.

### Focused protocol: 11.391424 CPU seconds

Only `temper-engine-protocol-forge-world::suite`: 36 tests, **11.391424 CPU s**. Largest walls were:

| Test | Wall s |
|---|---:|
| `github_land::pending_ci_across_token_hour_completes_on_fresh_token` | 13.536 |
| `rate::iterating_changes_installation_requests_per_hour` | 10.220 |
| `rate::moving_main_installation_requests_per_hour` | 8.336 |

Clock jumps and larger random fragments did not establish the 8.55-second aim on this builder. The protocol report's instrumentation identified fragmented joint/protocol work rather than idle seconds alone: the pending-pull story made 54,560 joint ticks and 2,381 requests; moving-main at N=3 after clock jumps made 35,635 ticks and 1,803 requests. The workloads and token-hour span remain. The pending-CI story still schedules 3,720 simulated seconds, checks credential renewal and the final landing. Its final residual CPU was not separately attributed. A profile of the remaining request/fragment processing is still needed; these wall times are not four alone-CPU results against the one-second reference.

### Focused real-loop

Complete observation was not obtained. The first full selection measured **66.407326 CPU s**, but eight tests starved and stubborn cancellation failed. It is not a complete-layer under-budget result. The package-only eight-lane run measured 63.330682 CPU s in real-loop, with three starved tests. The one-lane run measured 68.038562 CPU s in real-loop, with the red turn starved and the green turn failed. The two-lane package run measured 59.929994 real-loop CPU s, with four real-loop stories and two step tests starved, plus the lock story failed. The collector reran that lock story alone: it passed, using 1.923059 process CPU s (not an empty-process-subtracted reference). Its shared-run failure reported undeclared running/starting task 2 and live agent child 14. These package-only figures exclude the other six real-loop binaries; they must not be compared as whole-layer totals.

Both assertion failures were read and reproduced by exact name through `cargo nextest run -p temper-real-loop-world --test suite -E 'test(=NAME)'`. Stubborn cancellation passed alone in **13.467 wall s**; the green turn passed alone in **13.855 wall s**. The former failure was overdue waiting for `task show 1 --until ended`; the latter was overdue waiting for `change show 3 --until landed --json`. The one-lane green failure recorded 8.543 s runnable wait and 10.433 s thread CPU within its 19-second judge patience. The starved red turn recorded 12.078 s runnable wait and 7.857 s CPU. These observations explain why partial runs cannot certify the aim; they do not justify changing patience.

The original full selection accounts for every placed real-loop binary as follows. The two real-loop-world entries are partial observations, not certified whole-binary costs.

| Binary | Tests selected | CPU s observed | Largest test walls, s |
|---|---:|---:|---|
| `temper-real-loop-world::suite` | 23 | 50.806274, partial | `terminal::a_person_at_the_terminal_reads_a_chats_committed_reply` (22.265, starved); `sign_in::a_terminal_sign_in_returns_to_the_real_listener` (21.902, starved) |
| `temper-worker-checkout-world::suite` | 1 | 7.073886 | `real_delivery::a_design_delivery_reaches_its_landing_branch_through_real_git` (12.741) |
| `temper-real-loop-world::live_text` | 1 | 3.899058, partial | `a_person_reads_a_chats_reply_as_it_is_written_then_its_committed_turn` (22.656, starved) |
| `temper-qa-tool::suite` | 174 | 2.572247 | `run::failed_filing_retries_each_tick_and_reports_one_line_after_an_hour` (2.401); `run::goal_state_failures_retry_but_unconfigured_or_landed_goals_are_not_polled` (1.262) |
| `temper-forge-setup-shell::suite` | 26 | 1.907510 | `boundary::process_and_file_locks_stay_in_their_declared_modules` (6.038); `fetch::local_tls_bottle_fetch` (0.310) |
| `temper-redeploy-tool::suite` | 42 | 0.070015 | `redeploy::no_unknown_and_absent_last_verdict_restore_then_drain_all_without_rechecking` (0.094); `composition::a_staging_failure_records_refused_without_begun_redeploy_or_referee` (0.032) |
| `temper-box-kit::parent` | 2 | 0.043420 | `an_unreadable_executable_waits_for_exit_before_it_is_judged` (0.047); `a_reaped_child_has_no_executable` (0.044) |
| `temper-box-machine::real_loop` | 3 | 0.034916 | `a_kind_removal_unlinks_its_group_file_and_started_record_on_the_real_ring` (0.155); `a_real_child_reads_the_secret_and_eof_on_stdin` (0.079) |

The real-loop report left substantial service/client work over thousands of world passes in green/red/GitHub stories, without final phase attribution. Stubborn cancellation retains a mostly idle real-clock grace. The other binaries have no newly identified residual cause here. A complete non-starved real-loop-world observation remains necessary; adding alone runs to partial shared processes would double-count work and would not repair that missing measurement.

### Fuzzy domain: 100.521904 CPU seconds

| Binary | Tests | CPU s | Largest tests, wall s |
|---|---:|---:|---|
| `temper-engine-domain-world::suite` | 33 | 45.044258 | `fuzzy::follows::fuzzy_follows_shard_1` (32.900); `_0` (27.692); `_3` (23.853); `_2` (17.538), all with the same prefix |
| `temper-conformance-world::suite` | 15 | 43.811881 | `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` (32.650); `fuzzy::shared::a_changed_judge_is_crashed_at_every_live_commit` (26.186); `fuzzy::stories::a_lost_merge_is_crashed_at_every_live_commit` (20.264) |
| `temper-engine-forge-world::suite` | 10 | 8.557658 | `fuzzy::landing_batches::seeded_batches_keep_the_fast_forward_prefix_and_cleanup_invariants` (12.063); `fuzzy::gate_keys::non_base_gate_keys_ignore_generated_bases_and_ignored_paths` (2.956) |
| `temper-worker-checkout-world::fakes` | 2 | 2.827622 | `fuzzy::simulation::random_worlds_settle_with_every_client_heard` (10.286); `fuzzy::merges::seeded_merge_worlds_preserve_complete_trees_and_parent_graphs` (0.354) |
| `temper-worker-domain-world::suite` | 2 | 0.280485 | `fuzzy::whole::seeded_worker_delivery_and_contact_stories_replay` (0.622); `fuzzy::runtime::runtime_random_turn_sizes_and_tiny_windows_settle` (0.486) |
| `temper-client-domain::suite` | 0 | 0 | No selected fuzzy tests |
| `temper-client-view::suite` | 0 | 0 | No selected fuzzy tests |
| `temper-twin-tool::suite` | 0 | 0 | No selected fuzzy tests |

Engine-domain and conformance together are **88.4%** of the layer. The follows report explicitly left fixture setup, scripted/referee work, snapshots and close unseparated; it did not establish its four-shard under-14-CPU-second aim. The escalation matrix still took **9.972 wall s** here; remaining root/store work versus hashing was not attributed. Forge-route's measured fuzzy CPU scarcely changed in its matched comparison. The conformance change retained every live-commit fork and the continuation after recovery; its report did not establish the under-30-CPU-second aim, and this binary is 43.811881 CPU s here. Those are the first two residual workloads to profile, not permission to remove seeds or cuts.

Forge-world still has the client-event work noted above. The checkout fake-world CPU is close to the 2.8-second move but is not held to that move in isolation: the layer's total is the budgeted quantity. Its largest wall exceeds the fuzzy ten-second reference, but only a ledger-selected alone run can establish a CPU breach. No individual residual cause was identified for checkout or worker-domain.

## The goal's changes and what they reported saving

Sources: the eight pull requests into `temper66709/g6709` listed below, plus their implementation reports. The pull-request bodies supply the CI baselines and intended causes; most implementation reports explicitly deferred aggregate CPU-after measurement to the landing report. **Unreported CPU savings are unknown, not zero and not inferred by subtracting wall from CPU.** Before/after samples from different builders are not controlled savings measurements, and the rows below must not be added together.

| Change and pull request | Cause fixed | Reported CPU evidence / saving |
|---|---|---|
| 6729, **Jump protocol story clocks and use larger seeded random fragments**, #2676 | Timer/output-aware clock jumps instead of idle one-second stepping; the four hour/rate stories use seeded random pieces up to 4,096 bytes instead of 127. | No aggregate after CPU or isolated CPU saving reported. CI before 10.0–10.5 s. Staged alone wall reductions were noisy; fragment/joint work remained dominant. |
| 6730, **Cheaper direct-domain draining, immutable setup and store forks**, #2707 | Stop after two quiet passes, counting delivery progress; avoid a discarded default store; cache immutable parsing/settings/profile; copy durable backing rather than replay transaction history when forking. | No aggregate CPU saving reported. Before binary 139 s focused / 43 s fuzzy. Final scale-test alone walls were 8.156 / 1.210 s versus 6.093 / 0.839 s before; those do not establish shared-process cache savings. Per-call setup diagnostics were wall measurements, not isolated CPU savings. |
| 6731, **Skip idle forge ticks and construct route fixtures once**, #2751 | Revision/deadline-based idle fence and horizon jumps; one configuration construction; no discarded default store; incremental copied-landing boundary checks; cached defaults. | Matched focused binary **140.218939 → 113.185059 CPU s**, saving **27.033880 s**. Fuzzy **41.707992 → 41.680703**, a 0.027289 s difference, effectively unchanged. Reader-coincident focused samples ranged up to 144.900611 s, so 27.03 s is not a guaranteed combined-tree saving. Individual causes were not separately CPU-measured. |
| 6732, **Trim follows authentication and settling, and escalation replay work**, #2653 | Reuse valid owner sessions; replace unconditional settle padding with conditions/timers; compare appended replay evidence and check changed store keys with invalidation. | No aggregate CPU saving reported. Four after alone shard walls totaled 25.015 s; the report explicitly did not claim the under-14-CPU-second aim. Before binary fuzzy CPU approximately 43 s. |
| 6737, **Avoid repeated conformance crash-cut setup and observation work**, #2658 | Lazy diagnostic formatting; incremental worker expenses; range reads for inbox/connectors; reads-only oracle snapshot; immutable per-process configuration cache. | No aggregate CPU saving reported. CI before 4.9 s focused / 41 s fuzzy; after nextest walls 1.694 / 14.211 s are not CPU. Every cut's recovery and suffix remained. |
| 6738, **Index forge-world change histories and bound referee lookups**, #2654 | Index history coordinates by change; bounded event/identity buckets and closed intervals instead of repeated scans. | No aggregate CPU saving reported. CI before 7.6 s focused / 8 s fuzzy. Sampled history reads (0.00816 s) and referee work (0.00217 s) were small wall costs; the one-third CPU reduction was unproven. |
| 6739, **Make real-loop stories cheaper without weakening their checks**, #2736 | Version/progress-aware Until retries; derive startup operation credit without discarded preparation; incremental LLM counts and revision-aware snapshots; size fake GitHub encodings before writing. | No binary/layer after CPU reported. Alone CPU after: green turn **8.392215**, stubborn cancellation **0.861028**, red turn **7.769362**, connected GitHub **5.469634** s, versus supplied CI before 13.5 / 13.3 / 12.9 / 8.6 s. These cross-builder figures are not isolated aggregate savings. Cancellation still took 12.954 wall s, mostly its real-clock grace. |
| 6752, **Split checkout fake-disk stories from real-git delivery**, #2649 | Place fake-disk domain stories in `fakes`; leave the real-git/owned-Forgejo story in `suite`. This is placement, not an optimization. | Local `fakes` CPU **1.489128** focused / **2.708964** fuzzy; real-git `suite` **9.714641** focused. Moves rounded up to **1.5 / 2.8 s**. No CPU saving claimed; all 84 tests retained. |

## What remains for the user

The complete domain and protocol observations do **not** demonstrate the twentieth-under aim on this builder. They are not evidence that a repair saved nothing at the CI gate: there is no controlled before sample here, no main ledger, and the measured builder is different. The real-loop result still needs complete observation. No budget or production/test code was changed to turn a warning green. The user decides the next work from these measurements and the gate's subsequent figures.
