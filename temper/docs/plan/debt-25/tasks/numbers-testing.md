+++
id = "numbers-testing"
part = "numbers"
kind = "feature"
needs = []
files = ["jig/testing/jig-test-connector/src/tests/settings.rs", "jig/testing/jig-test-service/src/tests/clients.rs", "jig/testing/jig-client-test-domain/src/family/wire.rs", "jig/testing/jig-client-test-records/src/activity_reads.rs"]
effort = "medium"
minutes = 30
+++

# Four test kits name their numbers

## Goal

Jig D035's rows in `jig/testing`: 17 of the Measure's 37. Each file is small, and its fix is known.

Acceptance:

- **`jig-test-connector`, `tests/settings.rs`:** `PROPOSAL` names 12. `proposal_record` drops its `proposal` parameter, since all six calls pass 12. The file's other `proposal: 12` and `Some(12)` read `PROPOSAL`.
- **`jig-test-service`, `tests/clients.rs`:** `System::new(16, 16)` reads `limits.root.connector`, as `startup.rs` does. The capacities stay 16 and 16.
- **`jig-client-test-domain`, `family/wire.rs`:** `PANEL` replaces 77 and `PANEL + 1` replaces 78. The capacities are `limits.rows` and `limits.rows + 1`, as `connector_bounds.rs`'s rule has them.
- **`jig-client-test-records`, `activity_reads.rs`:** `first_turn(bytes)` and `second_turn(bytes)` hold the two `turn_line` calls, so `decoded` and `unread(FULL_TURN)` stop writing them out. Those two calls stay, with a comment naming why: `jig-client-protocol-link/src/tests/fixtures/turns.rs` pins the same values.
- These four files' Measure lines read 2 in all. Expected values don't change.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/notes/numbers.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`

## Owns and may change

- **Owns:** the files in the header. Test lines only: `jig-client-test-domain` depends on `jig-client-view`, which stays off.
- **`limits` not in reach at `clients.rs`'s line:** take it from what the test already builds, or add one `let`. Don't add a parameter to a shared helper.

## Steps

1. One file at a time, with `heavy cargo nextest run -p <its package> --lib` after each.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-test-connector -p jig-test-service -p jig-client-test-domain -p jig-client-test-records --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-test-connector -p jig-test-service -p jig-client-test-domain -p jig-client-test-records --lib`, then `-p jig-client-protocol-link --lib`. Commit and push.

## Report

The common note's items; the four files' Measure count. These are renames: no rule to break by hand.
