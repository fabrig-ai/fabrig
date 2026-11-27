# The core: its vocabulary toward the root

## 1. Its vocabulary toward the root

### 1.1 What it hears

- **from the store,** through the root: the answers to its loads, by the
  store's port (2.4); the records restored at a restart; that the loading it asked for
  is done. It does not hear commits' answers: the journal does (../root.md,
  section 4), and feeds back what the core asked to resume after one;
- **from hosts:** a host's hello with its slots, what it hosts and its
  agent's bounds; turns, with the messages a run refused, calls, answers
  and facts of runs; notices relayed from a run's agent, a source
  limited (with how long until its limit resets, where its LLM provider
  gives that, which the core makes a time on its clock as it hears it)
  or refused, or a generation unauthorized (../accounts-sources.md, 2); each
  program source's login on a worker, which follows its hello and which
  its half checks (`fleet.md`); a host lost or back;
- **from parties:** sign-ins and their ends, a credential to mint for an
  identity the application's server verified (../people/README.md, section 3);
  keyed requests, each one operation or a compound request of several
  (../people/acts.md, 1); reads, and watches opened and closed (`views.md`);
- **from accounts' protocol:** a generation granted, a failure, a
  sign-in's address for the client that asked and its subject once
  signed in, a revocation's outcome, the component closed
  (../accounts-sources.md, 2, and ../accounts.md, 5);
- **from the application,** through its root: each generation of an
  application's credential granted (`README.md`, 6.1); a project's period begun,
  saying when the next begins if its source knows. The core keeps no
  calendar, infers no cadence and schedules nothing of its own
  (../authority-budgets.md, section 1);
- **from connectors,** each by number, in jig's terms (../connectors.md,
  section 2): resources' roles and kinds of hold; pools' slots; effects'
  descriptions; verdicts; procedures' step decisions; outbox outcomes;
  classified news for subscribers; sections and workspace items ready,
  with their sizes; adoptions; drift; a restart step done;
- **from itself:** an output it asked to resume once released (1.2), and
  its children's timers, through `fire`.

### 1.2 What it asks for

Each request the core makes says which of three it is:

- **part of the decision:** a record saved or erased, which goes into
  the decision's commit; and the hand-offs the root completes before the
  step returns: describe this effect, judge this requirement, keep or
  drop this effect, take this procedure's parameters, gather or cut this
  section;
- **held:** what must wait for the decision's commit to be durable: an
  assignment, a message relayed to a run, a cancel, an acknowledgement,
  an answer to a call or to a party, a load (so that it sees every commit
  before it; 2.4), an outbox entry to make, a resumption of the core's own; and
  a party's read's answer and a watch's delivery, behind the last commit
  made (`decisions.md`, 1.2);
- **now:** what decides nothing and shows nothing durable: a run's text
  streamed to watchers, the answer to a run's read, a run's admitted call
  handed out to be decoded (1.4), a host's refusal before anything was
  admitted.

The root keeps each as the core said (../root.md, section 3); the journal
releases held ones in order, and the conformance world checks that
nothing held left early (../testing.md, section 5).

### 1.3 Decisions and hand-offs

A decision is a route through several children and connectors, completed
within one step of the root's (skein/docs/foundation/programming-model.md, 4.5):

