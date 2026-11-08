+++
id = "front-command"
part = "box-front"
kind = "feature"
needs = ["seam-surface", "skein-foreground", "serve-sign-in"]
files = [
  "temper/crates/temper/src/tui_front.rs",
  "temper/crates/temper/src/tui.rs",
  "temper/crates/temper/src/lib.rs",
  "temper/crates/temper/src/commands.rs",
  "temper/crates/temper/src/tests/tui.rs",
  "temper/crates/temper/src/tests/commands.rs",
  "temper/docs/help/reference/commands.md",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/protocol/README.status.md",
]
effort = "high"
minutes = 90
+++

# `temper internal tui --box ADDRESS`: the TUI the box starts in front of it

## Goal

The box spawns its TUI in front as `temper internal tui --box
http://HOST:PORT` (`temper/docs/design/protocol/setup.md`, 7.6, *Its
command*). This task gives the binary that command: it reads the local
link the box hands on descriptor 3, opens the controlling terminal and
takes its foreground, and runs the TUI on that terminal with its front
set, the box's lines on descriptor 4 (`temper/docs/design/client/README.md`,
6.17, *Its start*, *Its terminal*; `notes/contracts.md`, sections 5, 8 and
9). The box's side that spawns it is `box-front` and `binary-front`'s.

Acceptance:

1. **The row:** `commands.rs` gains, after `["internal", "forge-child"]`,
   `Row { words: &["internal", "tui"], who: Who::Spawned, forms: &["--box
   ADDRESS"], line: "Run the TUI in front of the box that started it,
   signed in by the link the box hands it.", arguments: &[Argument {
   shown: "ADDRESS", line: "The box's engine, http://HOST:PORT, on
   loopback." }], settings: &[], notes: "" }`; `COMMANDS`
   grows to 24.
2. **Its parsing:** `lib.rs`' `Command` gains `TuiFront { address:
   Box<[u8]> }`; `configure_row` takes `["internal", "tui"]` with exactly
   `--box ADDRESS`, anything else `invalid arguments`; `run` dispatches it
   to `tui_front::run(&address, &Clock::new())`, its exit code the
   command's, as `Command::Tui`'s.
3. **`tui_front.rs`** (a new module, `mod tui_front;`), `pub(crate) fn
   run(argument: &[u8], clock: &Clock) -> u8`, in order:
   - the address by `tui::address` (a loopback `http://HOST:PORT`; its
     refusals as `tui::Refused::line` prints them, exit 1);
   - descriptor 3 read to its end, at most 4,096 bytes, then closed
     (`skein_shell::close_startup_fd(Fd::new(3))`), through a pure
     `pub(crate) fn handed(bytes: &[u8]) -> Option<Secret>`: none for no
     bytes or more than 4,096; none refuses, `temper internal tui: fd 3:
     expected the local link`, exit 1; the bytes never printed, logged or
     put in a panic;
   - `skein_shell::open_terminal()`, then `take_foreground` on it, then
     `terminal_modes` on it; any failure refuses, `temper internal tui:
     terminal: <the error>`, exit 1, before any mode is set;
   - then as `tui::start` runs bare `temper` from its directory on: the
     directory, the note (`read_note` at the given address), scheme,
     depth and opener as bare `temper`'s; `raw_input(terminal,
     Keys::Pass)`; `before_report(kept.on_abort(terminal, LEAVE))`;
     `open_signals(Watch::TerminationAndWindow)`; the roots; `Settings`
     with `engine` the given address, `link_file` as bare `temper`'s
     (`tui::link_path`), and `front: Some(run::Front { link, lines:
     Fd::new(4) })` (`seam-surface`'s); `Run::new(settings, root, user, seed,
     terminal, terminal, signals)`, the engine's link file attached as
     `local_run` attaches it; `drive`; the modes restored; the address kept
     as bare `temper` keeps it; its exit code as bare `temper`'s.
   - Where `run::Settings` has `tui-launchpad-2`'s `boxed` on your base,
     the front sets it `true`; where it hasn't, nothing (say which).
4. **Tests,** each seen failing first: `src/tests/tui.rs` gains
   `the_handed_link_is_one_to_4096_bytes`, a table: no bytes, none; a
   link, `Some` and equal by `expose`; 4,096 bytes, `Some`; 4,097, none.
   `src/tests/commands.rs`: `command_words` gains `Command::TuiFront { ..
   } => vec!["internal", "tui"]`, and `every_row_parses_from_an_example`
   the example `(&["internal", "tui"], &["internal", "tui", "--box",
   "http://127.0.0.1:8367"])`, and two refusals there (`internal tui`
   alone, `internal tui --box`) asserted `Err`.
   `temper/docs/help/reference/commands.md` regenerated
   (`the_page_commands_is_what_the_table_renders`).
5. **Status:** delete the **Not yet** bullets naming `front-command` in
   `temper/docs/design/protocol/setup.status.md` and
   `temper/docs/design/protocol/README.status.md` (section 4's listing,
   `temper/docs/design/protocol/README.md`, already names the command), and in
   `temper/docs/design/client/README.status.md` the one for 6.11 and
   6.17, `temper internal tui --box` (it names `front-command`; if it
   still names `binary-front`, delete it all the same and say so). No
   other design text.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`,
  sections 1 (`open_terminal`, `take_foreground`), 5 (the front's
  descriptors), 8 (`run::Settings.front`, `run::Front`) and 9 (this
  command).
