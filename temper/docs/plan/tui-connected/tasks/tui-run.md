+++
id = "tui-run"
part = "tui"
kind = "feature"
needs = ["tui-host", "screen-terminal"]
files = [
  "temper/crates/temper-client-shell-tui/src/run.rs",
  "temper/crates/temper-client-shell-tui/src/run/**",
  "temper/crates/temper-client-shell-tui/src/lib.rs",
  "temper/crates/temper-client-shell-tui/src/limits.rs",
  "temper/crates/temper-client-shell-tui/Cargo.toml",
  "temper/crates/temper-client-shell-tui/src/tests.rs",
  "temper/crates/temper-client-shell-tui/tests/**",
  "Cargo.lock",
]
effort = "high"
minutes = 170
+++

# temper-client-shell-tui: the TUI's loop

## Goal

The TUI's one host on skein's loop: the seam's host, the native transport
and its files, skein's full-screen terminal on stdin and stdout, and the
termination and window signals, composed as `temper open`'s terminal
composes its parts. It reports what it hears to the host, carries the
host's duties, gives the terminal the host's frames, and closes on every
way out. The binary (`tui-command`) drives it.

Acceptance:

1. `run` is `temper/docs/plan/tui-connected/notes/tui.md`, section 5's
   `run::Settings`, `run::Run` (`new`, `signed_in`, `refused`) and its
   `skein_shell::Host`; `limits` gains `terminal(limits, depth) ->
   skein_ui_screen_terminal::Settings` (`interval`, `depth`, `most`,
   `paste` from the limits), and `valid` also checks
   `Terminal::operations(&terminal(limits, Depth::True))` is `Some`. The
   crate's manifest gains `skein-io` and `skein-shell`
   (`notes/contracts.md`, section 1; `tui-host` gave it the rest).
2. `Run::new` makes the host (`host::Settings { limits }`, the seed, time
   zero), the native shell (`jig_client_shell_native::Shell::new` with the
   engine, the directory, `limits.native.connections` and `.io`, adopted
   on `root` with `user` and the seed), the terminal
   (`Terminal::new(terminal(&limits, depth), size)`, adopting `input` and
   `output`), and an io of its own adopting `signals`; its operations are
   the three's sum. It gives back the descriptor io refused.
3. **Each iteration** does what `temper/docs/design/client/README.md`,
   6.16, says, in its order: the signals' io resumed, completed and fired
   as `temper open`'s terminal does; the native shell iterated; the
   terminal iterated; the host's clocks; once the shell has loaded,
   `Report::Start` with the terminal's size, the settings' scheme and
   address, the records and the credential; then `Kept` when the shell's
   `kept()` answers, each of the shell's events as `Heard`, `Resize` when
   the terminal's size differs from the last reported, each key the
   terminal holds as `Key`, and `Due` when the host's deadline has passed;
   each report's duties carried at once (`asked` to `shell.down`, `keep`
   to `shell.keep`, `address` kept for `signed_in`, `open` ignored with a
   debug assertion that it is none, `close` closing); then, once any report
   was given since the last frame, `terminal.draw(host.frame())`; then the
   submissions published.
4. **Routing:** a completion the terminal owns (`Terminal::owns`) to it;
   one tagged with bit 61 to the signals' io, the tag removed; every other
   to the native shell. The signals' io's submissions are tagged with bit
   61; the terminal's and the shell's pass as they are.
5. **Closing,** once, by the host's `close`, the terminal's `ended()`, or
   the signals' `Shutdown` or `Failed`: the terminal closed
   (`Terminal::close`), the native shell closed, the signals' entity
   aborted; no key is taken and no frame drawn after it, while the shell's
   events and `kept()` are still reported (`Heard`, `Kept`). `io::Event::Resized`
   is given to the terminal (`Terminal::resized`). The exit is
   `Exit::Code(0)` once the terminal is `closed()`, the shell `is_empty()`
   and every queue and the io drained; `Exit::Code(1)` the same way when
   `refused()`.
6. **Refused:** the native shell idle and never loaded (the condition
   `temper open`'s native client checks, `client.rs`, `iterate`'s end) sets
   `refused` and closes.
7. Tests (Steps, 3), each named in the report with its time.

## Inputs

