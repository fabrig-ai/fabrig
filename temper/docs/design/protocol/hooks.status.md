# hooks: status

2026-11-27. Where temper's code lags or departs from [hooks.md](hooks.md).

- **Not yet:** 2.2, 4.1 and 5: smith's conventions name no hooks directory, and a run's checks are `.temper/pre-pr` at both moments.
- **Not yet:** 2.2, 4.2, 4.3 and 5: temper reads no `.temper/hooks/`; the merge and close jobs run the project's `merge_job` and `close_job` alone, and the worker spawns a hook's command unchecked.
- **Not yet:** 5: fabrig keeps no `.temper/hooks/`, and the reference profile names `merge_job` and `close_job`.
