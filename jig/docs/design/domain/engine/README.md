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

- [`vocabulary.md`](vocabulary.md): what crosses between the core and the root, and the engine a root calls.
- [`waits.md`](waits.md): a task's waits, each a record with its kind, since and bound.
- [`decisions.md`](decisions.md): decisions, commits and the store.
- [`world.md`](world.md): the core's domain world and its stories.
- [`executors-transcripts.md`](executors-transcripts.md): turns, transcripts and resumption.
- [`executors-tools.md`](executors-tools.md): the tools the engine offers a run.
- [`settings-application.md`](settings-application.md): an application's settings.
- [`settings-proposals.md`](settings-proposals.md): proposing settings and deciding a proposal.
- [`settings-store.md`](settings-store.md): how settings are kept and loaded at startup.
- [`settings-catalogue.md`](settings-catalogue.md): the model catalogue and a provider's list.
- [`project-charters.md`](project-charters.md): a project's own charters, held by revision and bound by name.

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
- **It speaks one vocabulary to the root** (`vocabulary.md`, section 1): the store's
  records, the hosts', the parties', the accounts', and the connectors'
  in jig's terms. It knows connectors by number.
- **A decision commits once,** and nothing leaves before it does
  (`decisions.md`, section 1). The core says, of every output, whether it waits for the
  decision's commit; the root's journal holds it until it is durable.
- **A restart is the core's script** (`restart.md`): its own records, then
  each connector's, then the runs adopted, and decisions open; the
  connectors then catch up while the engine serves, reading afresh and
  settling their outboxes, each decision waiting for the facts it reads.
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
  and no io (`world.md`, section 1).

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

jig-domain-contract        the values the core and its connectors exchange, by capability, and the operations' (../connectors.md, 2)
jig-domain-link            the link's values, shared by the fleet, the hub and accounts (../root.md, 2)
```

```
jig-domain-core/src/
├── lib.rs          the core: its state, its configuration and construction
├── engine.rs       the engine: the core behind the functions a root calls (vocabulary.md, 2)
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
(`temper/docs/design/draft/archive/crate-graph.md`, 5):

- **It holds what crosses the core's edge:** every item a crate other than the core names (the
  events, requests, writes and limits of `vocabulary.md`, section 1; the records, keys and run proofs; the
  documents, reads and settings; the numbers), with every item that their definitions, their
  inherent methods and their trait impls reach. None of it names `Core`. A file goes whole where
  the rest of it reaches nothing but the vocabulary and itself. Otherwise its vocabulary goes and
  the rest stays, so the core's own state, such as `settings::Restoring`, stays with the core.
- **`jig-domain-core` keeps `Core`:** its state, its construction, its step, every method and
  function that reads or changes it, the helpers only they use, and the step tests that drive it.
  `Core` isn't cut, since every module adds methods to it
  (`temper/docs/design/draft/archive/crate-graph.md`, 5.2). So the core stays over the size bound
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
hand-offs finish within the decision (`vocabulary.md`, 1.4), without a reservation for a
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
    its current window starts (`executors.md`, 1, and `executors-transcripts.md`, 1);
  - the calls committed for each live task since its last committed turn,
    by attempt and name, with the core's part of their answers (`executors-tools.md`, 1);
  - the decisions in progress, between the hand-offs the root completes
    within one step (`vocabulary.md`, 1.3);
  - the restart script's state (`restart.md`);
  - the settings: the defaults, the changes over them, the held
    revisions and their charter versions (`settings.md`).

## 4. Its vocabulary toward the root

Moved to [`vocabulary.md`](vocabulary.md), section 1.

## 5. Decisions and the store

Moved to [`decisions.md`](decisions.md), section 1.

## 6. Accounts

The accounts child domain keeps the deployment's LLM accounts, and an
application's credentials beside them (6.1). ../accounts.md says what its
LLM accounts are: their holders, sources and uses (its section 3), each
source's state and how the child keeps it, from the restart's first step
to the close (3.5), and the accounts chosen with a run's check (`vocabulary.md`, 1.3). The
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
(../accounts-sources.md, 2):

- **Named, never seen:** each by the application's number for it and a
  generation, with how long it is valid. Their values are the
  application's protocol layer's, in a table of its own, read from a
  file or from an exchange it refreshes.