- `temper/docs/plan/tui-connected/notes/common.md`.
- `temper/docs/plan/tui-connected/notes/contracts.md`, sections 1, 3
  (`Terminal`, `Settings`, `Depth`, the tag 1) and 6: they bind.
- `temper/docs/plan/tui-connected/notes/tui.md`, section 5.
- `temper/docs/design/client/README.md`, 6.16 (the loop, closing, the
  refused directory) and 6.13 (the reports and duties).
- `skein/docs/design/ui-screen.md`, section 8 (the terminal: entering,
  keys, frames, resize, closing).
- The models: `temper/crates/temper-client-shell-terminal/src/terminal/run.rs`
  (the whole composition: io limits for signals, tags, `complete`,
  `publish`, the `Host` impl) and `temper/crates/temper-client-shell-terminal/src/client.rs`
  (the shell's `loaded`, `kept`, `events`, the refused directory's
  condition).
- For the machine story: `skein/tests/ui-screen` (`screen-terminal`'s world:
  pipes on the simulator, keys fed, output drained), `skein/tests/io/src/private.rs`
  (a `skein_world::World` whose `Machine` is `skein_fake_machine`'s files,
  answering a private root) and `jig/tests/one-process/src/world.rs`
  (`client`: a native shell's process with its root and signal source).

## Owns and may change

**Owned:** the `files` above. `src/lib.rs` gains `pub mod run;`;
`limits.rs` gains `terminal` and the line in `valid`; nothing else of
`host` or `limits` changes.

**Readers and neighbours:** none: the binary is `tui-command`'s. The
manifest's dev-dependencies for the story (`skein-sim`, `skein-world` or
`skein-sim-world`, `skein-fake-machine`, as the models' manifests name
them) are yours.

**Options, each with its default:**

- The signals' io limits: `temper open`'s terminal's (`run.rs`, `new`),
  with `intake`, `receive` and `output` at its read size.
- `Resize` before `Start`: none; the start carries the size then.
- The terminal's keys while a report's duties close it: the rest dropped.
- The machine story's harness: the default is a `skein_world::World`
  hosting the run as `skein/tests/io/src/private.rs` hosts its own, its
  machine `skein_fake_machine`'s files with the TUI's directory laid
  empty; if driving the run directly on `skein_sim::Sim` with the
  simulator's file calls answered by `skein_fake_machine::Machine` is
  simpler, do that and say so. If either needs a harness past about 200
  lines, keep the unit tests of Steps 3, report the story under `## Left`
  with what it lacked, and the task is done without it.

## Steps

1. `limits::terminal` and `valid`'s line.
2. `run`: `Settings`, `Run::new`, the iteration of acceptance 3, routing,
   closing, refused, the `Host` impl (`work_pending`, the deadlines,
   `is_empty`, `exit`, `operations`) as `run.rs` of `temper open` has them,
   with the terminal's and the host's beside.
3. Tests:

   ```rust
   // src/tests.rs, beside tui-host's rows
   #[test]
   fn the_run_is_made_from_the_shipped_limits() {
       // limits::valid(&limits::tui()); Terminal::operations(&limits::terminal(&tui(), Depth::Indexed))
       // is Some; as the binary makes them, so the gate meets a refusal before a terminal does
   }

   // tests/suite/run.rs: the run on skein's simulated machine, no engine reached
   #[test]
   fn every_way_out_leaves_the_terminal_as_it_was_found() {
       // rows: the input's end (the peer closes stdin), Ctrl+C's byte (0x03), SIGTERM on the
       // signal source; each from a fresh run with an empty directory and no credential:
       // the output starts with ENTER, holds "Sign in this terminal to 127.0.0.1:8367"
       // once Start has drawn, ends with LEAVE and nothing after it; the run's exit is Code(0)
   }
   ```

   Bite: skip `Terminal::close` on the signals' `Shutdown` once; see the
   SIGTERM row fail (no `LEAVE`), and restore.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-shell-tui --all-targets -- -D warnings`; `heavy
   cargo nextest run -p temper-client-shell-tui`. Commit
   ("temper-client-shell-tui: the TUI's loop over the native transport and
   the screen terminal"), push.

## Report

As the implementer's text says, and: the machine story's harness (or what
it lacked), each row's time, and how the run behaves when the shell's
files refuse the directory before `Start`.
