# budgets: status

2026-11-16. Where the code lags or departs from [budgets.md](budgets.md).

- **Not yet:** 3: `results.json` holds no `workspace`, so a run records nothing of the binaries it didn't run.
- **Not yet:** 2, 4, 10: `budgets()` refuses a profile's `moves`, and a layer's budget is its table's `cpu_s` alone: no row or tree has a `ceiling_s`, and a layer with no table has no budget.
- **Not yet:** 5, 7, 8, 9: no ledger: `ledger()` doesn't exist, `summary` and `execute_one_process` take no `ledger`, `tools/tiers/one-process` and `tools/tiers/cpu-summary` no `--ledger`, a run writes no `ledger.json`, and a run is judged by what it measured against each layer's ceiling, with no figure at main, held budget, main's red or named cause.
