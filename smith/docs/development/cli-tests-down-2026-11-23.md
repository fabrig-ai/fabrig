# smith's command-line tests below end to end: where each goes

2026-11-23. For goal D13 of `temper/docs/design/draft/slow-tests-down.md` (5.3), from four read-only surveys of `smith::end_to_end` at the goal's commit `b574a13436`, checking `../../../temper/docs/development/slow-tests-survey-2026-11-23-smith-jig.md`, group 6. The placement is `smith/docs/design/testing.md`, 2.3 and 2.5; the seams are `smith/docs/design/shell/README.md`, 2.3.

The binary's sources (`smith/tests/end-to-end/end_to_end.rs` and the modules beside it, not `live*.rs`) hold 76 test functions and, counting each independently asserted table row, 95 cases (the survey's 92 counted the withdrawn-settings and offered-tools tables otherwise). **32 cases in 31 functions stay** (section 1), one per path through `main`. **63 cases in 45 functions move** and one function is shortened, in ten groups (section 2), one change each. Nothing is checked less (the draft, 3.2): each moved assertion is held by the existing test named **held**, or by the lower test its group adds, named **new**. A new holder takes the slow test's name unless an existing test is extended.

## 1. What stays

| File | Tests | Path through `main` |
|---|---|---|
| `end_to_end.rs` | `the_terminal_modes_are_unchanged_after_smith_exits`, `end_of_input_at_the_terminal_lets_the_run_answer_then_exits_0`, `a_window_size_change_reaches_the_status_line`, `an_interrupt_cancels_the_run`, `the_shipped_local_and_agent_answer_on_a_terminal`, `a_first_run_signs_in_and_ends_with_a_commit_in_place` | `smith` on a pseudo-terminal: modes, end of input, `SIGWINCH`, `VINTR`, a configured start, a first sign-in through the real callback listener and git |
| `end_to_end.rs` | `a_product_settings_metadata_refusal_reaches_stderr`, `local_host_io_failure_keeps_exit_1` | startup refusals: the native settings file, the native state directory |
| `end_to_end.rs` | `smith_check_json_shows_each_value_with_its_layer` | `smith check --json` |
| `end_to_end.rs` | `smith_exec_json_ends_accepted_with_its_events_and_exit_0`, `smith_exec_json_writes_its_result_file_beside_the_event_stream`, `smith_exec_ephemeral_removes_its_private_files_after_acceptance`, `the_trace_file_is_read_as_it_grows` | `smith exec`: `--json`, `-o`, `--ephemeral`, `--trace` |
| `end_to_end.rs` | `smith_login_signs_in_against_the_fake_issuer_and_exits_0` | `smith login` on a configured registration |
| `end_to_end.rs` | `smith_agent_spawned_by_the_sample_host_answers`, `an_agent_configuration_metadata_refusal_reaches_stderr`, `an_agent_channel_end_is_reported_to_stderr` | `smith-agent`: spawned, its document refused, its channel ending |
| `credentials.rs` | `a_first_sign_in_registers`, `logout_revokes_the_record`, `key_commands_keep_values_private` (its piped and terminal logins only, two cases) | `smith login` registering through the preset; `smith logout` revoking; a key from a pipe and refused at a terminal |
| `program_check.rs` | `check_reports_a_signed_in_program_account_as_usable`, `exec_refuses_a_program_account_not_signed_in_before_its_run` | a program account checked by `smith check`, and refused by `smith exec` |
| `program_commands.rs` | `program_login_checks_status_without_inheriting_host_credentials`, `program_logout_removes_the_login_and_checks_status` | `smith login` and `smith logout` on the fake program |
| `program_runs.rs` | `exec_spawns_one_print_run_on_its_program`, `a_program_chat_answers_before_input_ends`, `a_program_turn_exit_keeps_the_run_failure`, `a_program_chat_shows_text_before_interrupting_an_uncommitted_turn` | a program account's exec, plain chat, the program's exit, an interrupt at the terminal |
| `program_children.rs` | `exec_on_claude_asks_two_sub_agents_side_by_side` | the program's descendants |
| `program_lifetime.rs` | `an_aborted_agent_takes_its_program_and_relay_with_it` | a process tree killed |
| `relay.rs` | `the_agent_binary_refuses_a_relay_without_its_secret` | `smith-agent --mcp-relay` refused |

