# scenario-worlds: figures

Instruction counts are Callgrind Ir, unoptimized test builds, on the same x86-64
builder slot. Each case runs alone through nextest's `suite` binary with an exact
name filter and `NEXTEST_DOUBLE_SPAWN=0`. The moved cases use the fuzzy profile
after the move; their bodies and seed counts are unchanged. Process profiles use
`--trace-children=yes`; fail-stop also sends the profiler's own diagnostics to a
separate log so they cannot enter the report being asserted. Listing processes
are excluded. Fail-stop counts sum the test process and its fixture children.

Costliest and median refer to the pristine native row: 189 tests, run one at a
time with child user plus system CPU accounting. Even rows use the lower median.
Before and after retain those test names, rather than changing the sample when
its rank changes. The remaining rows are the task's specified representatives,
changed tests, or supplementary profiles. `unknown` means no complete instruction
profile, never an estimate or a passing profiler run.

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `skein-conformance-real::suite` | `conformance::ring::sockets::backpressure_stalls_a_sender_and_reading_resumes_it` | 81,011,411 | 61,798,657 | Costliest: deadline snapshots, then pattern generation | Reap first and take one nonblocking backend turn before accounting |
| `skein-conformance-real::suite` | `conformance::ring::sockets::records_wrong_for_the_socket_state` | 7,059,319 | 1,578,979 | Lower median: deadline snapshots on immediate refusals | Completed operations need no new deadline |
| `skein-conformance-real::suite` | `conformance::ring::files::a_file_is_made_written_at_offsets_read_back_and_stated` | 12,698,275 | 2,384,777 | 10,579,511 Ir in wait_begins before; 397,705 after | Reap first; enter deferred work without blocking; keep a deadline if anything remains in flight |
| `skein-conformance-sim::suite` | `conformance::backpressure_stalls_a_sender_and_reading_resumes_it` | 1,341,576,696 | 465,793,151 | Costliest: repeated pattern construction and simulated transfers | 16 calm worlds become 4; 4 smoke worlds stay |
| `skein-conformance-sim::suite` | `conformance::a_close_with_bytes_unread_meets_a_send` | 11,122,646 | 5,328,112 | Median: repeated socket scenarios | 16 calm worlds become 4; smoke worlds stay |
| `skein-conformance-sim::suite` | `conformance::a_calm_world_pairs_each_race_as_the_ring_does` | 43,542,271 | 11,529,157 | Six explicit races and an established-connect cancel per seed | 112 scenario runs become 28; every pairing check stays |
| `skein-conformance-sim::suite` | `conformance::a_file_is_made_written_at_offsets_read_back_and_stated` | 30,712,809 | 13,413,326 | Repeated file scenarios and boundary checks | 16 calm worlds become 4; smoke worlds stay |
| `skein-conformance-sim::suite` | `conformance::a_connection_lives_and_ends_over_ipv4` | unknown | 101,039,244 | Supplementary after-only profile of transfer copies | CALM changes through the shared helper; source-copy gain is below the stop rule |
| `skein-http-world::suite` | `transcripts::every_transcript_decodes_to_its_expectation_whatever_the_cuts` | 1,158,931,907 | 601,450,632 | Costliest: four runs of each golden, bytes read one at a time | Two seeds per fixture; every golden comparison stays |
| `skein-http-world::suite` | `transcripts::every_event_stream_reads_to_its_expectation_whatever_the_cuts_and_scans` | 434,273,572 | 434,273,604 | Median: event-stream decoding and metered delivery | Stays: all four named chunk sizes |
| `skein-http-world::suite` | `requests::every_request_transcript_comes_to_its_expectation_whatever_the_pace` | 304,201,086 | 143,144,308 | Five runs of each request golden | Two seeds per fixture; every head, body and outcome check stays |
| `skein-http-world::suite` | `fuzzy::client::a_transcript_cut_short_anywhere_decodes_as_the_reference_reads_its_prefix` | 583,167,779 | 583,168,024 | Generated cuts compared with the reference | Moves from transcripts to fuzzy::client, with all 188 runs and checks |
| `skein-http-world::suite` | `fuzzy::server::a_request_transcript_cut_short_anywhere_comes_to_what_the_reference_reads_of_its_prefix` | 1,171,294,679 | 1,171,295,310 | Generated prefixes compared with the reference | Moves from requests to fuzzy::server, with all 1,808 cuts and checks |
| `skein-protocol-world::suite` | `worlds::one_end_closing_while_the_other_sends_leaves_both_settled` | 584,651,574 | 929,211,659 | Costliest: generated JSON and stacked decoding | 30 becomes 46: the first prefix with four resets has 23 seeds, doubled for margin |
| `skein-protocol-world::suite` | `worlds::the_wire_resetting_at_any_moment_leaves_both_settled` | 160,908,075 | 30,676,450 | Lower median: repeated reset worlds | 20 becomes 4; the fuzzy sweep injects resets and asserts that one fell |
| `skein-protocol-world::suite` | `worlds::an_llm_s_stream_goes_from_the_server_s_top_to_the_client_s_top` | 225,104,587 | 34,719,646 | Generated streaming worlds | 12 becomes 2; ids > 0 stays |
| `skein-protocol-world::suite` | `worlds::a_connection_carries_two_or_three_calls_through_both_stacks` | 215,449,640 | 125,094,335 | Repeated multi-call worlds | 8 becomes 4; explicitly assert that both call counts appeared |
| `skein-protocol-world::suite` | `worlds::a_seed_replays_to_the_same_run` | 118,309,352 | 86,948,501 | Two complete runs for each replay seed | 4 seeds become 2; both runs still compare in full |
| `skein-protocol-world::suite` | `worlds::a_slow_consumer_at_the_server_s_top_stops_the_upload_at_the_client_s` | 273,156,432 | 273,159,000 | JSON string decoding: 86,420,843 Ir after | Stays: three seeds, each asserting upload_held and complete requests |
| `skein-fail-stop-tests::suite` | `fail_stop::a_panic_on_a_second_thread_aborts_the_process_after_its_report` | 1,043,350,176 | 348,430,893 | Backtrace symbolization in fixture processes; parent plus children summed | Costliest: three borrowed-payload runs become one |
| `skein-fail-stop-tests::suite` | `fail_stop::owned_and_non_string_payloads_follow_the_default_report` | 680,491,706 | 680,804,926 | Backtrace symbolization in fixture processes; parent plus children summed | Lower median: two named-payload runs stay; they carry the other backtrace values |
| `skein-world-tests::suite` | `end_to_end::an_observer_can_be_abandoned_before_its_first_submission` | unknown | unknown | Costliest: Callgrind cannot complete the process operations; native runs pass | Stays: eight fixture starts for two backends, two modes and queued/unsubmitted observation |
| `skein-world-tests::suite` | `end_to_end::abandonment_removes_the_cgroup_after_its_blocked_member_exits` | unknown | unknown | Median: Callgrind cannot complete the process operations; native runs pass | Stays: one fixture tree and its blocked member |
| `skein-world-tests::suite` | `end_to_end::repeated_stderr_overflow_unwinds_without_aborting` | unknown | unknown | Specified cause: Callgrind cannot complete the process operations; native runs pass | 20 fixture starts become 3; exact diagnostic checked on every unwind |
| `skein-world-tests::suite` | `end_to_end::a_binary_that_exits_at_once_settles_with_counts_above_zero` | unknown | unknown | Specified representative: Callgrind cannot complete the process operations; native runs pass | Stays: three containment forms, each with a fixture start |
| `skein-conformance-sim::suite` | `conformance::child_processes_and_pipes` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_connection_lives_and_ends_over_ipv6` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_peer_closes_gracefully` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_send_after_the_peer_closed` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_connect_where_nothing_listens` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::where_an_address_is_in_use` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::ipv6_sockets_are_ipv6_only` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::records_wrong_for_the_socket_state` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_full_accept_queue` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_close_with_bytes_unread_meets_a_recv` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_reset_after_the_end_of_stream` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_client_closed_before_its_connection_is_accepted` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_waiting_accept` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_waiting_recv` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_recv_racing_bytes` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_an_accept_racing_a_connect` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_waiting_connect` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_connect_established_while_its_client_was_away` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::closing_a_listener_resets_the_connections_waiting_on_it` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::an_accept_past_the_descriptor_limit` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_calm_world_reads_and_writes_whole_as_the_ring_does` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::renames_over_across_and_beneath` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::removes_of_files_directories_and_links` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::new_directories_and_one_removed_while_open` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::listings_whole_one_at_a_time_and_cut_short` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_root_beneath_a_root` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::paths_that_leave_their_root_and_paths_that_stay` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::what_the_owner_may_not_do` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::an_open_past_the_descriptor_limit` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_read_of_a_file` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::the_group_is_signalled_after_its_leader_exits` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::usage_counts_the_child_only_after_reaping` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::appends_preserve_the_prefix_and_follow_the_other_descriptors_bytes` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_an_append_stops_it_or_comes_too_late` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::stat_keeps_the_owner_and_counts_hard_links_through_removal` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::path_only_metadata_needs_no_permission_on_its_entry` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::window_on_a_descriptor_that_is_not_a_terminal_answers_other` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_write_open_finds_what_exists_and_truncate_sets_its_length` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_directory_lock_is_busy_through_a_second_open_until_the_first_closes` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_no_replace_rename_refuses_a_taken_target_in_linux_order` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |

Native CPU is supporting evidence, not the instruction measure above. Process
costs vary substantially: even unchanged process cases get faster in this pair.
These totals therefore do not attribute every CPU second saved to this change.

| Row | Tests before / after | Native CPU s before / after |
|---|---:|---:|
| `skein-conformance-real` | 82 / 82 | 1.486921 / 0.668318 |
| `skein-world-tests` | 43 / 43 | 2.099656 / 1.356667 |
| `skein-http-world` | 7 / 5 | 0.547582 / 0.244476 |
| `skein-conformance-sim` | 47 / 47 | 0.707791 / 0.449495 |
| `skein-fail-stop-tests` | 2 / 2 | 0.449436 / 0.309627 |
| `skein-protocol-world` | 8 / 8 | 0.410156 / 0.417854 |

## Found and not fixed

- **Real and simulated conformance: pattern generation.** The real ring
  backpressure case retains 40,370,907 Ir in `scenarios::pattern` (65.33% of its
  after profile); the simulated case retains 324,802,733 Ir (69.73%). A shared
  precomputed fixture or a different pattern-construction algorithm would be
  another change to the shared scenario fixture, with both backends and the
  fuzzy scenarios to hold its byte pattern. The fixed CALM minimum and unchanged
  SMOKE counts leave this work. It is recorded here rather than hidden by another
  seed reduction.
- **Conformance's contract copies stay.** `Run::start` retains bytes that
  `handed_back` compares, including receive tails, list names and entries, and
  spawn pipes. None of those copies is unread. In the supplementary lifecycle
  profile all slice-to-box construction together is only 2,609,792 Ir (2.58% of
  101,039,244 Ir). Recycling `transfer`'s returned send box would remove a subset
  of that; the focused lifecycle gain is below about 0.05 CPU s. It stops there.
- **Real accounting for waits still unanswered stays.** The file case still
  pays 257,799 Ir in `Deadline::watch_process`. The nonblocking turn does not
  guarantee every asynchronous file operation has finished. The conservative
  check arms whenever any kernel still has a flight; a per-token probe would
  require a backend interface change. `PATIENCE`, scheduler accounting and the
  idle-listener path stay. All three deadline tests pass.
- **HTTP keeps its byte and chunk work.** The two-seed response golden retains
  373,016,067 Ir in `World::below_acts`, inclusive of receiving and decoding;
  the event-stream median retains 145,287,917 Ir in `sse::pump`. One-byte body
  reads and the four named event-stream chunks are the checks' input forms and
  stay. Metered allocation contributes 83,855,016 Ir in the response golden
  and 80,838,246 Ir in the event-stream case; `skein_heap::Counting` belongs to
  the other plan. A further kit or decoder optimization would need its own
  cause-specific change rather than another reduction of these forms.
- **Protocol keeps its generators and JSON work.** The 46-world closure case
  pays 366,935,334 Ir in `scenario::partway`, including encoding, and
  253,984,137 Ir in the JSON writer's `put_string`; its `escaped_len` is
  178,199,210 Ir. These inclusive counts overlap. The slow-consumer case pays
  86,420,843 Ir in JSON string decoding. JSON product changes belong to the
  other plan, and the task preserves the scenario generators used by fuzzy.
  The holding tests retain 3, 3 and 6 seeds: each asserts `writer_held`,
  `upload_held` or `upload_stopped`. Further work would optimize shared
  generation/encoding without changing those worlds. The profiles put about
  0.9--1.1 million Ir outside each protocol test body; they do not measure all
  of the request's 17 ms native process floor. Process startup is another plan's.
- **Fail-stop needs three fixture deaths and symbolized reports.** One borrowed
  fixture's backtrace display costs 346,453,769 Ir before (99.88% of that child).
  That path is intentional: each of the three payload/backtrace forms still
  prints its symbols and aborts. Making it cheaper by omitting symbols or
  reusing a process would change what is checked. The fixture count is 5 to 3.
- **Process worlds: complete Ir is unknown.** Callgrind 3.24 cannot run these
  process paths on the pristine base: legacy overflow and blocked-tree startup
  fail with `Child(Other(95))`; a log also reports unsupported syscall 424
  (`pidfd_send_signal`), and the unsubmitted observer cannot settle before its
  unchanged deadline. The failed profiles stop at 2,328,864 Ir (overflow),
  4,788,865 Ir (exit), 4,064,667 Ir (costliest) and 2,211,899 Ir (median), not
  complete-test counts. The native baseline and final runs pass. Complete
  instruction comparisons require a profiler supporting these operations,
  rather than a changed timeout, containment path or test. What stays is eight
  fixture starts in the costliest observer case, one tree in the median,
  three containment starts in the exit case and three overflow starts.
  `supervised_stderr_overflow_keeps_a_bounded_prefix_and_drains_a_natural_tree`
  keeps 262,144 zero bytes: no pipe-capacity constant justifies lowering it.

## Tests that went

- `transcripts::a_transcript_cut_short_anywhere_decodes_as_the_reference_reads_its_prefix`:
  held intact by
  `fuzzy::client::a_transcript_cut_short_anywhere_decodes_as_the_reference_reads_its_prefix`,
  which passed in the checked code at `e5f7735bafee9614d7040416ccca19be15a0d4fa`.
- `requests::a_request_transcript_cut_short_anywhere_comes_to_what_the_reference_reads_of_its_prefix`:
  held intact by
  `fuzzy::server::a_request_transcript_cut_short_anywhere_comes_to_what_the_reference_reads_of_its_prefix`,
  which passed in the checked code at `e5f7735bafee9614d7040416ccca19be15a0d4fa`.

The moves add 188 response worlds and 1,808 request-prefix worlds to fuzzy,
with every reference-reader check. Their after instruction total is
1,754,463,334 Ir; their measured native fuzzy CPU is 0.254003 s together.
The cut is injected directly, so it falls without a probabilistic-fault tally.
Named cut-short behaviors remain in focused as
`the_end_of_the_stream_drops_an_event_cut_short` and
`an_event_cut_before_dispatch_drops_its_document_even_when_json_was_whole`.
The stack case passed in the 371-test focused run; the reader case passed
alone in `skein-http` as `tests::sse::the_end_of_the_stream_drops_an_event_cut_short`,
at the code commit above.

## Counts

- `fail_stop::a_panic_on_a_second_thread_aborts_the_process_after_its_report`
  and `fail_stop::owned_and_non_string_payloads_follow_the_default_report`:
  5 fixture runs become 3. The unnamed borrowed-string run uses `full`; named
  owned-string and opaque-payload runs use unset and `1`. Every run retains all
  of `assert_report`, which is byte-for-byte unchanged. Bite: disable the
  backtrace only for `RUST_BACKTRACE=1`; the opaque row fails because the first
  frame occurs zero times instead of exactly once. Restored.
- `end_to_end::repeated_stderr_overflow_unwinds_without_aborting`: 20 unwinds
  become 3. The second observes what the first left behind; the third checks
  that it does not build up. Every unwind still asserts the exact diagnostic.
  Bite: a temporary counter changes the second call's String panic payload;
  `unwind 1 preserves the scenario's failure` fails with that changed message.
  Restored.
