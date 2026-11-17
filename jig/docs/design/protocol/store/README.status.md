# store: status

2026-11-16. Where jig's code lags or departs from [README.md](README.md).




- **Not yet:** 8: the worst case (`sizes.rs`) does not count the registry's rows held at open, their parsed schema texts, the derived plans' tables or the one `Workspace` fitted to them (`layouts.md`, 7); change 2846 left it, its memory bounded by the registered families' texts.
- **Departs:** 9: the focused stray-cleanup story chooses its first cut
  at `Change::Make`, not the required first removal. The gap and the
  whole-suite budget proof remain `../../domain/testing.status.md`, 9.
- **Departs:** 9: the core's world on the simulated disk kills no
  snapshot racing commits and no group of several commits in flight:
  those rows run in the store's world
  (`jig/tests/store`'s `crashes::snapshots_racing_linked_commits_and_several_commits_in_flight_keep_whole_prefixes`),
  a person and a task naming it queued below the component and swept
  over every file operation, with and without a snapshot's map chunk
  pending, whose referee checks a whole prefix and the task's requester
  present, not the start's link pass, because a snapshot waits for the
  log past its bound (skein's `data/kv.md`, 5), and a group for commits
  queued below the component.
- **Departs:** 9: `jig/tests/store/src/model.rs` reads a copy as a settings revision's and panics on any other row, because the core copies only that one, so no story reaches it; `jig/crates/jig-protocol-store/src/tests/commits.rs` holds a copy of another row and section 5's two rules of a copy.
- **Not yet:** 6.1: a page of history, spend or notes cut before its first row. 9: the reference binary killed around each phase of a turn's commit (`jig/testing/jig-test/tests/process.rs` kills and restarts it once); `jig/tests/ops` and `jig/testing/jig-conformance` run `jig-fake-store`, where no world fakes the store.
