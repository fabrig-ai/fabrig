+++
id = "jp-conformance"
part = "project-delete"
kind = "feature"
needs = ["jp-delete", "jp-referee"]
files = [
  "jig/testing/jig-conformance/src/scenarios.rs",
  "jig/testing/jig-conformance/src/scenarios/**",
  "jig/tests/conformance/src/**",
  "jig/tests/conformance/tests/suite/removal.rs",
]
effort = "high"
minutes = 90
+++

## Goal

Test a project's deletion in jig's conformance world (playground-reset.md,
5 "Tested in jig's conformance world" and 8): one case per family of
jig's store, each a shared scenario played on the testing application,
deleting a project whose rows fill that family, swept by a crash after
every commit, and judged by the referee's erasure promise (`jp-referee`):
at the end no row names the project, its tasks left the durable view
only after the answer, and the rest of the world kept every promise it
keeps today.

Acceptance (`jig-conformance-world`, new `tests/suite/removal.rs`):

- `removal_cases_run_on_the_testing_application`: each new scenario runs
  uncut and its ending check holds.
- One sweep per scenario, `<scenario>_ends_whole_after_every_cut`, with
  `replay::every_cut`: the referee finds no violation at any cut.
- The scenarios, each named for the family it fills (all in
  `Scenario`, none in `ALL`, so other applications' shared runs are
  unchanged):
  - `RemoveProjectTasks`: a person's conversation run to a result and
    ended, its delegate, a deployment task, a live conversation in grace
    when the deletion comes; the tasks family's rows of each.
  - `RemoveProjectCore`: a run with a claim, a call, a terminal and a
    transcript, an escalation decided; the core family's rows of the
    task.
  - `RemoveProjectPeople`: two parties reading the conversation, a role
    held by a second party, the owner's retained `Rename`; the people
    family's rows (the retained answer stays and answers *unknown*).
  - `RemoveProjectNotes`: notes in the project, a goal and a resources
    scope.
  - `RemoveProjectAccounts`: an account whose uses name the project and
    another.
  - `RemoveProjectByOwner`: the owner's ask instead of the deployment's
    act, on the tasks case's world.
- `a_corrupted_deletion_is_caught`: the world's negative faults
  (`src/negative.rs`) gain `Fault::LeftRow` (the root drops one erase of
  the deletion's commits): the referee answers `Promise::Erasure`.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/design/draft/playground-reset.md`, sections 5 and 8.
- `jig/docs/design/domain/testing.md` (scenarios, cuts, the referee).
- `jig/testing/jig-conformance/src/scenarios.rs` (`Scenario`, `Action`,
  `program`, `ScenarioApplication`) and `jig/tests/conformance/src/scenarios.rs`
  (the testing application's bindings), `replay.rs`.
- `temper/docs/plan/playground-reset-r2/tasks/jp-referee.md`,
  `jp-delete.md`.

## Owns and may change

Owns the files in the header. May change, as `notes/scope.md` says: the
shared kit's `Action` (new actions such as `RemoveProject { by_owner:
bool }` and those the cases need to fill a family) and `program`; every
exhaustive match on `Scenario` or `Action` the variants force, in jig and
temper: `temper/tests/conformance/src/{lib.rs,config.rs,scenarios.rs}`
and `temper/tests/conformance/tests/support/heap_ceilings.rs` bind none of
the new scenarios (`configuration` answers `None`, an action or check arm
is unreachable for them, a heap ceiling arm takes the default), so
temper's shared runs are unchanged; the world's `tests/suite/main.rs`
(`mod` line); `jig/tests/core/src/**` where the testing application needs
an observation it lacks (as `jp-referee` added `ProjectRemoved`).

Not yours: the core's deletion (fix a fault it shows only by a question,
with the failing case and its cut); temper's application.

## Steps

1. Shared kit: the six `Scenario` variants, out of `ALL`; their programs
   in `program(scenario)`: set up the family's rows, ask the deletion,
   drain, end; the `Action`s they need.
2. The testing application (`jig/tests/conformance/src/scenarios.rs`
   and `scenarios/`): `configuration` for each (tiny limits, the
   deployment granted `remove_project` in `engine.acts`, the owner
   seeded), its actions, and its ending check: the referee's erasure
   promise holds and the family's rows of the project are gone in the
   last durable snapshot, its ledgers closed.
3. The negative fault `LeftRow` in `src/negative.rs`, as the existing
   faults are made.
4. Tests: the acceptance's. Then, one at a time:
   - `heavy cargo nextest run -p jig-conformance-world --test suite -E 'test(/removal/)'`
     (binary name from `jig/tests/conformance/Cargo.toml`);
   - `heavy cargo nextest run -p jig-conformance -p jig-conformance-world`;
   - `heavy cargo check --tests -p temper-conformance-world` (use the
     package name of `temper/tests/conformance`), then its focused tests;
   - `cargo fmt`; clippy over the packages you edited; old forms from
     `jig/` and `temper/`.
   A sweep over its budget is red: measure it and ask, as `AGENTS.md`, 6
   says; never move it out.
5. Commit and push.

## Report

The standard report, plus: each scenario's commit count and its sweep's
wall time; any family whose rows the cases could not fill, and why.