- `CALM`: 16 becomes 4 throughout the focused conformance scenarios listed in
  the table. `SMOKE=4`, `CHAOS=200`, `LATE_RESETS`, `each_seed`, `RACES` and
  racing helpers stay. `Sim::land` draws completion order with `rng.chance(500)`.
  Traces of seeds 0--3 show both orders: at seed 0 the CancelFirst recv reaps
  cancel then target, and the accept reaps target then cancel. The race test
  still runs all three explicit races for recv and accept plus the established
  connect, 112 scenario runs to 28. Bite: classify a successful cancel as
  Completed instead of Stopped in `Cancelling::pairing`; seed 0, CancelFirst
  fails with Completed versus Stopped. Restored.
- `transcripts::every_transcript_decodes_to_its_expectation_whatever_the_cuts`:
  4 seeds become 2, 188 runs become 94; each fixture still meets every golden
  head, outcome and body assertion. A native sample before reduction read
  0.170643 CPU s. Bite: drop delivered body bytes in the client world; seed 0
  fails because an ended body still has reference bytes left. Restored.
- `requests::every_request_transcript_comes_to_its_expectation_whatever_the_pace`:
  5 seeds become 2, 180 runs become 72; each fixture still meets every golden
  call, body and outcome assertion. A native sample before reduction read
  0.072663 CPU s. Bite: drop delivered body bytes in the server world; seed 0
  fails because an ended body still has reference bytes left. Restored.
