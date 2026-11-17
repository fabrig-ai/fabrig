# jig-domain-api: one engine, a few ports

Draft, 2026-11-17. jig's domain grew organically. Its model is sound, but what an application meets is jig's internal pipeline: some 200 kinds of message on the core's edge, sorted by timing before destination, a second vocabulary for connectors, three spellings of the same operations, and roots that reach into the core's children. This draft gives jig's domain the API it would have had if designed from scratch, in line with skein's `skein/docs/foundation/programming-model.md` and jig's kit idea (`jig/docs/design/README.md`, 1), and the way to get there from today's code without a flag day. It covers the domain layer only: the protocol layer and the client change only where they build or read the engine's messages. Its execution file is [jig-domain-api.execution.md](jig-domain-api.execution.md).

## 1. In one page

- **The model stays** (section 17): the five primitives, one decision and one commit through lib's journal, outputs marked by when they may leave, the owner keeping the value, restart as the core's script, plumbing and sugar, level-triggered views, recovery classes and verdicts.
- **One engine, a few ports, named for their peers** (sections 4 and 5). `Engine` has `step`, `due`, `fire`, `next_deadline` and `idle`, and nothing else public. What comes in is `In`, by peer: the store, hosts, parties, connectors, accounts, the application, and its own resumptions. What goes out is `Out`, by mark first (a write, a connector's ask, held until the commit, or now) and then by peer.
- **Connectors declare what is static and are asked only for what needs judgement** (section 6). A connector's kinds are one `Declaration` given at construction; what depends on a resource, a member or a run is one `Ask` enum of twelve families, of three kinds (judgements, notifications, operations that end later), answered by one `Up`. Both live in a contract crate, `jig-domain-contract`, which the engine and every connector name, as `jig-domain-link` already is between the fleet, the hub and accounts.
- **One set of operations for every actor** (section 7). A party's compound request, a run's tool call and a procedure's step are each an `Act`: an actor and its operations, in the shapes each source makes today, checked by one authority and refused in one shape.
- **One mechanism per concern** (sections 5.1, 6.6 and 10): one load with one typed terminal, paced by the engine; one way to name a part a connector keeps; one opaque continuation; run preparation inside the engine.
- **Decoded at the edge** (section 8): calls and turns are decoded by the protocol layer on the way in, every path disposing of what it decoded, so nothing goes from the domain to the protocol layer and back mid-decision.
- **One way to every host** (section 9): the engine's own slots become a worker of their own in the engine's process, as one-process deployments already run their workers, so the engine's root holds no hub and `jig-domain-hub-link` goes.
- **The core's children are private** (section 10). Their values reach others through the core's vocabulary; no production crate steps them, holds them or names their crates.
- **A root's jig wiring fits on a page** (section 11) and grows only with its connectors, which `jig/docs/design/README.md`, 1 already promises. The conformance kit answers jig's own ports itself (section 12).
- **Getting there** (section 15 and the execution file): the engine first as a pass-through every root adopts at once; then port by port, each milestone moving all three roots; then the clean-up that makes the engine the real API. Counts that may only fall measure it (section 16).

## 2. What is there today

Counted on main at `fb973d1173` (2026-11-16), outside tests unless said, and checked again at `422be3b57f`, where jig is the same:

- **The core's edge** (`jig/crates/jig-domain-core-vocabulary/src/vocabulary.rs`): `Event`, 72 variants in. `Request`, out: `Ask` 34, `Held` 41, `Now` 39, `LinkNow` 7, `Write` 5. Connectors add `effects::connector::Ask`, 39, and its `Event`, 19. Children's enums cross it whole: `tasks::Event` 86, `fleet::Event` 38, `people::Event` 19 among them.
- **Sorted by timing, then destination.** A host's outputs, a party's, the store's and a connector's are mixed in `Held` and `Now`, so a root must know each of 80 variants to send it where it goes.
- **The same verbs spelt three ways:** a party's `people::Ask` (39: create, delegate, propose, withdraw, stop, cancel, release, amend, move, decide…), a run's `NamedAction` (22: message, amend, release, decide, propose…, and five reads), and a procedure's `ProcedureStep` (batch, amendments, cancels, messages, proposals).
- **Ten loads by name:** `Now::StartPreparation`, `SeenConversation`, `LoadEndedRead`, `EndedProposal`, `DecidedProposal`, `EndedPersonTask`, `EndedTask`, `PartyInputs`, `EndedEscalation`, `Held::NotesLoad`, beside `Held::Load` and `ReadLoad`, each with its own answer, most answers carrying a `busy` flag the root fills.
- **Run preparation through the root:** `Now::Activate` and `Event::Activate { ready, busy }`, `StartPreparation`, `BriefCorePlanned`, `CompleteBrief`, `WorkspaceRequest`, `RunPrepared`, `ClaimPrepared` and `take_due`, with roots answering some of them with fixed values (`jig/docs/design/domain/root.md`, 14).
- **Hand-outs:** a run's call goes out to be decoded and comes back typed (`Now::Call`, `Event::NamedAction`); turns' and answers' payloads stay with the root and come back translated (`LinkNow`, `Event::TurnPayload`, `AnswerPayload`, `AcceptedTurn`, `RefusedPayload`); a settled call's answer is rendered and handed back (`Event::SettledCall`).
- **The boundary leaks.** temper's root (`temper/crates/temper-engine-domain`) calls 60 distinct core methods besides the step. The three roots reach the core's children's state 389 times (`core.tasks`, `core.people` and the rest), temper's 338 of them, and build the children's own events 96 times, temper's 69 (`tasks::Event::HoldFrom` 12, `WakeProcedure` 10, …). `ops`'s reference root steps `jig_domain_tasks::step(&mut core.tasks, …)` itself to install hold kinds. 67 manifests outside the core name one of its children's crates.
- **So roots are large and diverge.** `ops`'s root is 3,604 lines, the test root 4,504, temper's 37,965, about 14,000 of them in files that by their names are jig's wiring. The roots keep different shapes and orders (`jig/docs/development/debt/D004-ops-root-jig-wiring.md`), and the conformance kit needs an adapter of 19 functions, 328 to 766 lines per adopter (`jig/docs/development/debt/D030-conformance-not-drop-in.md`).
- **Inside:** `Core` is one object of 92 fields at `422be3b57f`, 83 when `jig/docs/development/debt/D003-core-one-object.md` last measured it; its continuations are bare numbers (`D002-core-untyped-continuations.md` there); and `TaskRecord` holds every executor's state in 89 fields, 78 at `D014-task-record-union.md`'s check.

## 3. Who meets jig's domain

- **An application's engine root,** which composes the engine with its connectors and its own children (`jig/docs/design/domain/root.md`). It meets every port.
- **A connector's author,** who meets the contract (section 6) and the operations a procedure makes (section 7).
- **A worker's root,** which composes the hub with smith's agent and its workspaces (section 9).
- **The protocol layer,** which builds `In` from bytes and reads `Out` into bytes: the store's codecs, the clients' component, the workers' channel, the accounts' component.
- **Worlds and the conformance kit,** which play every peer (section 12).

## 4. The engine

### 4.1 Its surface

```rust
// jig-domain-core: the engine's whole public surface
pub struct Engine { /* private: the core and its children */ }

impl Engine {
    pub fn new(config: Config, connectors: &[contract::Declaration], limits: &Limits) -> Result<Engine, Problem>;
}
pub fn step(engine: &mut Engine, env: &Env<Limits>, input: In, out: &mut Queue<Out>);
pub fn due(engine: &Engine, now: Time, wall: Wall, admit: Admit) -> Option<Timer>;
pub fn fire(engine: &mut Engine, env: &Env<Limits>, timer: Timer, out: &mut Queue<Out>);
pub fn next_deadline(engine: &Engine, now: Time, wall: Wall, admit: Admit) -> Option<Time>;
pub fn idle(engine: &Engine) -> bool;
```

- **Timers keep today's admission.** `Admit` is `All`, or `Finishing` while the root holds new work at its watermark, so only a timer that advances admitted work is due then, as `due` and `finishing_due` choose today (`jig/crates/jig-domain-core/src/routing/timers.rs`). Deadlines take both clocks, since a sign-in expires on the wall.
- **Where it lives.** `Engine` lives in `jig-domain-core`, and its vocabulary, `In`, `Out`, `Peer` and each port's enums, in `jig-domain-core-vocabulary`, along the same module paths (`jig/docs/design/domain/engine/README.md`, 3). No crate is added for it, so none of the 48 manifests that name `jig-domain-core` changes for the name.

### 4.2 In and out, by peer

```rust
pub enum In {
    Store(store::In),              // a load's terminal; the store's reports
    Host(Channel, host::In),       // hellos, turns, calls and answers decoded, notices, losses
    Party(party::In),              // sign-ins, acts, reads, watches, deliveries taken
    Connector(u16, contract::Up),  // answers to asks, and what a connector says unasked
    Account(account::In),          // the accounts' component; the application's credentials
    Application(application::In),  // periods, stopping, settling
    Resume(Resume),                // its own continuation, fed back once released
}

pub enum Out {
    Write(store::Write),           // into this decision's commit
    Ask(u16, contract::Ask),       // to a connector, within the step
    Held(Peer),                    // leaves once this decision's commit is durable
    Now(Peer),                     // decides nothing and shows nothing durable
}

pub enum Peer {
    Store(store::Out),
    Host(Channel, host::Down),
    Party(party::Out),
    Connector(u16, contract::Released),
    Account(account::Out),
    Application(application::Out),
    Resume(Resume),
}
```

- **Peer first, inside each mark.** The root needs four arms to put the engine's outputs into the journal and none to know what they are (section 11). After the commit, `iterate` sends each released output by its peer: to the protocol layer, to the connector it names, or back to the engine.
- **Every name crossing is a token or a number,** as today (`skein/docs/foundation/programming-model.md`, 4.2): `Channel` a host link's, `Resume` the engine's own, a connector by its number.

### 4.3 Four marks

The core already says of each output whether it is part of the decision, held or now (`jig/docs/design/domain/engine/README.md`, 4.2). The marks stay and become the outer layer of `Out`:

- **`Write`:** a record saved or erased, into the decision's commit (`jig/docs/design/domain/engine/README.md`, 5.6).
- **`Ask`:** a hand-off to a connector within the step (`jig/docs/design/domain/engine/README.md`, 4.3), of one of three kinds (6.3). The engine asserts at the decision's end that every judgement it asked for was answered, as its `*_routes_drained` predicates do today.
- **`Held`:** an output someone outside could act on, released after the commit (`jig/docs/design/domain/engine/README.md`, 5.2).
- **`Now`:** the journal's door (`jig/docs/design/domain/root.md`, 3).

### 4.4 Admission

- **One class per input,** a free function of the vocabulary: `class(&In) -> Class`, `Taken`, `New` or `Public`. What finishes admitted work is `Taken` (the store's answers, connectors' answers, hosts' completions, resumptions); new external work is `New`; a public read is `Public`. The root admits `New` under its watermark and `Public` under the public one, and lets what is always taken pass a held event in order, as today (`jig/docs/design/domain/root.md`, 3 and 10; `skein/docs/foundation/programming-model.md`, 7.1). No input mixes classes (section 14.4), so nothing is split.
- **`idle(&Engine)`** says the engine has nothing to do until an input comes or its next deadline passes. It replaces `quiescent` and `finish_owed`'s five reasons, which stay the engine's to compute (`jig/docs/design/domain/root.md`, 3).
- **The engine paces its own work:** its loads by `Limits::load_slots`, its activations by them, its decisions' hand-offs by `Limits::routes`. The root answers no `ready` and no `busy`.

### 4.5 Nothing else

`Engine` has no other public method and no public field. The 60 methods temper's root calls today, and those `ops` and the test root call, each become an input of a port, an output, or the engine's own business (section 13). Worlds read the engine as its peers do, and the store as the restart oracle does (`jig/docs/design/domain/engine/README.md`, 9).

While the move runs, the pass-through keeps a temporary access to the whole `Core`, shared and exclusive, by method and never by `Deref`, so that every use roots make of it today, its children's state and its counters included, compiles unchanged. Each milestone narrows it, and the clean-up removes it.

## 5. The ports

### 5.1 The store

- **Writes:** `Save`, `Erase`, `DropTranscript`, `DropHistory`, `CopySettingsRevision`, as today, and the start's own writes (below).
- **One load and one terminal.** `Peer::Store(Load { load, range })`, held, so it sees every commit before it (`jig/docs/design/domain/engine/README.md`, 5.3), and `In::Store(Loaded { load, result })`, whose `result` is typed: rows with whether more remain, cut to its bytes and by how much, or failed, with the reasons today's answers carry (gone, too linked, a turn's problem, a transcript cut or unread; `jig/crates/jig-domain-core-vocabulary/src/reads/vocabulary.rs`, `TranscriptLoad`). The engine knows what each load is for. The ten named loads of section 2, their answers and their `busy` flags go.
- **Order is stated, not implied** (`skein/docs/foundation/programming-model.md`, 7.2): store answers keep their arrival order, and a released callback's place among them, as the predicates of `jig/docs/design/domain/root.md`, 5 hold today, with their tests. Pacing loads bounds how many are in flight and keeps no order. What the test root keeps between steps for this (its queue of store answers, counted in its backlog, and its public reads, capped apart) moves into the engine with its order, or stays the root's, by the user's word on one order for every root (section 18).
- **The start through the ports.** The start keeps its script, its phases and their order (`jig/docs/design/domain/engine/restart.md`; `jig/crates/jig-domain-core-vocabulary/src/start.rs`, `Phase`): the core's ranges and inboxes loaded, each connector restored, the links made, the sweep, the store's generation switched, the wires and the start's marker committed before anything leaves after an upgrade, runs adopted, each connector reading afresh and settling its outbox, then open. Each phase's exchange becomes a port's: the engine's own loads through the store port; the generation's switch a store output and the wires a write; the journal's numbering restored an application output (5.5), which the root applies once the decision carrying it is accepted; what waits for the start held by the root's admission, as today; each connector's restoration by `Ask::Restart`, the connector loading its own records. The root's `restart.rs` goes once every phase has a port (`jig/docs/design/domain/root.md`, 8 becomes the engine's alone).
- **The store's reports,** `Fill` and `Census`, are inputs of the port. Commits' answers stay the journal's.

