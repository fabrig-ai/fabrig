+++
id = "pd-store"
part = "project-deletion"
kind = "migration"
needs = ["jig-acts"]
files = [
  "jig/crates/jig-protocol-store/schema/people-v14.schema",
  "jig/crates/jig-protocol-store/schema/core-v7.schema",
  "jig/crates/jig-protocol-store/src/codec.rs",
  "jig/crates/jig-protocol-store/src/codec/**",
  "jig/crates/jig-protocol-store/src/generated/**",
  "jig/crates/jig-protocol-store/src/lib.rs",
  "jig/crates/jig-protocol-store/src/sizes.rs",
  "jig/crates/jig-protocol-store/src/format.rs",
  "jig/crates/jig-protocol-store/src/keys/core.rs",
  "jig/crates/jig-protocol-store/src/tests/codec.rs",
  "jig/crates/jig-protocol-store/src/tests/codec/**",
  "jig/crates/jig-protocol-store/src/tests/format.rs",
  "jig/crates/jig-protocol-store/golden/**",
  "jig/crates/jig-domain-core/src/lib.rs",
  "jig/crates/jig-domain-core/src/stored.rs",
  "jig/crates/jig-domain-core/src/removal.rs",
  "jig/tests/store-codecs/**",
  "jig/tests/store/golden/**",
]
effort = "high"
minutes = 60
+++

## Goal

jig's store moves once for the whole deletion (`notes/contracts.md`,
D2): people v13 to v14, carrying jig-acts' `Ask::RemoveProject`,
`Act::RemoveProject`, `Act::Cancel` and `Outcome::ProjectRemoved`; core
v6 to v7, carrying the kept deletion, a new live core family
`Removal`. Types and codecs only, no behaviour: the row is restored at
start into the core and nothing writes it yet. No other task of the run
moves a jig family; tasks family stays v9.

Acceptance:

- `people-v14.schema` and `core-v7.schema` exist; the codecs encode and
  decode the four people values and the `Removal` row, both ways,
  exhaustively; jig-acts' "Encoded from pd-store on." arms are gone;
- a store whose people or core family is at the old version refuses to
  open with the existing `OpenRefusal::Format` naming the family;
- the drift test, the layout golden and the codec round trips pass with
  regenerated code and golden bytes;
