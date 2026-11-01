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

## Answers

2026-11-01. The core world's `Observer::durable_at`
(`jig/tests/core/src/observations.rs`) panics "immutable activity row
disappeared" when the deletion erases a task's activity: may this task
change the observer? Yes, the minimal repair, as a neighbour edit of
that file:

- the observer learns each task's project from the `Live`/`Ended` rows
  it reads, and the projects removing from the `Removal` rows
  (`CoreKey::Removal(project)`) it has seen durable at this or an
  earlier commit (a removal never goes back);
- an activity row of a task of a removing project may disappear: its
  entry leaves the observer's `activity` map, and its `Ended` row may go
  with it. Any other disappearance or change panics as today; the
  existing `activity_referee_rejects_changed_or_erased_lines` passes
  unchanged;
- add `activity_of_a_removing_project_may_be_erased` to its
  `activity_tests`: a task's `Ended` row of project P, its activity
  row, a `Removal { project: P, stage: Erasing }` row, then the activity
  erased: accepted; and the same erasure for a task of another project:
  rejected;
- another observer or referee check the deletion trips the same way (a
  row of the removing project, or of one of its tasks, that the check
  holds may never go) gets the same rule in the same file, each named
  in the report's Changed; a check that fails for another reason, or a
  referee (`r::Referee`) change, is a question.

2026-11-01. A restart after `Ending` is durable refuses at Switch,
because `restart/joins.rs` and `start/links.rs` require the project's
policy, which the removal's close erased: may this task change project
resolution there? Yes, as neighbour edits of those two files:

