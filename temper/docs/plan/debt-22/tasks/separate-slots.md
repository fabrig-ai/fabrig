+++
id = "separate-slots"
part = "spot-review"
kind = "feature"
needs = ["tasks-pins", "tasks-families"]
files = ["jig/crates/jig-domain-tasks/src/boundary.rs", "jig/crates/jig-domain-tasks/src/inbox.rs", "jig/crates/jig-domain-tasks/src/stored/fit.rs", "jig/crates/jig-domain-tasks/src/lib.rs"]
effort = "high"
minutes = 60
+++

# jig-domain-tasks: the four separately priced messages are their own family

## Goal

Finding 3 of `temper/docs/plan/debt-22/inputs/debt20-review.md`. `Slot` holds two answers, so the split between the four messages priced in their own slots and the rest is typed three times. About 35 production lines fall. Behaviour stays.

Acceptance:

- `boundary.rs`: `pub enum Separate { Amendment, FinishRequested, Met, NothingAwaited }`, with one doc line citing `jig/docs/design/domain/tasks.md`, 5.7 and 8.2; `MessageFamily::Separate(Separate)`; `Slot` holds the other seven. `lib.rs` exports `Separate`.
- `inbox.rs`'s `room` and `stored/fit.rs`'s `ordinary` and its load check each split on `MessageFamily::Separate(_)` against the rest, with no list of `Slot` variants where the variant doesn't matter. `fit.rs`'s split of `Met | NothingAwaited` from `Amendment | FinishRequested` stays, on `Separate`.
- `Word::finish_requested` is one `==` against `MessageFamily::Separate(Separate::FinishRequested)`.
- `rg -n 'Slot::(Amendment|FinishRequested|Met|NothingAwaited)' jig/crates/jig-domain-tasks/src` reads nothing.
- `wake.rs`'s `category` puts `MessageFamily::Separate(_)` in 7, with what's there: no message changes category. `activity.rs`, `positions.rs`, `subscriptions.rs` and `compound/staged.rs` gain the arm the compiler asks for.
- `tests.rs`'s family column names `Separate`; its local `match` and every other assertion stay. `tests/startup/detail.rs`'s refusal rows pass untouched.
- `heavy cargo nextest run -p jig-domain-tasks -p jig-tasks-world` passes. `heavy cargo check -p jig-domain-core --tests` and `heavy cargo check -p jig-protocol-store --tests` pass.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/inputs/debt20-review.md`, finding 3
- `jig/docs/design/domain/tasks.md`, 5.7, 8.2

## Owns and may change

- **Owns:** the files in the header. `wake.rs`, `activity.rs`, `positions.rs`, `subscriptions.rs`, `compound/staged.rs`, `tests.rs`: the arm or the row only.
- **Public interface:** `MessageFamily` and `Slot` are exported, and no package outside `jig-domain-tasks` names either (`rg -n 'MessageFamily' --type rust jig temper smith skein`); the request names this change, so `Slot`'s narrowing is allowed. `Separate` is an addition. `MessageKind` stays, and no stored or wire form moves.
- **If a reader outside the crate has appeared** since this plan's base, stop and ask.

## Steps

1. The type and `family()`; the three splits; `finish_requested`; the compiler's arms.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-tasks --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items. No new test: `tasks-families`' table and `tasks-pins`' refusal rows hold this, and the report names that they ran unchanged but for the family column.
