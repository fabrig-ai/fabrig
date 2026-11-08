+++
id = "tui-start"
part = "tui"
kind = "migration"
needs = ["shell-local"]
files = [
  "temper/crates/temper-client-shell-tui/src/start.rs",
  "temper/crates/temper-client-shell-tui/src/start/**",
  "temper/crates/temper-client-shell-tui/src/lib.rs",
  "temper/crates/temper-client-shell-tui/Cargo.toml",
  "temper/crates/temper-client-shell-terminal/src/address.rs",
  "temper/crates/temper-client-shell-terminal/src/lib.rs",
  "temper/crates/temper/src/tui.rs",
  "temper/crates/temper/src/open.rs",
  "temper/crates/temper/src/tests/tui.rs",
  "Cargo.lock",
]
effort = "high"
minutes = 90
+++

# The TUI's start moves into its shell, for both binaries

## Goal

The TUI's arguments, address and link-file helpers and its start, today
`temper/crates/temper/src/tui.rs`, move into `temper-client-shell-tui` as the
module `start`, and the parse of an engine's address or a sign-in link, today
`open.rs`' `target`, into `temper-client-shell-terminal` as `address`; each line
the start prints names the binary it is given
(`temper/docs/design/client/README.md`, section 2, *The clients' start, one for
two binaries*; 6.11, *In the deployment's client*; 6.12). The box's bare `temper`
and `temper ADDRESS` run it named `temper`, every line and file as today; the
deployment's client (`client-table`) runs it named `temper-client`. This is the
small move the request asks for; expect a catch-up with `tui-box-front`
(`notes/common.md`).

Acceptance:

1. **`temper_client_shell_terminal::address`** as `notes/contracts.md`, section
   6, `target`'s body verbatim; `lib.rs` gains `pub mod address;`. `temper`'s
   `open.rs` reads it: its own `Target` and `target` go, its `Refused` keeps its
   six variants and `line`, `impl From<address::Refused> for Refused` mapping the
   two, and `temper open`'s lines are unchanged.
2. **`temper_client_shell_tui::start`** as contracts, section 7: `tui.rs`' items
   moved, bodies verbatim but for the name (`Refused::line(name)`,
   `terminal_refusal(name, given)`, `run(name, argument, clock)`) and the paths:
   `temper_shell::link_file`, `temper_shell::directories::make_private`,
   `temper_client_shell_terminal::address::target`, a private bytes-to-`Path`
   helper for `crate::path`, `crate::` for this crate's own `run`, `limits`,
   `opener`. `terminal_refusal(name, false)` is `"{name}: no command; {name}
   --help lists them\n"`, formatted here. Module doc citing 6.11 and 6.12.
   `lib.rs` gains `pub mod start;`; `Cargo.toml` gains `temper-shell` and
   `skein-fail-stop`, and dev `skein-scratch`.
3. **`temper` runs it:** `tui.rs` keeps `pub(crate) fn run(argument:
   Option<Box<[u8]>>, clock: &Clock) -> u8`, calling `start::run("temper",
   argument.as_deref(), clock)`, and `pub(crate) use` of the moved items `temper`
   still reads; `lib.rs`' dispatch and `temper`'s `record` feature are unchanged.
4. **The tests move with what they test:** of `temper/crates/temper/src/tests/tui.rs`,
   every test reading only the moved helpers moves to
   `temper-client-shell-tui/src/start/tests.rs` (declared from `start.rs`),
   under its name, `crate::link_file` and `crate::directories::resolve` read as
   `temper_shell`'s (the link-path test takes `Paths.data`);
   `bare_temper_and_temper_address_parse_and_refuse` stays in `temper`, its
   assertions unchanged but `why.line("temper")` and `terminal_refusal("temper",
   …)`.
5. **The lines name the binary,** seen failing first: `start/tests.rs` gains
   `the_lines_name_the_binary`, a table of each `Refused` and `terminal_refusal`
   with `true` and `false`, for `temper` (today's lines, as the box's test has
   them) and `temper-client` (6.11, *In the deployment's client*: `temper-client:
   the TUI needs a terminal on stdin and stdout`, `… run temper-client
   http://HOST:PORT and paste it`, `temper-client: no command; temper-client
   --help lists them`).

## Inputs

- `temper/docs/plan/client-binary/notes/common.md`, with *When `tui-box-front`
  has landed on your base*; `notes/contracts.md`, sections 3, 6 and 7.
- Design: `temper/docs/design/client/README.md`, section 2 (`temper-client-shell-tui`'s
  modules, *The clients' start, one for two binaries*), 6.11 (the exit table, *In
  the deployment's client*), 6.12; 6.1 (`temper open`'s address).
- The code: `temper/crates/temper/src/tui.rs` (all of it), `open.rs` (`Target`,
  `Refused`, `target`, `start`), `lib.rs` (`path`, `Command::Tui`'s dispatch,
  `type EnvironmentEntry`), `src/tests/tui.rs`, `src/tests.rs`;
  `temper/crates/temper-client-shell-tui/src/lib.rs`, `run.rs` (`Run`, `Settings`,
  `with_local_link`), `opener.rs`, `limits.rs`, its `Cargo.toml` (its `record`
  feature); `temper/crates/temper-client-shell-terminal/src/lib.rs`;
  `temper-shell`'s `link_file` and `directories` as `shell-local` left them.

## Owns and may change

Owns the two new modules and their `lib.rs` lines, `temper-client-shell-tui`'s
`Cargo.toml` lines, `temper`'s `tui.rs`, `open.rs` and `src/tests/tui.rs`.

Not yours: `temper-client-shell-tui`'s `host.rs`, `run.rs`, `limits.rs`,
`tests.rs` and `journey/**` (the in-flight `tui-box-front` changes them);
`temper`'s `lib.rs`, `commands.rs`, `box_command.rs`.

Options, each decided:

- **If `temper/crates/temper/src/tui_front.rs` is on your base** (`tui-box-front`
  landed): its helpers that aren't the command itself (`handed`, and a start
  tail `tui.rs` shares with it) move into `start` beside the others, `pub`; the
  command stays in `temper`, reading them through `temper`'s `tui` names or
  `start::`, with its tests' assertions unchanged; say what moved. If it isn't,
  touch nothing of it.
- **`run`'s pass by value:** `temper`'s `tui::run` keeps its signature and its
  `#[expect(clippy::needless_pass_by_value, …)]`; `start::run` takes
  `Option<&[u8]>` and needs none.
- **`EnvironmentEntry`:** `start`'s own alias, the same tuple as `temper`'s; no
  conversion is needed.
- **A moved test that reads `temper`'s `Directories`** stays in `temper` and
  says why.

## Steps

1. The failing test of acceptance 5 against the new signatures; then the moves.
   `heavy cargo check -p temper-client-shell-terminal -p temper-client-shell-tui
   -p temper --tests` until it builds; then once more with `--features
   temper/record` (`heavy cargo check -p temper --features record`), as
   `temper-client-journeys` builds it.
2. `heavy cargo nextest run -p temper-client-shell-tui --lib -E
   'test(/start::/)'`; `heavy cargo nextest run -p temper --lib -E
   'test(/tests::tui::/)'`.
3. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-shell-terminal -p temper-client-shell-tui -p temper
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-shell-terminal -p temper-client-shell-tui -p temper`; `heavy
   cargo check -p temper-client-world -p temper-client-journeys -p
   temper-one-process-world -p temper-real-loop-world --tests`.
4. Commit ("The TUI's start moves into temper-client-shell-tui, its lines naming
   their binary") and push.

## Report

`Done:`, `Failed:` or `Question:` first, then the files, which tests moved and
the counts in each package, whether `tui_front.rs` was on your base and what you
did with it, the runs and their counts, the rule the new test bit on, anything
left.
