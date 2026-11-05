+++
id = "turn-states"
part = "forgejo-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/plans/turn.rs"]
effort = "medium"
minutes = 30
+++

# skein-forge: `turn`'s plan holds its states

## Goal

`turn::Plan` onto the note's shape, as `skein/docs/development/debt/D021-forgejo-plan-states.md` asks. Behaviour stays. Production lines stay about level.

Acceptance:

- The note's search A reads nothing for `turn.rs`, but for `turn::Mode` as `new`'s argument.
- Only the mode's own list is made: `runs` for `Runs`, `entries` for `Tree`, `jobs` for `Run`, with today's capacities, at the moment `new` makes them.
- The note's tests pass, with no assertion changed. `plans/tests/calls/turn.rs` runs unchanged.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/notes/forgejo-plans.md`, all but the other plans' shapes
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** `turn.rs`. `plans/tests/calls/turn.rs`, only for a pin it lacks, one row.
- The running charges in this file use `crate::measure`'s constants and stay.

## Steps

1. `State`, then `begin`, `row`, `done`.
2. A rule to break: make `State::Runs` count a row whose `commit` differs as a match; a test in `plans/tests/calls/turn.rs` should go red. If none does, that's the pin.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; search A's count for the file; the rule broken and what failed.
