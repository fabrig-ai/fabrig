+++
id = "pd-close"
part = "project-deletion"
kind = "feature"
needs = ["pd-cancel", "pd-wire", "pd-people-gone"]
files = [
  "jig/crates/jig-domain-core/src/routing/removal.rs",
  "jig/crates/jig-domain-core/src/routing/deployment_acts.rs",
  "jig/crates/jig-domain-core/src/routing/people.rs",
  "jig/crates/jig-domain-core/src/routing/projects.rs",
  "jig/crates/jig-domain-core/src/projects.rs",
  "jig/crates/jig-domain-core/src/tests/removal.rs",
  "jig/crates/jig-domain-tasks/src/domain.rs",
  "jig/crates/jig-domain-people/src/domain.rs",
  "jig/crates/jig-domain-people/src/domain/acts.rs",
  "jig/tests/core/tests/suite/removal.rs",
  "jig/tests/core/tests/suite/main.rs",
]
effort = "high"
minutes = 60
+++

## Goal

`RemoveProject`'s decision, request and act (draft 5, "Closed at once"
and "Its tasks ended"): in one decision the project is closed, gone to
every list and request, its deletion kept as a `Removal` row at stage
`Ending`, its live tasks cancelled by the deployment, and the request
answered `ProjectRemoved`. What follows the last task's end is
`pd-delete`'s.

Acceptance:

- a project owner's `Ask::RemoveProject`, and a granted
  `Act::RemoveProject`, answer `ProjectRemoved { project }`; a party
  who is no owner of it is refused as `project_admission` refuses
  (`pd-wire`); an unknown or already removing project is refused as an
  unknown project is today; jig-acts' *not offered* lines for both are
  gone;
- in that decision: `Removal { project, stage: Ending }` saved; the
  project out of `Core.projects` and its name out of `Core.names` (its
  name free again: draft 5), its authority policy dropped
  (`authority::Event::Dropped`), people's `Event::ProjectClosed` sent
  (`pd-people-gone`); every live root task of the project cancelled by
  the deployment with `pd-cancel`'s `cancel_live` (no grace, the reason
  "project deleted"), their delegates with them;
- after it, no new task, message, adoption or settings change names
  the project: each is refused as on a project that does not exist,
  and no list, `me` or document names it; the core's own makers
  (recurring and standing tasks, connector repairs, procedures, a
  connector's `Adopt` act) make nothing in it;
- tests below.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 4).
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `jig/docs/design/domain/people.md`, 5.1-5.3; `jig/docs/design/domain/tasks.md`,
  section 7 (the compound; cancelling a held or waiting task ends it at
  once).
- The models: `routing/projects.rs` `make_project_request`/
  `make_project_act` (a project decision in one commit with its
  answer); `routing/deployment_acts.rs` `cancel_live` (`pd-cancel`);
  `jig/crates/jig-domain-core/src/removal.rs` (`pd-store`'s row);
  `jig/crates/jig-domain-people/src/domain/removal.rs` (`pd-people-gone`).

## Owns and may change

Owned: the `files` above. Neighbours:

- every core maker of tasks and every core admission point that looks a
  project up (by `Core.projects`, `authority.policy(project)` or
  people's roles): a guard line where the project's absence is not
  already refused; list each in the report;
- `jig-domain-core/src/acts/objects.rs`: `project_admission` made
  `pub(crate)` (private at the base), nothing else;
- `jig-domain-core/src/lib.rs`: `mod`/field lines only (`pd-store`'s
  file, merged before this task);
- `jig/tests/core/src/**` (world helpers), `jig/testing/**`: helpers
  and arms only.

Pre-approved:

- the deletion's cancels need no answer: give their compounds a
  continuation that drops the tasks child's answer (a
  `PersonTaskRoute` variant or a request token kept apart); if the
  continuations' room is full, cancel what fits and leave the rest to
  `pd-delete`'s resume, which cancels any live root task of a removing
  project again;
- if dropping the authority policy at once breaks a live task's
  closing (its funding or release reads the policy), keep the policy
  until `pd-delete`'s last step and refuse by `Core.removals` instead,
  and say so.

## Steps

1. `jig-domain-tasks` `domain.rs`: `pub fn project_roots(&self,
   project: u32) -> Box<[u64]>`, the live tasks of the project whose
   requester is no task (roots), by number.
2. `routing/removal.rs` (new; module doc citing draft 5 and people.md
   5.2):

   ```rust
   pub(super) fn remove_request(core, env, work, out, request: Token, person: u64, project: u32);
   pub(super) fn remove_act(core, env, work, out, request: Token, project: u32);
   /// The decision both share: row, closing, cancels, answer.
   fn close(core, env, work, out, request: Token, project: u32);
   ```

   `routing/people.rs` routes the party's `Ask::RemoveProject` to
   `remove_request` after `acts::objects::project_admission`;
   `deployment_acts.rs` routes `Act::RemoveProject` to `remove_act`.
3. People: drop jig-acts' refusals (`domain.rs` `valid_ask`/
   `invalid_ask`, `domain/acts.rs` `valid`); the ask is of the
   project's scope, admitted like `Rename`.
4. Tests:
   - `jig-domain-core` `src/tests/removal.rs`:
     `a_removal_saves_its_row_and_closes_the_project_in_one_decision`
     (the writes of the decision: `Removal`, `Policy` and `Roles`
     erased, the answer held);
   - `jig-core-world` `tests/suite/removal.rs` (add `mod removal;` to
     `main.rs`):
     `an_owner_removes_a_project_and_its_live_chat_is_cancelled_by_the_deployment`;
     `a_member_who_is_no_owner_is_refused`;
     `a_removed_project_refuses_a_new_task_a_message_and_a_rename_as_unknown`;
     `a_removed_project_leaves_me_and_the_deployments_document`;
     `a_granted_connector_removes_a_project` (grant
     `ActKind::RemoveProject`);
     `a_project_removed_twice_answers_the_saved_answer_then_unknown`
     (the same key: saved `ProjectRemoved`; a new key: unknown);
     `a_waiting_and_a_held_task_end_at_once`.
5. Checks: `cargo fmt`; clippy on `jig-domain-core`,
   `jig-domain-tasks`, `jig-domain-people`; focused tests of those
   three (`--lib`), `heavy cargo nextest run -p jig-core-world --test
   suite`, `-p jig-people-world`, `-p jig-tasks-world` (this task
   changes what an unknown project's refusal covers: every reader runs);
   `heavy cargo check --tests` for `jig-conformance-world
   jig-ops-domain temper-engine-domain temper-engine-domain-world`.
   Commit, push.

## Report

fab's sections; `## Interfaces`: the routing functions, `project_roots`;
each guarded admission point and maker (file:line); how the cancels'
answers are absorbed.
