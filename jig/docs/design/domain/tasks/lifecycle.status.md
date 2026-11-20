# lifecycle: status

2026-11-20. Where the tree's code lags or departs from [lifecycle.md](lifecycle.md).

- **Not yet:** 2, 6 and 12.1: an attempt its worker's stop cut ends suspended, not interrupted: its end read `suspended` in the activity and its row, and its deadline moved by the time it spent suspended. [in flight: change 10948]


- **Not yet:** 6: an account's terminal read once its source is usable again is rechecked at once, with no backoff [in flight: change 9667].
- **Not yet:** 6, *A claim's cause stays readable while its attempt is placed*: a claim keeps no cause once it clears its retry notice, and nothing marks a recheck [in flight: change 9704].
- **Not yet:** 2 and 9: a standing task: no `NothingAwaited` notice, a finish parked, its `standing` ending (`standing.md`, section 2).
- **Not yet:** 6: a standing task's tries per session (`standing.md`, 5.1).
- **Not yet:** 6: a dependency's or input's result whose load the store answered busy counts as transient, not infrastructure (`jig-domain-core/src/routing/run/results.rs`, `loaded`).
- **Not yet:** 13: a task failed by its application, `tasks::Event::Fail` [in flight: change 10349].

- **Not yet:** 10: a drain; a project's scope; a
  party's proposal of a release; a hold parked until an event.
   widening, moves, successors and surviving questions remain `changes.status.md` (Not yet, the whole document).  markers remain `messages.status.md`, 3.
   conversation questions remain `messages.status.md`, 5.
   the missing stories remain `README.status.md`, 7.
