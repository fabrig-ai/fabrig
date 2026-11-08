+++
id = "skein-foreground"
part = "foreground"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-shell/src/terminal.rs",
  "skein/crates/skein-shell/src/posix/terminal.rs",
  "skein/crates/skein-shell/src/lib.rs",
  "skein/crates/skein-shell/src/tests.rs",
  "skein/tests/world/src/bin/terminal_fixture.rs",
  "skein/tests/world/tests/suite/end_to_end.rs",
  "skein/docs/design/shell.status.md",
]
effort = "high"
minutes = 60
+++

# skein-shell opens the controlling terminal and takes its foreground

## Goal

A child that a service starts in a process group of its own is a background
job on the person's terminal, which the kernel stops when it reads the
terminal or sets its modes. `temper box` will start its TUI as such a child
(`temper/docs/design/protocol/setup.md`, 7.6): the TUI must open the
controlling terminal and take its foreground, and the box take it back once
the TUI has ended. This task gives skein-shell the two calls, Linux and
macOS alike (`skein/docs/design/shell.md`, 6.3, *The terminal's foreground*).

Acceptance:

1. `skein_shell::open_terminal() -> Result<Fd, Error>` and
   `skein_shell::take_foreground(terminal: Fd) -> Result<(), Error>`, as
   `notes/contracts.md`, section 1 gives them, re-exported at the crate
   root beside `raw_input`; their bodies in `posix/terminal.rs`, shared by
   Linux and Darwin, under that module's `unsafe` expectation and its
   SAFETY comments' form.
   - `open_terminal`: `open("/dev/tty", O_RDWR | O_NOCTTY | O_CLOEXEC)`;
     an error is `Error::Other(errno)`.
   - `take_foreground`: blocks `SIGTTOU` in the calling thread's mask
     (`pthread_sigmask`), calls `tcsetpgrp(terminal, getpgrp())`, restores
     the previous mask whatever the call's result, and gives the call's
     error, `Error::Other(errno)`.
2. **A world story,** in `skein/tests/world/tests/suite/end_to_end.rs`
   beside `input_is_read_a_key_at_a_time`, named
   `a_child_takes_the_terminal_and_gives_it_back`, over
   `world_terminal_fixture` given the argument `foreground`, in
   `Mode::Terminal`:
   - the fixture, given `foreground`, first starts itself as a child in a
     process group of its own with the argument `child`, its standard
     descriptors inherited, and waits for it; the child calls
     `open_terminal`, `take_foreground`, `raw_input(terminal, Keys::Pass)`,
     writes `foreground\n` to its stderr, reads one byte from the terminal
     with a blocking read, restores the modes, writes `child key K\n` (K the
     byte) and exits 0; the parent then calls `take_foreground` on its own
     `open_terminal`'s descriptor, writes `back\n` to its stderr and runs as
     the fixture runs today (a key at a time, `q` ends);
   - the story: `AwaitStderr(b"foreground\n")`, `Write(0, b"a")`,
     `AwaitStderr(b"child key a\n")`, `AwaitStderr(b"back\n")`,
     `Write(0, b"q")`, reads to the end; the fixture exits 0 and its stderr
     holds the three lines in that order.
   - Without `take_foreground` (break it by hand: skip the call in the
     child) the child is stopped at its read and the story fails within its
     bound: say what you saw in the report.
3. **Refusals, natively** (`skein-shell/src/tests.rs`, one test,
   `the_foreground_is_refused_off_a_terminal`): `take_foreground` on a
   descriptor that is not a terminal (a pipe's end) is an error; the
   calling thread's signal mask after the call equals the mask before it.
4. `skein/docs/design/shell.status.md`: delete the **Not yet** bullet naming
   `skein-foreground`. No other design text changes.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`,
  section 1.
- `skein/docs/design/shell.md`, 6.3, *The terminal's foreground*, written by
  this plan's cut; `skein/docs/design/io.md`, 6 (a child's own process
  group).
- The code: `skein/crates/skein-shell/src/terminal.rs` (the public face:
  `raw_input`, `terminal_modes`), `posix/terminal.rs` (`read`, `set`,
  `modes`, its `unsafe` expectation), `lib.rs:99` (the re-export line),
  `src/tests.rs` (its `close` helper). The world: `skein/tests/world/src/bin/terminal_fixture.rs`
  (`main`, `run`), `skein/tests/world/tests/suite/end_to_end.rs:1263-1320`
  (`terminal_fixture()`, `run`, `Act`, `observations`).

## Owns and may change

Owns the two calls, their re-export, the fixture's `foreground` and `child`
modes, the story and the refusal test, the status edit. No other crate
changes: temper's callers come in later tasks.

Options, each decided:

- **Starting the child in the fixture:** the fixture is ordinary Rust; use
  `std::process::Command` with `std::os::unix::process::CommandExt::process_group(0)`
  and `std::env::current_exe()`. If `skein/tests/clippy.toml` refuses
  `std::process`, start it through skein-io's `Spawn` as
  `skein/tests/world/src/hosted.rs` does (every child is in a group of its
  own there), and say so in the report.
- **The child's read:** a blocking `std::io::Read` on a `File` made from the
  descriptor `open_terminal` gave (ordinary Rust in the fixture); close it
  after.
- **macOS:** the calls are POSIX and live in `posix/`; no Darwin-only code.
  The story runs where the world's terminal stories run.
- **The world's bound:** use the bound `run` gives the other terminal
  stories; never raise it.

## Steps

1. The refusal test and the story first, failing; then the calls; the
   narrowest test after each:
   `heavy cargo nextest run -p skein-shell --lib -E 'test(=tests::the_foreground_is_refused_off_a_terminal)'`
   and `heavy cargo nextest run -p skein-world-tests --test suite -E 'test(/a_child_takes_the_terminal_and_gives_it_back/)'`
   (check the package and binary names with `cargo metadata`; a run that
   selects nothing proves nothing).
2. Then the world's terminal stories together
   (`-E 'test(/end_to_end::(input_is_read|the_modes_are_found|a_size_change|a_child_takes)/)'`),
   `heavy cargo nextest run -p skein-shell`, `cargo fmt`,
   (skein has no `old_forms.py`),
   `heavy cargo clippy -p skein-shell -p skein-world-tests --all-targets -- -D warnings`,
   `python3 -I -B tools/citations/citations.py --base origin/main`.
3. The status edit; commit (why: a front end started as a child can stand at
   the person's terminal, and its parent can take the terminal back) with
   the trailer `Look-request: tui-box-front`; push.

## Report

As the implementer's text says, with the rule broken by hand for each test
and the failure seen, and how the fixture starts its child.
