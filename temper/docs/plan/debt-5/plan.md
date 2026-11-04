# debt-5: the plan

From `inputs/request.md` (fabrig/debt, 2026-11-04): four debt parts, one per tree. `notes/common.md`
holds the standing rules every task names.

This file grows with each push. What is listed here is what has been released so far.

## Parts and tasks

1. **temper-items** (temper D007, D008, D009, D010), slice 1.
   - `tasks/temper-docs-guide.md`, medium, ~20 minutes, needs nothing.
   - `tasks/temper-pipeline-names.md`, docs, medium, ~35 minutes, needs nothing.
   - `tasks/temper-account-counts.md`, medium, ~35 minutes, needs nothing.
   - `tasks/temper-fixtures-kit.md`, migration, medium, ~60 minutes, needs nothing.

2. **smith-lifecycles** (`inputs/smith-lifecycles-review.md`, findings 1 to 10; `skein/docs/foundation/programming-model.md, 5.4`),
   slice 2. `notes/smith-review.md` holds the check against the code. No finding is skipped.
   - `tasks/smith-domain-test-helpers.md`, medium, ~25 minutes, needs nothing: finding 2.
   - `tasks/smith-domain-machines.md`, medium, ~30 minutes, needs the last: findings 7, 8, 9.
   - `tasks/smith-agent-test-helpers.md`, medium, ~20 minutes, needs nothing: finding 2.
   - `tasks/smith-agent-machines.md`, medium, ~50 minutes, needs the last: findings 1, 3, 4, 9, 10.
   - `tasks/smith-host-io.md`, medium, ~40 minutes, needs nothing: findings 5, 6, 9, 10.

3. **skein-forge-root** (skein D010; `skein/docs/design/forge.md, 4.4`), slice 3. `notes/forge-root.md`
   holds the layout, the calls and what runs. The cut wrote 4.4 and its **Not yet**.
   - `tasks/forge-limits.md`, medium, ~25 minutes, needs nothing.
   - `tasks/forge-neutral-modules.md`, medium, ~35 minutes, needs the last.
   - `tasks/forge-decimal.md`, migration, medium, ~15 minutes, needs the last: the measure reads 0.
   - `tasks/forge-plan-failure.md`, medium, ~15 minutes, needs the last.
   - `tasks/forge-users.md`, medium, ~25 minutes, needs the last.
   - `tasks/connection-twins.md`, medium, ~20 minutes, needs `forge-plan-failure`.
   - `tasks/forge-root-close.md`, docs, medium, ~15 minutes, needs `forge-users` and `connection-twins`: the item goes.

Still being cut: jig D001.
