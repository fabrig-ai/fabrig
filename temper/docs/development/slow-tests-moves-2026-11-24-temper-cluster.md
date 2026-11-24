# temper's cluster, restore, checkpoint and start-over stories moved below

2026-11-24. The record that `temper/docs/design/draft/slow-tests-down.md`, 3.2 asks of its goal D18 (section 6): each story of `temper-box::process` that the goal deletes, shortens or folds, and beside each of its assertions the lower test that holds it. It was read again at `5eeadfc9ad`, from the survey `slow-tests-survey-2026-11-23-temper-1.md`, groups 3 and 4, and two read-only surveys of that commit. Failure-worlds' restart faults have landed (`temper/docs/design/draft/failure-worlds.execution.md`, 1, F8), so every combination moves now. *Added* marks a holder the goal's own change adds, under the name given here; every other holder already ran at that commit. Binaries are named as `tools/tiers/layers.toml` names them; paths are under `temper/` unless they name their tree.

What stays above is `temper/docs/design/testing/process-tier.md`'s: the native kill, stop and exit witnesses of the cluster's processes, the shipped deployment binaries' start, the copied place's launch, the checkpoint commands' dispatch and one start on a restored generation, and one owned setup killed, started again and removed.

## 1. The cluster's stories

Unchanged: `an_engine_alone_prints_its_lines_and_drains_on_a_signal`, `a_mixed_document_exits_1_naming_its_path`, `the_cluster_stops_in_reverse_each_exiting_0`, `a_worker_alone_with_the_owned_examples_worker_links_and_drains_on_a_signal`, `a_task_runs_on_each_worker_of_the_deployments_binaries` and `an_unexpected_member_exit_is_reported_after_its_client_settles`. Each kills, stops or starts a process and reads its end.

### 1.1 A task on each worker

`a_task_runs_on_each_worker`: **gone**. The cluster's fixture starts its engine and workers by `temper-engine` and `temper-worker` already (`testing/temper-box-kit/src/cluster.rs`, `Cluster::read`'s bindings); the deployment binaries' story adds `temper-agent` as each worker's agent.

| Assertion | Holder |
|---|---|
| One chat on each one-slot worker, placed apart | `temper-engine-domain-world::suite` `a_lost_workers_run_is_placed_on_the_other_worker_past_its_grace` (the two first assignments on distinct channels) |
| The members' start, their link, the agent's reply, the reverse stop exiting 0, no survivor | `temper-box::process` `a_task_runs_on_each_worker_of_the_deployments_binaries` |
| The chat closed with no result, its reply once in its committed turns, every page read whole | `temper-engine-domain-world::suite` `closing_an_open_chat_keeps_its_turns_and_settles_its_delegates`; the same oracle in the deployment binaries' story |

### 1.2 A worker killed mid-run

`a_worker_killed_mid_run_is_lost_and_its_run_answered_once_on_another`: **shortened** to `a_worker_killed_mid_run_ends_by_its_signal_and_its_link_closes`: w1 holding a run, killed before its tree, ends by signal 9 with no stop escalation; the engine reads its link closed; the other members stop in reverse, exiting 0, nothing left. It waits no grace.

| Assertion | Holder |
|---|---|
| No replacement within the grace; past it, another attempt on w2, from the committed turns, a stale turn refused | `temper-engine-domain-world::suite` `a_lost_workers_run_is_placed_on_the_other_worker_past_its_grace` and `a_worker_lost_mid_run_resumes_at_the_last_committed_turn`; `jig-core-world::suite` `a_vanished_worker_is_lost_and_another_worker_takes_the_retry` |
| Two begins; the killed attempt ends lost with no committed reply; the replacement parks and its reply is committed once | `temper-engine-domain-world::suite` `a_lost_attempts_reply_never_commits_and_its_replacement_commits_it_once` (*added*) |

### 1.3 A worker hung

`a_worker_hung_past_the_grace_is_fenced_and_links_again`: **shortened** to `a_worker_stopped_past_the_silence_links_again_as_the_same_process`: w1 stopped (`SIGSTOP`) past the engine's silence, its host read disconnected, continued within the fleet's grace; the same process links again and the same attempt answers there, one begin, nothing placed twice; the cluster stops in reverse. Only w1 starts; the second stop, past the grace, goes.

