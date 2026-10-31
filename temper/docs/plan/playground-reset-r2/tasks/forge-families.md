+++
id = "forge-families"
part = "forge-records"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-protocol-store/**",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/limits.rs",
  "temper/crates/temper-engine-domain-forge/src/restore.rs",
  "temper/crates/temper-engine-domain-forge/src/reconcile.rs",
  "temper/crates/temper-engine-domain-forge/src/kept.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/kept.rs",
  "temper/tests/codecs/tests/suite/fuzzy/store.rs",
]
effort = "high"
minutes = 80
+++

## Goal

Add the forge connector's four new store families, `Forge`,
`ForgeNumbers`, `Playground` and `KeptAct`, with their types, keys,
codec and restore, so that the forge-records, forge-remove and
playground parts all build on one fixed record shape. No behavior: the
connector only holds and restores the rows, and offers accessors and
save requests for them. Today's `Demo` and `DemoAct` families stay as
they are.

Acceptance:

- Each new family encodes and decodes to itself, its key lies in the
  forge's key interval, and its golden bytes are committed
  (`temper-protocol-store`'s `every_family_encodes_and_decodes_to_itself`,
  `every_forge_key_lies_in_the_interval_and_no_core_key_does` and the
  sizes tests, extended to the new variants).
- A restored row of each family is held by the domain and read back by
  its accessor; a restore past the family's capacity is refused by the
  restore fit as other families' are (new unit tests in
  `temper-engine-domain-forge`, `src/tests/kept.rs`).
- The codec world's store goldens (`temper-codecs-world`) pass with the
  new variants.
- Every package that depends on `temper-engine-domain-forge` or
  `temper-protocol-store` builds, its tests unchanged in meaning.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, section 2
  (the types, bytes and rules this task builds, verbatim) and section 4
  (the two limits).
- `temper/docs/design/protocol/store.md`, sections 2, 3 and 4 (families,
  keys, records and the store version: new families go in place,
  `STORE_VERSION` unchanged until the first release unless the crate's
  own rule says otherwise, in which case follow the crate).
- `temper/docs/design/draft/playground-reset.md`, sections 3, 4 and 6.2,
  for what the rows mean (cite it at type level).

## Owns and may change

Owns:

- `temper/crates/temper-protocol-store/**` (schema
  `schema/store-v10.schema` in place, `src/generated/`, `golden/`,
  `src/records.rs`, `src/records/`, `src/keys.rs`, `src/lib.rs`, its
  tests).
- `temper/crates/temper-engine-domain-forge/src/boundary.rs`,
  `lib.rs` (`stored_key`, `stored_bytes` and the exhaustive lists near
  `lib.rs:411`), `limits.rs`, `restore.rs`, `reconcile.rs`
  (`RecordKind`, references), and a new module `src/kept.rs` with its
  tests `src/tests/kept.rs`.
- `temper/tests/codecs/tests/suite/fuzzy/store.rs` and the codec
  world's store goldens.

May change:

- `temper/crates/temper-engine-domain-forge/src/domain.rs`: only the
  new fields of `Domain` (the held rows, below), their construction in
  the constructor near `domain.rs:534`, and the restore arms near
  `domain.rs:4861`. Nothing else of `domain.rs`: the forge-records part
  owns it next.
- Every literal of `temper_engine_domain_forge::Limits` in any crate or
  test world (rg `Limits {` naming that type): the two new fields only,
  each set to its default (below).
- Exhaustive matches on `temper_engine_domain_forge::Stored` or `Key`
  outside the owned files (rg `Stored::Demo` and `Key::Demo` for the
  sites; survey: `temper-engine-domain/src/store.rs`,
  `temper-service/src/store.rs`, test fixtures): an arm for each new
  variant that does what the arm does for its nearest kin (`Demo` for
  `Playground` and `ForgeNumbers`, `DemoAct` for `KeptAct`, `Writer`
  for `Forge`). Say each in the report.
- If `temper-service`'s limit names (`src/limits/names.rs`) must name
  the two new limits for the build to pass, add them there as
  `forge.forges` and `forge.definition_bytes` with their defaults, and
  nothing else of the service.

## Steps

1. **Types,** in `boundary.rs`, as `notes/contracts.md` section 2 has
   them, with the derives their kin carry (`Clone`, `PartialEq`, `Eq`,
   `Debug`; `Copy` where every field is): `ForgeRecord`, `FirstIntent`,
   `Playground`, `PlaygroundState`, `ResetStep`, `KeptAct`,
   `KeptActKind`. Variants:

   ```rust
   // Stored, after Flight and the landing rows, in key-byte order
   Forge(ForgeRecord),            // 31
   ForgeNumbers { next: u16 },    // 32
   Playground(Playground),        // 33
   KeptAct(KeptAct),              // 34
   // Key
   Forge(u16),
   ForgeNumbers,
   Playground,
   KeptAct([u8; 16]),
   ```

   `stored_key` gives `Key::Forge(row.number)`, `Key::ForgeNumbers`,
   `Key::Playground`, `Key::KeptAct(row.key)`. `stored_bytes` charges
   the definition's and intent's bytes as other boxed bytes are.

2. **Limits,** in `limits.rs`: `pub forges: u32` (default 4, the
   forge component's `forges`) and `pub definition_bytes: u32` (default
   8192), checked by the existing limit validation: `forges` at least 1
   and at most `u16::MAX`; `definition_bytes` at least 256. A
   `ForgeRecord` whose `definition` is longer than `definition_bytes`,
   or whose `first`'s names exceed `FIRST_NAME_BYTES`, is refused by the
   restore fit and by `kept::save_forge` (below).

3. **Held rows and accessors,** in `src/kept.rs` (`pub(crate)` module,
   its public items re-exported from `lib.rs`):

   ```rust
   // Domain fields (domain.rs constructor: capacities from Limits)
   pub(crate) forges: Map<u16, ForgeRecord>,          // forges
   pub(crate) forge_next: Option<u16>,                // ForgeNumbers.next
   pub(crate) playground: Option<Playground>,
   pub(crate) kept_acts: Map<[u8; 16], KeptAct>,      // capacity: tasks + 2 * forges

   impl Domain {
       pub fn forge_record(&self, forge: u16) -> Option<&ForgeRecord>;
       pub fn forge_records(&self) -> impl Iterator<Item = &ForgeRecord>; // number order
       pub fn forge_next(&self) -> u16;                   // 1 when none
       pub fn playground(&self) -> Option<&Playground>;
       pub fn kept_acts(&self) -> impl Iterator<Item = &KeptAct>;      // key order
   }
   // Request makers, pushing Request::Save / Request::Erase as Demo's rows do
   pub(crate) fn save_forge(d: &mut Domain, row: ForgeRecord, out: &mut Queue<Request>) -> Result<(), Full>;
   pub(crate) fn erase_forge(d: &mut Domain, forge: u16, out: &mut Queue<Request>);
   pub(crate) fn give_number(d: &mut Domain, out: &mut Queue<Request>) -> Option<u16>; // saves next + 1; None past u16::MAX
   pub(crate) fn save_playground(d: &mut Domain, row: Playground, out: &mut Queue<Request>);
   pub(crate) fn erase_playground(d: &mut Domain, out: &mut Queue<Request>);
   pub(crate) fn keep_act(d: &mut Domain, row: KeptAct, out: &mut Queue<Request>) -> Result<(), Full>;
   pub(crate) fn erase_act(d: &mut Domain, key: [u8; 16], out: &mut Queue<Request>);
   ```

   Use the crate's own map, queue and full types (whatever `demo_acts`
   and its save use); the names above are binding, their exact
   container types are yours. If an iterator return breaks the step
   code subset, return a boxed slice instead and say so. The
   `#[expect(dead_code, reason = "...")]` for request makers no caller
   uses yet is allowed, scoped to each item, with the reason naming the
   part that calls it (forge-records, forge-remove, playground).

4. **Restore,** in `domain.rs`'s restore arms and `restore.rs`'s fit:
   each new row inserted into its map or field; `ForgeNumbers` keeps the
   larger of the stored `next` and one past the largest restored
   forge's number (a restore of forge 5 with `next` 3 holds `next` 6).
   `reconcile.rs`: a `RecordKind` for each family; `Forge`,
   `ForgeNumbers` and `Playground` need nothing; a `KeptAct` needs its
   forge's record (a kept act whose forge has no record is reported as
   other dangling rows are).

5. **Codec,** in `temper-protocol-store`: the four values and keys in
   the schema, in place, regenerated with the crate's generator (read
   the crate's `lib.rs` and generated headers for the command; skein's
   `skein-codegen`), goldens for each new variant, key bytes 31-34 in
   `keys.rs`, and the size bounds (`src/lib.rs`'s application sizes)
   from the two limits. If the crate's tests require a version bump for
   a schema change, follow them and say so.

6. **Tests:** extend `fixtures.rs`'s `key()` and the records fixtures
   to draw the new variants; `src/tests/kept.rs` in the forge crate:
   `restored_forge_rows_are_held_and_numbers_follow_the_largest`,
   `restored_playground_and_kept_acts_are_read_back`,
   `a_definition_past_its_bound_is_refused`,
   `a_kept_act_without_its_forge_is_dangling`. Then:
   - `heavy cargo nextest run -p temper-protocol-store`;
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
   - `heavy cargo nextest run -p temper-codecs-world`;
   - `heavy cargo check --tests` on every direct dependent: `-p
     temper-engine-domain -p temper-service -p temper-engine-forge-world
     -p temper-engine-domain-world -p temper` (one command at a time);
   - `cargo fmt`; `heavy cargo clippy -p temper-protocol-store -p
     temper-engine-domain-forge --all-targets -- -D warnings` (and every
     other package you edited);
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
7. Commit (types and codec, then restore, then tests) and push.

## Report

The standard report, plus: each match arm or literal you edited outside
the owned files; the generator command you ran; whether the store
version moved; the container types behind the accessors.