### 5.2 Hosts

- **One vocabulary for every host,** in the core's vocabulary: `host::Down` (assignments, a job's assignment, messages relayed, a call's answer, a read's answer, grants, acknowledgements of turns and answers, cancels, refusals, a turn retried, sources' lists and checks, a list read) and `host::In`, decoded as it comes (section 8): a hello, lost and back, a turn and an answer as summaries with their bodies, a call as an `Act`, a `Read` or a connector call's envelope, a run's live text and tool input, notices (limited, refused, unauthorized), logins, sources' answers, a list's answer.
- **The workers' channel encodes and decodes them** to the records its wire has today (`jig/docs/design/protocol/channel/README.md`), so the wire doesn't change. The hub keeps its own vocabulary, decoded by its worker's side of the channel (section 9).
- **An assignment names its connectors' parts** (section 6.6), which the root puts together as it does today (`jig/docs/design/domain/root.md`, 7).

### 5.3 Parties

- **In:** a sign-in for an identity the application verified, a sign-out, a party's `Act` (section 7) with its key and the value it carried, a `Read`, a watch of a subject or of a run's text, an unwatch, a delivery taken, a public read.
- **Out:** an act's outcome, a read's answer, a delivery, a watch ended or refused, a run's text (now), a result for a person requester, a need arrived, a sign-in refused, a carried value unused, a public answer.
- **An application's own** requests, subjects and reads are acts, reads and watches like jig's, in a scope, decided or made by the connector they name through `Ask::Party` (6.3), their documents parts (6.6) (`jig/docs/design/domain/connectors.md`, 5.5).

