# The core

Provisional, 2026-10-07, revised 2026-10-28. What jig's core is, as a
domain layer inside an application's engine: its parts and how they fit,
what crosses between it and the application's root, how a decision
reaches the store before anything leaves the engine, how it restarts,
how it runs agents, the capabilities it keeps (the fleet, briefs,
notes, views, accounts), and its settings, changed at runtime. The model is ../model.md; tasks are ../tasks/README.md,
authority ../authority.md, connectors ../connectors.md, parties ../people/README.md,
hosts ../hosts/README.md, LLM accounts ../accounts.md; what the application's root
does with all of it is ../root.md. The mechanics are those of skein's
`../foundation/programming-model.md`. What is still open is listed in section 10.

## 1. In one page

- **The core is where work is decided.** It keeps the tasks, checks
  every action against authority, starts agents' runs on hosts, asks
  parties, and tells the application's connectors what to make, judge,
  step, project and release. It is a child of the application's root,
  and the parent of its own children.
- **A parent and its children.** The tasks child domain is the center
  of the core, keeping every live task; authority is policy; the fleet,
  people, accounts, briefs, notes and views are capabilities. The core
  routes between them and faces the root (section 3).
- **It speaks one vocabulary to the root** (section 4): the store's
  records, the hosts', the parties', the accounts', and the connectors'
  in jig's terms. It knows connectors by number.
- **A decision commits once,** and nothing leaves before it does
  (section 5). The core says, of every output, whether it waits for the
  decision's commit; the root's journal holds it until it is durable.
- **A restart is the core's script** (`restart.md`): its own records, then
  each connector's, then the runs adopted, the connectors reading afresh
  and settling their outboxes, and only then decisions.
- **Agents run from their task** (`executors.md`). A run is given its task's
  charter, a brief built for why it runs, the messages waiting in its
  task's inbox, its transcript's current window if it resumes, and the
  engine's tools its authority allows. Each turn it takes is
  committed as it ends; every call that changed something is committed
  and answered once, however often it is asked.
- **It computes what only it can** (`views.md`): the documents parties
  watch and read, from committed state: `me`, the signed-in party's own;
  task queries, over the live tasks it holds and the ended tasks' indexes
  in the store; what each task waits on, from its tree and the entries
  waiting in it; what a decision would do, as data. A conversation is
  known by its flag, never by its charter, and a template is served,
  never used.
- **Bounded.** The core holds live work only: live tasks, their inboxes
  and proposals, the runs in flight. Ended tasks, transcripts, history
  and notes are in the store, loaded when wanted.
- **Settings are the binary's defaults with the store's changes over
  them** (`settings.md`): charters, templates, the policies projects are
  made from, the rules, and an application's own kinds. An owner's
  change is one decision and makes a revision; a task records the
  revision it was made in, and its runs read their charter from it. A
  charter's change may be an overlay kept over its default
  (`overlays.md`).
- **The domain is complete** (skein/docs/foundation/programming-model.md, section 4): a world
  of domains and fakes runs everything the core does, with no protocol
  and no io (section 9).

## 2. The core in the engine

```
application root        the journal, routing, translation; faces the protocol layer
├── core                tasks, authority, people, fleet, accounts, brief, notes, views
├── <connectors>        the application's
├── jig-domain-host     jig's hub, for runs on the engine's own slots, if the application runs agents there (hosts.md)
├── smith-inline-agent  the hub's agent capability in the engine: smith's domains in this process, with no workspace
└── …                   the application's own children
```

- **The root faces the protocol layer,** and the core faces the root.
  Everything the core hears arrives through the root, already in the
  core's vocabulary; everything it asks for leaves through the root,
  which translates it and puts it into the journal (../root.md).
- **Hosts are reached through the root:** workers over the channel, which
  the protocol layer carries; the engine's own hub as a sibling the root
  routes to (../hosts/hub.md, 1), its agent smith's `smith-inline-agent`
  (../hosts/README.md, section 5). Both speak the same vocabulary. The core's
  fleet does not tell them apart beyond what placement needs.
- **The engine never reads a system** but through its connectors; what a
  workspace holds is for the run to find.
- **The store is the engine's alone.** Nothing else reads or writes it,
  so the engine needs no compare-and-swap against anyone: its own
  ordering of commits is the truth.

## 3. Structure

```
jig-domain-core             the core: routing among its children, runs, calls, the restart script, numbers
├── jig-domain-core-vocabulary  what crosses the core's edge: its events, requests, records, keys, documents, reads, settings, limits
├── jig-domain-tasks        the center of the core: tasks, batches, lifecycle, holds, inboxes, wakes, proposals
├── jig-domain-authority    authority's order and checks; the rules and the projects' policies
├── jig-domain-people       parties: identities, sign-ins, roles, requests, inboxes
├── jig-domain-fleet        hosts: slots, placement, attempts, relaying
├── jig-domain-accounts     LLM accounts and an application's credentials: sources, states, uses, granted
├── jig-domain-brief        a run's brief, from typed sections and connectors' sized ones, within a budget
├── jig-domain-notes        notes: scopes, entries, indexes
└── jig-domain-views        parties' watches, level-triggered

jig-domain-link            the link's values, shared by the fleet, the hub and accounts (../root.md, 2)
```

