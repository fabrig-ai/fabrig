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
