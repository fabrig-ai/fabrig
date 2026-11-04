+++
id = "forgejo-timeouts"
part = "fake-forgejo"
kind = "feature"
needs = []
files = ["skein/crates/skein-fake-forge-protocol/src/server.rs", "skein/crates/skein-fake-forge-protocol/src/tests/http.rs"]
effort = "medium"
minutes = 15
+++

# The fake Forgejo answers a timeout `500`

## Goal

`skein-fake-forge-protocol`'s `server.rs`, `down`, closes the connection on half the domain's timeouts,
`service.rng.chance(500)`, an undesigned draw. `skein/docs/design/fake-forge.md`, 4 says one timed out is
`500`, and leaves closing to the `reset` and `drop` faults. (Bug 14's other half, re-checked: holds.)

Acceptance:

- That branch is gone; a domain timeout answers `500` on every seed. No type or signature changes.
- The changed test fails on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/fake-forge.md`, 2.3, 4.

## Owns and may change

- Owns: `src/server.rs` (`down`), `src/tests/http.rs`.
- The test pins the old behaviour: `a_reset_closes_before_reading_and_a_timeout_drops_or_answers_500`. Rename
  it `a_reset_closes_before_reading_and_a_timeout_answers_500`; its last loop's rows expect `closed = false,
  Some(500)` for every seed.
- Other non-zero `timeouts` over HTTP are `tests/git.rs`'s, a path with no such draw (surveyed).
- `service.rng`'s draws shift only where a domain timeout reaches the HTTP layer.

## Steps

1. Change the test; run it on the old code and quote the failure at seed 0.
2. Delete the branch.
3. `heavy cargo nextest run -p skein-fake-forge-protocol`, then the focused tests of `skein-fake-forge-tests`,
   `skein-forge-protocol-world`, `forgejo-conformance`. `cargo fmt`, `python3 -I -B
   skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-fake-forge-protocol --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the test showed on the old code.
