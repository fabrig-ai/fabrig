+++
id = "numbers-people"
part = "numbers"
kind = "feature"
needs = []
files = ["jig/tests/people/**", "jig/testing/jig-people-kit/**"]
effort = "medium"
minutes = 25
+++

# The people world names its sign-ins and holds

## Goal

Jig D035's people row: 6 of the Measure's 37. `world.sign_in_owner()` exists, but `SIGN_IN` is written four times, `PROJECT` three times, and `CONNECTOR` and `PERSON` twice.

Acceptance:

- `tests/people/src/fixtures.rs` calls `world.sign_in_owner()`.
- The kit or the world's `src` holds constants for the owner's and second party's sign-in and user numbers, and `world.rs`'s sign-in helpers and `simulation.rs`'s calls read them.
- `simulation.rs`: `LIFTED_HOLD` and `RELEASED` name 31 and 2, on all their lines.
- `start_restore.rs`: the table's task 12 and message 7 are named.
- `SIGN_IN`, `PROJECT`, `CONNECTOR` and `PERSON` are each named once in the world or its kit, unless the values differ (`numbers.md`).
- The world's Measure lines read 0. Expected values don't change.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/numbers.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`

## Owns and may change

- **Owns:** the files in the header. The three `ask`s are different verbs, and they stay.

## Steps

1. The constants, then `fixtures.rs` and `world.rs`, then the suite files.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-people-world -p jig-people-kit --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-people-world -p jig-people-kit`. Commit and push.

## Report

The common note's items; net test lines. These are renames: no rule to break by hand.
