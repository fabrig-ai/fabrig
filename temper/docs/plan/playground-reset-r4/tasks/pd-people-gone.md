+++
id = "pd-people-gone"
part = "project-deletion"
kind = "feature"
needs = ["pd-store"]
files = [
  "jig/crates/jig-domain-people/src/boundary.rs",
  "jig/crates/jig-domain-people/src/domain.rs",
  "jig/crates/jig-domain-people/src/domain/removal.rs",
  "jig/crates/jig-domain-people/src/domain/parties.rs",
  "jig/crates/jig-domain-people/src/inbox.rs",
  "jig/crates/jig-domain-people/src/tests/removal.rs",
  "jig/crates/jig-domain-people/src/tests.rs",
  "jig/tests/people/src/**",
  "jig/tests/people/tests/suite/main.rs",
  "jig/tests/people/tests/suite/removal.rs",
]
effort = "high"
minutes = 50
+++

## Goal

The people child's part of a project's deletion (draft 5), as events
the core sends; nothing sends them yet (`pd-close` sends the first,
`pd-delete` the second, `pd-erase-task` the third):

- **`Event::ProjectClosed { project }`,** in the deletion's decision:
  the project leaves the people child at once. Its roles row and its
  policy copy are erased and dropped from memory, so no party holds a
  role in it, no `me` names it, and every later request naming it is
  refused as a request naming a project that does not exist is today.
  The parties the project made (its services) go with it, and their
  sign-ins.
- **`Event::ProjectGone { project }`,** at the deletion's last step:
  every retained answer naming the project (a party's `Answer` whose
  ask names it, a connector's `Acted` whose act names it) is erased, so
  the same key sent again is judged afresh and answers *unknown*, as an
  answer past its retention does (draft 5, "What outlives it").
- **`Event::TaskErased { task, project }`,** for each of the project's
  tasks as `pd-erase-task` erases it whole: every party's read
  positions of the task (`ReadConversation`, `EndedRead`) and inbox and
  conversation contributions naming it are dropped and their rows
  erased, as `Event::Forget` drops the requester's today, for every
  party (draft 5, "Each party's inbox and read positions drop what
  named its tasks").

Acceptance: the tests below pass; nothing else of the people child
changes meaning (its world's suites pass).

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 4).
- `jig/docs/design/domain/people.md`, sections 3 and 4 (parties, roles),
  5.1 (keyed answers, their retention), 5.2, 5.3 (retained acts).
- `jig/crates/jig-domain-people/src/boundary.rs` (`Stored::Roles`,
  `Stored::Policy`, `Stored::Answer`, `Stored::Acted`, `Event::Forget`
  as the model of an event that retires rows), `domain/acts.rs`
  (`acted`, `act_alarms`), `domain.rs` (`answers`, `answer_alarms`).

## Owns and may change

Owned: the `files` above. Neighbours: `domain/acts.rs`, one
`pub(super) fn` appended at its end that erases the retained acts
naming a project (its `Acted` fields are private there); `pd-cancel`
edits `valid` and `valid_outcome` in that file at the same time, so
touch nothing else in it. `jig-domain-core`'s people
routing (`routing/people.rs`), only for the compiler's arms for the new
events' `Request`s if any (the events themselves are sent from
`pd-close`/`pd-delete`); `jig/tests/core/**` and `jig/testing/**`
exhaustive matches, arms only.

Pre-approved:

- "names the project": an ask or act whose `project` field (or its
  batch's, its compound's, its scope's) is the project; a person-wide
  ask (sign-in, account change of a person) names none. Write it as one
  total function `names_project(&Ask, project) -> bool` (and one for
  `Act`), exhaustive, no wildcard.
- a service whose identity or record names its project goes with it;
  if services name no project in their records, a service whose only
  role was in the project goes; if neither can be told, leave services
  and say so in the report.
- if a party's in-memory inbox, read positions or conversation
  contributions are keyed by project here, drop the project's in
  `ProjectClosed` too; the per-task ones are `pd-erase-task`'s.

## Steps

1. `boundary.rs`: the three `Event` variants, each with its doc citing
   draft 5 and people.md 5.2.
2. `domain/removal.rs` (new, step code): `closed(domain, env, project,
   out)` and `gone(domain, env, project, out)`, each emitting
   `Request::Erase { key }` for each row it drops and changing memory
   in the same step; `Key::Roles(project)`, `Key::Policy(project)`, the
   services' `Key::Person`/`Key::SignIn` rows; then `Key::Answer` and
   `Key::Acted` rows naming the project, their alarms cancelled; and
   `task_erased(domain, env, task, project, out)` beside
   `inbox.rs`' `forget_event`, for every party holding a position or
   contribution of the task.
3. Tests:
   - `jig-domain-people` `src/tests/removal.rs`:
     `a_closed_project_erases_its_roles_and_policy_and_no_party_holds_a_role_in_it`,
     `a_closed_projects_services_go_with_their_sign_ins`,
     `a_gone_project_erases_each_answer_and_act_naming_it_and_no_other`,
     `an_ask_sent_again_after_its_project_is_gone_is_judged_afresh`,
     `an_erased_task_leaves_no_partys_read_position_or_inbox_entry`;
   - `jig-people-world`, in a new `tests/suite/removal.rs` (its `mod` line in
     `tests/suite/main.rs`; the lead's narrowing: `acts.rs` there is
     `pd-cancel`'s): `a_project_closed_then_gone_survives_a_restart_as_gone`
     (the erases committed, a restart restores no row of it).
4. Checks: `cargo fmt`; clippy on `jig-domain-people`; `heavy cargo
   nextest run -p jig-domain-people --lib`, `-p jig-people-world`;
   `heavy cargo check --tests -p jig-domain-core -p jig-core-world`.
   Commit, push.

## Report

fab's sections; `## Interfaces`: the three events, `names_project`'s
paths and signatures; what "names the project" covers, variant by
variant; how services were told.