- one helper beside `task_references_resolve` in `restart/joins.rs`,
  `project_resolves(&self, project) -> bool`: the project's policy, or
  a kept `Removal` row for it (`self.removals`; a removal is restored
  with the core's rows, before Switch). `task_references_resolve` and
  the placed-hold check in `unresolved_tasks` use it in place of
  `authority.policy(..).is_some()`; `start/links.rs` `link_project`
  asserts it the same way. Every other startup check stays as it is;
- add to `src/tests/restart_checks.rs`
  `a_live_task_of_a_removing_project_resolves_before_switch`: a live
  task of a project with no policy and a kept `Removal` resolves, and
  the same task without the `Removal` does not (the existing tests pass
  unchanged);
- the cold-start unit case for a persisted `Releasing` row, which the
  run left for after this, is then added to `src/tests/removal_stages.rs`.

2026-11-01, from the same run's "Left". `erase.rs` `erase_record`
leaves the `Delegate { requester, task }` index of an erased delegate
task, so `rows_naming` finds it: may this task erase it? Yes, as a
neighbour edit of that function: the `tasks::Party::Task(requester)` arm
erases `tasks::Key::Delegate { requester, task }` beside
`EndedDelegate`; the delegate test then goes green with no workaround in
`routing/removal.rs`.

2026-11-01, from the same run's "Left". The existing
`src/tests/removal.rs` asserts every saved `Removal` is `Ending`, which
this task's controller makes untrue for a project with no live task
(`Erasing` in the same decision): may this task change it? Yes, that
assertion only: a project with a live task saves `Ending`; one with
none saves `Ending` and then `Erasing` (or later) in the same decision.

2026-11-01. The `start/script.rs` `Phase::Open` call is unreachable from
the roots, which all open through the legacy `restart_done(Open)`: add a
resume entry point and call it from the roots, or move the roots to the
newer start API? The entry point. Moving the roots is out of this task.
These are neighbour edits:

- `routing/mod.rs`, beside `resume_fleet` and in its shape: `pub fn
  resume_removals(core: &mut Core, env: &Env<Limits>) -> Requests`.
  When stopping it is empty, as `resume_fleet` is. Otherwise it runs
  `removal::resume` and drains the sibling work through `step`, as
  `Phase::Open` does now, then `finish_extent`. It asserts the core is
  open (`Restart::Open`, or the start script past `Open`).
  Re-export it in `lib.rs` beside `resume_fleet`. `start/script.rs`
  `Phase::Open` calls it and pushes its output into `out`, so that
  both paths run one body.
- Each root's existing `RestartStep::Open` arm, in the same decision
  right after `restart_done(step)`, calls `resume_removals` and routes
  its `Requests` the way that root routes `resume_fleet`'s:
  `jig/testing/jig-test-domain/src/lib.rs` (`route_core`),
  `jig/examples/ops/domain/src/restart.rs` (its core routing, before
  `SendCloses`) and `temper/crates/temper-engine-domain/src/restart.rs`
  (`route_core_requests`). Any other root found calling
  `restart_done` with `Open` gets the same call. No other line of
  those files changes. `restart_done`'s signature and the stored
  format stay as they are.
- Test: add a case to `src/tests/removal_stages.rs` where a persisted
  `Erasing` row, restored through the legacy restart to `Open` and then
  passed to `resume_removals`, sweeps `Ended` from the first task.
  The crash case then goes green with no other change. The step-4
  cross-tree `cargo check` covers the three roots.

2026-11-01. Run 4's push to `playground-reset-r4/pd-delete` was refused
as non-fast-forward: may its work go to a fresh branch
(`playground-reset-r4/pd-delete-4`), or does fab reconcile the existing
one? Neither: the work goes to `playground-reset-r4/pd-delete`, made a
fast-forward by a merge in the run's own branch. fab finds a task's work
by its branch's name, so a fresh name would leave it unmerged and the
stale branch in its place. The branch on origin holds run 3's checkpoint
(`9cb9a14b91`), which the run's base (a newer trunk) does not contain;
run 4's commits (`ba6eb013b2`, `d20446d94c`) exist only in the slot it
ran in. The next run, before its first edit:

- if `git cat-file -e d20446d94ceb82f3991d4c9af4952a37b4f82046` succeeds
  in its checkout, `git merge --no-edit d20446d94c` (run 4's work), then
  `git merge -s ours --no-edit origin/playground-reset-r4/pd-delete`,
  the message saying run 4's commits supersede run 3's checkpoint;
- otherwise `git merge --no-edit origin/playground-reset-r4/pd-delete`
  (run 3's checkpoint) and carries on from it through the answers above;
  run 4's report (`fab/runs/playground-reset-r4/pd-delete-4/report.md`
  on the dispatcher) says what it built.

Either way the push is the plain `git push origin
HEAD:refs/heads/playground-reset-r4/pd-delete`; no rebase, no force,
no other branch. A push refused again is reported, never worked round.

2026-11-01, from run 4's "Left". `src/tests/erase.rs`
`a_task_requesters_ended_delegate_edge_is_erased` expects the exact set
erased, without the `Delegate` index the approved `erase_record` repair
now erases: may this task change it? Yes, that expected set only: add
`Key::Tasks(tasks::Key::Delegate { requester: PARENT, task: TASK })`
beside the `EndedDelegate` key.

2026-11-01, from run 4's "Left". `jig-test-domain`
`tests::link::stopping_continues_a_projection_deferred_watch_without_another_commit`
fails at "the goal is live before Stop", and whether it fails without
this task is unknown. The task changes that root's `RestartStep::Open`
arm, so the failure may be its own: run that test once on the run's base
before the first edit (the run's merges above aside). Failing there, it
is inherited: name it under Left with both logs, it is not this task's.
Passing there, it is this task's: `resume_removals` with no kept removal
must route nothing and leave the root's decision as it was; fix it in
`routing/mod.rs` or the root's Open arm, no other line.

2026-11-01, on four runs without landing: does the task need recutting?
No. Each run stopped at a different boundary (the observer, startup
resolution, the legacy Open, the push), each now answered above, and run
4 built the whole task with its checks green but the two `Left` items
answered above. The next run starts with the merges above, then applies
the `erase.rs` expected-set answer and the `jig-test-domain` link answer,
reruns step 4's checks and pushes. A boundary the answers above already
grant is not asked again; only a new one is a question.
