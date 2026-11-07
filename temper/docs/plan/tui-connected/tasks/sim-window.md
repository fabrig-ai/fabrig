+++
id = "sim-window"
part = "screen"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-sim/src/sim.rs",
  "skein/crates/skein-sim/src/sim/*.rs",
  "skein/crates/skein-sim/src/trace.rs",
  "skein/tests/sim/tests/suite/processes.rs",
  "skein/docs/design/simulator.status.md",
]
effort = "medium"
minutes = 45
+++

# The simulated machine has a terminal's window

## Goal

The full-screen shell's terminal asks its input's window size when the
window changes (`skein/docs/design/ui-screen.md`, section 8). On skein's
simulated machine a window record answers `ENOTTY` always, and a signal
source hears only terminations, so the terminal's resize cannot run in
`tests/ui-screen`. The simulator gains a window per descriptor and a
window change delivered to a signal source.

This task departs from the implementer's text in one way: a status entry
is yours to remove (acceptance 4).

Acceptance:

1. `Sim::set_window(pid, fd, size)` and `Sim::deliver_window_changed(pid,
   fd)` exist as `temper/docs/plan/tui-connected/notes/contracts.md`,
   section 5, has them.
2. `Op::Window { fd }` on a descriptor given a window answers
   `Ok(Done::Window(size))`, the size set last; on any other descriptor it
   answers `Err(Error::Other(25))` (`ENOTTY`), as today. A window given to
   a descriptor goes when the descriptor closes.
3. A window change waits on the signal source in order with its
   terminations, and a read of the source completes with
   `Done::Signalled(Signalled::WindowChanged)`, as a termination's does. A
   source with nothing pending parks its read as today.
4. `skein/docs/design/simulator.status.md` loses its **Not yet** entry
   for 3.2's window, the rest unchanged.
5. Rows in `skein/tests/sim/tests/suite/processes.rs`, beside
   `a_hosted_service_hears_an_interrupt_without_exiting`: a window record
   on a descriptor with a window answers its size, a changed size answers
   the new one, one with none answers `ENOTTY`; a window change and a
   termination delivered in that order are read in that order.

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, section 5: binds.
- `skein/docs/design/simulator.md`, 3.2, *A terminal's window*.
- `skein/docs/design/kernel.md`, 6.3, the window record.

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours:** the trace (`trace.rs`) may record the new
delivery as it records a termination's, if its summary needs a variant:
the compiler's arms only. The simulator's digest, if it digests a
process's signal sources or descriptors, includes the windows so that a
replay that differs in them differs in its digest; if it digests neither,
add nothing.

**Options, each with its default:**

- Where a window lives: a map from descriptor to size on the process, as
  `signal_fds` is. If descriptors of inherited pipes live elsewhere, keep
  the window beside them and say so.
- A window on a descriptor of another process, or on one not open, is
  the world's defect: `self.fail(pid, ...)`, as `deliver_service_signal`
  fails on a missing source.

## Steps

1. The window: `set_window`, the answer in the `Op::Window` arm of
   `sim.rs` (the one that answers `Error::Other(25)` today), the window
   dropped at close.
2. The change: `deliver_window_changed`, pushing
   `Signalled::WindowChanged` as `deliver_service_signal` pushes a
   termination, completing a parked read the same way. Share the
   delivery's tail between the two rather than copying it.
3. The rows (acceptance 5), in the shape of `hosted_signal`. Bite: answer
   `ENOTTY` for a descriptor with a window once, and see the row fail.
4. The status entry (acceptance 4).
5. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo
   clippy -p skein-sim -p skein-sim-tests --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-sim -p skein-sim-tests`. Commit
   ("skein-sim: a descriptor's window and a window change, so a full-screen
   terminal resizes on the simulated machine"), push.

## Report

As the implementer's text says, and: whether the digest or the trace
changed, and the rows' time.
