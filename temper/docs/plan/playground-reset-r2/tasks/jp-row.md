+++
id = "jp-row"
part = "project-delete"
kind = "interface"
needs = ["jig-acts"]
files = [
  "jig/crates/jig-domain-core/src/removal.rs",
  "jig/crates/jig-domain-core/src/stored.rs",
  "jig/crates/jig-domain-core/src/restart/cursor.rs",
  "jig/crates/jig-domain-core/src/tests/removal_row.rs",
  "jig/crates/jig-protocol-store/schema/core-v6.schema",
  "jig/crates/jig-protocol-store/src/generated/core_v6.rs",
  "jig/crates/jig-protocol-store/src/generated/layouts.rs",
  "jig/crates/jig-protocol-store/src/codec/core.rs",
  "jig/crates/jig-protocol-store/src/codec/core/**",
  "jig/crates/jig-protocol-store/src/keys/core.rs",
  "jig/crates/jig-protocol-store/src/intervals.rs",
  "jig/crates/jig-protocol-store/golden/core/**",
  "jig/crates/jig-protocol-store/src/tests/fixtures/**",
  "jig/crates/jig-protocol-store/src/tests/codec/core_rows.rs",
  "jig/crates/jig-protocol-store/src/tests/keys.rs",
]
effort = "high"
minutes = 70
+++

## Goal

Add the project deletion's kept row (playground-reset.md, 5, "Kept until
whole"): a live core record `CoreRecord::Removal(ProjectRemoval)` under
`CoreKey::Removal(u32)`, one per project being deleted. It is encoded,
keyed and restored at the start's `LoadCore` into the core, which holds
it and answers whether a project is being removed. No decision makes
the row yet: `jp-close` saves it, and `jp-delete` moves it through its
stages and erases it.

Acceptance:

