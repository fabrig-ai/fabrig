# The services' routing

This document holds the engine and worker services' routing, split from [`README.md`](README.md).

## 1. The engine's service

In the shape of jig's `protocol/service.md`, section 2:

```
temper-engine-service                   iterate: io ↔ components ↔ root
├── temper-engine-domain                the root, with the journal (../domain/root.md)
└── the protocol layer
    ├── jig-protocol-store              commits, loads, transcripts, secrets; temper's families (store.md)
    ├── jig-protocol-engine-workers     the worker's link; temper's parts as bytes (channel.md)
    ├── jig-protocol-accounts           LLM accounts (credentials-llm-accounts.md, section 1)
    │   └── skein-oauth-accounts
    ├── the clients' listener           temper's, on jig's server part: TCP on loopback (../client/engine.md)
    │   └── jig-protocol-engine-clients-http   jig's default HTTP mapping, under /jig, with its example guards
    ├── jig-protocol-engine-clients     the clients' component: jig's messages; the forge's client family as bytes (../client/engine.md)
    ├── skein-forge-connection          the forge component: every kept forge's API; the webhooks' listener, TCP on loopback (forge.md)
    ├── the setup host                  skein-forge-setup's in-process host: the setup's plans (forge/engine.md, 6)
    ├── the identities' table           git identities' values (credentials-git-identities.md, section 1)
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
  ; `../domain/root-restart.md`, 1.2). Each forge's access is the component's
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
  `../domain/root-translation.md`, 2.4).
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
  through the routing (credentials-git-identities.md, section 1; `../domain/root-translation.md`,
  2.1).
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
  them (configuration/startup.md, section 1).

## 2. The worker's service

In the shape of jig's `protocol/service.md`, section 6:

```
temper-worker-service                   iterate
├── temper-worker-domain                the worker's root: jig's hub, smith's host domain, the checkout
└── the protocol layer
    ├── jig-protocol-worker-link        the link to the engine; temper's parts as bytes (channel.md)
    ├── smith-host-protocol             one channel per agent process, over its pipes; LLM grants filled into its start
    ├── jig-protocol-worker-agents      the program sources and the worker route, between smith's host half and the link
    ├── agent processes                 the agent's program, temper-box internal agent or temper-agent, each in a process group of its own
    ├── the identities' table           git identities' values, from grants (credentials-git-identities.md, section 1)
    └── temper-worker-protocol-git      the git component: the checkout's io (git.md)
```

No store; starting, ending and smith's payloads are jig's (jig's
`protocol/service.md`, section 6). What temper adds:

- **Agents run as the agent's program:** the worker's `agent` member
  (configuration/worker.md, section 2), with the arguments it gives, its
  environment holding no credential: the box's `temper-box internal agent
  AGENT.json`, or a deployment's `temper-agent AGENT.json` (`binaries.md`, 3),
  each spawned from the copies the worker pinned at its start, the
  program's and AGENT.json's (`../domain/worker/agent-processes.md`, section 1).
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
  operations under the worker's root directory (git-operations.md, section 1).
- **Local channel refusals** use jig's public `u16` constants
  `REFUSED_LIMITS = 2` and `REFUSED_FRAMING = 3` from
  `jig-protocol-worker-link`, including diagnostic matches (jig's
  `protocol/channel/README.md`, 9.1; skein's `net/channel.md`, 5.1). These are
  channel reasons, separate from jig's application reasons at 256 and
  above. Existing worker diagnostic tests check those matches; the
  wire and `Closed` facts keep their values.
- **Starting:** configuration, the worker's root directory opened, the
  agent pinned in it (`../domain/worker/agent-processes.md`, section 1), the
  components, then the link dialled at once (jig's
  `protocol/service.md`, section 6).
- **Ending:** the first termination signal drains the worker, and its
  stop bound ends it (`../domain/worker/README.md`, section 2); the git
  component refuses new preparations once the worker stops, and runs
  every save and push the runs' ends ask for, so a drained run's work is
  saved; at the stop bound, the root's abandon has the service abort, as
  a second signal does: every agent's io and every git invocation closed,
  their groups killed.
- **Under a service manager,** as a deployment runs its workers
  (`temper help server`), the worker's unit kills its whole control
  group when it stops or its process dies, systemd's
  `KillMode=control-group`, so no agent, program or command of the
  worker's outlives it, and its next start finds nothing of its previous
  life still writing in its workspaces
  (`../domain/worker/agent-processes.md`, section 1).

## 3. Open questions

- **Generating the routing** (jig's `protocol/service.md`, section 9).
  Default: copied from jig's reference service and filled.
