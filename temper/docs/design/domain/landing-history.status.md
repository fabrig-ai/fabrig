# landing-history: status

2026-11-13. Where temper's code lags or departs from [landing-history.md](landing-history.md).

- **Not yet:** 2.1, 3.1, 3.2's `ChangeHistory`, 4.1, 4.2 and 7: the families, their records, keys and patch, the restart's interval below byte 128, and `Domain::write_event` [in flight: change 2496].
- **Not yet:** 4.3, 4.4 and 5.1's paging: the forge's loads of the history through the root, and the forge's read a page at a time [in flight: change 2501].
- **Not yet:** 4.5 and 6: the sweep by age and count, the hourly check, a project's deletion, and `forge.history_days` and `forge.history_events` [in flight: change 2506].
- **Not yet:** 2.2 and 2.4: `ready`, `unready`, the queue's `joined` and `departed`, `selected`, `paused`, `resumed`, a change's `held` and `released`, and `ended`, written by their steps [in flight: change 2502].
- **Not yet:** 2.2: `update`, `updated`, a merge job's `job` and `job-ended`, the queue's `conflict`, `asked`, `answered`, `pushed`, a queue merge's `turn`, `landed`, `recorded` as observed, and `after`, written by their steps [in flight: change 2503].
- **Not yet:** 2.2: `delivered`, which needs the worker's delivery receipts to reach the root; a follow-up after change 2503.
- **Not yet:** 2.2 and 3.3: a landing's `joined` and `departed`, `formed`, `round-ended`, the prepare job's `job`, `job-ended` and `conflict`, `landing-ended` with the row's `last_left` and each member's `round`, and a landing's `held` and `released`, one per member, a cut's `round-ended` as 3.3 says [in flight: change 2504].
- **Not yet:** 2.2, 3.3 and 3.4: `gate`, `gate-ended`, `proved`, `search`, `searched`, `probe`, `probe-ended`, the landing's `turn` and a squashed member's `recorded`, one per member [in flight: change 2505].
- **Not yet:** 5.1 and 5.3: `temper ctl landing history`, its forge read on the client's wire and its relay [in flight: change 2508].
- **Not yet:** 5.2: `change show` of a change whose row went [in flight: change 2509].
- **Not yet:** 8: the fuzzy histories, and the process tier's queue stories reading their landing's history [in flight: change 2510].
