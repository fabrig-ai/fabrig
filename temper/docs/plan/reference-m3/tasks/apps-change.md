+++
id = "apps-change"
part = "appsettings"
kind = "feature"
needs = ["apps-surface"]
files = ["jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/settings/application.rs", "jig/crates/jig-domain-core/src/settings/change.rs", "jig/crates/jig-domain-core/src/settings/check.rs", "jig/crates/jig-domain-core/src/routing/settings/dispatch.rs", "jig/crates/jig-domain-core/src/effects.rs", "jig/crates/jig-domain-core/src/tests/settings/application.rs", "jig/crates/jig-domain-people/src/refusals.rs", "jig/crates/jig-domain-people/src/boundary.rs", "jig/docs/design/domain/engine.status.md"]
effort = "high"
minutes = 150
+++

# apps-change: a change of an application's settings, judged and kept

## Goal

A party's `ChangeSettings` naming items of an application's declared kinds is checked by
the core, judged by each kind's connector within the decision's step, refused whole when a
connector refuses (*judged*, with its words) or when nothing changes (*empty*), and, once
committed, kept by the connectors in the same decision, the core keeping each setting's
state, size and named charters, never its bytes. A charter an application's value names is
held by the *reference* check.

Acceptance:

- The unit tests below pass in `jig-domain-core`; the crate's and `jig-domain-people`'s
  focused tests pass unchanged; `jig/tests/core`'s settings stories pass unchanged (a change
  of meaning of the settings decision: run them).
- The **Not yet** of `jig/docs/design/domain/engine.status.md` loses "17.5" alone, keeping
  "an application's kinds in 17.6 and 17.7" (later tasks close those).

## Inputs

- `jig/docs/design/domain/engine.md`, 17.1, 17.2 (**Refused whole**, **One decision**),
  17.5 (all but **At each start**), 17.8 (**The current settings**).
- `jig/docs/design/domain/connectors.md`, section 17 (**Judged in the change's decision**,
  **Kept in the same decision**).
