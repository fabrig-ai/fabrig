# jig's design

Provisional, 2026-10-06, revised 2026-10-22. What jig is, how an
application is built around it, and what keeps the application's wiring
from breaking what jig promises. This document comes first. The documents
it lists in section 12 go deeper into each part. The mechanics are those
of skein's `programming-model.md`. What is still open is listed in
section 14.

## 1. In one page

- **A kit for agentic applications.** jig runs work done by agents,
  procedures and people, keeps that work in its own store, and drives
  external systems through connectors. It aims to be as flexible as a
  coding agent at a terminal, and adds what such an agent lacks:
  - work that outlives processes and machines;
  - many agents at once;
  - rules no agent can loosen;
  - effects on other systems made once.
- **Five primitives** (`domain/model.md`):
  - the **task**: what is asked;
  - the **executor**: who does it, which is an agent, a procedure or a
    person;
  - the **message**: everything a task hears once it has started;
  - **authority**: what a task may do, narrowed whenever it delegates;
  - the **connector**: an external system.

  Everything else is built from them: plans, coordinators, chats, goals,
  reviews.
- **The application owns the root.** skein's model nests step machines
  inside one another, and the top one is always the application's. The
  application's root domain composes jig's core with the application's
  connectors and its own domain logic, and translates between them
  (programming-model.md, 4.5). Extension is composition: no traits, no
  generic items, no callbacks. That is what makes jig a kit and not a
  framework.
- **The toolkit rule.** The application owns its roots and its loops,
  its server and its URL space, its presentation and its tree. jig gives
  parts, which the application composes, and decides nothing that is the
  application's. A part of jig's that would decide for it (a page, a
  path, a guard, a confirmation) is optional: a default or a reference,
  never mandated.
- **jig gives a mid-level core.** The core is the parent of tasks,
  authority, people, the fleet, accounts, briefs, notes and views. It
  routes among them and keeps the engine's numbers, its runs and their
  calls, and the order of restart. The root's code grows with the
  application's connectors, not with jig's internals.
- **Rails, not boilerplate** (`rails.md`, section 1). The root routes and translates;
  it never decides or orders. The rails that keep this true:
  - the commit barrier is a generic container in skein-lib, the
    journal, and it is the root's only way out;
  - the core runs restart and effects as a script;
  - every part declares its worst cases, and the journal asserts them;
  - a conformance world crashes the application's domain at every
    commit;
  - a reference root is the template every application copies.
- **Connectors meet the core through one vocabulary** (section 7):
  resources as paths, effects keyed, topics, procedures' steps,
  requirements as verdicts. A connector keeps its own state per task,
  keyed by the task's number. The core never sees a connector's types.
- **The store, the hosts and the client follow the same pattern.**
  - The store holds jig's records and the application's, in one commit.
  - Agents are optional. An application may have none, run them inside
    its engine, or run them on workers, in workspaces of its own kinds.
  - The client is one, `jig-client`, with no view (`client-stack.md`, section 1). It
    carries jig's primitives as plumbing, and the application's own
    families as bytes on the same connection; a shape of work, such as a
    chat, is sugar built from the plumbing. Presentation, the server and
    the URL space are the application's, and jig's reference UI is
    optional. It runs on any platform.

  In each case the application composes jig's part with its own: in its
  roots, in its protocol layer, and in its client.
- **The core never depends on smith.** Neither does the hub, which
  carries smith's payloads as bytes. Only the roots that compose smith's
  agent capabilities, the protocol layers' pieces that decode or encode
  smith's payloads, and the client's decoding of turns below its
  surface, link smith. An application without agents never does.
- **Flexibility and orthogonality over convenience.** An application
  writes some predictable code to wire jig in. Agents write such code
  well, and getting it wrong cannot break a promise silently.

## 2. What jig is for