- `temper/docs/design/protocol/setup.md`, 7.6, *Its command*, *Its
  descriptors*, *The sign-in it hands*; `temper/docs/design/client/README.md`,
  6.17, *Its start*, *Its terminal*, *Signed in by the handed link*, and
  6.11, 6.12, 6.16 (bare `temper`'s start, directory and exit); skein's
  `skein/docs/design/shell.md`, 6.3, *The terminal's foreground*.
- The code: `temper/crates/temper/src/tui.rs`: `Refused` and `line` (24-43),
  `address` (45), `directory` (60), `link_path` (89), `read_note` (95),
  `opener` (105), `scheme` (139), `depth` (151), `run` (163), `start`
  (177-261: the order this command follows), `local_run` (263),
  `keep_address` (272); `lib.rs`: `Command` (75-96), `configure_row`'s
  `internal` arms (155-190), `run`'s dispatch (about 290); `commands.rs`
  (`internal forge-child`'s row, about 280); `temper-client-shell-tui/src/run.rs`,
  `Settings`, `Front`, `Run::new` (`seam-surface`'s names; `tui-seam`, which may run beside you, gives them their bodies); `lib.rs`' `given_secret`
  (a bounded read to the end, about 678).
- Tests: `src/tests/tui.rs` (its tables' shape), `src/tests/commands.rs`
  (`command_words`, `every_row_parses_from_an_example`,
  `each_row_answers_its_help`, `the_page_commands_is_what_the_table_renders`,
  regenerated with `SKEIN_CODEGEN_WRITE=1`).

## Owns and may change

Owns `tui_front.rs`, the row, the `TuiFront` variant and its arms, the
tests above, `commands.md`'s regeneration and the three status edits. In
`tui.rs`, `local_run` and `keep_address` may become `pub(crate)`, and
nothing else there: `tui-launchpad-2` is changing `tui::start` in flight
(`notes/common.md`), so keep this command in a file of its own.

Not yours: `box_command.rs`, `fake_backends.rs` and
`temper/crates/temper/tests/suite/box_kinds.rs` (`box-front` may run beside
you and adds a field there); the box's `--no-tui` and the box's row
(`binary-front`); `temper/docs/help/*.md` but `reference/commands.md`; the
TUI's crates (`seam-surface`'s and `tui-seam`'s).

Options, each decided:

- **Reading descriptor 3:** if skein-shell gives a way to read an
  inherited descriptor to its end, use it; else open `/dev/fd/3`
  (`std::fs::File::open`, both systems), `take(4097)`, `read_to_end`. No
  `unsafe`.
- **Sharing `tui::start`:** if this command's copy of `start`'s tail (the
  signals to the exit) passes about 80 lines, move that tail into one
  `pub(crate)` function of `tui.rs` taking the input and output
  descriptors and an `Option<run::Front>`, called by both, and name it in
  the report; else keep the copy in `tui_front.rs`.
- **A terminal refusal's words:** if `skein_shell::Error` has no
  `Display`, print its `Debug`.

## Steps

1. The failing tests of acceptance 4; then the row, the variant, and
   `tui_front.rs`. The narrowest test after each: `heavy cargo nextest
   run -p temper --lib -E 'test(/tests::tui::/) | test(/tests::commands::/)'`.
2. Regenerate the reference: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest
   run -p temper --lib -E
   'test(=tests::commands::the_page_commands_is_what_the_table_renders)'`,
   then the same without the variable.
3. Runs, through `heavy`, counts in the report: `heavy cargo nextest run
   -p temper --lib`; `heavy cargo nextest run -p temper-docs`. The command
   runs only on a terminal a box hands it: no test here runs it
   (`binary-front`'s focused story does).
4. The status edits; `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper
   --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
5. Commit (why: the box can start the TUI in front of it, signed in by a
   link it hands and standing at the person's terminal), with
   `Look-request: tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says: each test added, the rule broken by hand
for it and the failure seen; the runs' counts; whether `boxed` was set;
whether `tui::start`'s tail was shared.