- **Granted through the root:** the application's root tells the child
  each generation granted, as the accounts' component does for an
  account (`vocabulary.md`, 1.1). One is usable while a granted generation is still
  valid (../authority-checks.md, 1.3).
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
  notes' indexes in use; the preparations waiting for notes, and those
  waiting for their connectors' plans, at most one entry each per live
  task (`briefs.md`, 1.4 and 2.2); a brief complete, until its run is
  prepared;
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
  the worst case with the rest, as every decision is (`../../rails.md`,
  1.2), so one request holding a whole conversation's close, or a whole
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
  ); its ended part is a load, within the loads' limits (`decisions.md`, 1.3). None
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

Moved to [`world.md`](world.md), section 1.

## 10. Open questions

- **A person's whole `me` for a run** (`executors-tools.md`, 1): a run's read as its person
  reads no frame, which spans the person's projects, so an application
  composes what the person is in the run's project from the project, its
  inbox filtered by it and its task queries. Default: as it is; a frame
  narrowed to one project if an application needs the person's pool or
  offer there and no project read gives it.

- **A run's `later`** (`executors-tools.md`, 1): a run's `delegate` and `amend` name no
  `later`, so only a party's request or a template gives one. A tool member
  for it waits for a planning agent that needs a delegate's later runs on
  other models.
- **A number freed:** a catalogue's 64 numbers last the deployment's life, a
  withdrawn model keeping its own (`settings-catalogue.md`, 1). A number no held revision and no
  live task reads could be given again, should a provider's lists come to
  use them up.

- **Room for a replacement settings proposal** (`settings-proposals.md`, 1, `settings.md`, 8): whether
  admission may discount the proposer's own pending set at
  `settings.proposals`. Default: check the full bound before replacement,
  refuse the fresh proposal and retain the pending set. Replacement
  grants no extra slot.
- **A settings change past the fleet's needs bound** (`fleet.md`): a
  change whose charters' needs pass `fleet.needs_bytes` is accepted, and
  the next start refuses as `NeedsPastLimit`, naming the bound to raise.
  Default: so; a proposal's admission that priced its needs would refuse
  it before it is held.

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
- **A transcript's load with no slot** (`briefs.md`, 1.1, `executors-transcripts.md`, 1): a window
  has no capacity answer, so nothing answers one for room: the engine keeps the
  activation due while no load is free, and a window's later pages
  in the load its first took (`decisions.md`, 1.3). A window
  answered `Failed` for room would count it as transient,
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
  (smith's `domain/session-context.md`, section 1) resumes rather than starts
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
  changing on its list (`settings-catalogue.md`, 2). Default: one request, and a name's figures are
  what its first read gave, a change of settings changing them.
- **What a model is good at:** a description in configuration, or
  structured capabilities a planning agent could filter on. Default: a
  description (`briefs.md`, section 1).
- **A release's removed default that a change names** (a template
  changed to run a charter the release drops): the start is refused,
  naming both (`settings-store.md`, 1), or the default is kept as a change of its own,
  recorded in the release's revision. Default: refused, as a document
  naming a charter it lacks is. The refusal is the backstop: a release
  that drops such a default carries its owner's written job for the
  stored changes that name it (`../../protocol/store/jobs.md`, 4),
  so a release that leaves those references unrepaired meets the refusal.
- **An application's kinds pinned,** a value read from a task's
  revision as a charter is, should an application's setting shape what a
  running task is given. Default: never pinned (`settings-application.md`, 1).
- **Delegates on their parent's revision,** so that a goal's subtree
  keeps the settings its plan was made under, as a change's option.
  Default: later; delegates take the newest (`settings.md`, 3).
- **A place a program started without its tools at,** for other tasks
  (`fleet.md`): the fault keeps only its own task off that source's
  login on that worker, since other runs went fine on the same account
  and hosts while two tasks failed there (the request "Bug fixes C4",
  2026-11-18). Default: so; every task kept off the place, and the
  deployment's document naming it, if such faults come to strike
  every run there.
- **Other init faults as a place's** (smith's `protocol/programs-claude-code.md`,
  1.6): an init lacking a granted built-in, or listing one not granted,
  stays the run's own failure, though a program's version on one host
  may cause it. Default: so, until a deployment meets it.
