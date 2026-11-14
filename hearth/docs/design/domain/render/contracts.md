# Contracts between one-owner's changes

2026-11-14. The shapes the changes of `temper/docs/design/draft/one-owner.md`'s pilot share while they run (`temper/docs/design/draft/design-economy.md`, 4.3). They bind as a delivered design section does. The goal's last change deletes this document and its status; the lasting design is `README.md`, 5.1, `hearth/docs/design/README.md`, 4.2, and `hearth/docs/design/client/README.md`, 3.2.

The changes, by the names their specs use:

- **the vocabulary:** the new crate and render's re-exports (section 1);
- **the parts pins** and **the errors pins:** printed-form tests in `hearth-client-protocol-records` (section 4);
- **the parts printer:** the records crate prints render's spec, parts and changes; the builder's copies of them go (sections 2 and 3);
- **the errors printer:** the records crate prints render's errors; the builder's copies of them go (sections 2 and 3);
- **the machines:** `hearth-protocol-config`, `hearth-protocol-git` and `hearth-protocol-mmdebstrap` name the vocabulary's values; the builder's copies of them go (section 3).

## 1. The vocabulary crate

- **`hearth-domain-render-vocabulary`,** at `hearth/crates/hearth-domain-render-vocabulary`: `no_std` step code on the `step` lint profile, as render is, depending on `skein-lib` and `hearth-lib` alone. The root `Cargo.toml` lists it as a member and a workspace dependency; render depends on it.
- **One module tree:** each item keeps the module path it has in render, so `hearth_domain_render_vocabulary::X::Y` is `hearth_domain_render::X::Y` for every item below. Each of render's modules that held one re-exports it from the vocabulary, so every public path render had still names it, alternate re-exports included (`parts::Meta`, `parts::software::Script`, `parts::config`'s items from its private children, `parts::packages`' from `change`, `diff`'s from `resolution`, `errors::{Class, Syntax, Reserved, Rule}`, `evaluate::{Value, Entry, Filter}`), and `hearth_domain_builder::render` with them.
- **The items,** by their path in render, with their derives:
  - `parts`: `Spec`, `Part`, `PartRecord`; `parts::config`: `Config`, `TreeEntry`, `Node`, `Group`, `GroupKind`, `User`, `UserKind`, `Member`, `Preset`, `Action`, `Tmpfile`, `Line`, `LineKind`, `Script`, `Change`, `EntryChange`, `Sizes`, `GroupKeyed`, `UserKeyed`, `MemberChange`, `PresetKeyed`, `LineKeyed`, `ScriptsChange`; `parts::boot`: `Boot`, `Change`; `parts::data`: `Data`, `DataEntry`, `DataValue`, `DataClass`, `DataChange`, `DataKeyed`; `parts::meta`: `Meta`, `Placement`, `Change`, `NamePair`, `MemberPair`, `PlacementPair`; `parts::live`: `Live`, `Covered`, `Consumer`, `Action`, `Change`, `ConsumersKeyed`; `parts::secrets`: `Secrets`, `SecretFile`, `Password`, `Identity`, `IdentityKind`, `Algorithm`, `Change`, `Named`; `parts::software`: `Software`, `Download`, `How`, `Fetch`, `Change`, `DownloadKeyed`; `parts::packages`: `Packages`, `Seeds`, `SeedUser`, `SeedGroup`, `Sources`, `Component`, `Variant`, `Apt`, `Preseed`, `PreseedKind`, `Initramfs`, `Microcode`, `Change`, `Names`, `SourcesPair`, `VariantPair`, `PreseedsPair`, `InitramfsPair`, `AptPair`, `SeedsChange`, `SeedIds`, `SeedUserKeyed`, `SeedGroupKeyed`; `parts::exports`: `Exports`, `Change`, `ExportChange`.
  - `diff`: `Change`, `TargetChange`, `Side`, `PartChange`, `Rebuild`, `Needs`, `Tally`, `Hunks`, `Line`, `LineKind`, `ResolutionChange`, `Listed`, `Moved`.
  - `errors`: every type `Error` reaches: `Error`, `Place`, `Kind`, `Decode`, `Field`, `Limit`, `Declared`, `Object`, `Expected`, `Required`, `Takes`, `Schema`, `Root`, `ExportFault`, `Exported`, `Cause`, `Guard`, `Format`, `TypeName`, `Repository`, `Tool`, `Command`, `Ended`, `Composition`, `Host`, `Collision`, `Subject`, `Id`, `Ids`, `Owns`, `Why`, `Resolution`, `StoreError`, `StoreFault`, `StoreLimit`, `Step`, `Reserved`, `Rule`; `errors::syntax`: `Syntax`, `Expected`.
  - The values those reach elsewhere in render: `keys::{Resolution, Keys}`, `revision::{Oid, Position, Runs, Firmware}`, `revision::declaration::{Map, Name, Fragment}`, `evaluate::FaultKind`, `evaluate::filter::Filter`, `evaluate::value::{Value, Entry}`, `exports::Exported`, `resolution::Inst`, `vocabulary::{Class, Outcome}`. `Fragment` isn't reached by the rest; it is here for the machines (section 3). The machines change moves `revision::Mode` in too, for the git machine's listing.