## 2. What moves

`LP` is `smith-local-process-world::suite` (`smith/tests/local-process/tests/suite/`), `AP` `smith-agent-process-world::suite`, `HP` `smith-hosts-world::suite`, `S` `smith-local-shell`'s lib tests (`smith/crates/smith-local-shell/src/`), `step` `smith::step` (`smith/tests/step/`). "No value" is the slow test's check that no synthetic secret reaches standard output or error.

### 2.1 The result's file and the private chat (LP, the finalizer's seam)

| Slow test (`end_to_end.rs`) | Its assertions | Holder |
|---|---|---|
| `smith_exec_replaces_the_result_file_with_only_its_text` | a longer `answer.txt` before; exit 0; the file and plain standard output exactly the answer; the last line starts `smith: accepted in ` | new LP |
| `smith_exec_result_file_failure_exits_1_after_an_accepted_run` | a directory at `answer.txt`; exit 1; the last line exactly `smith: run accepted, but its result could not be written to answer.txt: IsADirectory`; the last record `session.ended` with exit 1; `run.completed` accepted; the destination still a directory | new LP; held in part: `smith-local-service`'s `result_failure_changes_acceptance_before_the_final_exit` |
| `smith_exec_result_file_failure_warns_before_a_parked_answers_last_line` | the same destination, a waiting reply, `--chat plan`; exit 5; the `IsADirectory` warning the line just before the needs-input line naming `smith exec --chat plan`; the last record's exit 5; `run.completed` parked | new LP; held in part: `result_failure_keeps_the_parked_exit_behind_shell_output` |
| `smith_exec_ephemeral_removes_its_private_files_after_a_failed_answer` | a content-filter reply; exit 1; `session.ended` exit 1; `run.completed` failed; nothing left beneath the runtime directory, and no chat beneath the state directory | new LP; held in part: `failure_removes_private_records_and_their_directory` |
| `smith_exec_ephemeral_keeps_a_foreign_file_and_warns_before_the_last_line` | a foreign file put in the private directory during the run; exit 0; that directory kept, alone beneath the runtime directory, mode `0700`, holding the foreign file alone, its bytes unchanged; the path-naming `NotEmpty` warning the line just before the acceptance line | new LP (all but the mode); new S beside `compose/files.rs`'s tests: the private directory made natively beneath its base with mode `0700`; held in part: `a_foreign_file_survives_and_warns_before_exit` |
| `an_ephemeral_directory_is_removed_when_product_startup_refuses` | `--ephemeral` with a trace beneath an absent directory; exit 1; nothing on standard output; nothing left beneath the runtime directory | new, S or LP: a start refused after the private directory was made hands it to the cleanup, which removes it through the kernel; held in part: `compose/cleanup.rs`'s tests |

### 2.2 A program account in the local host's world (LP)

| Slow test | Its assertions | Holder |
|---|---|---|
| `chat_crossing.rs`: `an_exec_chat_crosses_from_direct_to_a_program_and_back` | one named chat, three headless legs, direct, program, direct; each exit 0, one accepted result and one answered, unaborted session end; one start per leg, resumed `false`, `true`, `true`; the middle leg's records in its `.agent` trace, its two print runs, the summary without MCP and then the window with it | new LP, over the same chat files across its invocations, no transcript seeded by hand; held in part: LP `a_chat_resumes_from_its_files_across_runs_of_the_binary`, AP `a_conversation_is_carried_over_from_the_direct_backend` and `a_conversation_is_carried_over_to_the_direct_backend` |
| `program_runs.rs`: `a_program_exec_traces_its_run_to_the_agent_file` | exit 0, accepted; the `.agent` trace with a fresh start, the prompt's text, a completed `finish`; the product's stream with its header first, its terminal last, one run completion and none of the child's detailed records (`program_runs/records.rs`) | new LP; held in part: LP `the_spawned_agents_records_stay_in_its_own_trace`, AP `a_program_run_s_events_name_its_backend` |
| `program_runs.rs`: `a_program_chat_completes_its_live_view` | exit 0 after the report shows; its text once; `tool Read: result`; the model and account label; usage and the budget left; no tool duration invented | new LP, on the interactive front, not plain; held in part: AP `a_built_in_call_is_told_with_its_text` |

