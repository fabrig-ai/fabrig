# Known reds

Failures on goal 1705's branch that no live change owns; delivery checks
count them as inherited (`tools/pre-pr/design.md`, section 5). Each is
dropped once fixed.

From main at 54bd5ff619, merged into the goal's branch on 2026-11-13:

- lint skein-lib clippy::chunks_exact_to_as_chunks skein/crates/skein-lib/src/crypto_rng.rs
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
- lint temper-service clippy::assigning_clones temper/crates/temper-service/src/engine.rs
- test smith-bench::suite pipeline::grading_and_archives_keep_agent_commits_and_untracked_files_outside_the_timer
- test smith-bench::suite runner::a_recorded_answer_is_observed_before_the_tree_settles
- test smith-bench::suite runner::an_overflowing_stderr_retains_its_prefix_and_settles_the_tree
- lint temper-settings-form clippy::too_many_lines temper/crates/temper-settings-form/src/kinds.rs
- lint temper-engine-domain clippy::too_many_lines temper/crates/temper-engine-domain/src/route/applications.rs
- lint temper-engine-domain clippy::useless_conversion temper/crates/temper-engine-domain/src/route/kinds.rs
- lint temper-client-view clippy::too_many_lines temper/crates/temper-client-view/src/tests/toast.rs
- test temper-client-journeys-tool tests::the_overlay_is_empty
- lint jig-protocol-config clippy::disallowed_methods jig/crates/jig-protocol-config/src/tests/seed_policy.rs
- lint skein-codec clippy::chunks_exact_to_as_chunks skein/crates/skein-codec/src/patch/plan.rs
- lint jig-domain-fleet clippy::unneeded_wildcard_pattern jig/crates/jig-domain-fleet/src/attempt.rs
- lint jig-domain-fleet clippy::needless_bool jig/crates/jig-domain-fleet/src/sources.rs
- lint jig-domain-accounts clippy::needless_bool jig/crates/jig-domain-accounts/src/limits.rs

Raised by the dated nightly in a file of main's that the goal does not touch, and so inherited:

- lint fabrig-boundary clippy::assert_is_empty tests/boundary/tests/shared_test_code.rs

From the goal's own list before that merge, and not on main's:

- lint temper-engine-domain-forge clippy::too_many_lines temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge clippy::needless_borrow temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge clippy::needless_bool temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge clippy::disallowed_methods temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge unfulfilled_lint_expectations temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge unfulfilled_lint_expectations temper/crates/temper-engine-domain-forge/src/boundary.rs
- lint jig-protocol-store clippy::too_many_lines jig/crates/jig-protocol-store/src/tests/phase.rs
