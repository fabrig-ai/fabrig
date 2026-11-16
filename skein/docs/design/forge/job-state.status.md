# job-state: status

2026-11-16. Where the tree's code lags or departs from [job-state.md](job-state.md).

- **Not yet:** 5: a declared wait, and a commit-CI job's steps while it
  runs: the fake runs every commit's CI at once and lists a running
  commit-CI job's steps as none (`skein-fake-forge-domain/src/ci.rs`,
  `src/actions.rs`) [in flight: change 5297].
