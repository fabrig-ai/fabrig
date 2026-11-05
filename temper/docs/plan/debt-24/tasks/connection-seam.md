+++
id = "connection-seam"
part = "connection-worlds"
kind = "interface"
needs = ["hand-github", "hand-forgejo"]
files = ["skein/crates/skein-forge-connection/Cargo.toml", "skein/crates/skein-forge-connection/src/lib.rs", "skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/seam.rs", "skein/crates/skein-forge-connection/src/component/tests.rs", "skein/tests/forge/machine/Cargo.toml", "Cargo.lock"]
effort = "high"
minutes = 35
+++

# skein-forge-connection: the `seam` feature

## Goal

The machine world reads the component's private installation rows, jobs and connections, so 217 tests can move with their asserts unchanged. `skein/docs/design/forge-connection.md`, 10 says what the seam is and who turns it on. See `temper/docs/plan/debt-24/notes/hand.md`, "The seam".

Acceptance:

- `skein-forge-connection/Cargo.toml` has `[features] seam = []`, and `default` doesn't hold it. `skein-forge-machine-world`'s line turns it on, and no other package's does: `rg -n 'seam' -g Cargo.toml .` reads those two manifests.
- The `test_*` accessors on `Component` move from `component/tests.rs` to `component/seam.rs`, under `#[cfg(any(test, feature = "seam"))]`. What they return and the items the note lists are reachable from the world under the feature: `#[cfg(feature = "seam")] pub mod seam` re-exporting them from `lib.rs` is the default.
- A build without the feature has the crate's public surface unchanged. `heavy cargo check -p skein-forge-connection` and `heavy cargo check -p temper-service` pass, and `heavy cargo tree -p temper -e features -i skein-forge-connection` (no build, `cargo tree` runs directly) doesn't show `seam`.
- `cfg(test)` code the moved tests lean on (`grow.rs`, `connection.rs`, `hooks/connection.rs`): read each; what a moved test reads goes under the same `cfg`, and what it doesn't stays. No production behaviour differs under the feature.
- `heavy cargo nextest run -p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass, with no assertion changed.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/hand.md`
- `skein/docs/design/forge-connection.md`, 10

## Owns and may change

- **Owns:** the files in the header. Other `skein-forge-connection/src/**` only for a `pub(crate)` that must become `pub` under the seam's re-export, or a `cfg` line.
- **An item public with no `seam`** is a breaking question the other way: nothing that isn't public today becomes public in a default build. A module may need `pub` with its re-export gated.
- **The step subset holds** in `seam.rs`: it's step code, compiled in no service.
- **`--all-features`** clippy is run once for this crate too.

## Steps

1. The feature; `seam.rs`; the re-export; a world line that reads one item, as the proof.
2. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings` and the same with `--all-features`; the tests above. Commit and push.

## Report

The common note's items, what the seam exposes. An addition with no rule of its own: nothing to break by hand.
