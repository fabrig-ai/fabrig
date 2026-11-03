+++
id = "apps-surface"
part = "appsettings"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/settings/application.rs", "jig/crates/jig-domain-core/src/effects/connector.rs", "jig/crates/jig-domain-core/src/lib.rs", "jig/crates/jig-domain-people/src/settings.rs", "jig/crates/jig-domain-core/src/tests/settings/application.rs", "jig/crates/jig-domain-core/src/tests/settings.rs"]
effort = "medium"
minutes = 90
+++

# apps-surface: an application's kinds, items and asks, as types

## Goal

jig gets the types an application's own settings cross the core in (contracts C1 and C2):
a kind declared in `Config`, a setting of an application's kind, the judged fault, the
connector's items and the asks and events that judge and keep them. Nothing acts on them:
the core declares no behaviour for them, every match the new variants reach gets the
compiler-forced arm this spec gives, and no behaviour of any existing change, start or
route moves. The bodies are later tasks (`apps-core`, `apps-store`, `apps-wire`).

Acceptance:

- The types below exist with these names, fields, variants and derives, documented at type
  level citing `jig/docs/design/domain/engine.md`, 17.5 (the asks: also
  `jig/docs/design/domain/connectors.md`, section 17).
- `Config { kinds: Box<[Kind]>, .. }` exists; every `Config` literal in the workspace sets
  `kinds: Box::default()`.
- `heavy cargo check --workspace --all-targets` is clean; the focused tests of
  `jig-domain-people` and `jig-domain-core` pass, with the new unit test below.

## Inputs

- `jig/docs/design/domain/engine.md`, 17.1 (the table's last row), 17.5, 17.6 (**The owner
  keeps the value**), 17.7.
