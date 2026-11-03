+++
id = "connector-replaced"
part = "restore"
kind = "feature"
needs = []
files = ["jig/testing/jig-test-connector/src/settings.rs", "jig/testing/jig-test-connector/src/boundary.rs", "jig/testing/jig-test-connector/src/domain.rs", "jig/testing/jig-test-connector/src/limits.rs", "jig/testing/jig-test-connector/src/tests/settings.rs", "jig/testing/jig-test-store/**"]
effort = "high"
minutes = 75
+++

# connector-replaced: jig's test connector keeps what its settings' changes replaced

## Goal

`jig-test-connector`, which owns the test application's kinds of settings, keeps, beside each
change it saves on `KeepSettings`, the value the setting held before, by the revision the
change made (contract B3; `jig/docs/design/domain/connectors.md`, 17, "What its items
replaced"). Nothing reads or erases those values yet: `restore-connector` answers a restore
from them and erases them on `ForgetReplaced`.

Acceptance:

- On `Event::KeepSettings { revision, proposal, items }`, for each item whose setting held a
  change with a value before it (`changes` holds `Some(bytes)` for it), the connector saves a
  record `SettingReplaced { kind, scope, number, revision, value }` and keeps it in its state;
  a setting that followed its default, was removed, or had no change keeps none. A release's
  `SaveSettingsDefaults` keeps none.
- A restart restores them (the records restored as the connector's other records are).
- The unit tests below pass with the crate's and `jig-test-store`'s focused tests.

## Inputs

- `jig/docs/design/domain/connectors.md`, 17; `jig/docs/design/domain/engine.md`, 17.5
  ("History"), 17.10 ("A connector keeps what its items replaced").
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B3; `notes/common.md`.

## Owns and may change

- Owned: `jig/testing/jig-test-connector/src/settings.rs` (`State` gains `replaced:
  BTreeMap<(Key, u64), Box<[u8]>>`, the key the setting's and the revision; `keep` takes the
  revision); `src/boundary.rs` (`Record::SettingReplaced { kind: u32, scope: u32, number: u32,
  revision: u64, value: Box<[u8]> }`, `RecordKey::SettingReplaced { kind, scope, number,
  revision }`); `src/domain.rs` (the `Event::KeepSettings` arm passing `revision`, the
  restore of the new record); `src/limits.rs` if the connector's worst case prices its
  records; `src/tests/settings.rs`; `jig/testing/jig-test-store/**` (its schema
  `schema/test-connector-v2.schema` gains `record RecordSettingReplaced { kind: u32 scope: u32
  number: u32 revision: u64 value: bytes 65536 }` and the `Value` arm `setting_replaced`, in
  place; the generated code, `src/keys.rs`, `src/records.rs`, its fixtures and goldens).
- Neighbour files, the new record's arms only: matches on the test connector's `Record` or
  `RecordKey` in `jig-test-connector/src/client_documents.rs`, `jig/testing/jig-test-domain`,
  `jig/tests/test-connector/src/world.rs`, `jig/tests/core/src/conformance/naming.rs` (a
  record of the connector's own, named as `SettingChange` is).
- Options, pre-approved:
  - If the test connector's worst case of records (its limits) is checked by a test, price
    the replaced values as one whole set of its kinds' values per revision the history may
    keep (`limits.settings.history` as the core's `Limits` give it, or the connector's own
    limit beside its kinds' bounds if the core's is not reachable): say which.
  - If `jig-test-store`'s schema version must move for a new record in place (a test refuses
    an in-place change), stop and ask; M3 changed it in place (`setting_change`).
  - If `revision` arrives as `u64` and the store key wants `u32`, keep `u64` (B3).

## Steps

1. `settings.rs`: `pub(crate) fn keep(domain, revision: u64, proposal, items, out)`; for each
   item, before its change is saved or erased, if `domain.settings.changes.get(&key)` is
   `Some(Some(bytes))`, insert `((key, revision), bytes.clone())` into `replaced` and push
   `Request::Save { record: Record::SettingReplaced { .. } }`.
2. `domain.rs`: `Event::KeepSettings { revision, proposal, items } => crate::settings::keep(
   domain, revision, proposal, &items, out)`; the restore of `Record::SettingReplaced` into
   `replaced`.
3. `jig-test-store`: the record, key and codec, in place, with a golden of the new record.
4. Tests in `src/tests/settings.rs`, beside `project_release_erases_settings_and_proposals_in_that_scope_only`:
   - `a_kept_change_keeps_the_value_it_replaced_by_its_revision`: set number 1 to `b"a"` at
     revision 2, then to `b"b"` at revision 3: the requests save `SettingReplaced { .., revision:
     3, value: b"a" }` and nothing replaced at revision 2.
   - `a_change_over_a_default_or_a_removal_keeps_nothing_replaced`: a set over the default,
     a removal, then a set over the removal: no `SettingReplaced` saved.
   - `replaced_values_are_restored_at_a_restart`: the records saved above, restored into a
     fresh connector, give the same `replaced` state.
5. Narrow tests: `heavy cargo nextest run -p jig-test-connector --lib -E 'test(/settings/)'`
   (check the count); `heavy cargo nextest run -p jig-test-store`. Then `cargo fmt`; `heavy
   cargo clippy -p jig-test-connector -p jig-test-store --all-targets -- -D warnings`; the
   focused tests of `jig-test-connector`, `jig-test-store`, `jig-test-domain` and
   `jig/tests/test-connector`'s package. Commit and push.

## Report

The record's encoding and key; how the restart restores it; any worst-case pricing changed.
