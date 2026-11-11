# forge: status

2026-11-11. Where skein's code lags or departs from [README.md](README.md).

`forgejo.md`, section 1.2 has no known gap in Forgejo's `Read::Commits`: its plan keeps
the oldest commits the answer's rows and bytes hold, giving up the newest
as older ones come, counts every row Forgejo gives against
`total_commits`, and answers a page those hold whole with `more`, a page
they don't, or a range whose oldest commit alone passes the answer,
`TooLarge` (`skein-forge/src/forgejo/plans/commits.rs`;
`a_range_past_the_answer_keeps_its_oldest_commits`). The pin's `compare`
transcript lists its three commits newest first, as the plan reads them.

- **Not yet:** 3.3, by plan `forge-polls-quiet`: Forgejo's dialect
  sends no validator, and GitHub's sends one on the delivery log's first
  page alone [in flight: change 155].
- **Not yet:** 3.2: `Pull`'s `ci`: every `Pull` and `PullFor` reads its
  head's CI [in flight: change 158].
- **Not yet:** 3.2, 3.3: `skip` on `Reviews`, `Remarks` and `PullFiles`,
  and `cut` on their answers [in flight: change 161].
- **Not yet:** 3.2: `Item`'s `since`, which neither dialect sends.
- **Not yet:** 3.2: `PullFiles`' `sides`, which neither dialect reads:
  every page reads both sides [in flight: change 436].
 
- **Not yet:** 3.7: `Branches`' names bounded by `rows`
  (`skein-forge/src/forgejo/plans/reads.rs`, and `measure.rs`'s row count).
 
- **Not yet:** 3.2, 3.3, 3.7: `Read::Range` and its answer, in
  neither dialect. No fact records the three narrowing filters; the fake
  honours them by design (../testing/fakes/forge.md, 4).
- **Not yet:** 3.2: `Runs` sent with `workflow=` in place of
  `workflow_id=` (`skein-forge/src/forgejo/client/files.rs`); `Pull`'s
  CI folds every page's rows where a short first page's `state` serves,
  and `Statuses` reads `more` from `total_count`, which is the page's
  (`plans/actions.rs`) [in flight: change 158].

- **Not yet:** 3.2: `Labels`, `CreateLabel` and `UpdateRef`, and `CreateIssue`'s labels, in neither dialect (`skein-forge/src/api.rs`'s `Read` and `Write`). 3.6: `Protection`'s `pulls_required` and `pulls_bypass`. 3.7, 8: the missing GitHub routes remain `github.status.md`, 4.5. 4.3: the artifact observations remain `forgejo.status.md`, 2.   the job-log, comments-page and removed-file observations remain `forgejo.status.md`, 2.

- **Departs:** 4.2: the plans' handler departure remains `forgejo.status.md` (Departs). 4.2: the decoded-kind match departure remains `forgejo.status.md` (Departs).  The client/server own-enum departure remains `forgejo.status.md`, 1.

- **Not yet:** 3.2: `PullFor`'s `head` and `base` filters.
