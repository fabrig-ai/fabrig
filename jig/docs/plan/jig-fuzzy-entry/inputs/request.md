# jig-fuzzy-entry: jig's fuzzy suite gets its gate entry

`tools/tiers/budgets.toml` has gate entries for other trees. This plan adds jig's fuzzy suite's, beside them, and nothing else:

```toml
[gate.jig.fuzzy]
cpu_suite_s = 120
```

The user asked for the limits to be turned on. 120 is the existing figure for a tree's fuzzy suite (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1), and the queue's record reads jig's suite well under it, so no test or code changes to make it hold.

Add one comment line beside the other trees' comments, in these words:

```toml
# Jig's fuzzy suite: 120 s of the strategy's section 8, on as CPU seconds; whether that is its figure stays open.
```

## Limits

- **No `[gate.jig.default]`.** The focused suite is over its figure, and its entry waits on the user's word.
- Copy `[gate.hearth.fuzzy]`'s shape exactly: one key, `cpu_suite_s`.
- Change `tools/tiers/budgets.toml` alone. No other entry, tree or figure.
- One direct task, low effort, no parts. Its check is `tools/tiers/test_cpu.py`, which validates the file's shape. Nothing is measured in this plan.

## Sources

- `tools/tiers/budgets.toml`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
