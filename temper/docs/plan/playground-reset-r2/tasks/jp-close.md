+++
id = "jp-close"
part = "project-delete"
kind = "feature"
needs = ["jig-ask", "jp-row", "jp-tasks", "jp-scopes"]
files = [
  "jig/crates/jig-domain-core/src/routing/removal.rs",
  "jig/crates/jig-domain-core/src/routing/removal/**",
  "jig/crates/jig-domain-core/src/routing/deployment_acts.rs",
  "jig/crates/jig-domain-core/src/acts/**",
  "jig/crates/jig-domain-core/src/acts.rs",
  "jig/crates/jig-domain-core/src/documents/**",
  "jig/crates/jig-domain-core/src/documents.rs",
  "jig/tests/core/tests/suite/removal_close.rs",
]
effort = "high"
minutes = 90
+++

## Goal

Build the deletion's first decision (playground-reset.md, 5, "Closed at
once"; `notes/contracts.md`, 1): a granted `Act::RemoveProject` or an
owner's `Ask::RemoveProject` on a live project, in one decision, saves
the project's `Removal` row at `RemovalStage::Ending`, closes the project
to new work (every request naming it refused *unknown*, as on a project
that is gone) and out of every list, cancels each of its live tasks as a
conversation's close cancels it, and answers `ProjectRemoved` once the
decision commits. When its last live task ends, the row moves to
`Erasing { after: None }`. The later stages are stubs here, with the
module layout `jp-erase` and `jp-finish` fill in at once.

Acceptance (`jig-core-world`, new `tests/suite/removal_close.rs`):

- `the_deployments_remove_project_answers_project_removed`: granted, on a
  live project: answered `ProjectRemoved { project }` after its commit;
  the commit saves `CoreRecord::Removal` at `Ending`.
- `an_owners_remove_project_answers_project_removed`; a non-owner's is
  refused as lacking the role (as `jig-ask` left it).
- `a_project_gone_or_being_removed_is_unknown`: a second `RemoveProject`
  (fresh key) on the same project, and one on a number never made: both
  *unknown*.
- `a_removing_project_takes_no_new_work`: after the answer, a person's
  new conversation, a message to its live conversation, an adoption
  (deployment `Adopt`), a settings or policy change, a rename, a role
  change, a deployment `Close` or `Cancel` on its task: each *unknown*.
  A request on another project is served as before.
- `a_removing_project_leaves_every_list`: the deployment's and the frame's
  project lists, and every other document that lists projects, no longer
  show it.
- `its_live_tasks_are_cancelled`: a live conversation with a delegate
  and a deployment task end as a cancel ends them (delegates with no
  grace, each root with the grace a `Close` with `grace: None` gives); a
  task in grace finishes within it; the compound's own answer goes to no
  one.
- `the_last_live_task_ending_moves_the_row_to_erasing`: once none is
  live, the row is saved at `Erasing { after: None }` in the decision
  that ended the last one; with no live task at the request, in the
  request's own decision.
