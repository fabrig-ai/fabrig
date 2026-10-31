+++
id = "jp-referee"
part = "project-delete"
kind = "feature"
needs = ["jig-acts"]
files = [
  "jig/testing/jig-conformance/src/referee.rs",
  "jig/testing/jig-conformance/src/referee/removal.rs",
  "jig/tests/core/src/observations.rs",
  "jig/tests/core/src/observations/naming.rs",
  "jig/tests/core/tests/suite/naming.rs",
  "jig/tests/conformance/tests/suite/referee_removal.rs",
]
effort = "high"
minutes = 80
+++

## Goal

Give jig's conformance referee the evidence and the promise a project's
deletion is judged by (playground-reset.md, 5 and 8, "the referee
checking at the end that no row names the project"): an independent walk
of the durable store counting, per project, the rows that name it or one
of its tasks, and a promise that once a project's removal has been
answered and the world is quiescent, the count is zero; and that the
tasks of a removed project may leave the durable view while any other
task's leaving stays a violation. The world's deletion itself comes
later (`jp-delete`); this task judges it from synthetic evidence and
from rows written by the codecs.

What names a project, for the walk (`notes/contracts.md`, 1): every row
of jig's families that carries the project number or the number of one
of its tasks (live or ended at any time the observer saw), save two
kinds that outlive the project by design: its funding ledgers (tasks'
`Ledger` and `ClosedLedger`, contract 1) and the people family's retained
answers (`Answer` and `Acted`, which answer *unknown* once the project
is gone: contract 1's last bullet; erasing them would let a replayed key
act again). An account naming the project in its uses counts.

Acceptance:

- `jig-core-world`, new `tests/suite/naming.rs`, on a real test store
  with rows written through the codecs:
  - `every_family_row_naming_a_project_is_counted`: one row of each kind
    that names project 3 or a task of it (tasks `Live`, `Ended`,
    `History`, `Milestone`, `Activity`, `Stub`, `Update`, `Result`,
    `EndedByParty`, `EndedByProject`, `EndedDelegate`, `Delegate`,
    `PlacedHold`, `PersonProposal`, `DecidedProposal`; core `Projection`,
    `RunProof`, `Call`, `Terminal`, `EscalationDecision`,
    `ProposalDecision`; settings `Proposal` by a task; people `Roles`,
    `Policy`, `ReadConversation`, `EndedRead`; notes `Line` and `Entry`
    of its scopes; an account with `Use::Project(3)`; a transcript or
    attempt row of its task): each is counted once for project 3.
  - `ledgers_and_retained_answers_are_not_counted`: its `Ledger`,
    `ClosedLedger`, an `Answer` and an `Acted` naming it: zero.
  - `another_projects_rows_are_not_counted`: rows of project 4 count for
    4 only.
- `jig-conformance-world`, new `tests/suite/referee_removal.rs`, feeding
  the referee observations as `referee.rs`'s tests do:
  - `a_removal_with_no_row_left_holds`: `ProjectRemoved` answered, then a
    durable snapshot naming nothing of it, then quiescence: no violation.
  - `a_row_left_after_a_removal_is_a_violation`: the same with one
    `History` row of its task left: `Promise::Erasure` (new) at the
    check.
  - `a_removed_projects_tasks_may_leave`: its tasks leave the durable view
    after the answer: no violation; `another_tasks_leaving_is_still_lost_work`:
    a task of another project leaving gives today's violation.
  - `rows_before_the_answer_are_not_judged`: a snapshot naming the project
    before its answer is no violation.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 1.
- `temper/docs/design/draft/playground-reset.md`, sections 5 and 8.
- `jig/docs/design/domain/testing.md` (the referee's promises and its
  evidence: rg `Promise` and `referee`).
- `jig/docs/design/protocol/store.md`, 4.4 (every family and its rows).
- Survey of the families: `jig-protocol-store/src/keys/*.rs` and the
  record types they key (`jig-domain-tasks/src/boundary.rs`'s `Stored`,
  `jig-domain-core/src/stored.rs`, `jig-domain-people/src/boundary.rs`,
  `jig-domain-notes/src/boundary.rs`, `jig-domain-accounts/src/records.rs`).

## Owns and may change

Owns the files in the header. May change, as `notes/scope.md` says:
`jig/testing/jig-conformance/src/referee.rs` (new `Observed` variants,
`Snapshot`'s new field, `Promise::Erasure`, the check), every exhaustive
match on `Observed` or `Promise` they force, `jig/tests/core/src/lib.rs`
and `observations.rs` (the walker's `mod` line, filling `Snapshot`'s new
field in `durable_at`, and observing `ProjectRemoved` answers where the
observer sees deployment acts' and parties' outcomes), the two worlds'
`tests/suite/main.rs` (`mod` lines). The walk reads the store through the
test store's own row iteration (`jig_test_store_files::Store`); if it has
none, add a read-only iterator there and say so.

Not yours: the conformance scenarios and their cuts (`jp-conformance`);
the core's deletion.

## Steps

1. `referee.rs`: `Snapshot` gains `pub naming: BTreeMap<u32, u32>`, rows
   naming each project as the walk counts them (projects with none left
   out). `Observed::ProjectRemoved { project: u32 }`, observed when a
   party or the deployment receives `Outcome::ProjectRemoved`. The
   referee keeps the removed projects; at the end-of-world check
   (`check_at` once quiescent, or the scenario-ending check where the
   referee judges quiescence today) each removed project's `naming` in the
   last durable snapshot is zero, else `Violation { promise:
   Promise::Erasure, .. }`. Where the referee today judges a task leaving
   the durable view as lost work, a task of a removed project (its project
   from the snapshot it was last seen in) may leave once the removal was
   observed. Put the new logic in `referee/removal.rs`.
2. `observations/naming.rs`: `pub fn naming(store: &Store, tasks: &mut
   BTreeMap<u64, u32>) -> BTreeMap<u32, u32>`: walks every row of jig's
   families decoded by the store codecs; `tasks` (kept by the observer
   across snapshots) learns each task's project from `Live` and `Ended`
   rows and is never pruned, so a row naming an erased task is still
   attributed. Each record kind's fields are read directly, written here
   from the record types, not by calling the core's planner
   (`jp-planner`): the walk must be independent of what it judges.
3. Tests: the acceptance's. Then, one at a time:
   - `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/naming/)'`;
   - `heavy cargo nextest run -p jig-conformance-world --test suite -E 'test(/referee_removal/)'`
     (binary names from each world's `Cargo.toml`);
   - `heavy cargo nextest run -p jig-conformance -p jig-core-world -p jig-conformance-world`;
   - `cargo fmt`; clippy over the packages you edited; old forms from `jig/`.
4. Commit and push.

## Report

The standard report, plus: each record kind the walk reads and the field
it attributes by; any row kind you found that names a task by a field
the walk cannot attribute.
