+++
id = "widen-kit"
part = "skein-widening"
kind = "migration"
needs = []
files = ["skein/crates/skein-lib/src/**", "skein/crates/skein-codec/src/**", "skein/crates/skein-json/src/**", "skein/crates/skein-kv/src/**", "skein/crates/skein-http/src/**", "skein/crates/skein-tls/src/**", "skein/crates/skein-channel/src/**", "skein/crates/skein-link/src/**", "skein/crates/skein-shell/src/**", "skein/crates/skein-sim/src/**", "skein/crates/skein-sim-world/src/**"]
effort = "medium"
minutes = 55
+++

# skein: `skein-lib`, `skein-codec`, `skein-json`, `skein-kv`, `skein-http`, `skein-tls`, `skein-channel`, `skein-link`, `skein-shell`, `skein-sim`, `skein-sim-world` widen through lib's helpers

## Goal

These crates write 137 widening conversions by hand at this plan's base. Each `u32` widens with `usize_of` and each native length with `len_u64`, following the note, so nothing but a real `u64` conversion stays. Behaviour doesn't change, and production lines don't rise.

Acceptance:

- The note's two searches on `skein/crates/skein-lib`, `skein/crates/skein-codec`, `skein/crates/skein-json`, `skein/crates/skein-kv`, `skein/crates/skein-http`, `skein/crates/skein-tls`, `skein/crates/skein-channel`, `skein/crates/skein-link`, `skein/crates/skein-shell`, `skein/crates/skein-sim`, `skein/crates/skein-sim-world` list only what your report names as staying.
- `heavy cargo clippy` and `heavy cargo nextest run` pass on these packages, with what they asserted unchanged.
- The branch's net production lines are at most 0.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `temper/docs/plan/debt-20/notes/widening.md`: the rule, what stays, the searches, the run
- `skein/docs/development/debt/D002-index-and-length-widening.md`
- `skein/docs/design/lib.md`, 6

## Owns and may change

- **Owns:** the files in `files`.
- **Another crate's site stays:** other `widen-*` tasks own the rest of skein. Don't delete the item: `widen-close` does.
- **`skein-tls`** is also checked alone (`heavy cargo check -p skein-tls`), since its features differ standalone (`skein/AGENTS.md`).
- **`skein-codec`'s schema compiler:** a string that prints generated code stays. Its own parsing and layout code moves.
- **`skein-lib`:** `crate::usize_of`, `crate::len_u64` or the `length` module's path, as the file reads best.

## Steps

1. Replace, crate by crate, a commit per crate or a few together: the compiler sorts the types (the note).
2. `-p skein-lib -p skein-codec -p skein-json -p skein-kv -p skein-http -p skein-tls -p skein-channel -p skein-link -p skein-shell -p skein-sim -p skein-sim-world` is what the note's run names. Commit and push.

## Report

The note's items.
