# jig-domain-api: one engine, a few ports

Draft, 2026-11-17, brought up to date on 2026-11-26. jig's domain grew organically. Its model is sound, but what an application meets is jig's internal pipeline: some 200 kinds of message on the core's edge, sorted by timing before destination, a second vocabulary for connectors, three spellings of the same operations, and roots that reach into the core's children. This draft gives jig's domain the public API one would want to find when starting a project on jig: jig is the core and takes care of the work, and connectors are built on top of it, consuming it and adding to it, each concern on its own side of one contract. It stays in line with skein's `skein/docs/foundation/programming-model.md` and jig's kit idea (`jig/docs/design/README.md`, 1), and says how to get there from today's code without a flag day. It covers the domain layer only: the protocol layer and the client change only where they build or read the engine's messages. Its execution file is [jig-domain-api.execution.md](jig-domain-api.execution.md), its reasons [jig-domain-api.notes.md](jig-domain-api.notes.md).

## 1. In one page

- **Building on jig is writing connectors and a page of root** (section 3). A connector depends on one crate of values, the contract, and on nothing else of jig's. A root composes the engine with its connectors and fits on a page. The protocol layer reads and writes the engine's ports.
- **jig decides about work, a connector about its system** (3.2). jig keeps tasks, their executors, messages, authority, parties, runs, hosts, accounts, waits, holds, scheduling, restart and its own records. A connector keeps its system: its resources, facts, effects, news, what an agent sees of it, and its procedures' steps. A connector adds to jig by declaring its kinds, and uses jig by acting through the same operations as a party.
- **The model stays** (section 17): the five primitives, one decision and one commit through lib's journal, outputs marked by when they may leave, the owner keeping the value, restart as the core's script, plumbing and sugar, level-triggered views, recovery classes and verdicts.
- **One engine, a few ports, named for their peers** (sections 4 and 5). `Engine` has `step`, `due`, `fire`, `next_deadline` and `idle`, and nothing else public. What comes in is `In`, by peer: the store, hosts, parties, connectors, accounts, the application, and its own resumptions. What goes out is `Out`, by mark first (a write, a connector's ask, held until the commit, or now) and then by peer.
- **A connector takes part by capability** (section 6). It declares its kinds once, in one `Declaration`, and the engine asks it only about the capabilities it declared: its effects, its resources, its topics, its parts, its procedures, its projections, its settings, its application's requests, and its own lifecycle, which every connector has. Each capability is a module of the contract crate, `jig-domain-contract`, with its asks and what the connector says up, so a connector's author reads only the modules it uses.
- **One set of operations for every actor** (section 7). A party's compound request, a run's tool call and a procedure's step are each an `Act`: an actor and its operations, in the shapes each source makes today, checked by one authority and refused in one shape.
- **One mechanism per concern** (sections 5.1, 6.6 and 10): one load with one typed terminal, paced by the engine; one way to name a part a connector keeps; one opaque continuation; run preparation inside the engine.
- **Decoded at the edge** (section 8): calls and turns are decoded by the protocol layer on the way in, every path disposing of what it decoded, so nothing goes from the domain to the protocol layer and back mid-decision.
- **One way to every host** (section 9): the engine's own slots become a worker of their own in the engine's process, as one-process deployments already run their workers, so the engine's root holds no hub and `jig-domain-hub-link` goes.
- **The core's children are private** (section 10). Their values reach others through the core's vocabulary; no production crate steps them, holds them or names their crates.
- **A root's jig wiring fits on a page** (section 11) and grows only with its connectors, which `jig/docs/design/README.md`, 1 already promises. The conformance kit answers jig's own ports itself (section 12).
- **Getting there** (section 15 and the execution file): the engine first as a pass-through every root adopts at once; then port by port, each milestone moving all three roots; then the clean-up that makes the engine the real API. Counts that may only fall measure it (section 16).

## 2. What is there today

Counted on main at `de275134b0` (2026-11-26), outside tests unless said, by the commands in [the notes](jig-domain-api.notes.md), with the draft's first count at `422be3b57f` (2026-11-17) in brackets where it moved:

- **The core's edge** (`jig/crates/jig-domain-core-vocabulary/src/vocabulary.rs`): `Event`, 76 variants in [72]. `Request`, out: `Ask` 38 [34], `Held` 46 [41], `Now` 40 [39], `LinkNow` 8 [7], `Write` 5. The connector handoffs added another 38 [39] asks and 19 answers at that revision; these now use the contract's `Ask` and `Up`. Children's enums cross it whole: `tasks::Event` 93 [86], `fleet::Event` 48 [38], `people::Event` 20 [19] among them. A week's work on waits, worker restarts and saved parts added 22 variants to the edge, each one a root had to route by hand.
- **Sorted by timing, then destination.** A host's outputs, a party's, the store's and a connector's are mixed in `Held` and `Now`, so a root must know each of 86 variants [80] to send it where it goes.
- **The same verbs spelt three ways:** a party's `people::Ask` (41: create, delegate, propose, withdraw, stop, cancel, release, amend, move, decide, pause a worker…), a run's `NamedAction` (23: message, amend, release, decide, propose, mark read…, and five reads), and a procedure's `ProcedureStep` (batch, amendments, cancels, messages, proposals).
- **Ten loads by name:** `Now::StartPreparation`, `SeenConversation`, `LoadEndedRead`, `EndedProposal`, `DecidedProposal`, `EndedPersonTask`, `EndedTask`, `PartyInputs`, `EndedEscalation`, `Held::NotesLoad`, beside `Held::Load` and `ReadLoad`, each with its own answer, most answers carrying a `busy` flag the root fills.
- **Run preparation through the root:** `Now::Activate` and `Event::Activate { ready, busy }`, `StartPreparation`, `BriefCorePlanned`, `CompleteBrief`, `WorkspaceRequest`, `RunPrepared` (now carrying a resumed run's kept parts), `ClaimPrepared` and `take_due`, with roots answering some of them with fixed values (`jig/docs/design/domain/root.md`, 14).
- **Hand-outs:** a run's call goes out to be decoded and comes back typed (`Now::Call`, `Event::NamedAction`); turns', answers' and saved parts' payloads stay with the root and come back translated (`LinkNow`, `Event::TurnPayload`, `AnswerPayload`, `AcceptedTurn`, `RefusedPayload`, `PartPayload`); a settled call's answer is rendered and handed back (`Event::SettledCall`).
- **The boundary leaks.** temper's root (`temper/crates/temper-engine-domain`) calls 67 [60] distinct core methods besides the step, the three roots 73. They reach the core's children's state 438 times [389] (`core.tasks`, `core.people` and the rest), temper's 385, and build the children's own events 108 times [96], temper's 77 (`tasks::Event::HoldFrom` 12, `WakeProcedure` 13, …). `ops`'s reference root steps `jig_domain_tasks::step(&mut core.tasks, …)` itself to install hold kinds. 27 production manifests [20] outside the core, its vocabulary and the children name a child's crate, 77 with test kits [67].
- **So roots are large and diverge.** `ops`'s root is 3,671 lines, the test root 4,762, temper's 34,535, about 13,000 of them in files that by their names are jig's wiring, and temper's crate sits just under its 60,000-line bound with its tests. The roots keep different shapes and orders (`jig/docs/development/debt/D004-ops-root-jig-wiring.md`), and the conformance kit needs an adapter of 26 functions [20], 376 to 738 lines per adopter (`jig/docs/development/debt/D030-conformance-not-drop-in.md`).
- **Inside:** `Core` is one object of 99 fields [92], 83 when `jig/docs/development/debt/D003-core-one-object.md` last measured it; its continuations cross the edge as bare tokens (`D002-core-untyped-continuations.md` there); and `TaskRecord` holds every executor's state in 96 fields [89], 78 at `D014-task-record-union.md`'s check.

## 3. Building on jig

What an application's author meets, in the order they meet it (`jig/docs/design/README.md`, 11).

### 3.1 Who writes what

| You write | You depend on | You meet |
|---|---|---|
| a connector, one per external system | `jig-domain-contract` alone | its declaration, the asks of what it declared, what it says up and what is released to it (section 6); the operations it may make (section 7) |
| the engine's root | `jig-domain-core` and your connectors | the engine's five functions, its four marks, one arm per connector (sections 4 and 11) |
| the protocol layer | `jig-domain-core-vocabulary` and jig's service parts | the ports' values, to encode and decode (sections 5 and 8) |
| a worker's root, if agents run on workers | the hub and smith's agent | the workers' channel, unchanged (section 9) |
| your worlds | jig's conformance kit | two functions between your root's types and jig's ports (section 12) |

### 3.2 The line between jig and a connector

| jig decides | A connector decides |
|---|---|
| what work there is: tasks, their executors, messages, delegation, their lifecycle; what each waits on and until when; holds and their release | what its system is: its resources' names and roles, the facts about them and the verdicts on its own requirements |
| who may do what: authority, parties and their acts, proposals, escalations, person tasks | what its effects are, made once from the outbox, and what became of each |
| when work runs: admission, scheduling, runs on hosts, accounts, restart | what news matters, classified for its subscribers |
| what is kept: jig's records, in one commit with the application's | what an agent sees of its system: brief sections, reads, workspace items, documents |
| | how each of its procedures steps, and what it waits on |

- **A connector adds to jig** by its declaration (6.2): its kinds of resource, effect, read, topic, part, procedure, setting and request become jig's, ordered by authority as jig's own are, and jig asks it only about those.
- **A connector uses jig** by acting (section 7): its procedures' steps are acts in their tasks, and it may act in the deployment's name, checked by the authority every party meets.
- **Neither reaches into the other.** The core never sees a connector's types (`jig/docs/design/README.md`, 7), and a connector sees jig only through the contract, which holds values alone.

### 3.3 A connector in one page

A connector to a ticket tracker that comments on tickets, follows their changes and tells agents what a ticket says, sketched as its author would write it:

```rust
// tickets-domain: depends on skein-lib and jig-domain-contract alone
use jig_domain_contract::{self as jig, effects, lifecycle, parts, topics};

pub enum Event { Ask(jig::Ask), Released(jig::Released), Tracker(tracker::Answer) }
pub enum Out { Up(jig::Up), Write(Write), Held(tracker::Call), Now(tracker::Call) }

pub fn declaration(number: jig::Connector) -> jig::Declaration {
    jig::Declaration {
        resources: Box::new([TICKET]),                 // tickets/<project>/<number>, writer-held
        effects: Box::new([COMMENT]),                  // keyed, retried after a restart, waits for nothing
        topics: Box::new([TICKET_CHANGED]),
        parts: Box::new([TICKET_SECTION]),             // a brief section: the ticket's text
        ..jig::Declaration::empty(number)
    }
}

pub fn step(tickets: &mut Tickets, env: &Env<Limits>, event: Event, out: &mut Queue<Out>) {
    match event {
        Event::Ask(jig::Ask::Lifecycle(ask)) => lifecycle(tickets, env, ask, out), // restore, read afresh, settle
        Event::Ask(jig::Ask::Effects(ask)) => effects(tickets, env, ask, out),     // describe, judge, keep, drop
        Event::Ask(jig::Ask::Resources(ask)) => resources(tickets, ask, out),      // named, held, released
        Event::Ask(jig::Ask::Topics(ask)) => topics(tickets, ask),                 // who follows which ticket
        Event::Ask(jig::Ask::Parts(ask)) => parts(tickets, env, ask, out),         // plan, gather, cut
        Event::Ask(jig::Ask::Procedures(_) | jig::Ask::Projections(_)
                   | jig::Ask::Settings(_) | jig::Ask::Requests(_)) => {}          // not declared: never asked
        Event::Released(jig::Released::Make(entry)) => out.push(Out::Held(tickets.call(entry))),
        Event::Tracker(answer) => tracker(tickets, env, answer, out),              // outcomes, news
    }
}
```

Its records are its own, keyed by jig's task numbers, and its root arm is section 11's. Nothing in it names a core child, an internal event or a root's helper.

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

- **Timers keep today's admission.** `Admit` is `All`, or `Finishing` while the root holds new work at its watermark, so only a timer that advances admitted work is due then, as `due` and `finishing_due` choose today (`jig/crates/jig-domain-core/src/routing/timers.rs`). Deadlines take both clocks, since a sign-in expires on the wall, and cover every wait's bound (`jig/docs/design/domain/engine/waits.md`, 5): `fire` acts at one as the engine's own timer, holding, passing up or giving up, and no root arms one.
- **Where it lives.** `Engine` lives in `jig-domain-core`, and its vocabulary, `In`, `Out`, `Peer` and each port's enums, in `jig-domain-core-vocabulary`, along the same module paths (`jig/docs/design/domain/engine/README.md`, 3). No crate is added for it, so none of the 59 manifests that name `jig-domain-core` changes for the name.
- **Its names.** `In` and `Out` while today's `Event` and `Request` live beside them; the clean-up (section 15) gives the engine skein's names, `Event` and `Request`, once the old ones are gone, so an application's author meets the same two words in jig as in every step machine they write.

### 4.2 In and out, by peer

```rust
pub enum In {
    Store(store::In),              // a load's terminal; the store's reports
    Host(Channel, host::In),       // hellos, turns, calls and answers decoded, notices, losses
    Party(party::In),              // sign-ins, acts, reads, watches, deliveries taken
    Connector(Connector, contract::Up), // answers to asks, and what a connector says unasked
    Account(account::In),          // the accounts' component; the application's credentials
    Application(application::In),  // periods, stopping, settling
    Resume(Resume),                // its own continuation, fed back once released
}

pub enum Out {
    Write(store::Write),           // into this decision's commit
    Ask(Connector, contract::Ask),       // to a connector, within the step
    Held(Peer),                    // leaves once this decision's commit is durable
    Now(Peer),                     // decides nothing and shows nothing durable
}

pub enum Peer {
    Store(store::Out),
    Host(Channel, host::Down),
    Party(party::Out),
    Connector(Connector, contract::Released),
    Account(account::Out),
    Application(application::Out),
    Resume(Resume),
}
```

- **Peer first, inside each mark.** The root needs four arms to put the engine's outputs into the journal and none to know what they are (section 11). After the commit, `iterate` sends each released output by its peer: to the protocol layer, to the connector it names, or back to the engine.
- **Every name crossing is a token or a number,** as today (`skein/docs/foundation/programming-model.md`, 4.2): `Channel` a host link's, `Resume` the engine's own, a `Connector` by its number.

### 4.3 Four marks

The core already says of each output whether it is part of the decision, held or now (`jig/docs/design/domain/engine/vocabulary.md`, 1.2). The marks stay and become the outer layer of `Out`:

- **`Write`:** a record saved or erased, into the decision's commit (`jig/docs/design/domain/engine/decisions.md`, 1.6).
- **`Ask`:** a hand-off to a connector within the step (`jig/docs/design/domain/engine/vocabulary.md`, 1.3), of one of three kinds (6.3). The engine asserts at the decision's end that every judgement it asked for was answered, narrowing what its `*_routes_drained` predicates assert over its route queues today.
- **`Held`:** an output someone outside could act on, released after the commit (`jig/docs/design/domain/engine/decisions.md`, 1.2).
- **`Now`:** the journal's door (`jig/docs/design/domain/root.md`, 3).

### 4.4 Admission

- **One class per input,** a free function of the vocabulary: `class(&In) -> Class`, `Taken`, `New` or `Public`. What finishes admitted work is `Taken` (the store's answers, connectors' answers, hosts' completions, resumptions); new external work is `New`; a public read is `Public`. The root admits `New` under its watermark and `Public` under the public one, and lets what is always taken pass a held event in order, as today (`jig/docs/design/domain/root.md`, 3 and 10; `skein/docs/foundation/programming-model.md`, 7.1). No input mixes classes (section 14.4), so nothing is split.
- **`idle(&Engine)`** says the engine has nothing to do until an input comes or its next deadline passes. It replaces `quiescent` and `finish_owed`'s six reasons, which stay the engine's to compute (`jig/docs/design/domain/root.md`, 3).
- **The engine paces its own work:** its loads by `Limits::load_slots`, its activations by them, its decisions' hand-offs by `Limits::routes`. The root answers no `ready` and no `busy`.

### 4.5 Nothing else

`Engine` has no other public method and no public field. The 67 methods temper's root calls today, and those `ops` and the test root call, 73 in all, each become an input of a port, an output, or the engine's own business (section 13). Worlds read the engine as its peers do, and the store as the restart oracle does (`jig/docs/design/domain/engine/world.md`, 1).

While the move runs, the pass-through keeps a temporary access to the whole `Core`, shared and exclusive, by method and never by `Deref`, so that every use roots make of it today, its children's state and its counters included, compiles unchanged. Each milestone narrows it, and the clean-up removes it.

## 5. The ports

### 5.1 The store

- **Writes:** `Save`, `Erase`, `DropTranscript`, `DropHistory`, `CopySettingsRevision`, as today, and the start's own writes (below).
- **One load and one terminal.** `Peer::Store(Load { load, range })`, held, so it sees every commit before it (`jig/docs/design/domain/engine/decisions.md`, 1.3), and `In::Store(Loaded { load, result })`, whose `result` is typed: rows with whether more remain, cut to its bytes and by how much, or failed, with the reasons today's answers carry (gone, too linked, a turn's problem, a transcript cut or unread; `jig/crates/jig-domain-core-vocabulary/src/vocabulary.rs`, `TranscriptLoad`). The engine knows what each load is for. The ten named loads of section 2, their answers and their `busy` flags go.
- **Order is stated, not implied** (`skein/docs/foundation/programming-model.md`, 7.2): store answers keep their arrival order, and a released callback's place among them, as the predicates of `jig/docs/design/domain/root.md`, 5 hold today, with their tests. Pacing loads bounds how many are in flight and keeps no order. What temper's root and the test root keep between steps for this, a queue where a store answer waits behind the root's own work in progress, counted in its backlog, stays theirs: it is their admission (`jig/docs/design/domain/root.md`, 3), and the engine, which holds no work between steps, has nothing for an answer to wait behind. Public reads' cap goes with admission (4.4).
- **The start through the ports.** The start keeps its script, its phases and their order (`jig/docs/design/domain/engine/restart.md`; `jig/crates/jig-domain-core-vocabulary/src/start.rs`, `Phase`): the core's ranges and inboxes loaded, each connector restored, the links made, the sweep, the store's generation switched, the wires and the start's marker committed before anything leaves after an upgrade, runs adopted, then open, each connector catching up after it, reading afresh and then settling its outbox, as `Ask::CatchUp` asks today. Each phase's exchange becomes a port's: the engine's own loads through the store port; the generation's switch a store output and the wires a write; the journal's numbering restored an application output (5.5), which the root applies once the decision carrying it is accepted; what waits for the start held by the root's admission, as today; each connector's restoration and catching up by its lifecycle's asks (6.3), the connector loading its own records. The root's `restart.rs` goes once every phase has a port (`jig/docs/design/domain/root.md`, 8 becomes the engine's alone).
- **The store's reports,** `Fill` and `Census`, are inputs of the port. Commits' answers stay the journal's.

### 5.2 Hosts

- **One vocabulary for every host,** in the core's vocabulary: `host::Down` (assignments, a job's assignment, messages relayed, a call's answer, a read's answer, grants, acknowledgements of turns, answers and saved parts, a part refused busy, cancels, refusals, a turn retried, a worker paused to drain or stop, usage read or its read cancelled, sources' lists and checks, a list read) and `host::In`, decoded as it comes (section 8): a hello, lost and back, paused, a turn, an answer and a saved part as summaries with their bodies, a call as an `Act`, a `Read` or a connector call's envelope, a run's live text and tool input, usage, notices (limited, refused, unauthorized), logins, sources' answers, a list's answer.
- **A worker's restart is the engine's to judge,** as it is today (`jig/docs/design/domain/engine/executors.md`, 6): a hello that leaves out a live attempt suspends its run rather than failing it, a resumed run's assignment carries its saved parts, and nothing of it reaches a root.
- **The workers' channel encodes and decodes them** to the records its wire has today, its layouts named by fingerprint and converted both ways (`jig/docs/design/protocol/channel/layouts.md`), so the wire doesn't change. The hub keeps its own vocabulary, decoded by its worker's side of the channel (section 9).
- **An assignment names its connectors' parts** (section 6.6), which the root puts together as it does today (`jig/docs/design/domain/root.md`, 7).

### 5.3 Parties

- **In:** a sign-in for an identity the application verified, with its expiry on the wall where the application caps it, a sign-out, a party's `Act` (section 7) with its key and the value it carried, a `Read`, a watch of a subject or of a run's text, an unwatch, a delivery taken, a public read.
- **Out:** an act's outcome, a read's answer (a project's waits among them, nearest bound first, by page), a delivery, a watch ended or refused, a run's text (now), a result for a person requester, a need arrived, a sign-in refused, a carried value unused, a public answer.
- **An application's own** requests, subjects and reads are acts, reads and watches like jig's, in a scope, decided or made by the connector they name through its `requests` capability (6.3), their documents parts (6.6) (`jig/docs/design/domain/connectors-facts.md`, 1.5).

### 5.4 Accounts and credentials

The accounts' component's events and requests, as today (`jig/docs/design/domain/accounts-sources.md`, 2), and each generation of an application's credential granted (`jig/docs/design/domain/engine/README.md`, 6.1).

### 5.5 The application

What the engine and its application's service say to each other: in, a project's period (`jig/docs/design/domain/engine/vocabulary.md`, 1.1), stopping and settling (`jig/docs/design/domain/engine/restart.md`, 1); out, a project made, drained, restarting with its wait, the journal's numbering restored at a start, a brief failed for its log, a restore refused, and the settings the service applies itself: workers' admission and providers' configuration. Today these are spread over `Request::ProjectMade`, `Held::Drained` and `Restarting`, `Now::BriefFailed` and `RestoreRefused`, `Ask::Workers` and `Providers`, the start's header the roots apply themselves, and `Event::Period`, `Stopping` and `Settling`.

### 5.6 Resume

One opaque continuation, `Peer::Resume(Resume)`, which the root feeds back as `In::Resume` once released, always taken. It carries what the core asks back after a commit: today's `Held::Relay`, `TaskTerminalAcknowledged`, `TaskTurnKept`, `TaskPartKept`, `StopRun` and `PublicWait`, the start's continuations, and the fleet's held events that roots step in their own ways (D004). Its token is the core's flight's id, which the core already keeps typed inside as `Id<Flight>` (D002).

## 6. The connector contract

### 6.1 One crate, values only

`jig-domain-contract` holds what the engine and an application's connectors exchange, one module per capability (6.3) and a few shared values:

```rust
// jig-domain-contract
pub struct Connector(pub u16);        // a connector's number in its application
pub struct Declaration { /* 6.2 */ }
pub enum Ask { Lifecycle(lifecycle::Ask), Resources(resources::Ask), Effects(effects::Ask),
               Topics(topics::Ask), Parts(parts::Ask), Procedures(procedures::Ask),
               Projections(projections::Ask), Settings(settings::Ask), Requests(requests::Ask) }
pub enum Up  { Lifecycle(lifecycle::Up), Resources(resources::Up), Effects(effects::Up),
               Topics(topics::Up), Parts(parts::Up), Procedures(procedures::Up),
               Projections(projections::Up), Settings(settings::Up), Requests(requests::Up),
               Act(ops::Act) }        // any connector may act in the deployment's name (7.1)
pub enum Released { Make(Entry), Answered(Key, ops::Outcome) }   // 6.5
pub mod ops;                          // section 7: Act, Actor, Op, Read, Outcome, Refusal
```

- **Below the engine and every connector:** it depends on skein-lib and `jig-domain-link` alone, and the core's children and vocabulary depend on it, so the graph stays acyclic. The values it needs move down into it from the children and the core's vocabulary, re-exported along their old paths.
- **Values, not decisions** (`skein/docs/foundation/programming-model.md`, 4.6): it holds no state and decides nothing. A helper that decides, such as `RepairSeed::delegate` (`jig/crates/jig-domain-core-vocabulary/src/effects/connector.rs`), stays with the core.
- **What a connector's author reads** is `Declaration`, `Ask`, `Up`, `Released`, the modules of the capabilities it declares, and `ops` if it acts.

### 6.2 Declared once

```rust
pub struct Declaration {
    pub connector: Connector,
    pub resources: Box<[ResourceKind]>,    // a path's shape, and the holds a resource of it may take
    pub effects: Box<[EffectKind]>,        // its resource kinds, form, recovery class, waiting for dependencies, priced
    pub reads: Box<[ReadKind]>,            // what its agents' reads may ask, and their bounds
    pub topics: Box<[TopicKind]>,          // goals' topics among them
    pub parts: Box<[PartKind]>,            // the brief sections, workspace items and documents it serves
    pub procedures: Box<[ProcedureKind]>,  // its executors, and the codes of what their steps wait on
    pub symbols: Box<[Symbol]>,            // which it offers, and which tasks may name each
    pub settings: Box<[SettingsKind]>,
    pub requests: Box<[RequestKind]>,      // its application's own requests and subjects
    pub projects_goals: bool,
}
```

- **A capability is taken by declaring a kind of it.** A connector that declares no procedure kind is never asked about procedures, and its `step` matches those asks with an empty arm (3.3). Its lifecycle is asked of every connector.
- **What is the kind's or the deployment's is declared,** never said per instance: an effect kind's form, recovery class and whether it waits for dependencies (`jig/docs/design/domain/connectors-effects.md`, 1.1 and 1.3), today repeated in every `EffectDescription`, taken from the connector's configuration where its system decides it (`ops`'s infrastructure's recovery follows its backend); the holds each kind of resource may take; the order among effect and read kinds that authority holds as data (`jig/docs/design/domain/connectors.md`, 3.1).
- **What depends on a resource or a member is judged and said:** a resource's role and hold in a project, said up as today's `Resource` (a forge branch is shared, exclusive and waiting, or exclusive and refusing, by its adoption); the holds a batch member needs, its symbols resolved, by the one holdings ask for every source (6.3), from the member's spec, executor, home, tracking and parent, refusing with the symbol or the holder.
- **`Engine::new` checks every declaration** and refuses a start that names a kind no connector declared. Roots' installing of hold kinds, by stepping the tasks child, goes.

### 6.3 Asked by capability

Each ask is a judgement (◆), answered by an `Up` before the step returns, which the engine asserts at the decision's end; a notification (·), answered by nothing; or an operation that ends later (▷), answered by an `Up` naming it whenever it ends, the engine keeping the state that waits for it, as today.

| Capability | jig asks | It says up |
|---|---|---|
| `lifecycle`, every connector | ▷ restore your records; ▷ catch up: read afresh, then settle your outbox; · a project adopted; ▷ a deleted project released | each stage done; a project released; unasked: an adoption |
| `resources` | · named, unnamed, held, released, a writer slot, an attempt lost, what a run left; ◆ the holds batch members need, whoever made the batch | holdings, or refused with the symbol or the holder; unasked: a resource's role and hold in a project, a pool's slots, drift |
| `effects` | ◆ describe; ◆ judge one requirement at one state; · keep under a key or a proposal, drop, forget a proposal's payload or a staged call's part; ▷ resolve an uncertain entry; ▷ settle a closing task's effects; · release what it held | described, refused or busy; a verdict; a closing task settled; unasked: an outbox entry's outcome |
| `topics` | · subscribe, unsubscribe | unasked: news, classified for its subscribers as waking, kept or dropped |
| `parts` | ◆ plan a run's parts (6.6); ▷ gather a run's parts, a document, a read or a task line's parts; · cut, drop; · a claim made, a job claimed | planned; gathered with their sizes, or failed; unasked: the tasks whose parts changed |
| `procedures` | · a procedure made, activated, messaged, released from a hold, closing; · told what its step made or why it was refused, or why its waits were | a step decided (7.4); unasked: a task's waits, whenever they change |
| `projections` | ▷ project a goal; · forget a projection | a projection settled; unasked: its outbox entry's outcome |
| `settings` | ◆ judge; · keep, restore, forget replaced values, save defaults; ◆ the models' catalogue | judged; the catalogue |
| `requests` | ▷ decide an application's request; · drop it, its core part answered | decided; unasked: subjects marked changed |

Nine capabilities and about 50 cases, from 76 asks in two enums; [the notes](jig-domain-api.notes.md) map each of today's to its case. Hand-offs stay bounded and acyclic (`jig/docs/design/domain/engine/vocabulary.md`, 1.3).

### 6.4 Said up

What a connector says up is its capabilities' `Up`s and `Up::Act`: about as many cases as today's 19 connector events, none of them a copy of a core child's event. Answers name the ask they answer; what a connector says unasked (an entry's outcome, news, a resource's role, drift, an adoption, subjects changed, a task's waits) names its subject, and the engine refuses whole, telling the connector why, a report about a subject that isn't the connector's or past its bounds, as it refuses a procedure's waits today (`jig/docs/design/domain/engine/waits.md`, 6).

### 6.5 Released after the commit

`contract::Released`: make this outbox entry (`jig/docs/design/domain/connectors-effects.md`, 1.2); this act's answer, under its key. The root feeds each to its connector after the commit, as today (`jig/docs/design/domain/root.md`, 9).

### 6.6 Parts

- **One way to name what the owner keeps** (`jig/docs/design/domain/model.md`, 5): `PartRef { connector, part: Token, size }`, for a brief section, a workspace item, a run's read's answer, a call's connector part, a party's document of an application subject, and a public document.
- **A run's parts are planned, then gathered.** The engine plans a run from a preparation context, a value of the contract: the task and its attempt, its requester and root, its parent and its role among its delegates, its spec, its saved resources, its charter's pinned documents and what it may modify, and the core's own sections with their sizes and priorities and the slots left. Each connector the task names answers with what it would give, its sections and workspace items with their sizes and priorities and the writes, holds and credentials they need, once the holds they wait on are taken. The engine places them within the budget, displacing what is optional, as temper's root does today (`temper/crates/temper-engine-domain/src/assemble.rs`), then asks each connector to gather or cut what it placed.
- **A claim is checked again as it is made:** a part whose connector's holdings changed since its plan is cut, and a run whose task moved is abandoned, as today's preparation waits, pauses and fails (`Now::RunPreparationFailed`).
- **Put together by the root, as today:** an output that names parts is put into the journal with each part handed over by the connector that owns it, in its own type (`jig/docs/design/domain/root.md`, 7). That is one function in a root, not one per kind of output.
- **What goes:** `BriefCorePlanned`, the gathers, `CompleteBrief`, `WorkspaceRequest`, `Prepare`, `WorkspacePrepared`, `RunPrepared` and `ClaimPrepared`, and the fixed answers roots keep for them (`jig/docs/design/domain/root.md`, 14).

### 6.7 What a connector writes, and what it can rely on

A connector's own records, effects and outputs stay its own types. Its step takes `Ask(contract::Ask)`, `Released(contract::Released)` and its system's events, and its output has the engine's four marks with `Up` in place of `Ask`: `Up(contract::Up)`, `Write`, `Held`, `Now` (3.3). So a root's arm for a connector reads as its arm for the engine (section 11).

- **jig promises a connector:** it is asked only about what it declared; a judgement is asked within a decision and waited for in it, and nothing else is; an operation that ends later is waited for across a restart too; its writes go into the same commit as jig's, and nothing is released to it before that commit; after a start, it is restored before it is asked anything, and catches up before anything new is admitted; it never meets another connector's kinds or a core child's types.
- **A connector promises jig:** it answers every judgement in the step that asked it; it keys its own records by jig's task numbers; it says unasked what it saw of its system, as outcomes, news, drift and waits; and it makes an effect only from an outbox entry jig released to it.

## 7. One set of operations

### 7.1 Act, Actor, Op

```rust
// jig-domain-contract::ops
pub struct Act { pub by: Actor, pub ops: List<Op> }

pub enum Actor {
    Party { sign_in: Id<SignIn>, key: Key, reply: ReplyTo },
    Run { task: Id<Task>, attempt: Id<Attempt>, call: CallKey, reply: ReplyTo },
    Procedure { task: Id<Task>, step: Id<Step> },
    Deployment { connector: Connector, key: Key, reply: ReplyTo },
}
```

- **Numbers are typed where an author meets them:** a task, an attempt, a sign-in and a step are lib's `Id` of their kind (`skein/crates/skein-lib/src/id.rs`), so no two can be swapped; inside, the engine keeps what it keeps today.

- **`Op`, by group:** work (create a batch, delegate into a gathering, conclude, message, introduce, amend, cancel with a grace, release a hold at its revision, take a fresh measurement of a held judgement once `temper/docs/design/draft/archive/whose-red.md`, 6.3 has landed, stop, move to oneself, move to the newest settings, leave, lift); proposals and decisions (propose an action, a batch, a goal or settings; withdraw; decide a proposal or an escalation); person tasks (take, hand back, answer); a read position (seen, words marked read); topics (subscribe, unsubscribe); notes (write, edit lines, delete); a project's own (make, rename, remove, its policy, roles, pools, visibility, adopt, make a service, reissue a service's credential); workers (pause one to drain or stop); settings and models; LLM accounts; and the application's own, named by its connector. Each names the revision it was decided on, as today (`jig/docs/design/domain/model.md`, 11).
- **An act holds the shapes its source makes today, and no others:** a party's compound request, task operations in one project within operations per request, each admitted against the state the earlier ones staged (`jig/docs/design/domain/people/acts.md`, 1), or one operation; a run's call, one operation; a procedure's step, its decision (7.4). A wider combination, across owners or with settings, accounts or an application's requests, is new semantics a later design would have to make atomic first.
- **"A party does what an agent does"** (`jig/docs/design/domain/model.md`, 9) becomes a type: the operations are one, the actor differs, and authority checks one shape. Which actor may make which operation is authority's table: only parties change settings, make a project's and accounts' operations, and take, hand back or answer a person task; a run may propose settings, as `NamedAction::ProposeSettings` does today; procedures decide nothing (`jig/docs/design/domain/people/acts.md`, 1).
- **Keys keep their meaning.** A party's act under a key it used before is the same act or a conflict, judged against the retained record, which keeps what makes two equal, as today's keeps the original ask; `Seen` keeps its own rule, an answer the request table doesn't retain (`jig/docs/design/client/domain/contract/requests.md`, 1).
- **Where each comes from:** a party's act from the clients' component; a run's from its tool call, decoded at the edge (section 8); a procedure's from its step, in `Up` (6.4); the deployment's from a connector (`jig/docs/design/domain/people/acts.md`, 3), or from the application itself for a person's task in no project or a recurring procedure.
- **Today's three spellings go:** `people::Ask` and its families, `NamedAction`'s operations, and `ProcedureStep`'s lists.

### 7.2 Reads

A run's reads are a party's, read as its task's requester or as itself: one `Read` for a task's text, settings, a proposal, a person's documents, notes recalled, a project's waits and an application's read, with one answer. Today: a party's `reads::Read` and a run's `NamedAction::ReadTask`, `Settings`, `PersonRead`, `ReadProposal` and `Recall`, with seven answers (`Now::TaskRead`, `SettingsAnswer`, `PersonAnswer`, `ProposalRead`, `NotesIndexed`, `NotesRecalled`, `NotesRefused`).

### 7.3 Outcomes and refusals

Each operation has its outcome, and a refusal one shape whatever its source: the operation it stopped at, why (moved, with what it is now; denied, with authority's findings and whom a proposal could go to; invalid; busy; unknown), and the facts a client or a run shows (`jig/docs/design/client/domain/contract/answers.md`, 1). Today's `people::Outcome`, `StepRefusal`, `tasks::Problem`, `EffectRefusal`, `HoldingsRefusal` and `WatchRefusal` are its cases or go.

### 7.4 A procedure's step

A procedure's decision is an envelope around its operations, as `ProcedureStep` is today (`jig/crates/jig-domain-core-vocabulary/src/runs/delegation.rs`): the words its spec takes, the dependencies its work started from, which no amendment removes (`jig/docs/design/domain/connectors-procedures.md`, 1.1), its operations, and its end: a result; a hold, with its words naming its cause, object, since and way out (`jig/docs/design/domain/tasks/lifecycle-holds.md`, 3), and what it holds from; or waiting. What a waiting procedure waits on, at most four waits each with its bound, it says whenever that changes, not only at a step's end (`jig/docs/design/domain/engine/waits.md`, 6), and the engine lists, bounds and ends those waits as its own. Only the operations become `Op`s; the envelope stays the step's.

## 8. Decoded at the edge

- **Structure is decoded on the way in, all of it** (`skein/docs/foundation/programming-model.md`, 4). The engine's protocol layer decodes a run's call (its tool and input, with `jig-protocol-tools` and the application's own codecs) and a turn's and an answer's summary (size, spend, the last message read, whether it opens a window, messages refused, saved state, whether the program ran without its tools) as they arrive. The engine keeps a turn's body, and a saved part's, as bytes it stores and never reads (`jig/docs/design/README.md`, 9).
- **A call to jig's tools** arrives as an `Act` or a `Read` by `Actor::Run`. **A call to a connector's effect or read** arrives in two parts: its envelope (the run, the attempt, the call's key, the connector, a token) to the engine, and its typed request to that connector, kept under the token.
- **Fencing follows decoding** (section 14.2), and keeps what binds a call today: its owning attempt, its name within its length and the engine's room for it (`jig/crates/jig-domain-fleet/src/call.rs`), the tool it names and its contract (`jig/crates/jig-protocol-charter/src/call.rs`), its input within its bound (`temper/crates/temper-engine-service-vocabulary/src/tools.rs`). The protocol layer allocates nothing past those bounds, and a call that doesn't decode arrives as its failure, a value, and is refused as today.
- **Every path disposes of what it decoded.** A connector call's part is staged inert: only a fresh, admitted call has it described and kept. A busy, duplicate, malformed, withdrawn, stale or lost call's part is forgotten by its token, an `effects` notification (6.3). A call asked again is answered from its record before anything runs (`temper/crates/temper-engine-domain/src/route/host_route.rs`), and uses the value its owner already keeps.
- **A settled call keeps its rendered bytes.** The protocol layer renders a call's answer once, and the bytes come back as an input of the host port, which the engine keeps in the call's record (`jig/crates/jig-domain-core-vocabulary/src/runs/conversation.rs`), so a call asked again, across a redeploy too, gets the bytes it was first given.
- **Charters' sizes are checked at configuration** by their encoded bound (`skein/docs/foundation/programming-model.md`, 6.3), so no root checks an assembled charter before a claim, as `ops`'s does today (D004).
- **What goes:** `Now::Call`, `Now::Ask`'s bytes, `LinkNow`'s payloads (`DropPayload`, `TurnPayload`, `AnswerPayload`, `AcceptedTurn`, `RefusedPayload`, `PartPayload`), `Event::NamedAction`, `NamedAnswer`, `TurnPayload`, `AnswerPayload`, `AcceptedTurn`, `RefusedPayload` and `PartPayload`, and the payload tables roots and `jig-domain-hub-link` keep for them. `LinkNow::Asked` and `Relayed`, a read's answer to a host, become `host::Down`.

## 9. Hosts and workers

- **The engine's own slots become a worker of their own,** in the engine's process: a worker root composing the hub and `smith-inline-agent`, with no workspace, under its own protocol layer, joined to the engine by an in-process address, as one process already runs an engine with its workers, from the same parts as between hosts (`jig/docs/design/protocol/service.md`, 10; `jig/docs/design/domain/hosts/README.md`, 3). The engine's root then holds no hub, and the fleet reaches every host one way: through the host port and the workers' channel.
- **`jig-domain-hub-link` goes.** Its job is translating between the fleet's and the hub's vocabularies inside the engine's root (`jig/docs/design/domain/root.md`, 5). Its tables (payloads, saved parts, call routes, assignments) and its turns' retry deadlines each move first to where a worker's path keeps them, the worker's side of the channel or the hub, and only then does the crate go.
- **The price** is the channel's encoding, opening and liveness for every message between the engine and its own slots, which a one-process deployment already pays for its workers. Agents in the engine keep their shape's promises (`jig/docs/design/domain/hosts/README.md`, 3): no workspace, the engine's tools and the connectors' reads, LLM traffic through the engine's process.
- **The hub's agent face stays:** smith's parent face, its payloads bytes in the hub and typed in the agent, by hand-out to that worker root's protocol layer (`jig/docs/design/domain/hosts/hub.md`, 8), as on any worker whose root composes `smith-inline-agent`.
- **A worker's root** composes the hub, smith's agent capability and its workspaces, as today (`jig/docs/design/domain/hosts/README.md`, 4). Its channel's wire doesn't change.

## 10. Inside the engine

- **The children are private.** No production crate but the core, its vocabulary, the contract and the children depends on a child's crate; their values reach every other crate through the core's vocabulary, re-exported along its module paths (`skein/docs/foundation/programming-model.md`, 4.6), and what the protocol, store, configuration, charter and client crates name is listed as it moves. Each child's own test kit and world keep stepping it, as its world must (`jig/docs/design/domain/testing.md`, 3). `jig-boundary` checks it. The escape hatch of `jig/docs/design/README.md`, 4 goes (section 14.3).
- **What roots do with the children moves inside or to a port:** hold kinds to the declarations; the tasks events roots build (`HoldFrom`, `WakeProcedure`, `Hold`, `Cancel`, `SubscribeTopic`, `Settled`, `Fail`, …) to `Up` or to the engine's own routes; the people's and the fleet's events to their ports. What the core already does alone stays alone: its waits' alarms, a parked task's stuck walk (`jig/docs/design/domain/tasks/stuck.md`), a worker's suspension.
- **Run preparation is the engine's pipeline:** a task due, its window loaded, its parts planned with each named connector (6.6), gathered, its claim checked and committed, its assignment held. The root takes no step of it.
- **Its own work is its own type,** apart from `In`, and its continuations are one typed lookup, a flight's id, with no map keyed by task standing for one (D002).
- **`Core`'s state is grouped by concept,** each part with its own `impl` in its own module (D003).
- **A task's record is its common fields and one executor part:** agent, procedure, person or job (D014). It is a registered family's record, so it changes as `AGENTS.md`, 2 says any schema changes: no version moves, its layout is named by its fingerprint, and a value at another layout is converted both ways by a plan derived from the two texts, an added field with its default, a renamed part with its former name, a dropped one with its tombstone, and a change no plan derives as its owner's written job (`jig/docs/design/protocol/store/layouts.md`).

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
            core::Out::Ask(connector, ask) => match Connectors::of(connector) {
                Connectors::Observability => observability(root, env, observability::Event::Ask(ask), &mut decision, &mut out),
                Connectors::Infrastructure => infrastructure(root, env, infrastructure::Event::Ask(ask), &mut decision, &mut out),
            },
            core::Out::Held(peer) => decision.push_held(assemble(root, peer)),
            core::Out::Now(peer) => root.journal.now(assemble(root, peer)),
        }
    }
    assert!(out.is_empty(), "a decision's hand-offs end within its routes");
    root.journal.accept(decision);
}

fn observability(root: &mut Domain, env: &Env<Limits>, input: observability::Event,
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

- **After the commit,** `iterate` sends released outputs by peer: `Peer::Resume` back to the engine, `Peer::Connector` to its connector as `Event::Released`, every other to the protocol layer (`jig/docs/design/domain/root.md`, 3).
- **What is left to an application per connector:** one arm in its event, one in the asks, one in release, one variant in its store's `Key`, `Record` and `Write` and its `Output`, and the translation of the contract if the connector keeps a vocabulary of its own. `admission.rs`'s classes, `restart.rs`, `link.rs`, `assemble.rs`'s kinds and `client_documents.rs`'s loop go from `jig/docs/design/domain/root.md`, 12's tree, and so do the hub and `smith-inline-agent` from its section 2's (section 9).
- **What a root never does** stays as written (`jig/docs/design/domain/root.md`, 11), and now holds by construction: it has nothing of jig's to decide with.

## 12. The conformance kit

The kit plays jig's peers itself (`jig/docs/design/domain/testing.md`, 5): the store on skein-kv's in-memory mode, scripted hosts, scripted parties, the accounts' component. Since every one is a port, it finds them in an application's outputs through one function the application writes, from its `Output` to `Option<&core::Peer>`, and one into its `Event` from `core::In`. With the root's step and release and its systems' fakes, that is the adapter: an estimate of about five functions in place of today's 26 (`jig/testing/jig-conformance/src/harness.rs`), which the kit's milestone measures, since today's also carry a cold start's observation, its rehearsal and the store's evidence.

## 13. Where today's edge goes

Every variant of today's edge, and every connector ask, has one place: an input or an output of a port (section 5), a case of a capability (6.3), an operation or a read (section 7), or the engine's own work (section 10), and some go with the hand-outs (section 8) and the engine's hub (section 9). [The notes](jig-domain-api.notes.md) keep the map, row by row, at `de275134b0`, and each milestone moves the rows it names.

## 14. Departures

Each departs from a rule or from jig's design, and each is the user's call, made on 2026-11-17 as this draft recommended. The milestone that carries one writes the change into the documents named.

### 14.1 A contract crate shared by siblings

Siblings share no types, and their parent translates between them (`skein/docs/foundation/programming-model.md`, 4.5). jig's own contract with connectors is then translated by every application's root, in both directions. `jig-domain-contract` (6.1) is shared instead, as `jig-domain-link` already is (`jig/docs/design/domain/root.md`, 2). The wording 4.5 gains: **A kit's contract.** A kit may give siblings a crate of values that both name: below each, depending on lib and other vocabularies alone, holding no state and deciding nothing; a change to it breaks every sibling's build, on purpose. jig's are `jig-domain-link` and `jig-domain-contract`. 4.6's "A sibling domain doesn't" gains "but through a kit's contract". `jig/docs/design/domain/root.md`, 6's translation of the core's connector vocabulary goes; a connector that keeps its own vocabulary translates inside itself.

### 14.2 Decoding before fencing

Today the fleet admits a call from its task's live attempt and only then hands it out to be decoded (`jig/docs/design/domain/engine/vocabulary.md`, 1.4; `jig/docs/design/protocol/channel/owners.md`, 1.3). The protocol layer decodes every call on the way in instead (section 8), within the same bounds, and the engine fences what it decoded. A stale attempt's call costs its decoding, and the round trip through the root goes.

### 14.3 Private children, no escape hatch

`jig/docs/design/README.md`, 4 lets an application compose the core's children without the core. No application does, and the children's public crates are how roots came to reach into them. They become the engine's alone (section 10).

### 14.4 One class per input

A system's event may finish admitted work and bring new work at once, and the root splits it, the connector capturing a baseline for the rest before the settlement (`jig/docs/design/domain/root.md`, 3). Instead the connector's protocol layer delivers the two as two events, the settlement first, and the connector keeps in its own state, per resource, the observation the news will be judged against, set where today's baseline is captured, so the news event carries none. Its milestone states what that observation is at first, what the connector's own writes do to it, how two settlements before one news coalesce, and how a repeated absence still tells the tasks that rely on it without another deletion record, keeping the outcomes of `ops`'s three mixed events (`HealthChanged`, `ServiceDrift`, `EnvironmentDrift`) and their stories. `Class::Mixed`, the split and the baselines in the events go.

### 14.5 The root copied, not shipped

A root's shape is the same in every application, but jig can't ship it, since step code has no generic items outside lib (`skein/docs/foundation/programming-model.md`, 10.3), and a role for shared generic mechanism is an open question of skein's (`skein/docs/design/draft/component-kit.md`, 6.2). With section 11's root, what is copied fits on a page, so roots stay copied from `ops`'s (`jig/docs/design/domain/root.md`, 12) and this draft needs no new role.

## 15. Getting there

- **The engine first, as a pass-through every root adopts at once.** Its hard parts are what it takes from the roots (loads, the start, run preparation, parts), which the three roots do differently today (D004). Built alone, it would be checked by nothing. Built as a pass-through, `In::Legacy` and `Out::Legacy` carrying today's events and requests and a temporary access to the whole core (4.5), it changes no behaviour, and every later step is a real move checked by every root's own suites.
- **Then port by port.** Each milestone adds a port's variants, moves into the engine what the roots do for it, moves the three roots and the protocol pieces that feed it, and removes its cases from `Legacy` and from the access. The cheapest and most mechanical first: the store's loads, the contract's crate and its capabilities' modules, the declarations, the asks, the start, a run's parts, its workspace and claim, the engine's own slots, decoding at the edge, the operations and then their records, the remaining ports, admission.
- **Then the clean-up.** `Legacy` and the access go; the core emits the ports itself and the old vocabulary goes; the children become private; the debt the narrow edge unlocks is paid (D002, D003, D014, D017, D030); the engine takes skein's names for its input and output (4.1); the design documents are rewritten to the new edge, with a guide for an application's author from section 3, and `ops`'s root is copied again as the reference.
- **No flag day.** Everything moves inside one process and one repository. No wire between the engine, its workers and its clients changes: the engine's own slots use the channel's wire as it is. The stored records that change (a task's record, D014, and a party's retained requests, 7.1) change as `AGENTS.md`, 2 says any schema changes, by a plan derived from the two layouts or a written job, and an older build reads what a newer one wrote. One goal can move a port in jig and in all three roots together, since jig and temper are trees of one repository.
- **Each milestone is one goal of one concern,** lands whole, writes its own design into jig's documents and their status as it runs, and keeps the focused and fuzzy suites green. The execution file lists them, with their order.

## 16. Counts that may only fall

Measured on main before the first milestone, each with the command that counts it, recorded in the execution file, as skein's component-kit draft does (`skein/docs/design/draft/component-kit.md`, 4). Each falls with the milestones, and none may rise:

1. variants on the core's edge that a root matches, `Legacy`'s included, against the ports' once the pass-through has landed;
2. `Legacy`'s variants, to 0;
3. distinct core methods called outside `jig-domain-core` and its tests (67 in temper's root at `de275134b0`, 73 in the three roots), to 0;
4. accesses to the children's state outside the core (`core.tasks` and the rest: 438 in the three roots, 385 in temper's), to 0;
5. children's events built outside the core (77, 15 and 16 in temper's, `ops`'s and the test root), to 0;
6. production manifests outside the core, its vocabulary, the contract and the children that name a child's crate (27; 77 with the test kits, which stay), to 0;
7. `jig-domain-hub-link`'s lines (1,309), to 0;
8. the lines of the three roots outside tests (3,671, 4,762 and 34,535), and of temper's wiring files (about 13,000), read at each landing;
9. the conformance kit's `Application` functions (26), to about 5;
10. `Core`'s fields (99) and `TaskRecord`'s (96), by D003's and D014's measures.

## 17. What stays

The five primitives and the model (`jig/docs/design/domain/model.md`); one decision, one commit, through skein-lib's journal, the root's only way out (`jig/docs/design/rails.md`, 1.2 and 1.3); the marks; restart as the core's script and its order (`jig/docs/design/domain/engine/restart.md`); the owner keeping the value and the root putting outputs together; hand-offs within a decision, bounded and acyclic; recovery classes, verdicts, holds and the outbox (`jig/docs/design/domain/connectors.md`); plumbing and sugar, and the client's API (`jig/docs/design/domain/model.md`, 9); level-triggered views; settings as defaults with changes over them; agents in the engine's process, with no workspace; the root owned by the application, with no traits, generic items, closures or `dyn` in step code (`jig/docs/design/README.md`, 4).

## 18. Open questions

- **One order for every root** (D004): the order of release, store answers and public reads, which the roots keep differently and the user deferred on 2026-11-06. `ops`'s root releases the journal's ready batch in one call, within its release bound, and keeps no queue of store answers and no cap on public reads; the test root steps released callbacks before its other outputs; temper's releases one held output at a time; temper's and the test root queue a store answer behind work in progress and cap public reads apart (`jig/testing/jig-test-domain/src/admission.rs`, `temper/crates/temper-engine-domain/src/admission.rs`). This plan meets it where the engine takes over loads, the start and run preparation. Settled as the store's loads moved (2026-11-26): the store answers' queue stays with the roots that have work for it to wait behind, temper's and the test root, since the engine holds no work between steps, and the public reads' cap goes with admission (4.4). Default for release: the milestone first writes each root's order down as tests, then takes the simplest order that keeps them all, and brings the user any it cannot keep.
- **The hub's agent face** (section 9): smith's payloads cross between the hub and an inline agent by hand-out, now inside the engine's own worker root as on any worker. Default: as it is; a later draft may decode them at the edge too.
- **Connectors that keep their own vocabulary:** `ops`'s and temper's translate in their roots today. Default: each moves onto the contract where the move is mechanical, the rest translate at their own step's edge rather than in the root, counted, and new connectors speak the contract.

## 19. Reasons

In [jig-domain-api.notes.md](jig-domain-api.notes.md), beside this draft, as `DESIGN.md` asks, with the map of today's edge (section 13) and the commands behind sections 2 and 16.
