+++
id = "deployment-release"
part = "deployment-release"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-tasks/src/control.rs",
  "jig/crates/jig-domain-tasks/src/tests/control.rs",
  "jig/docs/design/domain/tasks.status.md",
]
effort = "high"
minutes = 35
+++

# The deployment releases a procedure from a hold of its own

## Goal

A held change whose cause has gone must go on with no person asked (`temper/docs/design/domain/forge.md`, 8.4), and today jig lets the deployment cancel a task but never release one. Add the one release, generic to any connector's procedure, and nothing else. jig names no temper here, in code or comment.

Acceptance:

- `Event::Release` with `by: Party::Deployment { project }` releases a procedure task of that project held by its own `Procedure`, `EffectFailed`, `Stalled` or `Budget` hold, with no placed hold. The task returns where it was and `due_reason` is `Why::Released`, as any release. Its history records the deployment as `by`.
- It stays `Refusal::Reference` for: another project's task; an agent, job or person task; a task with a placed hold; a task held for any other reason (`Stopped`, `Placed`, `Drift`, `Effects`, `Failures`, `Dependency`, and the rest); a task not held. A refused release changes nothing.
- `crate::control::allowed` still answers false for the deployment: amend, move and message are not widened.
- `tasks.status.md`'s entry for it is gone.

## Inputs

- `jig/docs/design/domain/tasks.md`, section 7, **Who may**; 5.5.
- `jig/docs/design/domain/connectors.md`, 6.3.
- `temper/docs/plan/repair-green/notes/common.md`; `notes/contracts.md`, **The deployment's release**.

## Owns and may change

- Owns `jig/crates/jig-domain-tasks/src/control.rs` (`control_entrance` and a helper), `jig/crates/jig-domain-tasks/src/tests/control.rs`.
- `jig/docs/design/domain/tasks.status.md`: delete `7: the deployment's release of a procedure from a hold of its own;` alone (`notes/common.md`).
- If `check_release` or `placed::check_release` needs a guard to hold the refusals, add it there, and say so in the report. No change to `Release`, `Party` or `Refusal`: if one looks needed, that is a `Question:`.

## Steps

`control_entrance` today:

```rust
Party::Deployment { project } => {
    cancelling
        && match record(domain, number) {
            Some(task) => task.project == project,
            None => false,
        }
}
```

Keep its shape: the project's check stands, and `cancelling ||` a new `own_hold(task)` beside it. Sketch:

```rust
// A procedure's own hold, which its connector may find gone (domain/tasks.md, 7).
fn own_hold(task: &TaskRecord) -> bool;
```

`Executor::Procedure { .. }`, `task.placed` none, `Phase::Held { why, .. }` with `why` as above. List every `Hold` and `Phase` and `Executor` variant: no `_`, no `matches!`.

Tests in `tests/control.rs`. The file's doc line says deployment control stays within cancellation: make it say what is now true. `a_deployment_cancels_a_task_of_its_project_and_no_other` keeps its rows: its task is no procedure held, so release stays refused there. Add:

- `a_deployment_releases_a_procedure_from_its_own_hold`: for each of the four holds, a procedure task (`tests/changed.rs` shows `Executor::Procedure`), held through `Event::Hold` or `Event::HoldFrom`; released; the phase is back, `due_reason` is `Released`, the history row's `by` is the deployment.
- `a_deployment_releases_nothing_else`: a table of the refusals above, each `Refusal::Reference`, the record equal before and after.

Break the rule by hand for each (drop the executor check; drop the placed check) and see each fail. Then `cargo fmt`, `python3 -I -B scripts/old_forms.py` from `jig/`, `heavy cargo clippy -p jig-domain-tasks --all-targets -- -D warnings`, `heavy cargo nextest run -p jig-domain-tasks --lib`, `heavy cargo nextest run -p jig-domain-core`. Commit and push.

## Report

As the implementer's text says, and whether a guard went beyond `control_entrance`.
