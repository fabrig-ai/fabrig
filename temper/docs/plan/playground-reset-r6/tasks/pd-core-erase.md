+++
id = "pd-core-erase"
part = "pd"
kind = "feature"
needs = ["pd-core-decide", "pd-tasks-erase", "pd-people-purge", "pd-store-reads"]
files = [
  "jig/crates/jig-domain-core/**",
  "jig/tests/core/**",
  "jig/testing/jig-conformance/src/referee.rs",
  "jig/testing/jig-conformance/src/referee/**",
]
effort = "high"
minutes = 120
+++

## Goal

The `Erasing` stage of a project's deletion (contracts J4, J6): every ended task of the
project erased whole at once, as retention would erase it at the horizon, and its decided
person proposals with it; then the row moves to `Releasing` (whose work is
`pd-core-release`'s). A start resumes a deletion at `Erasing` from the beginning of its scan.

Acceptance:
- **The scan.** For the one deletion at `Erasing` (deletions erase one at a time, in project
  order), the core pages `reads::Scope::ProjectRows { project }` with its own load
  (`Held::ReadLoad`, a load token of the core's own, not a party's read slot; `party: 0`),
  page after page by position, until `next` is `None`.
- **Each decided person proposal** a page returns: erased (`Write::Erase`), with every `Update`
  row the core's index functions derive for it (`indexes.rs`).
- **Each ended task** a page returns (record `R`): the core pages
  `reads::Scope::TaskRows { task }` and erases every row returned (`Write::Erase(row.key())`);
  for each `Activity` row also the reply `Update` rows `indexes.rs` derives for that line
  (the keys its save made). When `TaskRows` is done, one decision finishes the task:
  `tasks::Event::Erase { record: R }` (its ended row, indexes, edges, stub, holds), the core's
  own `Result` row and result `Update` (as `forget.rs`/`indexes.rs` compute them),
  `Write::DropTranscript { task }`, and `people::Event::EraseTask { task, project }`. The ended
  row goes in this last decision, so a crash before it finds the task again at the next scan
  and redoes it (every erase is of a key, harmless when absent).
- **Bounded.** A page's rows are bounded so that one page's writes fit one decision's write
  bound (the implementer names the bound and the rows per page it gives); a task's rows take
  as many pages as they need.
- **Then** `people::Event::Deletion { project, Some(Releasing) }`; the next deletion at
  `Erasing`, if any, starts.
- **At each start**, on `tasks::Event::Restored` (after `pd-core-decide`'s `Ending` resume),
  a deletion at `Erasing` starts its scan from the beginning.
- After `Erasing`, the store holds no row of the project's ended tasks (tasks, core and people
  families), no transcript file of theirs, and no decided person proposal of the project.
- The world's observer and referee accept an erased task: its erase is reported as a fact
  that clears every receipt the referee holds for it (task, ended, turns, terminals), so the
  cold-store check does not expect them after a crash.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J4 (Erasing list), J6.
- `jig/docs/design/protocol/store.md`, 8.5 (a dropped transcript) and 11.1 (retention).
- The code: `forget.rs` (the keys of an ended owner), `indexes.rs` (result and reply update
  keys), `reads.rs` (`Held::ReadLoad`, `Event::ReadLoaded`, its `reads` slab),
  `routing/result_reads.rs` (core-owned loads), `jig/tests/core/src/observations.rs`
  (`submitted` ~667, the `Forgotten` fact and its read-marker assertion ~704),
  `jig/testing/jig-conformance/src/referee.rs` (`Forgotten` ~944, `ColdStore` ~1069).

## Owns and may change

Owned: `jig/crates/jig-domain-core/**`, `jig/tests/core/**`, and in `jig-conformance` its
referee (`src/referee.rs`, `src/referee/**`) for the erased-task fact only. The shapes of
`tasks::Event::Erase`, `people::Event::EraseTask`, the scopes and `Write::DropTranscript` are
earlier tasks'.

Options, pre-approved:
- The erased-task fact: default, a new `Observed::Erased { task }` the observer emits when a
  commit erases `tasks::Key::Ended(task)` with no forget intent, and the referee clears `Task`,
  `Ended`, `Turn` and `Terminal` receipts of the task on it. If extending `Forgotten` is
  smaller and keeps the forget checks for forgotten-once-read tasks, do that; say which.
- If the observer's assertion that read markers go with their task's ended row fires because
  the task's ended position went in an earlier page's decision, scope that assertion to
  forgotten tasks.
- If `Held::ReadLoad` cannot carry a core-owned load without a party, use the core's
  existing core-owned load path (`routing/result_reads.rs`) for the same scopes.

## Steps

1. A new module `deletion/erase.rs` (or beside `pd-core-decide`'s deletion module): the scan
   state (one per core: project, phase, task, position) in memory only; the load answers; the
   per-task finish; the move to `Releasing`; the start hook.
2. The observer and referee fact.
3. Tests in `jig/tests/core/tests/suite/deletion.rs`:
   - `erasing_leaves_no_row_of_the_projects_ended_tasks` (a project with a conversation that
     delegated, a person task, turns and a transcript, history, an answered question, a decided
     person proposal; delete; drain to `Releasing`; walk the store's rows: none of those tasks'
     keys remains, the transcript reads nothing; the other project's tasks are whole).
   - `an_erased_tasks_inbox_entries_and_positions_are_gone` (its requester's inbox and read
     positions no longer name it, before and after a restart).
   - `erasing_pages_through_a_task_larger_than_one_page`.
   - `a_crash_during_erasing_resumes_from_the_start_and_ends_the_same` (crash after each
     commit of the erasing stage in turn; each run ends with the same rows as the uncut run).
   Commands: `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/deletion::/)'`,
   then `heavy cargo nextest run -p jig-domain-core --lib`,
   `heavy cargo nextest run -p jig-core-world --test suite`,
   `heavy cargo nextest run -p jig-conformance --lib` and
   `heavy cargo nextest run -p jig-conformance-world --test suite` (the referee change is a
   change of meaning for both worlds).
4. `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-core-world -p jig-conformance --all-targets`;
   old forms; commit and push.

## Report

The page bound used and why; any row of an ended task you found that neither scope returns
nor the record names (it is a question for `pd-conformance`'s inventory).
