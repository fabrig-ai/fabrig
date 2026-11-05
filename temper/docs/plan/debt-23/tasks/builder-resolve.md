+++
id = "builder-resolve"
part = "builder-lifecycles"
kind = "feature"
needs = ["builder-child"]
files = ["hearth/crates/hearth-protocol-builder/src/resolve.rs", "hearth/crates/hearth-protocol-builder/src/tests/resolve.rs"]
effort = "high"
minutes = 75
+++

# hearth-protocol-builder: `Resolve`'s run, indices and terminal are one enum

## Goal

Hearth D006. `resolve.rs` has `closing` twice, with `ended` beside one, and `terminal`, `indices` and `walker` as `Option`s beside `Scratch`. They become one stage, a scratch that holds its walker, and pipes that hold a `Held`. About 0 to +15 net lines.

The sketch binds:

```rust
enum Stage {
    Running,                     // scratch, run, pipes; no terminal
    Indices(Indices),            // exit 0; the deadline still runs
    Stopping { indices: Indices, terminal: RenderEvent }, // it fired during the indices
    Ending(RenderEvent),         // close resources, clean scratch
    Spent,                       // terminal handed up
}
enum Scratch { Absent, Made, Walking(Walker), Removing, Cleaned }
struct PipeReader { held: Held, reader: Reader, ended: bool }
```

Acceptance:

- **A pin first, on today's code:** no lib test fires a deadline during the indices; `tests/resolve.rs` fires one only during the run. One row: the deadline fires while the indices are being kept, and the terminal comes as today. It passes first. Break it by dropping the deadline's check in the indices stage, see it fail, restore. This row stays.
- `rg 'closing: bool|terminal: Option|indices: Option|walker: Option' hearth/crates/hearth-protocol-builder/src/resolve.rs` reads nothing.
- The tests in the note pass, with no assertion changed but the new row's.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/builder-states.md`
- `hearth/docs/development/debt/D006-lifecycles-in-flags.md`
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the files in the header. `Protocol`'s reads of `resolving` stay through `Resolve`'s methods; the `Option` around it is `builder-current`'s.
- **`PipeReader.ended` stays:** a fact the reader reported, not a stage.

## Traps

- `ready()` looks at `indices` first, while `resume` pops `pending` first.
- `index()` and `index_settled()` must also answer in `Stopping`, because a store keep may be in flight.
- `Some(true)` after `stop()` is unreachable, since `stop` sets `failed` and `resume` then gives `None` or `Some(false)`.
- The note's owner rule and close order.

## Steps

1. The pin.
2. `PipeReader`, then `Scratch`, then `Stage`.
3. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-protocol-builder --all-targets -- -D warnings`; the tests. Commit and push.

## Report

The common note's items; what you broke for the pin and what failed.
