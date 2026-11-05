+++
id = "git"
part = "ceilings"
kind = "feature"
needs = ["sums"]
files = ["hearth/crates/hearth-protocol-git/**", "hearth/scripts/old-forms.txt"]
effort = "medium"
minutes = 20
+++

# git's decoder bound

## Goal

`hearth_protocol_git::limits::worst_case` is named for what it bounds, and has a test of its value. git's world already holds a ceiling, so
nothing there moves.

Acceptance:

- `limits::decoder_bound(decoder, limits)` replaces it, by `notes/contracts.md`, 1.
- A test of its value stands in `hearth-protocol-git`, for every `Decoder`.
- `git grep -n worst_case -- hearth/crates/hearth-protocol-git` prints nothing.

## Inputs

- `hearth/docs/plan/hearth-memory-1/notes/common.md`
- `hearth/docs/plan/hearth-memory-1/notes/contracts.md`, 1 and 5
- `skein/docs/foundation/programming-model.md`, 6.3

## Owns and may change

- **Owns:** the files under `files`.
- **Not yours:** `hearth/tests/git`, which reads no formula.

## Steps

1. Rename it in `hearth/crates/hearth-protocol-git/src/limits.rs`.
2. Add the value test, `hearth/crates/hearth-protocol-git/src/tests.rs` or a module beside it: a table of `Decoder` rows at small limits with the
   expected `Some(..)`, and an overflowing row (`notes/common.md`, "A formula left").
3. `cargo fmt`, clippy on `hearth-protocol-git`, focused tests of `hearth-protocol-git` and `hearth-git-world`.
4. `notes/common.md`, "The end of every task". Commit and push.

## Report

The first line, the tests run and how many, and what bit.
