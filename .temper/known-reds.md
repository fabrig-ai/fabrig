# Known reds

Failures on the goal branch that no live change of goal 1706 owns; delivery
checks count them as inherited (`tools/pre-pr/design.md`, section 5).

- lint jig-protocol-store clippy::manual_let_else jig/crates/jig-protocol-store/patches/p0001.rs
- lint jig-protocol-store unfulfilled_lint_expectations jig/crates/jig-protocol-store/patches/context.rs
- lint skein-lib clippy::chunks_exact_to_as_chunks skein/crates/skein-lib/src/crypto_rng.rs

Inherited from main at the goal's start, 2026-11-12.

Failing on this goal's branch and no live change's; delivery checks pass them. Each is dropped once fixed.
- test temper-real-loop-world::suite lost::a_lost_answer_reaches_its_name_from_another_directory
- test temper-box::suite box_listeners::a_fresh_box_prints_its_block_and_nothing_else
- test temper-real-loop-world::suite terminal::up_down_and_the_page_up_line_read_the_conversation
- test temper-real-loop-world::suite terminal::a_person_at_the_terminal_reads_a_run_of_calls
- test temper-playground-world::suite walk::story_a_kept_log_replays_natively
- test temper-playground-world::suite walk::story_a_walks_from_the_welcome_to_the_csv_change_landed
- test temper-real-loop-world::suite lock::two_acts_on_one_directory_take_its_lock_in_turn
- test skein-conformance-real::suite deadline::an_idle_listener_wait_fails_with_its_figures
- test temper-real-loop-world world::diagnostics::tests::forced_overdue_observes_real_tasks_operations_and_dispatch
- test temper-box::suite box_listeners::a_box_without_openssl_logs_sign_in_unavailable_once_and_stays_up
- test temper-box::suite box_listeners::a_later_box_refuses_a_taken_port_naming_the_port_and_document
- test temper-box::suite support_tests::github_file_scan_excludes_only_jigs_generation_secrets_store
- lint jig-protocol-store clippy::manual_let_else jig/crates/jig-protocol-store/src/../patches/p0001.rs
- lint jig-protocol-store unfulfilled_lint_expectations jig/crates/jig-protocol-store/src/../patches/context.rs
- lint jig-protocol-store clippy::disallowed_types jig/crates/jig-protocol-store/src/tests/catch_up.rs
- lint jig-protocol-store clippy::disallowed_methods jig/crates/jig-protocol-store/src/tests/catch_up.rs
- lint jig-protocol-store clippy::disallowed_macros jig/crates/jig-protocol-store/src/tests/catch_up.rs
- lint jig-protocol-store clippy::needless_borrow jig/crates/jig-protocol-store/src/tests/policy_patch.rs

Inherited from main, raised by change 1766 on 2026-11-12.

- lint temper-service clippy::assigning_clones temper/crates/temper-service/src/engine.rs

- test smith-bench::suite pipeline::grading_and_archives_keep_agent_commits_and_untracked_files_outside_the_timer
- test smith-bench::suite runner::a_recorded_answer_is_observed_before_the_tree_settles
- test smith-bench::suite runner::an_overflowing_stderr_retains_its_prefix_and_settles_the_tree
- lint temper-settings-form clippy::too_many_lines temper/crates/temper-settings-form/src/kinds.rs
- lint temper-engine-domain clippy::too_many_lines temper/crates/temper-engine-domain/src/route/applications.rs
- lint temper-engine-domain clippy::useless_conversion temper/crates/temper-engine-domain/src/route/kinds.rs
- lint skein-lib clippy::chunks_exact_to_as_chunks skein/crates/skein-lib/src/crypto_rng.rs

- lint temper-client-view clippy::too_many_lines temper/crates/temper-client-view/src/tests/toast.rs
- test temper-client-journeys-tool tests::the_overlay_is_empty
- lint skein-codec clippy::chunks_exact_to_as_chunks skein/crates/skein-codec/src/patch/plan.rs
- lint skein-http clippy::question_mark skein/crates/skein-http/src/server.rs
- lint jig-client-domain clippy::byte_char_slices jig/crates/jig-client-domain/src/contract/accounts.rs
- test temper-box::suite forge_land::setup_accepts_a_project_free_first_repository_row
- test temper-box::suite forge_land::terminal_login_cleanup_excludes_a_restart_import
- test temper-box::suite forge_land::a_held_login_returns_when_validation_panics_between_boxes
- test temper-box::suite startup::serve_checks_the_owned_agents_admission_at_each_bound
- test temper-box::suite startup::a_failed_terminal_return_allows_recovery_without_reopening_import
- test temper-box::suite startup::a_held_record_at_its_path_owes_the_guard_nothing
- test temper-box::suite startup::a_lent_record_not_back_keeps_its_store_and_the_storys_panic
- test temper-box::suite support_tests::printed_links_count_and_fail_without_disclosing_secrets
