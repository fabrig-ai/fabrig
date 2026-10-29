# Status

Updated 2026-10-29. How far smith's code is from its design.

| Area | Built | Not yet |
|---|---|---|
| `domain/README.md` | The run, session, tools and local-host domains, including transcripts, resume, delivery and answered-call recovery. | 4: MCP and supervised processes with a view. |
| `domain/run.md` | 3.1, 6, 9.1: explicit Open mode, text-only replies that listen, and native text-only final turns. 3.3, 5.3, 8.1, 9.1: framed compaction summaries, jobs, checks and budgets. 3.4: an Open run opens with its person's words as main's first user message. | 5.1: MCP tool source. 5.3, 15: work beside writing children and children from a parent prefix. 6, 9.2: mid-work messages and threshold notes. |
| `domain/session.md` | 3 to 5, 8.1, 8.2: account grants, jobs, bounded prompt window, failed-write ordering and framed compaction with `ToolChoice::None`. | 11: provider-native compaction. |
| `domain/tools.md` | 3 to 5: jobs and waits that yield to other calls. Other implementation detail unknown. | 5: containment. |
| `domain/host.md` | 2, 3: main-conversation live text handed to the owner. 4, 8, 9: local host and inline agent; generated older-store admission is prepared. | 2, 9.1: in-process LLM deltas. 4: contained trees. 8: MCP host tools. 12: isolated mode. |
| `protocol/README.md` | The codecs, channel halves, LLM/machine components, agent process and spawning/local hosts; 8: credential schemes and versioned account notices. | 4, 11: connected agent. 5: MCP. 9: contained trees, Codex routing extras and WebSocket. |
| `protocol/channel.md` | 3 to 8: start's live-text ask and bounded best-effort main text pieces; credential grants with schemes and typed notices. | 2, 7: connected-agent credential/refusals. 4: reasoning/input live-content producers. |
| `protocol/charter.md` | 2, 5, 6: charter/result codecs, credential forms and prompt-window material; recorded as built. | — |
| `protocol/transcript.md` | 2 to 6: turns, jobs, failed-write ordering and account-aware transcripts; recorded as built. | — |
| `protocol/llm.md` | 3 to 7: jobs, bounded prompts, compaction, channel live-text deltas and named credential dialects. | 6: reasoning/input live producers. 12: edit formats per dialect. |
| `protocol/agent.md` | 2, 4: machine component, jobs and program-backed agents. | 3, 7: view and contained processes. |
| `protocol/events.md` | 3.6, 3.9, 3.10, 6: job events, account/program notices and compaction material. | 3.2: `containment: "trees"`. |
| `protocol/limits.md` | 2 to 7: derived edge limits, jobs, task role, prompt guard, account notices and bounded live text; memory returned when use ends. 3.8: agent and local watermarks, uncapped internal stages and hand-offs, stated dispatch orders and shipped pacing. 5: edge validity without a process heap sum or startup memory fit check; declared memory parsed and handed on. 11: measured scenario ceilings. | 3.8: lags: sent-message timestamps remain capped, until `ready/smith/lc6-live-queues` lands. Turn measurements and lossy live pieces stay capped at their edges. |
| `protocol/hosts.md` | 2: on-demand reads, one record/piece/end each. 3: reap after demanded end. 5.2 to 5.6: program accounts and login/logout; 5.3: generated local-store layout admission is prepared. | 3: contained tree. 4: connecting to an agent later. |
| `protocol/programs.md` | 1 to 9: Claude Code backend, its fake, sign-in, tools, stream, notices, budgets and sessions; live checks exist. 5.9: Open runs on a program backend (replies, listening, parking, wind-down; the domain's Yield request; the stream's JSON token bound from its line ceiling); the fake scripted per conversation by cue, reading by line, and its built-ins acting on the workspace. The real program's stream (system/init without the prompt tool, system notices passed over, a later init checked as the first, a thinking turn's records under one message id as one completion, its final usage), a result contract's StructuredOutput, the program's own call; and a turn interrupted by the control request, SIGINT a cancel's alone; an interrupt of the agent's says so in the next message; the live suite passes against Claude Code 2.1.311. every MCP tool schema an object with no top-level combinator, alternatives under `outcome`; one outcome path per run (StructuredOutput under a contract, finish without). | |
| `shell.md` | 4 to 7: interactive/exec product, task role and account sources. 4.1, 4.3, 5.3: older-chat refusal words, exit 1 and trace admission are prepared. 2.5: the hosted face, portable, checked for wasm32. | 5.1: sync startup directories into parents. 8, 9: per-tree limits and `isolate`. |
| `testing.md` | Simulated, real-loop, process and opt-in live tiers. 2.5, 6.1: real-directory adapter and separate QA profile are prepared. 2.5: whole upper assertions kept until lower seams exist. 6: measured total-peak ceilings. | 2.1, 2.2, 6: fault sweeps, real signals, richer referees, tiny limits, paired job ends, state-digest replay and containment. 2.3, 4, 8: own-account live checks exist but have not run. |
| `benchmarks.md` | 3, 5, 6, 8 to 11, 16: tasks, pins, pricing, summaries and parity verdict; wall-clock comparison includes RT. Harness-login, Claude Code arm and credentials probe exist. | 3.1, 5.2: missing/outdated calibrations. 6, 7, 10, 11.1: Claude Code comparison, Anthropic probes and real harness-account runs; mid-work/build-environment probes. 11.3: delegation experiment. 15: concurrency. 16.6: projection excluding a change's own ratio. |

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
- `protocol/llm.md`, 8: lags: completion, rendering, job-problem and
  size-refusal functions retain their old names, until
  `ready/smith/d-domain-names` lands; that branch supplies
  `completion_bound`, `render_bound`, `job_problem_bound` and
  `size_refusal_bound`. `domain/README.md`, 4: lags: the root's feedback
  function retains its old name, until `ready/smith/d-domain-names` lands,
  supplying `feedback_bound`.
