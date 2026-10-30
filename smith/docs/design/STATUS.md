# Status

Updated 2026-10-29. How far smith's code is from its design.

Each design document's status is in `<doc>.status.md` beside it:
- [`domain/README.md`](domain/README.status.md)
- [`domain/run.md`](domain/run.status.md)
- [`domain/session.md`](domain/session.status.md)
- [`domain/tools.md`](domain/tools.status.md)
- [`domain/host.md`](domain/host.status.md)
- [`protocol/README.md`](protocol/README.status.md)
- [`protocol/channel.md`](protocol/channel.status.md)
- [`protocol/charter.md`](protocol/charter.status.md)
- [`protocol/transcript.md`](protocol/transcript.status.md)
- [`protocol/llm.md`](protocol/llm.status.md)
- [`protocol/agent.md`](protocol/agent.status.md)
- [`protocol/events.md`](protocol/events.status.md)
- [`protocol/limits.md`](protocol/limits.status.md)
- [`protocol/hosts.md`](protocol/hosts.status.md)
- [`protocol/programs.md`](protocol/programs.status.md)
- [`shell.md`](shell.status.md)
- [`testing.md`](testing.status.md)
- [`benchmarks.md`](benchmarks.status.md)

Not single documents:

| Area | Built | Not yet |
|---|---|---|

## Dynamic memory contracts

- `domain/README.md`, 4: built: the named hand-off predicates, count-based
  child-fact drains through `pending_facts()`, uncapped domain stages and
  queues, and the turn-hand-off slab growing on demand until reclaim.
- `protocol/limits.md`, 3.8, and `shell.md`, 2.2: built: the terminal's
  uncapped hand-off, private six-slot `ACCOUNT_EDGE_ROOM`, and shipped
  `DEFAULT_PACING = 2_302`, independent of a domain's output bound.
- `domain/run.md`, 8.2, `domain/host.md`, 3, and `protocol/events.md`, 5.2:
  built: delivery `owned_bound()` beside `owned_bytes()`, the front
  projection's `View::owned_bound`, and `codec_bound` for event records.
- `protocol/llm.md`, 8, `domain/README.md`, 4, and `protocol/agent.md`, 4:
  built: `completion_bound`, `render_bound`, `job_problem_bound`,
  `size_refusal_bound`, `feedback_bound` and `ConfiguredModel::owned_bound`.
- `domain/tools.md`, 6: built: `result_bound` and `batch_result_bound`
  describe result receiving ownership.
- `testing.md`, 6, and `protocol/limits.md`, 5 and 11: built: remeasured
  total-peak ceilings in run, session, host, hosted-process, machine,
  protocol-machine, local-process store and agent fuzzy scenarios. No
  world exercises a domain's output flow control. The hosted-process
  ErrorTail case shares the scenario's measured total ceiling; it has no
  separate 4,096-byte allowance.
- `domain/session.md`, 10: built: one exact type-size contract check,
  `TURN_KIND_SIZE` = 192 bytes on x86_64 for `size_of(Turn) +
  size_of(Kind)`, an `assert_eq` in
  `smith/crates/smith-domain-session/src/tests/builtin.rs`
  (`the_turn_and_kind_cells_keep_their_size`), not an allocator ceiling.
- `protocol/limits.md`, 3.6 and 3.8: built: `EdgeRoom { to_domain: 1,
  below: 6 }`, `EDGE_ROOM` and `edge_room(&Limits)`, one host-channel
  entrance's credit, with no `max_out` alias. Past the credit an entrance
  waits; `send_live` and `offer_fact` drop and count the piece's bytes or
  fact instead.
- `domain/host.md`, 2, and `protocol/channel.md`, 4 and 8: built: two
  edges filled by the agent process, `LIVE_SLOTS = 4` and received-turn
  measurements at the host's turn credit; sent-message timestamps uncapped.
- `domain/README.md`, 4, `domain/run.md`, 11, `domain/session.md`, 9,
  and `domain/tools.md`, 6: built: no per-entrance output or fact
  allowances; uncapped request queues and lossless fact queues. Run,
  session and tools expose `pending_facts()` for count-based drains.
- `domain/host.md`, 9.1: built: the root and inline slot expose
  `pending_content()`; the local domain takes that count plus one empty
  content pop, marking the slot drained for reclaim. Root child facts
  drain by `pending_facts()`. The inline wrapper exposes
  `pending_slot_facts()` for its lifecycle facts; its native root-fact
  drain pops until empty. The agent service's framed fact drain uses its
  work quantum.
