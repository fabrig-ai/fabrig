# The protocol layer

Provisional, 2026-10-03, revised 2026-11-02. temper's protocol layer:
how temper composes jig's protocol layer with components of its own (its
forge's, its checkout's, its client's) into the processes of one binary.
jig owns the layer's shape (jig's `protocol/README.md`) and the services'
(jig's `protocol/service.md`); this document says what temper adds and
the crates it is made of. The mechanics are those of skein's
`programming-model.md` (sections 2, 4 and 9); the entities it carries are
those of `../domain/`. Each boundary of temper's own is designed in a
document of its own (section 2). What is open across them is listed in
section 11. The cluster, temper as several processes of one machine for
its tests, is `cluster.md`.

## 1. In one page

- **Translation, and nothing else.** The protocol layer turns bytes into
  the domains' entities and back, and decides nothing about the domain
  (programming-model.md, section 4). The domain decides deadlines,
  retries and what a failure means; the protocol layer runs the call, its
  timer and its one attempt.
- **jig's protocol layer, composed.** jig ships the components for its
  own vocabularies: the store, the workers' channel, LLM accounts,
  clients, configuration (jig's `protocol/README.md`, section 5).
  temper's services own them as children beside smith's, skein's and
  temper's own components, and route between them, io and the roots, as
  jig's reference service does (jig's `protocol/service.md`, section 2).
- **What temper adds:** the forge's protocol (skein's forge component,
  with its webhooks' listener, handed temper's values and routed to the
  root and the forge connector); the checkout's io
  (the git component); its parts' codecs on jig's channel; its families
  in jig's store; its git identities; its host tools' translation; its
  client (its wire, its client root and view on `jig-client`, its
  terminal); the clients' listener, on which it mounts jig's clients'
  component and serves its forge routes; the setup host, skein's, in the
  engine (forge/engine.md, 6); and the forge's setup's commands outside the
  services (setup/README.md). Each is a crate of its own (`components.md`).
- **Foreign protocols are stacks of skein's machines.** Forgejo's REST
  API and GitHub's are skein's forge kit, each a dialect of it, over
  skein's HTTP client and JSON (skein's `forge/README.md`); git's arguments in and text out are temper's decoders
  over io's processes. temper's records on jig's channel, in jig's store
  and on the client's wire are generated from schemas by skein's
  generator.
- **Standalone first.** `temper-box serve` runs the engine and one worker in
  one process, on one loop, joined by jig's in-process address; the
  worker's agents are child processes of temper's own binary (`temper-box
  internal agent`); a person's client reaches the engine on loopback;
  the forge is polled, and hinted by its webhooks on loopback when they
  are configured (section 6; forge/README.md, section 4). Hosts apart,
  webhooks from another host, the forge as a sign-in provider and the
  forge's client documents come after (section 11). The browser is a
  person's client on loopback too (`../client/web/README.md`).
- **The cluster, for tests.** The same `temper-box serve`, on a
  document of another shape, is an engine alone or a worker alone. One
  engine alone and its workers alone, each a process of its own on one
  machine, joined over loopback in plaintext, with the fakes as
  processes beside them, are the cluster that temper's process tier
  runs from one description (`cluster.md`). It is no
  deployment's shape: a box keeps its one worker inside `temper-box serve`.
- **The box's binary, `temper-box`:** `serve`, `internal agent`, `open`,
  `program-login`, the forge's `forge prepare`, `forge run`, the bare form, and the
  agents' `ctl`, which `../client/ctl/README.md` designs (`binaries.md`).
- **No retries below the domain.** A call goes out once and ends once,
  with the failure the domain's vocabulary defines (`components.md`).
- **The forge's load follows its changes.** The calls a pass makes grow
  with what changed on the forge, never with its history, its labels or
  the items temper holds, and the worlds count them by HTTP request
  (forge/README.md, section 5).
- **Secrets never reach a domain.** A domain names a credential by number
  and generation; values live in the protocol layer's tables and in jig's
  secrets' store: LLM accounts' as jig keeps them, and each forge's
  records under temper's tag, from which its access and its git identity
  are made (section 8; credentials.md).
- **Tested at every tier:** codecs and parsers against golden bytes and
  transcripts of real peers; the forge component and the git component in
  worlds of their own; temper's client joined to the engine's root; the
  one-process world on skein's simulator; the real binary with an owned
  Forgejo, its runner and real git; a live tier with a real model
  (section 10; `../testing/README.md`).

## 2. Reading order

1. **jig's `protocol/README.md` and `protocol/service.md`:** the layer
   temper composes, the service parts jig ships for it, and the routing
   temper copies.
2. **README.md:** this overview.
3. **configuration/README.md:** what `temper-box serve` and its other commands are
   told at start.
4. **credentials.md:** the secrets temper's processes hold: LLM accounts
   as jig's, the forges' records in jig's secrets' store, the forges'
   access, git identities, and the values clients' requests carry.
5. **channel.md:** temper's parts on jig's channel, and a git identity's
   value.
6. **store.md:** temper's families in jig's store.
7. **llm.md:** where LLM calls are made, host tools, the charter and a
   run's result.
8. **git.md:** the checkout's io: the git component.
9. **forge/README.md:** the forge's protocol layer on skein's forge kit: what
   temper hands the forge component, the setup host, its marks, its
   webhooks' address, and what it relies on of Forgejo and GitHub.
10. **setup/README.md:** the forge's setup: connected, managed and owned, the
    setup host in the engine, and `temper box`.
11. **sign-in.md:** people's sign-in with Google on a deployment's
    public address: its members, routes, endings and binding.
