+++
id = "fleet-claimed"
part = "spot-review"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-fleet/src/**"]
effort = "medium"
minutes = 25
+++

# jig-domain-fleet: a claimed attempt holds its claim

## Goal

Finding 5's last line in `temper/docs/plan/debt-25/inputs/debt23-review.md`, checked against the code. `Claiming { to, workstream, kinds, version }` already names the parent's claim, and `State::Waiting` holds one. `State::Claimed` unpacks the same four beside `at`, and code that moves from one to the other packs and unpacks them.

Acceptance:

- `State::Claimed` is `{ claiming: Claiming, at: Where }`.
- `State::Claimed { to, at, workstream, kinds, version }` is spelled nowhere, and an arm that passes all four on passes the `Claiming` whole.
- Production falls by about 8 lines. No test's assertion changes.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, finding 5
- `jig/docs/design/domain/engine.md`, 6 and 7.2, for what a claim is

## Owns and may change

- **Owns:** the crate's `src/**`: `attempt.rs`, `call.rs`, `grants.rs`, and `tests/` in their spelling only.
- **Stay off:** `Claiming`'s fields. `jig-domain-tasks` and `jig-protocol-engine-clients` have `Claiming`s of their own, which this doesn't touch.

## Steps

1. The variant, then the readers. An arm that reads only `at` keeps `..`.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-fleet --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-fleet --lib`. Commit and push.

## Report

The common note's items. This is a move: no rule to break by hand.
