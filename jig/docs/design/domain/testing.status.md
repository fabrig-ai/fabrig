# testing: status

2026-10-29. Where jig's code lags or departs from [testing.md](testing.md).

- **Built:** 3 to 7, 9: operations/indexes, two-clock restore, seed capacities and Seen commit/restart cases. 7: referees check that a task's settings revision is newest when made or moved and an assignment carries its run's revision. 6: fixed-scenario heap ceilings; the empty closure uses `EMPTY_CLOSURE_HEAP_CEILING` (1,300 bytes), including queue growth.
- **Not yet:** 4: preview completion. 7: ops worlds on real store.
