# Known reds

Failures on the goal branch that no live change of goal 1706 owns; delivery
checks count them as inherited (`tools/pre-pr/design.md`, section 5).

- lint jig-protocol-store clippy::manual_let_else jig/crates/jig-protocol-store/patches/p0001.rs
- lint jig-protocol-store unfulfilled_lint_expectations jig/crates/jig-protocol-store/patches/context.rs
- lint skein-lib clippy::chunks_exact_to_as_chunks skein/crates/skein-lib/src/crypto_rng.rs

Inherited from main at the goal's start, 2026-11-12.

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
- test temper-client-journeys-tool tests::the_overlay_is_empty
- lint temper-engine-domain clippy::too_many_lines temper/crates/temper-engine-domain/src/route/applications.rs
- lint temper-engine-domain clippy::useless_conversion temper/crates/temper-engine-domain/src/route/kinds.rs
