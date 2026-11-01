+++
id = "pd-core-release"
part = "pd"
kind = "feature"
needs = ["pd-core-erase", "pd-accounts-uses", "pd-test-connector"]
files = [
  "jig/crates/jig-domain-core/**",
  "jig/tests/core/**",
]
effort = "high"
minutes = 110
+++

## Goal

The `Releasing` stage of a project's deletion and its end (contracts J4, J5): the project's
notes, policy copy and name, roles, live ledgers, account uses and placed holds go; every
connector is told and answers; then the deletion row is erased, the name is free again and
the number is never given again. A start resumes a deletion at `Releasing`, and a start after
a deletion restores every retained answer that names the gone project.

Acceptance:
- **Notes.** For the deletion at `Releasing`, the core pages `reads::Scope::Notes { project }`
  with its own load (as `pd-core-erase`'s scan) and erases each note entry and index line
  returned (`notes::Key::Entry { name }`, `notes::Key::Line { scope, name }`), page after page,
  until none is left. Every scope of the project goes (project, goal and resources scopes);
  the notes child keeps no state that needs telling (its index cache names scopes no request
  names again).
- **One decision** then: `Write::Erase(Key::People(people::Key::Policy(project)))`; the
  project out of `core.projects`, `core.names`, `core.permission_roles`,
  `core.template_overrides`, `core.resource_roles` and the authority domain
  (`authority::Event::Dropped { project }`); `people::Event::DropProject { project }`;
  `tasks::Event::ForgetProject { project }` (live ledgers closed, settled spend kept);
  `accounts::Event::ForgetProject { project }`; and `Request::Ask { connector, ask:
  Ask::ReleaseProject { project } }` for every connector in `core.connectors`, the set of
  those not yet answered kept in memory.
- **`Event::ProjectReleased { project, connector }`** removes the connector from that set (an
  answer for a project not releasing, or a connector not asked, changes nothing); when the set
  is empty, `people::Event::Deletion { project, None }` erases the row. A core with no
  connectors erases it in the deciding decision.
- **After it,** `MakeProject` may take the project's name again, and the next project's number
  is still the next after the highest ever given (`Deployment::projects`); a resent
  `RemoveProject` under its key answers the saved `ProjectRemoved` while retained and `Unknown`
  after (people, already).
- **At each start**, on `tasks::Event::Restored` (after the `Ending` and `Erasing` resumes): a
  deletion at `Releasing` redoes the notes pages and the one decision (each erase harmless
  when absent; a project whose policy row is already gone has nothing of its own in core
  memory) and asks every connector again.
- **Retained answers restore.** A start whose store holds retained answers naming a gone
  project (people `Answer` and `Acted` rows: `ProjectRemoved`, `Renamed`, `PolicyChanged`,
  `RolesSet`, `ServiceMade`, `Adopted`, `ProjectMade`, …) restores ready: every core restore
  check that refuses an answer for a project without a policy (`restart/joins.rs`,
  `start/links.rs`, and any the implementer finds) accepts it instead.
- Settings of a project's scope: none exist (every setting is the deployment's, scope 0):
  nothing is erased for them, and the report says so (contracts J4).

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J4 (Releasing list), J5.
- `temper/docs/design/draft/playground-reset.md`, section 5 ("Then its own rows", "What
  outlives it").
- The code: `projections.rs` `settled`/`forget` (the pattern of a connector set answered one
  by one), `policy.rs` `restore_policy`, `projects.rs` `make_project`, `restart/joins.rs`,
  `start/links.rs`.

## Owns and may change

Owned: `jig/crates/jig-domain-core/**` and `jig/tests/core/**`. Shapes are earlier tasks'.

Options, pre-approved:
- If notes pages must be erased in the same decision as their load answer and that exceeds a
  decision's write bound, lower the page's rows (as `pd-core-erase` bounds its pages).
- If a restore check cannot tell a gone project from a corrupt row, accept a project number
  below or equal to `Deployment::projects` that has no policy, and keep refusing a number
  above it.

## Steps

1. The release module (beside `pd-core-erase`'s), the answer route of `Event::ProjectReleased`
   (replacing `pd-vocabulary`'s empty decision), the start hook, the restore relaxations.
2. Tests in `jig/tests/core/tests/suite/deletion.rs`:
   - `releasing_erases_the_projects_own_rows_and_frees_its_name` (notes in each scope, a pool
     and a period with spend, an account with `[Use::Project(p), Use::Party(x)]`, an adoption
     through the test connector, a project-scoped placed hold; after the deletion ends: no
     `Policy`, `Roles`, `Ledger`, note, hold or connector row names the project; the closed
     ledgers keep their spend; the account's uses are `[Party(x)]`; `MakeProject` of the same
     name succeeds with a new number).
   - `the_row_stays_until_every_connector_answers` (two connectors; one answers; the row stays;
     the second answers; the row goes).
   - `a_start_during_releasing_asks_every_connector_again`.
   - `a_start_after_a_deletion_restores_its_retained_answers` (the act's `Acted` row and an
     owner's `Renamed` answer for the gone project; restart; ready; the act resent answers
     `ProjectRemoved`).
   - `a_crash_after_each_commit_of_a_deletion_ends_the_same` (cut after every commit of a whole
     deletion in turn, from the deciding commit to the row's erase; each run resumes to the same
     rows as the uncut run).
   Commands: `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/deletion::/)'`,
   then `heavy cargo nextest run -p jig-domain-core --lib`,
   `heavy cargo nextest run -p jig-core-world --test suite`,
   `heavy cargo nextest run -p jig-conformance-world --test suite`,
   `heavy cargo nextest run -p temper-engine-domain --lib` and
   `heavy cargo nextest run -p temper-engine-domain-world --test suite` (the restore
   relaxations are a change of meaning for temper's root).
3. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-core-world --all-targets`;
   old forms; commit and push.

## Report

The restore checks you relaxed; that no setting of a project's scope exists; every row that
still names a deleted project after the deletion ends, if any, with its family.
