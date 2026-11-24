# lifecycle-holds: status

2026-11-24. Where the tree's code lags or departs from [lifecycle-holds.md](lifecycle-holds.md).

- **Not yet:** 1, *A wait past its bound holds*: no hold for `Waited`, and no `Wait` source [in flight: change 16514].
- **Not yet:** 1, *A wait past its bound holds*: no bound alarm: a `news`, `host` or `account` wait passes its figure with nothing done, and a waiting task passes its deadline unheld [in flight: change 16531].
- **Not yet:** 2, *Its reason is never empty*: a party's `Stop` carries no reason, and a placement with an empty reason is admitted [in flight: change 16544].
- **Not yet:** 3: a hold carries its category and a preparation's or a connector's words alone, and the escalation's entry says "Held task needs a decision" [in flight: change 16545].
- **Not yet:** 1: an account's terminal read once its source is usable again is rechecked at once, with no backoff [in flight: change 9667].
- **Not yet:** 1, *A claim's cause stays readable while its attempt is placed*: a claim keeps no cause once it clears its retry notice, and nothing marks a recheck [in flight: change 9704].
- **Not yet:** 1: a standing task's tries per session (`standing.md`, 5.1).
- **Not yet:** 1: a dependency's or input's result whose load the store answered busy counts as transient, not infrastructure (`jig-domain-core/src/routing/run/results.rs`, `loaded`).

- **Not yet:** 2: a drain; a project's scope; a
  party's proposal of a release; a hold parked until an event.
   widening, moves, successors and surviving questions remain `changes.status.md` (Not yet, the whole document).  markers remain `messages.status.md`, 3.
   conversation questions remain `messages.status.md`, 5.
   the missing stories remain `README.status.md`, 7.
