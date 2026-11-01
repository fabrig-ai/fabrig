+++
id = "pd-people-purge"
part = "pd"
kind = "feature"
needs = ["pd-acts"]
files = [
  "jig/crates/jig-domain-people/**",
  "jig/tests/people/**",
]
effort = "high"
minutes = 80
+++

## Goal

Give the people child's two deletion events their bodies (contracts J4): `EraseTask` drops
every party's inbox entry, read position and result reference naming one erased task;
`DropProject` drops the project's roster and every volatile reference to the project. A start
after a deletion restores the retained answers that name the gone project.

Acceptance:
- `Event::EraseTask { task, project }` leaves nothing in the people domain naming `task`:
  no `waiting`, `needs`, `replies`, `chat_totals` entry, no `recent`, `unread` or
  `result_census` reference, `chat_updates` lowered as `forget_chat` lowers it; it emits
  `Request::Erase { key: Key::ReadConversation { party, task } }` for every live position it
  holds for the task, and `Request::Erase { key: Key::EndedRead { task, party } }` for the
  ended positions it knows (the requester's, as `forget_event` erases it, whether or not the
  project still has roles). Repeating it is harmless.
- `Event::DropProject { project }` removes `roles[project]` and emits
  `Request::Erase { key: Key::Roles(project) }`; drops `waiting` under every
  `Whom::Role { project, .. }`, every `chat_updates` and `result_census` entry keyed by
  `(_, project)`, and every remaining volatile entry whose value names the project; afterwards
  `has_project(project)` is false, `project_members(project)` is `None`, and no party's
  `project_roles` lists it. Repeating it is harmless.
- A start whose store holds retained `Answer` or `Acted` rows naming a project without a
  roles row (for example `Outcome::RolesSet { project }`, `ServiceMade` in it, `Renamed`)
  restores: they stay retained answers, replayed as saved until their retention, then gone
  (contracts J1, J4). Today `restored` refuses such a `RolesSet` answer `Unknown`
  (`domain.rs` ~2128) and `domain/start.rs` `start_link` asserts the roles row (~90): both
  accept the absent project instead.
- Party records (`Stored::Person`), sign-ins and `ReadResults` positions are untouched: parties
  are the deployment's, not the project's.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J4 (Erasing and Releasing lists).
- `temper/docs/design/draft/playground-reset.md`, section 5 ("What outlives it").
- `jig/docs/design/domain/people.md`, sections 4 (roles) and 6 (the inbox).

## Owns and may change

Owned: `jig/crates/jig-domain-people/**` and its world `jig/tests/people/**`. Nothing outside.
The events' shapes are `pd-acts`'s: do not change them.

Options, pre-approved:
- If an ended position for a party other than the requester can exist in memory, erase it
  too; ended positions the people child does not hold are the core's to erase (they are
  task-prefixed store rows the core reads, `pd-core-erase`).
- If dropping a volatile entry needs a count the child keeps beside it (a capacity counter),
  lower it there as the existing removal paths do.

## Steps

1. `inbox.rs`: `erase_task(domain, limits, task, project, out)` composed of the existing
   removal paths: `replace_waiting(task, [])`'s effect on waiting/needs/replies, `forget_chat`,
   and a new removal of result references naming `task` from `unread`, `recent` and
   `result_census`. Route `Event::EraseTask` to it (after restore; before restore it is
   ignored as other runtime events are).
2. `domain.rs`: `drop_project(domain, project, out)` for `Event::DropProject`.
3. Restore: `restored` and `start_link` accept retained answers naming a project with no roles
   row; keep every other check (people references, kinds, shapes).
4. Tests, in `src/tests/` (new `deletion_purge.rs`, registered in `tests.rs`):
   - `erasing_a_task_drops_its_inbox_entries_positions_and_results` (a person task waiting,
     a chat with replies and a read position, an ended result in `unread`; after `EraseTask`
     nothing names the task and the position erase is emitted).
   - `erasing_a_task_twice_changes_nothing_more`.
   - `dropping_a_project_drops_its_roster_and_role_entries` (two projects; the other keeps
     its roles, entries and counts).
   - `a_retained_roles_answer_restores_after_its_project_is_dropped` (restore rows: a person,
     a `RolesSet { project: 2 }` answer, no `Roles(2)` row; restore completes ready).
   And in `jig/tests/people/tests/suite/` (new `deletion.rs`, registered in `main.rs`):
   `a_dropped_project_leaves_no_people_row_but_its_answers` (drive the world: roles, an
   answered `SetRoles`, `DropProject`; the store holds the answer and no `Roles` row; a restart
   restores ready).
   Commands: `heavy cargo nextest run -p jig-domain-people --lib -E 'test(/deletion_purge/)'`,
   `heavy cargo nextest run -p jig-people-world --test suite -E 'test(/deletion/)'`, then the
   whole `heavy cargo nextest run -p jig-domain-people --lib` and
   `heavy cargo nextest run -p jig-people-world --test suite` (the restore change is a change of
   meaning for people's readers).
5. `cargo fmt`; `heavy cargo clippy -p jig-domain-people -p jig-people-world --all-targets`;
   old forms; commit and push.

## Report

Each volatile structure you cleared and how; the restore checks you relaxed, with their old
refusal.
