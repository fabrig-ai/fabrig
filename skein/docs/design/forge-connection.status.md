# forge-connection: status

2026-11-03. Where skein's code lags or departs from [forge-connection.md](forge-connection.md).

- **Departs:** 8: `skein/docs/foundation/programming-model.md`, 5.4 asks
  for one explicit state enum per lifecycle, each state holding exactly
  its data, no lifecycle in flags or `Option`s beside it, exhaustive
  matches and one handler per cell;
  `skein-forge-connection/src/connection.rs`'s `Connection` keeps
  `draining`, `closing`, `written` and request data beside a thin `Phase`,
  `hooks/connection.rs`'s `Hooked` keeps flags and `Option`s, and
  `installations.rs`'s `GitHub` keeps mint, read and identity flags and
  `Option`s beside its state enums; no reason is recorded (the code's
  side is `skein/docs/development/debt/D020-forge-connection-states.md`).
  8.1, 8.4: a forge connection's running deadline must follow its phase
  in one place after each transition, and a webhook connection has Head,
  Body, Answering and Closing states holding their data and deadlines;
  `skein-forge-connection/src/connection.rs` and `component.rs`,
  `component/calls.rs`, `component/github_calls.rs` and
  `component/github.rs` assign connection deadlines at separate sites,
  while `hooks/connection.rs`'s `Hooked` has no such four-state enum,
  using `malformed`, `ended`, `done`, `hint`, `hint_state` and `closing`
  instead; no reason is recorded (the code's side is
  `skein/docs/development/debt/D020-forge-connection-states.md`). 7.2:
  each forge must have its own connection in addition to the calls'
  `connections` limit; `skein-forge-connection/src/component.rs`'s
  `drive_forgejo` gates `open_own` on `call_connections`, which excludes
  only GitHub's own connections, and `component/calls.rs`'s `assign`
  reserves only GitHub's own connection from call reuse, so Forgejo's
  own connection still shares the calls' pool; no reason is recorded.
