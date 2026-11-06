+++
id = "store-pin"
part = "spot-review"
kind = "feature"
needs = []
files = ["hearth/crates/hearth-domain-store/src/tests/find.rs", "hearth/crates/hearth-domain-store/src/tests/fixtures.rs"]
effort = "medium"
minutes = 15
+++

# hearth-domain-store: a failed entry keeps the store's standing decline

## Goal

Finding 7's second pin in `temper/docs/plan/debt-25/inputs/debt23-review.md`, checked against the code. `enter.rs` declines a store on a failed entry only `if store.standing == Standing::Usable`, so a store already declining keeps why. `a_find_without_an_index_is_declined` starts both its rows from `Unusable { step: Step::List }` or `Usable` (`fixtures::full()` sets `Unusable` despite its name), so removing that guard stays green.

Acceptance:

- A row in `a_find_without_an_index_is_declined` (`tests/find.rs`) starts from `Decline::Full { .. }`, has an entry fail (`blob`, `Event::Failed`), and expects a find to answer `Declined` with that same `Full`. The test becomes a table of store and expected `why`.
- Removing the `if store.standing == Standing::Usable` guard in `enter.rs` turns that row red. Restore it.
- The other two rows still assert what they did.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, finding 7
- `hearth/crates/hearth-domain-store/src/lib.rs`'s `Standing`, and `enter.rs`'s `Event::Done | Event::Failed` arm

## Owns and may change

- **Owns:** the files in the header. A fixture beside `failed_entry` takes the starting decline, or the row sets `store.store.standing` as `fixtures::full()` does: compact either way, your choice.
- **`fixtures::full()` keeps its name** here: `tests/enter.rs` reads it too, and a rename adds nothing this pin needs.

## Steps

1. The row and its fixture. Break the guard, see the row fail, restore.
2. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-domain-store --all-targets -- -D warnings`; `heavy cargo nextest run -p hearth-domain-store --lib`. Commit and push.

## Report

The common note's items; what you broke and what failed.