```
jig-domain-core/src/
├── lib.rs          the core: its state, its configuration and construction
├── vocabulary.rs   what crosses to the root: Event, Now, Request, Limits (section 4)
├── routing/        the step, the step helpers and flights; the routes by kind: party/, run/, task/, effect/, view/, settings/ (4.4)
├── start/          the start: load, restore, census, link, sweep, switch, open (section 6)
├── runs/           a run's choice, charter, credentials, delegation and preparation; live runs; brief and transcript text (sections 7 and 9)
├── admission/      what a party or a task may make or change: compound checks, roles, amendments, goals, person tasks, escalations, proposals (4.4, 11.5)
├── funding/        pools and their funding (sections 12 and 13)
├── effects/        connectors' effects, resources, relays and projections (section 5)
├── settings/       settings, revisions, templates and charters (section 17)
├── documents/      the documents the core makes for parties (section 11)
├── reads/          parties' reads and watches (section 11)
├── acts/           acts and their admission (11.5)
├── inbox/          the inbox and its census (11.4)
├── numbers.rs      deployment numbers and their counters (5.4)
├── stored.rs       durable records, keys and run proofs (5.4, 7)
├── translate.rs    its children's vocabularies translated into each other's
├── limits.rs       its limits' validity (section 13)
└── tests/
```

Each directory is one concept of this document; a route lives under the kind of route it is, whatever child it steps.

`jig-domain-core-vocabulary/src/` has the same tree, holding each module's vocabulary: `numbers.rs`,
`limits.rs`, `translate.rs` and `public.rs` whole, most of `vocabulary.rs`, `stored.rs` and
`settings/`, and the parts of the other modules that cross the core's edge.

The core's vocabulary is a crate of its own, below the core
(`temper/docs/design/draft/crate-graph.md`, 5):

- **It holds what crosses the core's edge:** every item a crate other than the core names (the
  events, requests, writes and limits of section 4; the records, keys and run proofs; the
  documents, reads and settings; the numbers), with every item that their definitions, their
  inherent methods and their trait impls reach. None of it names `Core`. A file goes whole where
  the rest of it reaches nothing but the vocabulary and itself. Otherwise its vocabulary goes and
  the rest stays, so the core's own state, such as `settings::Restoring`, stays with the core.
- **`jig-domain-core` keeps `Core`:** its state, its construction, its step, every method and
  function that reads or changes it, the helpers only they use, and the step tests that drive it.
  `Core` isn't cut, since every module adds methods to it
  (`temper/docs/design/draft/crate-graph.md`, 5.2). So the core stays over the size bound
  (`tools/crates/design.md`, 3.1) until its state is grouped into owned parts
  (`jig/docs/development/debt/D003-core-one-object.md`).
- **The two share one module tree:** an item keeps its module path, so
  `jig_domain_core_vocabulary::documents::TaskQuery` is `jig_domain_core::documents::TaskQuery`.
  Each of the core's modules re-exports its namesake in the vocabulary whole, and the core's root
  re-exports the vocabulary's root. The vocabulary's root re-exports what the core's root re-exports
  of its items. So a caller builds through the same paths in either crate.
- **Each crate names the one it needs:** a library whose own code needs only the vocabulary, and
  none of whose other dependencies reach the core, depends on `jig-domain-core-vocabulary`. The
  store's, the engine clients', the configuration's and the protocol's other crates are among them.
  Its tests may still use the core. Every other crate names `jig-domain-core`. So a change to how
  the core decides recompiles the core and what names it, not the protocol. A build from scratch
  compiles the core beside the store's crate, not before it.
- **Its tests** are the step tests of its items that build without `Core`. Those that drive `Core`
  stay in the core and reach the items through its re-exports.

The tree follows skein/docs/foundation/programming-model.md, 4.5: each child domain has its
own vocabulary, limits and world; a parent owns its children's state and
routes between them; siblings share no domain types, and the core
translates between them with small total functions.

The queues between the core and its children are uncapped. Their
hand-offs finish within the decision (4.4), without a reservation for a
child's largest output. No child declares an output maximum.
Authority's stored denial count is `stored_finding_limit` (../authority.md, section 2); people's
`outputs_bound` is reserved by the settings route at the store's commit
edge (../people/README.md, section 2).

- **`tasks` is the center of the core:** it keeps every live task, and
  knows a task's lifecycle and nothing of runs, connectors or clients.
- **`authority` is policy:** decisions over data, keeping only the rules
  and policies in force.
- **The others are capabilities,** each knowing its own concern: the
  fleet the hosts, people the parties, accounts the LLM accounts that pay
  for runs and the credentials runs are lent, the brief a run's context,
  notes what agents learn, views who watches.
- **The core keeps little of its own:**
  - the deployment's numbers: the next task number and every other number
    the core gives, committed as they are used;
  - the runs in flight: for each, its task's number, its attempt, and
    what its assignment was made from, so an answer can be routed and a
    brief rebuilt (`executors.md`);
  - for each live agent task, its conversation's first attempt and where
    its current window starts (`executors.md`, 1 and 2);
  - the calls committed for each live task since its last committed turn,
    by attempt and name, with the core's part of their answers (`executors.md`, 3);
  - the decisions in progress, between the hand-offs the root completes
    within one step (4.3);
  - the restart script's state (`restart.md`);
  - the settings: the defaults, the changes over them, the held
    revisions and their charter versions (`settings.md`).

## 4. Its vocabulary toward the root

### 4.1 What it hears

- **from the store,** through the root: the answers to its loads, in
  pages; the records restored at a restart; that the loading it asked for
  is done. It does not hear commits' answers: the journal does (../root.md,
  section 4), and feeds back what the core asked to resume after one;
