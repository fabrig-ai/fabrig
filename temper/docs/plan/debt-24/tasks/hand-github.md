+++
id = "hand-github"
part = "connection-worlds"
kind = "migration"
needs = ["hand-face"]
files = ["skein/tests/forge/machine/tests/suite/hand/github.rs", "skein/tests/forge/machine/tests/suite/hand/github/**", "skein/tests/forge/machine/src/hand.rs", "skein/crates/skein-forge-connection/src/tests.rs", "skein/crates/skein-forge-connection/src/tests/github_start.rs", "skein/crates/skein-forge-connection/src/tests/github_installations.rs", "skein/crates/skein-forge-connection/src/tests/runtime.rs"]
effort = "medium"
minutes = 25
+++

# The public GitHub tests move to the machine world

## Goal

Skein D030's public GitHub batch: `github_start`, `github_installations`, `runtime`, about 73 tests. By `temper/docs/plan/debt-24/notes/hand.md`.

Acceptance:

- Their `Harness` tests are in `tests/suite/hand/github/`, under the same file names. What isn't on `Harness` stays in the crate. `github_start`'s helpers that the crate's other GitHub files read stay there too, with a copy in `hand.rs`, as the note says.
- Each test's asserts read as they did.
- `heavy cargo nextest run -p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass; the world gains what the crate loses.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/hand.md`
- `skein/docs/design/forge-connection.md`, 10

## Owns and may change

- **Owns:** the files in the header. `hand.rs` only gains what its files read, at its end: `hand-forgejo` holds it at once, and fab merges the two.
- **A test that reads a private item** isn't in this batch: it stays, and `hand-rows` takes it. Say which.

## Steps

The script from `hand-face`'s report; `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-machine-world -p skein-forge-connection --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the assert comparison; the time. A move: no rule to break by hand.
