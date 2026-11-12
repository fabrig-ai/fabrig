# Configuration

Provisional, 2026-10-14, revised 2026-11-02. What temper's processes
are told when they start: the one JSON document `temper-box serve` reads,
and the forge's setup's commands with it, with jig's parts as jig
defines them and temper's own members beside them; the private files it
names; the agent's configuration; what `temper-box open` and `temper-box
program-login` are given; limits; what holds at a first start and at
every start; and the formats an operator writes. A forge is not
configuration: it is a record in temper's store, made at runtime
(../forge/engine.md, 1), and its credentials are records in jig's secrets'
store (../credentials.md, section 2).
jig's shape and rules are jig's `protocol/configuration.md`; this
document says what temper adds. What is open is listed in `README.md`, section 9.

## 1. In one page

- **One document per `temper-box serve`,** in jig's shape: three top-level
  members, `engine` (jig's engine part), `workers` (jig's worker parts)
  and `application` (temper's members). Nothing else at the top level.
  The forge's setup's commands read the same document (section 6). Its
  members give its shape: standalone, or the cluster's engine alone or
  worker alone (section 2).
- **Three directories,** configuration, data and cache, by skein's
  arguments' rule: the document is always `temper.json` in the
  configuration directory, each path in it is typed by the directory it
  is relative to, and no secret sits in the configuration directory
  (`directories.md`).
- **Read once, before the loop, and strict.** Unknown, missing,
  duplicate or out-of-bound members refuse the start, naming the path
  (jig's `protocol/configuration.md`, section 1). A change is a restart.
- **jig's parts are jig's:** `jig-protocol-config` parses them; temper
  writes no parser of its own for them, and adds only the checks its
  shapes need (section 2).
- **temper's members,** under `application`: `machine`, `landing`,
  `charter`, `clients`, `sign_in`, `webhooks`, `demo`, `welcome` and
  `limits` for the engine (`engine.md`), and `workers`, one entry per
  worker (`README.md`, section 3).
  `webhooks` is the forge component's webhooks' listener, on loopback
  (../forge/README.md, section 4).
- **No forge is configured.** A forge is a record of the forge
  connector in temper's store, made by the act that sets it up or
  connects it, numbered next at runtime and never reused, its number
  keying its records (../forge/engine.md, 1 and 2). A first run's document
  names no forge.
- **The document keeps the machine's** (`engine.md`): the clients' and
  webhooks' listeners; `machine`, the places a box's owned Forgejo and
  its fake forge bind, which `temper-box init --box` chooses and the box
  holds from its start (../setup/box.md, 1), and GitHub's, github.com's unless it
  names a fake; `demo`'s fake LLM's port; and the limits, the forges
  kept among them as a count (`limits.md`). A forge set up or connected
  takes its place from `machine`, or from what the person gives.
- **`demo`, the playground's scripted model** (../setup/playground.md): the fake
  LLM the box runs, its port, its LLM provider, its key and its
  scripts; null for none. `temper-box serve` reads its key alone, at the
  playground's start.
- **`welcome`, whether the document shows the welcome** (`engine.md`),
  whoever wrote it: `true` from `temper-box init --box`, absent otherwise.
- **Not configuration:** forges, which people set up or connect at
  runtime; repositories, which people adopt at runtime; the branch
  namespace, which the root derives from the deployment's id; the forge
  writer's identity, which comes from each forge's connect and which
  skein's forge client keeps in the main store (../credentials.md, 2.1);
  a forge's credentials, which come from the client or the engine's
  setup host (../credentials.md, section 2).
- **No secret in the document,** and no forge credential in a file the
  document names. Each forge's credentials are records in jig's secrets'
  store under temper's tag (../credentials.md, 2.1). The private files the
  document names are jig's, a worker's secret and its digest, read at
  start, and the scripted model's key, read at the playground's start;
  an owned or fake forge's handover is in its directory (section 5).
- **LLM providers, never accounts.** jig's `engine.llm` names the LLM
  providers, written out whole: their endpoints, models and prices, and
  the sources each offers. Accounts are made at runtime from the client,
  and jig keeps them (section 2; ../credentials.md, section 3).
- **One format rule** for temper's typed members: member names are the
  Rust field names, and the parser follows the types (section 8).
- **Limits by name,** their numbers in `temper-service`'s limits module,
  overridden by dotted name. The forge component's (`renew_before`,
  `log_poll`, `log_catch_up`, `log_pages` among them) are given once for
  every forge, never per forge; the forges kept, `forge.forges`, and the
  repositories, `forge.repositories`, are counts for the deployment
  (`limits.md`).
- **Standalone shape:** exactly one worker, joined to the engine by the
  in-process name `workers`; the clients' listener, and the webhooks'
  listener when configured, on loopback, the clients' at a fixed port
  for a GitHub forge; the agent program is temper's own binary. `temper-box
  init` writes it, and every deployment is it.
- **The cluster's two shapes,** for tests on one machine
  (`../cluster.md`): an engine alone, with no worker and a workers'
  listener on loopback in plaintext, and a worker alone, with no engine,
  dialling it. A document of none of the three is refused (`README.md`, section 2).
- **The agent's configuration is smith's** (section 4), and `temper-box open`
  takes no file at all (section 7).
- **Settings leave the document** (`settings.md`):
  jig's charters, templates, policies and rules, and temper's landing
  rules and forge's charter, become settings in the store, changed by a
  decision an owner accepts, with the binary's stock values as their
  defaults. The document keeps what a start needs before it reads its
  store, and what belongs to the machine.

## 2. The document

```
{
  "engine":      jig's engine part
  "workers":     [ jig's worker part ]
  "application": temper's members
}
```

- **`engine`** is jig's engine part (jig's `protocol/configuration.md`,
  section 3), in jig's formats (its section 10). Its `clients` member is
  what jig's clients' component takes; the clients' listener is temper's
  (`engine.md`).
- **`engine.llm`** is jig's LLM providers (jig's
  `protocol/configuration.md`, section 3), written out whole, never
  implied: each with the endpoint its added models take, and its
  endpoints. It holds no model: models are settings, the binary's
  defaults with the store's changes over them (`stock.md`; jig's
  `domain/engine/settings.md`, 11). Each provider has the
  sources it offers, a sign-in with its public registration, keys, a
  program, or none; and the facts its terms ask temper to show. It holds
  no account, key or record: accounts are made from the client
  (../credentials.md, section 3). temper adds no member; the registration's
  name hint is `temper` in every deployment (jig's `domain/providers.md`,
  2.2). The owned example writes out OpenAI with keys and ChatGPT's
  sign-in, a plan's, using its public Responses API (../setup/init.md, 1); a
  provider temper does
  not ship is another entry, such as a local model's endpoint on
  loopback with no source to hold.
- **A sign-in's redirect** binds a loopback listener on an ephemeral
  port for each sign-in, unless the LLM provider's registration fixes a
  port (jig's `protocol/configuration.md`, section 3; skein's
  `oauth/accounts.md`, 5). A fixed port is checked at start against the
  deployment's own listeners: the clients' and the webhooks', the
  machine's owned and fake `listen` (`engine.md`) and `demo`'s fake LLM's
  (../setup/playground.md), whose ports it must not share on loopback.
- **A sign-in's hosts** are resolved once at start, as skein's accounts
  driver takes them (skein's `oauth/accounts.md`, 2): its token endpoint's,
  and the origins of its issuer, its keys and its revocation. One that
  does not resolve, the default document's `auth.openai.com` on a
  machine without DNS (../setup/init.md, 3), refuses no start: its address is
  held at `127.0.0.1:0`, which no listener can hold, so each exchange of
  that provider's sign-in, a sign-in, a refresh or a revocation, fails
  at its connect as unreachable, until a start that resolves it, as a
  held forge waits for one (../setup/init.md, 3). An address, port, server
  name or trust file that is wrong still refuses the start, naming its
  member; Google's sign-in's endpoints (`../sign-in.md`, 3.1) still refuse
  it, naming the host.
- **`workers`** is a list of jig's worker parts (jig's
  `protocol/configuration.md`, section 4).
- **`application`** holds temper's members (`engine.md` and `README.md`, section 3).
- **jig's reference fixtures** are a complete example of jig's parts: the
  reference binary's test configuration (`jig-test`'s
  `tests/support/mod.rs`, made from jig's channel world's `fixtures.rs`).
  temper's process tier writes its own the same way (`../../testing/process-tier.md`
  ), and the owned example is a whole document (../setup/init.md, 1).
- **What standalone requires of jig's parts,** each checked at start, a
  failure naming its path:
  - `workers` has exactly one entry, whose `engine` is `{"in_process":
    "workers"}`;
  - `engine.hosts.listener` is `[{"in_process": "workers"}]`, and
    `engine.hosts.workers` names that one worker;
  - `engine.connectors` is exactly `[1]`: the forge connector, temper's
    one connector, whose number both ends of the client's wire fix as
    `temper-client-protocol-wire`'s `FORGE` (`../../client/forge/records.md`);
  - the worker's `agent` paths are absolute (section 4).

  The worker's `agent` names temper's own binary (section 4), which no
  check can tell from another program.
- **The shape is read from the members,** before any other check, and
  each shape's checks are its own (`../cluster.md`, 1):

  | Shape | `engine` | `workers` | `application` |
  |---|---|---|---|
  | standalone | present, `hosts.listener` `[{"in_process": "workers"}]` | one, its `engine` `{"in_process": "workers"}` | every member, `workers` one entry |
  | engine alone | present, `hosts.listener` one socket address on loopback (in `127.0.0.0/8`, or `::1`), plaintext, its port fixed or 0; `hosts.workers` one or more | `[]` | every member, `workers` `[]` |
  | worker alone | absent | one, its `engine` a socket address on loopback, plaintext | `workers`, one entry, and `limits`; no other member |

  - an engine alone is checked as standalone is, but for its listener
    and its workers, by the table, each worker's `digest` a data path
    of its own;
  - a worker alone is checked for its worker part, its entry of
    `application.workers` (section 3), its agent's configuration
    (section 4) and the limits its side takes (`limits.md`), and for
    nothing of an engine's;
  - any other document is refused, naming the member that departs from
    the nearest shape: an in-process listener beside a socket, an
    engine with both a worker and a socket listener, a worker alone
    with another member. An in-process listener tells jig's core that
    no worker outlives the engine (jig's `domain/engine/restart.md`),
    which a socket's workers would.
- **What temper's root requires of jig's parts,** in every shape with an
  engine, each checked at start, a failure naming its path:
  `engine.limits`' `fleet.engine_slots` is 0, and every charter's `hosts` is `"workers"`.
  The root has no hub child, so the fleet must place every run on a
  worker (`../../domain/root.md`, section 2).
- **Templates are jig's two defaults,** chat (template 1, label 1) and
  goal (template 2, label 2), as jig's `protocol/configuration.md`,
  3.1 gives them, the chat's executor the charter chats run under;
  temper defines no template of its own. One flag departs from jig's
  defaults: the chat is ended by its requester, `"ended_by":
  "requester"` (jig's `protocol/configuration.md`, 3.2), so that a chat
  stays open past its run's turn until its person closes it
  (`../../domain/agent/charter.md`, 5); the goal is ended by its result, as
  every delegation is. Its client's lists filter on
  those labels (`../../client/pages/README.md`), and a project's policy may override
  their defaults as jig's allows.
- **A project's permission mapping,** jig's `roles` of each of
  `engine.projects` (jig's `domain/people/README.md`, section 4), names the
  forge's permissions by number under the forge connector, 1: 1 read, 2
  write, 3 admin. An adoption gives a repository's collaborators the
  roles their permissions map to (`../../domain/forge/adoption.md`).
- **Reading,** by `temper-box serve`, in this order:
  1. the three directories, resolved and checked (`directories.md`); then
     the document, `temper.json` in the configuration directory, whole,
     with skein's startup kit, within the bound jig's reference binary
     sets on a configuration's bytes;
  2. jig's parts, by `jig-protocol-config`, and the shape they give;
     `application`, with the members its shape takes, by
     `temper-service`'s startup, with `jig-protocol-config`'s readers, so
     every problem names its path the same way
     (`application.clients.listener`); `machine`, `demo`, and the
     setup's and the box's limits, by their own crates' checks (../setup/init.md and ../setup/playground.md and ../setup/README.md, section 7
     );
  3. the checks across the whole that its shape holds: the references
     of `README.md`, section 2 and `engine.md` and `README.md`, sections 3 to 4,
     a git identity's largest value against the channel's bound on a
     credential's value (`engine.md`), the limits against each other and
     the edge bounds (`limits.md`), and the channel's start checks on
     temper's parts (../channel.md, section 8);
  4. every private file the document names, each within its bound
     (section 5);
  5. the agent's configuration (section 4).

  Every problem found is reported, one line each on stderr, and `temper-box
  serve` exits 1 before io starts. No forge is read here: the forge
  connector's records are restored from the store, and the engine's
  routing loads their credentials from jig's secrets' store, once io
  starts, before the forge component starts; a forge whose records do
  not serve is not connected, never a refused start (section 6;
  ../forge/engine.md, 1; ../credentials.md, 2.1).
- **Paths are typed:** each is a configuration, data or cache path,
  relative to its directory or absolute, as jig's parts' are (jig's
  `protocol/configuration.md`, section 9), or a path that must be
  absolute; `directories.md` gives each member's. None is resolved against
  the working directory.

## 3. temper's members for a worker

**`welcome`:** a boolean, or absent: `true` shows the welcome
(`../../client/pages/README.md`) and lets the deployment offer the
playground (../setup/playground.md); `false` or absent, none.

- **The document alone says:** a start reads the member whoever wrote
  the document. `temper-box init --box` writes `true`, `temper-box init` alone
  nothing (../setup/init.md, 2), and no flag of any command changes it,
  `temper-box internal serve`'s `--box` among them (../setup/box.md, 4).
- **Its readers:** the client, from the deployment's read
  (`../../client/forge/records.md`), and `StartPlayground`'s offer (../setup/playground.md
  ).

**`application.workers`:** a list with one entry per entry of the
top-level `workers`, in the same order; standalone and a worker alone
have one, an engine alone none. Each:

| Member | Type | Bounds | Feeds |
|---|---|---|---|
| `root` | a directory, a data path | made private (0o700) if absent; an existing one must be the process's user's, private, and not reached through a link | the worker's workspace directory: every checkout's workspace is made under it (../git.md, section 2; `../../domain/worker/checkouts.md`) |
| `git` | string, or `null` | an absolute path to an executable file, checked at start, whose `git version` is at least the one ../git.md, section 3 names; `null` for none, as `temper-box init` writes it on a machine with no git (../setup/init.md, 2) | the program the git component runs for each invocation in a repository whose record names none (../channel.md, section 4), in the environment ../git.md, section 5 gives it; with `null`, each such invocation fails, naming its forge's host, and the row of each connected forge but a fake one warns (../git.md, section 3; `../../client/forge/deployment.md`). A forge whose record names a git program runs that instead: the playground's fake forge the machine's fake `git` (`engine.md`; ../setup/init.md, 2) |

- **The worker's root takes the rest from jig's and smith's:** the hub's
  limits from jig's worker part; smith's host limits and the bounds its
  hello states from the agent's configuration (section 4), as jig's
  reference binary derives them, but for the agents the host holds, which
  are its slots (`../../domain/worker/README.md`, section 5); its stop bound from
  jig's graces; the checkout's limits from temper's profile (`limits.md`).
- **Workers are alike** (jig's `protocol/configuration.md`, section 4):
  no credential but the worker's secret, and no forge configuration.
  Remotes, git identities and a forge's git program come with each
  assignment, in its repositories' records (../channel.md, section 4).
- **A worker on another host** takes its own machine's git as its `git`:
  a fake forge is on loopback alone (../setup/playground.md), and serves no
  worker on another host. The cluster's workers are on its machine, and
  it serves them (`../cluster.md`, 2).

**The engine-away bound** is jig's worker member `graces.grace_ms`: how
long a worker keeps hosting its runs with no engine that has admitted
its hello (jig's `domain/hosts/hub.md`, 6; `../../domain/worker/README.md`, section
2). `temper-box init` writes 60,000, so that an engine's restart (its
store's load, its switch, its listeners again) passes within it and
each run is adopted where it stands; a deployment whose engine takes
longer to start raises it.

- **The drain's bound** is jig's worker member `graces.drain_ms`: how
  long a worker told to stop lets its runs end their turns before it
  cancels them (`../../domain/worker/README.md`, section 2). It may be
  absent, and is then 180,000, the draft's three minutes
  (`../../draft/redeploys.md`, 6.1); `temper-box init` writes none.
- **It is ordered under the engine's grace.** The stop bound a worker
  states is the longer of its ping interval, its silence deadline and
  this grace together, and its drain's bound; then the longer of
  `agent_stop_ms` and `delivery_ms`, then `save_ms`: 186 seconds with
  jig's link profile (5 and 15 seconds), `temper-box init`'s graces
  (60,000, 5,000, 1,000 and 1,000) and the drain's default, which
  outlasts the 80 seconds of notice and grace. It is a deadline the
  worker keeps (jig's `domain/hosts/hub.md`, 6), and it must stay below
  the engine's `fleet.grace`, 240 seconds in both of jig's profiles,
  derived from it (jig's `domain/engine/fleet.md`), or the hello is
  refused; a deployment that raises the one raises the other with it.
- **The service manager's stop timeout** sits above the stop bound: a
  unit that kills the worker sooner is a crash, losing the workspace's
  changes since its last save or delivery, and the attempt is lost
  (`../../draft/redeploys.md`, 6.1). The worker's unit in `temper help
  server` gives `TimeoutStopSec=` the stop bound and a margin, 240
  seconds with these graces.
- **A hello refused for a charter's reserve names the setting.** The
  fleet judges a worker's checks' timeout against each modifying
  charter's `reserve_time` in the store's settings (jig's
  `domain/engine/fleet.md`). The refusal's words, on the wire and in
  the engine's log, name both: `charter N: needs checks within R ms; the
  agent's profile.policy.check_timeout is O ms; raise charter N's
  reserve_time (temper help settings) or lower the timeout`, within the
  refusal text's 256 bytes.
- **A start judges its own worker against the store,** not the
  document. Standalone, the engine's start checks its worker's agent
  against the configured charters only for the deployment's own bounds
  (a message, an inbox, the turns kept unacknowledged, `limits.md`); a
  charter's turns, time, waiting, checks or models are the store's
  settings' once a store exists, so a shortfall there is printed as a
  warning naming the configuration's path, and the hello's check, which
  names the setting, keeps runs off a worker too small for them.

## 4. The agent's configuration

- **smith's agent configuration** (smith's `protocol/agent.md`, section
  4), a file of smith's, named by jig's worker part's `agent` member:
  `configuration` the absolute path of smith's agent configuration, and
  `program` and `arguments` in one of two forms:
  - **the box's:** `program` the absolute path of temper's binary and
    `arguments` `["internal", "agent", <that same path>]`, so each agent
    process is `temper-box internal agent AGENT.json` (../binaries.md
    ); `temper-box init` writes them so, with smith's agent configuration
    as `agent.json` in the configuration directory (../setup/init.md, 2);
  - **a deployment's:** `program` the absolute path of `temper-agent`
    (../binaries.md, 3) and `arguments` `[<that same path>]`, so each agent
    process is `temper-agent AGENT.json`; a worker alone's document,
    written by hand until a cluster's `init` is designed
    (`../../draft/deployment-binaries.md`, section 3), names it so.

  Any other `program`, each of its `arguments` absolute, runs as
  written, with no model list (../README.md, section 5). They are absolute
  because the worker spawns the agent with its arguments as written
  (jig's `protocol/configuration.md`, section 9).
- **Read once, pinned:** by the worker's start, `temper-box serve` or
  `temper-worker serve`, for smith's host limits, the channel's limits
  and the agent's bounds, as jig's reference binary reads it; the start
  then pins those same bytes, with the program, in its root
  (`../../domain/worker/README.md`, section 5), and each agent process
  reads the pinned copy as it starts. Editing the file, or replacing the
  program, changes what the worker's next start pins, never a running
  worker's agents.
- **It holds no credential:** an agent's grants come over smith's
  channel (../credentials.md, section 3).
- **A model's reasoning allowance,** its `reasoning_item` (smith's
  `protocol/agent.md`, section 4), bounds each reasoning item the model
  sends; past it, with smith's default `oversized_reasoning` of `fail`,
  the call fails and the run holds at once, naming the bound and its
  value (`../../domain/worker/README.md`, 4.3). smith derives its largest turn
  from it, so the allowance and the hub's bounds go together: the start
  refuses a document whose bounds do not hold that turn (`limits.md`).
  temper's documents give a plan's model an allowance its reasoning
  does not reach in ordinary use (../setup/init.md, 1), a default a person
  may lower, with the bounds it derives.
- **`programs`** (smith's `protocol/agent.md`, section 4): each program
  the agent may run as a backend, by its adapter, with the program's
  absolute path and its programs directory, which holds a configuration
  directory for each program source, `<directory>/NAME`
  (../credentials.md, section 3). Its paths are smith's, absolute, but for
  Claude Code's `path` `"fetched"`, temper's pinned build, which temper
  turns into its launcher's absolute path before smith reads the
  document (../setup/owned.md, 6); the
  programs directory `temper-box init` writes is `programs` in the data
  directory (`directories.md`). The owned example's agent configures
  Claude Code's (../setup/init.md, 1). A programs directory need not exist
  until `temper-box program-login` makes it; one that exists must be a
  directory of the user's own with no group or other bits, not reached
  through a link at its name, or the start is refused, naming it
  (skein's `runtime/io/files.md`, 3). A linked parent is accepted. temper reads
  nothing inside it and creates no missing programs directory at serve.
- **The same names as the engine's** (jig's `protocol/configuration.md`,
  section 4), checked at start: every endpoint of `engine.llm`'s
  providers must be in the agent's configuration under the same name,
  every model a charter allows must be declared there, and every
  program an LLM provider offers must be among the agent's `programs`,
  or the start is refused, naming it. One process reads both, so a
  mismatch that would fail every run's charter is caught before the
  loop.
- **The worker must fit the configured charters.** After those name checks,
  startup builds jig's core configuration once, expands its instruction files,
  and projects its fleet needs with jig's `fleet_needs::configured`. The
  worker's bounds are its agent configuration's `bounds`, which its
  hello carries unchanged, and jig's `shortfall` selects the first unmet
  requirement. Startup
  names that requirement's configuration path, required and offered amounts,
  with its dimension's units; the same core configuration then checks the
  root. Admission policy stays in jig, including checks fitting within a
  modifying charter's reserve.

## 5. Secrets' files

| File | Named by | Holds | Read by |
|---|---|---|---|
| an owned or fake forge's handover: its admin's token | `admin_token` in the forge's directory under data, named by its number, `forge-N` for an owned forge (../setup/README.md, section 3); named by no member | the token's bytes | the engine's setup host, once, at start or the box's `started` (../setup/owned.md, 5); a fake forge, before binding (../setup/playground.md) |
| a worker's secret, its digest | jig's parts, data paths: `worker.secret` and `worker.digest` in the data directory as `temper-box init` writes them (../setup/init.md, 2) | jig's formats (jig's `protocol/configuration.md`, sections 6 and 9) | jig's components |
| Google's client secret | `application.sign_in.google.secret`, a data path | the secret's bytes | `temper-box serve`, once, at start, within `sign_in.secret_bytes` |
| the OpenID provider's signing key, and each of its confidential clients' secrets | jig's `engine.clients.provider` (jig's `protocol/configuration.md`, section 3) | jig's formats: the key PKCS#8 PEM, made by `temper-box forge sign-in` (../setup/owned.md, 8) or a box's first run (`../openid-provider.md`, 9) | `temper-box serve`, once, at start, for jig's helper that builds the issuer (`../openid-provider.md`, 2) |
| the scripted model's key | `application.demo.llm.key`, a data path: `demo/llm.key` as `temper-box init` writes it (../setup/init.md, 2) | the key's bytes | the fake LLM, at its start; `temper-box serve`, at the playground's start, which makes the scripted account with it (../setup/playground.md) |

- **A secret file holds its bytes and nothing else** but a final
  newline, which is not part of the secret (jig's
  `protocol/configuration.md`, section 9).
- **Private reads,** each within its bound, with skein's
  `read_private_file` (skein's `runtime/shell.md`, 6.1): a file readable by
  anyone but the process's user, or reached through a link, is refused,
  naming it. For the handover, the forge stays not connected, or set up
  failed at the box's `started`; the engine and other forges run on
  (../setup/owned.md, 5). Its bound is `records.value_bytes` (`limits.md`).
- **A present handover is always taken over.** At start or the box's
  `started`, the setup host reads it once, even with a kept
  `AdminToken`, runs `TakeOver`, and removes the file only after its
  `Done` and the record kept in jig's secrets' store (../setup/owned.md, 5).
  Removal must succeed before reconcile; its failure leaves the file
  for the next start or ask to take over again. With no file, the host
  reconciles from the kept record; with neither, the forge is not set
  up.
- **A set-up forge's directory,** under data and named by its number,
  and its other files are ../setup/README.md, section 3's, made by the setup's
  commands and removed with the forge (../forge/engine.md, 3); `temper-box serve`
  reads none of them but the handover.
- **Never written by `temper-box serve`,** but for the handover it removes
  once taken. An operator replaces a worker's secret and restarts; a
  forge's credentials are replaced from the client, with no restart
  (../credentials.md, section 5).
- **Every forge credential in jig's secrets' store,** under temper's
  tag, and in no file but an owned or fake forge's handover until it is
  taken:
  a forge's tokens, its App's key, its webhooks' secret, its user, from
  the client or from the engine's setup host (../credentials.md, sections
  2 and 2.1).
- **No LLM account's file.** LLM accounts' records and keys are in
  jig's secrets' store, and a program's login is in its configuration
  directory, which temper never reads (../credentials.md, section 3).

## 6. First start and every start

- **Every command that reads the document checks it,** once its three
  directories are resolved and checked (`directories.md`). `temper-box serve`
  checks it whole, as `README.md`, section 2 says. `temper-box program-login`, every
  `temper-box forge` command and `temper box` check its structure, its
  members, the limits and `machine`, and none of the checks that need
  a private file read, since a set-up forge's files do not exist before
  its commands make them (../setup/init.md). A `temper-box forge`
  command finds its forge's kind in the box's group file, `fetch`,
  `prepare` and `run` taking an owned forge with none (../setup/box.md, 4),
  and its setup entry in `machine` (../binaries.md; ../setup/box.md, 1).
  Each reads
  the private files its own part needs alone: a `temper-box forge` command
  its forge's, by ../setup/owned.md; `temper box` and `temper-box
  program-login` none, the latter reading the agent's configuration
  besides (`README.md`, section 4); `temper-box serve` every one the document names, and
  of a forge's directory an owned or fake forge's handover alone
  (`README.md`, section 5).
- **A deployment's first start** is jig's (jig's
  `protocol/configuration.md`, section 7): the first with an empty store,
  whose owners and projects the core seeds from `engine`. temper adds:
  - **periods from the smallest source,** as jig's reference service
    gives them (jig's `protocol/configuration.md`, section 5): before the
    core admits a party's request, the engine's routing gives the root
    one period for each configured project, its budget the project's
    spend per period, with no next start;
  - **each project's landing rules** join its policy as it is seeded;
  - **a project made after the first start** (jig's `MakeProject`, its
    `jig/docs/design/domain/model.md`, 6.1) has the deployment's landing rules, which
    every start applies (`../../domain/root.md`, section 5), and none of
    `landing.projects`, which names seeded projects alone; its first
    period is made with it (jig's `domain/people/acts.md`, 2). The default
    document seeds no project, its first owners the deployment's alone,
    and offers policy 1, which every project made starts from; the
    green-CI landing rule is in `landing.deployment` for every project
    (../setup/init.md, 3). A deployment whose first start seeded a project
    keeps it: no later start removes or renames it.
- **Every start:**
  - jig's parts apply as jig says (jig's `protocol/configuration.md`,
    section 7);
  - temper's members apply as configured: the machine's places, the
    deployment's landing rules, the forge's judges for every rule, the
    charter, the limits and the workers. The forges are their records,
    restored with the forge connector's (../forge/engine.md, 1);
  - every limit is checked again, temper's and jig's together, with the
    channel's start checks on temper's parts (`limits.md`; ../channel.md,
    section 8), so a `git` base or a limit changed since the last start
    is refused before io starts when a remote or a part could pass its
    bound;
  - the root's configuration is held to the root's limits before the
    root is made (`../../domain/root.md`, section 5): a start that breaks
    one is refused, naming it;
  - each provider's fixed `sources.sign_in.redirect_port` differs from
    the clients', webhooks', the machine's owned and fake and `demo`'s
    fake LLM's listener port, even between IPv4 and IPv6. An absent or
    ephemeral listener reserves no fixed port. A connected forge's
    remote API is no local listener;
  - once io starts, the forge connector restores every forge's record,
    and an ended one's removal resumes (../forge/engine.md, 3); the engine's
    routing loads every kept forge's credentials from jig's secrets'
    store before the forge component opens a forge. A forge whose
    records are missing, incomplete, unreadable, or made for another
    kind or `api` (their `Origin`) is not connected, named, and the
    other forges run on; nothing is erased. Rows of a forge with no
    record are left, and named once in the log (../credentials.md, 2.1);
  - each forge's writer is restored from the main store, where skein's
    forge client keeps it (`Key::Writer(forge)`; ../credentials.md, 2.1);
  - a GitHub forge's delivery log is read from its newest delivery on:
    temper keeps no `guid` across starts, and skein's forge client
    reads each adopted repository afresh (../forge/README.md, a GitHub App's
    delivery log); `forge_component.log_catch_up_ms` bounds only a later
    start of a forge within the same process (`limits.md`);
  - carried rows that a run left at a stop are erased, a decision of
    the core that the routing carries out; a connect they belonged to
    ends interrupted, and the client asks to paste again
    (../credentials.md, 2.2);
  - an owned forge's handover is taken, when its file exists (section
    5), at the start, and at the box's `started` for a forge set up
    from the client, while the engine serves (../setup/owned.md, 5, and ../setup/box.md, 1);
    each managed and owned forge is reconciled by the engine's setup
    host, and shows "connecting" until reconcile is done, while the
    engine serves (../setup/README.md, the setup host in the engine); an owned
    forge with neither a handover nor a record of one is not set up
    until the client asks the box for it (../setup/owned.md, 5);
  - under the box, `temper-box internal serve` takes the listeners the box
    gives it, from fd 3, `LISTEN_FDS` of them, its clients' and webhooks'
    listeners claiming them by their addresses, and `--box` adopts the
    box's pipe, the two fds after them, refusing its start, naming the
    fd, when either is not an open pipe (../setup/box.md, 1); a start without
    the flag has no pipe, and leaves those fds alone;
  - every kept forge's identity that has a value is announced at
    generation 1 before the root's restart; a GitHub forge's, once its first tokens
    are minted (../credentials.md, section 4). Every connected forge's user
    is checked by the forge component's first exchange (../credentials.md,
    section 5);
  - the branch namespace is derived from the deployment's id, the same
    at every start;
  - a later start gives no period from the smallest source, but the
    calendar's (`../../domain/routing.md`, 7) where a project's next start
    has passed: the store keeps each project's
    current one (jig's `domain/engine/README.md`, 5.4).
- **Repositories are never configured.** A person adopts each at runtime
  through the client, by the forge's number, the owner and the name, and
  the store keeps it (jig's `protocol/configuration.md`, section 3;
  `../../client/forge/family.md`).
- **What a change reaches,** after its restart: `machine`'s places,
  the forges made after it; a kept forge keeps the place and git
  program its record holds (../forge/engine.md, 1), while its setup entry is
  `machine`'s part of its kind (../setup/init.md), so a forge moved
  to another place is one removed and set up or connected again
  (../forge/engine.md, 3); the webhooks' listener, from the first delivery on; a worker's
  `git`, every invocation, checked at the start.
- **A forge's credentials are no change of configuration:** they are
  connected, replaced and forgotten from the client, with no restart
  (../credentials.md, section 5).

## 7. The client's arguments

- **`temper-box open DIRECTORY LINK_OR_ADDRESS`** takes no configuration
  file: its two arguments are `../../client/line.md`, 1's, and its limits are
  constants of temper's client (`../../client/README.md`, section 2).
- **`temper ctl DIRECTORY ...`** takes none either (`../../client/ctl/README.md`,
  section 6).
- **`temper-box program-login [DIRECTORIES] NAME`** reads the same document
  as `temper-box serve`, and the agent's configuration it names
  (../credentials.md, section 3). `DIRECTORIES` is `[--config DIR] [--data
  DIR] [--cache DIR]`, each with its variable and its default (`directories.md`
  ), given anywhere among the command's arguments.
- **`temper-box internal agent AGENT.json`** reads smith's agent configuration
  (section 4).
- **`temper-box forge prepare|run [DIRECTORIES] FORGE`** and **`temper box
  [DIRECTORIES]`** read the same document as `temper-box serve` (section 6;
  ../setup/owned.md and ../setup/box.md); **`temper-box init [DIRECTORIES] [--box]`**
  writes it (../setup/init.md, 2). Reconcile is no command: the
  engine's setup host runs it, at each start and at a connect (../setup/README.md,
  the setup host in the engine).

## 8. Formats

**The format rule** for temper's typed members, from the Rust types they
feed:

- **member names** are the field names;
- **an enum's variant without fields** is a string of its name in
  snake_case (`"clean"`);
- **a variant with fields** is an object with one member named after the
  variant in snake_case, whose value is the field's for one unnamed field,
  or an object of the fields by name (the machine's GitHub
  `transport`: `{"tls": {"server_name": ..., "trust": ...}}`);
- **byte strings** are JSON strings, UTF-8;
- **durations** are integers of milliseconds, the member's name suffixed
  `_ms`;
- **an absent option** is `null`, never a missing member;
- **a map keyed by a number** is a list of objects, each holding its key
  under the key's name (`landing.projects`: `project`);
- **integers** are within their type's range; **booleans** are `true` or
  `false`;
- **a value of one of jig's types** is written as jig's configuration
  writes it, so one document spells one type one way: a landing rule's
  `pattern`, jig's authority pattern, is
  `{"segments": [...], "last": "exact" or "open", "value": ...}`, as in
  jig's `rules` and `projects`.

**jig's formats,** by citation (jig's `protocol/configuration.md`, section
9): an address (`{"socket": ...}`, `{"host": ..., "port": ...}`,
`{"in_process": ...}`); a secret file; a digest file. A `transport`
of temper's, the machine's GitHub's or Google's endpoints', offers the
choices jig's LLM endpoints offer (plaintext to
a loopback socket address
only; TLS with a server name and a trust file, or the machine's roots),
written by the format rule as one member of temper's own type, where
jig's endpoints spell them as sibling members (`"transport": "tls"`,
`"server_name"`, `"trust"`).

**The `application` member for standalone,** one owned Forgejo's place
and one worker, complete:

```json
"application": {
  "machine": {
    "owned": {
      "forgejo": "fetched",
      "runner": {"binary": "fetched", "labels": ["temper-host:host"],
                 "capacity": 1, "path": "/usr/local/bin:/usr/bin:/bin"},
      "listen": {"socket": "127.0.0.1:3000"},
      "person": {"login": "ada", "email": "ada@example.invalid"},
      "remote": "http://127.0.0.1:3000"
    },
    "github": null
  },
  "landing": {
    "deployment": [],
    "projects": [{
      "project": 1,
      "rules": [{
        "connector": 1,
        "kind": 4,
        "pattern": {
          "segments": ["forge", "127.0.0.1:3000", "acme", "widget", "branch"],
          "last": "exact",
          "value": "main"
        },
        "enforce": true,
        "ci": true,
        "up_to_date": true,
        "gates": [{"number": 1, "blocking": true, "freshness": "clean"}],
        "approvals": []
      }]
    }]
  },
  "charter": 2,
  "clients": {"listener": {"socket": "127.0.0.1:7100"}, "bundle": null, "public": null},
  "webhooks": {"listener": {"socket": "127.0.0.1:7101"}},
  "sign_in": null,
  "demo": null,
  "limits": {"profile": "small", "overrides": {}},
  "workers": [{"root": "workspaces", "git": "/usr/bin/git"}]
}
```

A landing rule may name a repository no project has adopted yet: it
covers nothing until a person adopts `acme/widget`. The document names
no forge and no credential: the person sets the owned Forgejo up from
the client, under the box, which makes its record on these places
(../forge/engine.md, 2), or connects GitHub, or a Forgejo elsewhere as its
admin, with nothing changed here; a forge's git identity takes its
bot's login and, on Forgejo, the `GitToken` record.

**The whole document,** jig's members elided as `...` (this skeleton is
not itself valid JSON):

```
{
  "engine": {
    "deployment": ..., "owners": ..., "providers": ..., "rules": ...,
    "projects": [{"number": 1, ...}],
    "charters": [{"number": 1, ...}, {"number": 2, ...}],
    "templates": [{"number": 1, ...}, {"number": 2, ...}],
    "hosts": {
      "workers": [{"name": "worker", "digest": "worker.digest"}],
      "listener": [{"in_process": "workers"}]
    },
    "llm": ...,
    "clients": {"link": true, "approval": true, ...},
    "connectors": [1],
    "limits": {"profile": "small", "overrides": {"authority.segments": 8, ...}},
    "memory_bytes": ...
  },
  "workers": [{
    "engine": {"in_process": "workers"},
    "name": "worker",
    "secret": "worker.secret",
    "slots": ..., "graces": ...,
    "programs": {"check_interval_ms": ...},
    "agent": {
      "program": "/usr/local/bin/temper",
      "configuration": "/home/me/.config/temper/agent.json",
      "arguments": ["internal", "agent", "/home/me/.config/temper/agent.json"]
    },
    "limits": {"profile": "small", "overrides": {"hub.event_bytes": ..., ...}},
    "memory_bytes": ...
  }],
  "application": { ... as above ... }
}
```

Its `engine.limits` names the overrides `limits.md` asks, and its worker's
`limits` those the worker's check against the agent asks (`limits.md`).
The owned example, `docs/examples/owned.json`, is a whole document for
one owned Forgejo's place, with its own values (../setup/init.md, 1).

## 9. Open questions

- A landing branch with another name than `main`: `stock.md`, section 1.
- **The reference's project rule in stage one** (`reference.md`): its settings
  description must include the project's own rule 1 alongside the
  deployment's rules, so a goal's branches do not inherit the stock's
  CI and review. Default: apply `project.json` after making the project;
  the golden's missing obligation remains a gap in the status file.

- Ask temper's real-model walk: `stock.md`, section 1.

- **A cut sub-agent answer's rest** (`settings.md`, 7): smith's `domain/run.md`,
  section 13 leaves retention for a continuation undecided. Default:
  derive the answer room from the payload and ask a new child, narrower,
  for omitted material; temper declares no continuation setting.
- **A terminal approved at a public name** (3, clients): Google on the
  web and the between-hosts approval wait for `../../draft/tui.md`, section
  7's flow. Init's `approval` enables the loopback flow beside the
  printed link; it grants no approval on a listener beyond loopback.

- **A coordinator's repository notes** (`stock.md`, `reference.md`): charter 5 asks
  for notes about flakes and inherited failures in the repository's
  scope, which the stock's goal-only note authority does not grant.
  Default: retain the narrow authority; satisfying that instruction
  needs separate authority work, and remains a gap in the status file.

- The reference's later runs: `reference.md`, section 1.

- Settings refusal positions: `settings.md`, section 9.
- The checks' deadline: `reference.md`, section 1.
- A reference call of 128 KB: `settings.md`, section 9.
- Large arguments through a real program: `settings.md`, section 9.

- A chat checking the forge before declaring a repository absent: `stock.md`, section 1.
- **Four agents' resident memory** (`stock.md`): the slot, workspace and git
  operation fit checks admit four; they do not establish the memory used
  by four agent processes and their programs on a laptop. Default: four
  slots, with the after-landing measurement in `../setup/README.md`, section 9;
  a person's existing document keeps its configured concurrency.
- A real provider's list: `settings.md`, section 9.
- The real-browser check of watch sizing: `limits.md`, section 1.
- A page refused busy: `../../client/pages/README.md`, section 3.
- A party's fair share of the watch rooms: `limits.md`, section 1.
- `views.`'s other names: `limits.md`, section 1.
- A Claude-only box on the small profile: `settings.md`, section 9.
- A message, a report or a person's spec past what a box takes: `settings.md`, section 9.
- A brief past the host part's room: `settings.md`, section 9.
- Requirements an application fixes: `settings.md`, section 9.
- A kept GitHub forge whose base ends in `/`: `../forge/engine.md`, section 9.
- **A repository renamed or moved after adoption.** Default: its
  resource path, which landing patterns match, and its remote keep the
  owner and name of its adoption; Forgejo redirects the old path, and
  git follows the redirect (skein's conformance, `skein/docs/design/testing/fakes/forge.md`,
  section 7: `moved-repository-is-a-redirect`;
  `crates/temper-git-transcripts-tool/facts.txt`,
  `git-follows-a-renamed-repository-redirect`).
- Profiles' numbers: `limits.md`, section 1.
- **A configuration's format version** (jig's `protocol/configuration.md`,
  section 8). Default: none; unknown members refuse the start.
- A configuration directory moved or copied: `directories.md`, section 2.
- The rest of an assignment's transcript and of a person's read: `limits.md`, section 1.
- **A turn past what temper's client takes.** The engine carries a
  turn of smith's largest size whole, but temper's clients take a
  document within their own bound, jig-client's `document_bytes` as
  each shell sets it (`../../client/README.md`, section 2), which a turn
  of the agent's largest size with its wrapper can pass. Default: such a
  turn, which no plan's model reaches in ordinary use, shows as too
  large to read (jig's `protocol/store/loads.md`, 3).
- Who writes the case's workflows: `reference.md`, section 1.