- **from hosts:** a host's hello with its slots, what it hosts and its
  agent's bounds; turns, with the messages a run refused, calls, answers
  and facts of runs; notices relayed from a run's agent, a source
  limited (with how long until its limit resets, where its LLM provider
  gives that, which the core makes a time on its clock as it hears it)
  or refused, or a generation unauthorized (../accounts.md, 3.5); each
  program source's login on a worker, which follows its hello and which
  its half checks (`fleet.md`); a host lost or back;
- **from parties:** sign-ins and their ends, a credential to mint for an
  identity the application's server verified (../people/README.md, section 3);
  keyed requests, each one operation or a compound request of several
  (../people/acts.md, 1); reads, and watches opened and closed (`views.md`);
- **from accounts' protocol:** a generation granted, a failure, a
  sign-in's address for the client that asked and its subject once
  signed in, a revocation's outcome, the component closed
  (../accounts.md, 3.5 and section 5);
- **from the application,** through its root: each generation of an
  application's credential granted (6.1); a project's period begun,
  saying when the next begins if its source knows. The core keeps no
  calendar, infers no cadence and schedules nothing of its own
  (../authority.md, section 7);
- **from connectors,** each by number, in jig's terms (../connectors.md,
  section 2): resources' roles and kinds of hold; pools' slots; effects'
  descriptions; verdicts; procedures' step decisions; outbox outcomes;
  classified news for subscribers; sections and workspace items ready,
  with their sizes; adoptions; drift; a restart step done;
- **from itself:** an output it asked to resume once released (4.2), and
  its children's timers, through `fire`.

### 4.2 What it asks for

Each request the core makes says which of three it is:

- **part of the decision:** a record saved or erased, which goes into
  the decision's commit; and the hand-offs the root completes before the
  step returns: describe this effect, judge this requirement, keep or
  drop this effect, take this procedure's parameters, gather or cut this
  section;
- **held:** what must wait for the decision's commit to be durable: an
  assignment, a message relayed to a run, a cancel, an acknowledgement,
  an answer to a call or to a party, a load (so that it sees every commit
  before it), an outbox entry to make, a resumption of the core's own; and
  a party's read's answer and a watch's delivery, behind the last commit
  made (5.2);
- **now:** what decides nothing and shows nothing durable: a run's text
  streamed to watchers, the answer to a run's read, a run's admitted call
  handed out to be decoded (4.4), a host's refusal before anything was
  admitted.

The root keeps each as the core said (../root.md, section 3); the journal
releases held ones in order, and the conformance world checks that
nothing held left early (../testing.md, section 5).

### 4.3 Decisions and hand-offs

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

### 4.4 The usual routes

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
| a goal changing | tasks → its projecting connectors | commit; projections made |
| a project's period | authority (its spend within the policy) → tasks (the period's funding opened; recurring tasks' batches; standing allotments renewed) | commit; tasks started |
| a timer | its child | as its child says |

- **A run's call is fenced, then decoded.** The fleet admits a call only
  from its task's live attempt; the core hands the admitted call out, and
  the engine's protocol layer decodes its input and gives the typed
  request back to the root, which routes it as the rows above say
  (`../../protocol/channel/README.md`, 5.3; ../root.md, section 5). No domain parses a
  call's input.
- **A budget given `AsAllowed`** is resolved by authority as the batch is
  admitted (../authority.md, section 7). A pool's amount left is a live
  number only the core holds, which no client could read and send back
  without racing. The task keeps the amount, and the answer, a party's or
  a run's, says what was given.
- **A compound request is one decision** (../people/acts.md, 1): the route
  above runs once per operation, in order, within one step of the root's,
  and its hand-offs are bounded by its operations (`README.md`, section 7).

## 5. Decisions and the store

### 5.1 Commits

- **One decision, one commit.** What a decision changes in every child
  and connector it touched is one commit: records written or removed, as
  one.
- **Whole across families, through a kill.** That commit is one
  skein-kv commit (`../../protocol/store/README.md`, 5), so a process killed at
  any point of its write, after any record, mid-frame or between
  families, starts again from what was durable: the last whole
  decision, never part of one. No family's record is durable without
  the records of its decision that it links to.
- **Nothing links ahead of what it names.** A decision saves a record
  that names a party, task, project or revision only when the core
  holds what it names, made in that decision or an earlier one. A
  person named by an input from outside the core (a procedure's or a
  connector's batch, an agent's ask or tool call, an application's
  request) that the people child does not hold is refused where the
  input is admitted, saying which, and never written, so a start finds
  every link it makes (`restart.md`).
- **Numbered and in order.** The journal numbers commits and the store
  applies them in that order. Its answer is that a commit is durable,
  which makes every earlier one durable too.
- **The state runs ahead.** A child's state changes as the decision is
  made, not when it commits, so the next decision sees it.
- **New work is paced by the root's watermark.** The backlog counts the
  journal's commits and outputs, the root's work and its queued store
  answers (../root.md, sections 3, 4 and 10). What finishes admitted work,
  including the store's answers, is always taken. New external work
  past the watermark is handed back before any child changes; its
  sender keeps it and offers it again. No route reserves journal room.
  The core's `Limits::routes` bounds each drain within a step, with
  its predicate asserted empty afterward (../root.md, section 10). A
  commit's write limit remains the store's edge
  (skein/docs/foundation/programming-model.md, 7.1).
