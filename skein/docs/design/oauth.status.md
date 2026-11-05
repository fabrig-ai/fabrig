# oauth: status

2026-11-05. Where skein's code lags or departs from [oauth.md](oauth.md).

- **Not yet:** 4: `skein-oauth/src/json.rs`'s `Json` and `Collector` derive
  `Debug` over their tokens, and a `Json` holds a token response, though only
  as a local of `machine.rs`'s reader, in no record that prints: the request
  of 2026-11-05 did not name them, and they stand among lib.md, 6's carriers.
- **Departs:** 3: `skein/docs/foundation/programming-model.md`, 5.4 asks
  for one explicit state enum per lifecycle, each state holding exactly
  its data, no lifecycle in flags or `Option`s beside it, exhaustive
  matches and one handler per cell; `skein-oauth/src/machine.rs` keeps
  `Client::phase` beside `active: Option<Exchange>` and tests the phase
  by comparison, while `skein-oauth-accounts/src/component.rs` keeps
  `accounts`, `names`, `sources`, `bindings` and `keepers` in parallel
  lists and five flags in `State::Record`, and `exchange.rs` keeps
  `pending_grant`, `aborted` and `file_active` beside `Exchange::stage`;
  no reason is recorded (the code's side is
  `skein/docs/development/debt/D022-oauth-states.md`).
