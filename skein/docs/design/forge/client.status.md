# forge-client: status

2026-11-26. Where skein's code lags or departs from [client.md](client.md).

- **Not yet:** 4, *Moved*, 4.2 and 8: the client keeps a writer flag (`Event::Writer`), takes a forward move under a writer as its own by an ancestry read, and reports `Drift` with no actor [in flight: change 18924].



- **Not yet:** 4: a branch never there rests at `slow` like any absent branch (`keep/answer.rs`, `rest_resource`).
- **Not yet:** 4: a commit's hint reads the pull requests with it as their head (`keep/poll.rs`, `affected`).
- **Not yet:** 4: a cut page of reviews, read on with `skip`; no review page is cut.
- **Not yet:** 5: fresh reads of one resource asked together are each a call (`calls.rs`, `queue`).
- **Not yet:** 4: a pull request's size alone is not news (`keep/answer.rs`, `resource_answered`) [in flight: change 532].

- **Not yet:** 2.3: `Branches`' names are checked against `rows`
  (`skein-forge/src/measure.rs`, `answer_rows`).
- **Not yet:** 4: drift's `Fresh` read is a `Compare`.

- **Not yet:** 10: the update conformance gap remains `forgejo.status.md`, 2.
