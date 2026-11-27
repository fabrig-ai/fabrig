# hooks: status

2026-11-27. Where temper's code lags or departs from [hooks.md](hooks.md).

- **Not yet:** 2.2, 4.1 and 5: smith's conventions name no hooks directory, and a run's checks are `.temper/pre-pr` at both moments. [in flight: change 20339]
- **Not yet:** 4.2 and 5: temper reads no `.temper/hooks/`; the merge and close jobs run the project's `merge_job` and `close_job` alone. [in flight: change 20352]
- **Not yet:** 2.2 and 4.3: the worker spawns a hook's command unchecked. [in flight: change 20340]
- **Not yet:** 5: fabrig keeps no `.temper/hooks/`, and the reference profile names `merge_job` and `close_job`. [in flight: change 20341]
