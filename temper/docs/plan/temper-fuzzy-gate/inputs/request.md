# temper-fuzzy-gate: temper's fuzzy suite gets its gate entry

`tools/tiers/budgets.toml` holds the other trees' gate entries. This plan adds temper's fuzzy entry beside them, and nothing else:

```toml
[gate.temper.fuzzy]
cpu_suite_s = 120
```

Copy that shape exactly: `cpu_suite_s` alone, and no `wall_hard_s`, which these tables don't take. Place it after `[gate.jig.fuzzy]`.

The limit is being turned on. 120 is the strategy's figure for a tree's fuzzy suite (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1), and the queue's record reads temper's fuzzy suite at 102 to 105 CPU seconds over its last six full runs, so no test or code changes to make it hold.

Add one comment line beside the entry, after the line for jig's fuzzy suite, in these words:

```toml
# Temper's fuzzy suite: 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
```

## Limits

- Change `tools/tiers/budgets.toml` alone. No other entry, tree or figure; temper's focused suite gets no entry. If another tree's entries have landed by the time this lands, keep them all.
- One direct task, low effort, no parts. Its check is `tools/tiers/test_cpu.py`, which validates the file's shape. Nothing is measured in this plan.
- `walk = false`.

## Sources

- `tools/tiers/budgets.toml`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
