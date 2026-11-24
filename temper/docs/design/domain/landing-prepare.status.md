# landing-prepare: status

2026-11-24. Where the tree's code lags or departs from [landing-prepare.md](landing-prepare.md).

- **Not yet:** 1.2 (*Its outcome*): a kept prefix's commits are read
  for at most `forge.client.rows` pages, so a member's history longer
  than that is never taken as its prefix
  (`temper-engine-domain-forge/src/landing.rs`, `round_member`) [in
  flight: change 15531].

- **Not yet:** 1.3: the project's
  prepare and key commands, as jobs (`worker/jobs.md`).
