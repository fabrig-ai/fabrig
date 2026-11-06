+++
id = "numbers-accounts"
part = "numbers"
kind = "feature"
needs = []
files = ["jig/tests/accounts/**"]
effort = "medium"
minutes = 30
+++

# The accounts world names its stories' rows, once

## Goal

Jig D035's accounts row: 8 of the Measure's 37. The world also repeats its fixtures: `ALICE` and `BOB` three times, and `PROVIDER` three times beside `src/lib.rs`'s own. Its `want`, `save`, `answer` and `refused` are copied.

Acceptance:

- **Named once:** `ALICE`, `BOB` and `PROVIDER` live in `jig/tests/accounts/src/lib.rs`, and the suite files read them. `ends.rs`'s `BOB = 9` is `ALICE` elsewhere (`model.rs`): apply `numbers.md`'s rule on different values, whichever way keeps every assertion.
- **`sign_in.rs`:** story names for accounts 5 and 10 and sources 11 and 13.
- **`model.rs`:** `Next { number: 23 }` names `NEXT`, and the two rows read `NEXT`, `NEXT + 1` and `NEXT + 2`, stating the rule. `NONE_PROVIDER` replaces 3.
- **`start_restore.rs`:** the existing `ACCOUNT` and new `PROGRAM_SOURCE` and `PROGRAM_PLACE`, also on the lines after it.
- **Acts:** `want`, `save` and `answer`, and `refused` and `commit` where their bodies are the same, each live once in the world's `src` or a suite support module. Near-identical ones stay separate if merging them needs a parameter that adds nothing.
- The world's Measure lines read 0. The test lines fall by about 35. Expected values don't change.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/numbers.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`
- `jig/docs/design/domain/accounts.md`, for the verbs' names

## Owns and may change

- **Owns:** `jig/tests/accounts/**`.
- **Reader:** if `jig/testing/jig-protocol-accounts-kit` already holds the name you want, read it from there and don't change it.

## Steps

1. The names into `src/lib.rs`, then the files, then the acts.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-accounts-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-accounts-world`. Commit and push.

## Report

The common note's items; net test lines; any value kept under a distinct name, and why. These are moves: no rule to break by hand.