- `protocol/events.md`, 3.2 and 6; `protocol/limits.md`, 9: built:
  events v4 and `smith check --json` name `domain.content_edge_records`
  for the root's optional `Queue<Content>`, with overflow dropped and
  counted. Run, session, tools and inline lifecycle facts have no
  configured allowance. Local-domain's own observation queue retains its
  cap; host-domain's separate legacy setting remains independent.
- `testing.md`, 6: built: `LISTING_TERMINALS_PEAK_CEILING` is 160,000
  bytes for a measured 124,960-byte total, applying a quarter of headroom
  rounded up after held and peak accounting became one total. It is the
  only raised ceiling (from 150,000); other remeasured ceilings fell or
  stayed unchanged.

`protocol/transcript.md`, 7, and `domain/host.md`, 3: built: generated
`encoded_bound` and `decoded_bound`, and host `Delivered::owned_bound`.
The transcript v6 generator uses `Names::Bounds`; its golden bytes are
unchanged. The root's `feedback_bound` and `ConfiguredModel::owned_bound`
have no old-name aliases, and jig and temper use the bound names.

`testing.md`, 2 and 6: built: patch and drift tests check the
generated bound vocabulary through `skein_codegen::generate`;
world stages ask for no room and the run world's order log keeps no room
field. World content settings and calm fact inventories are named
constants independent of entrance allowances.

## Top-tier assertions kept

`testing.md`, 2.5: product program admission through private
`Local::with_admission`, state-directory creation before the lower
factory, provider login-notice selection in `provider::run` against real
files and the kernel, and private `skein-world` `end_to_end::Binary`
capture (including the benchmark stderr assertion) stay at the top tier
until a seam is designed.

kept: 29 `end_to_end` tests: no lower test holds the whole assertion yet.
Lower partial assertions do not replace their external effects, exact diagnostics, output ordering or product admission.

- `credentials::signing_in_again_uses_the_issued_id`
- `credentials::another_subject_preserves_the_record`
- `credentials::a_declined_plan_scope_keeps_the_record`
- `credentials::a_duplicate_subject_names_its_account`
- `credentials::an_unconfirmed_revocation_says_where_to_disconnect`
- `credentials::a_racing_first_login_preserves_the_winner`
- `a_first_run_signs_in_and_ends_with_a_commit_in_place`
- `smith_on_a_terminal_signs_in_and_commits_in_place`
- `the_live_outcome_referee_runs_on_the_shared_fake_binary_harness`
- `standard_profile_runs_a_large_coding_write_and_the_following_checks`
- `local_chat_store_io_failure_keeps_exit_1`
- `smith_check_names_both_sides_of_a_budget_past_the_profile_ceiling`
- `smith_exec_replaces_the_result_file_with_only_its_text`
- `smith_exec_json_writes_its_result_file_beside_the_event_stream`
- `smith_exec_result_file_failure_warns_before_a_parked_answers_last_line`
- `smith_exec_ephemeral_removes_its_private_files_after_a_failed_answer`
- `smith_exec_ephemeral_keeps_a_foreign_file_and_warns_before_the_last_line`
- `an_ephemeral_directory_is_removed_when_product_startup_refuses`
- `program_check::check_exits_3_for_a_program_account_not_signed_in`
- `program_check::check_says_a_program_signed_in_otherwise`
- `program_check::a_failed_program_check_is_exit_3`
- `program_check::exec_refuses_a_program_account_not_signed_in_before_its_run`
- `program_check::check_inspects_an_unused_program_account_without_refusing_startup`
- `program_rechecks::program_exec_keeps_the_agent_check`
- `program_rechecks::program_chat_refuses_a_signed_out_second_run`
- `program_rechecks::program_chat_runs_both_lines_after_input_ends`
- `program_runs::a_program_run_never_sees_the_person_s_credential_variables`
- `program_runs::a_program_turn_exit_keeps_the_run_failure`
- `program_runs::a_program_chat_completes_its_live_view`

The pinned product adapter/check derivation in
`smith/live_configuration::live::tests::the_product_adapter_user_layer_passes_the_shipped_check`
also remains: no lower test loads the pinned product layer and checks the
whole derived check deadline. The package retains 61 `end_to_end` tests
and 70 focused tests in total; the shrink removed 32 upper tests.
