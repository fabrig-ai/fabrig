# forge-client: status

2026-11-20. Where skein's code lags or departs from [client.md](client.md).

- **Not yet:** 7.8: the bound as one function, `stop_bound` (`outbox/stop.rs` computes it inline) [in flight: change 10967].
- **Not yet:** 8: what it still owes, its owed fresh reads and its entries not yet looked for, answered to its owner [in flight: change 10971].
- **Not yet:** 5: its holders, the calls holding the slab's places in admission order [in flight: change 10972].


- **Not yet:** 4: a branch never there rests at `slow` like any absent branch (`keep/answer.rs`, `rest_resource`).
- **Not yet:** 4: a commit's hint reads the pull requests with it as their head (`keep/poll.rs`, `affected`).
- **Not yet:** 4: a cut page of reviews, read on with `skip`; no review page is cut.
- **Not yet:** 5: fresh reads of one resource asked together are each a call (`calls.rs`, `queue`).
- **Not yet:** 4: a pull request's size alone is not news (`keep/answer.rs`, `resource_answered`) [in flight: change 532].

- **Not yet:** 2.3: `Branches`' names are checked against `rows`
  (`skein-forge/src/measure.rs`, `answer_rows`).
- **Not yet:** 4: drift's `Fresh` read is a `Compare`. 7.4: an update is
  proved by a `Compare`'s empty commits (`outbox.rs`, `outbox/answer.rs`),
  not a `Range`.

- **Not yet:** 7.3, 10: the update conformance gap remains `forgejo.status.md`, 2.