- **A failed commit stops the engine** (../model.md, section 4). Nothing
  waiting for that commit, or any later one, goes out. A restart starts
  from the last durable commit.

### 5.2 What waits for a commit

Every output that someone outside the engine could act on waits for the
commit it follows from:

- **effects:** an outbox entry is handed to its connector to make
  (../connectors.md, section 4);
- **assignments:** a run is assigned only once its claim is durable, so
  an attempt never runs that a restart would not know of;
- **acknowledgements:** a host's answer, and a run's turn, are
  acknowledged once committed; until then the host keeps them;
- **answers to tool calls** that changed state, and to parties'
  requests;
- **messages relayed to a live run,** so a run never reads what a
  restart would not have;
- **loads,** so a load sees every commit before it;
- **what parties read and watch:** a read's answer and a watch's
  delivery decide nothing, but are made from state that may run ahead of
  the store, so each waits for the last commit made before it, as an
  output of a decision with no commit of its own (`views.md`);
- **grants:** a generation sent to an attempt (section 6) decides
  nothing, but goes to the attempts the fleet keeps, which run ahead of
  the store: one whose assignment waits for its commit is an attempt a
  restart would not know of, and its host does not know it yet either.
  So a grant waits for the last commit made before it, and leaves after
  its attempt's assignment, since held outputs leave in order
  (`../../README.md`, 6.2).

What decides nothing and shows nothing durable waits for nothing: a
run's text streamed to watchers as it is written (`views.md`), and the
answers to a run's reads.

### 5.3 Loads

- **What is loaded, not held:** ended tasks and their results;
  transcripts; history; notes' entries; anything a client pages through.
- **A load is a request** with a token and one terminal event, rows in
  bounded pages (skein/docs/foundation/programming-model.md, 4.4). What waits on it is in a
  state that says so: a brief gathering, a run being prepared to resume,
  a party's page, an amendment adding an ended task (../tasks/changes.md
  ). That amendment's request stays unanswered while the rows load,
  one at a time: each ended task it adds, then the ended tasks above it
  that its reach is read through, at most one per level of the tree. Once
  they are in, it is admitted again from the start, on the rows and on
  live state as it then is: the pass that found them missing answered
  nothing and committed nothing. If what waits goes first (its run ends,
  its party leaves, the engine restarts), its load goes with it, and its
  request is left as any unanswered one is.
- **What a decision loads stays as loaded until it is decided.** Its
  loads are held (4.2): each is sent once every commit made before it is
  durable, so it sees them. While they are in flight, a request whose
  decision would change what they load is answered busy (../people/README.md,
  section 5, a mark on an ended conversation). Otherwise two decisions
  that each loaded one record before either changed it would both decide
  on the old record, and the later commit would undo the earlier.
- **Bounded:** the loads in flight, and the bytes a load brings, are
  limits; a load past its bytes is cut, and says how much.
- **Parties' reads** are documents and pages the client's contract lists
  (`../../client/domain/contract/README.md`, section 4): from live state where the
  engine holds it, loaded otherwise, and answered once the last commit
  made is durable (5.2).
- **Ended tasks are read through the store's indexes** (`views.md`, 2): an ended
  task by its key, a list of them through one ended index, each row
  joined to its task's record, and an ended conversation's read position,
  by point reads in the same step (`../../protocol/store/loads.md`, 3).

### 5.4 What the store keeps of the core

| Record | Written by | Loaded |
|---|---|---|
| the deployment: its id, made at its first start, and its numbers | the core | at start |
| settings: the defaults the newest revision was made with, the changes over them, the held revisions' records, the charter versions they read and pending proposals' sets (`settings.md`, 7) | the core | at start, after the deployment, before every other record |
| settings' history: the records of revisions no longer held, within the history limit (`settings.md`, 7) | the core | on demand |
| projects: resources and roles, homes, policy, parties' roles | people | at start |
| parties: identities, sign-ins | people | those with a live sign-in or a role, at start; others as they sign in |
| parties' requests' keys, with their answers, for a retention | people | at start, those within it |
| tasks: every field of ../tasks/README.md, section 3 (its label, flags, last activity, LLM choice, kept account and revision among them), holds and their queues, writer slots, the last turn committed for a live attempt, and the revision a claim was prepared under (`settings.md`, 3) | tasks | live ones at start; ended ones on demand |
| LLM accounts: each account's holder, LLM provider, subject and display name, uses, and its sources in their order, each with its kind, a program's name, its state and a limit's reset; never a program's login on a host, which hosts report (../accounts.md, 3.1 and 3.5) | accounts | at start |
| the indexes of ended tasks: by requester, every ended task a party requested; by project, the ended tasks flagged listed; each saved in the decision that ends its task (`views.md`, 2; `../../protocol/store/loads.md`, 4) | tasks | on demand |
| parties' read positions over conversations: a live one's; an ended one's, moved by the decision that ends the task and erased with it (`views.md`, 4) | people | live ones at start; ended ones on demand |
| calls committed since each live task's last turn, by attempt and name, with the core's part of their answers | the core | with their task |
| each live agent task's conversation: its first attempt, where its current window starts, whether an upgrade closed it, and whether an upgrade crossed its claim (`executors.md`, 2) | the core | with their task |
| funders' numbers: each party's pool and each project's spend, per period; a period's beginning, and when the next begins if its source said | tasks | at start |
| a task's history: made, amended, held, released, cancelled, by whom, why; and its activity: words written to it, runs begun and ended, what it heard (`executors.md`, 2) | tasks | on demand |
| inboxes: messages not yet taken | tasks | with their task |
| proposals: pending, and decided ones' outcome | tasks | pending ones at start |
| transcripts: turns, with their spend and whether each opens a window | the core | to resume, from the current window's start; on demand |
| notes | notes | indexes per scope in use; entries on demand |
| secrets: minted credentials' digests, by their sign-in's number, and services' credentials' digests; LLM accounts' records and keys, by their account and source, each kept before it is lent (`../../protocol/store/secrets.md`; `../../protocol/llm.md`, sections 5 and 6) | the protocol layer, never the domain, which names them by account, source and generation, or by a sign-in's or a service's number | at start, by the protocol layer |

