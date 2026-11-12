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

Inherited from main (merge 725c605915), found by change 2116 of goal 2105 on 2026-11-12.

- lint jig-protocol-store clippy::disallowed_methods jig/crates/jig-protocol-store/src/lib.rs
- lint smith-protocol-program clippy::items_after_test_module smith/crates/smith-protocol-program/src/mcp.rs
- lint fabrig-boundary clippy::assert_is_empty tests/boundary/tests/codecs.rs

Inherited from main (commit 0e504506c7, merged by hand in 6d3aa97b7d), found by change 2117 of goal 2105 on 2026-11-12.

- lint temper-engine-domain-forge clippy::too_many_lines temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge clippy::needless_borrow temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge clippy::needless_bool temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge clippy::disallowed_methods temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge unfulfilled_lint_expectations temper/crates/temper-engine-domain-forge/src/boundary.rs
- lint temper-engine-domain-forge unfulfilled_lint_expectations temper/crates/temper-engine-domain-forge/src/landing.rs

Inherited from main (commit fab52fee98, merged in 837703982a), found by change 2121 of goal 2105 on 2026-11-12.

- lint temper-worker-domain clippy::chunks_exact_to_as_chunks temper/crates/temper-worker-domain/src/jobs/prepare.rs
