+++
id = "pd-sweep"
part = "project-deletion"
kind = "interface"
needs = ["jig-acts"]
files = [
  "jig/crates/jig-domain-core/src/reads.rs",
  "jig/crates/jig-domain-core/src/reads/sweep.rs",
  "jig/crates/jig-domain-core/src/routing/vocabulary.rs",
  "jig/crates/jig-domain-core/src/routing/mod.rs",
  "jig/crates/jig-protocol-store/src/boundary.rs",
  "jig/crates/jig-protocol-store/src/loads.rs",
  "jig/crates/jig-protocol-store/src/intervals.rs",
  "jig/crates/jig-protocol-store/src/tests/sweep.rs",
  "jig/testing/jig-test-store/src/**",
  "jig/testing/jig-test-store-files/src/**",
]
effort = "high"
minutes = 60
+++

## Goal

The two store doors a task's erasure needs (draft 5, "erased whole at
once ... its transcript's file with it"; `jig/docs/design/protocol/store.md`,
8.5 and 11.1): the core may drop a task's transcript, and may load,
page by page, the rows a sweep names, typed, so it can erase them by
key in its own decisions. Surface and plumbing only: nothing in the
core sends either yet (`pd-erase-task` does). Every root routes both,
temper's included, as it routes `NotesLoad` and `Write::Erase` today.

Acceptance:

- `jig_domain_core::reads::Sweep` with `admits`, `Write::DropTranscript`,
  `Held::Sweep` and `Event::Swept` exist as below;
- the store's `Load::Sweep` answers exactly the rows `Sweep::admits`
  names, in key order, paged (test below), and `Write::DropTranscript`
  from the core reaches the store's existing `Write::DropTranscript`;
