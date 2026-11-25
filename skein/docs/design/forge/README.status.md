# forge: status

2026-11-17. Where skein's code lags or departs from [README.md](README.md).

`forgejo-operations.md`, section 1 has no known gap in Forgejo's `Read::Commits`: its plan keeps
the oldest commits the answer's rows and bytes hold, giving up the newest
as older ones come, counts every row Forgejo gives against
`total_commits`, and answers a page those hold whole with `more`, a page
they don't, or a range whose oldest commit alone passes the answer,
`TooLarge` (`skein-forge/src/forgejo/plans/commits.rs`;
`a_range_past_the_answer_keeps_its_oldest_commits`). The pin's `compare`
transcript lists its three commits newest first, as the plan reads them.

- **Not yet:** 8: the missing GitHub routes remain `github-operations.status.md`, 1.5.
