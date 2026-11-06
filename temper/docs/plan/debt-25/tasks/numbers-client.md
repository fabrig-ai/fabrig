+++
id = "numbers-client"
part = "numbers"
kind = "feature"
needs = []
files = ["jig/testing/jig-client-scripted-engine/src/tests.rs", "jig/testing/jig-client-scripted-engine/src/tests/**", "jig/testing/jig-client-scripted-engine/src/codec/tests.rs", "jig/tests/client/tests/suite/settings.rs"]
effort = "medium"
minutes = 30
+++

# The scripted engine and the client world name their sign-ins

## Goal

Jig D035's rows in the client: 5 of the Measure's 37. `tests/escalation.rs` already names sign-ins 11 and 22 and parties 1 and 2, so the names exist and need one home.

Acceptance:

- The scripted engine's tests read one set of names for their sign-ins and parties, hoisted from `tests/escalation.rs` to `tests.rs`, plus one for sign-in 12. `PROJECT`'s two copies are one. The sign-in map's inserts in `tests.rs` and `codec/tests.rs` read them.
- The sweep: the lines holding `11` that mean this sign-in read the name. Read each: some are other numbers, and those stay. Your report counts both.
- `jig/tests/client/tests/suite/settings.rs`: a local `SIGN_IN` replaces 11 (also at the file's other uses), and `jig_client_kit::PARTY` replaces 7, if its value agrees. If it doesn't, keep the party's value under a local name.
- These files' Measure lines read 0. Expected values don't change.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/numbers.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`

## Owns and may change

- **Owns:** the files in the header. Test lines only: `jig-client-world` depends on `jig-client-view`, which stays off.

## Steps

1. The scripted engine's names, then its sweep, then the client world's file.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-client-scripted-engine -p jig-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-client-scripted-engine --lib`, then `-p jig-client-world --test suite -E 'test(settings)'`. Commit and push.

## Report

The common note's items; lines swept and lines left as other numbers. These are renames: no rule to break by hand.