### 5.4 Accounts and credentials

The accounts' component's events and requests, as today (`jig/docs/design/domain/accounts.md`, 3.5), and each generation of an application's credential granted (`jig/docs/design/domain/engine/README.md`, 6.1).

### 5.5 The application

What the engine and its application's service say to each other: in, a project's period (`jig/docs/design/domain/engine/README.md`, 4.1), stopping and settling (`jig/docs/design/domain/engine/restart.md`, 1); out, a project made, drained, restarting with its wait, the journal's numbering restored at a start, a brief failed for its log, a restore refused. Today these are spread over `Request::ProjectMade`, `Held::Drained` and `Restarting`, `Now::BriefFailed` and `RestoreRefused`, the start's header the roots apply themselves, and `Event::Period`, `Stopping` and `Settling`.

### 5.6 Resume

One opaque continuation, `Peer::Resume(Resume)`, which the root feeds back as `In::Resume` once released, always taken. It carries what the core asks back after a commit: today's `Held::Relay`, `TaskTerminalAcknowledged`, `TaskTurnKept`, `StopRun` and `PublicWait`, the start's continuations, and the fleet's held events that roots step in their own ways (D004). Its token is the core's flight's id (D002).

## 6. The connector contract

### 6.1 One crate, values only

`jig-domain-contract` holds what the engine and an application's connectors exchange: the declaration, the asks, what is said up, what is released, parts, a run's preparation context, and the operations of section 7 with the values they carry (a batch's members, amendments, cancels, messages, proposals, resource names, grants, budgets).

- **Below the engine and every connector:** it depends on skein-lib and `jig-domain-link` alone, and the core's children and vocabulary depend on it, so the graph stays acyclic. The values it needs move down into it from the children and the core's vocabulary, re-exported along their old paths.
- **Values, not decisions** (`skein/docs/foundation/programming-model.md`, 4.6): it holds no state and decides nothing. A helper that decides, such as `RepairSeed::delegate` (`jig/crates/jig-domain-core-vocabulary/src/effects/connector.rs`), stays with the core.

### 6.2 Declared once

```rust
pub struct Declaration {
    pub number: u16,
    pub resources: Box<[ResourceKind]>,   // a path's shape, and the holds a resource of it may take
    pub effects: Box<[EffectKind]>,       // its resource kinds, form, recovery class, waiting for dependencies, priced
    pub reads: Box<[ReadKind]>,
    pub topics: Box<[TopicKind]>,         // goals' topics among them
    pub symbols: Box<[Symbol]>,           // which it offers, and which tasks may name each
    pub settings: Box<[SettingsKind]>,
    pub parts: Box<[PartKind]>,           // the brief sections, workspace items and documents it serves
    pub projects_goals: bool,
}
```

- **What is the kind's or the deployment's is declared,** never said per instance: an effect kind's form, recovery class and whether it waits for dependencies (`jig/docs/design/domain/connectors.md`, 4.1 and 4.3), today repeated in every `EffectDescription`, taken from the connector's configuration where its system decides it (`ops`'s infrastructure's recovery follows its backend); the holds each kind of resource may take; the order among effect and read kinds that authority holds as data (3.1 there).
- **What depends on a resource or a member is judged and said:** a resource's role and hold in a project, said up as today's `Resource` (a forge branch is shared, exclusive and waiting, or exclusive and refusing, by its adoption); the holds a batch member needs, its symbols resolved, by the one `Holdings` ask for every source (6.3), from the member's spec, executor, home, tracking and parent, refusing with the symbol or the holder.
- **`Engine::new` checks every declaration** and refuses a start that names a kind no connector declared. Roots' installing of hold kinds, by stepping the tasks child, goes.

### 6.3 Asked in the step

| Family | Cases | Today |
|---|---|---|
| `Effect` | describe; settle: keep under a key, keep under a proposal, drop; resolve an uncertain entry; forget a proposal's payload or a staged call's part | `Describe`, `DescribeProposal`, `Keep`, `KeepProposal`, `Drop`, `DropProposal`, `ResolveUncertain`, `Ask::Effect` |
| `Judge` | one requirement at one state | `Judge` |
| `Step` | a procedure made, activated, messaged, closing; told what its step made or why it was refused | `ProcedureMade`, `ProcedureActivate`, `ProcedureMessage`, `ProcedureClose`, `StartProcedure`, `StepMade`, `StepRefused`, and roots' `tasks::Event::WakeProcedure` |
| `Resources` | named, unnamed, held, released, a writer slot, an attempt lost, what a run left | `Names`, `Unname`, `Hold` (twice), `ReleaseHold`, `Writer`, `Lost`, `Left` |
| `Holdings` | the holds batch members need, whoever made the batch | `TaskHoldings`, `DelegateHoldings`, `ProcedureHoldings`, `RepairRefused`, `DelegateRefused` |
| `Topic` | subscribe, unsubscribe | `Subscribe`, `Unsubscribe`, `EndTopic`, `SubscriptionDone`, `UnsubscriptionDone`, `DropSubscription` |
| `Part` | plan a run's parts; gather a run's parts, a document or a read; cut; drop | `Gather` and `CutTo` (twice each), `Drop`, `Prepare`, `Read`, `Document`, `DropDocument`, `Public`, and roots' `BriefCorePlanned` and `WorkspaceRequest` |
| `Run` | a claim made, a job claimed | `ClaimDone`, `JobClaimed` |
| `Project` | adopt, release a deleted project, project a goal, forget a projection, settle a closing task's effects, release what it held | `Adopt` (twice), `ReleaseProject`, `ProjectGoal`, `Project`, `ForgetProjection`, `Close`, `Release` (twice) |
| `Settings` | judge, keep, restore, forget replaced values, save defaults, the models' catalogue | `JudgeSettings`, `KeepSettings`, `KeepProposalSettings`, `DropProposalSettings`, `RestoreSettings`, `ForgetReplaced`, `SaveSettingsDefaults`, `Catalogue` |
| `Party` | decide an application's request, drop it, its core part answered | `Application`, `DropApplication`, `ApplicationPartAnswered` |
| `Restart` | restore your records, read afresh, settle your outbox | `Restore`, `ReadAfresh`, `SettleOutbox` |

Twelve families and about 45 cases, from 73 asks in two enums, each case of one of three kinds:

- **A judgement** (describe, judge, holdings, a part's plan, settings judged, an application's request decided) is answered by `Up` before the step returns. The engine asserts it at the decision's end.
- **A notification** (named, unnamed, held, released, a writer slot, lost, left, told, keep, drop, forget, a claim made) is answered by nothing.
- **An operation that ends later** (a part gathered from the connector's system, a closing task's effects settled, a deleted project released after its commit, a projection settled, a restart's stage) is answered by an `Up` naming it whenever it ends; the engine keeps the state that waits for it, as today.

Hand-offs stay bounded and acyclic (`jig/docs/design/domain/engine/README.md`, 4.3).

### 6.4 Said up

- **Answers:** described (or refused, or busy); a verdict; a step decided (7.4); holdings (or refused, naming the symbol or the holder); a run's parts planned; parts gathered with their sizes, or failed; settings judged; an application's request decided; a restart's stage done; a closing task's effects settled; a project released; a projection settled.
- **Unasked:** an outbox entry's outcome, a task's or a projection's; news, classified for its subscribers; a resource's role and hold in a project; a pool's slots; drift; an adoption; subjects marked changed; an act in the deployment's name.

About as many as today's 19, but one enum, and none of them a copy of a core child's event.

### 6.5 Released after the commit

`contract::Released`: make this outbox entry (`jig/docs/design/domain/connectors.md`, 4.2); this act's answer, under its key. The root feeds each to its connector after the commit, as today (`jig/docs/design/domain/root.md`, 9).

### 6.6 Parts

- **One way to name what the owner keeps** (`jig/docs/design/domain/model.md`, 5): `PartRef { connector, part: Token, size }`, for a brief section, a workspace item, a run's read's answer, a call's connector part, a party's document of an application subject, and a public document.
- **A run's parts are planned, then gathered.** The engine plans a run from a preparation context, a value of the contract: the task and its attempt, its requester and root, its parent and its role among its delegates, its spec, its saved resources, its charter's pinned documents and what it may modify, and the core's own sections with their sizes and priorities and the slots left. Each connector the task names answers with what it would give, its sections and workspace items with their sizes and priorities and the writes, holds and credentials they need, once the holds they wait on are taken. The engine places them within the budget, displacing what is optional, as temper's root does today (`temper/crates/temper-engine-domain/src/assemble.rs`), then asks each connector to gather or cut what it placed.
- **A claim is checked again as it is made:** a part whose connector's holdings changed since its plan is cut, and a run whose task moved is abandoned, as today's preparation waits, pauses and fails (`Now::RunPreparationFailed`).
- **Put together by the root, as today:** an output that names parts is put into the journal with each part handed over by its connector's own `hand_over`, which returns the connector's own type (`jig/docs/design/domain/root.md`, 7). That is one function in a root, not one per kind of output.
- **What goes:** `BriefCorePlanned`, the gathers, `CompleteBrief`, `WorkspaceRequest`, `Prepare`, `WorkspacePrepared`, `RunPrepared` and `ClaimPrepared`, and the fixed answers roots keep for them (`jig/docs/design/domain/root.md`, 14).

### 6.7 What a connector writes

A connector's own records, effects and outputs stay its own types. By convention its output has the engine's four marks, `Up` in place of `Ask`: `Up(contract::Up)`, `Write`, `Held`, `Now`. Its input is `Ask(contract::Ask)`, `Released(contract::Released)` and its system's events. So a root's arm for a connector reads as its arm for the engine (section 11).

## 7. One set of operations

### 7.1 Act, Actor, Op

```rust
pub struct Act { pub by: Actor, pub ops: List<Op> }

pub enum Actor {
    Party { sign_in: u64, key: [u8; 16], reply: ReplyTo },
    Run { task: u64, attempt: u64, call: CallKey, reply: ReplyTo },
    Procedure { task: u64, step: u64 },
    Deployment { connector: u16, key: [u8; 16], reply: ReplyTo },
}
```

- **`Op`, by group:** work (create a batch, delegate into a gathering, conclude, message, introduce, amend, cancel with a grace, release a hold, stop, move to oneself, move to the newest settings, leave, lift); proposals and decisions (propose an action, a batch, a goal or settings; withdraw; decide a proposal or an escalation); person tasks (take, hand back, answer); a read position (seen); topics (subscribe, unsubscribe); notes (write, edit lines, delete); a project's own (make, rename, remove, its policy, roles, pools, visibility, adopt, make a service); settings and models; LLM accounts; and the application's own, named by its connector. Each names the revision it was decided on, as today (`jig/docs/design/domain/model.md`, 11).
- **An act holds the shapes its source makes today, and no others:** a party's compound request, task operations in one project within operations per request, each admitted against the state the earlier ones staged (`jig/docs/design/domain/people/acts.md`, 1), or one operation; a run's call, one operation; a procedure's step, its decision (7.4). A wider combination, across owners or with settings, accounts or an application's requests, is new semantics a later design would have to make atomic first.
- **"A party does what an agent does"** (`jig/docs/design/domain/model.md`, 9) becomes a type: the operations are one, the actor differs, and authority checks one shape. Which actor may make which operation is authority's table: only parties change settings, make a project's and accounts' operations, and take, hand back or answer a person task; a run may propose settings, as `NamedAction::ProposeSettings` does today; procedures decide nothing (`jig/docs/design/domain/people/acts.md`, 1).
- **Keys keep their meaning.** A party's act under a key it used before is the same act or a conflict, judged against the retained record, which keeps what makes two equal, as today's keeps the original ask; `Seen` keeps its own rule, an answer the request table doesn't retain (`jig/docs/design/client/domain/contract/requests.md`, 1).
- **Where each comes from:** a party's act from the clients' component; a run's from its tool call, decoded at the edge (section 8); a procedure's from its step, in `Up` (6.4); the deployment's from a connector (`jig/docs/design/domain/people/acts.md`, 3), or from the application itself for a person's task in no project or a recurring procedure.
- **Today's three spellings go:** `people::Ask` and its families, `NamedAction`'s operations, and `ProcedureStep`'s lists.

### 7.2 Reads

A run's reads are a party's, read as its task's requester or as itself: one `Read` for a task's text, settings, a proposal, a person's documents, notes recalled and an application's read, with one answer. Today: a party's `reads::Read` and a run's `NamedAction::ReadTask`, `Settings`, `PersonRead`, `ReadProposal` and `Recall`, with seven answers (`Now::TaskRead`, `SettingsAnswer`, `PersonAnswer`, `ProposalRead`, `NotesIndexed`, `NotesRecalled`, `NotesRefused`).

### 7.3 Outcomes and refusals

Each operation has its outcome, and a refusal one shape whatever its source: the operation it stopped at, why (moved, with what it is now; denied, with authority's findings and whom a proposal could go to; invalid; busy; unknown), and the facts a client or a run shows (`jig/docs/design/client/domain/contract/README.md`, 3). Today's `people::Outcome`, `StepRefusal`, `tasks::Problem`, `EffectRefusal`, `HoldingsRefusal` and `WatchRefusal` are its cases or go.

### 7.4 A procedure's step

A procedure's decision is an envelope around its operations, as `ProcedureStep` is today (`jig/crates/jig-domain-core-vocabulary/src/runs/delegation.rs`): the words its spec takes, the dependencies its work started from, which no amendment removes (`jig/docs/design/domain/connectors.md`, 6.1), its operations, and its end: a result, a hold and what it holds from, or waiting. Only the operations become `Op`s; the envelope stays the step's.

## 8. Decoded at the edge

- **Structure is decoded on the way in, all of it** (`skein/docs/foundation/programming-model.md`, 4). The engine's protocol layer decodes a run's call (its tool and input, with `jig-protocol-tools` and the application's own codecs) and a turn's and an answer's summary (size, spend, the last message read, whether it opens a window, messages refused, saved state) as they arrive. The engine keeps a turn's body as bytes it stores and never reads (`jig/docs/design/README.md`, 9).
- **A call to jig's tools** arrives as an `Act` or a `Read` by `Actor::Run`. **A call to a connector's effect or read** arrives in two parts: its envelope (the run, the attempt, the call's key, the connector, a token) to the engine, and its typed request to that connector, kept under the token.
- **Fencing follows decoding** (section 14.2), and keeps what binds a call today: its owning attempt, its name within its length and the engine's room for it (`jig/crates/jig-domain-fleet/src/call.rs`), the tool it names and its contract (`jig/crates/jig-protocol-charter/src/call.rs`), its input within its bound (`temper/crates/temper-engine-service-vocabulary/src/tools.rs`). The protocol layer allocates nothing past those bounds, and a call that doesn't decode arrives as its failure, a value, and is refused as today.
- **Every path disposes of what it decoded.** A connector call's part is staged inert: only a fresh, admitted call has it described and kept. A busy, duplicate, malformed, withdrawn, stale or lost call's part is forgotten by its token (`Ask::Effect`). A call asked again is answered from its record before anything runs (`temper/crates/temper-engine-domain/src/route/host_route.rs`), and uses the value its owner already keeps.
- **A settled call keeps its rendered bytes.** The protocol layer renders a call's answer once, and the bytes come back as an input of the host port, which the engine keeps in the call's record (`jig/crates/jig-domain-core-vocabulary/src/runs/conversation.rs`), so a call asked again, across a redeploy too, gets the bytes it was first given.
- **Charters' sizes are checked at configuration** by their encoded bound (`skein/docs/foundation/programming-model.md`, 6.3), so no root checks an assembled charter before a claim, as `ops`'s does today (D004).
- **What goes:** `Now::Call`, `Now::Ask`'s bytes, `LinkNow`'s payloads (`DropPayload`, `TurnPayload`, `AnswerPayload`, `AcceptedTurn`, `RefusedPayload`), `Event::NamedAction`, `NamedAnswer`, `TurnPayload`, `AnswerPayload`, `AcceptedTurn` and `RefusedPayload`, and the payload tables roots and `jig-domain-hub-link` keep for them. `LinkNow::Asked` and `Relayed`, a read's answer to a host, become `host::Down`.

## 9. Hosts and workers

- **The engine's own slots become a worker of their own,** in the engine's process: a worker root composing the hub and `smith-inline-agent`, with no workspace, under its own protocol layer, joined to the engine by an in-process address, as one process already runs an engine with its workers, from the same parts as between hosts (`jig/docs/design/protocol/service.md`, 10; `jig/docs/design/domain/hosts/README.md`, 3). The engine's root then holds no hub, and the fleet reaches every host one way: through the host port and the workers' channel.
- **`jig-domain-hub-link` goes.** Its job is translating between the fleet's and the hub's vocabularies inside the engine's root (`jig/docs/design/domain/root.md`, 5). Its tables (payloads, call routes, assignments) and its turns' retry deadlines each move first to where a worker's path keeps them, the worker's side of the channel or the hub, and only then does the crate go.
- **The price** is the channel's encoding, opening and liveness for every message between the engine and its own slots, which a one-process deployment already pays for its workers. Agents in the engine keep their shape's promises (`jig/docs/design/domain/hosts/README.md`, 3): no workspace, the engine's tools and the connectors' reads, LLM traffic through the engine's process.
- **The hub's agent face stays:** smith's parent face, its payloads bytes in the hub and typed in the agent, by hand-out to that worker root's protocol layer (`jig/docs/design/domain/hosts/hub.md`, 8), as on any worker whose root composes `smith-inline-agent`.
- **A worker's root** composes the hub, smith's agent capability and its workspaces, as today (`jig/docs/design/domain/hosts/README.md`, 4). Its channel's wire doesn't change.

## 10. Inside the engine

- **The children are private.** No production crate but the core, its vocabulary, the contract and the children depends on a child's crate; their values reach every other crate through the core's vocabulary, re-exported along its module paths (`skein/docs/foundation/programming-model.md`, 4.6), and what the protocol, store, configuration, charter and client crates name is listed as it moves. Each child's own test kit and world keep stepping it, as its world must (`jig/docs/design/domain/testing.md`, 3). `jig-boundary` checks it. The escape hatch of `jig/docs/design/README.md`, 4 goes (section 14.3).
- **What roots do with the children moves inside or to a port:** hold kinds to the declarations; the tasks events roots build (`HoldFrom`, `WakeProcedure`, `Hold`, `Cancel`, `SubscribeTopic`, `Settled`, …) to `Up` or to the engine's own routes; the people's and the fleet's events to their ports.
- **Run preparation is the engine's pipeline:** a task due, its window loaded, its parts planned with each named connector (6.6), gathered, its claim checked and committed, its assignment held. The root takes no step of it.
- **Its own work is its own type,** apart from `In`, and its continuations are one typed lookup, a flight's id, with no map keyed by task standing for one (D002).
- **`Core`'s state is grouped by concept,** each part with its own `impl` in its own module (D003).
- **A task's record is its common fields and one executor part:** agent, procedure, person or job (D014). It is a registered family's record, so it changes as `AGENTS.md`, 2 says a registered schema changes: an added field with its default, a renamed part with its former name, and a change no plan derives as its owner's written job (`jig/docs/design/protocol/store/layouts.md`).

## 11. The root

```rust
pub fn step(root: &mut Domain, env: &Env<Limits>, event: Event) {
    let mut decision = root.journal.decide();
    let mut out = Queue::uncapped();
    match event {
        Event::Jig(input) => core::step(&mut root.engine, &env.of(env.limits.engine), input, &mut out),
        Event::Observability(input) => observability(root, env, input, &mut decision, &mut out),
        Event::Infrastructure(input) => infrastructure(root, env, input, &mut decision, &mut out),
    }
    for _ in 0..env.limits.routes {
        let Some(output) = out.pop() else { break };
        match output {
            core::Out::Write(write) => decision.push_write(Write::Jig(write)),
            core::Out::Ask(number, ask) => match Connector::of(number) {
                Connector::Observability => observability(root, env, observability::In::Ask(ask), &mut decision, &mut out),
                Connector::Infrastructure => infrastructure(root, env, infrastructure::In::Ask(ask), &mut decision, &mut out),
            },
            core::Out::Held(peer) => decision.push_held(assemble(root, peer)),
            core::Out::Now(peer) => root.journal.now(assemble(root, peer)),
        }
    }
    assert!(out.is_empty(), "a decision's hand-offs end within its routes");
    root.journal.accept(decision);
}

fn observability(root: &mut Domain, env: &Env<Limits>, input: observability::In,
                 decision: &mut Decision<Write, Output>, out: &mut Queue<core::Out>) {
    let mut said = Queue::uncapped();
    observability::step(&mut root.observability, &env.of(env.limits.observability), input, &mut said);
    for _ in 0..env.limits.routes {
        let Some(output) = said.pop() else { break };
        match output {
            observability::Out::Up(up) => core::step(&mut root.engine, &env.of(env.limits.engine),
                                                     core::In::Connector(OBSERVABILITY, up), out),
            observability::Out::Write(write) => decision.push_write(Write::Observability(write)),
            observability::Out::Held(output) => decision.push_held(Output::Observability(output)),
            observability::Out::Now(output) => root.journal.now(Output::Observability(output)),
        }
    }
}
```

- **After the commit,** `iterate` sends released outputs by peer: `Peer::Resume` back to the engine, `Peer::Connector` to its connector as `In::Released`, every other to the protocol layer (`jig/docs/design/domain/root.md`, 3).
- **What is left to an application per connector:** one arm in its event, one in the asks, one in release, one variant in its store's `Key`, `Record` and `Write` and its `Output`, and the translation of the contract if the connector keeps a vocabulary of its own. `admission.rs`'s classes, `restart.rs`, `link.rs`, `assemble.rs`'s kinds and `client_documents.rs`'s loop go from `jig/docs/design/domain/root.md`, 12's tree, and so do the hub and `smith-inline-agent` from its section 2's (section 9).
- **What a root never does** stays as written (`jig/docs/design/domain/root.md`, 11), and now holds by construction: it has nothing of jig's to decide with.

## 12. The conformance kit

The kit plays jig's peers itself (`jig/docs/design/domain/testing.md`, 5): the store on skein-kv's in-memory mode, scripted hosts, scripted parties, the accounts' component. Since every one is a port, it finds them in an application's outputs through one function the application writes, from its `Output` to `Option<&core::Peer>`, and one into its `Event` from `core::In`. With the root's step and release and its systems' fakes, that is the adapter: an estimate of about five functions in place of D030's nineteen, which the kit's milestone measures, since today's also carry a cold start's observation and the store's evidence.

## 13. Where today's edge goes

| Today | Where it goes |
|---|---|
| `Event`: `ReadLoaded`, `TranscriptLoaded`, `SeenConversationLoaded`, `EndedReadLoaded`, `DecidedProposal`, `EndedProposal`, `EndedTaskLoaded`, `PartyInputsLoaded`, `EndedPersonTask`, `EndedEscalation`, `DelegateValidated`, `DelegateInputFailed`, `Notes(..)`'s pages and failed loads | `In::Store(Loaded)`, its typed result (5.1) |
| `Event`: `Fill`, `Census`; `Tasks(Restore)`, `Tasks(Restored)`, `People(Restore)`, `SettingsCommit` | `In::Store` and the start's phases (5.1) |
| `Event`: `Fleet(..)`, `RunCall`, `RunInput`, `RunText` | `In::Host` (5.2) |
| `Event`: `TurnPayload`, `AnswerPayload`, `AcceptedTurn`, `RefusedPayload`, `NamedAnswer`, `SettledCall` | decoded at the edge: `In::Host`'s turns, answers and calls, a settled call's rendered bytes (section 8) |
| `Event`: `NamedAction`, `SubscribeTopic` (its read resources checked by authority), and `Notes(..)`'s writes, edits, index and recalls | `Act` or `Read` (section 7) |
| `Event`: `SignIn`, `Read`, `Watch`, `WatchRun`, `Unwatch`, `Delivered`, `StreamDelivered`, `PublicRead`, `People(..)` | `In::Party` (5.3) |
| `Event`: `Key`, `ChangeModel`, `ChangeSettings`, `ProposeSettings`, `Application` | `Act` by `Actor::Party` (7.1) |
| `Event`: `Act`, `PersonTask`, `StartRecurring` | `Act` by `Actor::Deployment` (7.1) |
| `Event`: `EffectConnector(..)`, `EffectStart`, `ProcedureStep`, `Holdings`, `DelegateHoldings`, `ProcedureHoldings`, `Marked`, `View(..)`'s documents made and subjects changed, `ApplicationPart`, `ApplicationFinished`, `ProjectionSettled`, `ProjectReleased`, `PublicAnswered`, `Brief(..)`, `BriefAssembled`, `WorkspacePrepared`, `ClaimPrepared`, `ClaimRefused`, `ConnectorRepair` | `In::Connector(Up)` (6.4), or gone with run preparation (6.6) |
| `Event`: `Account(..)` | `In::Account` (5.4) |
| `Event`: `Period`, `Stopping`, `Settling` | `In::Application` (5.5) |
| `Event`: `PublicDurable`, and `Tasks(..)` and `Fleet(..)` callbacks fed back | `In::Resume` (5.6) |
| `Event`: `Activate`, `EffectDeadline`, and `Tasks(..)` that roots build | inside: scheduled by the engine, fired by `fire`, or the declarations and `Up` (6.2, 6.4) |
| `Request::Write`, and the start's wires | `Out::Write` (4.3, 5.1) |
| `Request::Ask` and `effects::connector::Ask` | `Out::Ask` and its twelve families (6.3) |
| `Held`: `Load`, `ReadLoad`, `NotesLoad`; `Now`: `StartPreparation`, `SeenConversation`, `LoadEndedRead`, `EndedProposal`, `DecidedProposal`, `EndedPersonTask`, `EndedTask`, `PartyInputs`, `EndedEscalation`; the start's generation switch | `Peer::Store` (5.1) |
| `Held`: `Assign`, `ReadList`, `Sources`, `CheckSource`, `SignOutSource`, `Grant`, `Relayed`, `Inbound`, `Acknowledge`, `AcknowledgeTurn`, `Cancel`, `Refuse`, `TurnBusy`, `SettledCall`, `CallAnswer`, `NotesWritten`, `NotesDeleted` | `Peer::Host` (5.2) |
| `Now`: `RelayBusy`, `CallBusy`, `DropCall`, `Undelivered`, `EffectAnswer`, `SettledCallRefused`, `NoteBusy`, `Link(Asked)`, `Link(Relayed)`, and the reads' answers `TaskRead`, `SettingsAnswer`, `PersonAnswer`, `ProposalRead`, `NotesIndexed`, `NotesRecalled`, `NotesRefused` | `Peer::Host` (5.2), the reads' answers one (7.2) |
| `Now`: `Call`, `Ask`, and `Link`'s `DropPayload`, `TurnPayload`, `AnswerPayload`, `AcceptedTurn`, `RefusedPayload` | gone: decoded at the edge (section 8) |
| `Held`: `Acted`, `Read`, `Delivery`, `WatchEnded`, `PeopleReply`, `Result`, `PublicRead`, `NeedArrived`, `CarriedUnused`, `SourceSignedOut`, `ApplicationDelivery`, `ApplicationWatchEnded`, `Applied`; `Now`: `SignInRefused`, `WatchRefused`, `RunText` | `Peer::Party` (5.3), an application's document as a part (6.6) |
| `Held::MakeEffect` | `Peer::Connector(Released)` (6.5) |
| `Held`: `Relay`, `TaskTerminalAcknowledged`, `TaskTurnKept`, `StopRun`, `PublicWait` | `Peer::Resume` (5.6) |
| `Now::Account` | `Peer::Account` (5.4) |
| `Request::ProjectMade`; `Held`: `Drained`, `Restarting`; `Now`: `BriefFailed`, `RestoreRefused`; the start's header | `Peer::Application` (5.5) |
| `Now`: `Activate`, `CompleteBrief`, `BriefCorePlanned`, `WorkspaceRequest`, `RunPreparationFailed`, `RunPrepared`, `DropAssignment` | gone: run preparation inside (section 10), connectors' parts planned and gathered (6.6) |
| the 60 core methods temper's root calls, and `ops`'s and the test root's | an input of a port, an output, or inside (4.5) |
| roots' access to `core.tasks`, `core.people`, `core.fleet` and the other children | gone (section 10) |
| the hub and `jig-domain-hub-link` in the engine's root | gone: the engine's own slots a worker (section 9) |

## 14. Departures

Each departs from a rule or from jig's design, and each is the user's call, made on 2026-11-17 as this draft recommended. The milestone that carries one writes the change into the documents named.

### 14.1 A contract crate shared by siblings

Siblings share no types, and their parent translates between them (`skein/docs/foundation/programming-model.md`, 4.5). jig's own contract with connectors is then translated by every application's root, in both directions. `jig-domain-contract` (6.1) is shared instead, as `jig-domain-link` already is (`jig/docs/design/domain/root.md`, 2). The wording 4.5 gains: **A kit's contract.** A kit may give siblings a crate of values that both name: below each, depending on lib and other vocabularies alone, holding no state and deciding nothing; a change to it breaks every sibling's build, on purpose. jig's are `jig-domain-link` and `jig-domain-contract`. 4.6's "A sibling domain doesn't" gains "but through a kit's contract". `jig/docs/design/domain/root.md`, 6's translation of the core's connector vocabulary goes; a connector that keeps its own vocabulary translates inside itself.

### 14.2 Decoding before fencing

Today the fleet admits a call from its task's live attempt and only then hands it out to be decoded (`jig/docs/design/domain/engine/README.md`, 4.4; `jig/docs/design/protocol/channel/README.md`, 5.3). The protocol layer decodes every call on the way in instead (section 8), within the same bounds, and the engine fences what it decoded. A stale attempt's call costs its decoding, and the round trip through the root goes.

### 14.3 Private children, no escape hatch

`jig/docs/design/README.md`, 4 lets an application compose the core's children without the core. No application does, and the children's public crates are how roots came to reach into them. They become the engine's alone (section 10).

### 14.4 One class per input

A system's event may finish admitted work and bring new work at once, and the root splits it, the connector capturing a baseline for the rest before the settlement (`jig/docs/design/domain/root.md`, 3). Instead the connector's protocol layer delivers the two as two events, the settlement first, and the connector keeps in its own state, per resource, the observation the news will be judged against, set where today's baseline is captured, so the news event carries none. Its milestone states what that observation is at first, what the connector's own writes do to it, how two settlements before one news coalesce, and how a repeated absence still tells the tasks that rely on it without another deletion record, keeping the outcomes of `ops`'s three mixed events (`HealthChanged`, `ServiceDrift`, `EnvironmentDrift`) and their stories. `Class::Mixed`, the split and the baselines in the events go.

### 14.5 The root copied, not shipped

A root's shape is the same in every application, but jig can't ship it, since step code has no generic items outside lib (`skein/docs/foundation/programming-model.md`, 10.3), and a role for shared generic mechanism is an open question of skein's (`skein/docs/design/draft/component-kit.md`, 6.2). With section 11's root, what is copied fits on a page, so roots stay copied from `ops`'s (`jig/docs/design/domain/root.md`, 12) and this draft needs no new role.

## 15. Getting there

- **The engine first, as a pass-through every root adopts at once.** Its hard parts are what it takes from the roots (loads, the start, run preparation, parts), which the three roots do differently today (D004). Built alone, it would be checked by nothing. Built as a pass-through, `In::Legacy` and `Out::Legacy` carrying today's events and requests and a temporary access to the whole core (4.5), it changes no behaviour, and every later step is a real move checked by every root's own suites.
- **Then port by port.** Each milestone adds a port's variants, moves into the engine what the roots do for it, moves the three roots and the protocol pieces that feed it, and removes its cases from `Legacy` and from the access. The cheapest and most mechanical first: the store's loads, the contract's crate, the declarations, the asks, the start, a run's parts, its workspace and claim, the engine's own slots, decoding at the edge, the operations and then their records, the remaining ports, admission.
- **Then the clean-up.** `Legacy` and the access go; the core emits the ports itself and the old vocabulary goes; the children become private; the debt the narrow edge unlocks is paid (D002, D003, D014, D017, D030); the design documents are rewritten to the new edge and `ops`'s root is copied again as the reference.
- **No flag day.** Everything moves inside one process and one repository. No wire between the engine, its workers and its clients changes: the engine's own slots use the channel's wire as it is. The stored records that change (a task's record, D014, and a party's retained requests, 7.1) change as `AGENTS.md`, 2 says a registered schema changes. One goal can move a port in jig and in all three roots together, since jig and temper are trees of one repository.
- **Each milestone is one goal of one concern,** lands whole, writes its own design into jig's documents and their status as it runs, and keeps the focused and fuzzy suites green. The execution file lists them, with their order.

## 16. Counts that may only fall

Measured on main before the first milestone, each with the command that counts it, recorded in the execution file, as skein's component-kit draft does (`skein/docs/design/draft/component-kit.md`, 4). Each falls with the milestones, and none may rise:

1. variants on the core's edge that a root matches, `Legacy`'s included, against the ports' once the pass-through has landed;
2. `Legacy`'s variants, to 0;
3. distinct core methods called outside `jig-domain-core` and its tests (60 in temper's root at `fb973d1173`), to 0;
4. accesses to the children's state outside the core (`core.tasks` and the rest: 389 in the three roots, 338 in temper's), to 0;
5. children's events built outside the core (69, 13 and 14 in temper's, `ops`'s and the test root), to 0;
6. production manifests outside the core, its vocabulary, the contract and the children that name a child's crate (67 manifests today, its test kits among them, which stay), to 0;
7. `jig-domain-hub-link`'s lines, to 0;
8. the lines of the three roots outside tests, and of temper's wiring files, read at each landing;
9. the conformance kit's `Application` functions (19), to about 5;
10. `Core`'s fields (92 at `422be3b57f`) and `TaskRecord`'s (89), by D003's and D014's measures.

## 17. What stays

The five primitives and the model (`jig/docs/design/domain/model.md`); one decision, one commit, through skein-lib's journal, the root's only way out (`jig/docs/design/README.md`, 6.2 and 6.3); the marks; restart as the core's script and its order (`jig/docs/design/domain/engine/restart.md`); the owner keeping the value and the root putting outputs together; hand-offs within a decision, bounded and acyclic; recovery classes, verdicts, holds and the outbox (`jig/docs/design/domain/connectors.md`); plumbing and sugar, and the client's API (`jig/docs/design/domain/model.md`, 9); level-triggered views; settings as defaults with changes over them; agents in the engine's process, with no workspace; the root owned by the application, with no traits, generic items, closures or `dyn` in step code (`jig/docs/design/README.md`, 4).

## 18. Open questions

- **One order for every root** (D004): the order of release, store answers and public reads, which the roots keep differently and the user deferred on 2026-11-06. This plan meets it where the engine takes over loads, the start and run preparation. Default: `ops`'s order for every root; what the test root keeps between steps for store answers and public reads moves into the engine with its order, its count in the backlog and the public reads' own cap; public reads stay behind their own watermark (`jig/docs/design/domain/root.md`, 3).
- **The hub's agent face** (section 9): smith's payloads cross between the hub and an inline agent by hand-out, now inside the engine's own worker root as on any worker. Default: as it is; a later draft may decode them at the edge too.
- **Connectors that keep their own vocabulary:** `ops`'s and temper's translate today. Default: each moves onto the contract where the move is mechanical, the rest keep their translation, counted, and new connectors speak the contract.

## 19. Reasons

Kept here, since `temper/docs/design/draft/notes.md` is at its size limit.

- **What the user asked** (2026-11-17): "this session is about jig domain layer only, protocol and client set aside for now. I have the impression that it grew organically, and would like to see what we can do about it", and first, "given the functionality that jig offers, what would be its ideal API ... If we were to design it greenfield ... Still in line with skein's programming-model.md and still in line with the kit idea".
- **The shape** of sections 4 to 12 was the first answer; the user took its five departures (section 14) as recommended: "okay I'm convinced".
- **The way there** was the user's: "Create this new API as a facade over existing code, with no consumer yet, then slowly migrate all consumers ... and then cleanup making the facade the real API". The draft changes one step, giving the facade every root at once as a pass-through (section 15), since the parts it must absorb differ per root and a facade with no consumer is checked by nothing.
- **The review round** (one, 2026-11-17) found fifteen gaps, all taken: the start's own exchanges (5.1); typed load terminals and stated orders (5.1); timers' admission and clocks (4.1); declarations narrowed to what is static, with holdings and roles still judged (6.2); asks of three kinds (6.3); a run's preparation context, planned then gathered, checked again at its claim (6.6); acts only in today's shapes, procedures' envelopes, actors' permissions and keys (7.1, 7.4); a settled call's rendered bytes kept (8); disposal on every path of a decoded call (8); the crate graph kept acyclic (6.1); `jig-domain-hub-link`'s tables moved before it goes (9); the stored-record rule of `AGENTS.md`, 2 (10, 15); the children's own worlds (10); the map's holes; and the order's splits.
- **The draft's readings,** not the user's words: the engine in `jig-domain-core` rather than a crate of its own, so that no dependent changes for a name (4.1); the operations' values in the contract crate, so that a procedure's author names jig's operations as a party's client does (6.1); the engine's own slots as a worker in its process, so that every host is reached one way, the crate graph stays acyclic and `jig-domain-hub-link` has nothing left to do (section 9); a connector's own observation for news, so that a settlement and its news can arrive apart (14.4); and the defaults of section 18.
