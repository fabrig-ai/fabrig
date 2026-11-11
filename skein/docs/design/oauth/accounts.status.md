# accounts: status

2026-11-11. Where the tree's code lags or departs from [accounts.md](accounts.md).

- **Departs:**  the nonce-less profile gap remains `README.status.md`, 2.2.
   `skein/docs/foundation/programming-model.md`, 5.4 asks
  for one explicit state enum per lifecycle, each state holding exactly
  its data, no lifecycle in flags or `Option`s beside it, and one handler
  per cell; `skein-oauth-accounts/src/exchange.rs` keeps `pending_grant`,
  `aborted` and `file_active` beside `Exchange::stage`, and neither
  crate's machine keeps one handler per cell; no reason is recorded.
  the sign-in URL-state departure remains `README.status.md`, 3.
  8: `skein/tests/oauth-accounts`'s world branches on `story ==` 80 times,
  with further story comparisons and label helpers, against
  `skein/docs/foundation/testing-strategy.md`, 10's scenarios as data. The
  remaining rows are low-value work of level size, and the paste/listener
  ordering needs a script interpreted separately by each tier to preserve
  its visible events (`README.md`, section 6; the code's side is
  `skein/docs/development/debt/D048-oauth-accounts-world-copies.md`).