- `jig/docs/design/domain/people.md`, 5.1 (`ChangeSettings`'s row).
- `jig/docs/design/client/domain/contract.md`, section 4 (*judged*).
- `temper/docs/plan/reference-m3/notes/contracts.md`, C1, C2 (as `apps-surface` built them:
  `settings::application::{Item, Edit}`, `JudgeSettings`' `proposal` field).
- `temper/docs/plan/reference-m3/notes/common.md`.

## Owns and may change

- Owned: the `files` above. `jig-domain-people`'s `Refusal` gains one arm and its
  `Ask::ChangeSettings` one field (below); every match and literal they reach in jig gets the
  compiler-forced arm or field: in `jig-protocol-store`'s people codec and
  `jig-protocol-engine-clients`' refusal translation, `unreachable!("a judged refusal:
  apps-store")` / `("..: apps-wire")`; in temper, the arm alone (stated cross-tree allowance).
- Neighbour: `jig-domain-people/src/*.rs` where `Ask::ChangeSettings` is matched (field
  only); `jig/tests/core/tests/suite/settings.rs` only if an assertion names a refusal's
  exact shape that this task's arm changes (say so). `Current::new`'s callers (the core's
  tests and `documents/settings.rs`' test helper) pass `&[]`. Every `settings::Revision`
  literal and `settings::Record` match in jig (`jig-protocol-store`'s settings codec,
  `jig-protocol-engine-clients`' and `jig-client-protocol-link`'s settings translations,
  `jig-client-test-records`, their tests) gets `applications: Box::default()` or
  `unreachable!("an application's record: apps-store")`; temper's
  `temper-engine-domain/src/store/settings_bytes.rs` `record` sizes `Record::Application`
  as `size_of::<State>()` plus 4 bytes per named charter (the arm alone, the stated
  cross-tree allowance).
- Options, pre-approved:
  - The people child's *not theirs* rule for a project's items: if
    `people::Ask::ChangeSettings` can carry `projects: Box<[u32]>` (the projects whose items
    the change names, sorted, unique) and the people child's role check already has a
    per-project owner test, add the field and check each (default). If the people child has
    no per-project owner test reachable there, refuse every project-scoped item from a party
    who is not a deployment owner, and say so in the report.
  - If the people child's `Decided` cannot wait for the judges' answers within the step
    (it must arrive before the step ends), keep the change in `core` under the ask's
    `owner` token and push `Decided` from the `SettingsJudged` handler (default); if the
    people child asserts its answer synchronously, say so and stop: that is a question.

## Steps

1. **State** (`settings/application.rs`, module doc citing engine.md, 17.5):
   ```rust
   pub enum Follows { Default, Changed { revision: u32 }, Removed { revision: u32 } }
   pub struct State { pub follows: Follows, pub size: u32, pub charters: Box<[u32]> }
   pub struct Recorded { pub kind: u32, pub scope: u32, pub number: u32, pub edit: SizedEdit }
   pub enum SizedEdit { Set { size: u32 }, Remove, Reset }
   ```
   (`Clone, PartialEq, Eq, Hash, Debug`). `Current` gains `pub(crate) applications:
   Map<Setting, State>` (keys `Setting::Application` alone), sized by the kinds'
   `per_scope` times their scopes (the deployment, `limits` projects) at `Current::new`, which
   takes `&[Kind]`. `Revision` gains `applications: Box<[Recorded]>` (its `items` keep jig's
   kinds alone); `Record` gains `Application { setting: Setting, state: State }`, keyed
   `Key::Changed(setting)`. `Core` keeps `kinds: Box<[Kind]>` from `Config`.
2. **The core's check** (`check.rs`, `change.rs`), for each item of `Setting::Application`:
   *shape* when its kind is not declared, its scope is not its kind's (`Deployment`: 0;
   `Project`: an open project's number), its number is 0, or a `Set`'s bytes pass
   `value_bytes`; *count* when the settings its kind would hold in that scope (states not
   `Removed`, plus the items' sets) pass `per_scope`. These run with jig's checks, before any
   ask, naming the item.
3. **Judging** (`dispatch.rs`): after the core's checks pass and before
   `plan_held`, group the application items by their kind's connector (in item order) and
   send each connector one `Ask::Effect(connector::Ask::JudgeSettings { owner, revision:
   u64::from(number), proposal: None, items })`, where a `Value::Application(bytes)` set
   becomes `Edit::Set(bytes)`, a removal `Edit::Remove`, a reset `Edit::Reset`. Park the
   decision under `owner` (a fresh token). The `SettingsJudged` handler in `effects.rs`
   matches the owner, checks one `Judged` per item, and, once every connector has answered:
   - any `Refuse { why }`: the change is refused whole with
     `Refusal::Judged { at, setting, why }` (`jig-domain-people`'s new arm, the first
     refused item, `why` cut to `people::Limits.words` bytes and checked UTF-8, a refusal
     not UTF-8 worded as empty), nothing committed;
   - every jig item unchanged (`changes_current`) and every application item `Same`:
     *empty* (`Malformed::SettingsUnchanged`), as today;
   - an `Accept { size, charters }` with more charters than its kind's `charters`, or
     a `size` past `value_bytes`: *shape*, naming the item (a connector's fault, refused);
   - **reference:** the charters every application state would name after the change (the
     accepted items' and the unchanged states') must be offered by the settings the change
     leaves; otherwise *reference*, naming the application's setting. A jig item that
     removes a charter a state names is refused the same way.
   - otherwise the rest of `decide` runs as now. A connector that answers with a wrong
     count, an unknown owner or twice is a fault of the root: `panic!` naming it.
   With no application item, `decide` runs exactly as now (no ask, no token).
4. **Kept**, in the committing decision: each item `Accept`ed updates its state
   (`Set`: `Changed { revision }`, the accepted size and charters; `Remove`:
   `Removed { revision }`, size 0; `Reset`: `Follows::Default` with the size and charters the
   connector accepted for the default) and writes `Record::Application`; a `Same` item is
   left out of the state, the record and the keep. Each connector named gets one
   `Ask::Effect(connector::Ask::KeepSettings { revision, proposal: None, items })` of its
   items not `Same`. The revision's `applications` records each kept item's `SizedEdit`.
   The writes count in the settings room: if `route_writes` has no room for one more write
   per application item at its bound, raise the bound's arithmetic in `route_writes` and
   `writes_bound` by the kinds' `per_scope` sum, and say so in the report.
5. **Tests** (`src/tests/settings/application.rs`, a scripted connector answering the
   core's asks in the same step, as the effect judge tests do): a set accepted, kept (the
   `KeepSettings` items and the state's size); a refusal carrying the words, nothing
   committed; all `Same` refused as empty; a removal kept as `Removed`; a reset kept as
   following its default; a value naming charter 3 accepted while 3 is offered, and a
   change removing charter 3 then refused as *reference* naming the application setting;
   *shape* for an undeclared kind, a wrong scope, bytes past `value_bytes`; *count* past
   `per_scope`.
6. Narrow tests: `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/settings/)'`,
   `heavy cargo nextest run -p jig-domain-people --lib`, then
   `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/settings/)'` (or the
   package `jig/tests/core` names; check the count). `cargo fmt`; `heavy cargo clippy
   --workspace --all-targets`. Remove "17.5, and" from the status entry as Acceptance says.
   Commit and push.

## Report

The refusal's final shape; where the parked decision lives; the option taken for *not
theirs*; any change to the write bounds, with the numbers.
