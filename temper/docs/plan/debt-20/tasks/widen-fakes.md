+++
id = "widen-fakes"
part = "skein-widening"
kind = "migration"
needs = []
files = ["skein/crates/skein-fake-peers/src/**", "skein/crates/skein-fake-forge-github/src/**", "skein/crates/skein-fake-forge-forgejo/src/**", "skein/crates/skein-fake-forge-domain/src/**", "skein/crates/skein-fake-forge-http/src/**", "skein/crates/skein-fake-machine/src/**", "skein/crates/skein-fake-checkout/src/**"]
effort = "medium"
minutes = 40
+++

# skein: `skein-fake-peers`, `skein-fake-forge-github`, `skein-fake-forge-forgejo`, `skein-fake-forge-domain`, `skein-fake-forge-http`, `skein-fake-machine`, `skein-fake-checkout` widen through lib's helpers

## Goal

These crates write 103 widening conversions by hand at this plan's base. Each `u32` widens with `usize_of` and each native length with `len_u64`, following the note, so nothing but a real `u64` conversion stays. Behaviour doesn't change, and production lines don't rise.

Acceptance:

- The note's two searches on `skein/crates/skein-fake-peers`, `skein/crates/skein-fake-forge-github`, `skein/crates/skein-fake-forge-forgejo`, `skein/crates/skein-fake-forge-domain`, `skein/crates/skein-fake-forge-http`, `skein/crates/skein-fake-machine`, `skein/crates/skein-fake-checkout` list only what your report names as staying.
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
- **`skein-fake-checkout` has no `skein-lib` dependency:** its two sites stay and your report lists them.

## Steps

1. Replace, crate by crate, a commit per crate or a few together: the compiler sorts the types (the note).
2. `-p skein-fake-peers -p skein-fake-forge-github -p skein-fake-forge-forgejo -p skein-fake-forge-domain -p skein-fake-forge-http -p skein-fake-machine -p skein-fake-checkout` is what the note's run names. Commit and push.

## Report

The note's items.
