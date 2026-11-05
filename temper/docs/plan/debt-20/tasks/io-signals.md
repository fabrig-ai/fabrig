+++
id = "io-signals"
part = "io-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-io/src/signals.rs", "skein/crates/skein-io/src/tests/signals.rs"]
effort = "high"
minutes = 20
+++

# skein: `Signals` holds its lifecycle in a state

## Goal

Part of `skein/docs/development/debt/D018-io-pipe-and-file-states.md`. `Signals` follows `skein/docs/foundation/programming-model.md`, 5.4, as `stream.rs` and `listener.rs` do: data-carrying states, `mem::replace` then follow. Its events and submissions stay the same.

```rust
pub(crate) struct Signals { owner: Option<Token>, state: State }
enum State {
    Open { fd: Fd },
    Reading { fd: Fd, read: Id<Flight> },
    Settling { fd: Fd, read: Option<Id<Flight>>, cancels: u32 },
    Releasing { close: Id<Flight> },
    Closed,
}
```

Acceptance:

- `rg -n 'closing: bool|closed: bool|fd: Option<Fd>|close: Option<|Ok\(_\)|matches!' skein/crates/skein-io/src/signals.rs` finds nothing. `rg -c 'mem::replace\(&mut signals\.state, State::Closed\)' skein/crates/skein-io/src/signals.rs` is at least 2.
- `heavy cargo nextest run -p skein-io --lib -E 'test(/signals/)'` and `heavy cargo nextest run -p skein-world-tests --test suite -E 'test(/signals/)'` pass, with no assertion changed.
- By hand: drop `read == Some(target)`'s check in the cancel's retry; `a_signal_read_retries_an_unsubmitted_cancel_only_while_its_target_waits` goes red. Restore.
- The net production lines are at most +50 (estimate: +35: the exhaustive arms 5.4 asks for).

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `skein/docs/development/debt/D018-io-pipe-and-file-states.md` (its "What" counts are off: `Signals` has 2 bools and 4 `Option`s)
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/io.md`, 7
- `skein/crates/skein-io/src/stream.rs`: the model

## Owns and may change

- **Owns:** the files in `files`. The item stays: `io-file-limits` deletes it. `io.status.md` stays: `io-file-limits` closes its entry.
- **Reader:** `layer.rs` calls `new`, `resume`, `landed`, `close` and `is_closed`: their signatures stay, so it shouldn't change.
- **The file's header table** gains the two rows it lacks (Open, then Close, goes to Releasing; the retry of an unsubmitted cancel is `Settling`'s cell). A read error goes straight to `Releasing`, as today.
- **`Ok(_)`'s `unreachable!`** becomes `Done`'s arms, listed.

## Steps

1. The state, `resume`, `landed`, `close`; `maybe_close` becomes the state's follow.
2. Break the rule by hand, as the acceptance says.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-io --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-io --lib`; the tests above. The citations check. Commit and push.

## Report

The common note's items; what you broke and what failed.
