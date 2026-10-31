+++
id = "jp-finish"
part = "project-delete"
kind = "feature"
needs = ["jp-close", "jp-planner", "jp-proposals", "jp-tasks", "jp-people", "jp-children", "jp-referee"]
files = [
  "jig/crates/jig-domain-core/src/routing/removal/rows.rs",
  "jig/tests/core/tests/suite/removal_rows.rs",
]
effort = "high"
minutes = 80
+++

## Goal

Build the deletion's last two stages (playground-reset.md, 5, "Then its
own rows"; `notes/contracts.md`, 1). `Notes`: the project's decided
person proposals and its notes erased from the store, page by page.
`Rows`, one decision: its rows in the tasks, people and accounts
children dropped (`jp-tasks`, `jp-people`, `jp-children`), its policy row
erased, the core's and the authority's memory of it let go, every
connector told `connector::Ask::ProjectRemoved { project }`, and the
`Removal` row erased. Its name is free again; its number is never
reused (the `projects` high-water mark stays).

Acceptance (`jig-core-world`, new `tests/suite/removal_rows.rs`; a
project with no task, so `jp-close`'s path reaches `Notes` at once):

- `a_projects_notes_are_erased`: notes in its project scope, a goal
  scope and a resources scope, and a note of another project: after
  `Notes`, no `Line` or `Entry` of the project's scopes is in the store,
  the other's is; a notes index of the project's scope, asked before,
  is not served from the notes child's cache afterwards.
- `its_decided_proposals_are_erased`: a decided person proposal of the
  project and one of another: the first's row and its `Update` index row
  gone, the second's there.
- `its_own_rows_go_in_one_decision`: the `Rows` decision erases its
  `Roles` and `Policy` rows and its `Removal` row, closes its open
  ledgers to `ClosedLedger` rows, drops `Use::Project(project)` from an
  account's uses, and sends `ProjectRemoved { project }` once to each
  connector (the world's test connectors record it), in one commit.
- `nothing_names_the_project_after`: the naming walk (`jp-referee`'s
  `observations::naming`) counts zero for the project; its `ClosedLedger`
  rows and retained answers remain.
- `its_name_is_free_and_its_number_is_not`: a `MakeProject` with the same
  name succeeds, numbered above the removed project.
- `its_retained_answers_answer_unknown`: an owner's retained `Rename` key
  sent again answers *unknown*; the `RemoveProject` key, `ProjectRemoved`.
- `the_project_is_gone_for_every_request`: after the end, the requests
  `jp-close`'s fence covers still answer *unknown*.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 1
  (connectors told; spend outlives the project).
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `jig/docs/design/domain/notes.md` (scopes; rg `Scope`).
- `temper/docs/plan/playground-reset-r2/tasks/jp-close.md` (layout,
  `Work`, `enter`), `jp-proposals.md`, `jp-tasks.md`, `jp-people.md`,
  `jp-children.md` (the events this sends).

## Owns and may change

Owns `routing/removal/rows.rs` (replacing `jp-close`'s stubs) and the
new test file. May change, as `notes/scope.md` says: `routing/removal.rs`
where `advance` and `loaded` dispatch to `rows` (an arm each; `jp-erase`
edits `erase.rs` at the same time, not these); `projects.rs` (a
`pub(crate) fn drop_project(&mut self, project)` beside `make_project`,
undoing what it set: `projects`, `names`, `permission_roles`,
`template_overrides`, the authority's policy by `authority::Event::Dropped`);
every other core map keyed by a project (`resource_roles`' keys of the
project, per-project marks in `documents/` or `routing/`), through a
forget in its own module; `routing/children.rs`, `routing/notes.rs`,
`routing/people.rs`, `routing/tasks.rs` where the new child events are
stepped and their outputs tagged; the world's test helpers and
`tests/suite/main.rs`.

Not yours: erasing tasks (`jp-erase`); resuming at a start (`jp-delete`).

## Steps

1. `rows::start_notes`: step `notes::Event::DropProject { project }`;
   `Work::Proposals { after: None }`; load `Scope::DecidedProposals` with
   the page bounds `jp-erase` uses (`after` as `Position::Owner` of the
   last proposal's key).
2. `rows::loaded`, for `Proposals`: erase `row_keys` (requester `None`) of each row of the
   project (`PersonProposal.project`), deduplicated; next page from
   `next`, in memory; at the end, `Work::Notes` and load `Scope::Notes {
   project }` (the read that serves a project's notes today).
   For `Notes`: erase each `Line`'s key and its `Entry { name }` key
   (once per name); load the same scope again from its start (what was
   erased is gone) until a page is empty; then `enter(.., Rows, ..)`.
   An entry of another scope that references an erased entry is left as
   it is; say in the report what a read of it shows.
3. `rows::finish` (the `Rows` stage, one decision): step
   `tasks::Event::DropProject` (a `DropRefused` is a fault of the order:
   panic with the project), `people::Event::DropProject`,
   `accounts::Event::DropProject`; `Write::Erase` of
   `Key::People(people::Key::Policy(project))`; `core.drop_project`; for
   each of `core.connectors`, `Request::Ask { connector, ask:
   Ask::Effect(connector::Ask::ProjectRemoved { project }) }` (or the
   form `jig-acts` gave it: rg `ProjectRemoved` in
   `jig-domain-core/src/`); `Write::Erase(Key::Core(CoreKey::Removal(project)))`,
   `removals` and `removal_work` emptied of it; `Decided`.
4. Tests: the acceptance's. Then, one at a time:
   - `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/removal_rows/)'`;
   - `heavy cargo nextest run -p jig-domain-core -p jig-core-world -p jig-people-world`;
   - `heavy cargo nextest run -p jig-conformance-world -p jig-client-world -p jig-ops-world -p jig-test-connector-world`;
   - `heavy cargo check --tests -p temper-engine-domain -p temper-service`
     and `heavy cargo nextest run -p temper-engine-domain` (its root now
     receives `ProjectRemoved` asks, routed to nothing until `rm-`);
   - `cargo fmt`; clippy over the packages you edited; old forms from `jig/`.
5. Commit and push.

## Report

The standard report, plus: every project-keyed core map you found and
how it is let go; what a reference to an erased note reads as.
