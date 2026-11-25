# ledger: status

2026-11-25. Where temper's code lags or departs from [ledger.md](ledger.md).

- **Not yet:** 2.4, 4.6, 4.12 and 4.13: a pass that starts waiting keeps `held` on its build, an unstarted resume holds the waiting pass's build again, a resume may take the pass's own held build, and a `held` naming a waiting pass is read as it stands. [in flight: change 17938]