- **What an application gets** is what is hard to get right, held to the
  promises of `domain/model.md`:
  - authority holds;
  - every effect made once, as far as its system allows: keyed and
    conditional effects at most once, others held for a person when
    their outcome is uncertain;
  - recorded spend never passes a budget;
  - nothing leaves the engine before the commit it follows from;
  - order: dependencies first, one writer per resource;
  - nothing is lost, nothing is written over;
  - what the engine holds is bounded.

  jig's referee checks these promises in the application's own worlds,
  with the application's wiring and its systems' fakes (`rails.md`, 1.6).
- **What an application brings:**
  - its connectors: the systems it drives, their procedures, the reads
    they give agents;
  - its charters: what its agents are told to be;
  - its policy;
  - its presentation and its server, and its own families on jig's
    client.

  For example, `ops` brings an observability and an infrastructure
  connector, whose procedures remediate and provision, and charters for
  triage and incident leads (`examples.md`).
- **What jig is not:**
  - a workflow catalog: structure is the work's, made by agents with
    tools;
  - an agent: that is smith;
  - io: that is skein.

## 3. The system

```
people    a client: the application's presentation, around jig's client
engine    where work is decided: the application's process, around jig's core
store     skein-kv inside the engine: jig's records and the application's, committed together
workers   if any: execution hosts, the application's process, around jig's host
agents    if any: smith's, inside the engine or one run per agent process on a worker
systems   what the application's connectors drive
```

- **One engine per deployment.** It is the store's only writer and the
  only client of every connector's system (`domain/model.md`).
- **The engine decides, hosts host, agents think.** A host is the engine
  itself or a worker that dials it (section 9). Agents reach the engine
  only through tools their host relays.
- **Every process is the application's.** The application writes the
  root, the protocol layer, `iterate` and `main`, as with any service on
  skein. jig supplies the parts beneath its roots, and the pieces of its
  protocol layers that speak jig's vocabularies: both sides of the
  workers' channel, the LLM accounts', the host tools' translation, the
  clients' component and the client's wire, and the store, which holds
  jig's records and the application's (`protocol/components.md`, section 1, and `protocol/README.md`, section 9
  ; `protocol/store/README.md`). In a client, the application holds jig's
  client inside its own presentation (`client-stack.md`, section 1).

## 4. The application owns the root

```
application root    routing between its children, translation; its store, channel and web vocabularies
├── core            jig: tasks, authority, people, fleet, accounts, brief, notes, views, and their routing
├── <connector>     the application's: one subtree per system it drives
├── …
└── …               the application's own child domains, if it has any
```

- **Control.** The application chooses its connectors, what it adds
  around the core, its limits, and its entrances. The core is the top of
  a subtree of its own, as a connector is.
- **Composition only.** Step code defines no traits or generic items, and
  uses no `dyn` or closures (programming-model.md, 10.3). The root
  therefore matches on what the core and each connector ask for, and
  calls the one each request goes to.
- **Change arrives on purpose.** Matches are exhaustive, with no `_`
  arm, so a new variant in jig breaks every application's build in one
  place. A change in jig reaches an application through a new pin, and
  the compiler lists what the application must decide.
- **What the core never knows:** any connector's types, the application's
  records, and any wire format.
- **An escape hatch.** The core's children are public crates. An
  application may compose them without the core. If it does, it takes on
  the invariants the core keeps, and the conformance world (`rails.md`, 1.6) still
  applies to it.

## 5. The core

- **What it holds:**
  - the routing among its children, for what arrives: a run's turn,
    call or answer; a person's request; a task due; a timer;
  - the deployment's numbers;
  - the runs in flight, and the calls committed for each since its last
    turn;
  - the gathering of a run's brief;
  - the restart script (`rails.md`, 1.4).
- **Its vocabulary toward the root:**
  - the store: its own records, saved, erased, loaded and restored, as
    every child's are (`domain/engine/README.md`);
  - agents' hosts: the workers' channel, the engine's side, or the host
    inside the engine (section 9);
  - people's requests, and the replies to them;
  - LLM accounts: signed in and out, keys added and removed, wanted,
    granted, failed, and the limited, refused and unauthorized notices;
  - the connectors' vocabulary (section 7).
