+++
id = "jp-delete"
part = "project-delete"
kind = "feature"
needs = ["jp-erase", "jp-finish"]
files = [
  "jig/crates/jig-domain-core/src/start/script.rs",
  "jig/crates/jig-domain-core/src/start.rs",
  "jig/crates/jig-domain-core/src/routing/removal.rs",
  "jig/tests/core/tests/suite/removal.rs",
]
effort = "high"
minutes = 75
+++

## Goal

The anchor of the `project-delete` part: an admitted `RemoveProject`
(the deployment's act or an owner's ask) deletes the project whole, kept
across restarts, and its connectors are told `ProjectRemoved`
(playground-reset.md, 5, "Kept until whole": "no start serves a
half-deleted project"; `notes/contracts.md`, 1). `jp-close`, `jp-erase`
and `jp-finish` built the stages; this task resumes a kept removal at
each start, at its stage, and proves the whole story, cut by a restart
at every stage, in the core's world.

Acceptance (`jig-core-world`, new `tests/suite/removal.rs`):

- `a_project_is_deleted_whole`: a project with every kind of row the
  stages erase (a person's ended conversation with a delegate, a live
  one in grace, a deployment task, transcripts, notes in three scopes, a
  decided and a pending person proposal, a placed hold, a pool, an
  account used by it, two parties' read positions), deleted by the
  deployment's act: answered `ProjectRemoved`; once quiescent, the
  naming walk counts zero for it, its ledgers are closed history, each
  test connector got `ProjectRemoved` once, and a second project's rows
  are as they were.
- The same by an owner's ask: `an_owner_deletes_a_project_whole`.
- `a_deletion_cut_by_a_restart_ends_whole`: for each stage (`Ending` with
  a task in grace, `Erasing` after one task erased and in the middle of a
  task's pages, `Notes`, `Rows` saved but not yet run), the world crashed
  after that commit and started again: the start restores the row,
  serves no request on the project (*unknown*), lists it nowhere, resumes
  at the stage, and ends as the uncut run ends (same naming count, same
  connector asks, at least once and at most once per connector after
  the final commit).
- `two_projects_deleted_at_once_both_end`: two removals in flight, each
  ends whole.
- `a_start_opens_with_a_removal_under_way`: the start reaches `Open`
  while a removal still waits on a task's grace; other projects are
  served.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 1.
- `temper/docs/design/draft/playground-reset.md`, sections 5 and 8 (the
  engine's world: a deletion cut after each kept step, resumed to the
  same end).
- `jig/docs/design/domain/engine.md`, the start (rg `Phase` and
  `AdoptRuns`): the start's phases and what each may do.
- `temper/docs/plan/playground-reset-r2/tasks/jp-close.md`,
  `jp-erase.md`, `jp-finish.md`.

## Owns and may change

Owns the files in the header (`routing/removal.rs` after `jp-close`,
`jp-erase` and `jp-finish`, which have merged). May change, as
`notes/scope.md` says: `routing/removal/**` to fix what the whole story
shows (say each fix in the report); `restart.rs` and `restart/**` where a
restored removal is collected; the world's test helpers and
`tests/suite/main.rs`; temper's start translation
(`temper/crates/temper-engine-domain/src/restart.rs` and its route of
`Held::ReadLoad`) only if a load during the phase you resume in is not
carried there, doing for it what it does for the core's other loads.

Not yours: the conformance cases (`jp-conformance`); temper's connector
side (`rm-`).

## Steps

1. `start/script.rs`: once the start serves the core's loads and the
   children are restored (the `AdoptRuns` arm of `start_resume`, after
   `tasks::Event::Restored`, if `Held::ReadLoad` is answered there;
   otherwise the first point after `Open`), for each held removal in
   project order: rebuild its `removal_work` from the row's stage
   (`Ending`: cancel any live root not closing, then `task_ended`'s
   check; `Erasing { after }`: `erase::start(after)`; `Notes`:
   `rows::start_notes`; `Rows`: `rows::finish`), through
   `removal::advance`. The fence holds from `LoadCore` on (`jp-row`
   restores the row there), so the start never serves the project
   whatever phase this runs in; say which phase you chose and why.
2. Tests: the acceptance's, with the world's crash and restart helpers
   (`tests/suite/restart.rs` and `store.rs` are the nearest), the naming
   walk from `jig/tests/core/src/observations/naming.rs`. Then, one at a
   time:
   - `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/removal/)'`;
   - `heavy cargo nextest run -p jig-domain-core -p jig-core-world`;
   - `heavy cargo nextest run -p jig-conformance-world -p jig-people-world -p jig-client-world -p jig-ops-world -p jig-one-process-world`;
   - `heavy cargo check --tests -p temper-engine-domain -p temper-service`
     and `heavy cargo nextest run -p temper-engine-domain -p temper-service`;
   - `cargo fmt`; clippy over the packages you edited; old forms from `jig/`.
3. Commit and push.

## Report

The standard report, plus: the start phase a removal resumes in; how
many commits a deletion took in `a_project_is_deleted_whole`; any stage
fix the whole story needed.
