+++
id = "pd-delete"
part = "project-deletion"
kind = "feature"
needs = ["pd-project-rows"]
files = [
  "jig/crates/jig-domain-core/src/routing/removal.rs",
  "jig/crates/jig-domain-core/src/tests/removal_stages.rs",
  "jig/tests/core/tests/suite/removal_stages.rs",
  "jig/tests/core/src/removal.rs",
]
effort = "high"
minutes = 60
+++

## Goal

The task after which jig's `RemoveProject` (request and act) deletes a
project whole (`designers/ids.md`, `pd-delete`; draft 5, "Kept until
whole"): the `Removal` row moves through its stages, each resumed at
every start, until its last step erases it:

- **`Ending`** (from `pd-close`): when the last live task of the
  project ends, the row moves to `Erasing`. At a start, any live root
  task of the project is cancelled again (`cancel_live`), and with none
  live the row moves on.
- **`Erasing`:** the project's ended tasks are found by sweeping the
  `Ended` family (`Sweep::Ended`), each erased whole in turn
  (`erase::erase_task`, `pd-erase-task`); with none left, the
  project's own rows (`project_rows::erase_rows`, `pd-project-rows`);
  then the row moves to `Releasing`. At a start, the sweep begins again
  from the first task.
- **`Releasing`:** `connector::Ask::ReleaseProject { request, project }`
  to every configured connector (`Core.connectors`), one request token
  each, inside `vocabulary::Ask::Effect`; each `ProjectReleased` is
  marked; when every connector has answered, people's
  `Event::ProjectGone` (retained answers naming it), and the `Removal`
  row erased: the deletion is whole. At a start, the asks are sent
  again (contracts, section 4).

Acceptance: tests below; jig-acts' "nothing yet" arm for
`ProjectReleased` (`effects.rs`) is replaced by the marking; a
deletion cut by a crash after each stage's commit resumes and ends with
no row naming the project or any of its tasks, but its `ClosedLedger`
rows.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 4,
  D3, D5).
- `temper/docs/design/draft/playground-reset.md`, sections 5 and 8.
- `jig/docs/design/domain/people.md`, 5.3 ("Across a restart");
  `jig/docs/design/protocol/store.md`, 7.2 (restart's ranges), 11.1.
- `jig/crates/jig-domain-core/src/removal.rs` (`Removal`,
  `RemovalStage`), `routing/removal.rs` (`pd-close`), `erase.rs`
  (`erase_task`, `finished`), `project_rows.rs` (`erase_rows`,
  `rows_done`, `swept`), `routing/tasks.rs` (`tasks::Request::Ended`,
  where a task's end reaches the core), `start/script.rs`
  (`Phase::Open`: the core ready).

## Owns and may change

Owned: the `files` above. Neighbours, the line or function named:

- `erase.rs` `finished` and `project_rows.rs` `rows_done` (their
  bodies: call into `routing/removal.rs`), `project_rows.rs` `swept`
  (a page whose owner is the removal's `Ended` sweep goes to
  `removal::swept`);
- `routing/tasks.rs` (one call on `tasks::Request::Ended` for a task of
  a removing project), `start/script.rs` (one call at `Phase::Open`:
  resume every `Removal`), `effects.rs` (`ProjectReleased`'s arm),
  `lib.rs` (fields for the removal's volatile state: the `Ended`
  cursor, the release tokens), `tests.rs` (`mod removal_stages;`);
- `jig/tests/core/tests/suite/main.rs` (`mod removal_stages;`),
  `jig/tests/core/src/lib.rs` (`pub mod removal;`).

Pre-approved:

- one project's erasure runs one task at a time; several removing
  projects may run at once, each its own;
- a page of the `Ended` sweep holding none of the project's tasks
  continues from its last task's number; one holding several erases
  the first and sweeps again after it;
- a `ProjectReleased` with a token the removal does not hold (a resend's
  late answer) is dropped.

## Steps

1. `routing/removal.rs`: `task_ended(core, env, work, out, project)`,
   `resume(core, env, work, out)` (all rows, at open),
   `swept(core, env, work, owner, rows, more) -> Requests` (the `Ended`
   sweep), `erased(core, env, work, out, project, task)` (from
   `finished`), `rows_erased(core, env, work, out, project)` (from
   `rows_done`), `released(core, work, out, request, project)` (from
   `effects.rs`), each stage change saving the row in the same
   decision.
2. `jig/tests/core/src/removal.rs`: `pub fn rows_naming(world:
   &World, project: u32, tasks: &[u64]) -> Vec<root::Key>`: every
   stored row whose key or value names the project or one of the
   tasks, but `ClosedLedger` rows of the project's period and pools;
   the tasks are the project's, collected by the test before the
   deletion.
3. Tests:
   - `jig-domain-core` `src/tests/removal_stages.rs`:
     `the_last_task_ending_moves_the_row_to_erasing`,
     `releasing_waits_for_every_connector_then_erases_the_row`;
   - `jig-core-world` `tests/suite/removal_stages.rs`:
     `a_project_with_ended_and_live_chats_is_deleted_whole` (chats,
     a delegate, notes, a pool; `rows_naming` empty at the end, the
     transcript files gone, `ProjectReleased` heard from each
     connector);
     `a_deletion_cut_by_a_crash_after_each_stage_ends_whole` (crash
     after `Ending`, mid-`Erasing`, before `Releasing`'s answers; each
     restart ends with `rows_naming` empty and no `Removal` row);
     `a_retried_remove_after_the_deletion_answers_unknown`;
     `the_deleted_projects_name_makes_a_new_project`.
4. Checks: `cargo fmt`; clippy on `jig-domain-core` and
   `jig-core-world`; `heavy cargo nextest run -p jig-domain-core
   --lib`, `-p jig-core-world --test suite`; `fuzzy` for the core
   world's crash simulation if it reaches the removal (the failing test
   alone); `heavy cargo check --tests -p jig-conformance-world -p
   jig-ops-domain -p temper-engine-domain -p temper-engine-domain-world`.
   Commit, push.

## Report

fab's sections; `## Interfaces`: the routing functions, `rows_naming`;
each stage's resume rule as built; any row kind `rows_naming` found
that no task erases (say which, and leave the test red only if it is
yours).
