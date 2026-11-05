# oauth: status

2026-11-05. Where skein's code lags or departs from [oauth.md](oauth.md).

- **Departs:** 3: `skein/docs/foundation/programming-model.md`, 5.4 asks
  for one explicit state enum per lifecycle, each state holding exactly
  its data, no lifecycle in flags or `Option`s beside it, and one handler
  per cell; `skein-oauth-accounts/src/exchange.rs` keeps `pending_grant`,
  `aborted` and `file_active` beside `Exchange::stage`, and neither
  crate's machine keeps one handler per cell; no reason is recorded.