| Assertion | Holder |
|---|---|
| Past the grace the run is placed on w2; w1's listing of the fenced attempt is cancelled, its terminal and turn dropped | `temper-engine-domain-world::suite` `a_lost_workers_run_is_placed_on_the_other_worker_past_its_grace`; `jig-domain-fleet` lib `a_fenced_attempt_listed_again_is_cancelled_again` and `turns_from_fenced_attempts_are_acknowledged_without_freeing_the_host`; `jig-core-world::suite` `a_worker_returning_past_grace_cannot_charge_or_finish_its_old_attempt` |
| The fenced attempt's activity and the chat's committed turns unchanged by what w1 sends once back; w1's one slot takes a new run while the replacement is live; the reply in the turns once | `temper-engine-domain-world::suite` `a_fenced_attempts_late_traffic_changes_no_turn_and_its_worker_takes_new_work` (*added*) |
| Within the grace, the same attempt kept and its retained turns sent once | the shortened story; `jig-channel-world::suite` `losing_a_link_within_the_grace_keeps_the_native_run`; `jig-core-world::suite` `a_worker_reconnects_and_resends_retained_turns_after_hello` |

### 1.4 A worker started later

`a_worker_started_later_takes_the_waiting_task`: **gone**.

| Assertion | Holder |
|---|---|
| The second chat due, unclaimed and unplaced while w1's one slot is taken; its first and only assignment to w2 at w2's hello | `temper-engine-domain-world::suite` `a_late_workers_hello_takes_the_task_waiting_for_a_slot` (*added*); `jig-domain-fleet` lib `a_start_waits_for_a_worker_then_for_a_slot` |
| A worker started after the engine links and its process ends 0 | `a_worker_killed_mid_run_ends_by_its_signal_and_its_link_closes` starts w2 once w1 holds its run |

### 1.5 A worker restarted at its position

`a_worker_restarted_on_its_directories_links_at_its_position`: **gone**.

| Assertion | Holder |
|---|---|
| w1 stopped alone, exiting 0, started again on its directories, `linked` again and the engine's w1 again | `temper-box::process` `a_worker_restarted_mid_delivery_links_again_on_its_directories` (2.2) |
| A fresh hello at the same position after a settled run keeps its position and takes a new task there | `temper-engine-domain-world::suite` `a_worker_back_at_its_position_after_its_run_ended_takes_new_work_there` (*added*); `a_hello_keeps_its_configured_worker_position` |

### 1.6 Case 4.6's restarted workstation

`a_restarted_workstation_keeps_the_directors_question_and_answers_it_once_as_processes_without_a_hang_past_the_grace`, ignored and run nowhere: **gone**, with its adapter in the box kit (`testing/temper-box-kit/src/case.rs` and what only it reads). Its walk is the real-loop world's.

| Assertion | Holder |
|---|---|
| The director's first placement on ws, its read question kept unanswered across ws's stop, the retry elsewhere fenced, the person's answer delivered once | `temper-engine-domain-world::suite` `a_stopped_director_keeps_its_read_unanswered_question_once`, `a_stopped_chat_retries_elsewhere_with_a_fenced_journal` and `a_person_answers_a_question_its_task_has_already_read`; `temper-real-loop-world::suite` `a_restarted_workstation_keeps_the_directors_question_and_answers_it_once` |
| ws's two lives, one link each, the reverse stop | the shortened stories of 1.3 and 2.2 |
| No worker's input holds the forge's admin token | `temper-box::suite` `support_tests::cluster_turn_worker_inputs_exclude_the_forge_admin_token` (*added*) |

## 2. Landing by turn on the cluster

### 2.1 An engine killed mid-proof

`a_change_lands_by_turn_across_an_engine_killed_mid_proof`: **shortened** to `an_engine_killed_is_started_again_on_its_directories_and_its_worker_links_again`: a chat answered on w1, the engine killed, ending by signal 9; started again on its directories, it opens, w1 prints `linked` again, and the chat's committed turn reads as before; the cluster stops in reverse, exiting 0, nothing left. The landing and its paced proof go.

| Assertion | Holder |
|---|---|
| The landing found again after the kill: the same run, one preparation, one dispatch, one fast-forward; main at the candidate; the change landed, no hold | `temper-engine-domain-world::suite` `an_engine_restarts_in_each_turn_landing_phase`; `temper-engine-forge-world::suite` `a_restart_finds_the_dispatched_run_without_dispatching_again` and `a_timed_out_merge_found_merged_lands_every_member_at_its_prefix` |
| One ended landing; the member's and the round's pull requests merged; one dispatched run with its one job passed at the candidate; main's file holding the change's bytes | `temper-engine-forge-world::suite` `a_landing_of_one_is_prepared_on_its_branch_proved_and_fast_forwarded`; `a_restart_finds_the_dispatched_run_without_dispatching_again` (*added* reads, where it lacks them) |
| A crash at any iteration, a landing's phases among them | `temper-engine-forge-world::suite`'s restart sweep (`FORGE`; `temper/docs/design/testing/restarts.md`, 5 and 9) |
| The chat closed | `temper-engine-domain-world::suite` `closing_an_open_chat_keeps_its_turns_and_settles_its_delegates` |

