# sweeps: status

2026-11-26. Where skein's code lags or departs from [sweeps.md](sweeps.md).

- **Not yet:** 4.1 to 4.5 and 6: `skein_world::sweep` has no `Test`, `this_test!`, `cases`, `Case` or `gate()`, no modes, no case, cases or reach lines and no panic hook; `Sweep` keeps the set of kinds that fell, not their counts, and `check_fell` judges in every mode; skein-world has no `test_env` table [in flight: change 19499].
