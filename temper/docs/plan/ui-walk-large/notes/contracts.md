# ui-walk-large: the shapes that cross tasks

The run's binding calls. Each task builds its half from here, and needs no other task for it unless its `needs` says so. The design is `temper/docs/design/protocol/configuration.md`, 13.4; `temper/docs/design/domain/forge.md`, 5.6 and 5.7; `temper/docs/design/client/README.md`, 3.5; `jig/docs/design/domain/engine.md`, 17.6; `jig/docs/design/client/domain/contract.md`, 5.1.

## 1. The stock's room (`stock-room`)

With C, K and the chat's 24K as `stock.rs` has them, all from `checked`:

| value | was | is |
|---|---|---|
| `rules.ceiling.budget.spend`, policy 1's `ceiling.budget.spend` | 80 × 24K | 80 × 24K |
| policy 1's roles 0 and 1 `period_spend` (the pools) | 80 × 24K | 80 × 24K |
| policy 1's role 2 `period_spend` | 24K | 24K |
| `rules.period_spend`, policy 1's `period_spend` | 80 × 24K | **81 × 24K** |

On `temper init`'s box, K 4,228: pool 8,117,760, period 8,219,232, room 101,472.

## 2. `projects` (`proposal-projects`; read by `needs-proposal`)

A proposal of settings keeps `project` 0 and holder `Policy { project: 0, kind: Settings }`. Both still say where it is decided; Accept sends `project` 0 as it does today. `projects` says where it shows.

```rust
// jig-domain-core, settings
pub const PROPOSAL_PROJECTS: u32 = 32;
/// The projects a proposal's items touch, ascending and distinct, the lowest
/// PROPOSAL_PROJECTS (domain/engine.md, 17.6).
pub fn touched_projects(items: &[Item]) -> List<u32>;
// Setting::Application { scope, .. } with scope != 0 touches scope;
// Setting::ProjectPolicy { project } touches project; every other setting none.

// jig-domain-core, inbox.rs
impl Core {
    /// What a reader's row of this pending proposal names: touched_projects, or
    /// with none, each open project `party` owns, the lowest PROPOSAL_PROJECTS.
    pub(crate) fn settings_row_projects(&self, party: u64, key: (people::Proposer, u32)) -> List<u32>;
}

// jig-domain-core documents::InboxEntry and jig-client-domain contract::InboxEntry
pub projects: List<u32>, // empty on every row but a pending proposal of settings
#[must_use]
pub fn shows_in(&self, project: u32) -> bool {
    self.project == Some(project) || <self.projects holds project>
}

// jig-domain-core documents::Proposal and jig-client-domain contract::Proposal
pub projects: List<u32>, // touched_projects; empty for the deployment's and for every other proposal
```

Wire, `jig/crates/jig-client-protocol-wire/schema/client-v19.schema`, in place, at the end of each record:

```
record InboxEntry { ... cursor: InboxCursor projects: list 32 u32 }
record Proposal { ... acts: bytes 16777216 projects: list 32 u32 }
```

A read's project filter is `shows_in`. A row counts once in `cached_needs`, and the doc's rows are one per proposal.

## 3. Home's brief and sentence (`home-true`)

In `temper-client-view/src/pages/home.rs`, with the `status` document's existing fields and no wire change:

- A brief is **new** where `brief.landed() > seen`, or `previous` is some and `brief.at() > previous`.
- Headline's landed sentence: a new brief's `sentence`; else with N = `landed - seen`, the existing one or many word; else with N 0, `landed > 0` and `last_landed` some, `Word::HomeLandedLast`, "The last change landed SPAN ago.", SPAN as `TimeForm::Looked` says it, or under a minute `Word::HomeLandedNow`, "The last change landed just now."; else nothing. All of it nothing while To begin shows, as today.
- Lede and byline: a new brief's, else none.

## 4. `forge.sitting_ms` (`home-true`)

`temper_engine_domain_forge::Limits` gains `pub sitting: skein_lib::Duration`, the override `forge.sitting_ms` as `forge.queue_window_ms` is, invalid at zero as `queue_window` is. Both profiles set it to `SITTING`, which stays the constant they name. `status.rs`'s `looked` compares with `env.limits.sitting`.

## 5. Words the card takes (`needs-proposal`)

`Word::SettingsProposalTitle` keeps "How temper will run PROJECT", PROJECT the `projects`' names joined by ", " and " and ". With none, `Word::SettingsProposalTitleAll`, "How temper will run your projects". `HistoryHolder::Policy { project: 0, .. }` says `Word::SettingsProposalDeploymentOwners`, "Owners of this deployment", never "Owners of project 0".
