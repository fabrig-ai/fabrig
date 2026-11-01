+++
id = "jp-conform"
part = "jig-remove-project"
kind = "feature"
needs = ["jp-close", "jp-steps", "jp-rows"]
files = ["jig/testing/jig-conformance/src/**", "jig/tests/conformance/src/**", "jig/tests/conformance/tests/suite/removal.rs", "jig/tests/conformance/tests/suite/referee.rs", "jig/tests/conformance/tests/suite/main.rs"]
effort = "high"
minutes = 110
+++

## Goal

Test a project's deletion in jig's conformance world (draft
`temper/docs/design/draft/playground-reset.md`, section 5, "tested in
jig's conformance world", and section 8): a case per family of jig's
store, each giving the project rows in that family and deleting it, the
referee checking at the end that no durable row names the project; and
the deletion cut by a crash after each step's commit, resumed to the
same end.

Acceptance:

- **The referee's check** (`jig/testing/jig-conformance/src/referee.rs`,
  or a new `referee/removed.rs` beside it): one exhaustive
  `names_project(record: &jig_domain_core::Record, project: u32) -> bool`
  over every record of every jig family (core: settings, projections,
  run proofs, calls, turns, terminals, decisions, `Removal`, `Spent`;
  tasks; people; notes; accounts), true where any field names the project
  or a task of it (the task numbers the referee saw made in the project).
  At the end check (`check_at` at the program's end), for each project
  whose `ProjectRemoved` answer the referee observed: a durable row that
  names it is a violation, unless it is the project's own `Removal` row
  while the program has not let the deletion finish (a cut). The
  `Spent` rows are checked to have grown by the project's observed spend.
  A violation names the key.
- **A case per family**, in a new scenario module
  `jig/tests/conformance/src/scenarios/removal.rs` (registered in
  `scenarios.rs` as the others are) and tests in a new
  `tests/suite/removal.rs`: for each family, a program that makes a
  project (the deployment's `MakeProject`), gives it rows in that family
  (core: a run with turns and calls, a projection; tasks: a live
  conversation with a delegate, a change task, an ended task, a period and
  a pool with spend, a pending person proposal; people: roles, a retained
  answer naming it, read positions; notes: an entry in its project scope
  and one in a goal's; accounts: an account whose uses name it and
  another project), grants `remove_project`, deletes it by the act, runs
  until the deletion is whole, and ends with the referee's check passing;
  another project kept untouched throughout. One more case deletes by the
  owner's `Ask::RemoveProject` through the world's client path.
- **Cuts:** for the fullest case, the deletion cut by a crash right after
  the commit of each step (closing, each cancel batch, `Erasing`'s first
  page, `Spend`, `Notes`, `Own`), restarted, and run to the same durable
  end (the same rows, compared as the world's snapshots are).
- **Negative:** in `tests/suite/referee.rs`, a row left naming a removed
  project is a violation (as
  `a_read_position_erased_without_its_task_is_a_violation` builds its
  wrong store).

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 1).
- `temper/docs/plan/playground-reset-r5/notes/jig-remove-project.md`,
  sections 1, 2 and 5.
- `jig/docs/design/protocol/store.md`, 4.4 (every family);
  `skein/docs/foundation/testing-strategy.md`, section 1.
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s4-jig-projects.md`,
  sections 1 and 6 (the harness, the referee, `check_ending`'s model, the
  binaries).

## Owns and may change

- Owns the files in `files`.
- May change: `jig/tests/core/src/observations.rs` only if the
  conformance world's observer is defined there and must carry the
  `ProjectRemoved` answer (add the field, nothing else).
- Not yours: any `jig/crates/*` code. A row the deletion leaves is a
  finding: the test stays red for it, marked `#[ignore = "..."]` naming
  the row and the task that should erase it, and the report lists it under
  `## Left` (fab routes it); never weaken the check.
- Options: if a family's rows cannot be made through the world's programs
  (placed holds are designed, not built), skip that family with a
  comment citing `jig/docs/design/domain/tasks.status.md` and say so.

## Steps

1. The referee's `names_project` and end check; the negative test:
   `heavy cargo nextest run -p jig-conformance-world --test suite -E
   'test(/referee/)'`.
2. The scenario module and the per-family cases: `heavy cargo nextest run
   -p jig-conformance-world --test suite -E 'test(/removal/)'`.
3. The cuts.
4. `cargo fmt`; `heavy cargo clippy -p jig-conformance -p
   jig-conformance-world --all-targets -- -D warnings`; the focused tests
   of `jig-conformance`, `jig-conformance-world` (the referee is read by
   every conformance case: a change of meaning, so the whole suite), and
   `heavy cargo check --tests` for each package that depends on
   `jig-conformance` (`rg -l jig-conformance --glob Cargo.toml`). Commit
   and push.

## Report

As `notes/common.md` says; per family, the case's name and whether it
passed, and every row the deletion left.
