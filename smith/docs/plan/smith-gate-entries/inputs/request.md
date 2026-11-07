# smith-gate-entries: smith's suites get their gate entries

`tools/tiers/budgets.toml` holds hearth's gate entries. This plan adds smith's beside them, and nothing else:

```toml
[gate.smith.default]
cpu_suite_s = 30

[gate.smith.fuzzy]
cpu_suite_s = 120
```

Copy that shape exactly: `cpu_suite_s` alone, and no `wall_hard_s`, which these tables don't take. Place them after `[gate.hearth.fuzzy]`.

The limits are being turned on. 30 and 120 are the existing figures for a tree's two suites (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1), and the queue's record already reads smith's suites under them, so no test or code changes to make them hold.

Add one comment line beside the entries, under those for hearth's suites, in these words:

```toml
# Smith's focused and fuzzy suites: 30 s and 120 s of the strategy's section 8, on as CPU seconds; whether those are their figures stays open.
```

## Limits

- **Whether these figures are CPU seconds per tree is an open question, and it stays open.** Don't write that the user set them, and don't change `smith/docs/design/testing.md` or the strategy.
- Change `tools/tiers/budgets.toml` alone. No other entry, tree or figure. If another tree's entries have landed beside them by the time this lands, keep both.
- One direct task, low effort, no parts. Its check is `tools/tiers/test_cpu.py`, which validates the file's shape. Nothing is measured in this plan.
- `walk = false`.

## Sources

- `tools/tiers/budgets.toml`

## For every task of this plan

- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
