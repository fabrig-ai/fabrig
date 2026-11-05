+++
id = "widen-io"
part = "skein-widening"
kind = "migration"
needs = ["io-file-limits"]
files = ["skein/crates/skein-io/src/**"]
effort = "medium"
minutes = 15
+++

# skein: `skein-io` widens through lib's helpers

## Goal

`skein-io` writes 27 widening conversions by hand at this plan's base; `io-signals`, `io-pipe`, `io-file-states` and `io-file-limits` have just rewritten the files they sit in, so re-measure. Each `u32` widens with `usize_of` and each native length with `len_u64`, following the note. Behaviour doesn't change.

Acceptance:

- The note's two searches on `skein/crates/skein-io` list only what your report names as staying.
- `heavy cargo clippy` and `heavy cargo nextest run -p skein-io` pass, with what they asserted unchanged.
- The branch's net production lines are at most 0.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `temper/docs/plan/debt-20/notes/widening.md`: the rule, what stays, the searches, the run
- `skein/docs/development/debt/D002-index-and-length-widening.md`
- `skein/docs/design/lib.md`, 6

## Owns and may change

- **Owns:** the files in `files`. Don't delete the item: `widen-close` does.

## Steps

1. Replace: the compiler sorts the types (the note).
2. `-p skein-io` is what the note's run names. Commit and push.

## Report

The note's items.
