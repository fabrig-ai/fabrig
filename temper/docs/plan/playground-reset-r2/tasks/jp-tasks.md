+++
id = "jp-tasks"
part = "project-delete"
kind = "feature"
needs = ["jig-acts"]
files = [
  "jig/crates/jig-domain-tasks/src/drop_project.rs",
  "jig/crates/jig-domain-tasks/src/tests/drop_project.rs",
  "jig/tests/tasks/tests/suite/drop_project.rs",
]
effort = "high"
minutes = 60
+++

## Goal

Give the task hub what a project's deletion asks of it (playground-reset.md,
5): the live roots of a project, for the deletion to cancel, and, once
none of its tasks is live, the project's own rows in the tasks family
dropped: its open ledgers closed to history (spend outlives the project:
`notes/contracts.md`, 1), its placed holds and person proposals erased,
and its per-project state let go, so no later period, pool or recurring
start names it.

Acceptance (`jig-tasks-world`, new `tests/suite/drop_project.rs`):

- `live_roots_are_the_projects_trees`: two projects, each with a person's
  conversation that delegated once and a deployment task; `live_roots(1)`
  gives project 1's conversation and deployment task, not the delegate,
  and nothing of project 2.
- `dropping_a_project_closes_its_ledgers_to_history`: a project with a
  current period, a person's pool and a recurring allotment, spend posted
  on each: after `Event::DropProject`, each `Ledger` row is erased and a
  `ClosedLedger` row saved with the same counters, the pools' spend posted
  into their period as `retire` posts it; `domain.funding` holds nothing
  of the project; the other project's ledgers are untouched.
- `dropping_a_project_erases_its_holds_and_proposals`: a placed hold and
  a pending person proposal of the project are erased (their keys
  `Erase`d, gone from the domain); the other project's stay.
- `a_dropped_project_opens_no_period`: after the drop, a period timer or
  `PeriodNext` for the project does nothing and saves nothing, and a
  recurring start for it makes no task.
- `dropping_a_project_with_a_live_task_is_refused`: with a live task of
  the project, the drop changes nothing and emits
  `Request::DropRefused { project }` (new), which the core treats as a
  fault of its own order.
- A restart after the drop restores none of the project's ledgers (the
  closed rows are history, not restored): a case in the world's restart
  suite style, `a_dropped_projects_ledgers_stay_closed_across_a_restart`.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 1
  (spend outlives the project).
- `temper/docs/design/draft/playground-reset.md`, section 5 ("Then its own
  rows", "What outlives it").
- `jig/docs/design/domain/tasks.md`: funding (periods, pools, recurring),
  placed holds, person proposals (rg the section headings).
- `jig/docs/design/protocol/store.md`, 15.1 (closed funding to history).
- `jig/crates/jig-domain-tasks/src/retire.rs` (how a ledger closes).

## Owns and may change

Owns the files in the header. May change, as `notes/scope.md` says:
`jig-domain-tasks/src/boundary.rs` (the new `Event` and `Request`
variants), `domain.rs` (the dispatch arm, the accessor), `lib.rs` (the
`mod` line), `retire.rs` and `funders.rs` (expose `close` and the pool
posting as `pub(crate)` for reuse, their behavior unchanged),
`periods.rs` and `recurring.rs` (a dropped project ignored where a
period or a recurring start names it), `tests.rs` and the world's
`tests/suite/main.rs` (the `mod` lines), and the core's exhaustive matches
on `tasks::Request` (`jig-domain-core/src/routing/tasks.rs`'s `tag_tasks`:
`DropRefused` panics with a message naming the core's order, as other
impossible requests do there, or is logged as they are).

Not yours: deciding when to drop (`jp-delete`); the core's task erasure.

## Steps

1. `boundary.rs`:

   ```rust
   // Event:
   /// A deleted project's rows in this family, once none of its tasks is
   /// live (temper/docs/design/draft/playground-reset.md, 5).
   DropProject { project: u32 },
   // Request:
   /// A drop asked while a task of the project was live: nothing changed.
   DropRefused { project: u32 },
   ```

   `Domain`: `pub fn live_roots(&self, project: u32, out: &mut List<u64>)
   -> Option<()>` (live tasks of the project whose requester is not a
   task, in task order; `None` when `out` is full) and `pub fn
   live_in(&self, project: u32) -> bool`.

2. `drop_project.rs`: `pub(crate) fn drop_project(domain, env, project,
   out)`. With a live task of the project, `DropRefused`. Otherwise:
   - its `Funder::Pool { project, .. }` and `Funder::Recurring { project,
     .. }` ledgers posted into their period and closed as `retire`
     closes a pool (whatever their period's age), then its
     `Funder::Period { project, .. }` ledgers closed; a reservation left
     on any is released first (none should remain with no live task:
     debug-assert it);
   - its `placed` holds and `person_proposals` erased (`Request::Erase`
     of each key) and removed;
   - any per-project period or recurring state the domain keeps (rg
     `project` in `periods.rs`, `recurring.rs`, `funders/`) removed, so
     a later timer or event naming the project is ignored.

3. Tests: the acceptance's, in the world's style (`tests/suite/funding.rs`
   and `recurring.rs` are the nearest). Then, one at a time:
   - `heavy cargo nextest run -p jig-tasks-world --test suite -E 'test(/drop_project/)'`
     (the binary name from `jig/tests/tasks/Cargo.toml`);
   - `heavy cargo nextest run -p jig-domain-tasks -p jig-tasks-world`;
   - `heavy cargo nextest run -p jig-domain-core -p jig-core-world`;
   - `cargo fmt`; clippy over the packages you edited; old forms from `jig/`.
4. Commit and push.

## Report

The standard report, plus: which per-project state the domain held and
how each is dropped; whether any reservation was found at drop time in
the tests.
