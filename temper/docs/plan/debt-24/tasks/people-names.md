+++
id = "people-names"
part = "world-verbs"
kind = "feature"
needs = []
files = ["jig/tests/people/src/world.rs", "jig/tests/people/tests/**"]
effort = "medium"
minutes = 25
+++

# jig's people world: the owner signs in

## Goal

Jig D035's people part, 36 of the Measure's 262. `world.signin(10, 1)` is 17 of `signin`'s 33, and `world.signin(11, 2)` is 6.

Acceptance:

- `World` has `sign_in_owner()` and `sign_in_second()`, for those two pairs. `simulation.rs`'s `SIGN_IN = 10` goes, or its literals use it, whichever holds fewer lines. The rest of `signin`'s rows read a named sign-in and person, where it doesn't add a line per row, or stay where the number is the point.
- A one-shot untracked `sed` is safe for those exact literals, and only those.
- The Measure over `jig/tests/people` reads at most six, and each left is in the report with why.
- `heavy cargo nextest run -p jig-people-world` passes, with no assertion changed. Tests' net lines don't rise.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`

## Owns and may change

- **Owns:** the files in the header.

## Steps

The verbs; the `sed`; the rest by reading. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-people-world --all-targets -- -D warnings`; the test above. Commit and push.

## Report

The common note's items; the Measure before and after. No new rule: nothing to break by hand.