- The record encodes and decodes to itself for every stage
  (`jig-protocol-store`'s `core_rows_read_back_as_written`, its fixtures
  extended with the variant); its key sorts as its bytes do
  (`core_and_people_keys_sort_as_bytes_as_they_derive` with the new
  variant in `core_key`'s fixture); the drift test is clean after
  regenerating (`jig-store-codecs-world`'s `drift`).
- `RestoreRange::Core` admits the new key (it is live) and the restart
  cursor places it (`restore_cursor`'s tests, or a new case beside them).
- `Core::restore_core` of a `Removal` row answers `Restored::Live` and
  holds it: `core.removing(7)` is true after restoring `ProjectRemoval {
  project: 7, stage: RemovalStage::Ending }` and false for 8; a second
  row for the same project is `Restored::Rejected`
  (`jig-domain-core/src/tests/removal_row.rs`:
  `a_restored_removal_row_is_held`, `a_second_removal_row_for_a_project_is_rejected`).
- Every package of jig and temper builds; tests pass unchanged in meaning.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 1
  (the deletion goes on after its answer and ends whole across restarts).
- `temper/docs/design/draft/playground-reset.md`, section 5 (cite it at
  type level).
- `jig/docs/design/protocol/store.md`, 4.4 (jig's families), 5 and 5.3
  (schemas in place until the first release), 7.2 (the restart's ranges).
- `jig/docs/design/domain/engine.md`, 5.4 (the core's own rows).

## Owns and may change

Owns the files in the header. May change, as `notes/scope.md` says:
every exhaustive match on `CoreRecord`, `CoreKey`, `Record` or `Key`
the new variant forces, in jig and temper (the survey found:
`jig-protocol-store/src/{open.rs,commits.rs,historical.rs,transcripts/cut.rs,patching/phase/walking.rs,reads/point.rs}`,
`jig-domain-core/src/{reads/rows.rs,reads/vocabulary.rs,reads/settings.rs}`,
`jig/tests/client/src/joint/wire/cursor.rs` (it mirrors the key bytes),
`temper/crates/temper-engine-domain/src/restart.rs` (around 535),
`temper/crates/temper-engine-domain/src/archive.rs`), each arm doing for
`Removal` what it does for its nearest live kin, `RunProof` (a restored
live core row handed to `Core::restore_core`; a reader that never
serves core rows to parties refuses or skips it as it does `RunProof`).
`jig-domain-core/src/lib.rs`: the `mod removal;` line, the re-exports
below and the `Core` field below; the core's `Core::new` its initial
empty map. `jig-domain-core/src/tests.rs`: the `mod removal_row;` line.

Not yours: deciding anything with the row (`jp-close`, `jp-delete`); the
read scopes (`jp-scopes` owns `jig-protocol-store/src/reads/**` and
`intervals.rs`'s `Scope` arms, and keeps them; you add only the
`RestoreRange::Core` interval there).

## Steps

1. `jig-domain-core/src/removal.rs` (new; module doc citing
   `temper/docs/design/draft/playground-reset.md, 5`):

   ```rust
   /// A project's deletion, kept until its last stage (playground-reset.md, 5).
   #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
   pub struct ProjectRemoval {
       pub project: u32,
       pub stage: RemovalStage,
   }
   /// Its stages, in order.
   #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
   pub enum RemovalStage {
       /// Closed; its live tasks are being cancelled.
       Ending,
       /// Its ended tasks erased whole, scanning ended tasks after this number.
       Erasing { after: Option<u64> },
       /// Its notes erased.
       Notes,
       /// Its own rows erased, its connectors told; the row goes in that decision.
       Rows,
   }
   ```

   `Core` gains `pub(crate) removals: Map<u32, ProjectRemoval>` (bounded
   by `limits.authority.projects`, as `names` is) and
   `pub fn removing(&self, project: u32) -> bool` and
   `pub fn removal(&self, project: u32) -> Option<&ProjectRemoval>`.
   Re-export `ProjectRemoval` and `RemovalStage` from the crate root.

2. `stored.rs`: `CoreRecord::Removal(ProjectRemoval)` and
   `CoreKey::Removal(u32)`, the key placed after `Settings` in the enum
   so the derived `Ord` matches the bytes; `CoreRecord::key` maps it;
   `CoreKey::live()` is true for it. `restore_core`: insert into
   `removals` (`Restored::Live`); a project already held, or no room,
   `Restored::Rejected`.

3. Store: `keys/core.rs` writes and reads tag 10 then the project as a
   `u32` field as the codec writes `u32` fields; `schema/core-v6.schema`
   in place: `record ProjectRemoval { project: u32 stage: RemovalStage }`,
   an enum `RemovalStage` with its four arms (`erasing` carrying an
   optional `u64`), and a `removal` arm in `enum Value`; regenerate
   (`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-store-codecs-world
   --test drift`, which writes `generated/core_v6.rs`, the goldens and
   `generated/layouts.rs`); `codec/core.rs` both ways. If the format
   test pinning layout fingerprints
   (`current_layouts_are_the_eight_distinct_runtime_schema_fingerprints`)
   needs its expected fingerprint updated, update it: the change is in
   place (store.md, 5.3), no version moves.
4. `intervals.rs`: `RestoreRange::Core` becomes two intervals,
   `[JIG,0,1]..[JIG,0,4)` and the prefix `[JIG,0,10]`;
   `restart/cursor.rs`'s `place` gives `Core` two intervals, the second
   for `Removal`. `RestoreRange::admits` is unchanged if it already
   reads `live()`.
5. Tests: the acceptance's; fixtures: `tests/fixtures/keys.rs`'s
   `core_key` draws the new variant, `tests/fixtures/core_rows.rs` makes
   one per stage. Then, one at a time:
   - `heavy cargo nextest run -p jig-protocol-store`;
   - `heavy cargo nextest run -p jig-store-codecs-world -p jig-store-world`
     (use the package names of `jig/tests/store-codecs` and `jig/tests/store`);
   - `heavy cargo nextest run -p jig-domain-core --lib -E 'test(removal_row)'`,
     then `heavy cargo nextest run -p jig-domain-core`;
   - `heavy cargo check --tests -p temper-engine-domain -p temper-service`,
     then the focused tests of each temper package you edited, and
     `heavy cargo nextest run -p jig-core-world -p jig-client-world`;
   - `cargo fmt`; clippy over every package you edited; old forms from
     `jig/` (and `temper/` if you edited it).
6. Commit and push.

## Report

The standard report, plus: every forced arm outside the owned files;
the key's bytes as written; whether the layout fingerprint test moved.