- **Their own operations move with them,** `pub`: `PartRecord::name`, `Component::{ALL, name}`, `Variant::name`, `PreseedKind::name`, `Microcode::name`, `DataClass::name`, `parts::live::Action::name`, `parts::config::{Action::name, LineKind::name, GroupKind::encode}`, `Name::{takes, key}`, `Fragment::name`, `Filter::arity`, `keys::Resolution::key`, `Outcome::worst`.
- **What stays in render:** its state and decisions. `EntryChange::pair` becomes a free function of `parts::config`, taking the change as its first argument. `errors::{Collector, Order, push}`, `exports::value_items`, `diff::{Made, Layer}`, `limits` (`Limits`, `SHIPPED`, `FOUR_MADE`, `valid`), and every module's other items stay. The vocabulary names nothing of render's.

## 2. The records crate's shapes

`hearth-client-protocol-records` depends on the vocabulary, not on render, and holds no copy of an item of section 1. Its printers of render's values are free functions in its own modules (the vocabulary takes no JSON dependency); the forms they print are unchanged byte for byte.

- **`Record::Spec(Box<parts::Spec>)`, `Record::Change(diff::Change)`,** and `Terminal { outcome: Outcome, diff: Option<diff::Tally>, left_out: u32 }`, with `parts` and `diff` the vocabulary's: the parts printer's.
- **`Record::Error(errors::Error)`,** the vocabulary's `Error`, and the store's full answer as a record of the records crate's own, since `hearth-domain-store`'s limit has `layers` and render's hasn't (`hearth/docs/design/client/README.md`, 3.2): the errors printer's. It prints as the store's full error prints now.
- **`hearth_client_protocol_records::diff::side(side: Side) -> &'static [u8]`,** with `Side` the vocabulary's: `from`, `to`, `both`, replacing `Side::name`. The parts printer writes it. The errors printer calls it, and, if the parts printer hasn't landed, writes it itself, at that path, with that signature and body, so that the merge keeps one.
- **Kept, the records crate's own:** `Record` and `Type`, `Terminal`, `outcome::Outcome` (render's outcomes and `Unavailable`), the store's limit with `Layers`, `startup`, `progress`'s notices, `build`, `form`, `printer`, `limits` and `CLIENT_VERSION`.

## 3. The builder's translations

Each change removes the copies it owns, in `hearth/crates/hearth-protocol-builder/src/translate/` and `src/translate.rs`, and the tests and `_back` helpers in `src/tests/` that only invert them, and leaves the others' alone.

| Owner | The copies it removes |
|---|---|
| the parts printer | `translate/parts.rs` and `translate/parts/` whole; `translate/changes.rs` and `translate/changes/` whole; `translate/answers.rs`' `spec`, `metadata`, `optional_firmware`, `runs`, `boot`, `resolved` |
| the errors printer | `translate/errors/` whole; `translate/errors.rs`' translations from render's errors to the records' |
| the machines | `translate/hunks.rs` whole; `translate/listing.rs`' `mode_to_render`; `translate/documents.rs`' `runs`, `boot`, `optional_firmware`; `translate/declarations.rs`' `name` and the fragment's copy in `scripts`; `translate/errors.rs`' `config_field`, `declared`, `object`, `config_expected`, `required`, `config_takes`, `declaration_map`; `translate/resolution.rs`' copy of `Component` |

What stays, with its tests: `translate/answers.rs`' `outcome`, `layer`, `kind`, `failed` and `full` (the client's envelopes and build records; `full` builds the errors printer's store record); the rest of `translate/documents.rs` and `translate/declarations.rs` (configuration's positions keep the byte offset its decoder orders refusals by, and its declarations are compiled); `translate/errors.rs`' `fault` (it refuses stream faults and supplies limits) and the template's `syntax` and `syntax_expected`; `translate/instructions.rs`, `translate/operations.rs`, `translate/listing.rs`' `entry` (it prints the object id as hex), `translate/store.rs`, and `translate/resolution.rs` but for `Component`.

A projection a copy carried moves into the printer, where it prints: typed source names printed as their keys (`Schema::Source`), a filter printed by its name with its expected arity (`Syntax::Arity`), an export map's absent entries left out. A builder test that checks one becomes a printed-form test in the records crate, building render's values, its expectation unchanged.

## 4. Printed-form pins

A pin is a test in `hearth-client-protocol-records`' `src/tests/` that prints one record, or one variant of one, through the printer, as text and as a JSON line, and compares the bytes with literal expectations. The pins changes write them against the records crate's current types; the printers' changes rebuild their fixtures from the vocabulary's values and leave every expectation's bytes as they are.

A pin whose fixture only a copy's looser type could hold, a value render never makes (a free string where render holds a typed name, an arity its filter doesn't take), takes the nearest value render does make that prints the same form: the same wording, plural or prefix. Only the bytes that value prints move (the name, the number); the rest of the expectation stays as it was, and the change's report lists each such row.
