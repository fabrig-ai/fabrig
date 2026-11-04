# D014: `TaskRecord` unions every executor's state in 78 fields

- **Kind:** complexity
- **Where:** `jig-domain-tasks/src`, `TaskRecord`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (A3 F5; "reshape the record (a stored format, so the user's call)")
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Tasks (A3)": the 76-field `TaskRecord`)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (a task's state says which executor it is; dead arms go)
- **Size:** L
- **Status:** open
- **Paid when:** a task's record holds its common fields and one executor-specific part, so a field that cannot apply to the executor cannot be set.
- **Measure:** `awk '/^pub struct TaskRecord \{/,/^\}/' jig/crates/jig-domain-tasks/src/*.rs | grep -cE '^\s+pub [a-z_]+:'` (78 at base)

## What

`TaskRecord` has 78 `pub` fields (76 at the quality run): charter runs', persons',
procedures' and conversations' state side by side, most of them empty for any one task.
Code that reads a record matches on the phase and then checks fields that its executor
cannot have.

## Why it matters

Every new executor field widens every task; readers must know which fields are live for
which executor.

## What pays it back

A record with one executor part. The record is a stored format (`tasks_v9`), so the
change is the user's call (`AGENTS.md`, 7) and needs a store version step.