- **Connectors are numbered.** The core knows from its configuration that
  connector `k` exists and that its resources are paths under its
  prefix, and nothing more.
- **Its children** are each a child domain of their own, with their own
  vocabulary, limits and worlds (`domain/engine/README.md`, section 3):
  - **tasks**, the center of the core, which keeps every live task:
    batches, lifecycle, holds, inboxes, wakes, proposals;
  - **authority**: policy as data, checks and budgets;
  - **people**: identities, sign-ins, roles, requests, inboxes;
  - **fleet**: agents' hosts, slots, placement, attempts;
  - **accounts**: the LLM accounts, each a holder's with one LLM
    provider, with their sources, states and uses, and the application's
    credentials runs are lent (`domain/accounts.md`);
  - **brief**: a run's context, in typed sections within a byte budget;
  - **notes**: what agents learn, in scopes;
  - **views**: parties' watches, each delivered its subject's current
    document as it changes.
- **Agents are optional.** An application without them configures no
  charters. The batch check then refuses tasks with agent executors, and
  the fleet and accounts stay idle.

## 6. The rails

Moved to [`rails.md`](rails.md), section 1.

## 7. Connectors

- **The vocabulary between the root and a connector is jig's API**
  (`domain/connectors.md`):
  - resources named as paths of bytes, with roles: owned, fork,
    context; the objects behind them owned or participating;
  - write holds, and one writer at a time;
  - effects keyed per connector, made from the outbox, with uncertainty
    after a failure or a restart;
  - effect and read kinds as numbers, whose order authority holds as
    data;
  - topics, and news for subscribers, classified as waking, kept or
    dropped;
  - procedures' steps and their decisions;
  - projections: what a connector writes for people to read;
  - brief sections and reads for agents;
  - workspaces: what a worker prepares for a run (section 9);
  - drift, and adoption.
- **Requirements are verdicts.** Facts such as CI at a head or a
  reviewer's approval are a connector's. The connector that makes an
  effect judges its own facts and gives a verdict: met, wait, or refuse
  with a reason. The core holds one rule: no effect without its verdict.
- **What the core does not act on, it does not hold.** A connector keeps
  its own state per task, keyed by the task's number, and the root joins
  the two. What the core orders but does not read (a brief section, a
  read's answer, a connector's result) it names by token and size, and
  the root puts outputs together from their owners' parts
  (`domain/model.md`, section 5).
- **The set is closed in each application.** The application's root
  knows every connector and matches on them exhaustively. Nothing below
  the root knows any connector but itself.
- **A missing place in the vocabulary is jig's to add,** since no root
  can add one. A second application tests the vocabulary for that reason.

## 8. The store

- **jig's part:**
  - the core's records and keys;
  - their encoding, ordered as the keys are and versioned;
  - transcripts as files, one per task;
  - the secret records: minted credentials' and services' credentials'
    digests, and LLM accounts' records and keys (`protocol/store/secrets.md`
    ). A program's login is the program's own, on its host.
- **The application's part:**
  - its connectors' records: outboxes, procedures' states, what was made
    by key, and each connector's own state;
  - the root's keys, records and writes, which wrap both jig's records
    and its own;
  - its families' keys and encodings, handed to jig's store as bytes
    under the application's tag (`protocol/store/README.md`, section 7).
- **One commit spans both,** through the journal. The live records of
  both are loaded by the core's restart script (`rails.md`, 1.4). skein-kv is
  underneath, inside the engine (`domain/engine/decisions.md`, section 1).

## 9. Agents and their hosts

- **smith is the agent.** Nothing in smith assumes code, repositories or
  a workspace, so the same agent serves every application, told apart by
  its charters.
