# executors-tools: status

2026-11-27. Where the tree's code lags or departs from [executors-tools.md](executors-tools.md).

- **Not yet:** 1: `measure` [in flight: change 19918].


- **Not yet:** 1: the tools `move`, `succeed`,
  `hold` and `lift`.

- **Not yet:** 1, *A run's read as its person*: a task query of the
  project that names a person requester reads its ended part from the
  person's index, which spans projects (`reads/ended.rs`,
  `ended_part`), and a page that loads another project's row answers
  unknown whole (`reads/rows.rs`, `person_from_store`), where a party's
  read drops that row. temper's addresses name no requester; no test
  reads such a query.

- **Not yet:** 1, *Of its project alone*: an inbox page filtered by the
  run's project takes its `next` cursor from the last entry of the
  person's whole inbox, before the filter (`reads/rows.rs`, the live
  page's `last`), so whether there is a next page, and the cursor the
  root holds, can come from another project's entry; temper says only
  whether more wait. No test pages a filtered inbox.
