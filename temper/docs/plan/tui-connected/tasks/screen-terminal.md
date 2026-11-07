+++
id = "screen-terminal"
part = "screen"
kind = "feature"
needs = ["screen-keys", "sim-window"]
files = [
  "skein/crates/skein-ui-screen-terminal/src/terminal.rs",
  "skein/crates/skein-ui-screen-terminal/src/terminal/**",
  "skein/crates/skein-ui-screen-terminal/src/lib.rs",
  "skein/tests/ui-screen/**",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "high"
minutes = 150
+++

# skein-ui-screen-terminal: the terminal on skein's io

## Goal

`skein_ui_screen_terminal::Terminal`: the full-screen shell's terminal, on
skein's io as the text shell's terminal is. It enters the alternate
screen, reads keys, draws the host's newest frame at most once per
interval, asks the window's size when it changes, and on every way out
writes `LEAVE` last. Its world, `tests/ui-screen`, runs it on the
simulated machine.

`skein/docs/design/ui-screen.status.md` is not yours: tasks beside this
one run in parallel, and `screen-status` closes its entries once all have
merged.

Acceptance:

1. `Settings` and `Terminal` exist with the names and signatures of
   `temper/docs/plan/tui-connected/notes/contracts.md`, section 3,
   exported from `lib.rs`; the terminal's io operations carry tag 1
   (`NAMESPACE`), and `Terminal::owns` says which are its own.
2. It does what `skein/docs/design/ui-screen.md`, section 8, says:
   - `adopt` writes `ENTER` before anything else;
   - keys decoded as read, held for the host; the terminal asks to read
     only once the host has taken every key it gave;
   - frames: the newest frame the host gave drawn at most once per
     interval, as `draw` makes it from the last frame drawn; one write at a
     time, each at most the output's room (64 KiB), so a frame larger than
     that goes in pieces before the next; a frame given while one is being
     written waits, and only the newest waiting is drawn;
   - `resized` asks its input's window (`Request::Window`), one at a time,
     a resize heard meanwhile asked again once the answer lands; the size
     answered is `size()`, and the next frame is drawn whole; `Window::Gone`
     keeps the last size;
   - the input's end sets `ended()`; the terminal itself starts no close;
   - `close`: no new frame, the write begun finished, then `LEAVE`, then
     `closed()`, after which it writes nothing and asks no read.
3. `Terminal::operations(&settings)` is skein-io's count for its two
   pipes and its window request (as the text shell's is), `None` for
   settings it refuses: a `most` whose `draw_bound` is `None`, a paste of
   0 bytes, an interval of zero.
   `Terminal::new` refuses the same settings, and a size of 0 rows or 0
   columns.
4. `skein/tests/ui-screen` (`skein-ui-screen-tests`, `harness` profile, as
   `skein/tests/ui-text`) holds the world: one integration binary
   `tests/suite/main.rs` with a module `terminal`, its harness copied from
   `skein/tests/ui-text/tests/suite/terminal.rs`'s `Pipes` (a simulated
   process, inherited read and write pipes, a signal source; `drive`,
   `feed`, `drain`), with `sim.set_window` on the input and
   `sim.deliver_window_changed` for a resize, the host's part played by the
   test (taking keys, giving frames built with `Frame::put`, giving
   `Resized` to `resized`). Its stories, each a test:
   - `ENTER` is the first bytes written, before any frame;
   - keys split across reads arrive whole and in order, a paste among them;
   - frames paced: three frames given within one interval draw the first
     and then the newest, not the middle one;
   - a frame drawn while the output is held waits, and only the newest is
     drawn once it takes;
   - a frame larger than the output's room is written in pieces, whole,
     before the next frame's bytes;
   - a resize asks the window, `size()` is the new one, and the next frame
     is drawn whole (it starts by clearing);
   - `LEAVE` is the last bytes written, and nothing after it, on each way
     out, as rows of one table: `close` with nothing pending; the input's
     end, then `close`; `close` while a frame is being written (the frame's
     bytes whole, then `LEAVE`); Ctrl+C read, then `close` (the test plays
     the host that takes `Command::Close` from the key reader).

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, sections 1, 3 and
  5: they bind.
- `skein/docs/design/ui-screen.md`, sections 7, 8 and 9 (the terminal's
  tests).
- `skein/docs/design/ui-text.md`, sections 3 (its terminal and its
  operations) and 4 (one print at a time, the output's room); its
  `skein/crates/skein-ui-text/src/terminal.rs`, the sibling to follow.
- `skein/docs/design/io.md`, section 3 (pipes) and section 7 (`Resized`,
  the window request); `skein/docs/design/simulator.md`, 3.2, *A terminal's
  window*.

## Owns and may change

**Owned:** the `files` above: the crate's `terminal` module and its
`lib.rs` exports, and the test crate.

**Readers and neighbours:** the root `Cargo.toml` gains the test crate as
a member (no dependency entry: nothing depends on it); `Cargo.lock` its
entry. `skein-ui-screen-terminal`'s other modules (`keys`, `command`,
`draw`) are `screen-keys`'s and stay as they are; a `pub(crate)` helper
the terminal needs may be added to them.

**Options, each with its default:**

- Its io limits: as the text shell's `io_limits`, two sockets for the two
  pipes, `output` the output's room of 64 KiB, `intake` and `receive` a
  read of 4096 bytes. If skein-io needs a socket or a name for the window
  request, count it and say so.
- The frames it keeps: the last drawn and the newest given, each a
  `Frame::new(settings.most)` made at `new`, the newest copied in by
  `draw` (`clone_from`, or cell by cell). If copying a whole frame at each
  `draw` measures badly in the world's counting allocator, keep a flag and
  copy at drawing time, and say so.
- The keys' queue: capacity a read's bytes and one; the decoder gives at
  most one key per byte.
- The world's counting allocator and peaks: as `tests/ui-text`'s main, if
  it installs one; no new heap ceiling is asked here.

## Steps

1. `Settings`, `Terminal::operations`, `new`, `adopt` (with `ENTER`).
2. Reading: a read asked only when the host has taken every key; bytes to
   the decoder; the input's end.
3. Drawing: the interval, the newest frame, `draw` into the output's
   bytes, pieces at the output's room, one write at a time.
4. The window: `resized`, the request, the answer, the whole frame after.
5. Closing: the write finished, `LEAVE`, closed.
6. The world (acceptance 4). Bite: write `LEAVE` before finishing the
   frame's write once, and draw the middle frame once; see their stories
   fail, and restore.
7. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo
   clippy -p skein-ui-screen-terminal -p skein-ui-screen-tests
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   skein-ui-screen-terminal --lib` and `-p skein-ui-screen-tests`. Commit
   ("skein-ui-screen-terminal: the terminal enters, draws and leaves on
   skein's io"), push.

## Report

As the implementer's text says, and: the io limits chosen, how frames are
kept and copied, and the world's time.