Each connector keeps its own records beside these (../connectors.md,
section 2): its outbox, procedures' states and parameters, what it made
by key, its projections' states, and its own state. The application's
root wraps both in its store vocabulary (../root.md, section 6).

**Times** the store keeps are wall times (`env.wall`), so a record means
the same after a restart; a wait the core arms is a deadline on its
monotonic clock (`env.now`), computed from them (skein/docs/foundation/programming-model.md,
section 9). Accounts' source deadlines retain both clocks where the
limit or retry is created (`../accounts.md`, 3.5): timers use `Deadline.at`,
and every projection and stored reset uses its source-owned
`Deadline.wall`, never a conversion at read or save time. Restore keeps
the exact wall and maps it to the new monotonic clock once. Fleet grace
and adrift times are the fleet's own (`fleet.md`).

### 5.5 What the core asks of the store

The store is assumed (../model.md, section 2); this is what the core relies
on, and all it relies on:

- a commit of several records is applied whole or not at all;
- commits are applied in the order sent, and durable once answered;
- a load sees every commit answered before it was sent;
- records are keyed by the domain's own keys, and listed by key ranges,
  in bounded pages; what is live (live tasks, pending proposals, the
  outboxes, live resources) is kept in key ranges of its own, so a
  restart reads it without reading history;
- what one engine wrote, the next reads: records carry the version of
  their shape;
- nothing else writes it.

### 5.6 How a child's records reach a commit

Each child that keeps durable state, and each connector, defines its own
records and ordered keys in its own vocabulary. Neither a sibling nor
the store chooses their shapes. The parent translates their keys and
carries their records in its own store vocabulary; the store's protocol
layer encodes each shape with a version (5.5).

- **`Save { record }`** keeps the record under its own key, and
  **`Erase { key }`** removes that key. These requests are part of the
  decision the parent is making. They have no terminal event and do not
  ask the child to wait: its state changes as it decides (5.1).
- **`Restore { record }`** supplies one record at restart, and
  **`Restored`** says every record in the child's live ranges has been
  supplied. The child then decides from that restored state.
- **`Load { owner, range }`** asks for one bounded page of the child's
  own key range. Its one terminal event is **`Loaded { owner, rows,
  more }`**. The token is echoed, and `more` says another page remains,
  which the child asks for with a load of its own.

A child never waits for a commit and never holds an output back for one.
Its outputs leave it as it decides, each saying which of the three of
4.2 it is, and its parent passes them up. A child's world plays its
parent and the store: it keeps saved records, applies erases, commits at
the end of each step, and restarts by feeding the durable records as
`Restore` followed by `Restored`. Restart is therefore tested before the
root is built.

## 6. Accounts

The accounts child domain keeps the deployment's LLM accounts, and an
application's credentials beside them (6.1). ../accounts.md says what its
LLM accounts are: their holders, sources and uses (its section 3), each
source's state and how the child keeps it, from the restart's first step
to the close (3.5), and the accounts chosen with a run's check (4.3). The
protocol layer's accounts' component signs signed-in sources in,
refreshes them, keeps their records and keys before lending, and holds
their values (`../../protocol/llm.md`, section 5). The domain names a
source by its account and its place, and a credential by its source and
a generation, with how long it is valid, and never sees a value or a
record.

### 6.1 An application's credentials

A connector's workspace may need credentials of the application's own: a
token to read from a system and write to it. The accounts child keeps
them beside LLM accounts, and lends them as it lends a source's value
(../accounts.md, 3.5):

- **Named, never seen:** each by the application's number for it and a
  generation, with how long it is valid. Their values are the
  application's protocol layer's, in a table of its own, read from a
  file or from an exchange it refreshes.
- **Granted through the root:** the application's root tells the child
  each generation granted, as the accounts' component does for an
  account (4.1). One is usable while a granted generation is still
  valid (../authority.md, 8.3).
- **Lent to runs:** a connector names those a run's workspace needs as
  it gathers its items (../connectors.md, section 10); the assignment
  carries them as grants, each naming the application form, which
  selects the application's table (`executors.md`, 1;
  `../../protocol/channel/README.md`, section 7); and they follow their attempts
  as an account's do (`fleet.md`).
- **Unauthorized, limited and refused are LLM accounts' only.** A
  system that refuses a credential is the workspace's failure (../hosts/hub.md,
  3), and the application's to refresh where it keeps it.

## 7. Limits and working sets

What the core holds, each bounded by its limits and summed into its
worst case (skein/docs/foundation/programming-model.md, 6.3):

- live tasks, with their inboxes, subscriptions, references, holds and
  queues, and their dependencies, with the index of each resource's
  holders and waiters and each task's dependents (../tasks/holds.md, 7);
  pending proposals (../tasks/README.md, section 6);
