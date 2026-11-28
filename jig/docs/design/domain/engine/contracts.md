# Contracts: the connectors' asks

The shapes the changes of the goal that builds the connectors' port share
(`temper/docs/design/draft/design-economy.md`, 4.3): the contract's asks and answers case by
case, the declaration's new parts, and the engine's temporary bridge for the roots not yet
moved. They bind as delivered design does. The rules are `vocabulary.md`, 2.5 and
`../connectors.md`, 2; this document holds only what the code will hold once the goal lands, and
the goal's last change deletes it.

## 1. The capabilities' cases

Each capability's module of `jig-domain-contract` holds `pub enum Ask` and `pub enum Up`, with
these cases. A case's fields are those of today's variant it replaces, named as there, less any
`connector` field, which `Out::Ask` and `In::Connector` carry; a case that replaces two of
today's variants merges them. Each value a field names is the contract's, where the value move
put it. The kind of each ask: ◆ judgement, · notification, ▷ operation that ends later
(`vocabulary.md`, 2.5). Paths below: `C` is `jig-domain-core-vocabulary/src/vocabulary.rs`'s
`Ask` and `Event`, `E` is `effects/connector.rs`'s `Ask` and `Event`, `T` the tasks child's
`Event`.

| Module | `Ask` case | Kind, answered by | Today |
|---|---|---|---|
| `lifecycle` | `CatchUp` | ▷ `Done`, once per stage | `E::CatchUp` |
| | `Adopt { request, project, adoption }` | · | `C::Adopt` |
| | `ReleaseProject { project }` | ▷ `Released` | `C::ReleaseProject` |
| `resources` | `Unname { task }` | · | `E::Unname` |
| | `Hold { task, resource, entry }` | · | `C::Hold` |
| | `Lost { task, attempt, resources }` | · | `C::Lost`; `resources: Box<[Name]>` are the asked connector's names whose writer was that attempt, now lost (the tasks child's `lost_writes`, taken as the core makes the ask), so its connector reads them afresh from nothing of the root's; empty where it wrote none |
| | `Holdings { request, project, root, number, executor, spec, tracked, symbols }` | ◆ `Holdings` | `C::TaskHoldings` |
| | `DelegateHoldings { request, from, members }` | ◆ `DelegateHoldings` | `C::DelegateHoldings` |
| | `ProcedureHoldings { task, step, members }` | ◆ `ProcedureHoldings` | `C::ProcedureHoldings` |
| | `RepairRefused { repair, number, why }` | · | `C::RepairRefused` |
| | `DelegateRefused { task }` | · | `C::DelegateRefused` |
| `effects` | `Describe { owner }` | ◆ `Described`, `Refused` or `Busy` | `E::Describe` |
| | `DescribeProposal { owner, proposal }` | ◆ the same | `E::DescribeProposal` |
| | `Judge { owner, judge, state, resources }` | ◆ `Verdict` | `E::Judge` |
| | `Keep { owner, entry, task, key, project }` | · | `E::Keep`; `project: Option<u32>` is the task's project as the core reads it when it makes the ask, none where the task's row is gone, so a connector that keeps an entry's project needs nothing of the root's |
| | `KeepProposal { owner, proposal, task, project }` | · | `E::KeepProposal` |
| | `Drop { owner, answer, findings, why }` | · | `E::Drop` |
| | `DropProposal { proposal }` | · | `E::DropProposal` |
| | `Resolve { entry, again }` | ▷ `Outbox` | `E::ResolveUncertain` |
| | `Close { task, root, ending }` | ▷ `Closed` | `C::Close` |
| | `Release { task, root, ending, entry }` | ▷ `Released` | `C::Release` |
| `topics` | `Subscribed { request, key, subscription, topic }` | · | `C::SubscriptionDone`; the core asks it, then settles the run's call itself (`Subscribed { subscription }`), as it does for a subscription no connector owns: no call is handed to the application and no root answers it. `topic: u64` is the connector's topic of the subscription the core admitted (its `SubscriptionKind::Topic`), so the connector binds `subscription` to its topic, for its `News`, from the ask alone, whichever call (a topic call or a named subscribe) made it |
| | `Unsubscribed { request, key, subscription, topic }` | · | `C::UnsubscriptionDone`; the same, its call settled `Unsubscribed`; `subscription` and `topic` are the interest the call ended, as the core read it from the task's row to find its connector |
| | `Refused { request }` | · | `C::DropSubscription` |
| | `Ended { task, subscription }` | · | `C::EndTopic` |
| `parts` | `Document { watcher, scope, subject, party, window }` | ▷ `Made`, whenever its connector has read what it needs; the core's watcher waits for it | `C::Document` |
| | `DropDocument { subject }` | · | `C::DropDocument` |
| | `Read { reply, scope, read, party, window }` | ·, its answer the connector's own held output | `C::Read` |
| | `Lines(PartsAsk)` | ▷ `Lines`, whenever its connector has read what it needs (a store's held load); the core's pending read, under `token`, waits for it | `C::Parts` |
| | `Public { reply, kind, version, project, task, public }` | ◆ `Public` | `C::Public` |
| `procedures` | `Start { context, step, chat }` | · | `C::StartProcedure`; `chat: Option<u64>` is the conversation the task serves, as `Tasks::served_chat` gives it when the core makes the ask |
| | `StepMade { task, step, first, revisions }` | · | `C::StepMade` |
| | `StepRefused { task, step, why }` | · | `C::StepRefused` |
| | `WaitsRefused { task, why }` | · | `C::WaitsRefused` |
| `projections` | `Project { feed }` | ▷ `Settled` | `C::ProjectGoal` |
| | `Forget { goal }` | · | `C::ForgetProjection` |
| `settings` | `Judge { owner, revision, proposal, items }` | ◆ `Judged` | `E::JudgeSettings` |
| | `Restore { owner, proposal, items }` | ◆ `Restored` | `E::RestoreSettings` |
| | `Keep { revision, proposal, items }` | · | `E::KeepSettings` |
| | `KeepProposal { proposal, items }` | · | `E::KeepProposalSettings` |
| | `DropProposal { proposal }` | · | `E::DropProposalSettings` |
| | `ForgetReplaced { before }` | · | `E::ForgetReplaced` |
| | `SaveDefaults { revision }` | · | `E::SaveSettingsDefaults` |
| `requests` | `Decide { request, token, owner, person, scope, role, retention }` | ◆ `Decided`, or the core part the connector asks first (`C::ApplicationPart`, still `In::Core`) | `C::Application` |
| | `Drop { owner }` | · | `C::DropApplication` |
| | `PartAnswered { request, outcome }` | · | `C::ApplicationPartAnswered` |

| Module | `Up` case | Today |
|---|---|---|
| `lifecycle` | `Done { stage }` | `E::RestartDone`; the engine takes it as the roots' call to the core's catch-up completion does today |
| | `Released { project }` | `C::ProjectReleased` |
| | `Adopted { project, name, role, home }` | `E::Adopted` |
| `resources` | `Holdings { request, holdings, refused }` | `C::Holdings` |
| | `DelegateHoldings { request, holdings, refused }` | `C::DelegateHoldings` |
| | `ProcedureHoldings { task, step, holdings, refused }` | `C::ProcedureHoldings` |
| | `Resource { project, name, role, hold }` | `E::Resource` |
| | `PoolSlots { name, slots }` | `E::PoolSlots` |
| | `Drift { task, resource }` | `E::Drift` |
| | `ReadAfresh { resource }` | `T::ReadAfresh`, which roots build today |
| `effects` | `Described { owner, description }` | `E::Described` |
| | `Refused { owner }` | `E::DescribeRefused` |
| | `Busy { owner }` | `E::DescribeBusy` |
| | `Verdict { owner, judge, verdict, at, guarded, state }` | `E::Verdict` |
| | `Outbox { entry, task, outcome }` | `E::Outbox` |
| | `Closed { task }` | `E::Closed` |
| | `Released { task }` | `T::Settled`, which roots build today |
| | `Start { owner, origin }` | `C::EffectStart`, for a procedure's or a projection's effect; a run's call's still crosses as `In::Core` |
| `topics` | `News { task, subscription, topic, class, words }` | `E::News`; the connector names the `subscription` its `Subscribed` ask bound to that task and topic; after a restart, the root's start restore, outside the port (`vocabulary.md`, 2.5), gives it its subscriptions' numbers again from the restored tasks' rows, its stored form unchanged |
| `parts` | `Made { watcher, outcome }` | the views child's `Made`, which roots feed today as `C::View` |
| | `Lines { token, parts }` | `C::Parts` |
| | `Public { reply, answer }` | `C::PublicAnswered` |
| | `Marked { tasks }` | `C::PartsMarked` |
| `procedures` | `Step { task, step, code, decision }` | `C::ProcedureStep` |
| | `Waits { task, waits }` | `C::Waits` |
| | `Wake { task }` | `T::WakeProcedure`, which roots build today |
| `projections` | `Settled { goal }` | `C::ProjectionSettled` |
| | `Outbox { entry, outcome }` | `E::ProjectionOutbox` |
| `settings` | `Judged { owner, judged }` | `E::SettingsJudged` |
| | `Restored { owner, judged }` | `E::SettingsRestored` |
| | `Defaults { defaults, dropped }` | `E::SettingsDefaults`, said at a start as the roots' restart asks it today |
| `requests` | `Decided { request, outcome }` | `C::ApplicationFinished` |
| | `Marked { subjects }` | `C::Marked` |

The crate's root wraps them, as `vocabulary.md`, 2.5 shows: `Ask` and `Up` one case per module,
`Up::Act { key, act, carried }` (today's `C::Act` less `reply_to` and `connector`), and
`Released::Make { entry }` (today's `Held::MakeEffect`) and `Released::Answered { key, outcome }`
(today's `Held::Acted`'s outcome, for a connector's act). Each enum derives what today's does
(`Hash`, `Debug`) and `PartialEq`, `Eq`, as does every value its cases carry, so a connector's
own vocabulary that holds them keeps its equality; and none takes a `_` arm anywhere.

Not cased, and gone once no root names them: `E`'s `Names`, `Hold`, `ReleaseHold`, `Writer`,
`Make`, `ProcedureMade`, `ProcedureActivate`, `ProcedureMessage`, `ProcedureClose`, `Project`,
`Release`, `Subscribe`, `Unsubscribe`, `Read`, `Gather`, `CutTo`, `Take`, `Prepare`, `Left`,
`Adopt` and `Restore` asks, which the core never makes; and `E`'s `Procedure`, `SectionReady`,
`WorkspaceReady` events, which the core takes in an arm that does nothing.

An effect's owner is a token the core mints (`Core::effect_owner`), its staging flight, which
`effects::Up::Start` must name. A connector that starts an effect unasked (a procedure's step,
a projection's entry) can't mint one, and the core can't mint it at an ask, since one step or
projection may start several or none. So until the engine is the API (`vocabulary.md`, 3), a
root mints it through its access to the core when its connector asks for one in its own
vocabulary, and hands it back the same way; the connector keeps its effect under that owner
and says `effects::Up::Start { owner, origin }` itself. That exchange is among what a root still
translates, and its report lists it.

## 2. The declaration

```rust
// jig-domain-contract: the crate's root
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Connector(pub u16);
pub struct Declaration { pub connector: Connector, pub resources: Box<[ResourceKind]>,
                         pub effects: Box<[EffectKind]>, pub projects_goals: bool }
```

`Declaration::empty(connector: Connector)` declares nothing and projects no goal.
`StartProblem`'s connectors are `Connector`s (`vocabulary.md`, 2.1). Each root sets
`projects_goals` for each connector that takes a goal's projection today: temper's forge and the
test root's connectors that its harness projects for, and neither of `ops`'s.

## 3. The bridge, while the roots move

From the engine port's change until each root's own change, a root that has not moved routes the
new outputs back into its routing of today's through two functions of `jig-domain-core`'s
`engine` module, which the goal's last change deletes:

```rust
pub fn bridge_ask(connector: Connector, ask: contract::Ask) -> Request      // today's Request::Ask
pub fn bridge_released(connector: Connector, released: contract::Released) -> Request  // today's Held
```

They give back today's `Request::Ask { connector, ask }` and today's held `MakeEffect`, exactly as
the core made them before. `Released::Answered` has no exact counterpart today: `Held::Acted`'s
`ReplyTo` is a reply token, not a connector's number and act key, and neither is derived from the
other. So the bridge gives it back as one held case of its own, in-process alone and deleted with the
bridge: `Held::ConnectorActed { connector, key, outcome }`. A root that has not moved never meets it,
since its connectors' acts still cross as `Event::Act` in `In::Core` and are answered by
`Held::Acted` to their reply tokens; a holder's exhaustive match takes it in an arm of its own.
A root that has not moved keeps feeding its connectors'
answers as today's events in `In::Core`, which the core still takes. A moved root names neither
function nor today's variants.

## 4. Which change builds what

- The value move: every value the cases carry, in the contract.
- The enums: section 1's and 2's types, and each root's declarations with `projects_goals`.
- The engine port: `In::Connector`, `Out::Ask`, `Peer::Connector`; the core making and taking the
  contract's cases itself; asks only of declarers; the judgements' assertion; the bridge.
- The core's values for moved connectors: `procedures::Ask::Start`'s `chat`, `effects::Ask::Keep`'s
  `project`, and the subscribing calls the core settles itself, each root's answer to them gone.
- One change per root (`ops`, the test root, temper): its connectors speaking the contract, its
  asks routed one arm per connector, and no use of the bridge.
- The last: today's variants that only the bridge names, the bridge, and this document.
