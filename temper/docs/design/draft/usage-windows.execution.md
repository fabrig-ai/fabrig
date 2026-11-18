# Usage windows: execution

The milestones that carry [usage-windows.md](usage-windows.md) into the design and the code.

| Milestone | Executes | Needs | Its walk | State |
|---|---|---|---|---|
| U1, usage windows from runs and reads: every `rate_limit_event` reported as the account's window; a usage read for a program source when no run has reported for 30 minutes, and at a limited window's reset; both kept as kind 7 facts; each window's rolling one-hour rate served beside it, and Accounts' pace drawn from it | `usage-windows.md`, 2, 3 and 5 | none | a Claude subscription's row on Accounts, while a goal runs on it and again when the deployment is idle, shows its week and, near its limit, its 5-hour window | not started |

## Order

U1 alone. The look request `accounts-usage` draws what it writes.
