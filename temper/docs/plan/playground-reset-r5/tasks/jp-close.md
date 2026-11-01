+++
id = "jp-close"
part = "jig-remove-project"
kind = "feature"
needs = ["jp-acts", "pg-jig-uses", "jp-row", "jp-people"]
files = ["jig/crates/jig-domain-core/src/removal/close.rs", "jig/crates/jig-domain-core/src/policy.rs", "jig/crates/jig-domain-core/src/projects.rs", "jig/crates/jig-domain-core/src/routing/projects.rs", "jig/crates/jig-domain-core/src/routing/people.rs", "jig/crates/jig-domain-core/src/routing/application_routes.rs", "jig/crates/jig-domain-core/src/connector.rs", "jig/crates/jig-domain-core/src/documents/**", "jig/crates/jig-domain-core/src/tests/closing_projects.rs", "jig/tests/core/tests/suite/closing_projects.rs", "jig/tests/core/tests/suite/main.rs"]
effort = "high"
minutes = 100
+++

## Goal

Give a project's deletion its closing decision (draft
`temper/docs/design/draft/playground-reset.md`, section 5, "closed at
once"; part note section 2, step 1): on the deployment's granted
`Act::RemoveProject` or an owner's `Ask::RemoveProject`, in one decision
the project is gone to everyone, its deletion is kept, the application
is told, and its tasks' cancelling begins. Every later start keeps it
gone.

Acceptance:

- `removal::close::close(core, env, work, out, request, project)`
  (`jp-row`'s stub, filled; already routed from the act by `jp-steps` and
  from the ask by `jp-people`): a project not in `core.projects`, or
  `core.removing(project)`, is refused `Unknown`. Otherwise, one decision:
  `Write::Save(Record::Core(CoreRecord::Removal(Removal { project, step:
  Step::Ending })))` and the `core.removals` entry; the project out of
  `core.projects` and `core.names` (its name free, its number never given
  again: the counter is untouched); `people::Event::ProjectClosed {
  project }`; `Request::ProjectRemoved { project }` (held until the
  commit, as `ProjectMade` is); `removal::steps::begin(...)`; the answer
  `people::Outcome::ProjectRemoved { project }` (`Decided`, retained as
  every permanent answer). The authority's policy, the permission and
  template maps and the `Policy` row stay (step `Own` drops them).
- **Gone to everyone** from that decision: every core entrance that names
  the project answers as for a project number never made (assert equal
  outcomes in the tests): the people dispatcher's asks
  (`routing/people.rs`), application asks and reads of the project's
  scope (`routing/application_routes.rs`), adoptions and their answers
  and resource facts (`connector.rs`), a recurring start or a period for
  it, and every document and list (`documents/`: the deployment's
  projects, the project's own document, offers) shows nothing of it; a
  watch on the project's document ends as a watch on a vanished subject
  does. Where an entrance lives in a file another task owns
  (`routing/deployment_acts.rs`, `routing/tasks.rs`, `reads/*`), it is
  not yours: name it under `## Left`.
- **Room:** `make_project` counts kept removals against the projects'
  bound (`projects.len() + removals.len()`), refusing `Limit` before the
  authority's policy table is full, so a new project made while one is
  being deleted never trips the authority's admission.
- **At start:** `restore_policy` for a project with a kept `Removal`
  restores its policy, permission and template maps alone (for the tasks
  winding down), never into `projects` or `names`, and skips the name
  check against live projects (a live project may hold the name since).
- Tests: `jig-domain-core` unit tests, new
  `src/tests/closing_projects.rs` (room and restore cases);
  `jig-core-world`'s `suite` binary, new `closing_projects.rs` (its `mod`
  line in `main.rs`, between `client_peer_routes` and
  `completion_retry`): the granted act answered `ProjectRemoved` and the
  same key answered again from the retained answer; `ProjectRemoved`
  delivered to the root after the commit; the project in no list; a new
  project made with its name, with a new number; an owner's ask, through
  the world's client peer, and a member's refused; an ask, an adoption and
  an application read naming the removed project refused as for an
  unknown one; a restart with the deletion kept serving nothing of the
  project; an ungranted act refused *not offered*.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1,
  "a project removed is gone", "the application is told").
- `temper/docs/plan/playground-reset-r5/notes/jig-remove-project.md`,
  sections 2 (step 1, resumption), 3 and 4.
- `jig/docs/design/domain/people.md`, 5.1–5.3; `jig/docs/design/domain/core.md`,
  6.1 (projects).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s4-jig-projects.md`,
  sections 1 (project state and its readers) and 4 (the act's path).
- `make_project_request` in `routing/projects.rs`: the closing decision
  mirrors it.

## Owns and may change

- Owns the files in `files`. In `routing/people.rs` and
  `routing/application_routes.rs`, `connector.rs` and `documents/`, the
  removing guards and list filters only.
- May change: `jig/tests/core/src/**` for a world helper that grants
  `remove_project` or observes `ProjectRemoved` (add beside the world's
  others, nothing else there).
- Not yours: `removal.rs`, `removal/steps.rs`, `removal/rows.rs`,
  `routing/deployment_acts.rs`, `routing/tasks.rs`, `reads/vocabulary.rs`,
  `reads.rs`, `stored.rs` (tasks `jp-steps` and `jp-rows` run beside you).
- Options: if a document or read keeps showing the removed project through
  a file you do not own, list it under `## Left` with its path. If the
  people child's `ProjectClosed` lands after an ask in the same decision
  in a way that admits it, order the events so it does not, and say how.

## Steps

1. `close` and its unit tests; `restore_policy` and `make_project`'s room.
2. The guards and list filters, one entrance at a time, each with a case.
3. The core world's cases: `heavy cargo nextest run -p jig-core-world
   --test suite -E 'test(/closing_projects/)'`; `heavy cargo nextest run -p
   jig-domain-core --lib -E 'test(/closing_projects/)'`.
4. `cargo fmt`; `heavy cargo clippy -p jig-domain-core --all-targets --
   -D warnings`; the focused tests of `jig-domain-core`, `jig-core-world`,
   `jig-people-world`, `jig-client-world` (the documents' readers: a
   change of meaning). Commit and push.

## Report

As `notes/common.md` says; list each entrance guarded and each left
under `## Left`.
