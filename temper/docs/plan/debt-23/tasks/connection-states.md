+++
id = "connection-states"
part = "spot-review"
kind = "feature"
needs = ["connection-deadline"]
files = ["skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/connection/tests.rs", "skein/crates/skein-forge-connection/src/tests/connection.rs", "skein/crates/skein-forge-connection/src/component.rs", "skein/docs/design/forge-connection.status.md"]
effort = "high"
minutes = 45
+++

# skein-forge-connection: `Closed` and `Idle` hold no exchange, and `Closed` no deadline

## Goal

Finding 4 of `temper/docs/plan/debt-23/inputs/debt21-review.md`, checked against the code at this plan's base. `State` keeps three things `skein/docs/design/forge-connection.md`, 8.1 says it doesn't, and none reaches anything outward:

- **`Closed.exchange`** is only ever `Some` from `http_event`'s raw overflow, which returns `Moved::Finished` at once. All three readers of that (`drain`, `connection/forgejo.rs`, `connection/github.rs`) call `finish` in the same arm with `reuse: Close`, and `end` drops it. Nothing reads it between.
- **`Idle.exchange`** comes only from `State::idle` when an exchange is running, and `connect` and `Event::Ready` only reach it from `Connecting` and `Handshaking`, which hold none. The sends are guarded by `Phase::Idle` in the callers: check each `exchange.start` as you go (`component.rs`'s start read, `component/calls.rs`, `component/github.rs`, `component/github_calls.rs`). Only `a_connected_notice_keeps_the_in_flight_exchange` reaches this state, by calling `connect` twice.
- **`Closed.until`:** `rest` on a Closed connection, from `component.rs`'s `start_heard`, is cleared by `component/calls.rs`'s sweep in the same `drive`. Only `a_start_answer_can_hold_a_closed_connection_until_close_clears_it` sees it.

Acceptance:

- `State::Closed { closing, ended, draining }` and `State::Idle { draining, until }`. `deadline()` answers `None` for Closed, `rest` leaves Closed as it is, `in_flight` answers `false` for Closed and Idle, and `exchange_mut`, `into_exchange` and `ready` lose those arms' data.
- **The one departure from the common note:** the two tests named above go, by finding 4. They pin the states this task removes and nothing outward. No other test changes.
- `Receiving::Json { body: JsonBody, next: Option<tokenizer::Request> }` replaces `Exchange`'s `json_body` and `json_next`, so `discard_redirect`'s resets are `receive = Receiving::Empty` and its two queue clears. Do this only if a reading finds it safe: today `json_demand` and `pump_json_body` run on those fields whatever `receive` says, and `discard` sets `Receiving::Empty` without clearing `json_down`. If a demand can arrive with `receive` not `Json` (read `connection/forgejo.rs`, `connection/github.rs`, and what calls `discard`), or the change adds net lines, leave `Exchange` as it is, and the report says which. `Intake`'s allocation moving from `send` to `begin_json` can only lower a heap pin.
- `skein/docs/design/forge-connection.status.md`'s 8.1 entry, beginning "8.1: Closed and Idle, which the table has holding nothing", reads whole:

  > 8.1: Closed and Idle, which the table has holding nothing, hold what
  > `skein-forge-connection/src/connection.rs`'s `State` keeps past an
  > exchange, because the request of 2026-11-05 moved no test's events or
  > times: Closed its ending and its close under way; each a redirect's
  > `draining`. For that reason Idle does not always run `idle`: a connection
  > that connects with no holder keeps the deadline it opened under, and
  > `component/github.rs` rests one with no deadline, which
  > `component/calls.rs`'s sweep reads as passed.

  No other entry changes.
- `connection.rs`'s two comments on those variants ("Raw overflow holds...", "A start answer can hold...", "A connected notice can leave...") go or say what's now true. `grep -c 'exchange: Option<Exchange>' skein/crates/skein-forge-connection/src/connection.rs` reads 0.
- `heavy cargo nextest run -p skein-forge-connection --lib`, `-p skein-forge-machine-world`, `-p skein-forge-protocol-world`, `-p skein-forge-client-world`, `-p skein-forge-setup-world`, `-p skein-forge-dialect-tests` pass.
- About -25 net lines, more if `Receiving::Json` holds. Production falls.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`, with the departures above: the two tests go, and this task edits the status entry in the words above.
- `temper/docs/plan/debt-23/inputs/debt21-review.md`, finding 4
- `skein/docs/design/forge-connection.md`, 8.1; `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the files in the header. `component.rs` only if `start_heard`'s `rest` should go along with what it no longer does: it still rests an own Idle connection, so probably it stays.
- **`next_deadline`:** a Closed connection whose close is already under way could hold an `until` today, and so a wake. Removing it removes a wake at a time when nothing happens. If a test sees that (a pinned `next_deadline`, a retimed run), stop and ask: that's a visible change.
- **If an `exchange.start` is reachable outside `Phase::Idle`,** so that `Idle.exchange` is reachable, stop and ask with the path.

## Steps

1. Check the three claims above by reading, each in a line of your report.
2. The variants; `Receiving::Json` if it's safe.
3. The status entry.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; each claim's check; whether `Receiving::Json` went in, and why. No new rule, so no test to bite.
