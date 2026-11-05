+++
id = "io-pipe"
part = "io-states"
kind = "feature"
needs = []
files = ["skein/crates/skein-io/src/pipe.rs", "skein/crates/skein-io/src/tests/append.rs", "skein/crates/skein-io/src/tests/process.rs"]
effort = "high"
minutes = 75
+++

# skein: `Pipe` holds its lifecycle in a state

## Goal

Part of `skein/docs/development/debt/D018-io-pipe-and-file-states.md`. `Pipe` follows `skein/docs/foundation/programming-model.md`, 5.4, as `stream.rs` does: data-carrying states, `mem::replace` then `follow`. Its events, submissions and deadlines stay the same.

```rust
pub(crate) struct Pipe { child: Option<Id<Entity>>, way: Way, append: bool, write_timeout: Option<Duration>, independent: Reservation, state: State }
enum State {
    Pending,
    Adopted { fd: Fd, demand: Option<(Read, u32)> },
    Reading(Reading),   // fd, intake, demand: Option<Read>, source: Source
    Writing(Writing),   // fd, queued, queued_bytes, write, granted, demand: Option<u32>, finishing
    Flushing(Flushing), // fd, queued, queued_bytes, write, cancels, flush: Whole | Spill
    Settling { fd: Fd, read: Option<Id<Flight>>, cancels: u32 },
    Releasing { close: Id<Flight> },
    Closed,
}
enum Source { Idle, Reading(Id<Flight>), Eof, Ended }
```

| From | Event | To |
|---|---|---|
| Pending | activate | Adopted |
| Adopted | resume | Reading or Writing, by way (buffers made) |
| Adopted | Finish, Close | Releasing |
| Reading | count 0, then End told | `Source::Eof`, then `Ended` |
| Reading | Err | `Source::Ended`, `Failed` |
| Reading | Close, Abort | Settling (cancel the read), or Releasing if idle |
| Settling | read or cancel landed | Settling, or Releasing at zero |
| Writing | graceful Close | Flushing Whole (close deadline) |
| Writing | Abort, deadline | Flushing Spill, queue dropped, write cancelled |
| Writing | Err with a timeout | `Failed`, then as Abort |
| Writing | Err without one | `Failed`, Flushing (Whole if finishing, else Spill, queue kept) |
| Writing, finishing | drained | Releasing |
| Flushing Whole | Abort | Spill, queue dropped, write cancelled |
| Flushing | drained, no cancels | Releasing |
| Releasing | Close landed | Closed: the output's terminal, then `Closed` |

Acceptance:

- `rg -n 'pipe\.(closing|closed|finish|eof|ended|classic_grant|queued_bytes|intake)\b|struct_excessive_bools' skein/crates/skein-io/src/pipe.rs` finds nothing. `rg -c ': bool,' skein/crates/skein-io/src/pipe.rs` is 1 (`append`). `rg -c 'mem::replace\(&mut pipe\.state, State::Closed\)' skein/crates/skein-io/src/pipe.rs` is at least 5.
- **New pins, written and passing against today's code in their own first commit,** unchanged after it: a write error on a no-timeout pipe with bytes queued (Spill); a pipe's read error; the abort of a read and its unsubmitted cancel; Finish then Demand.
- `heavy cargo nextest run -p skein-io --lib` and `heavy cargo nextest run -p skein-io-world --test suite` pass, with no assertion changed.
- By hand: treat a short write as done in Flushing Whole (`close_flushes_under_its_own_deadline_and_short_writes_do_not_move_it` goes red); restart the queue after an abort (`a_write_that_wins_its_cancel_does_not_start_the_dropped_queue`); tell `Closed` before the output's terminal (`close_and_abort_carry_the_pending_terminal_before_actual_closed_without_a_second_answer`). Restore each.
- The net production lines are at most +130 (estimate: +100: the exhaustive arms 5.4 asks for). Compact first.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `skein/docs/development/debt/D018-io-pipe-and-file-states.md` (its "What" counts are off: `Pipe` has 7 bools and 8 `Option`s)
- `skein/docs/foundation/programming-model.md`, 5.4
- `skein/docs/design/io.md`, 3, 5.1 and 6
- `skein/crates/skein-io/src/stream.rs`: the model. `Reading` and `Writing` play `Open`'s reader and writer, `Flushing` plays `Closing` with `Flush`, and `independent` sits outside as `Stream::owed` does.

## Owns and may change

- **Owns:** the files in `files`. The item stays: `io-file-limits` deletes it.
- **Readers:** `layer.rs` (`append`, `inherited`, `window_fd`, `child`, `is_closed` and the six entry functions), `process.rs` (`pending`, `activate`, `is_closed`). Their signatures stay, so neither should change. `joined.rs`, `joined/end.rs` and `listener.rs` only name `Entity::Pipe(_)`.
- **What can't simply move, each kept as it is today:**
  - `child` is read by `conclude` after Closed, so it stays outside the state.
  - `independent` outlives the descriptor, and its terminal precedes `Closed`.
  - `intake` is lazy because neither `adopt_*` nor `activate` has `env`: hence `Adopted`.
  - A write's deadline is per write, armed at its submit: it stays in the handlers, not in a `tidy`.
  - A cancel retries only with `write_timeout` set and the write as its target. A read's unsubmitted cancel is not retried (unlike `Signals`).
  - In Spill, a short write counts as finished and a later error is ignored, while `Count(0)` panics there. It resubmits in Writing and Whole.
  - Finish then Demand still grants Room, and a later Send panics.
  - A Demand can sit beside a live grant, so `demand` and `granted` both stay. `classic_grant` folds into `granted > 0`.
  - Close on Pending is unreachable (tokens are told only after activation, and a failed spawn retires its pipes directly): `unreachable!`.
- **Splitting by way drops today's unused `Intake` on a write pipe and `Queue` on a read pipe.** Peak heap only falls, and that's fine. If a memory test pins an exact figure that moves, keep both allocations in the state and say so in your report.

## Steps

1. The four pins against today's code. Commit.
2. The state and the machine, `progress` becoming `follow`.
3. Break the rules by hand, as the acceptance says.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-io --all-targets -- -D warnings`; the tests above. The citations check. Commit and push.

## Report

The common note's items; each pin's name; what you broke and what failed; whether a memory figure moved.