- every root (jig's test root and service, the ops example, temper's
  root and service, every world's store) routes `Held::Sweep` to one
  page of `Load::Sweep` and answers `Event::Swept`, and translates the
  new write; `heavy cargo check --tests` passes for each (Steps).

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md`.
- `jig/docs/design/protocol/store.md`, 4.3 (ranges to intervals), 4.4,
  7.1 (pages), 8.5 (a transcript dropped), 11.1 (retention).
- Models to copy: `jig_domain_core::RestoreRange::admits`
  (`restart.rs`) and its intervals in `jig-protocol-store`
  (`intervals.rs`, `restore.rs`); the `NotesLoad` path end to end
  (`routing/notes.rs`, `Held::NotesLoad`, `jig-test-store`
  `route.rs` `StoreOut::NotesLoad`/`Flight::Notes`, temper's
  `route/held.rs` `notes_load`, `CoreRange::Notes`,
  `temper-service/src/store/ranges.rs`).
- `temper/docs/plan/playground-reset-r4/notes/surveys/s3-jig-project.md`,
  section 2 (the families; the transcript drop; the core `Write`
  readers listed under "core write variant addition").

## Owns and may change

Owned: the `files` above. Neighbours, routing and the compiler's arms
only, each mirroring `NotesLoad`'s or `Write::Erase`'s handling there:
`jig/testing/jig-test-domain/**`, `jig/testing/jig-test-service/**`,
`jig/examples/ops/**`, `jig/tests/**` (worlds' stores and models:
`jig/tests/store/src/model.rs`, `jig/tests/channel/src/world/store.rs`,
`jig/tests/core/src/**`, `jig/tests/one-process/src/referee.rs`, ...),
and temper's call sites: `temper-engine-domain` (`boundary.rs`
`CoreRange`, `route/held.rs`, `decision.rs`, `domain.rs`, `store.rs`,
`route.rs`), `temper-service` (`store/ranges.rs`, `store/route.rs`,
`route.rs`, `clients/translation.rs`), `temper/tests/**` (exhaustive
`Delivery` and `Write` matches). `pd-store` runs beside this task and
owns the store's codecs, `lib.rs` and the core's `lib.rs` and
`stored.rs`: touch them only for a compiler-forced arm, and say so.

Pre-approved: if the store's page assembly cannot filter by a
predicate, `intervals::sweep` gives exact intervals and `admits` is
their test oracle (as `RestoreRange`'s is); if a family named below is
not keyed with the task first, leave it out of `Task` and say so.

## Steps

1. `jig/crates/jig-domain-core/src/reads/sweep.rs` (new; `pub mod
   sweep; pub use sweep::Sweep;` in `reads.rs`), module doc citing
   store.md 11.1:

   ```rust
   /// Rows the core loads to erase them by key (protocol/store.md, 11.1).
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub enum Sweep {
       /// The tasks family's `Ended` rows numbered above `after`, by number.
       Ended { after: u64 },
       /// Every row keyed under one task in a family whose key begins
       /// with it: the tasks family's `History`, `Milestone` and
       /// `Activity`, people's `EndedRead`, the core's `Terminal` and
       /// `EscalationDecision`. Turns are the transcript's.
       Task(u64),
       /// The notes child's index lines of the project's scopes
       /// (`Scope::Project`, `Goal` and `Resources` of the project).
       Notes { project: u32 },
   }
   impl Sweep {
       #[must_use]
       pub fn admits(&self, key: &crate::Key) -> bool;
   }
   ```

2. `routing/vocabulary.rs`: `Write::DropTranscript(u64)` ("the task's
   transcript dropped whole, its file with it: store.md 8.5");
   `Held::Sweep { owner: Token, sweep: Sweep }` ("one page of the
   sweep's rows, after every earlier commit is durable"); `Event::Swept
   { owner: Token, rows: Box<[Record]>, more: bool }` (`more`: the
   store's page had a next cursor). `routing/mod.rs` routes
   `Event::Swept` to nothing yet, with `// pd-erase-task consumes it.`
3. `jig-protocol-store`: `Load::Sweep(core::reads::Sweep)` in
   `boundary.rs`; `loads.rs` and `intervals.rs` turn it into the
   families' intervals and answer `Loaded` as `Load::Notes` is; the
   commit path takes the core's `Write::DropTranscript(task)` as the
   store's `Write::DropTranscript { task }`.
4. Roots: `Held::Sweep` becomes one `Load::Sweep` page, bounded as each
   root bounds a `NotesLoad` page, its `Loaded` rows handed back as
   `core::Event::Swept { owner, rows, more: next.is_some() }`; temper
   gains `CoreRange::Sweep(jig_domain_core::reads::Sweep)` beside
   `CoreRange::Notes`, through its archive reads, as `notes_load` does.
   Each root's `core::Write` translation passes `DropTranscript` on.
5. Tests:
   - `jig-protocol-store` `src/tests/sweep.rs`:
     `a_task_sweep_loads_every_row_keyed_under_the_task_and_no_other`
     (rows of tasks 7, 8 and 9 in each named family; `Task(8)` pages
     back exactly task 8's, in key order, and `admits` agrees with each
     row); `an_ended_sweep_pages_ended_rows_above_its_number`;
     `a_notes_sweep_loads_the_projects_scoped_lines_alone`;
     `a_core_transcript_drop_removes_the_file_and_its_places` (through
     the core's write, as the store's own drop test does).
   - `jig-domain-core` unit test of `admits` over one key of each
     family (admitted and not).
6. Checks: `cargo fmt`; clippy on `jig-domain-core`,
   `jig-protocol-store`, `jig-test-store`, `jig-test-store-files`;
   focused tests of `jig-protocol-store` and `jig-domain-core --lib`;
   then `heavy cargo check --tests`, one at a time, for
   `jig-test-domain jig-test-service jig-ops-domain jig-store-world
   jig-channel-world jig-core-world jig-one-process-world
   jig-conformance-world temper-engine-domain temper-service
   temper-engine-domain-world temper-one-process-world
   temper-client-world temper-system-world` (take each name from its
   directory's `Cargo.toml` where it differs). Commit, push.

## Report

fab's sections; `## Interfaces`: `Sweep` and `admits`, the three
vocabulary items, `Load::Sweep`, temper's `CoreRange::Sweep`; which
families `Task` covers, by key; each root's file and line routing
`Held::Sweep`.