### 2.3 A program account's rechecks and environment (LP, after 2.2)

| Slow test | Its assertions | Holder |
|---|---|---|
| `program_runs.rs`: `a_program_run_never_sees_the_person_s_credential_variables` | exit 0, the exact report; the account's log status, status, print, each under the account's directory; none of the five credential variables, one `CLAUDE_CONFIG_DIR`, only `ANTHROPIC_CONFIG_DIR` among `ANTHROPIC_*`; no value of the person's key, token or other directory on either output; no MCP `finish` | new LP, one test with the next row, named `program_exec_checks_its_source_without_lending_host_credentials`; held in part: AP `no_credential_variable_reaches_the_program`, `a_program_turn_runs_to_its_finish` |
| `program_rechecks.rs`: `program_exec_keeps_the_agent_check` | exit 0, the exact report; two statuses, the host's and the agent's, then one print | the same new LP test |
| `program_rechecks.rs`: `program_chat_refuses_a_signed_out_second_run` | statuses signed in, signed in, not signed in; the first answer shown; exit 0; the account's login advice exactly on the last line; three statuses, one print | new LP; held in part: `smith-local-service`'s `each_failed_check_returns_its_reason` |
| `program_rechecks.rs`: `program_chat_interrupts_a_silent_recheck` | statuses signed in, signed in, silent; an interrupt once the third status runs; exit 130; three statuses, one print; settled within 5 s, the status child gone | new LP, the interrupt sent on the third status's observation, the bound in the world's time, the child exited and reaped; held in part: `interruption_answers_the_queue_before_the_live_child_settles` |
| `program_rechecks.rs`: `program_chat_runs_both_lines_after_input_ends` | two lines, then end of input; exit 0, two reports; four statuses, two prints, each with MCP; the second's `--resume` the first's `--session-id` | new LP; held in part: AP `a_parked_run_resumes_its_session` |

### 2.4 A sign-in's commit, the standard write and an unreadable head (LP)

| Slow test (`end_to_end.rs`) | Its assertions | Holder |
|---|---|---|
| `smith_on_a_terminal_signs_in_and_commits_in_place` | exit 0; the `Sign in: …/authorize` line; the record saved at its account's name in the credentials directory; the head moved; the four files' exact bytes, `original`, `result`, `command`, `checks-ran` | new LP: the sign-in with the checked change; the record's file name held where the credentials directory's layout is tested (S `credentials/directory/`), existing or new |
| `standard_profile_runs_a_large_coding_write_and_the_following_checks` | a write past 2048 bytes; seven queries; the tools' output in the follow-ups; three files committed with their exact bytes | held: AP `a_standard_coding_write_reaches_following_calls_and_checks` (threshold, queries, results, the checks' input); new LP: the committed bytes in place on the standard profile |
| `exec_in_a_repository_without_a_commit_exits_two_with_its_line` | a repository with no commit; exit 2; the last line exactly `smith: run refused: repository workspace: its head cannot be read`; no provider query | new LP, the fake checkout answering `rev-parse HEAD` as failed; held in part: `a_headless_start_whose_head_cannot_be_read_exits_refused`, `local_failures_keep_their_own_words` |

### 2.5 A grant's refresh (LP)

| Slow test (`credentials.rs`) | Its assertions | Holder |
|---|---|---|
| `an_exec_on_a_raised_lead_refreshes_its_record_at_once` | a current record, the refresh lead past its life; exit 0, one accepted answer; the generation 1 to 2; one refresh asked of the issuer, answered 200; no value; nothing left running | new LP, the refreshed grant saved before the first query and lent to it; held in part: LP `an_expired_grant_refreshes_through_the_shared_private_token_store` |
| `an_exec_before_the_earliest_refresh_time_keeps_its_record` | the same lead, an earliest refresh time ahead; exit 0, one accepted answer; no refresh asked; the generation 1; no value | new LP, the held grant lent; held in part: LP `a_current_sign_in_record_lends_its_grant_without_a_refresh`, skein's `no_refresh_starts_before_the_earliest_refresh_time` |

