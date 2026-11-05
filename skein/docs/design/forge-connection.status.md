# forge-connection: status

2026-11-05. Where skein's code lags or departs from [forge-connection.md](forge-connection.md).

- **Departs:** 8: `skein/docs/foundation/programming-model.md`, 5.4 asks
  for one explicit state enum per lifecycle, each state holding exactly
  its data, no lifecycle in flags or `Option`s beside it, exhaustive
  matches and one handler per cell;
  `skein-forge-connection/src/installations.rs`'s `GitHub` keeps mint,
  read and identity flags and `Option`s beside its state enums; no
  reason is recorded.
  8.1: a connection that has connected reaches Idle rather than Sending,
  an answered exchange goes through Idle before the next Sending, and a
  Closed connection is retired and a new one opened, where the table has
  each go straight on; no reason is recorded. `written` stays beside the
  state, as a fact of the last send read after its exchange has gone.
  8.1: Closed and Idle, which the table has holding nothing, hold what
  `skein-forge-connection/src/connection.rs`'s `State` keeps past an
  exchange, because the request of 2026-11-05 moved no test's events or
  times: Closed its ending, its close under way, a raw overflow's exchange
  until `finish` and a start answer's deadline after `Connection: close`;
  Idle a connected notice's exchange until its next write; each a
  redirect's `draining`. For that reason Idle does not always run `idle`:
  a connection that connects with no holder keeps the deadline it opened
  under, and `component/github.rs` rests one with no deadline, which
  `component/calls.rs`'s sweep reads as passed.
  8.4: Head holds a hint the root refused after `Next` went, offered
  before the next body, and Body one refused during it, which a `204`
  drops for its own and any other answer keeps; no reason is recorded. A
  refused head, a wrong signature and a malformed body wait in Answering
  until their answer's `Done`, where the table has them go straight to
  Closing, because Closing takes nothing more from the stream and could
  lose the answer. A fault's Closing stays under the deadline it had until
  its close goes, where the table has io's alone; no reason is recorded.
  7.1: a forge's dialect is not asked again, and the request of 2026-11-05
  that gave Forgejo's plans their states has it so too;
  `component/calls.rs`'s step matches `(side, &mut entry.dialect)`, a tuple,
  for every call it steps, because a GitHub call takes `gh` from its entry
  either way, and that request's debt item did not name this file.
  7.2:
  each forge must have its own connection in addition to the calls'
  `connections` limit; `skein-forge-connection/src/component.rs`'s
  `drive_forgejo` gates `open_own` on `call_connections`, which excludes
  only GitHub's own connections, and `component/calls.rs`'s `assign`
  reserves only GitHub's own connection from call reuse, so Forgejo's
  own connection still shares the calls' pool; no reason is recorded.
  7.4: each refusal names its limits, but
  `skein-forge-connection/src/limits.rs`'s `check` answers
  `Unusable::Zero`, which names none, for what `skein_tls::client::valid`,
  `skein_http::client::valid` and `skein_http::server::valid` refuse; no
  reason is recorded.
