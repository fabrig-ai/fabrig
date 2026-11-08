# limits-raise: four budgets near their limit rise by half, for now

The user, on 11-08: "if any limit is close to causing reds, just raise it 50% and will revisit". Four figures in `tools/tiers/budgets.toml` are over or close:

| Entry | Figure | Reads | New figure |
|---|---:|---|---:|
| `[browser-temper]` | 85 | 91.3, the day's median on 11-08 (over) | 127.5 |
| `[process-temper]` | 360 | 363 on the morning of 11-08 (over); 307 since its repair, with runs up to 334 | 540 |
| `[browser-skein]` | 145 | 137.6, the day's median on 11-07 | 217.5 |
| `[gate.temper.fuzzy]` | 120 | 107.2 in one process; up to 129.2 under nextest | 180 |

## What's wanted

1. **Change those four `cpu_suite_s` figures to the new ones.** Every other field and entry stays as it is.
2. **Update the file's comment lines that give a reason for each figure.** Each one names the new figure, and says it was raised by half on 11-08, by the user's word, for now, to be revisited. The fuzzy line, for example: "Temper's fuzzy suite: 180 CPU seconds, raised by half from the strategy's 120 by the user's word of 11-08, to be revisited." The header's reasoning for the other entries stays.
3. **Wherever a document states one of these four figures as the current one, it changes with it, saying the same.** That means `tools/tiers/design.md` (388, temper's fuzzy entries) and any `docs/design` or `docs/development` page that gives `browser-temper`'s, `process-temper`'s or `browser-skein`'s figure as current. The strategy's own 30 and 120 stay: they're the rule, and this is an exception to it. Historical accounts, such as plans and dated measurements, stay as they were.
4. **Fix any test of `tools/tiers` that pins one of the old figures** so that it reads the file rather than a copy, or names the new figure.

## Limits

- No other figure changes. Nothing is added to or removed from the file.
- `walk = false`. Effort low. One task.

## Sources

- `tools/tiers/budgets.toml`
- `tools/tiers/design.md`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands.** Never `tools/tiers/run`, nor a browser, process or live tier. No walk, render check or throwaway driver.
- **Decide, don't ask.** A small edit just outside a task's file list is made, and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
