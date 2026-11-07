# temper-fuzzy-gate: plan

Temper's fuzzy suite gets its gate entry in `tools/tiers/budgets.toml`: `[gate.temper.fuzzy]` with `cpu_suite_s = 120` after `[gate.jig.fuzzy]`, and one comment line after Jig's, by the user's word of 11-07 (`temper/docs/plan/temper-fuzzy-gate/inputs/request.md`).

## Parts and tasks

- **entry** (direct, slice 1, no walk): task `entry` (`tasks/entry.md`), medium effort, about 10 minutes, needing nothing. It owns `tools/tiers/budgets.toml` and adds four lines (the comment, the table, its key, a blank line). Its checks are `tools/tiers/test_cpu.py`, a validation of the real file through `cpu.py`'s `budgets()`, and the citation checker.

## Critical path

`entry` alone. It starts at the first push and the plan lands with it.

## What was decided

- **No design is written.** The request limits the plan to `budgets.toml`. Nothing under `temper/docs/design/` names a figure for temper's fuzzy suite, so nothing there goes stale. `skein/docs/foundation/testing-strategy.md`, 8 and 8.1, already gives the 120-second figure that the part cites.
- **Left by the request's limit:** `tools/tiers/design.md`, 7.3, lists Smith's, Skein's and Jig's gate entries and not temper's, and its section 8 still calls those figures open. The figures are 30 and 120 CPU seconds by the user's word of 11-07. The tiers' design should catch up with `budgets.toml` in a plan of its own. This one leaves it as the request asks.
- **Nothing is measured or added to a test.** The queue's record reads the suite at 102 to 105 CPU seconds over its last six full runs. The request names `test_cpu.py` as the check.
- **Kind `feature`, effort `medium`.** The request asks for low effort, but a spec takes only high or medium.

## Overlaps

Of the drafts beside this one, none of ui-ask-m1's branches touches `tools/tiers/budgets.toml`. The queued drafts' files are not known yet. If one of them lands a change to `budgets.toml` first, the task's pre-approved option keeps every entry and places the new lines relative to Jig's.