12. **`../client/README.md`:** temper's client: its client root on `jig-client`,
    its pages, the forge's client family, its view, the terminal, and the
    clients' listener of temper's server; then **`../client/ctl/README.md`**, `temper
    ctl`, the agents' tools.
13. **`../testing/README.md`:** the tiers below the domain.
14. **`../client/web/README.md`:** the browser, the second shell of
    temper's client.
15. **documentation.md:** the user documentation compiled into the
    binary, `temper help`, and agents' `read_docs`.
16. **tooling.md:** the contract with a project's own `prepare` and
    `key` commands, run as jobs on a worker for a landing by turn.
17. **baseline.md:** the store every later build upgrades from: its
    kept texts, the patch sets temper hands jig's store, the golden
    store, and the gate on stored changes.

## 3. Names

jig's names keep their meaning here (jig's `protocol/README.md`, section
3; jig's `domain/README.md`, section 4): the hub, the core, the fleet,
the worker's link, credential grant, an application's credential,
payload version, profile, the clients' component, the default HTTP
mapping, the reference service (`jig-test-service`), the reference
binary (`jig-test`).

| Name | What it is |
|---|---|
| standalone | one process on one person's machine: `temper-box serve` with the engine and one worker joined in process (section 6) |
| engine alone | `temper-box serve` on a document with an engine and no worker, its workers' listener on loopback (`cluster.md`, 1) |
| worker alone | `temper-box serve` on a document with one worker and no engine, dialling an engine alone on loopback (`cluster.md`, 1) |
| the cluster | one engine alone and its workers alone on one machine, with the fakes as processes, as temper's process tier runs it (`cluster.md`) |
| the cluster's description | `bin/temper-box/tests/fixtures/cluster/`: the cluster's processes and their documents, read by the process tier (`cluster.md`, 5) |
| the engine's service, the worker's service | each `iterate` with its root and protocol layer, in `temper-engine-service` and `temper-worker-service`, composed by `temper-service` (sections 4 to 6; `services.md`) |
| the routing | the protocol layer's code between io, components and the root (jig's `protocol/service.md`, section 2) |
| the forge component | skein's `skein-forge-connection` (forge/README.md; skein's `forge/connection.md`) |
| the git component | `temper-worker-protocol-git` (git.md) |
| temper's parts | the workspace items, the saved-work branch, the work, the preparation (channel.md) |
| payload version, store version, client version | temper's three own versions: `PAYLOAD_VERSION`, `STORE_VERSION`, `CLIENT_VERSION` (section 7) |
| temper's families | the forge connector's records in jig's main store (store.md) |
| a forge's records | its credentials, ids and accepted installations: rows of jig's secrets' store under temper's tag (credentials.md, 2.1) |
| git identity | one of temper's application credentials, one per forge: Forgejo's git token, or a GitHub App's installations' tokens, with a login and an author (credentials.md, section 4) |
| the identities' table | the protocol layer's table of git identities' values, on the engine and on each worker |
| a forge's access | what the forge component authenticates with: Forgejo's API token, or a GitHub App's id and key and its installations' tokens; the component's only (credentials.md, section 5) |
| the setup host | skein's in-process host for the setup's plans, a child of the engine's protocol layer (forge/engine.md, 6) |
| the forge's client family | temper's client side of the forge connector: its words, and the adoption form (`../client/forge/family.md`) |
| the terminal shell | `temper-client-shell-terminal` (`../client/line.md`) |
| the browser shell | `temper-client-shell-web` (`../client/web/README.md`, 2.1) |
| the bundle | the browser shell's module, glue and stylesheets, skein's bundler's output, which the binary holds and the clients' listener serves (`../client/web/README.md`, 2.2 and 2.6) |
| the tools | the agents' interface to temper, `temper ctl`: `temper-client-tools` and `temper-client-shell-cli` (`../client/ctl/README.md`, section 4) |
| the clients' listener | `temper-engine-service`'s listener for clients, on which jig's clients' component is mounted with jig's default HTTP mapping, under `/jig` (`../client/engine.md`) |
| the fake git program | git as the worker runs it, faked: its core `crates/temper-fake-git`, `temper-box fake git`, and its hosted half `crates/temper-fake-git-hosted` (`../testing/fakes.md`, 3) |
| the throwaway Forgejo | a Forgejo started by a test script on loopback, with SQLite in a scratch directory, removed after (`../testing/process-tier.md`) |
| locate | the forge connector's read of a repository's id by owner and name (skein's `forge/forgejo.md`, 1.2) |
| the webhooks' listener | the forge component's own listener for Forgejo's signed deliveries, plaintext on loopback, one path per forge (skein's `forge/connection.md`, section 6; forge/README.md, section 4) |

The forge's setup's names are setup/README.md's: connected, managed and owned
(setup/README.md, section 2), the forge's directory (setup/README.md, section 3), reconcile, a plan of the
setup host's (setup/reconcile.md),
the pin (setup/owned.md, 1), the preparation (setup/owned.md, 3), the box and its steps (setup/box.md).

## 4. The engine's service

In the shape of jig's `protocol/service.md`, section 2:

```
temper-engine-service                   iterate: io ↔ components ↔ root
├── temper-engine-domain                the root, with the journal (../domain/root.md)
└── the protocol layer
    ├── jig-protocol-store              commits, loads, transcripts, secrets; temper's families (store.md)
    ├── jig-protocol-engine-workers     the worker's link; temper's parts as bytes (channel.md)
    ├── jig-protocol-accounts           LLM accounts (credentials.md, section 3)
    │   └── skein-oauth-accounts
    ├── the clients' listener           temper's, on jig's server part: TCP on loopback (../client/engine.md)
    │   └── jig-protocol-engine-clients-http   jig's default HTTP mapping, under /jig, with its example guards
    ├── jig-protocol-engine-clients     the clients' component: jig's messages; the forge's client family as bytes (../client/engine.md)
    ├── skein-forge-connection          the forge component: every kept forge's API; the webhooks' listener, TCP on loopback (forge.md)
    ├── the setup host                  skein-forge-setup's in-process host: the setup's plans (forge/engine.md, 6)
    ├── the identities' table           git identities' values (credentials.md, section 4)
    └── timers                          the root's and every component's deadlines
```

