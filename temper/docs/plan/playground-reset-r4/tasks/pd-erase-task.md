+++
id = "pd-erase-task"
part = "project-deletion"
kind = "feature"
needs = ["pd-sweep", "pd-store", "pd-people-gone"]
files = [
  "jig/crates/jig-domain-core/src/erase.rs",
  "jig/crates/jig-domain-core/src/tests/erase.rs",
]
effort = "high"
minutes = 55
+++

## Goal

An ended task erased whole and at once (draft 5, "erased whole at
once, as its retention would erase it at the horizon ... its
transcript's file with it"; `jig/docs/design/protocol/store.md`, 11.1):
the mechanism retention would use, without its horizon timer. Given an
ended task's record, the core drops its transcript, sweeps every row
keyed under it and erases each by key, then, last, erases its record
and every row the record names, and tells the people child. The
project's deletion calls it for each of its ended tasks (`pd-delete`);
nothing else calls it yet.

Acceptance: after an erasure the store holds no row keyed by the task
or naming it (its `Ended` record, ended indexes, result and result
update, its replies' updates, history, milestones, activity,
terminals, escalation decisions, every party's read positions, its
per-task ledger, its transcript file and places), and no in-memory
entry of the core or the people child names it; a crash at any point
leaves the `Ended` record until the last step, so a second erasure of
the same record finishes the work (tests below).

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md`.
- `jig/docs/design/protocol/store.md`, 7.4 (indexes), 8.5, 11.1.
- `jig/crates/jig-domain-core/src/forget.rs` (the keys an ended task's
  forgetting erases, and how it computes them from the record: the
  model for the last step), `jig/crates/jig-domain-tasks/src/index.rs`
  (`Update`, `UpdateObject::Reply`, the ended index keys),
  `activity.rs` (`ActivityLine`: a reply line's `at` and message).
- `pd-sweep`'s `reads::Sweep`, `Held::Sweep`, `Event::Swept`,
  `Write::DropTranscript`; `pd-people-gone`'s `people::Event::TaskErased`.

## Owns and may change

Owned: the `files` above. Neighbours: `jig-domain-core/src/tests.rs`,
one `mod erase;` line placed directly after its `mod forget;` (`pd-close`, beside
this task, adds its own `mod removal;` line in that file); `jig-domain-core/src/lib.rs`
(`mod erase;` and one `Core` field), `routing/mod.rs` (route
`Event::Swept` to `erase::swept`, replacing `pd-sweep`'s comment line),
and the core's in-memory maps that hold per-task entries (inbox census,
result reads): one call each to drop the task. `pd-close` runs beside
this task and owns `routing/removal.rs`, `routing/people.rs`,
`routing/deployment_acts.rs` and `jig-domain-tasks/src/domain.rs`:
don't touch them.

Pre-approved:

- reply updates: by default their keys are computed from the reply
  lines the activity sweep returns (`Update { party: requester, at:
  line.at, task, object: Reply { .. } }`, as the tasks child writes
  them: find the writer and match it); if a reply's update key cannot
  be computed from its line, say so and leave those rows;
- a `Stub` of the task that the tasks child still holds is named by
  live work and stays; one it does not hold is erased by key;
- `ClosedLedger(Funder::Task(task))` is erased (spend is never per
  task: draft 5); a period's, a pool's or a recurring ledger
  (`Funder::Recurring`) is not this task's to touch: the project's rows
  close and erase the recurring ones (`pd-project-rows`; Answers).

## Steps

1. `erase.rs` (new, step code; module doc citing store.md 11.1 and
   draft 5):

   ```rust
   /// One task's erasure in flight; volatile: a restart begins it again
   /// from the record, which is erased last.
   pub(crate) struct Erasing { project: u32, record: Box<tasks::TaskRecord> }

   /// Begin erasing an ended task whole.
   pub(crate) fn erase_task(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>,
       out: &mut Queue<Request>, record: Box<tasks::TaskRecord>);
   /// A sweep page: erase each row by key, then sweep again or finish.
   pub(crate) fn swept(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>,
       owner: Token, rows: Box<[Record]>, more: bool) -> Requests;
   /// Whether this task's erasure is in flight.
   pub(crate) fn erasing(core: &Core, task: u64) -> bool;
   /// The erasure's end; `pd-delete` continues the deletion here.
   fn finished(core: &mut Core, env: &Env<Limits>, work: &mut Queue<Event>,
       out: &mut Queue<Request>, project: u32, task: u64);
   ```

   `Core` gains `erasures: Map<Token, Erasing>` (capacity: the projects
   bound; one erasure per removing project at a time). `erase_task`:
   `Write::DropTranscript(task)`, then `Held::Sweep { owner, sweep:
   Sweep::Task(task) }`. `swept`: `Write::Erase` of each row's key (and
   of each reply line's update); `more` sends the same sweep again (it
   follows the commit, so erased rows are gone); else the last
   decision: the record's own keys (`Ended`, `EndedByParty`,
   `EndedByProject` when listed, `Result`, the result `Update`,
   `EndedDelegate` when its requester is a task, the per-task ledger),
   `people::Event::TaskErased { task, project }`, the core's own maps,
   then `finished` (empty but for `// pd-delete continues the project's
   deletion here.`).
2. Tests, `jig-domain-core` `src/tests/erase.rs` (its `mod` line in
   `tests.rs`), on the crate's fixtures that end a task (as
   `src/tests/forget.rs` does), feeding `Event::Swept` by hand with the
   rows a store would hold:
   - `an_erasure_drops_the_transcript_and_sweeps_the_tasks_rows_first`
     (the first decision's writes and its held sweep);
   - `a_full_sweep_page_sweeps_again_and_an_empty_one_finishes`;
   - `the_last_step_erases_the_record_and_every_row_it_names`
     (each key, the people event);
   - `a_reply_lines_update_is_erased_with_it`;
   - `an_erasure_begun_again_from_the_same_record_finishes` (a restart
     between steps: the second `erase_task` sweeps what is left).
3. Checks: `cargo fmt`; clippy on `jig-domain-core`; `heavy cargo
   nextest run -p jig-domain-core --lib`; `heavy cargo check --tests -p
   jig-core-world -p jig-conformance-world -p temper-engine-domain`.
   Commit, push.

## Report

fab's sections; `## Interfaces`: the four functions and `Erasing`;
the keys of the last step, family by family; what is left by an option
taken (stubs held, reply updates not computable).

## Answers

2026-11-01. Question: may the sweep owner add discovery of this task's
archived recurring ledgers, or does acceptance leave them? Answer:
neither here. A `Funder::Recurring { project, task, period }` ledger
is a per-period allotment of the project's period: its spend posts
into the period's ledger when it closes (`retire.rs`), so it is the
period's accounting, keyed project first. The erasure leaves every
`Ledger` and `ClosedLedger` of `Funder::Recurring` (and an ended
record whose `funder` is one), and adds no sweep. `pd-project-rows`
closes the project's live recurring ledgers into the period and erases
them, then sweeps and erases its archived `ClosedLedger(Recurring {
project, .. })` rows (`Sweep::Recurring { project }`, one exact
interval), so the deletion still ends with no row naming the task.
The erasure's last step erases `Ledger(Funder::Task(task))` and
`ClosedLedger(Funder::Task(task))` as before. Report the recurring
ledgers as left to `pd-project-rows`.
