+++
id = "apps-connector"
part = "appsettings"
kind = "feature"
needs = ["apps-surface"]
files = ["jig/testing/jig-test-connector/src/settings.rs", "jig/testing/jig-test-connector/src/boundary.rs", "jig/testing/jig-test-connector/src/domain.rs", "jig/testing/jig-test-connector/src/lib.rs", "jig/testing/jig-test-connector/src/limits.rs", "jig/testing/jig-test-connector/src/tests.rs", "jig/testing/jig-test-connector/src/tests/settings.rs", "jig/tests/test-connector/src/world.rs", "jig/tests/test-connector/tests/suite/settings.rs", "jig/tests/test-connector/tests/suite/main.rs"]
effort = "high"
minutes = 110
+++

# apps-connector: jig's test connector owns a kind of the application's settings

## Goal

`jig-test-connector` owns one kind of the application's settings, in its own vocabulary
and its own records, as `jig/docs/design/domain/connectors.md`, section 17 has a connector
do: given its defaults at a start, it says which differ; it judges items (accept with size
and charters, same, refuse with words); it keeps a committed change's items, a proposal's
items, and drops them; it saves its defaults when told; it serves a value by kind, scope
and number. Its subtree world scripts the root above it. The root's routing between this
vocabulary and the core's asks is `apps-world`'s.

Acceptance:

- The connector's unit tests and its subtree world's new stories below pass; its other
  tests pass unchanged.
- No crate but `jig-test-connector` and its world changes.

## Inputs

- `jig/docs/design/domain/connectors.md`, section 17, and section 15 (**Its world**, the
  settings sentence of section 17).
- `jig/docs/design/domain/engine.md`, 17.5.
- `temper/docs/plan/reference-m3/notes/contracts.md`, C2 (the meanings the connector's
  vocabulary mirrors; the root translates).
- `temper/docs/plan/reference-m3/notes/common.md`.

## Owns and may change

- Owned: the `files` above (`settings.rs` new, its `mod` line).
- Neighbour: `jig/testing/jig-test-domain/src/**` where it matches the connector's `Event`
  or `Request` exhaustively: the compiler-forced arm alone,
  `unreachable!("settings routing: apps-world")`.
- Options, pre-approved:
  - The connector's records for values ride its existing `Record`/`RecordKey` (default:
    new arms `SettingDefault { kind, scope, number }`, `SettingChange { kind, scope,
    number }` with an `Option<Box<[u8]>>` value (none: removed), `SettingProposal {
    proposal, kind, scope, number }`); if the connector's record encoding is a schema with
    goldens, add the arms there in place and remake its goldens.

## Steps

1. `src/settings.rs` (module doc citing connectors.md, section 17): the connector's kind
   is declared in its `Config` as `settings: Option<SettingsKind { kind: u32, project:
   bool, value_bytes: u32, per_scope: u32, charters: bool, defaults: Box<[(u32, u32,
   Box<[u8]>)]> }>` (none: it owns no kind; every existing config gives none).
   Its rules, readable from the bytes alone: a value whose first byte is `b'!'` is refused
   with the words `"refused: <rest>"`; a value equal to the setting's current one is
   `Same`; with `charters`, a value of at least 4 bytes names the charter its first 4
   bytes encode (big-endian `u32`); otherwise accepted with its length as size.
2. `boundary.rs`: `Event` (from the root) gains `SettingsDefaults` (the start's ask for
   its comparison), `JudgeSettings { token, revision, proposal, items }`, `KeepSettings {
   revision, proposal, items }`, `KeepProposalSettings { proposal, items }`,
   `DropProposalSettings { proposal }`, `SaveSettingsDefaults { revision }`,
   `ReadSetting { token, kind, scope, number }`; items as the crate's own
   `SettingItem { kind, scope, number, edit: SettingEdit { Set(Box<[u8]>), Remove, Reset } }`.
   `Request` (to the root) gains `SettingsJudged { token, judged: Box<[SettingJudged]> }`
   (`Accept { size, charters }`, `Same`, `Refuse { why }`), `SettingsDefaults { defaults:
   Box<[SettingDefault { kind, scope, number, size, charters, differs }]> }` and
   `SettingValue { token, value: Option<Box<[u8]>> }`.
3. `domain.rs`: handle each as connectors.md, section 17 says: a reset judged as its
   default (none: as a removal); a proposal's judge reads the kept value; keep writes the
   records in the event's decision; a stored value of a kind or scope it no longer owns,
   or past its `value_bytes`, refuses its restore naming it. The connector's `restore` has
   no refusal today: add `Request::RestoreRefused { kind, scope, number }` (default); the
   root's arm for it is `apps-world`'s (here `unreachable!`).
4. Tests: unit tests in `src/tests/settings.rs` (each rule; keep and drop; defaults
   compared: same, changed, added, gone); subtree world `jig/tests/test-connector/tests/
   suite/settings.rs` (the root scripted above, a restart between steps): a change judged
   accepted and kept, read back after a restart; one refused with its words; a proposal
   kept, accepted (judged from the kept value), and another withdrawn (dropped); a
   restart with a changed default reporting `differs`, saved on `SaveSettingsDefaults`.
5. Narrow tests: `heavy cargo nextest run -p jig-test-connector --lib`;
   `heavy cargo nextest run -p jig-test-connector-world --test suite -E 'test(/settings/)'`
   (check the count). `cargo fmt`; `heavy cargo clippy --workspace --all-targets`. Commit
   and push.

## Report

The records added; the vocabulary's final names.
