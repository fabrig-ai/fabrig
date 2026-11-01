+++
id = "fr-family"
part = "forge-records"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/limits.rs", "temper/crates/temper-engine-domain-forge/src/restore.rs", "temper/crates/temper-engine-domain-forge/src/reconcile.rs", "temper/crates/temper-engine-domain-forge/src/tests/restore_fit.rs", "temper/crates/temper-protocol-store/**", "temper/tests/codecs/tests/suite/fuzzy/store.rs"]
effort = "high"
minutes = 90
+++

## Goal

Add the forge connector's two new store families, `Forge` (31) and
`ForgeNext` (32), with the record's types and their codec, restored
into the connector but not yet used: the surface every other task of
parts `forge-records`, `remove-forge` and `playground` builds on. No
behaviour changes; nothing makes a record yet (`fr-domain` does).

Acceptance:

- `temper_engine_domain_forge` exports, from `boundary.rs`, exactly the
  shapes of `notes/forge-records.md`, section 1 (`ForgeRecord`,
  `Entries`, `FirstIntent`, `Template`, `NewForge`, `ForgeChoice`,
  `MakeRefusal`) and `notes/remove-forge.md`, section 1 (`Removal`), each
  `Clone, PartialEq, Eq, Debug` as their neighbours are, with
  `ForgeRecord::new(forge, template)` and `ForgeRecord::configured(&self)
  -> Configured`.
- `Stored::Forge(ForgeRecord)` and `Stored::ForgeNext(u16)` after
  `Receipt`; `Key::Forge(u16)` and `Key::ForgeNext` after
  `Key::Receipt`, so `Key`'s derived order puts them last as their bytes
  do. `stored_key` and `stored_bytes` (`lib.rs`) cover them.
- `Limits.forges: u32` (the kept forges at most; `valid` requires it
  nonzero and at most `temper-client-protocol-wire`'s 16, written as a
  local constant `FORGES = 16` with a comment naming the wire's), and the
  three `ENTRY_*_BYTES` constants of the note in `limits.rs`. Every
  `Limits` literal in temper gains `forges: 4` (compiler-forced).
- `Request::ForgeKept { record: ForgeRecord }` exists (doc comment from
  the note, section 2), never emitted yet.
- `Domain::configure_playground(&mut self, template: Option<Template>)`
  stores the template in a new field (read by `fr-domain`), beside
  `configure_demo`.
- Restore: `Event::Restore` of a `Forge` row keeps it in a new
  `records: Map<u16, ForgeRecord>` (capacity `Limits.forges`), a
  `ForgeNext` row in `next: Option<u16>`; the restore fit (`restore.rs`)
  admits at most `forges` records, one `ForgeNext`, a record whose
  entries fit the byte bounds; reconciliation (`reconcile.rs`, and the
  record-kind classification in `domain.rs`'s reconcile arms) treats both
  as linkless connector rows, as `DemoAct` is. Nothing else reads them.
- `temper-protocol-store`: schema records of the note, section 1 (and
  `Removal`), `Value::forge`, `Value::forge_next` appended; key codec
  `FORGE = 31`, `FORGE_NEXT = 32` in `keys.rs`; `records.rs` encode,
  decode and the key-agreement check; `largest` and `bounds` cover the
  new rows; generated `v10.rs` and goldens regenerated in place,
  `STORE_VERSION` stays 10.
- Tests (temper-protocol-store's library tests):
  `forge_keys_are_family_thirty_one_then_the_number_widened`,
  `forge_next_key_is_its_family_byte_alone`,
  `forge_records_round_trip_with_entries_first_and_removal`,
  `a_forge_row_whose_number_disagrees_with_its_key_is_refused`,
  `forge_entries_past_their_bounds_are_refused`; the generated-key
  property tests (`tests/fixtures.rs`) generate both families; the
  codecs world's fuzzy store inputs (`temper/tests/codecs/tests/suite/fuzzy/store.rs`)
  gain both. Forge domain library test
  `restored_forge_records_are_kept_and_fit_their_bound` in
  `src/tests/restore_fit.rs`.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 2),
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md` (sections
  1 and 2), `temper/docs/plan/playground-reset-r5/notes/remove-forge.md`
  (section 1).
- `temper/docs/design/protocol/store.md`, sections 2 to 4 (families,
  keys, the version kept in place).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s1b-connector-store-acts.md`,
  section 1, "Adding a family": every extension point, with `DemoAct`
  (family 19) as the worked example, and `Flight` (21) for a recent one.

## Owns and may change

- Owns: `temper-engine-domain-forge`'s `src/boundary.rs`, `src/lib.rs`,
  `src/limits.rs`, `src/restore.rs`, `src/reconcile.rs` and the
  restore/reconcile arms of `src/domain.rs` (the `restore` function and
  the record classification near its end; nothing else in `domain.rs`),
  its `src/tests/restore_fit.rs`; all of `temper-protocol-store`;
  `temper/tests/codecs/tests/suite/fuzzy/store.rs`.
- May change, compiler-forced only: every exhaustive match over `Stored`,
  `Key` or the forge `Request` and every `Limits` literal in temper
  (`rg 'Stored::DemoAct|Key::DemoAct|Request::SetUpGroup'` finds them:
  `temper-fake/src/check.rs` ignores the new rows as it ignores `DemoAct`;
  `temper-engine-domain/src/route/forge_route.rs` drops `ForgeKept` with
  a comment `// fr-root forwards it`; the engine worlds' key mappings in
  `temper/tests/engine/forge/src/world.rs` and
  `temper/tests/engine/protocol-forge/src/protocol.rs`).
- Options: if `skein-codegen` cannot express `option bytes 4096` inside a
  record as the note writes it, use the nearest form the schema has
  (`option` of a one-field record) and say so. If a bound check would
  need `Limits.forges` in `temper-service`'s limit names, add only the
  literal (`forges: 4`) there; `fr-machine` adds the name `forge.forges`.

## Steps

1. Domain types and families; `lib.rs` key and bytes; limits.
2. Restore and reconcile arms; the restore-fit test.
3. Schema, regenerate (the codecs world's drift test,
   `temper/tests/codecs/tests/suite/drift.rs`, names the generator and
   checks generated code and goldens); key and record codecs; tests.
4. Forced arms across temper (`heavy cargo check -p PKG --tests` for each
   while errors last).
5. Narrow: `heavy cargo nextest run -p temper-protocol-store --lib`;
   `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/restore_fit/)'`; `heavy cargo nextest run -p temper-codecs-world
   --test suite -E 'test(/store/) | test(/drift/)'`.
6. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-protocol-store -p temper-codecs-world --all-targets -- -D
   warnings`; focused tests of `temper-engine-domain-forge`,
   `temper-protocol-store`, `temper-engine-domain`; `heavy cargo check
   --tests` for `temper-service`, `temper-fake`, `temper-engine-forge-world`,
   `temper-engine-protocol-forge-world`, `temper-engine-domain-world`.
   Old-forms from `temper/`. Commit and push.

## Report

As `notes/common.md` says; every file outside the owned list you edited
for forced arms, and any schema form you had to change.

## Answers

### 2026-11-01: Supply temper/docs/plan/playground-reset-r5/notes/remove-forge.md section 1 (recommended), or specify the exact forge Removal Rust and schema shapes directly? (playground-reset-r5/fr-family-1, foreman playground-reset-r5/fr-family/foreman-1)

Use `Removal { pub group: bool, pub secrets: bool }` and schema `record Removal { group: bool secrets: bool }`.
The cited `temper/docs/plan/playground-reset-r5/notes/remove-forge.md`, section 1, exists in this checkout.
It specifies: “`removal: option Removal`.”
The task’s acceptance requires this exact shape with `Clone, PartialEq, Eq, Debug`.
The next run implements that definition and completes the task’s codec, round-trip coverage and remaining acceptance checks.

The requested report file could not be written because this run’s filesystem is read-only.
