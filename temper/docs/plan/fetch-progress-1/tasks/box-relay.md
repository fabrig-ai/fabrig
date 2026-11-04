+++
id = "box-relay"
part = "box"
kind = "feature"
needs = ["fetched-line"]
files = ["temper/crates/temper-box/src/host.rs", "temper/crates/temper-box/src/machine.rs", "temper/crates/temper-box/src/pipe.rs", "temper/crates/temper-box/src/logs.rs", "temper/crates/temper-box/src/tests/**"]
effort = "high"
minutes = 80
+++

# The box passes the fetch's lines to serve

## Goal

The box turns each line of an asked fetch step's stdout into a `fetched` line on its pipe, shows none on its terminal, and never lets them crowd out its replies.

Three things the code does today make the plain relay wrong, so each is an acceptance item.

- The reply queue holds the longest group plus two lines, and a `Say` with no room is dropped (`host.rs`, `Order::Say`). Pushing every fetch line there can lose a `started` or a `failed`, and an ask then never ends.
- A step's lines reach the terminal three ways: `--echo steps`, a log not written, and a failure's tail.
- The machine moves on at the step's exit, so a line still in the pipe can follow the next step's `step` line.

Acceptance:

- a stdout line of the fetch step of the group the box is starting becomes `fetched FORGE BINARY DONE TOTAL` on the pipe, between `step FORGE fetch` and the next `step`;
- its log keeps it, and no `--echo steps`, unwritten log or failure's tail shows it;
- a line left in the pipe after the step's exit stays in the log and goes no further, with nothing named;
- a line the box cannot read is named on stderr, `box: fetch: unknown line`, and logged as a step's other lines are;
- a fetch line from a running step of a group not starting (a started group's once step run again) is named the same way and goes no further;
- with the pipe stalled, the box holds the latest line alone, and every reply is still written, the `fetched` line held being dropped by a reply;
- the longest pipe line is at most 64 bytes.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.1 (the table's `fetched` row), and 7.2 ("A fetch's lines", "Its logs", "A failure's tail"), and 13.1's bullet "A fetch's lines".
- `temper/docs/plan/fetch-progress-1/notes/contracts.md`, sections 2 and 3, and `notes/common.md`.

## Owns and may change

Owns `temper-box` but for what's left to it below.

Pre-approved:

- `Order` and `Said` stay `Copy`: the host reads the line and asks the machine, and the line never enters an `Order`.
- If `Sink` lacks a way to say it is fully written, use `drained()`, as `control_ready` does. If that does not hold between a push and its flush, add a small query to `sink.rs` and say so.
- A test helper a child's scripted stdout needs goes in `tests/fixtures.rs`, with a `fetching_steps()` beside `grouped_steps()`, its group `Stage::Fetch` then `Stage::Prepare` then `Stage::Run`.

## Steps

1. `pipe.rs`: `fetched` as contracts section 3 has it. Test in `tests/pipe.rs`: both forms, and `the_longest_fetched_line_fits_the_least_line_bound`, at forge 65535 and `u64::MAX` twice, against 64.
2. `machine.rs`:

   ```rust
   /// What the box does with a fetch line of this step (protocol/setup.md, 7.2).
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub enum Relay { Send, Unasked, Exited }
   /// `Send` while its group starts and it runs; `Unasked` while it runs for no ask.
   #[must_use]
   pub fn relay(&self, step: usize) -> Relay
   ```

   With a machine test for each.
3. `logs.rs`: `Piece` gains a field for a line kept in the log alone. `host.rs`'s `log_work` then neither echoes it nor pushes it to the tail. A piece for which the log could not be written stays silent too.
4. `host.rs`, where a child's `Output::Piece` is taken (`logs.put`): for stdout, `fd == 0`, of a step whose `asked.stage` is `Stage::Fetch`, read `temper_forge_setup::fetch::Fetched::read(&bytes)`. Then:
   - read, `Relay::Send`: hold `pipe::fetched(forge, fetched)` in a new `Control.fetched: Option<Box<[u8]>>`, replacing the one held, and log the piece alone;
   - read, `Relay::Exited`: log the piece alone;
   - read, `Relay::Unasked`, or not read: name it by `notice`, within `notice_room`, and log the piece as today.

   No `control` drops the line.
5. The held line is pushed on the sink only when the sink is fully written. `Order::Say` clears it before it pushes its own line, so a `fetched` never follows the next `step`. Do it where `control.sink.tick` runs.
6. Tests in `tests/host.rs`, through the `Harness`: `an_asked_fetch_relays_its_lines_between_its_steps`, `no_terminal_shows_a_fetch_line` (default, `--echo steps`, a failed fetch's tail holding its stderr line and no fetch line), `a_fetch_line_left_in_the_pipe_goes_no_further`, `a_line_it_cannot_read_is_named`, `a_started_group_s_fetch_line_is_named_and_dropped`, `a_stalled_pipe_keeps_the_latest_and_every_reply`. The last asserts a `failed` line reaches the pipe after more fetch lines than the reply queue holds.
7. `cargo fmt`; the old-forms check; `heavy cargo clippy -p temper-box --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-box`; `heavy cargo check --tests -p temper`. Commit and push.

## Report

As the implementer's text has it, with: how the held line is written, and each rule broken for a new test and the failure seen.
