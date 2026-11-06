# oauth: status

2026-11-06. Where skein's code lags or departs from [oauth.md](oauth.md).

- **Not yet:** 4: `skein-oauth/src/json.rs`'s `Json` and `Collector` derive
  `Debug` over their tokens, and a `Json` holds a token response, though only
  as a local of `machine.rs`'s reader, in no record that prints: the request
  of 2026-11-05 did not name them, and they stand among lib.md, 6's carriers.
- **Departs:** 3: `skein/docs/foundation/programming-model.md`, 5.4 asks
  for one explicit state enum per lifecycle, each state holding exactly
  its data, no lifecycle in flags or `Option`s beside it, and one handler
  per cell; `skein-oauth-accounts/src/exchange.rs` keeps `pending_grant`,
  `aborted` and `file_active` beside `Exchange::stage`, and neither
  crate's machine keeps one handler per cell; no reason is recorded.
  `skein-oauth/src/machine/fetch.rs`'s sign-in start holds its URL in a
  `(visit, url)` pair of `Option`s, with a `url.expect`, which the `debt-26`
  request's leftovers name: by reading, taking it from `Grant::SignIn` on the
  path with no fetch needs a new `unreachable!`, which that plan's task ruled out.
  6.8: `skein/tests/oauth-accounts`'s world keeps two owners, each with its own
  `Process`, and branches on `story ==` 80 times, against
  `skein/docs/foundation/testing-strategy.md`, 10, because the `debt-28` request
  of 2026-11-06 names that item and has one bigger than its size stop and ask,
  and its plan found this one 8 to 11 hours (the code's side is
  `skein/docs/development/debt/D048-oauth-accounts-world-copies.md`).
