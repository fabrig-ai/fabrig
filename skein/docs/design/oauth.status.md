# oauth: status

2026-11-05. Where skein's code lags or departs from [oauth.md](oauth.md).

- **Not yet:** 4: `skein-oauth/src/json.rs`'s `Json` and `Collector` derive
  `Debug` over their tokens, and a `Json` holds a token response, though only
  as a local of `machine.rs`'s reader, in no record that prints: the request
  of 2026-11-05 did not name them, and they stand among lib.md, 6's carriers.
  `skein/tests/oauth-accounts/tests/suite/debug.rs` has no row for
  `accounts::Component` or `fake::Issuer`, whose hand-written `Debug`s print
  through `finish_non_exhaustive`, because that request's plan took a row only
  if each built in about five lines.
- **Departs:** 3: `skein/docs/foundation/programming-model.md`, 5.4 asks
  for one explicit state enum per lifecycle, each state holding exactly
  its data, no lifecycle in flags or `Option`s beside it, and one handler
  per cell; `skein-oauth-accounts/src/exchange.rs` keeps `pending_grant`,
  `aborted` and `file_active` beside `Exchange::stage`, and neither
  crate's machine keeps one handler per cell; no reason is recorded.