### 2.2 A worker restarted mid-delivery

`a_worker_restarted_mid_delivery_lands_its_saved_work_by_turn`: **shortened** to `a_worker_restarted_mid_delivery_links_again_on_its_directories`: the delivery's push held by the story's git wrapper, w1 stopped alone, the push let go, w1 drains and exits 0; started again on its directories, it prints `linked` again and is the engine's w1; the change's one pull request reads its head holding both lines; the cluster stops in reverse, nothing left. The landing by `turn` goes.

| Assertion | Holder |
|---|---|
| The saved line and the later one reach main once, by one landing and one dispatch, nothing written twice | `temper-engine-domain-world::suite` `a_saved_delivery_survives_worker_loss_and_engine_restart_beside_proof` |
| A worker signalled while its delivery's push is held delivers once, its acknowledgements settled, and comes back on the same disk | `temper-worker-domain-world::suite` `a_worker_signalled_while_its_delivery_push_is_held_delivers_once_on_the_same_disk` (*added*); `workers_stop_at_each_turn_point_and_crashes_resume_on_same_or_another_worker` |
| The landing's reads (2.1's second row) | as 2.1 |

### 2.3 A unit stop

`a_unit_stop_settles_a_workers_held_turn`: **shortened** to its signals and their settling: the worker alone signalled as its unit does and a stray termination to its run's agent, reported without cancelling the held attempt; the held answer let go within the printed drain; the turn committed, read once; exit 0 before the printed stop bound; no process live at its exit and no forced cleanup.

