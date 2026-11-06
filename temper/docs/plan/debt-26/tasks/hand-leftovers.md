+++
id = "hand-leftovers"
part = "hand-leftovers"
kind = "migration"
needs = []
files = ["skein/tests/forge/machine/src/hand.rs", "skein/tests/forge/machine/src/hand/**", "skein/tests/forge/machine/tests/suite/hand/github/runtime.rs", "skein/tests/forge/machine/tests/suite/hand/forgejo/reads.rs", "skein/crates/skein-forge-connection/src/component/tests.rs"]
effort = "medium"
minutes = 40
+++

# The hand harness: split by dialect, and its small copies gone

## Goal

`src/hand.rs` runs 784 lines with a `use` in mid-file and two `impl Harness`, and it has small copies (the spot review of debt-24, finding 4). Split it to match `tests/suite/hand/{forgejo,github}`, and drop the copies. This moves code and pins no new rule: no test is added.

Acceptance:

- `skein/tests/forge/machine/src/hand.rs` holds the harness; `src/hand/forgejo.rs` and `src/hand/github.rs` hold each dialect's fixtures. No `use` sits in mid-file, and one `impl Harness` stays in each file at most.
- `skein_forge_machine_world::hand::...` keeps every path the suites read: `hand.rs` re-exports, so that suites change only where this spec says.
- `ok_dated` is gone: `tests/suite/hand/forgejo/reads.rs`, its one caller, calls `ok`.
- `tests/suite/hand/github/runtime.rs` holds one alias of `skein_forge_machine_world::hand`, not two, and its uses read it.
- `renewed_full` and `renewed_contents` are one function, in `skein-forge-connection/src/component/tests.rs`, and in `hand` if its pair is identical too.
- `skein-forge-machine-world`'s and `skein-forge-connection`'s lib tests pass with their count unchanged. Lines don't grow but for the split's `use` and `mod` lines: report the count.

## Inputs

- `skein/docs/design/forge-connection.md`, 10, *By hand*.
- `temper/docs/plan/debt-26/inputs/debt24-review.md`, first review, finding 4.
- `temper/docs/plan/debt-26/notes/common.md`.

## Owns and may change

Owns the files in `files`. In `component/tests.rs`, only `renewed_full`, `renewed_contents` and their callers: `forge-until` and `seam-shapes` hold the rest of that crate. `forge-fixtures` follows you and takes `hand`'s shared items into a module: don't start that here.

Options, each pre-approved:

- **What's a dialect's:** an item only `suite/hand/github` reads goes to `github.rs`; only `suite/hand/forgejo` to `forgejo.rs`; both, or the harness itself, stay in `hand.rs`. Read by both with a dialect's name (`github_settings`): the `github` file. A `pub use forgejo::*; pub use github::*;` at the top of `hand.rs` is fine in this world.
- If a call to a test helper at the end of `hand.rs` needs a private field, make it `pub(super)` or move the helper to `impl Harness`.

## Steps

Move, then `heavy cargo check -p skein-forge-machine-world --tests` until it's clean. `heavy cargo nextest run -p skein-forge-machine-world`, compare the count with your base's, then `heavy cargo nextest run -p skein-forge-connection --lib`. fmt, and `heavy cargo clippy -p skein-forge-machine-world -p skein-forge-connection --all-targets -- -D warnings`. Commit, push.

## Report

As your text says: lines before and after in `src/hand*` and the test count on both sides.
