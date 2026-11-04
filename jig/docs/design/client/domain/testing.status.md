# testing: status

2026-11-04. Where jig's code lags or departs from [testing.md](testing.md).

- **Not yet:** 2, 4: the test family's drawing code, its golden scenes and its act on a drawing: `jig-client-test-view` writes a drawing node with no drawing code. 4: no shapes test checks a `resolved_default` preview, since `me` serves none (`contract.md`, 5.1, and its status). 4: the shared stories for a task page's order and a record's requester.
- **Departs:** 4: the scripted engine keeps its own query admission (`reads.rs`, `watches.rs`) and its subtree counts and folds (`queries.rs`, `watches.rs`), which differ from the core's where no shared story reaches: queries the view cannot send, a person task's count, a fold's `held_below`; no reason is recorded.