### 2.6 `smith exec`'s start and words (S)

| Slow test (`end_to_end.rs`) | Its assertions | Holder |
|---|---|---|
| `a_headless_run_that_would_wait_exits_5_naming_its_chat` | a piped prompt, `--chat plan`, a waiting reply; exit 5; the last line ``smith: needs input: answer with `smith exec --chat plan` `` | new S `front/headless.rs`; held in part: `each_local_terminal_has_its_product_code`, smith-local-world's `a_headless_run_that_would_wait_parks_at_once_and_needs_input` |
| `an_empty_prompt_is_a_usage_error_exiting_2` | whitespace on standard input, then its end; exit 2; the last line `smith: usage: the prompt is empty` | new S `front/headless/input.rs` |
| `an_account_without_a_credential_exits_3_before_any_request` | an `env` account, its variable unset; exit 3; nothing on standard output; no chat tree made | new S, the start's credential refusal before any chat, channel or provider |
| `local_chat_store_io_failure_keeps_exit_1` | a file at the chat's path; exit 1; `File exists` in the line; the file's bytes unchanged | new S `compose/tests.rs`, over `chat_paths` |

### 2.7 `smith check`'s words (S)

| Slow test | Its assertions | Holder |
|---|---|---|
| `end_to_end.rs`: `withdrawn_endpoint_settings_exit_two_at_their_key`, six rows (`identity`, `dialect`, `preset`, an endpoint `codex`, a selected `model`, a declared `models.codex/…`) | `smith check` exits 2; standard error the refusal at its file and line, naming the key and its replacement | held: `withdrawn_endpoint_settings_name_the_replacement_at_their_key` (the six origins and lines); new S: a settings refusal through `smith check`'s verdict, exit 2, its line on standard error, nothing on standard output |
| `end_to_end.rs`: `smith_check_names_both_sides_of_a_budget_past_the_profile_ceiling` | `-c budget.turns=5000`; exit 2; nothing on standard output; the line naming `5000 (from -c)` and the standard profile's `4096 turns` | new S |
| `program_check.rs`: `check_exits_3_for_a_program_account_not_signed_in` | exit 3; the exact login advice on standard output; one status | new S `check/catalog_tests.rs`, from the program's answer; held: HP `a_source_never_signed_in_is_not_signed_in` |
| `program_check.rs`: `check_says_a_program_signed_in_otherwise` | `check --json exec`; exit 3; the other method's state, `api`; one status | new S; held: HP `a_source_signed_in_otherwise_names_how` |
| `program_check.rs`: `a_failed_program_check_is_exit_3` | exit 3; JSON `program_failed`; one status | new S; held: HP `an_unreadable_status_fails_as_unreadable` |
| `program_check.rs`: `check_inspects_an_unused_program_account_without_refusing_startup` | a required `work`, usable, and an unused `spare`, signed out; exit 0; `spare`'s metadata, its directory kept, its state and the warning; one status each | new S beside `an_unused_program_account_warns_without_refusing_check` |

### 2.8 `smith login`'s words (S)

Each row also checks its code and no value. The holders are `credentials/login/provider/driver/tests.rs`'s.

