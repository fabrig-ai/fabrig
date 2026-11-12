# keeping-up: status

2026-11-12. Where the tree's code lags or departs from [keeping-up.md](keeping-up.md).

- **Not yet:** 1: follows own watches too [in flight: change 105].
- **Not yet:** 1, *The branch heads*, at the bound: a run's push report and a released branch's head panic at a full table, and the other records drop the head silently, logging nothing and holding no task (`temper-engine-domain-forge/src/domain.rs`, `answered`, `accept_released_branch`) [in flight: change 1826].
- **Not yet:** 1, *The branch heads*, freed: no head leaves but a tier's deleted branch's, so the table fills as changes and goals end (`temper-engine-domain-forge/src/tiers.rs`, `outcome`) [in flight: change 1828].

- **Not yet:** 1, 2: a branch nothing has pushed: a change with no head reads its branch, its `PullFor` and its base's CI at every step; every commit's and item's hint concerns it; a hint matching no change wakes every change; its wait is `queue_window`; and a branch never there rests at `slow` (`temper-engine-domain-forge/src/domain/hints.rs`, `concerns`; `src/domain.rs`, `step_change`, `wait_change`; skein-forge-client's `keep/answer.rs`).
- **Not yet:** 2, 3: a gather reads its pull request with its head's CI and the head's checks again, every pull read reads CI, and a commit's hint reads its pull request in the client beside the commit's news (`src/domain.rs`, `step_read`).
- **Not yet:** 3: the repository's own facts: `Settings` reads the forge each time, and a `PullFor` that finds none drops the component's row, so the next call reads the repository again.
- **Not yet:** 2, 3: a landing's waits and a tier's flights poll at `poll`, never `Domain::poll_at`; a landing's gather reads a branch once for each member naming it; and no fresh read is shared (`src/landing.rs`, `wait_landing`; `src/tiers.rs`).

- **Not yet:** 3: containment is read as a commit list or a
  comparison, not a `Range`: the change step's up-to-date and catch-up
  ranges (`Read::Commits`, `temper-engine-domain-forge/src/domain.rs`,
  `ancestry_status`), a landing member's main (`landing.rs`) and a lost
  run's observed head (`domain.rs`, `lost_read`) (skein's `forge/README.md`,
  3.2).
