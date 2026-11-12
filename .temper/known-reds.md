# Known reds

Failing on the goal branch, inherited from main, owned by no live change.

The test below fails under load (it asserts settling within 180 ms of wall
clock) and passes alone. The person listed it on 2026-11-12. A separate
health goal makes it hold under load and removes this entry when it lands.

- test smith-bench::suite runner::an_overflowing_stderr_retains_its_prefix_and_settles_the_tree

The entries below come with main at 37bc4d2488, which pins the dated nightly
toolchain (rust-toolchain.toml), and reach the goal branch through change
1737's merge of main, 2026-11-12. The first group is main's own known-reds
list at that commit, still standing there. The second group is what nightly
clippy flags on main and no list names; change 1737 reported it.

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
- test smith-bench::suite pipeline::grading_and_archives_keep_agent_commits_and_untracked_files_outside_the_timer
- test smith-bench::suite runner::a_recorded_answer_is_observed_before_the_tree_settles
- lint temper-settings-form clippy::too_many_lines temper/crates/temper-settings-form/src/kinds.rs
- lint temper-engine-domain clippy::too_many_lines temper/crates/temper-engine-domain/src/route/applications.rs
- lint temper-engine-domain clippy::useless_conversion temper/crates/temper-engine-domain/src/route/kinds.rs
- lint skein-lib clippy::chunks_exact_to_as_chunks skein/crates/skein-lib/src/crypto_rng.rs

- lint temper-engine-domain-forge clippy::too_many_lines temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge clippy::needless_borrow temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge clippy::needless_bool temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge clippy::disallowed_methods temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge unfulfilled_lint_expectations temper/crates/temper-engine-domain-forge/src/landing.rs
- lint temper-engine-domain-forge unfulfilled_lint_expectations temper/crates/temper-engine-domain-forge/src/boundary.rs
- lint jig-protocol-store clippy::too_many_lines jig/crates/jig-protocol-store/src/tests/phase.rs

Main at 37bc4d2488 (and still at f6ffc08436) moves the client wire only
partly: jig-client-protocol-wire's VERSION is 20 and its schema file is
client-v20.schema, but that file's header and the generated code still say
jig-client-wire 19. The goal's version-constant test catches the mismatch, and
it reaches the goal branch through change 1737's merge of main, 2026-11-12.
The entry goes once main's header and VERSION agree.

- test jig-client-protocol-wire tests::versions::version_constants_equal_the_family_headers