- a `Removal` row saved, then restored at a restart, is in
  `Core`'s removals (a test in `jig-domain-core`).

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D2, section 4).
- `jig/docs/design/protocol/store.md`, 4.4 (jig's families), 5.1
  (codecs), 5.3 (the format row), 7.2 (the restart's ranges).
- `temper/docs/plan/playground-reset-r4/tasks/jig-acts.md` (the values
  this task encodes; its report's `## Interfaces`).
- The previous moves' shape: `schema/people-v12.schema` beside
  `people-v13.schema`, `src/generated/people_v12.rs`, and
  `jig/tests/store-codecs/tests/suite/drift.rs` (its historical list and
  write mode).

## Owns and may change

Owned: the `files` above.

Neighbours:

- `jig/crates/jig-protocol-store/src/**` beyond the owned files
  (`commits.rs`, `historical.rs`, `open.rs`, `patching/**`,
  `reads/point.rs`, `transcripts/cut.rs`, its other tests): the arms
  for the new key and record alone. `boundary.rs`, `loads.rs` and
  `intervals.rs` are `pd-sweep`'s, which runs beside this task: leave
  them unless the compiler forces an arm, and say so.
- `jig/crates/jig-domain-core/src/restart/*.rs`
  and `stored/fit.rs` (restoring and fitting the live `Removal` row,
  beside `RunProof`'s), `reads/*.rs` and `routing/tasks.rs` (compiler
  arms for the new `CoreKey`/`CoreRecord`).
- every other file of every tree the compiler names for the new
  `CoreKey`/`CoreRecord` variants (temper's `temper-engine-domain`
  `store.rs`, `archive.rs`, `decision.rs`, `restart.rs`,
  `route/escalation.rs`, `route/proposals.rs`, `temper-service`
  `store/ranges.rs`, and temper's engine world): the arm alone, the
  new row treated as `RunProof`'s live core row is.
- `jig/tests/store/tests/suite/layout_golden.rs` fixtures and the
  golden directory: refreshed by their write mode, never by hand.

Pre-approved:

- if the format row's wanted versions are constants elsewhere than
  `format.rs`/`component` schema, change them there;
- if a temper test pins jig's family versions or layout bytes, refresh
  it by its write mode and list it.

## Steps

1. **The row** (`jig/crates/jig-domain-core/src/removal.rs`, new,
   module doc citing draft 5 and `jig/docs/design/domain/people.md`, 5.2):

   ```rust
   /// A project's deletion, kept until its last step (draft 5).
   #[derive(Clone, PartialEq, Eq, Debug)]
   pub struct Removal {
       pub project: u32,
       pub stage: RemovalStage,
   }
   /// Where a deletion stands; each stage resumes at a start.
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub enum RemovalStage {
       /// Its live tasks cancelled; waiting for each to end.
       Ending,
       /// Its ended tasks being erased whole.
       Erasing,
       /// Its connectors told; waiting for each `ProjectReleased`.
       Releasing,
   }
   ```

   `CoreRecord::Removal(Removal)`, `CoreKey::Removal(u32)` (the
   project), `CoreKey::live` true; `Core` gains `pub(crate) removals:
   Map<u32, Removal>`, sized by the projects' bound, restored at start
   from the core live range. Nothing else reads it in this task.
2. **people v14:** copy `people-v13.schema` to `people-v14.schema`,
   append `AskRemoveProject { project }`, `ActRemoveProject { project }`,
   `ActCancel` (the fields of `ActClose`), `OutcomeProjectRemoved {
   project }` as new enum members after the existing ones; generate
   `people_v14.rs` by the drift test's write mode
   (`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p
   jig-store-codecs-world --test suite -E 'test(/drift/)'`), add
   ("people", 13) to its historical list and point the check at 14;
   switch `codec.rs`, `codec/people*.rs`, `lib.rs`, `sizes.rs`,
   `codec/core/calls.rs` to `people_v14`, keeping `people_v13`'s module
   as v12's is kept.
3. **core v7:** the same for `core-v6.schema` with the `Removal` record
   and key tag appended; `codec/core.rs` translates it.
4. **Layouts and goldens:** regenerate `generated/layouts.rs` and the
   layout golden by their write modes.
5. **Tests:** `jig-protocol-store`'s codec tests
   (`src/tests/codec.rs`, `src/tests/codec/people_outcomes.rs`,
   `src/tests/codec/core_rows.rs`): a round trip of each new value at
   its smallest and fullest; `src/tests/format.rs`: a store at people
   13 or core 6 refuses with `OpenRefusal::Format { family: People |
   Core, .. }`; `jig-domain-core` `src/tests/restart_checks.rs` (or the
   nearest restart test): `a_removal_row_is_restored_into_the_core`.
6. Checks, once: `cargo fmt`; clippy on `jig-protocol-store`,
   `jig-domain-core`, `jig-store-codecs-world`; focused tests of
   `jig-protocol-store`, `jig-store-codecs-world`, `jig-store-world
   --test suite`, `jig-domain-core --lib`; `fuzzy` for the store codecs
   (`jig-store-codecs-world`'s fuzzy decode, the failing test alone);
   `heavy cargo check --tests` of `jig-test-store jig-test-service
   jig-core-world jig-conformance-world temper-engine-domain
   temper-service temper-engine-domain-world`. Commit, push.

## Report

fab's sections; `## Interfaces`: `Removal`, `RemovalStage`, the
`CoreRecord`/`CoreKey` variants, `Core.removals`, the schema versions;
the regenerated files; any temper test refreshed.

## Answers

2026-11-01. **May pd-store add `Removal`'s restore interval in
`intervals.rs` and the matching typed interval in `restart/cursor.rs`?**
Yes. pd-sweep has landed on the run's trunk (99bcdda9e2), so
`intervals.rs` is no longer contested; both files are now this task's
for this change. `RestoreRange::Core` becomes two intervals in key
order: the existing `[JIG, 0, 1]..[JIG, 0, 4]`, then
`Range::prefix(&[JIG, 0, 10])` for `Removal`. Archived core rows and
settings history stay out of the core walk. `restart/cursor.rs` gives
`Core` a count of 2, with `CoreKey::Removal` in interval 1 and every
other admitted core key in 0, the same shape `People` and `Tasks` have.
`RestoreRange::admits` already admits it through `CoreKey::live`. Add a
`Removal` key to `src/tests/intervals.rs`' `cursor_keys`, so the walk's
typed and encoded cursors are still checked to agree. In the report,
name both files as changed by this answer.
