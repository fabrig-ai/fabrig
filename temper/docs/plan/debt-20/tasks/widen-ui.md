+++
id = "widen-ui"
part = "skein-widening"
kind = "migration"
needs = ["widen-kit"]
files = ["skein/crates/skein-ui/src/**", "skein/crates/skein-ui-web/src/**", "skein/crates/skein-ui-text/src/**", "skein/crates/skein-playground/src/**", "skein/examples/ui/**"]
effort = "medium"
minutes = 20
+++

# skein: `skein-ui`, `skein-ui-web`, `skein-ui-text`, `skein-playground` and the ui example widen through lib's helpers

## Goal

These crates write 43 widening conversions by hand at this plan's base. Each `u32` widens with `usize_of` and each native length with `len_u64`, following the note, so nothing but a real `u64` conversion stays. Behaviour doesn't change, and production lines don't rise.

It goes after `widen-kit` only to come later: other plans are changing `skein-ui` and `skein-ui-web`, and this task's lines there are few.

Acceptance:

- The note's two searches on `skein/crates/skein-ui`, `skein/crates/skein-ui-web`, `skein/crates/skein-ui-text`, `skein/crates/skein-playground` and `skein/examples/ui` list only what your report names as staying.
- `heavy cargo clippy` and `heavy cargo nextest run` pass on these packages, with what they asserted unchanged.
- The branch's net production lines are at most 0.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `temper/docs/plan/debt-20/notes/widening.md`: the rule, what stays, the searches, the run
- `skein/docs/development/debt/D002-index-and-length-widening.md`
- `skein/docs/design/lib.md`, 6

## Owns and may change

- **Owns:** the files in `files`.
- **`skein/examples/ui/view/src/generated/v1.rs` stays** (18 sites).
- **Don't reshape a line for its own sake** in `skein-ui/src/{tree,builder}.rs`, `skein-ui-web/src/{mapping,page}.rs` and `page/**`: other plans are changing those files, and only the sites themselves move.
- **Another crate's site stays:** other `widen-*` tasks own the rest of skein. Don't delete the item: `widen-close` does.

## Steps

1. Replace, crate by crate: the compiler sorts the types (the note).
2. `-p skein-ui -p skein-ui-web -p skein-ui-text -p skein-playground -p skein-ui-example-view -p skein-ui-example-server -p skein-ui-example-shell-web` is what the note's run names. Commit and push.

## Report

The note's items.
