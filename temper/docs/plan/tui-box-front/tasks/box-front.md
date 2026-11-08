+++
id = "box-front"
part = "box-front"
kind = "feature"
needs = ["box-lines"]
files = [
  "temper/crates/temper-box/src/host.rs",
  "temper/crates/temper-box/src/host/front_child.rs",
  "temper/crates/temper-box/src/logs.rs",
  "temper/crates/temper-box/src/tests/host.rs",
  "temper/crates/temper-box/src/tests/fixtures.rs",
  "temper/crates/temper-box/src/tests/logs.rs",
  "temper/crates/temper-box/tests/real_loop.rs",
  "temper/crates/temper/src/box_command.rs",
  "temper/crates/temper/tests/suite/box_kinds.rs",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 150
+++

# temper-box's host runs the TUI in front: its start, its descriptors and its end

## Goal

`temper box` in a terminal runs the TUI in front of its steps, a child of
the box's that is no step (`temper/docs/design/protocol/setup.md`, 7.6).
This task gives temper-box's host the front's process: its settings, its
spawn in the first block's place with its four descriptors, the local
link's URL on its fd 3, both secrets on serve's first stdin, its log and
tail, its end as the box's stop, the box's stop and give up with it, and
the exit. What the front is told on fd 4, and the terminal's lines routed
away while it runs, are `front-lines`' (next); the binary that sets the
front up is `binary-front`'s.

Acceptance, by setup.md, 7.6 (*No step*, *Its descriptors*, *The sign-in it
hands*, *The front's log*, *Its end is the box's stop*, *The box's own
stop*, *A give up with a front*, *Its exit*, *Where it lives*) and
`notes/contracts.md`, sections 4 and 5:

1. **Settings** (`host.rs`): `FirstLink` gains `pub local: Option<Secret>`,
   the first local link's secret; a new `pub struct Front { pub arguments:
   Box<[Box<[u8]>]>, pub link: Secret }` (the front's arguments after the
   program; the local link's URL), whose `Debug` writes `Front { arguments,
   .. }`, never the link; `Settings` gains `pub front: Option<Front>`, and
   its `Debug` a `front` field. `Service::new` asserts that a front comes
   with a first link holding `local` (the binary's rule, setup.md 7.6,
   *When*), and adds the front's five io entities (its process and four
   pipes) to `bounds.sockets` as it adds the listeners.
2. **No block, the front in its place:** with a front, `start_work` puts
   no block on the terminal: it hands `box.log` the block's words
   (`temper_box::front::block_words`) and then `after_block`, each line
   timed (`Formatter::notice`), calls `Formatter::admitted_block`, and asks
   for the front's spawn, before any step's. `block_ready` holds the first
   step until the front's spawn has completed, `Event::Spawned` or a spawn
   refused (`Event::Closed` with no process).
3. **The spawn:** `Request::Spawn` with `program`, `spawn_arguments(&settings,
   &front.arguments)` (the forge-child wrapper when `parent` is set),
   `spawn_environment(&settings.environment, 0)`, `root`, and the pipes
   `[{1, Out}, {2, Out}, {3, In}, {4, In}]`, in that order. Of its pipe
   tokens: 1 and 2 are read into the front's log and its tail as a step's
   stdout and stderr are (`Pipe::new(false)`, `Lines`, `Piece`, `Tail`),
   named `tui`; 3 is a `Sink` of one entry holding the local link's URL
   (`Secret::into_exposed`), closed once written (`tick(true, ..)`); 4 is
   held open, writing nothing in this task, and closed once the front has
   ended.
4. **Serve's first stdin:** where `first_link.local` is `Some`, serve's
   first spawn writes 64 bytes, the printed link's secret then the local
   link's, one entry, then its end; otherwise 32 bytes as today.
5. **The front's log:** `Logs::new` gains `front: bool`; with it a
   `tui.log` file sits after the steps' and before `box.log`, `box_step()`
   still the last; `Logs::front_step(&self) -> Option<usize>`. Made,
   moved and bounded as a step's, its pieces timed as a step's. Every
   place that names a log's step by `self.steps.get(piece.step)` names the
   front's `tui`.
6. **Its end:** at the front's `Exited` (or a refused spawn), `box.log`
   gets `tui exited <code>`, `tui killed by signal <n>` or `tui did not
   start` (`pipe::exited(b"tui", exit)`, `pipe::not_started(b"tui")`),
   then, while the machine has no outcome, `Input::Terminate` (so
   `Stopping` follows); an end other than `Exit::Code(0)` sets `failed`.
   `pub fn front_ended(&self) -> bool` is true from then on (false with no
   front).
7. **The box's own stop:** at a termination signal (`Event::Shutdown`,
   any of the three, where today's rule acts on it) while the front runs,
   the box sends it `Signal::Terminate` (`Target::Child`) before it inputs
   `Terminate` to the machine; `box.grace_ms` (`limits.grace`) later a
   second, and `box.grace_ms` after that a `Request::Close` of its process;
   a termination signal once it was signalled closes it at once. Its
   deadline joins `next_policy_deadline`.
8. **A give up:** the machine's give up neither signals nor closes the
   front; the box ends only once the front has ended, and exits 1.
9. **The end of the box:** `finish` and `is_empty` wait for the front's
   process, its two read pipes settled, its fd 3 sink and its fd 4 closed.
   `exit` is 1 when `failed` (the front's end other than exit 0 among its
   causes) or the outcome is not `Stopped`, as today.
10. **Tests** of *Steps*, each seen failing first.
11. **Status:** delete the **Not yet** bullet naming `box-front` in
    `temper/docs/design/protocol/setup.status.md`. No other design text.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`,
  sections 4 (serve's stdin) and 5 (the front).
- `temper/docs/design/protocol/setup.md`, 7.6 whole; 7.1, *A step's
  start*, *The first link's secret*; 7.2, *Stops*, *Gives up*, *Its own
  log*, *Its logs*, *A failure's tail*.
- `temper/crates/temper-box/src/front.rs` (`box-lines`): `block_words`.
- The code (`temper/crates/temper-box/src/`): `host.rs`: `FirstLink`
  (about line 32), `Settings` and its `Debug` (42-95), `Child` (110-150),
  `Service::new` (236-330: `io_limits`, `Logs::new`, adoptions),
  `event` (446-530: `Shutdown`, `Exited`, `Closed`), `spawned` (552-597:
  pipe tokens in order, serve's secret `Sink` at `pipes.get(4)`), `stream`
  (598-660), `children` (the spawn at about 1094-1140, the secret taken
  from `settings.first_link` at 1125-1128), `start_work` and `block_ready`
  (1157-1185), `log_work` (1219-1260: a piece's name and its tail),
  `finish` (1395-1435), `spawn_arguments`, `spawn_environment`,
  `spawn_pipes`, `io_limits` (1474-1560), the `Host` impl (1560-1830:
  `iterate`, `next_policy_deadline`, `is_empty`, `exit`); `logs.rs`:
  `Logs::new` (about 74-115), `box_step`, `max_notice` (about 479);
  `terminal.rs`: `Tail`, `Formatter::notice`; `pipe.rs`: `exited`,
  `not_started`; `machine.rs`: `Input::Terminate`, `stop` (1187),
  `give_up` (1166), `outcome`.
- Tests: `src/tests/fixtures.rs` (`Opening` 181, `with_terminal` 250-330,
  `answer` and `bind` 330-480: hosted children, `fixture_arguments` 936),
  `src/tests/host.rs` (`first_link_harness` 13, `stop_first_link` 33,
  `stdin_pipes` and `assert_spawn_carries_no_link` 102-116,
  `the_secret_is_on_serves_first_stdin_alone` 118,
  `a_stop_signals_each_long_step_twice_last_first` 440, `assert_stop_order`
  450, `a_long_step_that_keeps_exiting_is_given_up_naming_it` 610,
  `debug_of_settings_and_first_link_hides_credentials_and_the_url` 2066,
  `finish_child` 2144).

## Owns and may change

Owns the front in temper-box's host: the settings above, its state, its
spawn, its descriptors and its end, `Logs`' front slot, and the tests
below. The host's front state lives in `host.rs`, or in a submodule
`src/host/front_child.rs` (`mod front_child;` in `host.rs`) if it passes
about 150 lines.

Reader edits, the compiler's alone: `front: None` in every other
`Settings { .. }` literal and `local: None` in every other `FirstLink { ..
}` literal: `temper-box/tests/real_loop.rs` (about lines 461, 595, 647-651),
`temper/crates/temper/tests/suite/box_kinds.rs` (about line 162),
`temper/crates/temper/src/box_command.rs` (`Settings` about line 95,
`first_link`'s `FirstLink` about line 165); `Opening { .. }` literals in
`src/tests/host.rs` and `fixtures.rs` gain `front: None`. Nothing else in
`box_command.rs` (the binary's front is `binary-front`'s, which may run
beside `front-command`, whose files are `commands.rs`, `lib.rs`, `tui.rs`,
`tui_front.rs` and their tests: not yours). `front.rs` is `box-lines`':
read it, change nothing there; if a function it lacks is needed, write it
in the host and say so.

Not yours: what fd 4 carries, every routing of a terminal line to
`box.log` while the front runs, and the front's end on the terminal with
its tail (`front-lines`). In this task a terminal line while the front
runs still goes to the terminal, as today.

Options, each decided:

- **The front's grace:** if the machine's `Limits.grace` is not at hand in
  the host, keep a copy at `Service::new`; don't add a box limit.
- **A refused front spawn:** if a refused spawn reaches the host other
  than as `Event::Closed` with no process, treat what it does reach as
  *did not start* and name it in the report.
- **The harness's front:** `Opening` gains `front: Option<Front>`; the
  front's fixture arguments are `[b"never", b"tui"]`, so that a stubborn
  harness hosts it and names it `tui`, and `fixture_arguments` reads it
  with or without the wrapper. `Hosted` keeps descriptors 3 and 4 of a
  child (a field of yours), and a `read_fd(index, number, most)` beside
  `read_stdin` reads them.

## Steps

1. The harness: `Opening.front`, `Hosted`'s descriptors 3 and 4,
   `read_fd`, and a `front_harness(steps, parent)` in `tests/host.rs`
   beside `first_link_harness`: `first_link` with `local:
   Some(LOCAL_SECRET)` (`[0x5c; 32]`), `front: Some(Front { arguments:
   [never, tui], link: LOCAL_URL })` (`b"http://local/local"`), stubborn.
2. The failing tests, in `src/tests/host.rs`:
   - `the_front_starts_in_the_blocks_place_with_its_descriptors`: before
     any step's launch, one launch, the front's: through forge-child with
     the box's id when `parent` is set and directly without (two rows),
     its environment without `LISTEN_FDS` and `LISTEN_PID`, its pipes
     exactly `[{1, Out}, {2, Out}, {3, In}, {4, In}]`; its fd 3 reads the
     local URL whole and then its end; the box's stdout and stderr hold
     nothing; `box.log` holds `block_words`' lines, each timed, and then
     `Starting serve`; serve's stdin reads the printed secret then the
     local secret, 64 bytes, then its end; `assert_spawn_carries_no_link`
     (extended to both URLs and both secrets) holds for every launch; no
     log holds either URL or secret.
   - `the_fronts_end_stops_the_box`, a table of `(row, end, exit code)`:
     the front exits 0, so `box.log` holds `tui exited 0` before
     `Stopping` and `Stopped`, the steps stopped in reverse order
     (`assert_stop_order`'s shape), exit 0; it writes `boom\n` on its
     stderr then exits 3: `tui.log` holds `boom` after its time, `box.log`
     `tui exited 3`, exit 1; its process is killed by signal 9: `tui killed
     by signal 9`, exit 1; its spawn refused (`hold_spawn` then
     `fail_spawn`): no step launched, `tui did not start`, exit 1. Each
     row: `front_ended()` false before its end and true after.
   - `a_stop_signals_the_front_first_and_waits_for_it`: a first
     termination's first `Signal` request is the front's, before any
     step's; rows: the front exits 0 at it (exit 0); it ignores it (a
     second signal after `box.grace_ms`, a close after another, exit 1,
     `tui killed by signal 9` in `box.log`); a second termination closes
     it at once.
   - `a_give_up_leaves_the_front_running_until_it_ends`: a long step that
     keeps exiting is given up (as
     `a_long_step_that_keeps_exiting_is_given_up_naming_it` drives it);
     every step is closed while the front runs (no step's process left),
     the box still not empty and the front sent no signal; the front exits
     0; `box.log` holds `tui exited 0` and no `Stopping`; the box exits 1.
     A row where a termination comes while the front runs after the give
     up: the front is signalled, exits 0 at it, and the box exits 1.
   - `debug_of_settings_and_first_link_hides_credentials_and_the_url`
     gains a `Settings` with a front: its expected `Debug` string gains
     `front: Some(Front { arguments: [..], .. })`, and neither the local
     URL nor either secret appears; the existing string gains `front:
     None`.
   - In `src/tests/logs.rs`, if a table builds `Logs::new`, a row with a
     front: `tui.log` made beside `box.log`, `box_step()` unchanged.
3. The host, in the order of acceptance 1 to 9; the narrowest test after
   each: `heavy cargo nextest run -p temper-box --lib -E
   'test(/tests::host::/)'`.
4. Runs, through `heavy`, counts in the report: `heavy cargo nextest run -p
   temper-box` (its lib and `real_loop`); `heavy cargo nextest run -p
   temper --lib -E 'test(/box/)'`; `heavy cargo nextest run -p temper
   --test suite -E 'test(/box_kinds::/) | test(/box_listeners::/)'` (no
   front there: their box runs on pipes, and must stay green).
5. The status edit; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper-box -p temper --all-targets -- -D
   warnings`; `python3 -I -B tools/citations/citations.py --base
   origin/main`.
6. Commit (why: the box can run its TUI in front as a child of its own,
   hand it a sign-in, and stop when it ends), with `Look-request:
   tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says: each test added or extended, the rule
broken by hand for it and the failure seen; the runs' counts; where the
front's state lives; any option taken otherwise than its default.
