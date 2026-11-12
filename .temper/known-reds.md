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
