# Goal K5's contracts

2026-11-21. The shapes goal K5's changes share and no other area reads
(`temper/docs/design/draft/design-economy.md`, 4.3). They bind as a
delivered design section does. The goal's last change deletes this
document and its status. The lasting design is [maps.md](maps.md).

## 1. The fixture

- **Its family** is `mapped`, in skein's codec world: its schema
  `skein/tests/codec/schema/mapped-v1.schema`, its maps file
  `skein/tests/codec/schema/mapped.maps`, its records
  `skein/tests/codec/src/generated/mapped.rs` and its translation
  module `skein/tests/codec/src/generated/maps/mapped.rs`, each with
  its row in `tools/merge/codecs.txt`.
- **The merge job's test** with the real generator builds its conflict
  from these two sources: one side edits the schema, the other the maps
  file, each regenerating the translation module.

## 2. jig's store

- **Maps files:** `jig/crates/jig-protocol-store/schema/<family>.maps`
  for core, tasks, people, settings, notes, accounts, component and
  openings. Registry and secrets are the owner's shape throughout and
  keep none ([maps.md](maps.md), 1).
- **Translation modules:**
  `jig/crates/jig-protocol-store-records/src/generated/<family>.rs`,
  the crate's `generated::<family>`, for every family above but
  openings, whose module is
  `jig/crates/jig-protocol-store/src/generated/openings.rs`, since its
  translation and domain types are that crate's.
- **By-hand modules:** each family's existing module of the records
  crate, `crate::codec::<family>`, re-exporting what it keeps elsewhere.
- **The records crate's own converters:** the first jig change's live
  in `jig_protocol_store_records::codec::convert`; each later family's
  in its own by-hand module, `crate::codec::<family>`, named for what it
  converts, so that changes running side by side never define a
  function twice in one module. Two families may each hold one for the
  same type.
- **Drift:** `jig/tests/codec-families.rs`'s `Family` row gains the
  family's maps text and its translation module's path, both absent for
  a family without a maps file; `check()` compares and writes the
  translation module as it does the records'. The first jig change adds
  them and fills its rows; each later one fills its own.
- **What siblings rely on:** every public item of the records crate
  that another module or crate names keeps its name and signature: the
  facade `codec::{limits, names, encode, decode, lowering}`, tasks'
  `codec::tasks::carried` pairs, people's `codec::people::{limits,
  derived_limits, encode, decode, encode_ask, decode_ask,
  encode_request_key, decode_request_key}` and core's public
  `encode_*`/`decode_*` helpers.

## 3. temper's store

- **Its maps file** is `temper/crates/temper-protocol-store/schema/store.maps`,
  its translation module
  `temper/crates/temper-protocol-store/src/generated/store.rs`, the
  crate's `generated::store`. The first temper change writes the header
  and the by-hand module; the second keeps them.
- **The split:** the first temper change maps the declarations
  translated by `src/records/{change, landing, landing_log, history,
  tiers, closer, follows}.rs` and `src/records/tiers/post.rs`, with the
  shared leaves they name (`Number`, `Commit`, `Repository`,
  `JobAttempt`, `Check`, `UpdateStyle`); the second, every other
  declaration of the crate's translation.
- **Drift:** `temper/tests/codecs/tests/suite/drift.rs` compares and
  writes the translation module; the first temper change adds it.
- **Its entries keep their signatures:** `records::{encode, decode,
  row}`, `keys`, `bounds`, `application` and `jobs`, a generated
  problem becoming `Undecodable::Malformed` where an entry returns one,
  as it does today.