- `domain/tools.md`, 6: lags: result receiving bounds retain their old
  names, until `ready/smith/d-tools` lands; that branch supplies
  `result_bound` and `batch_result_bound`.
- `testing.md`, 6, and `protocol/limits.md`, 5 and 11: built: remeasured
  total-peak ceilings in run, session, host, hosted-process, machine,
  protocol-machine, local-process store and agent fuzzy scenarios. No
  world exercises a domain's output flow control. The hosted-process
  ErrorTail case shares the scenario's measured total ceiling; it has no
  separate 4,096-byte allowance.
- `domain/session.md`, 10: the two 240-byte checks in
  `smith/crates/smith-domain-session/src/tests/builtin.rs` and
  `smith/crates/smith-domain-session/src/tests/program.rs` are
  exact type-size contract checks on `size_of(Turn) + size_of(Kind)`, not
  allocator ceilings. Their implementation checks `<= 240`; their
  `*_RECORD_CELLS_CEILING` names and headroom comments still describe them
  as ceilings. No matching naming repair is on main or a `ready/smith/*`
  branch; that source item is left for a repair.

The remaining cross-tree compatibility names are not claims of internal
room reservation or a process heap formula: `smith-domain-session::max_out`,
`smith-protocol-channel::max_out`, host `Delivered::worst_case`, root
`feedback_worst_case`, and transcript `Turn::worst_case_bytes` remain on
main for their readers. The root feedback rename is on
`ready/smith/d-domain-names`; generated transcript bound names with old
wrappers alongside are on `ready/smith/b-gen-1`. Their removal waits for
readers to move.

## Top-tier assertions kept

`testing.md`, 2.5: product program admission through private
`Local::with_admission`, state-directory creation before the lower
factory, provider login-notice selection in `provider::run` against real
files and the kernel, and private `skein-world` `end_to_end::Binary`
capture (including the benchmark stderr assertion) stay at the top tier
until a seam is designed.

The following 29 `end_to_end` tests are kept: no lower test holds the
whole assertion yet. Lower partial assertions do not replace their
external effects, exact diagnostics, output ordering or product admission.

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
