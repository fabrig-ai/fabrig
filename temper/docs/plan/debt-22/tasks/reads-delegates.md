+++
id = "reads-delegates"
part = "forgejo-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/plans/reads.rs", "skein/crates/skein-forge/src/forgejo/plans/reads/done.rs"]
effort = "medium"
minutes = 30
+++

# skein-forge: `reads`'s five optional plans become one `Delegate`

## Goal

The first step of `reads::Plan`'s states, so `reads-states` starts from a green tree. `turn`, `commits`, `artifact`, `contents` and `actions`, each an `Option` in `reads::Plan`, are one lifecycle, and only one is ever set. Behaviour stays.

Acceptance:

- The note's search B reads nothing.
- `reads::Plan` holds `delegate: Option<Delegate>`, with `Delegate` as the note has it. `reads-states` moves it into `State::Delegated` and `State::Status`.
- A read that delegated before delegates now, at the same moment. `Status`'s use of `actions` keeps working: if it runs beside another field's life, it stays that way here.
- The note's tests pass, with no assertion changed.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/notes/forgejo-plans.md`, all but the other plans' shapes
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the files in the header. The leaf plans are other tasks': only their public methods are read here, and they stay.
- **No new test:** this moves code, and `plans/tests/**` and `skein-forge-connection`'s `tests/reads.rs`, `tests/contents.rs` hold it.

## Steps

1. `Delegate`, then every site; each `if let Some(..) = self.turn`-style dispatch is one `match`.
2. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; search B's count.
