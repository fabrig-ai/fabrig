+++
id = "widen-oauth"
part = "skein-widening"
kind = "migration"
needs = []
files = ["skein/crates/skein-oauth/src/**", "skein/crates/skein-oauth-accounts/src/**", "skein/crates/skein-oauth-issuer/src/**", "skein/crates/skein-fake-oauth/src/**"]
effort = "medium"
minutes = 30
+++

# skein: `skein-oauth`, `skein-oauth-accounts`, `skein-oauth-issuer`, `skein-fake-oauth` widen through lib's helpers

## Goal

These crates write 71 widening conversions by hand at this plan's base. Each `u32` widens with `usize_of` and each native length with `len_u64`, following the note, so nothing but a real `u64` conversion stays. Behaviour doesn't change, and production lines don't rise.

Acceptance:

- The note's two searches on `skein/crates/skein-oauth`, `skein/crates/skein-oauth-accounts`, `skein/crates/skein-oauth-issuer`, `skein/crates/skein-fake-oauth` list only what your report names as staying.
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
- **`skein-oauth/src/generated/v3.rs` stays** (9 sites).

## Steps

1. Replace, crate by crate, a commit per crate or a few together: the compiler sorts the types (the note).
2. `-p skein-oauth -p skein-oauth-accounts -p skein-oauth-issuer -p skein-fake-oauth` is what the note's run names. Commit and push.

## Report

The note's items.