Starting, ending, `iterate`'s passes, room and timers are jig's
(jig's `protocol/service.md`, sections 3 to 5). What temper adds:

- **The forge component,** speaking each forge's API, Forgejo's REST API
  under `/api/v1` or GitHub's in skein's dialect, polled at the domain's
  cadences (skein's `forge/forgejo.md`, section 1). The root's
  `Request::Forge { call, repository, op }` goes to it; it answers each
  call with exactly one `client::Event::Answered { call, cost, result }`,
  which the routing hands the root as a forge event
  (`../domain/routing.md`, 1). An event the root cannot take yet comes
  back to the component, which holds it and offers it again (jig's
  `protocol/service.md`, section 2, room); one the root returns during
  its drain stays with the component until it closes (forge/engine.md
  ; `../domain/root.md`, 8.2). Each forge's access is the component's
  alone (credentials.md, section 5).
- **A forge's start check** (credentials.md, section 5; skein's
  `forge/connection.md`, section 3): when a forge refuses its access, or
  it answers as a user other than the forge's own, that forge stops,
  never the engine: the component gives `Stopped { forge, why }`, the
  deployment's page names it with Replace (`../client/forge/deployment.md`),
  and its calls are answered `NotConnected`; the engine and every other
  forge run on (forge/engine.md, 3). Records that do not fit the configured
  forge leave it not connected, named the same way (credentials.md,
  2.1). A failure of jig's secrets' store itself stops the engine
  (credentials.md, section 5).
- **The setup host,** skein's in-process host component, beside the
  forge component: the routing runs its plans (reconcile at each start
  of a managed or owned Forgejo, a connect's, an adoption's access) and
  carries what passes between it, the root and the forge's records. It
  holds an admin's token or a GitHub App's key only while a plan runs
  (forge/engine.md, 6; setup/README.md, the setup host in the engine). On close,
  the setup adapter retains closing requested separately from host
  `Close` sent; it drains host output independently of active notes,
  store, forge and events gates, sends `Close` once bounded host room
  permits, and completes only on actual `Closed` after io settlement
  (setup/reconcile.md, 1, and setup/testing.md, 1). No application note-pop order is required;
  active-phase gates and skein's host/caller semantics remain in force.
- **The webhooks' listener,** the forge component's own, when
  `application.webhooks` names it: each delivery verified as it streams,
  answered, then made a hint, which the routing hands the root as a forge
  event (forge/engine.md and forge/README.md, section 4). A hint is never a fact: polling
  finds what a lost delivery told of (forge/README.md, section 5).
- **temper's parts.** As it encodes an assignment, the routing encodes
  the root's `ForgeWorkspace` as temper's workspace items, with
  `temper-protocol-worker-wire`, making each repository's remote from its
  forge's `git` base and the repository's owner and name
  (forge/engine.md, 1), with the owner's account id on a GitHub
  forge, so workers need no forge configuration.
  As it decodes an answer, it decodes the work and the preparation into
  the root's `Parts` (channel.md, sections 4, 5 and 7;
  `../domain/root.md`, 6.4).
- **temper's families.** The root's forge records go to the store under
  jig's application tag, encoded by `temper-protocol-store`, in the same
  commits as jig's; the forge's restart step loads them as one interval
  (store.md).
- **The identities' table,** filled from each Forgejo forge's records at
  start and from each GitHub forge's minted installation tokens as they
  come, keeping each identity's two newest generations; the routing puts
  each value in the table, then gives the root one `Event::Credential`
  for it, before its restart for a Forgejo forge's, and
  `jig-protocol-engine-workers` fills grants' values from the table
  through the routing (credentials.md, section 4; `../domain/root.md`,
  6.1).
- **Host tools, the charter and results through `temper-engine-smith`,**
  called in the routing, with smith's codecs, in the step that routes
  them, keeping no state (llm.md, sections 4 and 5; `../domain/routing.md`,
  2).
- **The clients' listener.** The service owns it, and mounts jig's
  clients' component on it with jig's default HTTP mapping under `/jig`,
  with the mapping's example guards (`../client/engine.md`; jig's
  `protocol/service.md`, section 2). A path jig's component does not claim
  goes to temper's forge routes, `/forges/...`, a GitHub connect's
  browser steps, each a setup host's run (`../client/forge/served.md`, 1), then
  to the browser's page and bundle; every other path, and a `GET` of the
  printed link's route, is answered not found.
- **The client's application wire.** The forge's client family's
  requests are decoded by `temper-client-protocol-wire` in the routing
  and answered by the root; every other part of the forge's is answered
  as unknown (`../client/forge/family.md` and `../client/engine.md`). An adoption names a
  forge, an owner and a name; the routing completes it with the forge's
  host, the host of its configured `git` base with `:port` when it names
  one (configuration/engine.md), and hands the root the party's keyed
  request; the root locates the repository, then makes jig's adoption
  request on the party's behalf (`../domain/routing.md`, 4;
  `../domain/forge/adoption.md`).
- **Periods,** from the smallest source, as jig's reference service gives
  them (configuration/README.md, section 6).

## 5. The worker's service

In the shape of jig's `protocol/service.md`, section 6:

