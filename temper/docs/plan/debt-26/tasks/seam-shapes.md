+++
id = "seam-shapes"
part = "seam-shapes"
kind = "feature"
needs = ["forge-until"]
files = ["skein/crates/skein-forge-connection/src/lib.rs", "skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/entry.rs", "skein/crates/skein-forge-connection/src/component/seam.rs", "skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/hooks/connection.rs", "skein/crates/skein-forge-connection/src/installations.rs", "skein/crates/skein-forge-connection/src/puller.rs", "tests/boundary/tests/skein_worlds.rs"]
effort = "high"
minutes = 50
+++

# The seam's shapes: fewer `pub`, one attribute, no unread names, and a gate

## Goal

The seam left shapes behind it (the spot review of debt-24, finding 3): a wrapper that likely dodges private-in-public, eight copies of one attribute, names nobody reads, and a design rule no gate holds. The cut has checked each against the trunk; the first two need a compile.

Acceptance:

- `component.rs`'s `mod own` is gone, with `OwnPlan` a plain `pub(crate)` enum; `GitHubForge`, `ForgeEntry`, `Base` and `Dialect` are `pub(crate)`, as are their fields where nothing outside reads them.
- `rg -n 'cfg_attr' skein/crates/skein-forge-connection/src` prints `lib.rs`'s alone, beside its `no_std` line: one crate-level attribute takes the eight `cfg_attr(feature = "seam", expect(...))` blocks.
- `component/seam.rs`'s `installation_bounds` and its `#[cfg(feature = "seam")] use` are gone, and `limits::installation_bounds` stays if something else reads it.
- `lib.rs`'s `pub mod seam` names only what `skein/tests/forge/machine` reads by name. On the trunk that drops `Base`, `Dialect`, `ForgeEntry`, `GitHubForge`, `installation_bounds`, `Connection`, `Hooked`, `Metadata`, `Resume`, `Work` and `Puller`: let the world's build settle each.
- `tests/boundary/tests/skein_worlds.rs` fails when a package other than `skein-forge-machine-world` turns on `skein-forge-connection`'s `seam` feature, and you've watched it fail.
- `skein-forge-connection`'s lib tests and `skein-forge-machine-world`'s pass unchanged; clippy is clean on the crate with and without `--features seam`. Production falls by about 45 lines: report the count.

## Inputs

- `skein/docs/design/forge-connection.md`, 10, *The seam*: "which only that world turns on".
- `skein/docs/design/testing.md`, 6, *A world's library is its own*.
- `temper/docs/plan/debt-26/inputs/debt24-review.md`, first review, finding 3.
- `temper/docs/plan/debt-26/notes/common.md`.

## Owns and may change

Owns the files in `files`. Reader files, call sites only: `skein/tests/forge/machine/src/**` and `tests/suite/**`, where a dropped name was read through another path; `hand-leftovers` is splitting `src/hand.rs` beside you, so touch it only if the build makes you.

Options, each pre-approved:

- If a type must stay `pub` for a `test_*` method's signature (`test_last_connection` returns `&mut Connection`), it stays `pub` in its private module, and just drops out of the `pub use`.
- If a world's test reads a name you'd drop, that name stays.
- **The crate-level attribute:** `#![cfg_attr(feature = "seam", expect(clippy::must_use_candidate, reason = "the machine-world seam exposes the existing state operations"))]`. Add `clippy::new_without_default` only if `Puller`'s `new` still fires once `Puller` leaves the `pub use`. If an `expect` goes unfulfilled in the test target, write `all(feature = "seam", not(test))`.
- If reverting a type to `pub(crate)` needs `mod own` after all, because of a `private_interfaces` the compile shows you, keep that wrapper, and say what fired in the report.

## Steps

The gate, built like `world_edges` beside it, so that its rows are a table over synthetic metadata as in `dependency_paths_hold_for_every_kind_and_tree`:

```rust
struct Dependency {
    name: String,
    directory: PathBuf,
    dev: bool,
    features: Vec<String>,
}

/// The seam's one world (skein/docs/design/forge-connection.md, 10).
fn seam_edges(packages: &[Package]) -> Vec<String>;
```

`skein_worlds_are_private_to_their_packages` extends its failures with `seam_edges(&packages)`. Its table: the world turning it on passes; another package turning it on fails, for a normal and a dev edge; another package depending on the crate with no feature passes. Three rows or four. To see it bite against the real tree, add `features = ["seam"]` to one other package's dependency by hand, run the test, read the failure, and restore.

Compile first with `heavy cargo check -p skein-forge-machine-world --tests`, as each removal is what proves it. Then: `heavy cargo nextest run -p skein-forge-connection --lib`; `heavy cargo nextest run -p skein-forge-machine-world`; `heavy cargo nextest run -p fabrig-boundary --test skein_worlds`; fmt; `python3 -I -B scripts/old_forms.py` from `skein/`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`, the same with `--features seam`, and `heavy cargo clippy -p skein-forge-machine-world -p fabrig-boundary --all-targets -- -D warnings`. Commit, push.

## Report

As your text says: production lines before and after, the names dropped and kept, whether the wrapper could go, and the gate's failure on your hand break.
