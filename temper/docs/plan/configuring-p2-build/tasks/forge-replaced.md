+++
id = "forge-replaced"
part = "restore"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/settings.rs", "temper/crates/temper-engine-domain-forge/src/kinds.rs", "temper/crates/temper-engine-domain-forge/src/tests/kinds.rs", "temper/crates/temper-engine-domain/src/route/kinds.rs", "temper/crates/temper-protocol-store/**"]
effort = "high"
minutes = 90
+++

# forge-replaced: the forge keeps what its kinds' changes replaced

## Goal

temper's forge, which owns temper's kinds of settings, keeps, beside each change it saves on
jig's `KeepSettings`, the value the setting held before, by the revision the change made
(contract B3; jig's `domain/connectors.md`, 17, "What its items replaced"). Nothing reads or
erases those values yet: `restore-forge` answers a restore from them and erases them on
`ForgetReplaced`.

Acceptance:

- `SettingSlot` gains `Replaced(u64)`, the revision; temper's store `SettingSlot` gains
  `replaced: Number` (store-v11, in place).
- `Forge::keep_settings` takes the revision; for each item whose setting held a change with
  a value (`setting_rows` holds `Edit::Set(..)` at its `Change` slot) before the item applies,
  it saves a `SettingRow` at `SettingSlot::Replaced(revision)` with that `Edit::Set(..)`; a
  setting that followed its default, was removed or had no change keeps none. A release's
  `save_settings_defaults` keeps none.
- A restart restores the new rows as it restores the forge's other setting rows, and they
  change nothing the forge reads (its held rules, its charters).
- The unit tests below pass with the crates' focused tests.

## Inputs

- jig's `jig/docs/design/domain/connectors.md`, 17; `jig/docs/design/domain/engine.md`, 17.5
  ("History") and 17.10.
- temper's `temper/docs/design/protocol/configuration.md`, 13.2 ("Kept by the forge").
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B3; `notes/common.md`.

## Owns and may change

- Owned: `temper-engine-domain-forge/src/settings.rs` (`SettingSlot`); `src/kinds.rs`
  (`keep_settings`' revision and the replaced rows; `rebuild_settings` and every reader of
  `setting_rows` skipping the `Replaced` slot as it skips `Proposal`); `src/tests/kinds.rs`;
  `temper-engine-domain/src/route/kinds.rs` (the `KeepSettings` arm passes `revision`);
  `temper-protocol-store/**` (`schema/store-v11.schema`'s `SettingSlot`, the generated code,
  `src/keys.rs`' slot tags, `src/records/settings.rs`, its tests and goldens).
- Neighbour files, the new slot's arm only: matches on `SettingSlot` in
  `temper-engine-domain-forge/src/tests/restore_fit.rs`, `src/tests/records.rs`,
  `temper-service/src/tests/family.rs`, `temper-engine-domain/src/tests/kinds.rs`, and where
  the forge's records are restored or fitted at a start (`restore.rs`, its fit checks): a
  `Replaced(_)` row is restored and fitted as a `Change` row's value is.
- Options, pre-approved:
  - If the forge's worst case of rows (its limits or a fit check) bounds its setting rows,
    price the replaced rows as one whole set of its kinds' values per revision the history
    may keep (jig's `limits.settings.history`, as the root's limits give it); say where.
  - If `temper-protocol-store`'s format must move for the slot in place (a test refuses an
    in-place change), stop and ask; M3's `kinds-store` changed it in place.
  - If another plan's change to `route/kinds.rs` or `kinds.rs` reaches the trunk first and
    conflicts, keep both sides' intent.

## Steps

1. `settings.rs`: `SettingSlot { Default, Change, Proposal(u64), Replaced(u64) }` (sorts
   last).
2. `kinds.rs`: `pub fn keep_settings(&mut self, env, revision: u64, proposal, items, out)`:
   for each item, read the `Change` slot's edit before saving or erasing it; when it is
   `Edit::Set(value)`, `save_setting(SettingKey { slot: SettingSlot::Replaced(revision), ..key },
   Edit::Set(value.clone()), out)`. `rebuild_settings` and the readers of current values read
   `Change` and `Default` alone, as now.
3. `route/kinds.rs`: `connector::Ask::KeepSettings { revision, proposal, items } =>` passes
   `revision`.
4. `temper-protocol-store`: the slot's arm `replaced: Number`, its key tag after
   `proposal`'s, the codec, a golden of a row at the new slot.
5. Tests in `src/tests/kinds.rs`, with its helpers (`stocked`, `item`, `save`, `restore`):
   - `a_kept_change_keeps_the_rule_it_replaced_by_its_revision`: kind 1's rule 1 set at
     revision 2 to `rule(true)`, then at revision 3 to `rule(false)`: the second keep saves a
     row at `Replaced(3)` holding `rule(true)`; the first saves none.
   - `a_change_over_the_stock_default_keeps_nothing_replaced`: kind 3's number 1 set over
     the stock: no `Replaced` row; a reset then a set: none.
   - `replaced_rows_are_restored_and_change_nothing_held`: the rows above restored into a
     fresh domain; the held rules and charters equal those of a domain restored without them.
6. Narrow tests: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/kinds/)'` (check the count); `heavy cargo nextest run -p temper-protocol-store`.
   Then `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain
   -p temper-protocol-store --all-targets -- -D warnings`; the focused tests of those three
   packages and `temper-service`. Commit and push.

## Report

The slot's encoding; the readers that skip it; any worst-case pricing changed.
