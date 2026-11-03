+++
id = "kept-started-from"
part = "kept-jig"
kind = "feature"
needs = ["kept-start"]
files = ["jig/crates/jig-domain-tasks/src/**", "jig/crates/jig-domain-core/src/**", "jig/crates/jig-protocol-store/schema/tasks-v9.schema", "jig/crates/jig-protocol-store/src/**", "jig/crates/jig-protocol-store/golden/**", "jig/tests/tasks/**", "jig/tests/core/**", "jig/docs/design/domain/connectors.status.md", "jig/docs/design/domain/tasks.status.md", "jig/docs/design/protocol/store.status.md"]
effort = "high"
minutes = 100
+++

# kept-started-from: a dependency started from is never removed

## Goal

A procedure's step may mark pending dependencies of its task as started from; jig keeps the
marks with the task's dependencies, stores them, and refuses an amendment that leaves a
marked dependency out, naming it (contract C2). temper's change procedure marks the
dependencies whose pushed heads its producer started from (task `early-route`).

Acceptance:

- `jig_domain_core::ProcedureStep.started_from` as C2 has it, carried into the tasks child's
  `ProcedureDecision` (a field of the same name) and checked and applied with the step's
  decision: a mark naming a task that is not among the step's task's dependencies refuses
  the step whole (`Refusal::Dependencies`, `blocked_by` naming it), nothing of it committed;
  a mark already kept changes nothing; a mark of a done dependency is kept.
- The task's record keeps `started_from: Box<[u64]>` beside `dependencies` (first marked
  first, each once); the tasks family stores it (`TaskRecord.started_from: list 64
  Number`), translated both ways, goldens as the store's tests require, and a restored task
  keeps its marks.
- An amendment (a party's, an agent tool's, a procedure step's) whose dependency list leaves
  out a marked dependency is refused as `Refusal::Dependencies`, its `blocked_by` naming
  exactly the marked ones it leaves out, pending or done; an amendment keeping every marked
  one and adding or removing others is admitted as today.
- The stories below pass; every test of `jig-domain-tasks`, `jig-domain-core`,
  `jig-protocol-store`, `jig/tests/tasks` and `jig/tests/core` passes.

## Inputs

- `jig/docs/design/domain/connectors.md`, 6.1 ("A dependency started from") and 6.3 (a
  step's marks; a step refused whole).
- `jig/docs/design/domain/tasks.md`, section 7 ("Adding and removing dependencies", its
  *Started from* and *Out of reach* bullets).
- `jig/docs/design/protocol/store.md`, 5.1 ("Added dependencies keep when and by whom") and
  section 5 (a format change before the first release).
- Status entries this task removes: `jig/docs/design/domain/connectors.status.md`, "6.1,
  6.3: a dependency marked started from by a step, kept with its task's dependencies.";
  `jig/docs/design/domain/tasks.status.md`, 7's "; a dependency started from (connectors.md,
  6.1), refused removal" (leaving the full stop after "outlive their asker");
  `jig/docs/design/protocol/store.status.md`, "5.1: the tasks family's dependencies marked
  started from (...): `tasks-v9`'s `TaskRecord` keeps none."
- `temper/docs/plan/reference-m6/notes/common.md`; `notes/contracts.md`, C2.

## Owns and may change

Owns the files in the header (task `kept-start`'s hunks in `jig-domain-tasks` and
`jig-domain-core` are on your base: build beside them).

- Where it goes: the step's check is `jig-domain-tasks/src/procedure.rs`'s `check`; its
  apply is the staged decision it builds; an amendment's dependencies are checked in
  `control.rs`'s `check_amend` (reach, unknown, ended) and applied in `apply_amend`. The
  core turns a `ProcedureStep` into the tasks child's `ProcedureDecision` where it routes a
  connector's step (`jig-domain-core/src/routing/**`).
- Reader and neighbour files: every `ProcedureStep { .. }` and `ProcedureDecision { .. }`
  literal, in jig and in temper (`temper/crates/temper-engine-domain/src/**`,
  `temper/crates/temper-engine-domain-forge/src/**`, their tests and worlds), its new field
  `started_from: Box::new([])` only (the other tree's lines, the field alone); every task
  record literal in jig's tests, `started_from: Box::new([])`.
- `jig/docs/design/protocol/store.md` is read by tests (`.config/gate/read-by-tests.txt`):
  run the tests of `jig-test-store`, `jig-test-store-files` and `jig-test-domain` that read
  it (`rg 'store.md' jig/testing`), and say which ran.
- Options: if the store's version policy (store.md, section 5) asks a new family version
  rather than an in-place change, follow it and name the version. If the problem type has
  no room for several `blocked_by` tasks, name the first left out and say so.

## Steps

1. The field on the core's step and the tasks child's decision; the check and the apply.
2. The record field, its store schema and translation, restore.
3. The amendment's refusal.
4. Stories in `jig/tests/tasks` (a suite file `tests/suite/started_from.rs`, with
   `kept-start`'s kept-start procedure):
   `a_step_marks_a_pending_dependency_started_from_and_an_amendment_dropping_it_is_refused`,
   `an_amendment_keeping_the_marked_dependency_and_adding_another_is_admitted`,
   `a_mark_naming_no_dependency_refuses_the_step_whole`,
   `a_marked_dependency_done_still_refuses_its_removal`; one in `jig/tests/core`:
   `a_party_amendment_dropping_a_started_from_dependency_is_refused_naming_it`; and a store
   round trip of a record with marks.
5. Remove the status texts above. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-domain-tasks -p jig-domain-core -p jig-protocol-store
   --all-targets -- -D warnings`; the focused tests of those packages and the worlds
   above; `heavy cargo check --tests -p temper-engine-domain -p temper-engine-domain-forge`
   and each temper world package whose literal you edited. Commit, push.

## Report

As the implementer's text says; name the store version written.
