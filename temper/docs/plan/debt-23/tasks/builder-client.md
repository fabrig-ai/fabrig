+++
id = "builder-client"
part = "builder-lifecycles"
kind = "feature"
needs = []
files = ["hearth/crates/hearth-protocol-builder/src/client.rs"]
effort = "high"
minutes = 45
+++

# hearth-protocol-builder: a client's output and stream are enums

## Goal

Hearth D006. `client.rs` has `active`, `refused` and `finished`, and `closing: Option<Token>`, `terminal` and `broken` beside them. `broken`'s `Fault` is never read, only tested for presence. About +10 net lines.

The sketch binds:

```rust
enum Output { Unbound, Bound(Token), Closing(Token), Closed }
enum Stream {
    Taking, Printing,
    Ending { outcome: Outcome, printing: bool },
    Finished { outcome: Outcome },
    Broken { outcome: Option<Outcome> },   // replaces broken + finished + a pre-reply terminal
}
```

Output's moves: `Bound` on `Closed` to `Unbound`, `Closing` on `Closed` to `Closed`. `bound()` is everything but `Unbound`, exactly today's `output || closing` table.

Acceptance:

- `rg 'active: bool|finished: bool|closing: Option|broken: Option' hearth/crates/hearth-protocol-builder/src/client.rs` reads nothing.
- The tests in the note pass, with no assertion changed.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/builder-states.md`
- `hearth/docs/development/debt/D006-lifecycles-in-flags.md`
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the file in the header. `lib.rs` calls this type's methods, and they keep their signatures.
- **`refused` and `failed` stay:** inputs to `outcome()`, not stages.
- **The two `Queue::with_capacity` declarations stay exactly as they are:** D001's records round rewrites them, and this keeps that merge clean.

## Traps

- `fail` leaves `terminal` alone.
- `finish` overwrites `terminal` with `Done` even when broken.
- `stop` pushes `Down::Finish` even when broken.
- `takes` also reads both queues.
- The `bind` assert passes today after a close, though the service's `output_bound` guard stops it being reached. Keep it passing.

## Steps

1. `Output`; then `Stream`.
2. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-protocol-builder --all-targets -- -D warnings`; the tests. Commit and push.

## Report

The common note's items. No new rule, so no test to bite.
