# forgejo-operations: status

2026-11-24. Where the tree's code lags or departs from [forgejo-operations.md](forgejo-operations.md).


- **Not yet:** 1: `Reviews`, `PullFiles` and a `Remarks` slice sized for real rows and cut at the answer's bytes, read on with `skip`: each holds as many of its listing's largest rows as `answer_bytes` holds (`skein-forge-connection/src/limits.rs`, `page_size`).
- **Not yet:** 1: `Read::Pull`'s `ci`; `PullFor`'s status read (`plans/reads/done.rs`, `pull_status`).
- **Not yet:** 1: `Settings` read with a row kept; `PullFor`'s and Actions' `404`s drop the row; a write's `401` or `403` keeps it (`plans/reads.rs`, `plans/failure.rs`, `plans/actions.rs`).
- **Not yet:** 1: `Read::PullFiles` without `sides`: every page reads both sides (`plans/contents.rs`).

- **Not yet:** 1: `Item`'s `since`, which neither dialect sends.

- **Not yet:** 1: `Branches`' names bounded by `rows`
  (`skein-forge/src/forgejo/plans/reads.rs`, and `measure.rs`'s row count).

- **Not yet:** 1: `Read::Range` and its answer, in
  neither dialect.
- **Not yet:** 1: `Runs` sent with `workflow=` in place of
  `workflow_id=` (`skein-forge/src/forgejo/client/files.rs`); `Pull`'s CI
  folds every page's rows where a short first page's `state` serves, and
  `Statuses` reads `more` from `total_count`, which is the page's
  (`plans/actions.rs`).

- **Not yet:** 1: `Read::Pull`'s `size`, from `changed_files`, `additions` and `deletions` [in flight: change 532].