- **One event in,** from the root.
- **Hand-offs out and back:** the core asks the root for what only a
  connector can say (a description, a verdict, a section's size), and
  the root asks the connector and gives the answer back to the core,
  before its step returns. Hand-offs are bounded per decision and
  acyclic: the core never asks a connector something whose answer needs
  another of the core's answers.
- **One end:** the core says the decision is complete, and the root
  accepts it into the journal (../root.md, section 3).

What must first wait for a load, a read or a host is a state of its own
(a task preparing, a brief gathering), and its decision is made in the
step its last answer arrives.

### 1.4 The usual routes

- **Sibling routes finish within their decision.**
  `sibling_routes_drained` requires every sibling continuation to finish
  before the core returns its decision, so a later entrance cannot split
  the decision's writes. The core world holds this under load in
  `jig-core-world`,
  `route_orders::a_wide_route_finishes_within_its_decision_before_a_later_entrance`.
- **A task's generated requests finish within its decision.**
  `task_routes_drained` requires the requests a task route generates to
  be routed before that decision ends. The core world holds this in
  `jig-core-world`,
  `route_orders::a_task_route_finishes_its_generated_requests_within_its_decision`.
- **A party's continuation finishes within its decision.**
  `party_routes_drained` requires the people route's continuation and
  answer to stay in the decision that admitted the request. The core
  world holds this in `jig-core-world`,
  `route_orders::a_people_route_finishes_its_continuation_within_its_decision`.
- **The fleet's follow-ups finish within their decision.**
  `fleet_routes_drained` requires the fleet route's generated follow-ups
  to finish before that decision ends. The core world holds this in
  `jig-core-world`,
  `route_orders::a_fleet_route_finishes_its_follow_ups_within_its_decision`.

| What arrives | Route | Then |
|---|---|---|
| a run's turn | fleet (fenced) → tasks (the inbox taken) → authority (spend) → views | commit; the turn acknowledged |
| a run's call to an engine tool | fleet → authority (the check) → tasks or notes | commit; the call answered |
| a run's call for an effect | fleet → the connector describes it → authority, its judges asked → the connector keeps it | commit; answered when made, or past its deadline |
| a run's call for a read | fleet → authority (the grant) → the connector serves it | answered now; nothing committed |
| a run's answer | fleet → tasks (result checked, task closing or idle; the messages it refused or left unread stay, and wake it by its policy) → authority (spend); what it left → its connectors | commit; acknowledged; effects made |
| a procedure's step | its connector → authority (each effect, its judges; each batch) → tasks | commit; effects made; tasks started |
| a connector's news | classified by the connector → tasks (inboxes, wakes) | commit; due tasks run, procedures step |
| an outbox outcome | its connector → tasks (settled; closing advances) | commit |
| a party's request | people → authority (their role and pool; a budget given `AsAllowed` resolved; a deadline given as a span made a wall) → tasks | commit; the party answered, with what an `AsAllowed` budget was given |
| a party's compound request | people (one project, within operations per request) → for each operation in order: authority, one pool counted across them → tasks | one commit, or none; one answer: all made, or the first operation refused and why |
| a project made (a party's `MakeProject`, or the deployment's act) | people (the maker an owner of the deployment) → the core (the name in shape and free, a configured policy, room; the next number) → authority (the policy copied) → people (the maker its owner) → tasks (its first period, with no next start) | commit; the maker answered; the root hears the project made |
| a task due | tasks → authority (the run may start) → brief, with its connectors gathering, and its transcript's current window loading → tasks (claim, with its inbox's messages for the start) → fleet (place) | commit the claim; assign |
| a task closing | tasks → its connectors (release) | commit; releases made |
| a goal or any task in its subtree changing | tasks → every enclosing goal’s projecting connectors, including a nested tracked task’s own | commit; whole plans and milestones made |
| a project's period | authority (its spend within the policy) → tasks (the period's funding opened; recurring tasks' batches; standing allotments renewed) | commit; tasks started |
| a timer | its child | as its child says |

- **A run's call is fenced, then decoded.** The fleet admits a call only
  from its task's live attempt; the core hands the admitted call out, and
  the engine's protocol layer decodes its input and gives the typed
  request back to the root, which routes it as the rows above say
  (`../../protocol/channel/owners.md`, 1.3; ../root.md, section 5). No domain parses a
  call's input.
- **A budget given `AsAllowed`** is resolved by authority as the batch is
  admitted (../authority-budgets.md, section 1). A pool's amount left is a live
  number only the core holds, which no client could read and send back
  without racing. The task keeps the amount, and the answer, a party's or
  a run's, says what was given.
- **A compound request is one decision** (../people/acts.md, 1): the route
  above runs once per operation, in order, within one step of the root's,
  and its hand-offs are bounded by its operations (`README.md`, section 7).

## 2. The engine

What a root calls: `Engine`, the core behind five free functions, and its vocabulary, `In` and
`Out` (`temper/docs/design/draft/jig-domain-api.md`, 4.1 and 4.5). For now it passes through
what has no port yet: section 1's vocabulary, whole but for the store's loads, which cross by
the store's port (2.4).

### 2.1 Its functions

