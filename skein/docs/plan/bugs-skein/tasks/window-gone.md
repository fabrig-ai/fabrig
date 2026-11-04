+++
id = "window-gone"
part = "io"
kind = "feature"
needs = ["io-new-valid"]
files = ["skein/crates/skein-io/src/layer.rs", "skein/crates/skein-io/src/tests/signals.rs", "skein/docs/design/io.status.md"]
effort = "high"
minutes = 45
+++

# A `Window` on a gone stream reaches no kernel

## Goal

`skein-io`'s `Request::Window` arm (`src/layer.rs`) uses `Fd::new(-1)` for a stream gone or closing and
submits it. The backends run `ioctl(-1)`, answer `EBADF`, and the owner gets `Gone`, so the outcome is
right, but a descriptor no backend issued reaches the kernel, against `kernel.rs`'s `Fd`. (Bug 15's
first half, re-checked: holds.) Answer `Gone` with no operation.

Acceptance:

- A `Window` naming a stream gone or closing submits nothing; its owner gets `Window { Gone }` on the
  next pass up.
- `Tables` gains `gone: Option<Token>`, the owner owed `Gone`. `down` holds events for the next pass up,
  so the arm sets it, and `resume` tells it, as it tells `refused`.
- `is_ready`, `is_empty` and the "one `Window` request in flight" assert count it.
- No public type or signature changes; `MAX_OUT_*` stay.
- The test below fails on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/io.md`, 7 ("A `Window` naming a stream gone or closing"), 3.1.

## Owns and may change

- Owns: `src/layer.rs` (`Tables`, the `Window` arm, `resume` and its doc, `is_ready`, `is_empty`),
  `src/tests/signals.rs`.
- `skein/docs/design/io.status.md`: remove the sentence "7: a `Window` naming a stream gone or closing
  submits `Fd(-1)`." from the **Not yet** entry (`notes/common.md`); `file-op-lane` owns the rest of it.
- No reader changes: `Tables` is `pub(crate)`, `Op::Window` and `Event::Window` do not change. Loops
  already call `resume` after `down` for `refused`. No test asserts `Fd(-1)` on a `Window`.

## Steps

1. In `src/tests/signals.rs`, write `a_window_on_a_stream_closing_or_gone_is_gone_without_the_kernel`,
   modelled on `a_window_on_a_terminal_that_hung_up_is_gone`: `adopt_read_pipe`, `Abort`,
   `rig.down(Request::Window { .. }).nothing()`, then `rig.next().events` is `[Window { Gone }]`; settle
   the entity, repeat on the reclaimed token, then `rig.empty()`. Run on the old code and quote the
   failure.
2. The fix.
3. `heavy cargo nextest run -p skein-io --lib`, the focused tests of `skein-io`, and `heavy cargo nextest
   run -p smith-local-service -E 'test(/terminal/)'`, whose `Pending` state waits on this terminal.
   `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-io --all-targets
   -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the test showed on the old code.