- the rules and the live projects' policies, within limits on projects,
  resources per project and parties holding roles;
- parties signed in, and their pending requests; their read positions
  over live conversations, one per live task flagged conversation
  (`views.md`, 4);
- the settings (`settings.md`, 8): the defaults and the changes over them, the
  held revisions and the charter versions they read;
- each live project's overrides of the templates, which the core only
  serves (`views.md`, 1);
- the calls committed since each live task's last turn, and the funders'
  numbers;
- hosts, their runs, and the calls and messages relayed;
- LLM accounts, with their sources and uses, within the accounts'
  bounds (../accounts.md, 3.6), and an application's credentials;
- decisions in progress and their hand-offs; loads in flight and their
  rows;
- briefs gathering, with the sizes of the connectors' sections, and the
  notes' indexes in use; the preparations waiting for notes, at most one
  entry per live task (`briefs.md`, 2.2);
- watches, each its subject, a task query's keys and order among them,
  its window and two marks (`views.md`).

None of it grows with the store's history. A new task, a party, a run or
a watch past its limit is refused at its entrance, and the asker told
why. The root adds the core's worst case to its connectors' and its own
(../root.md, section 10).

- **A compound request's decision** is priced as operations per request,
  the people child's limit (../people/acts.md, 1), times the costliest
  operation it may hold: a cancel of a whole tree, within tasks per tree,
  with the hand-offs to the connectors its tasks name. It is summed into
  the worst case with the rest, as every decision is (`../../README.md`,
  6.2), so one request holding a whole conversation's close, or a whole
  project's goals reordered, fits.
- **The check for circles of waits** is priced as a walk of every live
  task, with a mark and a counter per task and per resource, within the
  live tasks' and holds' limits; its scratch is sized at start, as the
  index is, and the tasks a circle names within their limit (../tasks/holds.md, 7, and ../tasks/README.md, 6
  ). A restart's judgement of every wait at once
  takes the same.
- **What reads and deliveries compute** is priced with them: a task
  query's live part is one pass over the live tasks, and what each of
  its lines waits on a walk of a tree, within tasks per tree (`views.md`, 2 and 3
  ); its ended part is a load, within the loads' limits (5.3). None
  of it is kept between deliveries.

## 8. Below the domain

What the protocol and io layers owe the core, through the application's
protocol layer, which composes jig's pieces with its own:

- **The store:** commits of typed records, numbered, answered once
  durable, and loads by key range in pages, over skein-kv; jig encodes
  the core's records, the application its connectors', each with a
  version per shape.
- **Workers:** jig's channel, the engine's side: assignments, messages,
  grants, cancels, answers to relayed calls, acknowledgements down;
  hellos, turns and answers with the messages refused, calls, facts and
  notices up (../hosts/README.md, section 8).
- **Connectors:** each its own protocol, the application's.
- **Parties:** jig's messages, carried by the application's transport to
  the clients' component, which knows no transport
  (`../../protocol/client/README.md`): jig's documents and the application's, its
  requests and reads, and credentials. jig mints, verifies, expires and
  revokes credentials; the printed link and a service's credential are
  its own ways of signing in, and any other is the application's, which
  asks the component to mint a credential for the identity it verified
  (../people/README.md, sections 3 and 9; `../../client/domain/contract/README.md`,
  sections 2 and 8).
- **Accounts:** sign-in, refresh, revocation, records and keys kept in
  the secrets' store before they are lent, and tokens lent, by skein's
  OAuth accounts driver (`../../protocol/llm.md`, sections 5 and 6;
  `../../protocol/store/secrets.md`); a value a party's request carries
  taken out and kept before the request reaches the core (../accounts.md,
  section 5); an application's credentials by its own protocol layer,
  each generation granted told to the core through the root (6.1).
- **Configuration:** the deployment's rules, connectors and their
  systems' addresses and credentials, LLM providers with their
  endpoints, sources and terms' facts, and models with their windows,
  outputs, efforts, prices and descriptions, but no account (../accounts.md,
  3.1), workers' secrets, the engine's
  own slots, the charters the deployment offers with their runs' budgets
  and reserves, the application's templates, which the core only serves
  (`views.md`, 1), procedures' settings, limits. Projects and their policies are
  the store's, seeded from configuration on a first start. The
  whole-config check after overrides admits only an actual configured
  project count within both authority and people roster capacities,
  ownerless projects included (`../../protocol/limits.md`
  ). `Core::new` asserts the matching
  `projects.len() + 1 <= limits.people.projects` programmer
  precondition, the one being the deployment's roles (../people/README.md,
  section 3), alongside its other configuration assertions. Operator
  input reaches that constructor only after validation; the core seeds
  the whole set and the deployment's roster, with no other implicit
  roster or partial seed.

## 9. The world

Each child domain has a world of its own, its parent and neighbours
scripted in it, its expectations in a referee; authority, which keeps no
state, is tested by its step tests.

The core's world runs the core with every child beneath it, under a root
of jig's own for testing, against fakes that share none of its types
(../testing.md, section 4): jig's test connector, with resources, holds,
pools, priced effects, requirements, procedures and faults drawn from
the seed; one to three scripted workers and the engine's own slots,
whose runs play scripts; scripted parties; the store on skein-kv's
in-memory mode, slow or failing a commit; and the engine restarting
cold, at drawn moments or at one a scenario chooses.

