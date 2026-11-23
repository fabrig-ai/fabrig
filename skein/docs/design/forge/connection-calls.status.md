# connection-calls: status

2026-11-23. Where the tree's code lags or departs from [connection-calls.md](connection-calls.md).

- **Not yet:** 1.6: by plan `forge-polls-quiet`: no read but the
  delivery log's first page sends a validator, nothing is kept for a
  `304`, and the limits have no `kept_bytes`.
- **Not yet:** 1.2: `Settings` reads the forge though a row is kept, a
  `404` of `PullFor` or of an Actions read drops the row, and a write
  refused `Forbidden` keeps it (`skein-forge/src/forgejo/plans/reads.rs`,
  `failure.rs`).
