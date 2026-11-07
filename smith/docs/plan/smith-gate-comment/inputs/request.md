# smith-gate-comment: smith's gate entries carry the user's word

`tools/tiers/budgets.toml` holds `[gate.smith.default]` at 30 and `[gate.smith.fuzzy]` at 120. The comment beside them says whether those are their figures "stays open". The user has since confirmed that they are, as CPU seconds, on 11-07. This plan says so, and nothing else.

Replace this line:

```toml
# Smith's focused and fuzzy suites: 30 s and 120 s of the strategy's section 8, on as CPU seconds; whether those are their figures stays open.
```

with this one:

```toml
# Smith's focused and fuzzy suites: 30 and 120 CPU seconds, the strategy's section 8, by the user's word of 11-07.
```

## Limits

- Change that one line of `tools/tiers/budgets.toml` alone. No entry, figure, other tree's comment, or doc changes. Other trees' lines say the same and stay as they are: that word was given for smith's entries.
- One direct task, low effort, no parts. Its check is `tools/tiers/test_cpu.py`, which validates the file's shape. Nothing is measured in this plan.
- `walk = false`.

## Sources

- `tools/tiers/budgets.toml`

## For every task of this plan

- **Only focused and fuzzy run in a plan.** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure. This overrides anything below or in a spec.
- **Decide, don't ask.** A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