**Every story ends with a cold restart,** the restart oracle (`restart.md`
): the engine restarts over the story's store, restores with every
link made, and reads back what the story wrote (the parties' documents,
the tasks' and the children's views), equal in their durable part to
their last answers before it. Each document type declares once, in the
one projection the oracle applies, the fields a restart rebuilds rather
than keeps, each with the step that makes it so: run-time observations
such as an account's last end, or a task's runtime on a host that died
with the engine. A field in doubt is durable; no story opts out but a
deliberately broken fixture; and a difference left after the projection
is a finding, a writer not keeping what its document shows. A link
that cannot be made refuses the restart (`restart.md`), and the oracle
fails the story naming the refusal's phase, record and link.
This is where the cross-record rules a restore once checked live, one
writer at a time: each child's rules are the oracle's to prove over
every writer of its records, never the restore's to judge.

Its stories: a conversation that answers, parks and resumes; a plan of
reports, a choice and procedure tasks, revised mid-way, and done; a
procedure's effect waiting for another connector's verdict, then made; a
delegate held past its tries, escalated to a person and released; a goal
cancelled with runs live and effects in flight, closing deepest first; a
tool call whose answer is lost with the channel and the engine, asked
again after a restart and answered from its record, made once; a run
handing a held resource to a procedure while it still holds the writer
slot; two tasks queued for a pool's last slot, and the pool shrinking
under them; an uncertain effect across restarts made before its retry
deadline; an effect of a kind that cannot be recovered, uncertain and
held for a person; a judge's facts changing between its verdict and the
effect; a policy narrowed while a proposal waits, an entry is committed
and a run is live; a note written, corrected by a person and recalled; a
recurring task across a period's reset; a person's words to a running
run; a conversation resumed from its current window after a compaction;
a message a run left unread, waking its task again as the run ends;
drift holding a task, and the client told why.

Its stories of what the core computes and serves: a task query watched
while tasks enter it, leave it, change its order and end, each delivery
after its commit, the ended ones then read through the ended indexes,
by requester and, for a listed task, by project, a filter past the
prefix coming back cut; what a task waits on moving from its delegates
to a party as a delegate asks its person a question, and back once it
is answered; a conversation's turns counted unread until its person
marks them seen, the conversation ended and read after a restart with
the position its person left, and one forgotten once read leaving
nothing that shows; `me` marked as a pool is spent from, deliveries
coalesced; a budget given `AsAllowed` answered with what was given.

Its stories of added dependencies (../tasks/lifecycle.md, 8): an agent task given
a dependency while its run is live, the run finishing done and parked,
its activity naming what it waits for, woken when the dependency is
done and finishing again, after a restart too; an idle agent task kept,
a message to it waiting in its inbox and never waking it, then
delivered as it goes on; a kept task's run asking an effect that waits
for dependencies, refused; a run amending a delegate to wait for
another, and one whose add would close a circle, refused naming it; the
reordering of two tasks queued for one write hold, ended done in the
new order.

Its stories of agents and procedures doing what parties do: a run
amending a delegate a party amended since it was shown, answered moved
with what it is now, then amending again; a run releasing a held
delegate, naming its hold's revision, and making an uncertain effect
again; a run proposing an amendment and an effect beyond its authority,
each accepted by a person; a run subscribing to a connector's topic and
woken by its news; a run cancelling a delegate with a grace it finishes
within; a procedure amending, cancelling and messaging its delegates,
one amendment refused as moved and the procedure stepping again on what
it is told.

Its stories of settings (`settings.md`): an owner's change of a charter,
a template and a policy in one set, committed in one decision, then
`me` re-read with the template, the next `MakeProject` copying the
policy, a narrowed rule refusing the next effect of a task made before
it, and a restart restoring the same settings; a change refused as
stale, as out of shape, and by a connector judging an application's
kind, each committing nothing; a restart with other defaults, recorded
as the release's, a changed setting kept over them and a reset
returning one to the new default; a charter changed while its task's
run is live, the run finishing on its version and its delegates made on
the newest; a conversation moved at its next run, and one whose charter
was removed staying where it is; a change moving running work, and a
removed charter living while a task holds it, a delegation to it
refused as not offered; a restart with old revisions held and a run
adopted on its version; the bound reached, the change refused naming
the oldest revision's tasks, those moved with `MoveToNewest`, and the
change then made; a revision leaving memory as its last holder ends.
Its stories of a proposal of settings (`settings.md`, 6): one proposed by a run and
accepted by an owner, moving running work; one made stale by another
change and withdrawn, its proposer told.

Its stories of an upgrade across a fresh mark (`executors.md`, 2), over the store
with fixture patch sets: a task mid-run on a worker, its window
resumed from an earlier attempt, upgraded across the mark, the worker
listing its run at its hello, the run stopped, the claim lost, its late
turn refused, its writer slot held until the connector reads afresh,
and its next attempt fresh with the closed window's tail; the same with
no worker listing it, lost once the grace has passed; an in-process
claim yet to commit a turn, settled at once as not started; a crash
after the switch before the restart, the next restart settling the
claim the same way; the next attempt claimed and the engine restarted
before its first turn, adopted as any claim; and a second upgrade
across another mark before that turn, settling it again.

Its stories of the restore before the switch (`restart.md`), over the store
with fixture patch sets: a patched copy restored whole, its restores'
commits in the copy, no secret served and no run adopted before the
switch, switched, and the manifest naming the position then; a live set
past limits lowered since, refused before the switch, the generation
patched from still active and unchanged, and a start of the build before
opening it as it was; a task naming a project the copy does not hold, a
link that cannot be made, refusing the start before the switch with the
task and the project named, the generation patched from the same; a
crash during the restore, the next start patching a fresh copy; and a
start with nothing pending, whose switch step switches nothing and
commits the wires alone.

