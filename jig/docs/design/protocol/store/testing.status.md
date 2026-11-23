# testing: status

2026-11-23. Where the tree's code lags or departs from [testing.md](testing.md).

- **Departs:** 1: the focused stray-cleanup story chooses its first cut
  at `Change::Make`, not the required first removal. The gap and the
  whole-suite budget proof remain `../../domain/testing.status.md`, 9.
- **Departs:** 1: the core's world on the simulated disk kills no
  snapshot racing commits and no group of several commits in flight:
  those rows run in the store's world
  (`jig/tests/store`'s `crashes::snapshots_racing_linked_commits_and_several_commits_in_flight_keep_whole_prefixes`),
  a person and a task naming it queued below the component and swept
  over every file operation, with and without a snapshot's map chunk
  pending, whose referee checks a whole prefix and the task's requester
  present, not the start's link pass, because a snapshot waits for the
  log past its bound (skein's `data/kv.md`, 5), and a group for commits
  queued below the component.
- **Departs:** 1: `jig/tests/store/src/model.rs` reads a copy as a settings revision's and panics on any other row, because the core copies only that one, so no story reaches it; `jig/crates/jig-protocol-store/src/tests/commits.rs` holds a copy of another row and `commits.md`, section 1's two rules of a copy.
- **Not yet:** 1: the reference binary killed around each phase of a turn's commit (`jig/testing/jig-test/tests/process.rs` kills and restarts it once); `jig/tests/ops` and `jig/testing/jig-conformance` run `jig-fake-store`, where no world fakes the store.
