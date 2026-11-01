+++
id = "pd-tasks-erase"
part = "pd"
kind = "feature"
needs = ["pd-vocabulary"]
files = [
  "jig/crates/jig-domain-tasks/**",
  "jig/tests/tasks/**",
]
effort = "high"
minutes = 80
+++

## Goal

Give the tasks child's two deletion events their bodies (contracts J4, J6). `Erase { record }`
is the tasks child's part of erasing one ended task whole: the rows of its families that the
ended row names. `ForgetProject { project }` is its part of a project's release: the
project's live ledgers, pending person proposals and placed holds.

Acceptance:
- `Event::Erase { record }` (the record is a `TaskRecord` of an ended task, loaded by the
  core) emits `Request::Erase` for exactly the keys `closing::end_task` saved for it, computed
  from the record as `end_task` computes them: `Key::Ended(number)`;
  `Key::EndedByParty { .. }` when the requester is a person; `Key::EndedByProject { .. }` when
  `flags.listed`; for a delegate (`requester: Party::Task(parent)`) `Key::EndedDelegate
  { requester: parent, task }` and `Key::Delegate { requester: parent, task }`. It also drops
  and erases the task's `Stub` if the child holds one, and every placed hold the child holds
  whose owner is the task or whose scope is its subtree. It emits nothing else and keeps no
  state of the record. An `Erase` for a number that is live is refused by ignoring it and
  emitting nothing (the core sends only ended tasks; a debug assertion may say so).
- `Event::ForgetProject { project }` closes each live ledger of the project
  (`Funder::Pool { project, .. }`, `Funder::Recurring { project, .. }`,
  `Funder::Period { project, .. }`) as `retire::close` closes one (its `Ledger` erased, its
  `ClosedLedger` saved: settled spend stays, contracts J4); drops and erases every pending
  person proposal of the project (`Key::PersonProposal`) and every placed hold of the project
  (`Key::PlacedHold`). A live task of the project when it arrives is left as it is (the core
  sends it only once none is live); its ledgers then stay too.
- History, milestone, activity, delegate-prefixed, decided-proposal and core rows are not this
  child's to erase here: the core erases them from its store reads (`pd-core-erase`).

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J4, J6.
- `jig/docs/design/domain/tasks.md`, its retention class and section 2 (the rows).
- `jig/docs/design/protocol/store.md`, 11.1 (retention) and 15.1 (closed funding).
- The code: `src/closing.rs` `end_task` (~306-472, the keys it saves), `src/retire.rs`
  `close`, `src/placed.rs` (~502-562, holds lifted and erased), `src/person_proposals.rs`.

## Owns and may change

Owned: `jig/crates/jig-domain-tasks/**` and `jig/tests/tasks/**`. Nothing outside. The
events' shapes are `pd-vocabulary`'s.

Options, pre-approved:
- A key `end_task` computes in a helper you can reuse: reuse it (one source for save and
  erase); if it is inline, extract it into a `pub(crate)` function both call.
- If closing a pool ledger needs its period closed first (the order `retire` keeps), close
  in that order.

## Steps

1. New module `src/erase.rs` (module doc cites `draft playground-reset.md, 5` and
   `tasks.md`'s retention class): `pub(crate) fn erase(domain, record, out)` and
   `pub(crate) fn forget_project(domain, project, out)`; `step` routes the events to them.
2. Tests in `src/tests/` (new `erase.rs`, registered in `tests.rs`):
   - `erasing_an_ended_person_task_erases_what_its_end_saved` (end a person-requested listed
     task, take the saves its end emitted, send `Erase` with its record: the erases are
     exactly the ended-family keys saved).
   - `erasing_an_ended_delegate_erases_its_edges_under_its_parent`.
   - `forgetting_a_project_closes_its_live_ledgers_and_keeps_their_spend` (a period and a
     pool with spend; after `ForgetProject` the `ClosedLedger` rows carry the same numbers and
     no `Ledger` of the project remains; another project's ledgers stay).
   - `forgetting_a_project_drops_its_person_proposals_and_holds`.
   And in `jig/tests/tasks/tests/suite/` (new `erase.rs`, registered in `main.rs`):
   `an_erased_task_leaves_no_ended_index_after_a_restart` (the world's store, restart, the
   restored child holds nothing of it).
   Commands: `heavy cargo nextest run -p jig-domain-tasks --lib -E 'test(/erase/)'`,
   `heavy cargo nextest run -p jig-tasks-world --test suite -E 'test(/erase/)'`, then
   `heavy cargo nextest run -p jig-domain-tasks --lib` and
   `heavy cargo nextest run -p jig-tasks-world --test suite` if you extracted a helper from
   `end_task`.
3. `cargo fmt`; `heavy cargo clippy -p jig-domain-tasks -p jig-tasks-world --all-targets`;
   old forms; commit and push.

## Report

The keys `Erase` erases, each with the `end_task` line that saved it.
