# D012: tasks spells one set of operations several ways

- **Kind:** duplication
- **Where:** `jig-domain-tasks/src/compound.rs`, `jig-domain-tasks/src/control.rs`, `jig-domain-tasks/src/boundary.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T5, A3 F3, F2)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Tasks (A3)": the operation vocabulary written four ways, near-duplicate boundary families)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (one operation type to route, refuse and apply)
- **Size:** M
- **Status:** open
- **Paid when:** a cancel, release, move, amend or message of a task has one type in tasks, used by party operations, tool controls, procedures and events alike.

## What

- `Operation` (`compound.rs`): `Cancel`, `Move`, `Amend`, `Release`, `Message`, `Stop`, each with its task;
- `Control` (`control.rs`): `Cancel` and `Release` again, with the same fields less the task;
- `ProcedureInput` (`boundary.rs`, line 2221), and `Event`'s own `Amend`, `Move` and `Message` (`boundary.rs`, from line 1576).

The made-up `ReplyTo(u64::MAX - 1)` tokens the review saw are gone. `boundary.rs` is
2,252 lines, the largest production file of tasks (D039).

## Why it matters

Each spelling has its own checks and its own routing; a new operation, or a new field on
one, is added three or four times.

## What pays it back

One `Operation` carried by each entrance, with the entrance's own context (who asked,
under which claim) beside it, not inside a copy of the variants.