```
temper-worker-service                   iterate
├── temper-worker-domain                the worker's root: jig's hub, smith's host domain, the checkout
└── the protocol layer
    ├── jig-protocol-worker-link        the link to the engine; temper's parts as bytes (channel.md)
    ├── smith-host-protocol             one channel per agent process, over its pipes; LLM grants filled into its start
    ├── jig-protocol-worker-agents      the program sources and the worker route, between smith's host half and the link
    ├── agent processes                 the agent's program, temper-box internal agent or temper-agent, each in a process group of its own
    ├── the identities' table           git identities' values, from grants (credentials.md, section 4)
    └── temper-worker-protocol-git      the git component: the checkout's io (git.md)
```

No store; starting, ending and smith's payloads are jig's (jig's
`protocol/service.md`, section 6). What temper adds:

- **Agents run as the agent's program:** the worker's `agent` member
  (configuration/README.md, section 4), with the arguments it gives, its
  environment holding no credential: the box's `temper-box internal agent
  AGENT.json`, or a deployment's `temper-agent AGENT.json` (`binaries.md`, 3),
  each spawned from the copies the worker pinned at its start, the
  program's and AGENT.json's (`../domain/worker/README.md`, section 5).
- **A model list's launch** (smith's `protocol/agent.md`, 6.1) runs the
  same program, its arguments by the run's form, and smith's host half
  puts `--list` before them: `internal agent AGENT.json` lists as
  `internal agent-list AGENT.json`, so `temper-box --list internal
  agent-list AGENT.json`; `AGENT.json` alone, the `agent` member's
  `configuration`, lists as `AGENT.json`, so `temper-agent --list
  AGENT.json`. Any other form has no list launch, and a list's read
  fails with no process, as one with no usable grant does.
- **temper's parts.** The routing decodes the workspace items into the
  root's `wire::Workspace` before the root sees the assignment, and
  encodes the root's `wire::Work` and `wire::Preparation` beside an
  answer (channel.md, sections 4, 5 and 7).
- **Credentials.** `jig-protocol-worker-link` takes grants' values out:
  LLM grants go to smith's host half in its form, a credential's value
  with its scheme or a program source's name (jig's `protocol/llm.md`,
  section 6); git identities' go into the identities' table,
  from which the routing hands the git component an identity's value
  with each operation that names it; the component never reads the table
  (git.md, sections 2 and 5).
- **The checkout's io.** Each `git::Op` the root asks for goes to the git
  component, which answers it with exactly one terminal; `Make` is file
  operations under the worker's root directory (git.md, section 4).
- **Local channel refusals** use jig's public `u16` constants
  `REFUSED_LIMITS = 2` and `REFUSED_FRAMING = 3` from
  `jig-protocol-worker-link`, including diagnostic matches (jig's
  `protocol/channel/README.md`, 9.1; skein's `net/channel.md`, 5.1). These are
  channel reasons, separate from jig's application reasons at 256 and
  above. Existing worker diagnostic tests check those matches; the
  wire and `Closed` facts keep their values.
- **Starting:** configuration, the worker's root directory opened, the
  agent pinned in it (`../domain/worker/README.md`, section 5), the
  components, then the link dialled at once (jig's
  `protocol/service.md`, section 6).
- **Under a service manager,** as a deployment runs its workers
  (`temper help server`), the worker's unit kills its whole control
  group when it stops or its process dies, systemd's
  `KillMode=control-group`, so no agent, program or command of the
  worker's outlives it, and its next start finds nothing of its previous
  life still writing in its workspaces
  (`../domain/worker/README.md`, section 5).

## 6. One process

Standalone is jig's one process (jig's `protocol/service.md`, section 10):

- **One service, one loop.** `temper-service` owns the engine's
  protocol layer and the worker's as components, each with its own root
  and journal, sharing io and nothing else; nothing crosses between them
  but bytes. Each half is a crate of its own, which the engine's and the
  worker's binaries link alone (`services.md`).
- **Joined by the in-process name `workers`:** the engine's workers'
  listener and the worker's link (configuration/README.md, section 2). The
  link's opening, authentication, liveness, relays and grants run as
  between hosts, in plaintext, with no TLS.
- **A client on loopback,** over TCP to the clients' listener: `temper-box
  open`, signing in by the link `temper-box serve` prints at start, or
  `temper box` before it starts serve (`setup/box.md`, 2).
- **Agents and git as child processes,** each in a process group of its
  own, which a stop signals whole (jig's `protocol/README.md`, section
  10); containment after a forced stop remains open (section 11). An
  agent's program, `claude`, is the agent's child in a group of its own
  (smith's `protocol/programs.md`, section 2), so killing an agent's
  group leaves its program running: only the agent's own stop ends it.
- **The forge outside,** reached over HTTP, and over TLS when it is not
  on loopback; its deliveries reach the webhooks' listener on loopback,
  so a forge on another host polls alone (forge/README.md, section 4).
- **The core knows its workers are in process:** the engine's workers'
  listener is the in-process name, from which jig's configuration tells
  the core so (jig's `protocol/configuration.md`, section 3;
  `../domain/root.md`, section 2).
- **Starting and ending together:** configuration for both, then the
  engine's start, then the worker's. A signal drains the engine; once it
  has drained, the service stops the worker, with no grace for its turns
  and answers: its runs are cancelled, each agent process ending its
  program before it exits, within smith's cancel, then kill, and the
  service closes the worker once no agent process is left, its git
  invocations' groups killed as io closes them (jig's
  `protocol/service.md`, section 10). The service exits only after. A
  second signal closes the worker at once, its agent processes' groups
  killed, and a program they started outlives them until agents run in
  contained trees (section 11). No worker outlives the engine, so the
  engine, on hearing the signal, settles the worker's agent claims
  as interrupted (jig's `domain/engine/restart.md`, 1): a stop spends no try
  of a run's task, a coding run's among them, and moves none of its
  change's counts; the task is due again at the next start, its next
  attempt resuming from the turns it committed and starting from the
  work it saved last (`../domain/worker/README.md`, 4.2 and `../domain/worker/checkouts.md`). After a crash,
  the next start settles them at once, lost or not started (jig's
  `domain/engine/restart.md`; `../domain/root.md`, 8.1). Job claims
  remain for restart recovery even after a drain, since their effects
  may have happened.
- **Its world** is `tests/one-process` (`../testing/README.md`, 2.5).

### 6.1 A long pass

One thread runs every stage of a pass, so a stage that takes long holds
all the others, and a client sees only that nothing answered. The service's
log says when that happened; nothing else about the loop changes.

- **The stages,** in the order a pass runs them, are `io` (io's resumes,
  completions and timers), `route` (its events to their owners), `engine`,
  `worker` and `out` (closes, the box's pipe, the components' requests,
  printed links, the publish and the reclaim). `temper-service`'s
  `iterate` runs them, and `iterate_stage` runs one. The binary's adapter
  adds its own two: `names` (a forge's name resolved) and `drain` (handover
  files, operator lines and links), the only stages that may block in a
  system call.
- **The clock.** Every stage still sees the pass's one `now`. Only the
  binary's adapter reads the monotonic clock between stages, and those
  times reach `temper-service`'s `long_pass::Watch` alone, a step that
  takes them as arguments and decides nothing but whether to write a line.
  A world that calls `iterate` has no watch.
- **The bound is 1 s,** `long_pass::LONG_PASS`, on a whole pass: no stage
  can pass it without its pass passing it. It is a thirtieth of `temper
  ctl`'s deadline (`../client/ctl/README.md`, section 9), so a stall big enough
  to cost a call that deadline shows at once; and across 1,543 `temper ctl
  me` reads on running boxes under a load of up to 27 on 8 CPUs, the longest
  whole read was 1.95 s, so a healthy box writes few of these lines or none.
  A pass of exactly the bound writes nothing.
- **The line** is one stderr line, which the box keeps in `serve.log`
  (`setup/box.md`, 2), naming the pass, its longest stage and the bound,
  with no payload:
  `temper-box serve: long pass: 1250 ms, engine 1200 ms of it, past the 1000 ms bound`.
- **At most one line in 10 s,** `long_pass::LINE_EVERY`. The first long pass
  writes at once. Those after it within the 10 s are held as the longest
  of them and a count, and the first pass past the 10 s, long or not,
  writes that, ending `; 3 more since the last line`, the count being the
  long passes it does not describe. So a long stall never hides behind a
  shorter one. A held line left at the service's end is written at its exit.
- **What it costs when nothing is slow:** seven clock reads and a few
  comparisons in a pass, no allocation and no write.
- **Its tests** hold the bound, the rate and the words on supplied times:
  `temper-service`'s `tests/long_pass.rs`, and the line's words beside
  `note_line` in `temper-service-shell`'s `host.rs`.

### 6.2 When the core does not answer

The engine's service hands a party's input to the root only while its
gate is open: the root started and its decisions open
(`../domain/root.md`, 8.1), the forges' records loaded and connected,
the first periods given (configuration/README.md, section 6) and the forges'
names resolved (forge/engine.md, 2). While the gate is shut, the clients'
component answers a party's input not ready and a public read busy, at
once. A watch refused while the gate is shut says not ready, distinct
from a full watch room. An input the root hands back at its watermark
waits in the service for room (`../domain/root.md`, section 4). The
client names not ready when its answer carries that refusal; the
service's log names what the gate waits on. No gate reason travels on
the client wire (`README.md`, section 11).

- **The line.** When the gate has kept a party's input from the root for
  1 s or more, answered not ready or held, the service writes one stderr
  line, which the box keeps in `serve.log` (setup/box.md, 2):
  `temper-box serve: the core is not answering: REASON`. It writes it again
  only when the reason changes, and once the gate passes a party's
  input again after such a line, `temper-box serve: the core answers
  again`. The 1 s is 6.1's bound, a thirtieth of `temper ctl`'s
  deadline. The time is the service's clock, judged as each party's
  input arrives, so a deployment no client calls writes nothing, and
  the line needs no timer of its own.
- **Its reasons,** in the order the gate tests them, the first that
  holds:

  | The gate | REASON |
  |---|---|
  | the root not yet opened | `starting: STEP`, the start's step in 9.1's words: `loading the core's records`, `loading the inbox`, `loading the forge's records`, `linking the forge's records`, `sweeping`, `switching the store`, `committing the wires`, `adopting runs`, `reading the forge afresh`, `settling the forge's outbox`, `opening decisions` |
  | the forges' records not yet loaded and connected | `loading the forges' records` |
  | the first periods not yet given | `opening the first periods` |
  | the forges' names not yet resolved | `resolving the forges' names` |
  | the root's commits past its window, waiting on the store | `waiting for the store to make commit N durable, the oldest of M in flight` |
  | the root at its watermark | `at its watermark: B of W in flight` |

  A root whose journal has stopped is not among them: it ends the
  engine (below).
- **A stopped journal ends the engine.** The root's `Stop` with what
  stopped its journal (`../domain/root.md`, section 4) ends `temper-box
  serve` with exit 1 and one line, `temper-box serve: the engine's journal
  stopped: WHY`, WHY one of `the store answered commit N, with S sent
  and D durable`, `a decision past the store's E writes`, `the commit
  counter at its end`. A commit the store failed keeps its own line,
  `temper-box serve: commit N failed`, and a start's refusal its own lines
  (`binaries.md`).
- **Bounded, and free when the core answers:** at most 240 bytes, no
  request's words and no secret; one reason held, and a comparison as
  each party's input arrives.
- **Its tests** hold the 1 s, the change of reason, the line on the gate
  passing again and each reason's words, on a service stepped with
  supplied times, in `temper-service`'s tests; the journal's stop and
  its line beside `note_line` in `temper-service-shell`'s `host.rs`.

## 7. Versions

jig's rules hold (jig's `protocol/README.md`, section 7): one version at a
time before release, a reader that refuses another and says so, and an
upgrade that stops the engine and its workers together. jig's versions
(its channel's, its store's format and families', its clients' per
owner) and smith's (its charter's, transcript's and result's) are as
jig's `protocol/README.md`, section 7 and smith's `protocol/README.md`,
section 8 give them. temper's sit beside them:

| Version | Owner | Covers | Checked |
|---|---|---|---|
| `PAYLOAD_VERSION` | `temper-protocol-worker-wire` | temper's parts: the workspace items, the saved-work branch, the work, the preparation; a git identity's value; and a delivery's evidence, smith's record as the smith temper links encodes it | the worker's credential at the link's opening: another is refused as *application* (channel.md, section 3) |
| `STORE_VERSION` | `temper-protocol-store` | temper's families | at the store's opening: a store written by another refuses to open (store.md, section 4) |
| `CLIENT_VERSION` | `temper-client-protocol-wire` | the forge's client family's records | per owner, as jig's: the engine serves exactly its own, and `temper-box open`, from the same binary, speaks it (`../client/forge/records.md`) |
| keys' markers | the forge connector | the keys temper writes into forge bodies | the key's version, its first byte; the marker has none of its own (forge/README.md, section 3) |
| configuration | none | — | unknown members refuse the start (configuration/README.md, section 2) |

- **The store's baseline is `bc23255eb7`'s store** (`baseline.md`; the
  user, 2026-11-11), not the first release: from it, a change of
  temper's families lands with its patch, which moves `STORE_VERSION`
  (`baseline.md`, 7). The payload's and the client's versions are no
  store's, and keep the first release as theirs (skein's
  `data/codec/README.md`, section 5): until it each keeps its current
  number, and a change a reader would notice changes the schema, its
  generated code and its one set of golden bytes in place, including a
  repin of smith that changes a delivery's record (channel.md, section 3).
  A reader still refuses another version.
- **Both channel components are given `PAYLOAD_VERSION`** when they are
  made; neither knows what it covers.
- **The store keeps the wires** the last start spoke (jig's
  `protocol/store/records.md`, 3, `Wires`): temper's shell hands jig's store
  `PAYLOAD_VERSION` as `temper-payload` beside jig's and smith's, so the
  next binary's rehearsal can say whether the workers may stay
  (`../draft/upgrades.md`, section 9).
- **What the baseline freezes.** From it (`baseline.md`), every byte of
  temper's a deployment keeps must stay readable by every later build
  (`../draft/upgrades.md`, 4.1, 6.3, 10 and 11). Made before it, in place, each
  in one change with its golden bytes:
  - **The forges' secrets from a schema:** temper's family
    `temper-secrets` (`temper-protocol-records`, `schema/secrets-v1.schema`),
    a versioned record per row of jig's secrets' store under temper's tag
    (credentials.md, 2.1): an enum of records, one per item (the three
    tokens and the webhook's secret each a `secret N`; the user and the
    app's id `u64`s, big-endian as every schema's; the app's slug a
    `text N`; its key a `secret N` holding its DER; the accepted
    installations a list; the origin its kind and address). The
    hand-written envelope and its little-endian numbers go. temper's set
    names the family at the baseline, `map: Secrets`, its keys temper's
    numbers below 2^63 (jig's `protocol/store/secrets.md`). A pasted
    bundle is a carried value, jig's from 2^63 up, kept only until the
    request that carries it is used (credentials.md, 2.2): its form is
    `temper-secrets`' `Pasted`, versioned, and one of another version is
    refused by name and pasted again, never patched.
  - **temper's store says what it holds** (store.md, section 4): a
    retained refusal `holds jig-store-people Refusal`; each list of
    answers' items, comments, reviews, statuses, remarks and files its
    `pieces`; each list of raw names (a branch's segments, protection
    contexts, labels, branch names) a declared item type and its
    `pieces`, its bytes moving in place where they must.
  - **An effect's key is an identity, never patched:** kept as written,
    in the store and in the forge's markers, compared byte for byte.
    Its first byte is its version (skein's `forge/client.md`, 7.1); from
    the baseline the connector reads every version it has written,
    and a purpose's tag keeps its meaning for good, a new meaning taking
    a new tag (forge/README.md, section 3). The marker's wrapper and the bot's
    marker are fixed forms.
  - **Small files** (`../draft/upgrades.md`, section 10): the forge's setup
    `state`, versioned JSON, is read in every version from the baseline
    on and written in the current at its next write; the
    preparation writes it before the engine starts, so it follows the
    forge's own upgrade, whose going back is its dump (setup/owned.md, 3).
    The worker's secret and digest (hex digits and a newline), the
    scripted model's key, the forge's sign-in client secret (setup/owned.md,
    8), the box's group files and started markers (setup/box.md, 1) and the admin token's
    handover are fixed forms, never versioned: a change is a new file
    name, read beside the old one for as long as a release may find it.
    The engine writes no small file before its start's wires are durable
    (jig's `protocol/store/records.md`, 3), so going back finds the engine's
    as they were; the preparation's `state` and an owned Forgejo's data
    are the forge's, and going back past a preparation that migrated it
    needs its dump (`../draft/upgrades.md`, 3.2 and 11).
  - **Golden bytes** pin each fixed form, each schema's records and the
    key's frame and marker, so a change fails a test naming it.

  Outside it: what an external program reads (Forgejo's and the
  runner's files, which their own upgrade carries), the fake backends'
  sandbox, logs, caches and worker roots.
- **The baseline's kept texts, sets, golden store and gate** are
  `baseline.md`'s: every stored family's current text kept at level 0
  with no patch (3); the four owners' sets handed by `temper-service`'s
  store shell in place of `PatchSets::NONE` (4); a store the baseline's
  build wrote read at level 0 (5); its golden store, which every later
  build starts on (6); and the focused check that holds each later
  stored change to its patch (7). Still open past it: a delivery's
  evidence with its kept form (jig's `protocol/store/README.md`, section 11).

## 8. Secrets

The secrets temper's processes hold, where each comes from, who holds it
and what it reaches are credentials.md's (section 2); the files they are
read from are configuration/README.md's (`configuration/README.md`, section 5). Across the layer:

- **Names in the domain, values beside it** (jig's `protocol/README.md`,
  section 8): no domain state, fact, record of the main store or log
  holds a value. Values move between components through the routing,
  never through a root.
- **temper's forge secrets are in jig's secrets' store:** each forge's
  records (its credentials, ids and accepted installations) under
  temper's tag, read whole at start and written only by the engine's
  protocol layer (credentials.md, 2.1), and the values clients' requests
  carry, kept by jig until their run ends (credentials.md, 2.2). The
  files read at start are configuration/README.md's (`configuration/README.md`, section 5).
- **Nothing crosses a host** in standalone: the link is in process, the
  client and the webhooks on loopback, and a forge off the machine is
  reached over TLS only (configuration/engine.md).
- **The forge's setup's secrets:** the engine's setup host holds an
  admin's token or a GitHub App's key only while a plan runs (setup/README.md,
  the setup host in the engine; credentials.md, section 5). The one setup
  file `temper-box serve` reads is an owned forge's handover `admin_token`,
  read once and removed after its `TakeOver` is done (forge/engine.md, 6;
  configuration/README.md, section 5).

## 9. What temper uses from jig, smith and skein

temper's protocol layer uses jig, smith and skein as they are, at the
revisions temper's `Cargo.lock` pins.

- **jig:** the components of `components.md`, with the service parts jig ships
  (its `protocol/service.md`, section 11), composed; the reference
  service's routing and the reference binary, copied; `jig-client` and
  its native transport; jig's client test kit,
  its check, faults and attack stories, for temper's client tiers; the
  conformance kit, as the domain's worlds use it.
- **smith:** the agent shell, the host domain and host half, the codecs;
  the agent process over skein's LLM client, and its program backend,
  Claude Code (smith's `protocol/programs.md`); smith's fake `claude`, in
  the worlds.
- **skein:** io with its in-process names, processes and process groups;
  the startup kit and its private reads; lib's SHA-256 and its HMAC;
  JSON; HTTP's client and server; TLS's client; the codec generator; the
  key-value store; the framed channel; the accounts driver, under jig's
  accounts; the UI kit and its text shell; the simulator, the fake
  machine, the fake LLM peer, the fake issuer and the fake person.
- **What is temper's own:** its presentation, since presentation is the
  application's (jig's `client/domain/client/README.md`), on skein's UI kit and
  text shell (`../client/view.md` and `../client/line.md`); and, where something
  would be better upstream, as jig's design allows, the fake git
  program, on skein's fake checkout and fake forge; and `temper-box program-login`, the
  application's command a program source's holder runs on each host
  (jig's `domain/accounts.md`, section 5).
- **Not used:** `jig-protocol-charter` and `jig-protocol-tools` (llm.md);
  agents in a host's own process (`smith-inline-agent`; section 11);
  jig's reference UI (`components.md`).

## 10. Testing

`../testing/README.md` applies skein's strategy (testing-strategy.md) to
temper: its tiers, fakes, layout and budgets. What checks this layer:

- **Step tests:** every codec against golden bytes, at its bounds and by
  fuzz targets; each translation both ways; git's argument and
  environment builders, and its output parsers against transcripts of
  real git (`temper-git-transcripts`); `skein-forge`'s decoders against
  transcripts of the throwaway Forgejo (skein's conformance,
  `skein/docs/design/testing/fakes/forge.md`, section 7);
  temper's pages as skein's text shell prints them (`../client/testing.md`
  ).
- **Machine worlds:** the forge component against a scripted HTTP peer
  (skein's `forge/connection.md`, section 10); the git component against
  scripted process ends and outputs; the terminal's host against a
  scripted stdin and stdout.
- **The forge's protocol world,** `tests/engine/protocol-forge`: the
  forge connector world's stories through bytes, over in-memory streams
  cut at random, with the cost checks by HTTP request (`../testing/README.md`,
  2.2).
- **The client's joined world,** `tests/client`: temper's client root
  and view, with skein's fake person and jig's check, joined to temper's
  engine root (`../testing/README.md`, 2.1, and `../testing/checks.md`, 3).
- **The one-process world,** `tests/one-process`: `temper-service` on
  skein's simulator, killed and restarted at drawn moments, drained by a
  shutdown. Its referee: nothing a party, the worker or the forge saw
  depended on a lost commit; a drained shutdown ends with nothing
  waiting for a commit; no secret's value in a domain's state, a store
  record or an agent process's environment (`../testing/README.md`, 2.5).
- **Reconcile's world** is skein's: reconcile against the fake forge's
  protocol layer, serving the setup's routes (skein's `forge/setup/README.md`,
  section 5; setup/testing.md, 2).
- **The process tier and the live tier:** the real binary end to end,
  outside the two suites: a chat answered, a change landed on an owned
  Forgejo with CI and webhooks, the binary killed and restarted, `temper
  box`, `temper-box program-login` over smith's fake `claude`, the cluster
  with a task on each of two workers alone and a worker lost (`cluster.md`, 6);
  then a real model (`../testing/process-tier.md` and `../testing/live-tier.md`; setup/testing.md,
  4).

## 11. Open questions

Each document keeps its own; these cross them:

- **The gate's reason on the client wire** (6.2): a distinct not-ready
  watch refusal lets a client distinguish startup or an unanswering core
  from watch capacity, but carries no reason. Sending the start's step
  or the gate's other reason needs jig's vocabulary, wire and client
  link to retain it. Default: the client names not ready and points to
  serve's log; the log alone names why (`../client/ctl/README.md`, section 11).
- **The TUI's later passes** (`binaries.md`): the browser's approval,
  recording and post-landing terminal walk stay with the client's
  design (`../client/README.md`, section 5).
- **Names while running.** io connects to addresses, resolved at
  startup, while a forge's address may move behind its CDN, and a name
  that does not resolve at the start leaves its forge unreachable for
  that run, not the start refused (setup/init.md, 3). Default: a restart
  resolves names again; skein's DNS client, or skein-forge-connection
  resolving at its connect, would mend it (a skein addition).
- **The kernel's share of a pass.** A long pass (6.1) times what the
  service runs, not the reap and the submit around it, since the submit
  holds the loop's wait, and `skein-shell`'s `drive` keeps both. A stall
  there writes no line. Default: so, until a walk's log leaves a stall
  unexplained.
- **HTTP/1.1 only.** Forgejo serves it; a forge that required HTTP/2
  would need skein to grow it. Default: HTTP/1.1.
- **Generating the routing** (jig's `protocol/service.md`, section 9).
  Default: copied from jig's reference service and filled.
- **Supervision** (jig's `protocol/service.md`, section 9): what
  restarts `temper-box serve` after a failed commit or a crash. Default:
  nothing of `temper-box serve`'s own; a box restarts it (setup/box.md, 2),
  and otherwise the person or the machine's
  service manager starts it again, and it recovers from the last
  durable commit.
- **Agents in the worker's process** (`smith-inline-agent`), with no
  containment, for one person's machine (jig's `protocol/service.md`,
  section 10). Default: agent processes, as `temper-box internal agent`.
- **Programs after a forced agent stop** (section 6): killing an agent's
  group leaves a program in its own group running, even in standalone.
  Ending both needs contained trees or a parent-death signal for the
  program (setup/README.md, section 9). Default: the orderly stop lets agents
  end their programs; a second signal or forced close has no such guarantee.
- **A whole stop's bound** (section 6; `cluster.md`, 4): an engine's stop ends
  within what its parts' own deadlines give, the forge's settling
  (`../domain/root.md`, 8.2), the drain, and its workers' channels
  within the stall deadline of the close (jig's `protocol/channel/README.md`,
  9.2), with no deadline of its own, while a service manager kills it
  at its own stop timeout. Default: so; a stop deadline over the whole
  is a figure, the user's to set, and a test of a stop within it
  follows.
- **After standalone,** each designed where it is named, and needed only
  beyond one person's machine: webhooks from another host, over TLS
  (forge/README.md, section 4); hosts apart: TLS on the engine's listeners,
  containment for shared hosts, a worker alone on another host (jig's
  `protocol/README.md`, section 10; on one machine it is the cluster's,
  `cluster.md`); the forge as people's sign-in
  provider (credentials.md, section 8); the forge's client documents and
  panels (`../client/forge/README.md`); and what the forge's setup leaves for later
  (setup/README.md, section 9). Default: standalone.
- **An engine restarted under live workers.** The engine announces its
  identities' values at generation 1 at each start, sound only while no
  worker outlives it (credentials.md, section 4). Default: the cluster
  restarts whole, its workers stopped before its engine (`cluster.md`, 4);
  generations kept across starts come with hosts apart (credentials.md,
  section 8).
- **`temper-box init` for the cluster's shapes.** Default: none; `temper-box
  init` writes standalone, and the cluster's documents are its
  description's templates (`cluster.md`, 5). Hosts apart would want it.
- **Limits across processes.** Each process checks its own document
  (configuration/limits.md); what holds only between an engine's
  limits and a worker's, the cluster's one template holds, writing the
  same `application.limits` into each of its documents, and jig's link
  holds the rest at its opening, by its terms and the hello's bounds
  (`cluster.md`, 3). Default: so; a digest of the limits at the
  link's opening, beside the payload's version, comes with hosts apart.
- **The description's Rust reader's home.** Default: temper's process
  tier's support (`cluster.md`, 5); skein-world's, once a second product runs a
  cluster of its own.
- **The box's members in the engine's closure** (`binaries.md`, 2): `temper-engine`
  checks a document's `box.` limits and `demo` member as the box's
  binary does, so it links the box's machine and the fakes' crates,
  `temper-box-machine` and `temper-fake`, with none of their commands (`components.md`
  ). Default: so; a check of those members in a crate of no side would
  cut both from its closure.
- **The box's `upgrade`** (`binaries.md`): the draft gives it to
  `temper-engine` (`binaries.md`, 2) and has the box keep the commands
  it has (`../draft/deployment-binaries.md`, sections 3 and 9); the box
  holds `temper-engine`'s `versions` row and code, and neither binary
  has `upgrade` yet. Default: the box's table takes `upgrade` as one
  constant with the engine's, once jig's rehearsal and going back are
  built.
