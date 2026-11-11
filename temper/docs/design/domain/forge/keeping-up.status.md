# keeping-up: status

2026-11-11. Where the tree's code lags or departs from [keeping-up.md](keeping-up.md).

- **Not yet:** 2: a commit's news and a checking change's pace: a commit's hint wakes every change using it into a whole fresh read, and a job's hint rereads its commit's pending CI at `hinted` doubling to `poll` through `settle` whatever its checks show, each later hint starting the window again (`temper-engine-domain-forge/src/domain.rs`, `ci_hint`; `src/settle.rs`); no story pins a checking change's requests a minute on Forgejo [in flight: change 49].

- **Not yet:** 3: containment is read as a commit list or a
  comparison, not a `Range`: the change step's up-to-date and catch-up
  ranges (`Read::Commits`, `temper-engine-domain-forge/src/domain.rs`,
  `ancestry_status`), a landing member's main (`landing.rs`) and a lost
  run's observed head (`domain.rs`, `lost_read`) (skein's `forge/README.md`,
  3.2).
