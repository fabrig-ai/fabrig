+++
id = "person-reads"
part = "person"
kind = "feature"
needs = ["person-surface"]
files = ["jig/crates/jig-domain-core/src/reads/person.rs", "jig/crates/jig-domain-core/src/reads.rs", "jig/crates/jig-domain-core/src/reads/rows.rs", "jig/crates/jig-domain-core/src/tests/reads/**", "jig/tests/core/tests/suite/person_reads.rs", "jig/testing/jig-core-kit/src/**", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 240
+++

## Goal

Make jig's run read as its person whole
(`jig/docs/design/domain/engine.md`, 7.3, *A run's read as its person*):
the reads that name the run's project or one task of it are answered as
the task's requester, a person, would be answered by a client's read of
the same, and every other read is answered unknown, so nothing of the
person's other projects reaches the run.

Acceptance:

- `reads/person.rs`'s `run`, after `person-surface`'s checks (asking
  family, person requester), admits exactly contracts, section 1's
  *Admitted reads*, the project being the run's task's: a read naming a
  project is checked at admission; a read naming a task is checked against
  the live task's project at admission, or, where the task is retained,
  against the loaded row's owner project when its load completes. Every
  other read answers `Now::PersonAnswer` with `Answer::None(Why::Unknown)`.
- An admitted read is made as `reads::start_jig` makes the person's own
  read (the same visibility, documents, pages, bounds and loads), with a
  destination of its own (`Destination::Person { to, key, project }` or
  the like) in place of a sign-in. Its answer is the same `Answer` a
  signed-in session of that person gets for the same read, delivered as
  `Now::PersonAnswer`; busy as `Answer::None(Why::Busy)`.
- A stored read's completion checks again, before answering: the run's
  attempt is current (a replaced attempt's answer is dropped, as
  `reads/settings.rs`'s `proposal_loaded` drops it), its requester is the
  same person, the person still holds a role in the project, and every row
  is of the project; where any fails, `None(Why::Unknown)`.
- An admitted `Read::Application` in the project's scope emits
  `Request::Ask { connector, ask: Ask::Read { reply: to, scope, read,
  party, window } }`, `party` the person, as a party's read does, and no
  `PersonAnswer`.
- Nothing is committed, no call record is kept, no read position moves.
- `jig/docs/design/domain/engine.status.md`: the **Not yet** entry on 7.3,
  *A run's read as its person*, is deleted.

## Inputs

- `temper/docs/plan/ui-ask-m2/notes/common.md` and
  `temper/docs/plan/ui-ask-m2/notes/contracts.md`, section 1.
- `jig/docs/design/domain/engine.md`, 7.3 (*A run's read as its person*,
  *A run's proposal read* for the stored re-check) and 5.3;
  `jig/docs/design/client/domain/contract.md`, section 6 and 6.1.
- The party read pipeline: `jig/crates/jig-domain-core/src/reads.rs`
  (`start`, `start_jig`, `Pending`, `Destination`, `loaded`) and
  `reads/rows.rs` (`scope_allowed`, `owner_rows`, `live_page`);
  `reads/vocabulary.rs` (`read_row_visible`, `row_visible`); the run
  reads to copy from: `reads/settings.rs` (`run`, `run_proposal`,
  `proposal_loaded`) and `reads/run_task.rs`.
- Fixtures: `jig/tests/core/tests/settings/support.rs`
  (`settings_run_with_limits`, `settings_goal`), `person-surface`'s story
  and `World::person_read` in `jig/tests/core/tests/suite/person_reads.rs`.

## Owns and may change

- **Owns:** `reads/person.rs`; the person destination in `reads.rs` and
  the branches of `rows.rs` it needs; the core's unit tests under
  `src/tests/reads/`; the story file `person_reads.rs`; jig-core-kit's
  person-read helpers.
- **Reader files, call sites only:** `routing/run/calls.rs` if `run`'s
  signature must change.
- **Options, pre-approved:** if `start_jig` cannot take a destination
  without a sign-in, split its admission from its reply (a function both
  `start` and `person::run` call) and say so; never copy its body. If
  `Read::Tasks`' live part admits a query naming no project for a person
  (their own requests), the run's read still refuses it, at admission. If
  a retained task's project can only be known once its row loads, check it
  then, as the acceptance says.

## Steps

1. The admission rule as one function, `admitted(read: &Read, project:
   u32) -> Admission` (named, project-checked, task-to-check, refused), with
   a unit table: a row per `Read` variant, inside and outside the project.
2. The destination, its completion and its re-checks.
3. The story, grown from `person-surface`'s into one table of rows in a
   world where the person holds a role in two projects, the run's and
   another, each with live and ended tasks, activity and a turn, a goal's
   note and something waiting on the person:
   - admitted, each compared with the same read through the person's
     signed-in session: `Project`, `Spend`, `Tasks` naming the project,
     `Notes`, `ScopedNotes` of a goal of it, `Inbox` filtered by it,
     `Task` live and retained, `Activity`, `Turn`, `EndedDelegates`;
   - refused unknown: each of those naming the other project or one of its
     tasks; `Tasks` naming no project; `Inbox` unfiltered; `Frame`;
     `Deployment`; `Settings`; `Spend { project: 0 }`; a party's
     `Proposal`;
   - `Application` in the project's scope reaching the connector with the
     person, and in the other's, unknown;
   - a stored read whose person loses their role before the load
     completes, unknown; the store's rows unchanged by every row.
   Break the project check once (admit any project) to see the other
   project's rows fail, and restore.
4. The status entry.
5. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-core-kit -p
   jig-core-world --all-targets -- -D warnings`; `heavy cargo nextest run
   -p jig-domain-core --lib`; `heavy cargo nextest run -p jig-core-world
   --test suite -E 'test(/person_reads|settings_reads|reads/)'`. Commit and
   push.

## Report

As the implementer's text asks; give the story's rows and which read
needed a load.
