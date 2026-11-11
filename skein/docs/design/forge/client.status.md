# forge-client: status

2026-11-11. Where skein's code lags or departs from [client.md](client.md).

- **Not yet:** 4: a branch never there rests at `slow` like any absent branch (`keep/answer.rs`, `rest_resource`) [in flight: change 157].
- **Not yet:** 4: a commit's hint reads the pull requests with it as their head (`keep/poll.rs`, `affected`) [in flight: change 158].
- **Not yet:** 4: a cut page of reviews, read on with `skip`; no review page is cut [in flight: change 161].
- **Not yet:** 5: fresh reads of one resource asked together are each a call (`calls.rs`, `queue`) [in flight: change 160].

- **Not yet:** 2.3: `Branches`' names are checked against `rows`
  (`skein-forge/src/measure.rs`, `answer_rows`).
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
