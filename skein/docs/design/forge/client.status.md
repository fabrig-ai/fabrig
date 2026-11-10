# forge-client: status

2026-11-10. Where skein's code lags or departs from [client.md](client.md).

- **Not yet:** 2.3: `Branches`' names are checked against `rows`
  (`skein-forge/src/measure.rs`, `answer_rows`).
- **Not yet:** 4: an item's read sends no `since` (`keep/poll.rs`).
 
- **Not yet:** 4: drift's `Fresh` read is a `Compare`. 7.4: an update is
  proved by a `Compare`'s empty commits (`outbox.rs`, `outbox/answer.rs`),
  not a `Range`.

- **Not yet:** 7.3, 10: no real-Forgejo conformance record shows a merge
  update of a head already holding its base, or a rebase's push overtaking
  a later push (`forgejo.md`, section 2). The client world's late copies read
  the head at landing, so its stories for both styles do not establish the
  rebase's concurrent-push behaviour. The connector world checks a dropped
  push by an explicit branch rewrite; the design takes the forced push at
  its worst until the real forge's behaviour is recorded.