- `jig/docs/design/domain/connectors.md`, section 17.
- `temper/docs/plan/reference-m3/notes/contracts.md`, C1 and C2: binding names. This spec
  places C2's `Item` and `Edit` at `jig_domain_core::settings::application::{Item, Edit}`
  because `jig_domain_core::settings::{Item, Edit}` already name the core's own change
  items (a party's `Change`); every field and variant is C2's.
- `temper/docs/plan/reference-m3/notes/common.md`.

## Owns and may change

- Owned: `jig/crates/jig-domain-people/src/settings.rs`;
  `jig/crates/jig-domain-core/src/settings.rs` (the `mod` and `pub use` lines, `Value`'s
  new arm); `jig/crates/jig-domain-core/src/settings/application.rs` (new);
  `jig/crates/jig-domain-core/src/effects/connector.rs` (`Ask`, `Event`, the new types);
  `jig/crates/jig-domain-core/src/lib.rs` (`Config`'s field); the new unit test file and its
  `mod` line.
- Neighbour files, **compiler-forced arms and the new literal field only**, each arm as the
  table below says, nothing else of the file changed:
  - every `Config { .. }` literal (jig's: `jig-domain-core`'s tests, `jig-protocol-config`'s
    `src/core.rs`, `jig-test-profile`, `jig-test-domain`'s fixtures, `jig/tests/core`,
    `jig/tests/engine-clients`, `jig/examples/ops`; temper's: `temper-engine-domain`'s
    `src/tests/resume_fixture.rs`, `temper/tests/engine/domain/src/walking.rs` and
    `src/oracle/config.rs`): `kinds: Box::default()`. temper's lines are the stated
    cross-tree allowance for a required field (edited in that field alone).
  - every exhaustive `match` on `jig_domain_people::Setting`, `SettingFault`,
    `jig_domain_core::settings::Value`, `effects::connector::Ask` or `effects::connector::Event`
    that the compiler names, in jig and in temper (temper's `temper-engine-domain`
    `route/forge_route.rs`, `temper-engine-smith` `src/settings.rs`,
    `temper-engine-domain` `src/store/settings_bytes.rs` and any other it names: the arm
    alone, the stated cross-tree allowance).

| Where | The arm |
|---|---|
| the core's `effects.rs` handler of `connector::Event` | `SettingsJudged { .. } \| SettingsDefaults { .. } => {}` (no ask is sent yet) |
| `vocabulary.rs`'s `effectconnector_new_work` | both new events `=> false` |
| a root's routing of `connector::Ask` (`jig-test-domain` `src/routes.rs`, `jig/examples/ops/domain/src/route.rs`, temper's `forge_route.rs`) | one arm for the five new asks: `unreachable!("no application kind is declared")` |
| any other match on `Setting::Application`, `SettingFault::Judged` or `Value::Application` (the core's settings, documents, reads and admission; the store codecs; the client translations; temper's) | `unreachable!("an application's setting: <task>")`, `<task>` the later task that gives it a body: `apps-core` in `jig-domain-core`, `apps-store` in `jig-protocol-store`, `apps-wire` in `jig-protocol-engine-clients`, the client crates and temper's |

  A match the arm would make non-exhaustive in a way the table does not cover (a `matches!`
  that changes meaning, a `_` arm that now catches the new variant silently): leave it,
  and list it in the report.
- Options, pre-approved:
  - If `Setting`'s `Ord` derive orders the new variant last, keep it (no reader relies on
    order past jig's kinds); if a sorted golden or a test's order breaks, say so in the
    report and leave the variant last.
  - If a `Config` literal sits in a file whose crate does not build under
    `--all-targets` for you, edit it anyway and say so.

## Steps

1. `jig-domain-people/src/settings.rs`: `Setting` gains
   `Application { kind: u32, scope: u32, number: u32 }` (scope 0 the deployment's, otherwise
   a project's number); `SettingFault` gains `Judged` (a unit variant: the words the
   connector gave ride beside it in later tasks; `SettingFault` stays `Copy`). Doc lines name
   engine.md, 17.1 and 17.5.
2. `jig-domain-core/src/settings/application.rs` (new, `pub mod application;` in
   `settings.rs`, module doc citing engine.md, 17.5):
   ```rust
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub enum KindScope { Deployment, Project }
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub struct Kind { pub number: u32, pub scope: KindScope, pub connector: u16,
       pub value_bytes: u32, pub per_scope: u32, pub charters: u8 }
   #[derive(Clone, PartialEq, Eq, Hash, Debug)]
   pub enum Edit { Set(Box<[u8]>), Remove, Reset }
   #[derive(Clone, PartialEq, Eq, Hash, Debug)]
   pub struct Item { pub kind: u32, pub scope: u32, pub number: u32, pub edit: Edit }
   ```
   `settings.rs` re-exports `pub use application::{Kind, KindScope};` (C1's paths); `Item`
   and `Edit` stay at `settings::application::` (the core's own `settings::{Item, Edit}`
   are unchanged). `Value` gains `Application(Box<[u8]>)`: a party's change item of an
   application's kind, `settings::Item { setting: Setting::Application { .. }, edit:
   Edit::Set(Value::Application(bytes)) }`, which `apps-core` hands its connector as an
   `application::Item`.
3. `jig-domain-core/src/lib.rs`: `Config` gains, after `defaults`,
   `/// An application's kinds of settings (domain/engine.md, 17.5).`
   `pub kinds: Box<[settings::Kind]>,`. Nothing reads it yet (a scoped
   `#[expect(dead_code, reason = "...")]` only if the compiler asks).
4. `effects/connector.rs`:
   - `Ask` gains, documented each in a line from C2:
     `JudgeSettings { owner: Token, revision: u64, proposal: Option<u64>, items:
     Box<[settings::application::Item]> }` (`proposal`: the accepted proposal whose kept
     values a `Set` item names, its bytes then empty; a field C2 allows),
     `KeepSettings { revision: u64, proposal: Option<u64>, items: Box<[..Item]> }`,
     `KeepProposalSettings { proposal: u64, items: Box<[..Item]> }`,
     `DropProposalSettings { proposal: u64 }`,
     `SaveSettingsDefaults { revision: Option<u64> }`.
   - `Event` gains `SettingsJudged { owner: Token, judged: Box<[Judged]> }` and
     `SettingsDefaults { defaults: Box<[DefaultState]> }`.
   - New, beside them:
     ```rust
     #[derive(Clone, PartialEq, Eq, Debug)]
     pub enum Judged { Accept { size: u32, charters: Box<[u32]> }, Same, Refuse { why: Box<[u8]> } }
     #[derive(Clone, PartialEq, Eq, Debug)]
     pub struct DefaultState { pub kind: u32, pub scope: u32, pub number: u32, pub size: u32,
         pub charters: Box<[u32]>, pub differs: bool }
     ```
5. The compiler-forced arms and `kinds` literals of `## Owns and may change`, found with
   `heavy cargo check --workspace --all-targets`.
6. Test `jig-domain-core/src/tests/settings/application.rs` (a `mod application;` line in
   the settings tests' module): one table test that builds a `Config` from the crate's test
   `config(seed)` with `kinds: Box::new([Kind { number: 1, scope: KindScope::Project,
   connector: 2, value_bytes: 64, per_scope: 4, charters: 1 }])`, starts the core as the
   existing settings tests do, and asserts the start's outputs equal those of the same
   config with no kinds: declaring a kind changes nothing yet.
7. Narrow tests: `heavy cargo nextest run -p jig-domain-people --lib`;
   `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/settings/)'` (check the
   count); `heavy cargo check --workspace --all-targets`; `cargo fmt`; `heavy cargo clippy
   --workspace --all-targets`. Commit and push.

## Report

The arms added, by file, with their bodies (a list); any match left as `## Owns and may
change` says; whether `Setting`'s order moved anything; the `Config` literals edited.

## Answers

2026-11-03. May the run correct the fixed settings-document price in
`jig/crates/jig-domain-core/src/tests/inbox/inbox_selection.rs` that the new variants
enlarge? Yes: the focused `jig-domain-core` tests are this task's acceptance and the
growth is this task's own type change, so the scope widens to that test's
`SETTINGS_DOCUMENT` (and, if it is where the growth lands, `TEMPLATE_ENVELOPES`) and its
comment, nothing else of the file. The comment names the carriers that grew and by how
much each, so the 4,272 bytes are a sum of named terms (which type's size moved, times how
many it prices), not a bare added number. Run that test alone, then the focused suites to
the end. The same allowance covers any other `jig-domain-core` or `jig` test whose fixed
byte price fails only because `Setting`, `Value`, `Ask` or `Event` grew: the expectation
and its comment alone, each listed in the report. A production bound, limit, `valid`
check or budget that fails is not covered: stop and ask.
