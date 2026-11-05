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
  8.4: Head holds a hint the root refused after `Next` went, offered
  before the next body; no reason is recorded. 7.2:
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
