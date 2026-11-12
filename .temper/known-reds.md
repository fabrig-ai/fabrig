# Known reds

Failing on the goal branch, inherited from main, owned by no live change.
The patches' entries are given twice: as the tree names the file, and as
clippy's diagnostics name it (`src/../patches/`, through `src/lib.rs`'s
`#[path]`).

- lint jig-protocol-store clippy::manual_let_else jig/crates/jig-protocol-store/patches/p0001.rs
- lint jig-protocol-store unfulfilled_lint_expectations jig/crates/jig-protocol-store/patches/context.rs
- lint jig-protocol-store clippy::manual_let_else jig/crates/jig-protocol-store/src/../patches/p0001.rs
- lint jig-protocol-store unfulfilled_lint_expectations jig/crates/jig-protocol-store/src/../patches/context.rs
- lint jig-protocol-store clippy::disallowed_types jig/crates/jig-protocol-store/src/tests/catch_up.rs
- lint jig-protocol-store clippy::disallowed_methods jig/crates/jig-protocol-store/src/tests/catch_up.rs
- lint jig-protocol-store clippy::disallowed_macros jig/crates/jig-protocol-store/src/tests/catch_up.rs
- lint jig-protocol-store clippy::needless_borrow jig/crates/jig-protocol-store/src/tests/policy_patch.rs

The test below fails under load (it asserts settling within 180 ms of wall
clock) and passes alone. The person listed it on 2026-11-12. A separate
health goal makes it hold under load and removes this entry when it lands.

- test smith-bench::suite runner::an_overflowing_stderr_retains_its_prefix_and_settles_the_tree

The test below fails because the host tools' contract test names an
undeclared variable, `TEMPER_CONTRACT_WRITE` (from the goal's own change
1738). Change 1954 renames it to `SKEIN_CODEGEN_WRITE` and removes this entry.

- test fabrig-boundary::cli_names every_name_is_declared
