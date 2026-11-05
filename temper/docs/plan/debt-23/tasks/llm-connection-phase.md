+++
id = "llm-connection-phase"
part = "llm-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-llm-connection/src/call.rs", "skein/crates/skein-llm-connection/src/component.rs"]
effort = "high"
minutes = 40
+++

# skein-llm-connection: `Phase` holds the connection's call

## Goal

Half of skein D023. The item's "call" is `Connection` in `call.rs`: a data-less `Phase` with `call: Option<Token>` beside it. `Phase` takes the call, the `phase ==` tests go, and the write-only `idle_at` goes.

The sketch binds; `Phase` stays `Copy`:

```rust
enum Phase {
    Connecting { call: Option<Token> },
    Handshaking { call: Option<Token> },
    Head { call: Token },
    Streaming { call: Token },
    Draining,
    Idle,
    Ready,
    Closing { call: Option<Token> },
}
```

Acceptance:

- `rg -n 'phase (==|!=) Phase::|idle_at|self\.call\b[^(]|connection\.call\b[^(]' skein/crates/skein-llm-connection/src/call.rs skein/crates/skein-llm-connection/src/component.rs` reads nothing. Reads go through `call()`, and `terminal()` is the transition: Head or Streaming to Draining; Connecting, Handshaking and Closing to `call: None`; the rest as they are.
- `closing && !waiting && call.is_none() && phase != Closing` becomes an exhaustive helper: Draining, Idle and Ready are true; Connecting and Handshaking are `call.is_none()`; Head, Streaming and Closing are false.
- **`sync_deadlines` is this machine's follow,** as the item's "then follow" asks. It stays where it is, and you add no call to it: `Table::arm` clears `whole` for good.
- `heavy cargo nextest run -p skein-llm-connection --lib` and `-p skein-llm-connection-world --test suite` pass, with no assertion changed.
- About +10 net lines.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `skein/docs/development/debt/D023-llm-client-states.md`
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/llm-connection.md`, 5

## Owns and may change

- **Owns:** the files in the header; `component.rs` in its five `phase` tests, its `idle_at` line, the two lines near its test helper that write `phase` and `call`, which merge into one, and `connection.call` reads.
- **`waiting`, `reservation`, `control`, `activity`, `deadlines` stay on `Connection`:** `refresh_waiting` recomputes `waiting` for every connection in every phase, and `reservation` outlives the call's token until `refresh_memory`'s pass. `llm-states-close` writes that in the status.
- **Keep:** in `route`, Head stays Head through the whole loop, and Streaming is set only after it, as the `sent` deadline depends on that.
- **Cells the new shape makes visible:** `connected` from `Connecting { call: None }`, `Ready` from `Handshaking { call: None }`. A reading has them unreachable: `llm_events` pops first in `route`, `Failed` and `Close` are consecutive, no deltas come before Start, and the link answers `Ready` only while `Ending::Open`. If today's code handles such a cell quietly, and you can't show it's unreachable, keep it quiet, with `call: Option<Token>` in Head and Streaming as well if that's what it takes, and say so.
- **The client isn't yours:** `llm-client-states` runs beside you, and this crate uses only its public API, which doesn't change.

## Steps

1. `Phase`, `call()`, `terminal()`; let the compiler find the rest.
2. The helper; `idle_at` goes.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-llm-connection --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; what you did with the unreachable cells. No new rule, so no test to bite.
