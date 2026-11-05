+++
id = "builder-read"
part = "builder-lifecycles"
kind = "feature"
needs = ["builder-child"]
files = ["hearth/crates/hearth-protocol-builder/src/read.rs"]
effort = "high"
minutes = 75
+++

# hearth-protocol-builder: a read's current work and its feed are enums

## Goal

Hearth D006's **Where** names `read.rs`, and its **What** doesn't, but the pattern is all there: `cache`, `body`, `decode`, `decoded`, `terminal` as `Option`s, `arrival` meaningful only with `cache`, `room: Option<bool>`, `finished_input`, `sent`. About +10 net lines.

The sketch binds:

```rust
enum Current {
    Body(Writer), Decoding(Decode), Decoded(Decoded),
    Caching { cache: Cache, arrival: Arrival }, // Arrival::Settled holds the last blob's terminal
    Spent,
}
enum Feed { Due { sent: usize }, Asked { sent: usize }, Room { sent: usize }, Finished }
```

Acceptance:

- `rg 'cache: Option|body: Option|decode: Option|decoded: Option|room: Option|finished_input' hearth/crates/hearth-protocol-builder/src/read.rs` reads nothing.
- The tests in the note pass, with no assertion changed.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/builder-states.md`
- `hearth/docs/development/debt/D006-lifecycles-in-flags.md`
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the file in the header.
- **`failed` and `ended` stay beside these, and don't merge:** `lib.rs`'s `failure()` reads `ended` for a read that hasn't failed, relying on the default `Stream`, and `Decode`'s `Err` sets `failed` and leaves `ended` alone.
- **`take_answer`'s `Cached` arm** looks unreachable, since `settle` parks a terminal first and `finish` takes that. Nobody's sure. Keep what it does; if the new shape can't hold it, say so.

## Steps

1. `Feed`; then `Current`.
2. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-protocol-builder --all-targets -- -D warnings`; the tests. Commit and push.

## Report

The common note's items. No new rule, so no test to bite.