- **`Engine`** holds the core and nothing else. `Engine::new(config, connectors, limits)`, its
  `connectors` a `&[Declaration]` (`../connectors.md`, 2), builds it as `Core::new` does, refusing
  what that refuses, and refuses a start whose declarations don't hold. Both refuse with
  `StartProblem`. It returns the first of these that fails, taking each declaration in the order
  given (its connector, then its resource kinds, then its effect kinds), then each configured
  connector, then the total:
  - `StartProblem::Declared { connector }`: a declaration's connector is not among
    `config.connectors`, or is declared twice;
  - `StartProblem::ResourceKind { connector, kind }` and `StartProblem::EffectKind { connector,
    kind }`: a connector declares one kind twice;
  - `StartProblem::Undeclared { connector }`: a configured connector has no declaration;
  - `StartProblem::HoldKinds { limit }`: the declarations' resource kinds together are more than
    `tasks.hold_kinds`, the tasks child's table.

  ```rust
  // jig-domain-contract: the crate's root, beside `Declaration`
  pub enum StartProblem {
      Membership(jig_domain_link::Problem),   // the workers' membership, as the link refuses it
      Declared { connector: u16 },
      ResourceKind { connector: u16, kind: u16 },
      EffectKind { connector: u16, kind: u16 },
      Undeclared { connector: u16 },
      HoldKinds { limit: u32 },
  }
  ```

  `StartProblem` is the engine's own, beside the declaration it judges; `jig-domain-link`'s
  `Problem` keeps the workers' cases alone (`Workers`, `WorkerName`), which the link's
  `Membership::new` refuses and the engine carries whole as `Membership`. Why its own: a
  declaration is the engine's, which the worker link never reads, and the link sits below the
  contract, so the link's error can't name what the contract declares. Built, the engine has
  installed every declared resource kind's rule in the tasks child, before anything is restored,
  and keeps the effect kinds, by which it judges each description (`../connectors-effects.md`,
  1.1). No root, kit or world installs hold kinds: the core's step takes no `tasks::Event::Kinds`
  from outside, and one given is a fault.
- **`step(engine, env, input, out)`** takes one input as one decision and appends its outputs to
  `out`, in the order the core made them. For `In::Core(event)` they are exactly what the core's
  own step returns for `event`, each as `Out::Core`, but for its loads, each of which leaves as
  `Out::Held(Peer::Store(..))` at its place (2.4). `In::Store` is taken the same way, as one
  decision.
- **`fire(engine, env, timer, out)`** fires one due timer the same way, as the core's `fire` does.
- **`append(requests, out)`** is the conversion `step` and `fire` make, for the requests one of
  the core's other entrances returned (2.3): it appends them to `out` in order, each load as
  `Out::Held(Peer::Store(..))` at its place and every other request as `Out::Core`.
- **`due(engine, now, wall, admit)` and `next_deadline(engine, now, wall, admit)`** keep today's
  timers, clocks and admission: `now` is the engine's clock and `wall` the wall's, since a
  sign-in expires on the wall. `Admit::All` admits every timer, as the core's `due` and
  `next_deadline` do; `Admit::Finishing`, while the root holds new work at its watermark
  (`../root.md`, 3), admits only a timer that advances admitted work, as `finishing_due` and
  `next_finishing_deadline` do (`jig-domain-core/src/routing/timers.rs`).
- **`idle(engine)`** is the core's own quiescence, the journal's left to the root that holds it:
  the core's `quiescent(journal_idle)` is `idle(engine) && journal_idle`. It includes the
  finishing the core owes (`../root.md`, 3).
- **Where they live.** `Engine` and the functions are the `engine` module of `jig-domain-core`; `In`,
  `Out` and `Admit` are the same module of `jig-domain-core-vocabulary`, which the core's re-exports
  whole (`README.md`, 3). No crate root re-exports them while the core's own `step` and `fire`
  hold those names.

### 2.2 In and out

- **`In::Core` carries today's `Event`, and `Out::Core` today's `Request`,** and neither enum
  has another variant. A request keeps its mark (1.2) inside `Out::Core`, and the root keeps it
  as it does today (`../root.md`, 3). For the draft's pass-through
  (`temper/docs/design/draft/jig-domain-api.md`, 4.5 and 15), jig names the variants `Core` because
  its sources keep out the draft's name `Legacy` (`jig/tests/boundary/src/names.rs`).
- **The ports come beside `In::Core` and `Out::Core`.** Each port of the draft
  (`temper/docs/design/draft/jig-domain-api.md`, 4.2 and 5) arrives as variants of `In` and `Out`
  and takes its cases out of `In::Core` and `Out::Core`, a milestone at a time, until they are
  empty and go. The store's is the first (2.4): `In::Store`, and `Out::Held` with its peer,
  `Peer::Store`. The other marks (`Write`, `Ask`, `Now`) and peers come with their ports.

### 2.3 The core, while the move runs

- **The whole core, by method:** `Engine::core` gives it shared and `Engine::core_mut`
  exclusive (`engine.core()`, `engine.core_mut()`). So every use a root, a kit or a world makes
  of `Core` today (its methods, its children's state, its counters, the step's other entrances
  `want_accounts` and `resume_fleet`) compiles through them unchanged. Each milestone narrows what
  passes through, and the clean-up removes them.
