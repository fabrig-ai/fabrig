+++
id = "store-flight"
part = "spot-review"
kind = "feature"
needs = ["store-pin"]
files = ["hearth/crates/hearth-domain-store/src/staging.rs"]
effort = "medium"
minutes = 10
+++

# hearth-domain-store: a chain's step writes `Flight::Idle` once

## Goal

Finding 2's store part in `temper/docs/plan/debt-25/inputs/debt23-review.md`, checked against the code. `staging::step` takes the flight with `mem::replace(&mut store.flight, Flight::Idle)` before anything else. `dropped`, `failed` and `published` are called only from under `step`, so their writes of `Flight::Idle` change nothing. Nor does the one in `step`'s `Next::Snapshotted` arm.

Acceptance:

- The five `store.flight = Flight::Idle` lines in `dropped`, `step`'s `Next::Snapshotted` arm, `failed` (two) and `published` go. `rg -c 'store.flight = Flight::Idle' hearth/crates/hearth-domain-store/src/staging.rs` reads no more than what stays for another reason, and your report names any that stays.
- `drop_layer`, which sets `Flight::Chain`, stays. No test changes. Production falls by five lines.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, finding 2

## Owns and may change

- **Owns:** the file in the header.
- **If a caller reaches one of these functions other than under `step`,** that write stays and your report says which.

## Steps

1. Check the callers (`rg -n '\b(dropped|failed|published)\(store' hearth/crates/hearth-domain-store/src`), then remove the lines.
2. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-domain-store --all-targets -- -D warnings`; `heavy cargo nextest run -p hearth-domain-store --lib`. Commit and push.

## Report

The common note's items. This removes writes that do nothing: no rule to break by hand.