Its referee holds the core to ../model.md, section 11, seen from outside;
applications' worlds reuse it on jig's conformance world (../testing.md,
section 5).

## 10. Open questions

- **A person's whole `me` for a run** (`executors.md`, 3): a run's read as its person
  reads no frame, which spans the person's projects, so an application
  composes what the person is in the run's project from the project, its
  inbox filtered by it and its task queries. Default: as it is; a frame
  narrowed to one project if an application needs the person's pool or
  offer there and no project read gives it.

- **A run's `later`** (`executors.md`, 3): a run's `delegate` and `amend` name no
  `later`, so only a party's request or a template gives one. A tool member
  for it waits for a planning agent that needs a delegate's later runs on
  other models.
- **A number freed:** a catalogue's 64 numbers last the deployment's life, a
  withdrawn model keeping its own (`settings.md`, 11). A number no held revision and no
  live task reads could be given again, should a provider's lists come to
  use them up.

- **Room for a replacement settings proposal** (`settings.md`, 6, 8): whether
  admission may discount the proposer's own pending set at
  `settings.proposals`. Default: check the full bound before replacement,
  refuse the fresh proposal and retain the pending set. Replacement
  grants no extra slot.

- **A party's scope beyond its asking runs** (`briefs.md`, 2.1): whether a delegate
  works for the person two tasks up and reads their preferences, whether
  other kinds of `../../draft/notes-memory.md` take a scope, and whether a run's
  own `note` and `recall` name a party's scope. Default: an asking run of
  the person's own task alone, through its application's named actions.
- **A line range on `recall`:** `recall` returns a note's whole body, which costs
  input, not output. A range joins it only if a walk shows it's needed
  (`briefs.md`, section 2; the user's idea, 2026-11-05).
- **A record whose link cannot be made,** at a start (`restart.md`): left
  out of the live state with what links through it, kept in the store,
  named in the start's notes to a party with a role in the deployment,
  and its child's sweep skipped, so that a writer's bug costs a record
  and not the start. For now it refuses the start, naming the record
  and its link (request `store-crash-safe`, 11-09); after the first
  release. A tool that repairs such a store is not designed: that
  request leaves it out.
- **A transcript's load with no slot** (`briefs.md`, 1.1, `executors.md`, 2): `TranscriptLoad` has
  no capacity answer, so a root never answers one for room: it keeps the
  activation due while its load table is full, and a window's later pages
  in the slot its first took (temper's `domain/root.md`, 7). A root that
  answered `TranscriptLoad::Failed` for room would count it as transient,
  and the try it spent would make the next run start fresh from the tail
  rather than resume. Default: no capacity answer, until the user says.

- **A fresh attempt's tail, lost before its first turn.** An attempt
  that starts fresh from a closed window, or past the resume limit,
  carries the old window's tail in its brief, but its proof keeps no
  window (`conversation` is its own attempt, which `valid_proof` holds
  every window at or after); lost before its first turn, by a crash or
  another mark, the attempt after it starts fresh without that tail.
  Default: so, rare and costing context alone; a proof that kept the
  tail's window apart from the current one would carry it.
- **Commits in flight:** how many, and whether the store should take
  several decisions as one commit when they wait together.
- **Briefs:** whether a brief's store loads and connectors' gathering
  share one budget; cancelling a brief while it gathers; how a missing
  section shows in what the agent reads.
- **Transcripts' limits:** how the resume limit is sized against a
  charter's models' windows, so that a conversation smith compacts
  (smith's `domain/session.md`, section 8) resumes rather than starts
  fresh; and how long the store keeps earlier windows' turns, which no
  run is sent.
- **Views:** a limit on watchers per subject; how a watch of a task's
  tree is bounded when the tree is large; whether, should a pass over the
  live tasks per delivery, or a walk of a tree per line, prove slow, the
  tasks child keeps what a task waits on along its lineage as entries
  come and go (`views.md`, 2 and 3).
- **Scaling out:** one engine per deployment until it binds; the store
  being the engine's alone is what would change first.
- **A list's reach past the first page and past a name's arrival:** a
  provider's later pages, and a known name's window, efforts or shown name
  changing on its list (`settings.md`, 12). Default: one request, and a name's figures are
  what its first read gave, a change of settings changing them.
- **What a model is good at:** a description in configuration, or
  structured capabilities a planning agent could filter on. Default: a
  description (`briefs.md`, section 1).
- **A release's removed default that a change names** (a template
  changed to run a charter the release drops): the start is refused,
  naming both (`settings.md`, 7), or the default is kept as a change of its own,
  recorded in the release's revision. Default: refused, as a document
  naming a charter it lacks is. The refusal is the backstop: a release
  that drops such a default carries its own patch for the stored
  changes that name it, as every release that changes what is stored
  does from the baseline on (temper's draft `draft/upgrades.md`, section 4),
  so only a release that forgot its patch meets the refusal.
- **An application's kinds pinned,** a value read from a task's
  revision as a charter is, should an application's setting shape what a
  running task is given. Default: never pinned (`settings.md`, 5).
- **Delegates on their parent's revision,** so that a goal's subtree
  keeps the settings its plan was made under, as a change's option.
  Default: later; delegates take the newest (`settings.md`, 3).