| Slow test (`credentials.rs`) | Its assertions | Holder |
|---|---|---|
| `signing_in_again_uses_the_issued_id` | exit 0; the issued client id, host and ID-token hints in the address; no agent hint; no plan notice | held: `a_second_login_reuses_registration_and_subject_hints`; new: the words rendered with the earlier full-scope record |
| `another_subject_preserves_the_record` | exit 2; one visit; the subject's refusal; the earlier bytes unchanged | held: `a_replacement_subject_keeps_the_saved_bytes`; new: its rendered line |
| `a_declined_plan_scope_keeps_the_record` | exit 3; the new record kept; the advice to sign in again allowing the plan, or to use a key | held: `declined_plan_scope_is_saved_and_reported_unavailable`; new: login's rendered advice |
| `a_duplicate_subject_names_its_account` | exit 2; names `first`; no new record, the earlier kept; one revocation | held: `a_duplicate_subject_rolls_back_only_the_new_account`; new: the other account found from the declarations, its line |
| `an_unconfirmed_revocation_says_where_to_disconnect` | `smith logout`; exit 0; one revocation asked; the record gone; the provider's address to disconnect at | held: `unconfirmed_revocation_still_removes_the_record`; new: its rendered line |
| `a_racing_first_login_preserves_the_winner` | exit 2; one visit; the account's refusal; the winner's bytes unchanged | held: `an_arriving_record_wins_before_the_login_save`; new: its rendered line |
| `a_seed_refuses_an_existing_public_file` | the test kit's seed refuses a `0644` file already there, its bytes unchanged | new: a unit test beside the kit's `provider_login.rs::keep_record` |

### 2.9 `smith logout`'s sources and a refused program sign-out (S, HP)

| Slow test | Its assertions | Holder |
|---|---|---|
| `credentials.rs`: `key_commands_keep_values_private`, its three logout rows (a `key` account, a `none` one, an `env` one) | each exit 0; only the key's file removed; no value | new S: logout's decision over each source kind, what it removes and says; removal on disk held: `smith-local-shell::real`'s `piped_keys_replace_privately`, or where goal D12 moves it. The function stays with its two logins |
| `program_commands.rs`: `a_refused_program_logout_keeps_the_login_and_escapes_its_bounded_detail` | exit 3; the login kept; the line with no raw ESC or BEL, CR, LF and controls escaped, the prefix dropped, the exact escaped tail of 1024 raw bytes | new HP: the sign-out refused with its detail, at the shipped 1024-byte bound; new S: that typed outcome's line and exit 3 |

### 2.10 `smith-agent`, the offered tools, the fake's lookup and the live judge (AP, step)

| Slow test | Its assertions | Holder |
|---|---|---|
| `end_to_end.rs`: `an_agent_transport_refusal_is_reported_to_stderr` | a document with a plaintext address off loopback; exit 1; nothing on the terminal; the last line exactly `smith: plaintext endpoint address must be loopback` | new AP `stories.rs`: refused at its start, one line, no channel; held in part: `plaintext_endpoints_refuse_addresses_outside_loopback` |
| `program_runs.rs`: `a_program_exec_offers_the_live_claude_story_s_tools` | exit 0; a run granted inspect, modify and shell; its first nonempty tool list passes the offered-tools check | new AP, through the kit's `offered_tools_fault`; held in part: `a_program_grant_starts_claude_with_its_tools` |
| `program_runs.rs`: `the_offered_tools_check_names_each_departure`, ten rows | `remove` allowed beside the required set; each of `Read`, `Glob`, `Grep`, `Write`, `Edit`, `Bash`, `permission_prompt` missing, named; `finish` or `sub_agent` added, its exact words | new step, the same rows |
| `program_children.rs`: `exec_on_claude_refuses_a_writing_sub_agent` | a child asking `inspect` and `shell`, refused with a reply naming its tools; exit 0; the exact refusal report; only the main's print | extended AP `a_program_child_asking_to_write_is_refused`: the `shell` row, its reply and report, the main's and the children's models told apart |
| `fake_lookup.rs`: `an_executable_fake_is_found_beside_smith`, `a_missing_fake_names_its_build_command`, `a_nonexecutable_fake_names_its_build_command` | a regular executable beside `smith` found by its path; absent, and mode `0600`, each refused naming `cargo build -p smith-fake-claude --bin fake-claude` | new step, the same names, the lookup's decision handed the file's facts |
| `end_to_end.rs`: `the_live_outcome_referee_runs_on_the_shared_fake_binary_harness`, two rows (a report, a change) | the live judge passes a settled report run and a settled change run | new, LP or step: the judge `smith::live` uses, over settled captures; held in part: LP `the_live_report_policy_rejects_mutated_accepted_observations`, `the_live_change_policy_rejects_mutated_delivery_observations` |
