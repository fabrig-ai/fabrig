+++
id = "kinds-store"
part = "kinds-forge"
kind = "migration"
needs = ["kinds-values"]
files = ["temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/restore.rs", "temper/crates/temper-engine-domain-forge/src/reconcile.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/tests/restore_fit.rs", "temper/crates/temper-engine-domain-forge/src/tests/records.rs", "temper/crates/temper-protocol-store/**", "temper/crates/temper-fake/src/check.rs", "temper/tests/engine/protocol-forge/src/protocol.rs", "temper/tests/engine/forge/src/world.rs", "temper/tests/codecs/**", "temper/docs/design/protocol/store.status.md"]
effort = "high"
minutes = 150
+++

# kinds-store: the forge's family `Setting`, byte 20

## Goal

temper's store keeps the forge's family `Setting` at byte 20 (a format change, in place:
`STORE_VERSION` stays 11, `temper/docs/design/protocol/store.md`, 4): the stock default the
binary last gave for each setting of kinds 1 to 3, its current change, and each pending
proposal's item. The forge saves, erases and restores its rows; what they mean (the held rules,
the charters, the judging) is `kinds-held`'s. Nothing writes a row yet.

Acceptance:

- In `temper_engine_domain_forge`'s `boundary.rs`: `Stored::Setting(settings::SettingRow)` and
  `Key::Setting(settings::SettingKey)`, each placed between `Binding` and `Flight` (the
  variants' order is the families' bytes' order, `protocol/store.md`, 2); a doc line citing
  `temper/docs/design/protocol/store.md`, 2 and `domain/forge.md`, 8.9.
- `lib.rs`: `stored_key` gives `Key::Setting(row.key)`; `stored_bytes` counts the row (a rule's
  segments, gates and approvals as the module's other byte counts do).
- `settings.rs` gains `pub fn rows(l: &Limits) -> Option<u32>`: the rows the family holds at
  most, checked arithmetic: `(1 + judge_projects) * landing_rules * 2` (kinds 1 and 2, a default
  and a change each) `+ 4` (kind 3's two numbers, a default and a change each) `+
  setting_proposals * ((1 + judge_projects) * landing_rules + 2)` (each pending proposal names
  each setting at most once). `Domain` gains `setting_rows: Map<settings::SettingKey,
  settings::Edit>`, sized by `rows` in construction (a construction that cannot size it is
  refused as the others are), filled by the restore (`Stored::Setting` inserts its row) and by
  nothing else yet; `restored_link_key`, or the file's equivalent, gains its arm.
- `restore.rs`: `Setting` rows fit `setting_rows`' room (`RecordKind::Setting` in
  `reconcile.rs`'s `RecordKind`, placed between `Binding` and `Flight`); a rule's gates past
  `change_policy.gates`, approvals past `roles`, or a pattern past `authority_segments`
  segments are a restore fit failure as other fields' are. `src/tests/restore_fit.rs`'s table
  gains the family.
- `temper-protocol-store`: the schema's records, a `setting: SettingRow` variant appended last
  to `Value` (so every other value's bytes stay), `keys.rs`' `SETTING: u8 = 20` with its
  encoder and decoder (the kind, the scope, the number as eight bytes each, then the slot's
  tag, 0 default, 1 change, 2 proposal and its number), `records/settings.rs` translating both
  ways, exhaustive, and `records::limits` taking the rule's bounds from the forge's `Limits`.
  Sketch of the schema, names the implementer's where the file has better ones (reuse
  `Freshness` and `Segment`):

  ```
  enum PatternLast { exact: Segment, open: Segment }
  record RulePattern { segments: bytes N pieces Segment last: PatternLast }
  record RuleGate { number: u32 blocking: bool freshness: Freshness }
  record RuleApproval { role: u32 people: u32 freshness: Freshness }
  record LandingRule { pattern: RulePattern enforce: bool ci: bool up_to_date: bool gates: list N RuleGate approvals: list N RuleApproval }
  enum SettingValue { rule: LandingRule, charter: U32 }
  record SizedValue { value: SettingValue size: u32 }
  enum SettingEdit { set: SizedValue, remove, reset }
  enum SettingSlot { default, change, proposal: Number }
  record SettingRow { kind: u32 scope: u32 number: u32 slot: SettingSlot edit: SettingEdit }
  ```

  The decoder refuses a kind other than 1 to 3, an unknown tag, a number past its width and
  trailing bytes (`protocol/store.md`, 3); the routing's key-and-record check holds for the
  family as for every other. Generated code and `golden/v11` regenerated through the drift
  test; key order still sorts as `Key`'s variants (`src/tests/keys.rs`, generated keys of the
  new family among them).
- Every exhaustive match outside the forge crate gains its arm only:
  `temper/crates/temper-fake/src/check.rs`, `temper/tests/engine/protocol-forge/src/protocol.rs`,
  `temper/tests/engine/forge/src/world.rs`, `temper/tests/codecs/tests/suite/fuzzy/store.rs`
  (its row generator makes rows of the family: each slot, each edit, both values).
- `temper/docs/design/protocol/store.status.md`: remove `family 20 \`Setting\` (the schema
  reserves its byte);` from its **Not yet**, nothing else.
- Packages pass: `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
  `-p temper-protocol-store --lib`; `-p temper-codecs-world --test suite`;
  `heavy cargo check -p temper-engine-domain -p temper-engine-forge-world
  -p temper-engine-protocol-forge-world -p temper-engine-domain-world -p temper-fake -p temper
  --tests`.

## Inputs

- `temper/docs/design/protocol/store.md`, section 2 (family 20, as this cut wrote it: the slot
  `Proposal(proposal)` and the row's edit), sections 3 and 4 (keys; records; in place until the
  first release); `temper/docs/design/domain/forge.md`, 8.9 and section 16 (what the forge
  keeps, and that it loads them at a start).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C2 (the proposal's
  number as the core names it) and C3.
- How a family was added in place, by a survey: plan reference-m8's
  `temper/docs/plan/reference-m8/tasks/tiers-surface.md` (families 28 and 29), its **Inputs**
  bullet on the steps and its regeneration command:
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`.

## Owns and may change

- Owns the files and packages in the header. In the forge crate: the variants, the arms, the
  field and its restore above; `settings.rs` gains `rows` alone (kinds-values' types are not
  changed; if one must change for the codec, say why in the report).
- The root (`temper-engine-domain`) gains no arm unless the compiler names one; if it does, the
  arm only, saying which.
- Not this task's: the held rules and charters, the judging and keeping methods, the defaults'
  comparison (`kinds-held`); the root's restart order (`kinds-start`).

Pre-approved options:

- If a store test pins the largest key or row (`src/tests/sizes.rs`) and the family moves it,
  update the figure to the new measure and give both. If it passes the application's ceiling
  (`largest_forge_record_fits_the_application_ceiling`), ask before shrinking any bound.
- If a figure under `temper/crates/temper/tests/figures/` moves, regenerate it as its test says.

## Steps

1. The forge's variants, `rows`, the field, the arms and the restore fit.
2. The store's schema, keys and records; regenerate; the store's tests: a round trip of each
   slot and edit and both values (`src/tests/records.rs` or a new `src/tests/settings.rs`), a
   key past its width and an unknown slot tag refused (`src/tests/keys.rs`), the family in
   the key-order property.
3. The arms outside the crate; the codecs world.
4. fmt, old forms, clippy over every package changed; the tests in Goal, each count reported.
5. Commit ("Keep the forge's settings of temper's kinds in their own family") and push.

## Report

The implementer's text's report, plus: the sizes that moved; the regenerated goldens (count);
the schema's record names as written.
