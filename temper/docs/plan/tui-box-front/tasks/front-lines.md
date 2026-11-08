+++
id = "front-lines"
part = "box-front"
kind = "feature"
needs = ["box-front"]
files = [
  "temper/crates/temper-box/src/host.rs",
  "temper/crates/temper-box/src/host/front_child.rs",
  "temper/crates/temper-box/src/tests/host.rs",
  "temper/crates/temper-box/src/tests/fixtures.rs",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 120
+++

# The box tells its TUI in front its link and its log, and keeps its terminal quiet

## Goal

While its TUI is in front, the box writes nothing on its terminal: its
printed links go to the front, and every line its terminal would carry
goes to `box.log` in its stead; each line `box.log` gets, and the box's
state (a step restarting, running again, given up), reach the front as the
box's lines on its fd 4 (`temper/docs/design/protocol/setup.md`, 7.6, *The
terminal is the front's* and *The box's lines*). Once the front has ended,
the terminal is the box's again, a failed front named there first. After
`box-front`, which runs the front and holds its fd 4 open, this task fills
it.

Acceptance, by setup.md, 7.6 and `notes/contracts.md`, section 6:

1. **The lines' sink:** at the front's spawn, its fd 4 token becomes a
   `Sink` of `temper_box::front::HELD` (256) entries, ticked as the
   others are (`tick(false, ..)`), closed once the front has ended (in
   `box-front`'s place for closing fd 4). Each line is
   `front::Line::bytes()`; a line the sink does not take (`push` refused)
   is dropped, and nothing else waits on it: no step, log, terminal or
   pipe to serve is held by it.
2. **`link`:** the sink's first line is `link` with `first_link.url`,
   pushed at the spawn. While the front runs, each fresh link serve prints
   (the plain pipe's `Output::Line`) is a `link` line, its line feed
   dropped, and no terminal block; after the front has ended a fresh link
   is printed as 7.2's fresh form (the first block was admitted at the
   start, `box-front`).
3. **`log`:** each item `box.log` gets, as `log_work` hands it to
   `Logs::put` from `box_lines`, is split by `front::logs` and pushed in
   order while the front runs, whether or not `box.log`'s write succeeds.
4. **The terminal's lines in `box.log`:** from the front's spawn until
   `front_ended()`, every item the host would put on its terminal goes to
   `box_lines` instead, each line timed as `box.log`'s are: a
   `Place::Terminal` fact and the failure block with its tail and `Log:`
   (`terminal_fact`, `failure_work`), every `notice` (a record's, a log's,
   `box: fetch: unknown line`, `box: serve: unknown pipe line`, `box:
   serve: pipe line too long`, `REFUSAL`), and a step's lines when its log
   is not written (`log_work`'s echo, untimed today, timed by
   `Formatter::notice` here). What gates on the terminal's room
   (`terminal.takes`, `notice_room`, `deferred`) gates on nothing there
   meanwhile. A `box.log` not written leaves its lines to the front's
   `log` lines alone until the front has ended; after it, 7.2's fallback
   to the terminal holds.
5. **The box's state:** at a `Fact::Restarting`'s placement,
   `front::state` gives `restarting STEP`, pushed then, and the step is
   marked; at that step's next `Event::Spawned`, `running STEP`, and the
   mark cleared; at `Fact::GivingUp`, `gave-up STEP`. Their `log` lines
   follow as `box.log` takes them.
6. **The terminal after the front:** once `front_ended()`, every item goes
   to the terminal as without a front, and nothing `box.log` already
   holds is written there again. A front that ended other than by
   exiting 0 is named there first: `HH:MM:SSZ tui exited <code>` (`tui
   killed by signal <n>`, `tui did not start`), its tail under it and
   `Log: <data>/box/logs/tui.log`, built as `failure_work` builds a
   step's (its two read pipes settled first, each line timed); a front
   that did not start has no tail.
7. **Tests** of *Steps*, each seen failing first.
8. **Status:** delete the **Not yet** bullet naming `front-lines` in
   `temper/docs/design/protocol/setup.status.md`. No other design text.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`,
  section 6 (the lines; the TUI reads them as `BoxLine`, section 7).
- `temper/docs/design/protocol/setup.md`, 7.6, *The terminal is the
  front's*, *The box's lines*, *The terminal after the front*; 7.2, *Its
  terminal*, *Its own log*, *Its logs* (a log not written), *A failure's
  tail*.
- `temper/crates/temper-box/src/front.rs` (`box-lines`): `Line`, `HELD`,
  `logs`, `state`.
- `box-front`'s host: the front's state (in `host.rs` or
  `host/front_child.rs`), its fd 4 token, `front_ended()`, its tail and
  `Logs::front_step()`; read its report for where each lives.
- The code (`temper/crates/temper-box/src/host.rs`, lines as on main
  before `box-front`): `terminal_fact` (685), `tail_candidate`,
  `failure_ready` and `failure_work` (705-750), `children` (the plain
  pipe's `Output::Line` block, about 1060-1070; `Output::Refused`),
  `start_work` (1157), `notice_room`, `notice` and `deferred_work`
  (1186-1218), `log_work` (1219-1260), `take_control_line` (1300-1330),
  `work_pending` (1660-1760); `sink.rs`, `Sink` (`push`, `tick`, `up`,
  `closed`).
- Tests: `src/tests/host.rs`, `box-front`'s `front_harness`, `read_fd`,
  `the_fronts_end_stops_the_box`; `the_terminal_keeps_errors_alone_and_box_log_the_rest`
  (291), `an_unwritable_box_log_is_named_once_and_its_lines_follow` (357),
  `a_restarted_serve_has_no_stdin_and_its_link_is_fresh` (154),
  `serves_pipe_lines_are_bounded_named_and_answered` (676), `ask`,
  `write_stdout`, `finish_child`; `host.rs`'s test-only `pressure`
  module.

## Owns and may change

Owns what the front is told on fd 4 and the routing of the terminal's
items while it runs, in temper-box's host, and the tests below. Not
`front.rs` (`box-lines`'; if a function it lacks is needed, write it in
the host and say so), not `logs.rs`, `machine.rs` or `terminal.rs` beyond a
call. Not the temper crate: `binary-front` and `front-command` may run
beside this task there (`box_command.rs`, `commands.rs`, `lib.rs`,
`tui.rs`, `tui_front.rs`, `fake_backends.rs`, its tests and help pages
are theirs).

Options, each decided:

- **Where a line is routed:** one host function that every terminal push
  goes through (`fn say(&mut self, item: Item)` or the like) deciding
  front or terminal; if a call site can't reach it, route it in place and
  name it in the report.
- **A test-only reading of the sink's held count:** add it to the
  `pressure` module (`pressure::front_held(&Service) -> u32`).
- **`box-front`'s rows:** extend `the_fronts_end_stops_the_box`'s rows
  with their terminal bytes (below) rather than a new test.

## Steps

1. The failing tests, in `src/tests/host.rs`:
   - `the_front_is_told_the_link_first_then_each_box_log_line`: on
     `front_harness`, read the front's fd 4 (`read_fd(.., 4, ..)`): first
     `link http://local/first\n`; then, line for line, `log ` and each
     line of `box.log` (`h.log()`), in order; serve writes
     `http://next\n` on its stdout (`write_stdout`): fd 4 gains `link
     http://next\n`, the box's stdout and stderr stay empty, and no log
     holds either URL.
   - `the_terminals_lines_go_to_box_log_and_the_front_while_it_runs`, a
     table: a long step that writes `boom` on its stderr and exits 1:
     `box.log` holds its restart line, `    <step>: boom` and `    Log:
     <path>`, each timed; fd 4 holds `restarting <step>`, then those
     three as `log` lines, then `running <step>` at its respawn; serve's
     unknown pipe line (`ask(b"bogus\n")`): `box: serve: unknown pipe
     line` in `box.log` and on fd 4; a step's log not written
     (`refuse_log`): the notice and the step's next line in `box.log`,
     timed, and on fd 4; a give up: `gave-up <step>` and its lines on fd
     4. Every row: the box's stdout and stderr hold nothing while the
     front runs. A last row: after the front exits 0, a step's exit is on
     the terminal alone, as 7.2 writes it, and nothing written before is
     repeated there.
   - `the_fronts_end_stops_the_box` (`box-front`'s) rows gain their
     terminal: exit 0, nothing; exit 3 after `boom`, exactly
     `HH:MM:SSZ tui exited 3\nHH:MM:SSZ     tui: boom\nHH:MM:SSZ     Log:
     /data/box/logs/tui.log\n`; killed by signal 9, its line and `Log:`
     with no tail; did not start, `tui did not start` alone.
   - `a_front_that_reads_nothing_holds_256_lines`: fd 4 never read, the
     simulator's pipe small; serve sends 300 unknown pipe lines: the
     sink holds at most 256 (`pressure::front_held`), `box.log` holds all
     300, `pressure::bounded` holds, serve's replies and spawns still
     proceed; the front then exits 0 and the box exits 0, with nothing
     open.
   Break each rule by hand once and see its test fail; say which in the
   report.
2. The host, in the order of acceptance 1 to 6; the narrowest test after
   each: `heavy cargo nextest run -p temper-box --lib -E
   'test(/tests::host::/)'`.
3. Runs, through `heavy`, counts in the report: `heavy cargo nextest run
   -p temper-box`; `heavy cargo nextest run -p temper --test suite -E
   'test(/box_kinds::/) | test(/box_listeners::/)'` (no front there: they
   must stay green).
4. The status edit; `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-box
   --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
5. Commit (why: the TUI in front shows the box's link and log, and the
   terminal under it stays the TUI's), with `Look-request: tui-box-front`
   directly above the other trailers; push.

## Report

As the implementer's text says: each test added or extended, the rule
broken by hand for it and the failure seen; the runs' counts; the routing
function and any call site routed in place.
