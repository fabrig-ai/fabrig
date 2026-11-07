# skein-gate-entries: skein's focused and fuzzy suites get their gate entries

`tools/tiers/budgets.toml` has gate entries for other trees. This plan adds skein's beside them, and nothing else:

```toml
[gate.skein.default]
cpu_suite_s = 30

[gate.skein.fuzzy]
cpu_suite_s = 120
```

The user asked for the limits to be turned on once a tree holds them. 30 and 120 are the existing figures for a tree's focused and fuzzy suites (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1), and the queue's record reads skein's suites under them, so no test or code changes to make them hold.

Add one comment line beside the entries, under the one for smith's suites, in these words:

```toml
# Skein's focused and fuzzy suites: 30 s and 120 s of the strategy's section 8, on as CPU seconds; whether those are their figures stays open.
```

## Limits

- **Whether 30 and 120 stay as written or become CPU budgets by the user's word stays open.** Leave `skein/docs/foundation/testing-strategy.md` exactly as it is. Don't write that the user set these figures.
- Change `tools/tiers/budgets.toml` alone. No other entry, tree or figure, and no `cpu_test_s` or any other key.
- One direct task, low effort, no parts. Its check is `tools/tiers/test_cpu.py`, which validates the file's shape. Nothing is measured in this plan.

## Sources

- `tools/tiers/budgets.toml`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
