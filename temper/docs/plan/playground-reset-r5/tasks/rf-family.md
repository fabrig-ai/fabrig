+++
id = "rf-family"
part = "remove-forge"
kind = "interface"
needs = ["fr-demo"]
files = ["temper/crates/temper-engine-domain-forge/src/**", "temper/crates/temper-protocol-store/**", "temper/crates/temper-engine-domain/src/route/demo.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/restart.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/tests/codecs/tests/suite/fuzzy/store.rs", "temper/tests/engine/domain/tests/suite/demo.rs"]
effort = "medium"
minutes = 90
+++

## Goal

The removal's surface (draft `temper/docs/design/draft/playground-reset.md`,
section 4; `notes/remove-forge.md`): store family 19 renamed from
`DemoAct` to `Kept`, its act `KeptAct` gaining `Cancel` and
`RemoveProject`; the connector's vocabulary for `RemoveForge`, its
requests to the service and the events back. No behaviour changes: the
demo's end keeps its two acts under the new names; nothing removes a
forge yet (`rf-domain`).

Acceptance:

- `notes/remove-forge.md`, section 4, in `temper_engine_domain_forge`:
  `Kept { key, act: KeptAct }`, `KeptAct::{Close, Cancel, RemoveProject,
  RemoveAccount}`, `Stored::Kept`, `Key::Kept`; `DemoAct`, `DemoActKind`,
  `Stored::DemoAct`, `Key::DemoAct` gone whole, every reader renamed
  (`rg 'DemoAct'` across temper is empty but for `Event::DemoActs` /
  `Request::ListDemoActs`, the demo's selection, which stay). Requests and
  events: `Request::SendKept(Kept)` and `Event::KeptAnswered { key, keep }`
  replace `SendDemoAct` / `DemoActAnswered`, `Event::SendKept` replaces
  `SendDemoActs` (the restart's resend: `temper-engine-domain/src/restart.rs`).
  `Limits`' `demo_act_bound` becomes `kept_bound = tasks + 2`.
- Removal vocabulary: `Ask::RemoveForge { forge: u16 }`;
  `Answered::RemoveForge(RemoveAnswer)`, `RemoveAnswer::{Removing,
  Refused(RemoveRefusal)}`, `RemoveRefusal::{NoSuchForge}`;
  `Request::ForgeEnded { forge }`, `Request::RemoveGroup { forge }`,
  `Request::EraseSecrets { forge }`; `Event::GroupRemoved { forge }`,
  `Event::SecretsErased { forge }`, `Event::ProjectRemoved { project:
  u32 }`. The connector answers `Ask::RemoveForge` `Refused(NoSuchForge)`
  and ignores the three events until `rf-domain`.
- Codec (`temper-protocol-store`): schema `KeptClose`, `KeptCancel`,
  `KeptRemoveProject`, `KeptRemoveAccount`, `enum KeptAct`, `record Kept`,
  `Value::kept` in `DemoAct`'s place (same tag position, so family 19's
  bytes for `Close` and `RemoveAccount` keep their value tags if the
  generator allows; else regenerate and say so); `keys::KEPT = 19`;
  goldens regenerated in place; `STORE_VERSION` stays 10.
- The root (`temper-engine-domain`): `route/demo.rs`'s `send_end_act`
  becomes the sender of any kept row, sending `Close` and
  `RemoveAccount` as today; `Cancel` and `RemoveProject` arms keep the
  row (`keep: true`) with a comment `// rf-root sends it`. The new
  requests' arms in `route/forge_route.rs` drop them with a comment `//
  rf-root forwards it`; `Ask::RemoveForge` is not admitted by the root
  yet (no wire reaches it).
- Tests: `temper-protocol-store`'s
  `kept_rows_round_trip_each_act_under_family_nineteen`; every existing
  test passes under the new names (the demo's DemoAct tests renamed
  `kept_*` where their names say demo act).

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 2,
  family 19), `temper/docs/plan/playground-reset-r5/notes/remove-forge.md`
  (sections 1, 2 and 4).
- `temper/docs/design/protocol/store.md`, 2 and 4 (family 19).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s1b-connector-store-acts.md`,
  section 1, "DemoAct family" and every reader it lists.

## Owns and may change

- Owns: the files in `files` (the forge domain crate whole; the store
  crate; the root's demo route, forge route arms, restart resend and the
  reply table's bound in `domain.rs`).
- May change, rename and forced arms only: every reader `rg 'DemoAct'`
  finds in temper (`temper-fake/src/check.rs`, the engine worlds'
  `world.rs` / `protocol.rs`, the client and system worlds' collectors,
  temper-service's `route.rs` and `clients/translation.rs`).
- Options: none beyond the generator's tag order above.

## Steps

1. Rename and the new variants in the domain; the bound.
2. Codec; regenerate.
3. Root and readers.
4. Narrow: `heavy cargo nextest run -p temper-protocol-store --lib`;
   `heavy cargo nextest run -p temper-engine-domain-forge --lib -E
   'test(/demo/) | test(/kept/)'`; `heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E 'test(/demo/)'`.
5. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-protocol-store -p temper-engine-domain --all-targets -- -D
   warnings`; focused tests of those three and `temper-codecs-world`;
   `heavy cargo check --tests` for `temper-service`, `temper-fake`, and
   the engine, client and system worlds. Old-forms from `temper/`.
   Commit and push.

## Report

As `notes/common.md` says; every file outside the owned list you edited.
