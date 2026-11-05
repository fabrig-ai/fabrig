+++
id = "small-plan-states"
part = "forgejo-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/plans/artifact.rs", "skein/crates/skein-forge/src/forgejo/plans/start.rs"]
effort = "medium"
minutes = 25
+++

# skein-forge: `artifact`'s and `start`'s plans hold their states

## Goal

`artifact::Plan` and `start::Plan` onto the note's shapes, as `skein/docs/development/debt/D021-forgejo-plan-states.md` asks. Behaviour stays. Production grows by at most 10 lines.

Acceptance:

- The note's search A reads nothing for `artifact.rs`; `start.rs` has no `ended` field.
- `artifact.rs` matches on its `State` per stage, and has no guard comparing a stage. Its `id.is_some()` check moves into `State::Listing`.
- `start::Plan` still ends at most once: `heard` after its end answers `None`. `begin` on `Ended` is `unreachable!`, as its owner runs it once; check `skein-forge-connection/src/start.rs` first, and if it can call `begin` twice, keep what happens today.
- The note's tests pass, with no assertion changed.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/notes/forgejo-plans.md`, all but the other plans' shapes
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the files in the header.

## Steps

1. `artifact.rs`, then `start.rs`.
2. A rule to break: make `artifact`'s `State::Listing` accept a second matching row; `plans/tests/calls/artifact.rs` should go red. If none does, that's a pin to add, one row.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; search A's count for the two files; the rule broken and what failed.