- **Who holds it:** outside `jig-domain-core` and its own tests, whoever builds a core builds an
  `Engine` in its place: a root, and a kit, a world or a test that drives one. It calls 2.1's
  functions where they stand for the core's own: `step` with `In::Core` for each event the core's
  `step` took, `fire` for its timers, `due` and `next_deadline` under the admission it keeps today
  (`Admit::All` for `due` and `next_deadline`, `Admit::Finishing` for `finishing_due` and
  `next_finishing_deadline`), and `idle` for `quiescent`. The core's other entrances (`restore`,
  the start's, the reads', `finish_decision`, `want_accounts`, `resume_fleet`) and every other use
  go through `core()` and `core_mut()`. The requests such an entrance returns (`resume_fleet`,
  `finish_decision`, the start's phases, the settings reads') the holder passes through `append`
  (2.1) before it routes them, so that every load reaches it one way, as 2.4 says, and the holder
  routes `Out` alone. What the holder does is unchanged: each event, timer and request crosses in
  the order it did, at the point of its decision it did. A crate that only reads the core for its holder, such as a
  world's inspection or a reader of the core's state, keeps taking `&Core`, from `core()`.
- **Never by `Deref`,** since a deref would make each use of the core read as a use of the engine:
  through two named methods every remaining use stays in sight, to review and to the counts. The
  counts of `temper/docs/design/draft/jig-domain-api.md`, 16 that read a use of the core (its
  methods called, its children's state reached) read `core()` and `core_mut()` as they read
  `core.`, and do not count those two, so a root that holds `Engine` in place of `Core` lowers
  no count by doing so.
- **Why a pass-through:** built alone, the engine would be checked by nothing. Passing through, it
  is checked by every root's suites as each root adopts it, and each later move is a real one
  (`temper/docs/design/draft/jig-domain-api.md`, 15).

### 2.4 The store's port

The engine's loads take one shape (`temper/docs/design/draft/jig-domain-api.md`, 5.1): one
output, a load, held, and one input, its answer, typed. What a load is for is the engine's
alone: a root carries the load to its store and the answer back, and reads neither.

```rust
// jig-domain-core-vocabulary's engine::store, re-exported along the same path by jig-domain-core
pub enum Out { Load { load: Token, range: Range } }      // Peer::Store, held
pub enum In { Loaded { load: Token, result: Answer } }   // In::Store, always taken

pub enum Range {
    Record(Key),                             // one of the core's records, by its key
    Proposal { task: u64, proposal: u64 },   // a task's proposal, from its retained history
    Window { task: u64, from: (u64, u32) },  // a task's current transcript window, from its first turn
    Notes(notes::Range),                     // a page of notes
    Read(Box<reads::Range>),                 // a party's or a run's read
}

pub enum Answer {
    Rows(Rows),
    Cut { turn: TurnRecord, bytes: u64, more: bool },  // a window's turn past its page's room
    Failed(Failure),
}

pub enum Rows {
    Records { records: Box<[Record]>, more: bool },
    Turns { turns: Box<[TranscriptRow]>, more: bool },
    Read { rows: Box<[reads::Row]>, next: Option<reads::Position> },
}

pub enum Failure {
    Busy,                       // the store had no room for it, and kept nothing
    Gone,                       // a read's range is no longer there
    TooLinked { joins: u32 },   // a read's first row needs more joins than the read's bound
    TurnProblem { task: u64, attempt: u64, turn: u32, owner: Box<tasks::TaskRecord>, why: reads::TurnFailure },
    Unread,                     // a window's first turn not found, or a frame past pages already given
                                // unread or refused: ends the load and discards those pages
    Failed,                     // the store could not read it
}
```

The engine's own enums gain the port (2.2): `In::Store(store::In)`, and `Out::Held(Peer)` with
`Peer::Store(store::Out)`.

- **Held, always.** Every load leaves as `Out::Held(Peer::Store(Out::Load { .. }))`, released
  once every commit made before it is durable, so that it sees them (`decisions.md`, 1.3). The
  root holds it as it holds any held output, gives it to its store once released, and gives
  each answer back as `In::Store(In::Loaded { .. })`, which it always takes (`../root.md`, 3).
  The core's own steps queue each load as `core::Held::Store`, among their other held requests
  in the one queue of requests they share, and the engine converts it at its edge (`step`,
  `fire`, `append`): no root receives one, and a root's match over `core::Held`, which lists every
  case, names it in an arm that routes nothing. The case leaves `core::Held` with `Out::Core` (3).
- **One token per load,** unique among the engine's loads in flight and the load's until its
  last answer. A `Window` is answered page by page, in the order its turns are read, each page
  but the last with `more`; every other range by one answer, whose `more` or `next` says only
  that rows remain past it, for the engine to load if it wants them. An answer whose token names
  no load in flight is dropped: what waited on it has gone first (`decisions.md`, 1.3).
- **What each range is answered with:**
  - `Record`: `Records` holding the record, or none where the store has none; or `Failed`.
    These are the engine's records loaded by key: an ended task, a party's read position over
    an ended conversation, a decided proposal or escalation (`../../protocol/store/keys.md`).
  - `Proposal`: `Records` holding the history record that keeps the proposal, which the store
    finds through the task's live record and the position it keeps of the proposal, or none; or
    `Failed`.
  - `Window`: `Turns`, each turn with its text as the protocol layer renders it, or `Cut`, a
    turn past its page's room, alone in its page, with its fields, its transcript empty and its
    length (`executors-transcripts.md`, 1; `../../protocol/store/transcripts.md`, 3); or
    `Unread`, its first turn not found or a frame past the pages already given unread or refused
    (`../../protocol/store/transcripts.md`, 3). `Unread` ends the load and discards the pages
    already given (`executors-transcripts.md`, 1). `Failed` also ends the load.
  - `Notes`: `Records` holding the page's notes records, `more` if any remain past it; or
    `Failed`.
  - `Read`: `Read`, the page's rows, each with its joins, and the position the next page
    begins at; or `Busy`, `Gone`, `TooLinked` or `TurnProblem`
    (`../../protocol/store/loads.md`, 3). A read answered `Failed` ends as one answered
    `Busy` does: the store could not serve it now.
  - An answer its range cannot have is read as `Failed`.
- **Paced by the engine** (`decisions.md`, 1.3): no root answers for room, and none keeps a
  load waiting for one.
- **In the order given.** The engine routes each answer in the step it is given, as one
  decision, and keeps none for a later step, so its store answers keep the order the root gives
  them. A root that keeps work of its own in progress between steps gives a store answer after
  the work before it (`../root.md`, 5, `store_answers_fifo`). A load's place among the held
  outputs keeps it behind those held before it, and a callback the journal releases to the
  engine comes back at its place in the release order (`../root.md`, 5,
  `prior_held_delivery_dispatched`).
- **Nothing stored or sent changes.** The port is an in-process value between the engine, its
  root and the store's routing (`../../protocol/store/README.md`, 3); the records and keys it
  loads are the store's as they are, and no wire carries it.
- **Why one answer for every load:** the engine knows what each load is for, so a root needs no
  arm per load and keeps no flag for one, while the reasons stay typed. A window is the one range
  the store pages by itself, so its later pages keep the load its first took and wait for no
  commit, and a window of any length loads once begun.

## 3. Later

- **The store's reports and the start's exchanges** (`temper/docs/design/draft/jig-domain-api.md`,
  5.1): `Fill` and `Census` as inputs of the store's port, and each phase of the start through
  the ports, the engine's own restart loads among them, with the start's milestone.
- **The public reads' cap.** A public read loads nothing (`../people/public-reading.md`, 9), so
  the cap that temper's root and the test root keep on public reads in flight stays theirs, with
  their public watermark, until the parties' port and admission take it
  (`temper/docs/design/draft/jig-domain-api.md`, 4.4 and 5.3).
- **The engine as the API** (`temper/docs/design/draft/jig-domain-api.md`, 4 to 6, by its
  execution file): the ports beside `In::Core` and `Out::Core`; one class per input, and `idle` in
  place of `quiescent` and `finish_owed`; then `In::Core`, `Out::Core` and the access gone, and
  `In` and `Out` named `Event` and `Request` once today's are.
- **The rest of a declaration** (`temper/docs/design/draft/jig-domain-api.md`, 6.2 and 6.3): its
  reads, topics, parts, procedures, symbols, settings, requests and whether it projects goals, each
  with the capability the engine asks it by, and the `Connector` number type with the ports that
  carry it; a start refused, too, for a kind its configuration names (a grant's, a requirement's, a
  template's symbol) that no connector declared, once reads and symbols are declared, since a
  grant's kind may be a read's. An effect kind's resource kinds, whether it is priced, and a
  resource kind's path shape wait for a check that reads them: the engine reads an effect's
  resources as names and its price as given.
