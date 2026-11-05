+++
id = "widen-forge"
part = "skein-widening"
kind = "migration"
needs = []
files = ["skein/crates/skein-forge/src/**", "skein/crates/skein-forge-client/src/**", "skein/crates/skein-forge-setup/src/**", "skein/crates/skein-forge-connection/src/**"]
effort = "medium"
minutes = 45
+++

# skein: `skein-forge`, `skein-forge-client`, `skein-forge-setup`, `skein-forge-connection` widen through lib's helpers

## Goal

These crates write 115 widening conversions by hand at this plan's base. Each `u32` widens with `usize_of` and each native length with `len_u64`, following the note, so nothing but a real `u64` conversion stays. Behaviour doesn't change, and production lines don't rise.

Acceptance:

- The note's two searches on `skein/crates/skein-forge`, `skein/crates/skein-forge-client`, `skein/crates/skein-forge-setup`, `skein/crates/skein-forge-connection` list only what your report names as staying.
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

## Steps

1. Replace, crate by crate, a commit per crate or a few together: the compiler sorts the types (the note).
2. `-p skein-forge -p skein-forge-client -p skein-forge-setup -p skein-forge-connection` is what the note's run names. Commit and push.

## Report

The note's items.
