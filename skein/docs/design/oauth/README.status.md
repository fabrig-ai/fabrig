# oauth: status

2026-11-11. Where skein's code lags or departs from [README.md](README.md).

`README.md`, section 2.2, and `accounts.md`, section 3 have no known gap in caller-selected profile claims,
their bounds and verification flags, or delivery beside the kept record
on sign-in and refresh, except for the nonce requirement below.

`accounts.md`, sections 3, 5, 7 and 8 have no known gap in the driver's
`redirect_bytes`, shared by the listener and pasted redirects, the
`redirect_room` and `listener_head` sizing helpers, or the owner's
application name. Focused rows cover synthetic 1,900- and 3,900-byte
codes through both paths and a browser's headers and cookie at the
redirect bound. The listener's own over-long redirect fails its sign-in
in the judging step with its length and bound, answers the long page
without a return, and settles after sending it; another state or path
and a duplicate state keep the refusal page. Owner-page figures are
deferred in `README.md`, section 6.

- **Not yet:** 4: `skein-oauth/src/json.rs`'s `Json` and `Collector` derive
  `Debug` over their tokens, and a `Json` holds a token response, though only
  as a local of `machine.rs`'s reader, in no record that prints: the request
  of 2026-11-05 did not name them, and they stand among ../runtime/lib.md, 6's carriers.
- **Departs:** 2.2: a refresh's profile reader requires a nonce claim, though
  it compares none (`skein-oauth/src/jwt.rs`, `refreshed_claims`). A token
  without one gives no profile claims and the refresh still succeeds, as
  `account-email/oauth-profile`'s acceptance requires. The task's stricter
  acceptance and section 2.2's nonce-independent profile checks disagree;
  reading a nonce-less token's profile remains unimplemented.
  3: the lifecycle-state and handler departures remain `accounts.status.md` (Departs).
  `skein-oauth/src/machine/fetch.rs`'s sign-in start holds its URL in a
  `(visit, url)` pair of `Option`s, with a `url.expect`, which the `debt-26`
  request's leftovers name: by reading, taking it from `Grant::SignIn` on the
  path with no fetch needs a new `unreachable!`, which that plan's task ruled out.
   the account-world scenario departure remains `accounts.status.md`, 8.
