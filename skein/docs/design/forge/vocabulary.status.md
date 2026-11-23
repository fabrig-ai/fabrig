# vocabulary: status

2026-11-23. Where the tree's code lags or departs from [vocabulary.md](vocabulary.md).

- **Not yet:** 1.2: `Merge`'s `Recorded { commit }`, Forgejo's
  `manually-merged` and GitHub's refusal of it; `MergeHow` has the style
  and fast-forward only alone (`skein-forge/src/api.rs`)
  [in flight: change 813].
- **Not yet:** 1.3: by plan `forge-polls-quiet`: Forgejo's dialect
  sends no validator, and GitHub's sends one on the delivery log's first
  page alone.
- **Not yet:** 1.2: `Pull`'s `ci`: every `Pull` and `PullFor` reads its
  head's CI.
- **Not yet:** 1.2, 1.3: `skip` on `Reviews`, `Remarks` and `PullFiles`,
  and `cut` on their answers.
- **Not yet:** 1.2: `Item`'s `since`, which neither dialect sends.
- **Not yet:** 1.2: `PullFiles`' `sides`, which neither dialect reads:
  every page reads both sides.

- **Not yet:** 1.7: `Branches`' names bounded by `rows`
  (`skein-forge/src/forgejo/plans/reads.rs`, and `measure.rs`'s row count).

- **Not yet:** 1.2, 1.3, 1.7: `Read::Range` and its answer, in
  neither dialect. No fact records the three narrowing filters; the fake
  honours them by design (../testing/fakes/forgejo.md, 1).
- **Not yet:** 1.2: `Runs` sent with `workflow=` in place of
  `workflow_id=` (`skein-forge/src/forgejo/client/files.rs`); `Pull`'s
  CI folds every page's rows where a short first page's `state` serves,
  and `Statuses` reads `more` from `total_count`, which is the page's
  (`plans/actions.rs`).

- **Not yet:** 1.2: `Labels`, `CreateLabel` and `UpdateRef`, and `CreateIssue`'s labels, in neither dialect (`skein-forge/src/api.rs`'s `Read` and `Write`). 1.6: `Protection`'s `pulls_required` and `pulls_bypass`. 1.7: the missing GitHub routes remain `github-operations.status.md`, 1.5.

- **Not yet:** 1.2: `PullFor`'s `head` and `base` filters.

- **Not yet:** 1.6: `Pull`'s `size` and `Size`, the forge's own counts at the pull request's head [in flight: change 532].
