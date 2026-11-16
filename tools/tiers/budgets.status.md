# budgets: status

2026-11-16. Where the code lags or departs from [budgets.md](budgets.md).

- **Not yet:** 3: `tools/tiers/one-process`'s run places no binary: `plan.json`'s binaries and `results.json`'s processes hold no `tree` or `layer`, `results.json` no `layers`, and `run` takes no `layers` [in flight: change 5591].
- **Not yet:** 2, 4, 5, 6: `tools/tiers/budgets.toml`'s gate tables are `[gate.<tree>.<profile>]` with one `cpu_suite_s` a tree, which `budgets()` validates and nothing reads; the summary has no `layers` or `trees`; `summary`, `execute_one_process`, `tools/tiers/one-process` and `tools/tiers/cpu-summary` take no `gate` or `--gate`; one-process reads no budgets unless given `--budgets` [in flight: change 5592].
