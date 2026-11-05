# ui-settings: the shapes that cross tasks

Binding for every task of the plan. Where a spec and this note differ, this note wins and the report
says so.

## Kinds 6 and 7 (`temper/docs/design/protocol/configuration.md`, 13.2 and 13.4)

The wire, `client-wire-v1.schema`, under *Values of temper's settings kinds*:

```
enum QuestionsRoute { none, foreman, up }
record ProjectQuestions { route: QuestionsRoute }
record ProjectWays { drafts: bool goals: bool changes: bool away: bool way: u32 }
```

The store, `store-v11.schema`, family 20, reusing its `Route`:

```
record ProjectQuestions { route: Route }
record ProjectWays { drafts: bool goals: bool changes: bool away: bool way: u32 }
enum SettingValue { rule: LandingRule, charter: CharterValue, questions: ProjectQuestions, ways: ProjectWays }
```

The forge, `temper-engine-domain-forge/src/settings.rs`:

```rust
pub enum Kind { DeploymentRules, ProjectRules, Charters, ProjectQuestions, ProjectWays } // 1, 2, 3, 6, 7

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ProjectWays { pub drafts: bool, pub goals: bool, pub changes: bool, pub away: bool, pub way: u32 }

pub enum Value { Rule(LandingRule), Charter(u32), Questions(temper_engine_domain_forge_change::Route), Ways(ProjectWays) }

pub enum Refusal { /* as today */ Way } // "a way that is not a template the project offers"
```

And on the forge's `Domain`:

```rust
/// Kind 6 of `project`, `Route::None` with none stored (configuration.md, 13.2 and 13.4).
#[must_use]
pub fn project_questions(&self, project: u32) -> temper_engine_domain_forge_change::Route;
```

`Setting::Application { kind, scope, number }`, kind 6 or 7, `scope` the project, `number` 1.

## Their names (`temper/docs/design/client/tools.md`, 4 and 5.3)

| Kind | Text | Selector in a settings file | Value in a settings file and `--json` |
|---|---|---|---|
| 6 | `project-questions P` | `{"kind": "project-questions", "project": P}` | `"none"`, `"foreman"` or `"up"` |
| 7 | `project-ways P` | `{"kind": "project-ways", "project": P}` | `{"drafts": true, "goals": true, "changes": true, "away": true, "way": 0}` |

## The entrances

Three places take "temper's kinds" as the range `1..=3`: `temper-service/src/clients/translation.rs`,
`temper-engine-smith/src/settings.rs` and `temper-client-tools/src/settings.rs`. Each takes kinds 6 and 7
from `ways-ctl` on. Until then a client's or an agent's item of those kinds is refused as a value of another
kind; `ways-kind`'s world test feeds its typed values directly.

## The address and the frame (`settings-frame`)

```rust
// temper-client-domain/src/address.rs
Address::Repositories { project: u32 } // /projects/<project>/settings/repositories
```

```rust
// temper-client-view/src/pages/settings.rs, new; `pages/home.rs::settings` and `settings_controls` go
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Tab { HowWorkRuns, Repositories, Forges, Accounts }

/// Opens the tab's `.col` and draws its kicker, heading, lede and tabs; the caller draws the tab's body and
/// closes it (temper/docs/design/client/README.md, 3.5, *Settings*).
pub(crate) fn open(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, tab: Tab);

/// How work runs, `Address::Settings`.
pub(crate) fn how_work_runs(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32);

/// Repositories, `Address::Repositories`.
pub(crate) fn repositories(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32);
```

`forge::forges::main` and `pages::accounts::main` take `project: Option<u32>`, the page's `rail_project`, and
call `open` with `Some`.

## The session's facts (`ways-session`)

```rust
// temper-client-domain/src/forge/ways.rs, new: `pub mod ways;` in forge.rs
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Ways { pub drafts: bool, pub goals: bool, pub changes: bool, pub away: bool, pub way: u32 }
impl Ways { pub const STOCK: Self; } // all false, way 0

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Route { None, Foreman, Up }

/// A control of How work runs; `Ways` for all of kind 7, which writes one record.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Control { Ways, Questions }

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Asked { Ways(Ways), Questions(Route) }

/// An act of How work runs, one request (5.13): `forge::Act::Ways(ways::Act)`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Act { Set { project: u32, asked: Asked } }

impl Side {
    /// Kind 7 of `project`. `None` until read, or if refused, or if `project` is not the open one.
    pub fn ways(&self, project: u32) -> Option<Ways>;
    /// Kind 6 of `project`, read while the tab is open. `None` as `ways`.
    pub fn questions(&self, project: u32) -> Option<Route>;
    /// What `control` has asked and not been answered: shown in its place, and disabling it.
    pub fn ways_asked(&self, control: Control) -> Option<&Asked>;
    /// `control`'s last refusal, until its next act.
    pub fn ways_refused(&self, control: Control) -> Option<&jig_client::contract::Refusal>;
    /// The session's `settings` watch: Home's facts read it.
    pub fn settings_handle(&self) -> jig_client::Handle;
}
```

`route::ways_progress(root, env, out)`, beside `route::playground_progress`, tells the side the open project
and whether the tab is open, after every step and fire. The way a project offers to start a goal by:

```rust
// temper-client-domain/src/forge/ways.rs
/// The template `ways.way` names if `project` offers it with `tracked` set, else the first such, in number order.
#[must_use]
pub fn default_way(me: &jig_client::contract::Me, project: u32, ways: &Ways) -> Option<u32>;
```

And in the view:

```rust
// temper-client-view/src/pages/chats.rs
/// Whether chats in `project` can write drafts: kind 7's `drafts` (temper/docs/design/client/README.md, 5.13).
#[must_use]
pub fn drafts_offered(root: &Domain, project: u32) -> bool;
```

If `drafts_offered` is on the trunk under another signature, keep that signature; it reads `drafts`, and with
none read answers false.

## How work runs' rows (`how-work-runs`)

A row is one function in `pages/settings.rs`, `.setting`: the label in a `span`, `small` under it, the control at its right, its refusal under it. The toggle's value is computed where
the row is drawn, from what the row shows: `Asked::Ways(Ways { goals: !shown.goals, ..shown })`, so a press sends the whole record.

```rust
// temper-client-view/src/forge/bindings.rs
Meaning::Ways(temper_client_domain::forge::ways::Act)
```

A select's options carry their route in the option's value, as `Act::Filter`'s carry theirs (`frame.rs`, `decode`).

## A proposal's values (`settings-proposal`)

```rust
// temper-client-domain/src/forge/ways.rs
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Proposed { Ways { project: u32, ways: Ways }, Questions { project: u32, route: Route } }

impl Side {
    /// The values of kinds 6 and 7 proposal `key` sets, from the forge's `ProposalItems`. `None` until
    /// delivered, or if refused.
    pub fn proposed(&self, key: u64) -> Option<&[Proposed]>;
}
```