- **The core never depends on smith.** It keeps turns opaque, and reads
  only a turn's size, its spend, the last message it read and whether it
  opens a window. Charters are put into smith's form by one translation
  of jig's, outside the core (`domain/hosts/charters.md`).
- **What the core needs is a host for each run, not a worker.** The
  core's contract with a run is:
  - a start: the charter, the brief, the current window of the
    transcript, the messages waiting, credential grants, an attempt
    number, the run's identity;
  - turns sent up, committed and acknowledged;
  - calls relayed and answered once, and messages relayed;
  - an answer at the end;
  - cancellation, and graces for a host that is lost.

  smith's host contract carries the same, as frames over a channel or as
  calls inside one process (smith's `domain/host.md`, sections 3 and 9).
- **Three shapes of application,** each with its costs in
  `domain/hosts/README.md`, section 3:
  1. **No agents,** procedures and people only, such as an approval
     pipeline: no host, no smith, no LLM accounts (section 5).
  2. **Agents in the engine,** such as a triager or a chat assistant,
     whose tools are the engine's and the connectors' reads: the
     engine's root composes the hub with `smith-inline-agent`, and the
     fleet places runs on the engine's own slots, with no workspace and
     no process containment.
  3. **Agents on workers,** for runs that execute commands, hold
     workspaces or need many machines: each run is its own agent
     process, whose tree a stop ends. On one person's machine the engine
     and its workers may share one process, the agents still processes
     of their own (`protocol/service.md`, section 10).

  An application may mix the second and third shapes: a charter says
  which hosts may run it.
- **One host for every shape that runs agents** (`domain/hosts/README.md`):
  - `jig-domain-host`, the hub: slots, admission, fenced attempts,
    relays, turns and the answer kept until acknowledged, cancel;
    composed by the engine's root for its own slots, and by a worker's
    root;
  - its agent, a capability of two kinds, both smith's, speaking smith's
    parent face: agent processes, supervised through
    `smith-host-domain`, on a worker; `smith-inline-agent`, smith's
    domains in the host's process, in the engine, or on a worker whose
    root chooses it;
  - smith's payloads, bytes in the hub and typed in the agent,
    translated by hand-out to the root's protocol layer
    (`domain/hosts/hub.md`, 8);
  - its link to the core, its root's: a channel the worker dials, or the
    core's fleet directly, in the engine.
- **Workspaces are the application's.** A connector says which of its
  resources a run needs prepared. A connector whose resources are not
  files gives a run tools instead. The worker's root is the
  application's: it composes jig's host, smith's host domain and the
  application's kinds of workspace (a checkout of a repository, say). The
  engine has no workspace, by policy.
- **Host tools:** the engine's tools are jig's, and each connector's
  reads are declared by the application.

## 10. The client

Moved to [`client-stack.md`](client-stack.md), section 1.

## 11. Building an application on jig

| Part | Written by |
|---|---|
| the engine's root domain: routing, translations, its vocabularies | the application, copied from the reference root |
| the core and its children | jig |
| connectors: their domains, protocol layers and fakes | the application |
| the journal | skein (skein-lib) |
| the store's encoding: of jig's records / of the application's, and their composition | jig / the application (`protocol/README.md`, section 9) |
| the protocol layers' components for jig's vocabularies: the workers' channel, accounts, host tools, configuration / its connectors' protocols, its parts' encodings | jig / the application (`protocol/components.md`, section 1, and `protocol/README.md`, section 6) |
| the engine's protocol layer's routing, `iterate` and `main` | the application, copied from the reference, around jig's service parts (`protocol/service.md`, section 7, and `protocol/service-parts.md`, section 1) |
| agents' hosts: the hub, the charter's translation / the worker's root, kinds of workspace, `main` (none without agents) | jig / the application |
| the agent and its capabilities, `smith-host-domain` and `smith-inline-agent` / its charters | smith / the application |
| the UI kit: the tree and its vocabulary, the builder, the diff, the phrasebook's machinery, Markdown, scenes, the outline's rules; the text shell; the browser parts and their default mapping; the bundler; the fake person, its browser face and the harness | skein (`skein-ui`, `skein-ui-text`, `skein-ui-web`, `skein-ui-bundle`, `skein-fake-person`, `skein-fake-person-browser`: skein's `ui/README.md`, section 2) |
| its presentation: its routes and URLs, its screens, words and look, its confirmations and drafts, its home and navigation; on skein's UI kit, on anything else, or on nothing | the application |
| the client: `jig-client`, its domain, jig's wire and its link; jig's helpers, `shapes`, `facts`, `thread` and `words` / its families' wire, carried as bytes by the same client, and which helpers it uses | jig / the application (`client/domain/client/README.md`; `protocol/client/README.md`) |
| the server: the clients' component, transport-agnostic, and credentials, minted, verified, expired and revoked; the printed link and services' credentials; the server part (`protocol/service-parts.md`, 1.4) / the clients' listener on that part, its routes and URL space, its pages and bundle, the component mounted under a prefix of its choosing, its sign-in flows and every guard | jig / the application, copied from jig's test application's (`protocol/service.md`; `protocol/client/README.md`) |
| the transport: jig's default HTTP mapping at both ends, `jig-client-protocol-http` and `jig-protocol-engine-clients-http`, with its example guards, and attack stories in jig's test kit / the default mapping as it is, wrapped, or its own: HTTP, a local socket, or in-process | jig / the application (`protocol/client/default-http.md`; `protocol/browser/README.md`) |
| the reference UI: pages, cards, the frame, the confirmation, drafts and the view, on skein's UI kit, over the native shell and jig's browser parts, and jig's test application's browser shell to copy / whether it takes it, whole or in part; its families' views beside it; its browser shell, `<application>-client-shell-web`, copied from jig's test application's, `jig-client-test-shell-web` | jig / the application (`client/domain/pages.md`; `client/domain/view.md`; `protocol/browser/README.md`) |
| the request table | skein (skein-lib) |
| the conformance world, the referee, jig's check, the attack stories, scripted workers and people, the client's test kits / its worlds and its systems' fakes | jig / the application |

Before code, an application writes its connectors' vocabularies and
limits, and the worst cases of its root. Then, in order:

1. its connectors' domains;
2. its root;
3. its worlds, on jig's conformance world;
4. its protocol layers, its `iterate` and `main`.

What it finds missing in jig along the way is added to jig, with the
application as its first user.

## 12. Documents

- **`rails.md`:** the rails jig supplies across its layers.
- **`client-stack.md`:** the client stack from application root to transport and presentation.
- **`crate-names.md`:** how jig's crates and an application's crates are named.
- **`domain/`:** jig's domain layer: the model, tasks, authority,
  connectors, the core, the application's root, parties, hosts, LLM
  accounts (`accounts.md`), the LLM providers' terms they keep
  (`providers.md`) and a program's login in a lent terminal
  (`terminal-login.md`), and testing. Its `README.md` gives the reading order
  and the names, and the decisions it took on what this document left
  open. The tasks' documents include failures and holds (`tasks/lifecycle-holds.md`)
  and dependencies and endings (`tasks/lifecycle-endings.md`). The core's
  include its vocabulary (`engine/vocabulary.md`), decisions and the store
  (`engine/decisions.md`), and its world (`engine/world.md`); transcripts
  (`engine/executors-transcripts.md`) and tools (`engine/executors-tools.md`);
  and application settings (`engine/settings-application.md`), proposals
  (`engine/settings-proposals.md`), the settings store (`engine/settings-store.md`)
  and the model catalogue (`engine/settings-catalogue.md`).
  Authority covers budgets and spend (`authority-budgets.md`) and checks
  (`authority-checks.md`). The connector documents cover effects and the
  outbox (`connectors-effects.md`), facts and events (`connectors-facts.md`),
  and procedures (`connectors-procedures.md`). The account documents cover
  sources and states (`accounts-sources.md`), provider lists (`accounts-lists.md`),
  and uses and account selection (`accounts-uses.md`). The testing documents
  include fakes and kits (`testing-fakes.md`).
- **`client/domain/`:** jig's client: `jig-client`, what it holds and
  its surface; the API between a client and the engine: plumbing
  operations, subjects and reads, documents and their revisions,
  versions; owners; the reference UI, its pages and cards and its view
  on skein's UI kit; and testing. Its `README.md` gives the reading
  order, the names, and the decisions it took.
  The client documents include `client/domain/client/vocabulary.md`,
  `client/domain/client/small-store.md` and `client/domain/client/sign-in.md`.
  The contract documents include `client/domain/contract/answers.md`,
  `client/domain/contract/reads.md` and `client/domain/contract/watches.md`.
- **examples.md:** the example application, `ops`, and the reference
  root.
- **`protocol/`:** jig's protocol layer: the components an application's
  protocol layers own, the engine's and the worker's services, the
  workers' channel, LLM accounts and LLM calls in a host's own process,
  configuration, templates among it, the client's protocol: jig's
  messages, their records and the clients' component; and jig's default
  HTTP mapping with its example guards, and jig's browser parts on
  skein's. Its `README.md` gives the reading order, the names,
  what jig needs from skein and smith, and what the domain owes, and
  `protocol/store/README.md` the store.
  The channel's documents include `protocol/channel/owners.md`,
  `protocol/channel/liveness.md`, `protocol/channel/worker-link.md` and
  `protocol/channel/testing.md`. The store's include `protocol/store/keys.md`,
  `protocol/store/commits.md`, `protocol/store/growth.md`,
  `protocol/store/testing.md`, `protocol/store/opening-owners.md` and
  `protocol/store/opening-rehearsal.md`.
  `protocol/components.md` covers composition, `protocol/needs.md` what jig
  needs from skein and smith, and `protocol/decisions.md` the layer's decisions.
  `protocol/service-parts.md` describes the service parts and
  `protocol/service-testing.md` their tests and worlds.
  `protocol/configuration-engine.md` gives jig's part of an engine's
  configuration, and `protocol/configuration-templates.md` its templates.
  The client's protocol includes `protocol/client/versions.md`,
  `protocol/client/small-store.md`, `protocol/client/shells.md` and
  `protocol/client/testing.md`.
  The browser's layer includes `protocol/browser/application.md`,
  `protocol/browser/loop.md`, `protocol/browser/sign-in.md` and
  `protocol/browser/serving.md`.

## 13. Conventions

- **A bare file name** names a document in this directory;
  `domain/model.md` names one of the domain's.
- **skein's foundation documents** are named by their own file names:
  `programming-model.md`, `testing-strategy.md`, `notes.md`. skein's
  design documents are named with skein's name: skein's `runtime/lib.md`.
- **smith's documents** are named with smith's name and their path in
  smith's `docs/design/`: smith's `domain/run.md`.
- **An application** appears only as an example, and says so.
- **Each document** says in one page what it is. Then it gives its
  structure, its parts, what it owes and is owed below the domain, its
  world, and what is open.

## 14. Open questions

The domain's design took decisions on most of what this document left
open (`domain/README.md`, section 5), and the client's on its own
(`client/domain/README.md`, section 5). What remains here:

- **Versioning:** how jig is released and pinned beyond a revision of
  its repository, and how a change to its vocabulary reaches every
  application. What is settled: an application takes `jig-client` and
  the engine's parts from one pin of jig, and each owner's client
  records are versioned apart, with a typed refusal
  (`client/domain/contract/README.md`, section 6).

## 15. Crates' names

Moved to [`crate-names.md`](crate-names.md), section 1.
