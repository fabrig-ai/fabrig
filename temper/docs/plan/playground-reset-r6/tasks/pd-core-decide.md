+++
id = "pd-core-decide"
part = "pd"
kind = "feature"
needs = ["pd-vocabulary"]
files = [
  "jig/crates/jig-domain-core/**",
  "jig/tests/core/tests/suite/**",
  "jig/tests/core/src/fixtures.rs",
  "jig/crates/jig-domain-people/src/inbox.rs",
  "jig/crates/jig-domain-people/src/tests.rs",
]
effort = "high"
minutes = 120
+++

## Goal

The core decides a project's deletion and J8's `Cancel` act (contracts J1, J2, J3, J8): the
deployment's `RemoveProject` act and the owner's `RemoveProject` request write the deletion
row at `Ending`, close the project at once to every request, list and document, cancel its
live tasks, and answer `ProjectRemoved`; once its last task has ended the row moves to
`Erasing` (whose work is `pd-core-erase`'s). A start resumes a deletion at `Ending`.

Acceptance:
- **Subject.** A project is *live* when `core.projects` holds it and the people child holds no
  deletion row for it (`Core::deleting(project) -> bool`, new, `core.people.deletion(project)
  .is_some()`). `RemoveProject` of a project that is not live is refused `Unknown`
  (saved as every act's refusal is). The request is the project's `Owner`'s alone
  (`acts/standing.rs`, as `Rename`); the act needs its grant (people, already).
- **One decision** (act or request): `people::Event::Deletion { project, stage:
  Some(Stage::Ending) }`; one compound of the deployment (`tasks::Event::Compound`, `by:
  tasks::Party::Deployment { project }`) cancelling every live root task of the project
  (`row.root == row.number`) that is not already closing, reason `b"Project deleted"`, a
  conversation with the grace of the first offered conversation template (as temper's
  `route/demo.rs` `send_end_act` picks it), any other root without grace (a root's cancel
  cancels its live subtree, `tasks/control.rs` `check_cancel`); every live watch whose subject
  is the project or a task of it ended (`views::End` as for a subject that is gone), every list
  that showed it marked changed; the answer `Outcome::ProjectRemoved { project }`. With no live
  task, the row is written at `Ending` and moved to `Erasing` in the same decision.
- **Closed at once.** While a project is deleting, every request naming it is refused
  `Refusal::Unknown`, at these choke points: `acts::request` (every party `Route`, its project
  or its task's project), `routing/deployment_acts.rs` `route` (`Close`, `Adopt`, `Cancel`,
  `RemoveProject`), `routing/application_routes.rs` `admitted` (a project scope),
  `routing/views.rs` watch admission (`WatchRefusal::Unknown`; task subjects by their task's
  project), reads (`Read::Project` and task reads: `Unknown`), `Event::StartRecurring` and
  `Event::Period` for it (refused or ignored as for an unknown project). It leaves every list
  and document: the frame's projects (`documents/small.rs`), the deployment's project list
  (`documents/deployment.rs`), inbox counts (`documents/inbox_deployment.rs`), and every
  other reader of `core.projects`, `core.names` or `people.project_roles` that lists projects
  (the implementer lists them). Its name stays taken until the row is erased (`pd-core-release`).
- **The offer.** The project document offers `acts::Act::RemoveProject` to its owners
  (`documents/offers.rs`' project list), judged as `Rename`.
- **`Cancel` (J8).** `people::Act::Cancel { project, task, reason }`: a live task of the
  project (any requester), not closing: one deployment compound cancelling its live delegates
  without grace, then the task without grace, reason as given; answered as a compound
  (`PersonTaskRoute::Compound`, as `close()` is). A task not live is answered as `Close` answers
  one (`result_reads::closed_subject`: `Ended` for an ended task); a task of another project
  `Unknown`.
- **Ending to Erasing.** When the last live task of a deleting project at `Ending` ends
  (`tasks::Request::Ended`, `routing/tasks.rs`), the same decision writes
  `Deletion { project, Some(Erasing) }`. Nothing else is done at `Erasing` here.
- **At each start**, in the core's decision on `tasks::Event::Restored` (AdoptRuns, every
  root's), each deletion row at `Ending` is resumed: any live root task of it not closing is
  cancelled as above (none is expected), and a project with no live task moves to `Erasing`.
  Party input is gated until Open by the roots (`restart_ready`), so no start serves the
  project.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J1, J2, J3, J4 (Ending), J8.
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `jig/docs/design/domain/people.md`, 5.3 (the deployment's acts, `Close`'s row of its table).
- `jig/docs/design/domain/tasks.md`, section 7 (the compound request).
- The code: `routing/deployment_acts.rs` (`route`, `close`), `acts/standing.rs`,
  `routing/people.rs` (~230 `tag_people` Route arm), `routing/views.rs` (~186, ~281),
  `routing/tasks.rs` (~1155 `Ended`, ~1620 `route_tasks`), `documents/offers.rs` (~129).

## Owns and may change

Owned: `jig/crates/jig-domain-core/**` and its world's tests `jig/tests/core/tests/suite/**`
and `jig/tests/core/src/fixtures.rs` (a granted connector, the reset-free helpers).
Nothing in other crates: the shapes are `pd-acts`' and `pd-vocabulary`'s.

Options, pre-approved:
- If the compound's reply needs a route entry to be consumed, add a variant of the core's own
  route table (as `PersonTaskRoute::Compound`) that discards `Done` and, for
  `CompoundRefused`, leaves the row at `Ending` for the next start (a refusal is not expected:
  the tasks are checked live in the same decision).
- If ending live watches in the deciding decision needs more room than the decision has,
  mark their subjects changed instead, so that their next delivery is the subject's end; say
  which you did.
- If a reader of the project list cannot tell deleting from live cheaply, filter with
  `Core::deleting` at the reader.

## Steps

1. `projects.rs` (or a new `deletion.rs` module, doc citing `draft playground-reset.md, 5`
   and contracts J3): `Core::deleting`, `pub(crate) fn remove_project(core, env, work, out,
   request, project) -> Result<(), people::Refusal>` used by the act and the request,
   `ending_done(core, project, out)` (the move to `Erasing`), `resume_ending(core, ...)`.
2. Wire the act (`deployment_acts::route`), the request (`routing/people.rs` / `acts/requests.rs`),
   `Cancel`, the fence and lists, the offer, the end hook and the restart hook.
3. Tests in `jig/tests/core/tests/suite/` (new `deletion.rs`, registered in `main.rs`):
   - `a_granted_connector_deletes_a_project_and_its_tasks_end` (two live chats with a delegate;
     the act answers `ProjectRemoved`; the row reads `Ending`, then `Erasing` once both ended;
     the conversation's grace is honoured).
   - `an_owner_deletes_a_project_from_its_document_offer` (the offer is there for the owner, not
     for a member; the request answers `ProjectRemoved`).
   - `a_deleting_project_refuses_every_request_naming_it` (a new chat, a message to a live chat,
     an adoption act, a policy change, a watch of its task, a read of its project document, a
     second `RemoveProject`: each `Unknown`; another project still serves).
   - `a_deleting_project_leaves_the_frame_and_the_deployment_list`.
   - `a_project_without_tasks_goes_straight_to_erasing`.
   - `an_unknown_or_deleting_project_is_refused_unknown_and_the_answer_is_kept` (resend of the
     same key answers the saved `ProjectRemoved`).
   - `a_start_during_ending_resumes_it` (crash after the deciding commit; the start resumes,
     the tasks end, the row reaches `Erasing`).
   - `the_cancel_act_cancels_any_live_task_of_its_project` (a deployment-requested task with a
     delegate; `Ended` for an ended task; `Unknown` for another project's).
   Commands: `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/deletion::/)'`,
   then `heavy cargo nextest run -p jig-domain-core --lib` and
   `heavy cargo nextest run -p jig-core-world --test suite` (the fence is a change of meaning
   for the core's readers), and `heavy cargo nextest run -p jig-conformance-world --test suite`.
4. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-core-world --all-targets`;
   old forms; commit and push.

## Report

The choke points and lists you changed, each with its file; anything that names a project
and was left (with why).

## Answers

2026-11-01. May this task change `jig/crates/jig-domain-people/src/inbox.rs` so a deleting
project leaves the inbox counts and cached selection? Yes, option (A): J3's "leaves every inbox
count at once" cannot hold from the core, since `counts` caps the private census before the core
reads it. `pd-people-purge`, which owns the crate, is merged into the trunk, so the widening
meets no running owner. Scope, and no more: in `inbox.rs`, the visibility and count predicates
(`counts`' three scans, `needs`, the recent/unread selection the core's documents read) treat a
project whose `self.deletion(project).is_some()` as one the person has no role in; roles,
`chat_updates`, `result_census` and every durable row stay until `DropProject`/`EraseTask`
(`pd-people-purge`, J4). No new public or stored vocabulary. A unit test in the crate's
`tests.rs` (counts and needs drop a deleting project's entries; a live project's stay exact,
`more_unread` with it), beside the core-world `deletion.rs` cases; run
`heavy cargo nextest run -p jig-domain-people --lib` too. The files list above is widened by
these two files.