- `a_restart_keeps_the_project_closed`: crashed after the answer's
  commit, the next start restores the row and the project stays closed,
  unlisted, and its tasks' cancels carry on.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 1.
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `jig/docs/design/domain/people.md`, 5.3 (deployment acts) and 5.1-5.2
  (owners' project acts).
- `jig/docs/design/domain/tasks.md`, the compound request (cancel and
  grace).
- `jig/crates/jig-domain-core/src/routing/deployment_acts.rs` (`close`,
  and `jig-acts`' `cancel`, which this reuses).

## Owns and may change

Owns the files in the header. May change, as `notes/scope.md` says:

- `routing/mod.rs`: the `mod removal;` line; `Event::ReadLoaded`'s
  dispatch, sending loads whose token is a removal's to
  `removal::loaded` before `reads::loaded`.
- `routing/tasks.rs`'s `tag_tasks`: in the `tasks::Request::Ended` arm, a
  call to `removal::task_ended(core, env, project, out)` when the task's
  project is being removed; `routing/people.rs` and `routing/projects.rs`
  where the act or ask is routed.
- Every admission that checks a project exists (rg `has_project`,
  `projects.as_slice().contains`, `names.get`, `policy(project)` in
  `jig-domain-core/src/`): each also refuses a removing project
  `Refusal::Unknown`; in files outside the header, the check and what
  it forces.
- `lib.rs` and `Core::new`: the `removal_work` field below and its
  empty map; any other state the fence needs, named after what it
  serves, said in the report.
- The world's `tests/suite/main.rs` (`mod` line).

Not yours: the row's types and codec (`jp-row`), erasing tasks
(`jp-erase`), notes and own rows (`jp-finish`), resuming at a start
(`jp-delete`).

## Steps

1. `routing/removal.rs` (module doc citing
   `temper/docs/design/draft/playground-reset.md, 5`), with submodules
   `routing/removal/erase.rs` and `routing/removal/rows.rs` holding stubs
   that `jp-erase` and `jp-finish` replace:

   ```rust
   /// The deletion's first decision: row, fence, cancels; answer held.
   pub(crate) fn close(core: &mut Core, env: &Env<Limits>, project: u32, out: &mut Queue<Request>)
       -> Result<(), people::Refusal>;
   /// Move a removal to `stage`, saving its row, and start the stage.
   pub(crate) fn enter(core: &mut Core, env: &Env<Limits>, project: u32, stage: RemovalStage, out: &mut Queue<Request>);
   /// Continue a removal at its stage (used after each answer, and by
   /// `jp-delete` at a start).
   pub(crate) fn advance(core: &mut Core, env: &Env<Limits>, project: u32, out: &mut Queue<Request>);
   /// One of a removing project's tasks ended.
   pub(crate) fn task_ended(core: &mut Core, env: &Env<Limits>, project: u32, out: &mut Queue<Request>);
   /// A removal's load answered.
   pub(crate) fn loaded(core: &mut Core, env: &Env<Limits>, load: Token, result: reads::Loaded) -> Requests;
   /// The load token of a removal, and whether a token is one.
   pub(crate) fn load_token(project: u32) -> Token;
   pub(crate) fn removal_of(load: Token) -> Option<u32>;

   // erase.rs (stub here): start the Erasing stage after `after`; take a loaded page.
   pub(super) fn start(core, env, project, after: Option<u64>, out);
   pub(super) fn loaded(core, env, project, result: reads::Loaded, out);
   // rows.rs (stub here): start the Notes stage; take a loaded page; the Rows decision.
   pub(super) fn start_notes(core, env, project, out);
   pub(super) fn loaded(core, env, project, result: reads::Loaded, out);
   pub(super) fn finish(core, env, project, out);
   ```

   Pick a token space no other core load uses (as `result_reads`' tokens
   are kept apart), and say which.

   The removals' transient state, shared by the two submodules, is
   yours to add (`lib.rs`'s `Core`, one field, and its empty map in
   `Core::new`):

   ```rust
   /// What a removal is waiting on; none is idle (never stored: a start
   /// rebuilds it from the row's stage).
   pub(crate) removal_work: Map<u32, removal::Work>,

   pub(crate) enum Work {
       /// A page of `Scope::EndedTasks` after this task (erase.rs).
       EndedPage { after: Option<u64> },
       /// Pages of `Scope::TaskRows` for this ended task of the project (erase.rs).
       TaskRows { record: Box<tasks::TaskRecord> },
       /// A page of `Scope::DecidedProposals` after this proposal (rows.rs).
       Proposals { after: Option<u64> },
       /// A page of `Scope::Notes { project }` (rows.rs).
       Notes,
   }
   ```

   The stubs keep a removal moving so each later task can be tested
   alone: `erase::start` enters `Notes` at once (no task erased);
   `rows::start_notes` enters `Rows` at once; `rows::finish` and both
   `loaded` stubs do nothing (the row stays at `Rows`).
2. `close`: *unknown* unless `core.projects` holds the project and
   `!core.removing(project)`. Otherwise: insert `ProjectRemoval {
   project, stage: Ending }` and `Write::Save(Record::Core(CoreRecord::Removal(..)))`;
   cancel every live root (`tasks.live_roots`, `jp-tasks`) by one
   compound as `jig-acts`' `cancel` builds it, as many compounds as the
   operations bound needs, by `tasks::Party::Deployment { project }`, with
   a route that swallows the compound's own answer; if no task is live,
   `enter(.., Erasing { after: None }, ..)` at once. The act's answer is
   `Outcome::ProjectRemoved { project }` through the deployment acts'
   `Decided` path; the ask's through the party reply path, as `Rename`'s.
   Replace `jig-acts`' and `jig-ask`'s *not offered* refusals with it.
3. `task_ended`: when `!core.tasks.live_in(project)`, `enter(..,
   Erasing { after: None }, ..)`; cancel any live root not yet closing
   (a task made in the same decision as the close, if any).
4. The fence (Owns, third bullet) and the lists: every document that
   lists projects skips a removing one (rg the documents that iterate
   `core.projects` or `names`).
5. Tests: the acceptance's. Then, one at a time:
   - `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/removal_close/)'`
     (binary name from `jig/tests/core/Cargo.toml`);
   - `heavy cargo nextest run -p jig-domain-core -p jig-core-world -p jig-people-world`;
   - `heavy cargo nextest run -p jig-conformance-world -p jig-client-world -p jig-ops-world`;
   - `heavy cargo check --tests -p temper-engine-domain`, and
     `heavy cargo nextest run -p temper-engine-domain` (temper's root
     routes the core's answers; its meaning changes where a project is
     refused *unknown*);
   - `cargo fmt`; clippy over the packages you edited; old forms from `jig/`.
6. Commit and push.

## Report

The standard report, plus: every admission you fenced (file and
function); the token space; how many compounds a project with many
live roots needs; the grace a root got.
