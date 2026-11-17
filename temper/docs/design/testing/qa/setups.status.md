# setups: status

2026-11-17. Where temper's code lags or departs from [setups.md](setups.md).

- **Not yet:** 5: `turn-landing`'s check reads its forge with `ctl forge list`, not waiting until forge 1 is connected again after the engine starts afresh, so `ctl forge repositories 1` can meet it connecting. [in flight: change 6665]
