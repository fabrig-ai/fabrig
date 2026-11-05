+++
id = "composed-sender"
part = "temper-shared"
kind = "feature"
needs = []
files = ["temper/tests/system/src/composed/link.rs"]
effort = "medium"
minutes = 10
+++

# temper: the composed world uses `Party::sender`

## Goal

`temper/tests/system/src/composed/link.rs` keeps a `sender` beside `Party::sender`, a D004 copy.

Acceptance:

- `rg -n 'fn sender' temper/tests/system/src/composed/link.rs` finds nothing.
- `heavy cargo nextest run -p temper-system-world --test suite -E 'test(composed)'` selects tests and passes.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`

## Owns and may change

- **Owns:** `temper/tests/system/src/composed/link.rs`.
- **Stays:** `word.words.clone()`, because `start_message` is private and renders otherwise; `identity`, a plain
  concatenation that `run_identity` would change; `hosted`'s absence: it takes a `hub::Assignment` and sets `workspace: None`
  and `save: false`, so it doesn't fit.

## Steps

1. `sender: word.from.sender()`; `fn sender` goes; `Decimal` leaves the import if nothing else uses it.
2. The test above.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-system-world --all-targets -- -D
   warnings`; the citations check. Commit and push.

## Report

The common note's items. Nothing to break by hand.
