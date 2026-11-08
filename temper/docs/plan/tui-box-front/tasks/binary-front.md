+++
id = "binary-front"
part = "box-front"
kind = "feature"
needs = ["box-front", "front-command", "tui-seam", "serve-sign-in", "skein-foreground"]
files = [
  "temper/crates/temper/src/box_command.rs",
  "temper/crates/temper/src/fake_backends.rs",
  "temper/crates/temper/src/lib.rs",
  "temper/crates/temper/src/commands.rs",
  "temper/crates/temper/src/tests/box_kinds.rs",
  "temper/crates/temper/src/tests/commands.rs",
  "temper/crates/temper/src/fake_backends/tests.rs",
  "temper/crates/temper/tests/suite/main.rs",
  "temper/crates/temper/tests/suite/box_front.rs",
  "temper/crates/temper/tests/suite/startup.rs",
  "temper/docs/help/reference/commands.md",
  "temper/docs/design/protocol/setup.status.md",
  "temper/docs/design/protocol/README.status.md",
]
effort = "high"
minutes = 140
+++

# `temper box` in a terminal runs its TUI in front, and `--no-tui` runs it as before

## Goal

The binary's side of the box's front (`temper/docs/design/protocol/setup.md`,
7.6, *When*, *The sign-in it hands*, *The terminal after the front*,
*Where it lives*): `temper box` and `temper box --fake-backends` take
`--no-tui`; in a terminal, with `--echo none` and a first link, the box
draws the local link, opens its controlling terminal, and gives temper-box
the front (`box-front`'s `Settings.front`), which runs `temper internal tui
--box` (`front-command`); once the front has ended the box takes the
terminal's foreground back. Otherwise the box runs as today, byte for byte.
One focused story shows it whole on a pseudo-terminal.

Acceptance, by setup.md, 7.6 and `notes/contracts.md`, sections 1, 4, 5
and 9 (the front's side: section 8 and `temper/docs/design/client/README.md`,
6.17):

1. **`--no-tui`:** the box's row (`commands.rs`) gains `[--no-tui]` at the
   end of both forms, an argument `--no-tui` ("Run the box with no TUI in
   front, as on a pipe."), and in its notes: in a terminal, with `--echo
   none`, the box runs the TUI in front, signed in, its link shown there;
   the block, `--fake-backends`' lines and the errors go to the box's log
   while it runs; quitting the TUI stops the box. `lib.rs`:
   `Command::Box { directories, echo, no_tui: bool }` and
   `Command::BoxFakeBackends { directories, bundle, echo, no_tui: bool }`;
   `configure_row`'s `["box"]` branch takes one `--no-tui` out of the
   arguments wherever it stands, before the echo and fake-backends
   parsing (a second is `invalid arguments`); the dispatch passes it.
2. **The front's condition:** `box_command::run(directories, echo,
   no_tui)` and `fake_backends::run(source, bundle, echo, no_tui)` pass it
   to `run_configuration(.., echo, no_tui, opening)`, which, after
   `first_link`, decides by a pure `pub(crate) fn front_wanted(no_tui:
   bool, echo: Echo, terminals: bool, first_link: bool) -> bool` (all of:
   no `--no-tui`, `Echo::None`, fds 0 and 1 terminals by
   `skein_shell::terminal_modes`, a first link), and then
   `skein_shell::open_terminal()` succeeding: a front. Otherwise
   `Settings.front` is `None`, `FirstLink.local` is `None`, and nothing
   else changes. The terminal is opened only when `front_wanted` holds.
3. **The front's settings:** `pub(crate) fn front(named, first: &mut
   FirstLink) -> Result<Front, String>`: the local link's secret by
   `skein_shell::crypto_seed()` (an error refuses as `first_link`'s does),
   `first.local = Some(secret)`; `listener` the clients' listener
   (`super::address(&named.clients)`, as `first_link` finds it, on
   loopback); its URL by `temper_service::clients::first_link_url(listener,
   None, &secret)`, never the configured public origin, past 4,096 bytes
   refused as `first_link` refuses its own; `Front { arguments: [internal,
   tui, --box, http://LISTENER], link: Secret::new(url) }`, `LISTENER` the
   listener as `SocketAddr` writes it (`[::1]:8367` for IPv6), so that the
   URL's host is the front's address (`notes/contracts.md`, section 5).
4. **The foreground taken back:** `run_configuration` drives, with a front,
   a `struct Fronted<'a> { service: &'a mut Service, terminal: Option<Fd>
   }` that implements `skein_shell::Host` by delegating each method to the
   service (not `skein_shell::Delegate`, whose `drain` can't be
   overridden), and whose `drain` calls the service's, then, the first
   turn `service.front_ended()` is true, `skein_shell::take_foreground` on
   the terminal (its error ignored, as 7.6 says) and closes the descriptor
   (`close_startup_fd`). A terminal still held when `drive` returns, or
   on a refusal after it was opened, is closed (it joins `owned`).
5. **Callers:** `tests/suite/startup.rs`' `temper::run(Command::Box { ..
   })` calls set `no_tui: true` (they run in the test's own process,
   whose fds 0 and 1 may be a developer's terminal).
6. **Tests,** each seen failing first:
   - `src/tests/box_kinds.rs`: `the_front_runs_only_in_a_terminal_with_echo_none_and_a_first_link`,
     a table over `front_wanted`: all held, true; each of the four off,
     false. `the_front_hands_a_local_link_at_the_clients_listener`,
     beside `first_link_draws_a_secret_and_uses_serves_url`: the local
     secret differs from the printed one, the front's link is
     `first_link_url(listener, None, ..)` of it, the arguments `internal tui
     --box http://LISTENER`, the link's host that address; rows for an
     IPv4 and an IPv6 listener, and one with a public origin set, where
     the printed link names the public origin and the front's address and
     link stay the listener's.
   - `src/tests/commands.rs`: the box's two examples end in `--no-tui`
     (`every_row_parses_from_an_example` wants one example per form), and
     its closing asserts check `no_tui: true` for both and `false` for
     `box` and `box --fake-backends` alone; `command_words`' box arm is
     unchanged.
   - `src/fake_backends/tests.rs`:
     `fake_backends_is_an_additive_box_only_switch` gains rows with
     `--no-tui` before, between and after the fake box's words (true) and
     twice (false).
   - **The focused story,** `tests/suite/box_front.rs` (`mod box_front;` in
     `main.rs`), `a_box_in_a_terminal_runs_its_tui_in_front_and_stops_with_it`:
     a fresh box (`box_link::command`'s directories, its runtime and
     `PATH`, included as `box_listeners.rs` includes it, and
     `XDG_STATE_HOME` a directory of the scratch's) started on a
     pseudo-terminal in a session of its own (`skein_shell::keeper::start_binary(&mut
     spawn, true)`, as `temper/tools/journeys/src/tui_again/terminal.rs`
     starts the TUI, its master resized to 80 by 24 and read on one
     kernel, never left full); waits for the engine's link file
     (`box_link::link_path`) to hold a link, which serve writes only once
     the handed local link is spent, so the front has signed in; waits for
     the master's modes to be raw (`terminal_modes`: no lines, no echo, no
     signals); writes one byte `0x03`; drains until the box exits: exit 0;
     `box.log` holds `tui exited 0`, `Stopping` and `Stopped` in that
     order; no file of `data/box/logs/` holds `secret=`; the
     pseudo-terminal's bytes never hold `Open this link in a browser`; on
     Linux, `support::parent::Guard::observing_test` finds every process
     ended, as `box_listeners.rs`' `stop` does. Each wait within
     `support::WITHIN`, the box's exit watched; a box that exits early
     fails the story with its `box.log`.
7. **Status:** delete the **Not yet** bullets naming `binary-front` in
   `temper/docs/design/protocol/setup.status.md` and
   `temper/docs/design/protocol/README.status.md` (section 4's listing,
   `temper/docs/design/protocol/README.md`, already names `--no-tui`). No
   other design text.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`,
  sections 1, 4, 5, 8 and 9.
- `temper/docs/design/protocol/setup.md`, 7.6 whole, 7.2 *The sign-in
  block* (the printed link's origin, which the front's is not);
  `temper/docs/design/client/README.md`, 6.17, *Its
  start* and *Its terminal* (what the front does with its terminal);
  skein's `skein/docs/design/shell.md`, 6.3, *The terminal's foreground*,
  and section 12 (the drain hook).
- `box-front`'s `temper_box::host::{Front, FirstLink.local,
  Settings.front, Service::front_ended}`; `front-command`'s `temper
  internal tui --box` (its row and `tui_front.rs`).
- The code: `temper/crates/temper/src/box_command.rs`: `run` (50),
  `Opening` (57), `run_configuration` (63-138: `owned`, the `Settings`
  literal at 95, `drive` at 136), `first_link` (143-166); `lib.rs`:
  `Command` (75-96), `configure_row`'s box branch (191-212), dispatch
  (248-253); `fake_backends.rs`: `configure` (58-85), `run` (86-105);
  `commands.rs`: the box's row (103-125); `printed.rs`, `running_word`;
  `temper-service/src/clients/printed.rs`, `link_public` (no public origin:
  the listener's own `link`); `skein-shell/src/drive.rs`, `Host` and
  `Delegate`.
- Tests: `src/tests/box_kinds.rs` (`first_link_*`, 1-95);
  `src/tests/commands.rs` (`every_row_parses_from_an_example` 44-96);
  `src/fake_backends/tests.rs` (208-224); `tests/suite/box_listeners.rs`
  (`scratch`, `stop`, `kept_link`), `tests/fake_backends/box_link.rs`
  (`command`, `link_path`), `tests/support/parent.rs`;
  `temper/tools/journeys/src/tui_again/terminal.rs` (a pseudo-terminal on
  one kernel: start, resize, read, write, drop).

## Owns and may change

Owns the box's `--no-tui`, the front's condition and settings, `Fronted`,
the tests above, the focused story and the status edit; the box's row and
`commands.md`'s regeneration (`the_page_commands_is_what_the_table_renders`,
`SKEIN_CODEGEN_WRITE=1`). Reader edits: `tests/suite/startup.rs`' three
`Command::Box` literals (`no_tui: true`); every other `Command::Box { .. }`
or `BoxFakeBackends { .. }` pattern the compiler names, the field alone.

Not yours: temper-box (`box-front`'s and `front-lines`'; `front-lines` may
run beside you there, and nothing of yours touches it); `tui.rs` and
`tui_front.rs` (`front-command`'s, before you); the help pages
`temper/docs/help/box.md` and `tui.md` (`box-help`, after you).

Options, each decided:

- **Where `--no-tui` is taken out:** in `configure_row` before the echo
  and fake parsing, so that `fake_backends::configure` never sees it; if
  the directories' resolver refuses the word first, take it out before
  `directories::resolve` and say so.
- **The story's environment:** this process's, with `box_link::command`'s
  own additions (`get_envs`), `XDG_STATE_HOME`, and `TERM` set to
  `xterm-256color` when absent; `TEMPER_ECHO` removed.
- **A story that can't have a pseudo-terminal** (`start_binary` refused):
  it fails, naming the error; the focused suite's `box_link_form` already
  opens `/dev/ptmx`. Don't skip it.

## Steps

1. The failing tests of acceptance 6 but the story; then `--no-tui`, the
   condition, the front's settings, `Fronted`. The narrowest test after
   each: `heavy cargo nextest run -p temper --lib -E
   'test(/tests::box_kinds::/) | test(/tests::commands::/) |
   test(/fake_backends::tests::/)'`.
2. The story; run it alone: `heavy cargo nextest run -p temper --test
   suite -E
   'test(=box_front::a_box_in_a_terminal_runs_its_tui_in_front_and_stops_with_it)'`.
   Break the foreground's return by hand (skip `take_foreground`) and say
   what the story shows, if anything; break `--no-tui` by hand and see the
   parsing rows fail.
3. Regenerate `commands.md` (`SKEIN_CODEGEN_WRITE=1` on
   `the_page_commands_is_what_the_table_renders`), then without it.
4. Runs, through `heavy`, counts in the report: `heavy cargo nextest run -p
   temper --lib`; `heavy cargo nextest run -p temper --test suite -E
   'test(/box_front::/) | test(/box_listeners::/) | test(/box_kinds::/) |
   test(/startup::/)'`; `heavy cargo nextest run -p temper-docs`;
   `heavy cargo check -p temper --tests` (the process tier's box stories
   start the box on pipes and need no change: confirm they compile).
5. The status edit; `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper
   --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
6. Commit (why: in a terminal `temper box` runs its TUI in front, and
   `--no-tui` keeps the box as it was), with `Look-request: tui-box-front`
   directly above the other trailers; push.

## Report

As the implementer's text says: each test added or extended, the rule
broken by hand for it and the failure seen; the story's runtime and what
breaking the foreground's return showed; the runs' counts; any option
taken otherwise than its default.
