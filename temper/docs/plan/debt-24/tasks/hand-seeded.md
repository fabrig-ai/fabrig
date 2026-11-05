+++
id = "hand-seeded"
part = "connection-worlds"
kind = "migration"
needs = ["hand-rows"]
files = ["skein/tests/forge/machine/tests/suite/hand/github.rs", "skein/tests/forge/machine/tests/suite/hand/github/**", "skein/tests/forge/machine/tests/suite/hand/forgejo/**", "skein/tests/forge/machine/src/hand.rs", "skein/tests/forge/machine/src/github.rs", "skein/crates/skein-forge-connection/src/tests.rs", "skein/crates/skein-forge-connection/src/tests/github_calls.rs", "skein/crates/skein-forge-connection/src/tests/github_unknown_repository.rs", "skein/crates/skein-forge-connection/src/tests/github_logs.rs", "skein/crates/skein-forge-connection/src/tests/github_files.rs", "skein/crates/skein-forge-connection/src/tests/github_files/fake.rs", "skein/crates/skein-forge-connection/src/tests/github_writes_heap.rs", "skein/crates/skein-forge-connection/src/tests/free_rate.rs"]
effort = "high"
minutes = 40
+++

# The tests that seed private rows move, and the crate's fake goes

## Goal

Skein D030's seed batch: `github_calls`, `github_unknown_repository`, `github_logs`, `github_files`, `github_writes_heap`, and `free_rate`'s last, about 106 tests. A seed helper sets installation rows without a start, on purpose, to skip startup's RSA work, and stays what it is: through the seam, not a real start, since a real start would change `connections_opened` and each test's cost. By `temper/docs/plan/debt-24/notes/hand.md`.

Acceptance:

- Their `Harness` tests are in `tests/suite/hand/`. What isn't on `Harness` stays in the crate.
- `github_files/fake.rs` is gone: its `Fake` merges into the machine world's `github::Fake`, or `github_files`' seven tests run on that one. If the two can't meet without changing what a test asserts, keep its answers beside the seven as a small local helper on `Inline`, with no HTTP of its own, and say why.
- Each test's asserts read as they did; `github_writes_heap`'s ceilings hold, or the common note's rule on a pin applies.
- `heavy cargo nextest run -p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass; the world gains what the crate loses.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/notes/hand.md`
- `skein/docs/design/forge-connection.md`, 10
- `skein/docs/design/fake-forge.md`, 10.6

## Owns and may change

- **Owns:** the files in the header. `src/component/seam.rs` and `src/lib.rs` may gain an accessor's re-export a test needs, and nothing more.
- **`github::Fake` is a shared face** of `joint` and `tests/suite`: an addition is fine, and what it does for them stays.

## Steps

The script from `hand-face`'s report; the fake; `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-machine-world -p skein-forge-connection --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the assert comparison; the time; how `github_files` came out. A move: no rule to break by hand.
