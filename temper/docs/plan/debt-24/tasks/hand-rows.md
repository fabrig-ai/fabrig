+++
id = "hand-rows"
part = "connection-worlds"
kind = "migration"
needs = ["connection-seam"]
files = ["skein/tests/forge/machine/tests/suite/hand/github.rs", "skein/tests/forge/machine/tests/suite/hand/github/**", "skein/tests/forge/machine/tests/suite/hand/forgejo.rs", "skein/tests/forge/machine/tests/suite/hand/forgejo/**", "skein/tests/forge/machine/src/hand.rs", "skein/crates/skein-forge-connection/src/tests.rs", "skein/crates/skein-forge-connection/src/tests/github_installation_loss.rs", "skein/crates/skein-forge-connection/src/tests/github_renewal.rs", "skein/crates/skein-forge-connection/src/tests/puller.rs", "skein/crates/skein-forge-connection/src/tests/reads.rs", "skein/crates/skein-forge-connection/src/tests/connection.rs"]
effort = "medium"
minutes = 30
+++

# The tests that read private rows move to the machine world

## Goal

Skein D030's read batch: `github_installation_loss`, `github_renewal`, `puller`, and the four `drv` tests `hand-forgejo` left, about 137 tests. They read through `skein_forge_connection::seam`. By `temper/docs/plan/debt-24/notes/hand.md`.

Acceptance:

- Their `Harness` tests are in `tests/suite/hand/`. What isn't on `Harness` stays in the crate; `github_renewal`'s helpers that `github_unknown_repository` reads stay too, with a copy in `hand.rs`, until `hand-seeded`.
- Each test's asserts read as they did; `puller`'s ceilings hold, or the common note's rule on a pin applies.
- `heavy cargo nextest run -p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass; the world gains what the crate loses.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/hand.md`
- `skein/docs/design/forge-connection.md`, 10

## Owns and may change

- **Owns:** the files in the header. `skein/crates/skein-forge-connection/src/component/seam.rs` and `src/lib.rs` may gain an accessor's re-export a test turns out to need, and nothing more.

## Steps

The script from `hand-face`'s report, with `.test_github` reached through the seam's import; `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-machine-world -p skein-forge-connection --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the assert comparison; the time. A move: no rule to break by hand.