| Assertion | Holder |
|---|---|
| The same attempt: its queries and turns, the released text once, no further query, one interrupted terminal, the saved branch holding the released write | `temper-worker-domain-world::suite` `a_drained_worker_and_a_stray_agent_signal_commit_the_held_turn_once` (*added*: the worker alone signalled, `Way::Signalled`, while its agent's model answer is held, the world as the agent reporting a stray signal of its own); `smith-agent-process-world::suite` `a_signalled_turn_is_committed_before_the_hosts_drain_ends_it`; `temper-worker-domain` lib `an_agents_signal_is_not_a_workers_cancel`; `temper-worker-domain-world::suite` `a_shutdown_drains_the_turn_saves_its_work_and_waits_for_every_acknowledgement` |
| That interrupted attempt due again with no try spent, no retry and no hold | `temper-one-process-world::suite` `a_signal_mid_run_interrupts_a_coding_run_and_the_next_start_takes_it_again` |

## 3. A deployment restored at its place

`a_stopped_deployment_restored_at_its_place_uses_new_files`: **shortened**. It keeps the copied journeys' fresh walk at the place, the stop, the archive, removal and unpacking of `config/` and `data/`, each file of `bin/` replaced, and one start of the restored deployment from the new `bin/` by `drive --ctl --deployment`, at whose ready every process runs a new file by device and inode, its rendered sockets and agent pin as now, one ctl read finding the project and its repository as they were, no sign-in link in any stream or kept log, exit 0 and nothing left. The check-only walk from the copy, the baseline drive, the fake Forgejo API reads and the other ctl reads go.

| Assertion | Holder |
|---|---|
| The walk's statuses and chain hash, fresh and from the copy; the patch's restart | `temper-client-journeys-tool` lib `walk_success_and_session_failures_keep_their_statuses`, `ctl_chain_plans_a_ctl_session_without_resets_from_fresh_or_copy` and `patch_validates_before_stop_then_restarts_engine_and_worker_without_sign_in` |
| Project 1 uncut with its one home, the managed forge connected, `work/hello` listed with no more page, no landing; the settings revision by `turn`, `batch` 2, one blocking `gate.yml` on forge and repository 1; each read the same on the next start | `temper-one-process-world::suite` `a_stopped_deployments_project_forge_settings_and_landings_read_the_same_on_its_next_start` (*added*); `floor_golden_store_opens_and_serves_the_kept_person`; `temper-engine-domain-world::suite` `changed_settings_restore_before_accounts_and_survive_a_restart` |
| The fake Forgejo's repository, its run and job on `main` the same, one dispatch and none again | `temper-engine-forge-world::suite` `a_restart_finds_the_dispatched_run_without_dispatching_again`; `skein-fake-forge-domain` lib `an_image_round_trips_the_lasting_forge_and_rebuilds_its_indexes` |
| An older layout opened, or refused below the floor | `temper-one-process-world::suite` `an_older_layout_opens_and_serves_the_kept_person`, `dev_image_opens_and_serves_on_this_build`, `a_story_keeps_its_history_through_a_settled_upgrade` and `floorless_golden_store_is_refused_without_writing` |

## 4. Checkpoint and restore

`a_stopped_checkpoint_is_listed_under_its_hold_then_removed` and `a_checkpoint_restores_earlier_state_and_its_safety_restores_the_later_state`: **folded** into `a_stopped_checkpoint_is_held_restored_and_removed`, on one fixture: a stopped `checkpoint`, its name and cut printed and its record published; `--hold` running a child that inherits its directory and environment, reads `TEMPER_CHECKPOINT`, lists the record, and whose exit 3 reaches the caller; a start that commits a project past it, a stop; `restore`, printing its source, generation and safety checkpoint and the undo; one start whose `me` reads the earlier projects; `--remove`, printing the name and leaving no directory. The safety checkpoint's restore and its start go.

| Assertion | Holder |
|---|---|
| Restoring the safety checkpoint brings the later state back | `jig-store-world::suite` `a_restore_and_its_safety_undo_recover_each_maps_logs_and_transcripts`; `temper-engine-shell::suite` `stopped_restore_switches_generation_and_keeps_source_and_safety_and_refuses_missing` |
| The commands' words and exits | `temper-engine-shell` lib `terminals_render_exact_checkpoint_lines_and_child_exits_for_each_binary` and `restore_terminals_render_exact_lines_and_exits_for_each_binary` |
| A stopped checkpoint's record and its removal | `temper-engine-shell::suite` `stopped_checkpoint_stands_with_its_record_and_removal_reports_missing_twice`; `jig-store-world::suite` `a_snapshot_finishing_under_the_hold_renames_after_release_and_the_copy_keeps_the_old_one` |
| `--hold`'s child built from its invocation | `temper-engine-shell` lib `path_resolution_requires_an_executable_and_spawn_uses_the_invocation` |

## 5. An owned setup cut short

`owned_setup_cuts_start_over_and_a_removal_cut_finishes_on_restart`: **shortened** to `an_owned_setup_killed_at_its_run_is_kept_and_a_removal_cut_finishes_on_restart`: the owner's set up killed with the box once its Forgejo runs; the next box finds the forge kept and its first repository not made; its removal answered, the box held after the answer and killed; the next box ends the removal whole at its start, no forge listed and no `forge-N`, `box/groups/N` or `box/started/N`; the cache's binaries those placed, unchanged; each box's tree drained. The fetch, preparation and started cuts, and the fifth setup, go.

| Assertion | Holder |
|---|---|
| A setup cut at its fetch, its preparation, its run and after its `started` record before the engine hears it: the group and its records as the cut leaves them, the next start's group dormant or restored, a removal answering removed and unlinking the forge's directory, its group file and its started record | `temper-box-machine` lib `an_owned_setup_cut_at_each_step_is_kept_unfinished_and_removed_whole` (*added*); `a_group_file_without_its_record_stays_dormant`; `a_kind_start_writes_then_runs_and_removal_unlinks_both_records` |
| The engine's half: a forge whose `started` it never heard kept unfinished across its restart, its first repository not made | `temper-engine-domain-world::suite` `a_forge_whose_start_was_never_heard_stays_unfinished_across_a_restart` (*added*); `an_asked_forge_takes_its_handover_at_started_and_connects` |
| A removal cut after its answer ends whole at the next start | `temper-box-machine` lib `a_removal_cut_after_its_answer_finishes_at_the_next_start` (*added*); `temper-engine-domain-world::suite` `removal_cut_after_decision_ends_whole` and `removal_cut_after_secrets_erased_ends_whole` |
| A setup after the cuts makes the next number, connects it and makes its first repository, with no download | `temper-engine-domain-world::suite` `a_removed_forges_number_is_never_given_again`; `temper-engine-service-component` lib `reconcile_then_create_then_connect_makes_the_first_repository_and_bot_workflow`; `temper-forge-setup-shell::suite` `local_tls_fetch` |
| A stop or crash at any moment of the engine's start | `temper-engine-domain-world::suite` `crash_with_an_unanswered_commit_and_forge_call_keeps_or_loses_the_commit` and the engine's restart sweep (`temper/docs/design/testing/restarts.md`, 4 and 9) |
