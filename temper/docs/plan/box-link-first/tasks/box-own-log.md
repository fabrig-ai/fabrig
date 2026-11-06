+++
id = "box-own-log"
part = "box-terminal"
kind = "feature"
needs = []
files = ["temper/crates/temper-box/src/terminal.rs", "temper/crates/temper-box/src/host.rs", "temper/crates/temper-box/src/logs.rs", "temper/crates/temper-box/src/tests/**", "temper/crates/temper-box/tests/real_loop.rs"]
effort = "high"
minutes = 80
+++

# The box's lifecycle moves to `box.log`

## Goal

What leaves the terminal is written to the box's own log, `DATA/box/logs/box.log`. The terminal keeps errors alone, each timed. The sign-in block stays as it is for now, at serve's first link: `box-first-block` changes it.

Acceptance:

- `Fact::place` as the contract has it, listing every variant.
- `Starting`, a group's starting, started and stopped, `Stopping`, `Stopped` and `Logs: <data>/box/logs/` are in `box.log` and on neither terminal stream.
- A group's failed, a restart, a give up, the box's refusals, `box: cannot write ...` and a failure's tail are on the terminal as today. Each line is timed `HH:MM:SSZ `, also before any block: `Formatter::notice` times every line.
- `--echo steps` adds every step's lines and no line of `box.log`.
- `Settings.log_lines` open `box.log`, in order, before `Logs:`.
- `box.log` is mode 0600, moves to `logs.previous/` with the others, rotates to `box.log.1` at `box.log_bytes`, and holds no link.
- Its last line is `Stopped`: the log closes only once its lines are written.
- A placement in the log never waits on a blocked terminal: a test holds the terminal's write and sees `Starting serve` reach the log and the step spawn.
- If `box.log` cannot be written, it is named once on the terminal in today's words, and its lines then go to the terminal as their own timed words.
- All of `heavy cargo nextest run -p temper-box` passes, with the tests' words moved to `box.log`.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.2: *Its terminal*, *Its own log*, the paragraph on typed facts, *Its logs*.
- `temper/docs/plan/box-link-first/notes/contracts.md`, section *temper-box*: `Place`, `Fact::place` and `Settings.log_lines`. The rest of that section is `box-first-block`'s.
- `temper/docs/plan/box-link-first/notes/common.md`.

## Owns and may change

Owns `temper/crates/temper-box/src/terminal.rs`, `host.rs`, `logs.rs`, `temper-box/src/tests/**` and `temper-box/tests/real_loop.rs`. `machine.rs` keeps its typed facts: its tests read no words and should not change.

Reader and neighbour files, each only for `log_lines: Box::new([])` in a `Settings` literal, which keeps them compiling:

- `temper/crates/temper/src/box_command.rs`, in `run_configuration`.
- `temper/crates/temper/tests/suite/box_kinds.rs`, about line 161.

Left for later tasks, not yours, to report under `## Left`: every red in package `temper`'s tests that reads moved words (`box-readers-log` owns them), and `First run:`, which `box-draws-link` moves.

Options, each pre-approved:

- `Logs` may hold `box.log` as one more entry of `files`, after the steps', or as a field of its own. Whichever: a step index must never reach `self.children` for it.
- `box.log`'s lines may wait in an uncapped queue of the host's: the design says what bounds the box is at its edges.

## Steps

The readers to move, from the survey, under `temper/crates/temper-box/`:

- `src/tests/terminal.rs`, about lines 38-44, 84-86, 141, 157-160.
- `src/tests/host.rs`, about lines 105, 182-185, 554, 562, 595, 627, 636, 818-826, 863, 910, 972, 1055, 1097-1099, 1283-1284, 1788-1811, 1842-1855.
- `src/tests/logs.rs`, about lines 68-69.
- `src/tests/restart.rs`, about line 145, a diagnostic counting `Starting serve`.
- `tests/real_loop.rs`, about lines 33-35 and 492-514: the stop's order moves to `box.log`.

`orders_runnable`, about `host.rs` line 343, gates a terminal fact on the terminal's room: a fact placed in the log gates on the log's.

Write first, in `src/tests/host.rs`: `the_terminal_keeps_errors_alone_and_box_log_the_rest`, reading both exactly at a fixed wall; `box_log_ends_with_stopped`; `a_blocked_terminal_holds_no_log_line_and_no_spawn`; `an_unwritable_box_log_is_named_once_and_its_lines_follow`. Add to the `logs.rs` tests that `box.log` rotates and moves.

Narrow runs: `heavy cargo nextest run -p temper-box --lib -E 'test(box_log)'`. Then fmt, clippy and the focused tests of `temper-box` as your text says. `heavy cargo check -p temper --tests` is your check on the literals; run no `temper` tests. Commit and push.

## Report

As your text says, with the place the log's lines wait in, and each `temper` test you left red that you noticed.