- `transcripts::every_event_stream_reads_to_its_expectation_whatever_the_cuts_and_scans`:
  4 stays 4, for chunk sizes 1, 7, 64 and 4096. The two moved prefix tests keep
  their 4 seeds per response fixture and their exact stepped request cuts.
- `worlds::an_llm_s_stream_goes_from_the_server_s_top_to_the_client_s_top`:
  12 seeds become 2. Seed 0 already observes 12 IDs/reconnection times;
  doubling the first successful prefix gives two worlds. `ids > 0` and the
  complete event count stay. Bite: omit the referee's ID tally; `ids > 0`
  fails. Restored.
- `worlds::a_connection_carries_two_or_three_calls_through_both_stacks`:
  8 seeds become 4. Seeds 0 and 1 draw three and two calls; double the smallest
  prefix that shows both. A new aggregate asserts both appeared, beside all
  original checks. Bite: force the generator to draw two calls only; the
  aggregate fails. Restored.
- `worlds::one_end_closing_while_the_other_sends_leaves_both_settled`:
  30 becomes 46, an increase required by the task's doubled-margin rule.
  The first four resets occur at seed offsets 4, 14, 17 and 22, so 0..23 is
  the smallest successful prefix and 46 is twice its length. `reset > 3`
  stays. Bite: remove the referee's settlement and hearing goals; the
  aggregate fails with zero resets. Restored.
- `worlds::the_wire_resetting_at_any_moment_leaves_both_settled`:
  20 seeds become 4. The fuzzy protocol sweep draws a reset with
  `rng.chance(100)` and asserts that a reset fell. Bite: prevent the
  settlement goal from accepting settled ends; seed 0 fails overdue with
  the Settled goal unmet. Restored.
- `worlds::a_seed_replays_to_the_same_run`: 4 seeds/two runs each become
  2 seeds/two runs each. Every field of each pair still compares. Bite:
  add a temporary run counter to the returned iteration count; seed 0's
  replay equality fails. Restored.

## Validation

At the checked code commit: formatting, the old-form ratchet, Clippy with
warnings denied for all seven changed packages, direct-dependent test
compilation for real and simulated conformance, 371 focused tests and 50 fuzzy
tests pass. The focused selection reads 5.950345 CPU s; HTTP and simulated
conformance fuzzy read 11.951125 and 3.800018 CPU s. Real conformance has no
fuzzy tests. The package runs include both backends and the three real deadline
checks. Slow tiers and walks do not run in this plan. No budget, timeout,
profile, public interface, shared Deadline mechanism, Wire or heap meter changes.
