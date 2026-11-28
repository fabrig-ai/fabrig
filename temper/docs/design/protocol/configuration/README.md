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

- **A person writes `temper.toml`** (`file.md`): a few lines of TOML,
  one file per binary, read through one table. The JSON document this
  document gives is the start's document the file gives (`file.md`,
  section 4), built by the code and never a file; `temper.json` is
  refused by name. Where a section here says a person writes a member
  or `temper.json`, `file.md` wins.
- **One document per `temper-box serve`,** in jig's shape: three top-level
  members, `engine` (jig's engine part), `workers` (jig's worker parts)
  and `application` (temper's members). Nothing else at the top level.
  The forge's setup's commands read the same document (`startup.md`, section 1). Its
  members give its shape: standalone, or the cluster's engine alone or
  worker alone (`README.md`, section 2).
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
  worker (`worker.md`, section 1).
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
  document names are jig's, a worker's secret, read at
  start, and the scripted model's key, read at the playground's start;
  an owned or fake forge's handover is in its directory (section 5).
- **LLM providers, never accounts.** The LLM providers and their
  endpoints are settings in the store, the stock's their defaults, each
  applied at the engine's next start (`providers.md`). Accounts are made
  at runtime from the client, and jig keeps them (section 2;
  ../credentials-llm-accounts.md, section 1).
- **One format rule** for temper's typed members: member names are the
  Rust field names, and the parser follows the types (section 8).
- **Limits by name,** their numbers in `temper-service`'s limits module,
  overridden by dotted name: the engine's and the box's by two kinds of
  settings, jig's `limits` and temper's `temper-limits`, which their
  documents no longer hold (`limit-settings.md`), and a worker alone's
  by its document until its figures come from the engine. The forge component's (`renew_before`,
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
- **The agent's configuration is smith's,** and temper writes it from
  the figures the engine sends each worker at its opening (`worker.md`, section 2;
  `figures.md`). A worker's document holds no limit and no `agent`.
  `temper-box open` takes no file at all (`README.md`, section 7).
- **Settings leave the document** (`settings.md`):
  jig's charters, templates, policies and rules, and temper's landing
  rules and forge's charter, become settings in the store, changed by a
  decision an owner accepts, with the binary's stock values as their
  defaults. The document keeps what a start needs before it reads its
  store, and what belongs to the machine.

- **Members that moved** (`moved.md`): what nobody changes is temper's
  constant, jig's deployment, providers, connectors, acts, seeded
  projects, clients' heartbeat, link and approval and a worker's graces
  and programs among them, and a worker's workspaces and git; the
  welcome is a setting; a forge's place is chosen when it's set up; the
  LLM providers are settings and the playground's scripted model the
  box's own (`providers.md`). A document naming one is refused, naming
  where it went. Where this document, `engine.md` or a setup document
  still describes one, `moved.md` and `providers.md` win.
- **Settings leave the document** (`settings.md`):
  jig's charters, templates, policies and rules, and temper's landing
  rules and forge's charter, become settings in the store, changed by a
  decision an owner accepts, with the binary's stock values as their
  defaults. The document keeps what a start needs before it reads its
  store, and what belongs to the machine.

Documents:

- `settings.md`: settings in the store and their lifecycle.
- `settings-kinds.md`: the settings kinds and their values.
- `settings-output.md`: output-derived bounds for larger models.
- `worker.md`: the worker and agent configuration.
- `startup.md`: configuration at the first and later starts.
- `limits.md`: the limits catalogue, stock and sizing.
- `limits-startup.md`: cross-limit startup checks.
- `limits-client.md`: engine and client bounds and watch rooms.

- `directories.md`: the configuration, data and cache directories.
- `file.md`: the TOML file and its table.
- `file-formats.md`: the format of each file a person writes, its moves
  and how every shipped format keeps reading.
- `engine.md`: the engine configuration.
- `figures.md`: the workers' figures.
- `derived.md`: derived figures and formulas.
- `limit-settings.md`: limits as settings.
- `moved.md`: members that moved.
- `providers.md`: LLM providers in the store.
- `stock.md`: stock defaults.
- `stock-settings.md`: default charters, templates, policy and project settings.
- `reference.md`: the reference configuration.

## 2. The document

```
{
  "engine":      jig's engine part
  "workers":     [ jig's worker part ]
  "application": temper's members
}
```

- **`engine`** is jig's engine part (jig's `protocol/configuration-engine.md`,
  section 1), in jig's formats (its section 10). Its `clients` member is
  what jig's clients' component takes; the clients' listener is temper's
  (`engine.md`).
- **The LLM providers are settings,** jig's kind `provider`, each with
  its endpoints, the stock's OpenAI and Anthropic its defaults, and the
  box's scripted provider its constant (`providers.md`); `engine.llm` is
  refused (`moved.md`, 3). A start runs the providers its engine last
  kept, and every check below of a provider runs on them. A provider
  holds no model, account, key or record: models are settings
  (`stock.md`; jig's `domain/engine/settings-catalogue.md`, 1), and accounts are
  made from the client (../credentials-llm-accounts.md, section 1). The
  registration's name hint is `temper` in every deployment (jig's
  `domain/providers.md`, 2.2); a provider temper does not ship is
  another setting, such as a local model's endpoint on loopback with no
  source to hold.
- **A sign-in's redirect** binds a loopback listener on an ephemeral
  port for each sign-in, unless the LLM provider's registration fixes a
  port (jig's `protocol/configuration-engine.md`, section 1; skein's
  `oauth/accounts.md`, 5). A fixed port is checked at start against the
  deployment's own listeners: the clients' and the webhooks', the
  machine's owned and fake `listen` (`engine.md`) and `demo`'s fake LLM's
  (../setup/playground.md), whose ports it must not share on loopback.
- **A sign-in's hosts** are resolved once at start, as skein's accounts
  driver takes them (skein's `oauth/accounts.md`, 2): its token endpoint's,
  and the origins of its issuer, its keys and its revocation. One that
  does not resolve, the default document's `auth.openai.com` on a
  machine without DNS (../setup/init.md, 6), refuses no start: its address is
  held at `127.0.0.1:0`, which no listener can hold, so each exchange of
  that provider's sign-in, a sign-in, a refresh or a revocation, fails
  at its connect as unreachable, until a start that resolves it, as a
  held forge waits for one (../setup/init.md, 6). An address, port, server
  name or trust file that is wrong still refuses the start, naming its
  member; Google's sign-in's endpoints (`../sign-in.md`, 3.1) still refuse
  it, naming the host.
- **`workers`** is a list of jig's worker parts (jig's
  `protocol/configuration.md`, section 4).
- **`application`** holds temper's members (`engine.md` and `worker.md`, section 1).
- **jig's reference fixtures** are a complete example of jig's parts: the
  reference binary's test configuration (`jig-test`'s
  `tests/support/mod.rs`, made from jig's channel world's `fixtures.rs`).
  temper's process tier writes its own the same way (`../../testing/process-tier.md`
  ), and the owned example is a whole document (../setup/init.md, 4).
- **What standalone requires of jig's parts,** each checked at start, a
  failure naming its path:
  - `workers` has exactly one entry, named `local`, whose `engine` is
    `{"in_process": "workers"}`: the box's worker, the setting `local`
    the box passes as its default (`../cluster.md`, 1);
  - `engine.hosts.listener` is `[{"in_process": "workers"}]`, and
    `engine.hosts.workers` is refused by name: workers are settings
    (jig's `domain/engine/workers.md`);
  - the worker part names no `agent` and no `limits` (`figures.md`,
    section 7).

  `engine.connectors` is temper's constant `[1]` (`moved.md`, section
  2): the forge connector, temper's one connector, whose number both
  ends of the client's wire fix as `temper-client-protocol-wire`'s
  `FORGE` (`../../client/forge/records.md`).

  The worker's agent is temper's own binary, derived (`figures.md`,
  section 6).
- **The shape is read from the members,** before any other check, and
  each shape's checks are its own (`../cluster.md`, 1):

  | Shape | `engine` | `workers` | `application` |
  |---|---|---|---|
  | standalone | present, `hosts.listener` `[{"in_process": "workers"}]` | one, its `engine` `{"in_process": "workers"}` | every member, `environment` among them |
  | engine alone | present, `hosts.listener` one socket address on loopback (in `127.0.0.0/8`, or `::1`), plaintext, its port fixed or 0; its workers settings an owner adds (`../cluster.md`, 2) | `[]` | every member but `environment` |
  | worker alone | absent | one, its `engine` a socket address on loopback, plaintext | `environment` or none; no other member |

  - an engine alone is checked as standalone is, but for its listener,
    by the table, and its workers, which it has none of by default;
  - a worker alone is checked for its worker part and its
    `application.environment` (`worker.md`, section 1), then for the figures its engine
    sends and the agent configuration written from them (`figures.md`,
    section 3), and for nothing of an engine's;
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
  goal (template 2, label 2), as jig's `protocol/configuration-templates.md`,
  1 gives them, the chat's executor the charter chats run under;
  temper's own, templates 4 to 8, are the stock's (`stock-settings.md`, 1), template
  8 a milestone's (`../../domain/milestones.md`, section 2). One flag departs from jig's
  defaults: the chat is ended by its requester, `"ended_by":
  "requester"` (jig's `protocol/configuration-templates.md`, 2), so that a chat
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
     setup's and the box's limits, by their own crates' checks (../setup/init.md and ../setup/playground.md and ../setup/crate-layout.md, section 1
     );
  3. the checks across the whole that its shape holds: the references
     of `README.md`, section 2 and `engine.md` and `worker.md`, sections 1 to 2,
     a git identity's largest value against the channel's bound on a
     credential's value (`engine.md`), the limits against each other and
     the edge bounds (`limits.md`), and the channel's start checks on
     temper's parts (../channel.md, section 8);
  4. every private file the document names, each within its bound
     (section 5);
  5. the agent's configuration (`worker.md`, section 2).

  Every problem found is reported, one line each on stderr, and `temper-box
  serve` exits 1 before io starts. No forge is read here: the forge
  connector's records are restored from the store, and the engine's
  routing loads their credentials from jig's secrets' store, once io
  starts, before the forge component starts; a forge whose records do
  not serve is not connected, never a refused start (`startup.md`, section 1;
  ../forge/engine.md, 1; ../credentials.md, 2.1).
- **Paths are typed:** each is a configuration, data or cache path,
  relative to its directory or absolute, as jig's parts' are (jig's
  `protocol/configuration.md`, section 9), or a path that must be
  absolute; `directories.md` gives each member's. None is resolved against
  the working directory.

## 5. Secrets' files

| File | Named by | Holds | Read by |
|---|---|---|---|
| an owned or fake forge's handover: its admin's token | `admin_token` in the forge's directory under data, named by its number, `forge-N` for an owned forge (../setup/README.md, section 3); named by no member | the token's bytes | the engine's setup host, once, at start or the box's `started` (../setup/owned.md, 5); a fake forge, before binding (../setup/playground.md) |
| a worker's secret | jig's worker part, a data path: `worker.secret` in the data directory as `temper-box init` writes it (../setup/init.md, 5); its digest is the worker's setting's, no file (jig's `domain/engine/workers.md`, 1) | jig's formats (jig's `protocol/configuration.md`, sections 6 and 9) | jig's components |
| Google's client secret | `application.sign_in.google.secret`, a data path | the secret's bytes | `temper-box serve`, once, at start, within `sign_in.secret_bytes` |
| the OpenID provider's signing key, and each of its confidential clients' secrets | jig's `engine.clients.provider` (jig's `protocol/configuration-engine.md`, section 1) | jig's formats: the key PKCS#8 PEM, made by `temper-box forge sign-in` (../setup/owned-sign-in.md, 1) or a box's first run (`../openid-provider.md`, 9) | `temper-box serve`, once, at start, for jig's helper that builds the issuer (`../openid-provider.md`, 2) |
| the scripted model's key | `application.demo.llm.key`, a data path: `demo/llm.key` as `temper-box init` writes it (../setup/init.md, 5) | the key's bytes | the fake LLM, at its start; `temper-box serve`, at the playground's start, which makes the scripted account with it (../setup/playground.md) |

- **A secret file holds its bytes and nothing else** but a final
  newline, which is not part of the secret (jig's
  `protocol/configuration.md`, section 9).
- **Private reads,** each within its bound, with skein's
  `read_private_file` (skein's `runtime/shell-startup.md`, 1.1): a file readable by
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
  directory, which temper never reads (../credentials-llm-accounts.md, section 1).

## 7. The client's arguments

- **`temper-box open DIRECTORY LINK_OR_ADDRESS`** takes no configuration
  file: its two arguments are `../../client/line.md`, 1's, and its limits are
  constants of temper's client (`../../client/README.md`, section 2).
- **`temper ctl DIRECTORY ...`** takes none either (`../../client/ctl/command-line.md`,
  section 1).
- **`temper-box program-login [DIRECTORIES] NAME`** reads the same document
  as `temper-box serve`, and the agent's configuration it names
  (../credentials-llm-accounts.md, section 1). `DIRECTORIES` is `[--config DIR] [--data
  DIR] [--cache DIR]`, each with its variable and its default (`directories.md`
  ), given anywhere among the command's arguments.
- **`temper-box internal agent AGENT.json`** reads smith's agent configuration
  (`worker.md`, section 2).
- **`temper-box forge prepare|run [DIRECTORIES] FORGE`** and **`temper box
  [DIRECTORIES]`** read the same document as `temper-box serve` (`startup.md`, section 1;
  ../setup/owned.md and ../setup/box.md); **`temper-box init [DIRECTORIES] [--box]`**
  writes it (../setup/init.md, 5). Reconcile is no command: the
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
    "name": "local",
    "secret": "worker.secret",
    "slots": ..., "graces": ...,
    "programs": {"check_interval_ms": ...},
    "memory_bytes": ...
  }],
  "application": { ... as above ... }
}
```

Its `engine.limits` names the overrides `limits.md` asks. The worker
takes its limits and agent configuration from the engine's figures
(`figures.md`), so its part names neither `limits` nor `agent`.
The owned example, `docs/examples/owned.json`, is a whole document for
one owned Forgejo's place, with its own values (../setup/init.md, 4).

## 9. Open questions

- A landing branch with another name than `main`: `stock.md`, section 9.
- **The reference's project rule in stage one** (`reference.md`): its settings
  description must include the project's own rule 1 alongside the
  deployment's rules, so a goal's branches do not inherit the stock's
  CI and review. Default: apply `project.json` after making the project;
  the golden's missing obligation remains a gap in the status file.

- Ask temper's real-model walk: `stock.md`, section 9.

- **A cut sub-agent answer's rest** (`settings-output.md`, 1): smith's `domain/run.md`,
  section 13 leaves retention for a continuation undecided. Default:
  derive the answer room from the payload and ask a new child, narrower,
  for omitted material; temper declares no continuation setting.
- **A coordinator's repository notes** (`stock-settings.md`, `reference.md`): charter 5 asks
  for notes about flakes and inherited failures in the repository's
  scope, which the stock's goal-only note authority does not grant.
  Default: retain the narrow authority; satisfying that instruction
  needs separate authority work, and remains a gap in the status file.

- The reference's later runs: `reference.md`, section 5.

- Settings refusal positions: `settings.md`, section 9.
- The checks' deadline: `reference.md`, section 5.
- A reference call of 128 KB: `settings.md`, section 9.
- Large arguments through a real program: `settings.md`, section 9.

- A chat checking the forge before declaring a repository absent: `stock.md`, section 9.
- **Four agents' resident memory** (`stock.md`): the slot, workspace and git
  operation fit checks admit four; they do not establish the memory used
  by four agent processes and their programs on a laptop. Default: four
  slots, with the after-landing measurement in `../setup/README.md`, section 9;
  a person's existing document keeps its configured concurrency.
- A real provider's list: `settings.md`, section 9.
- The real-browser check of watch sizing: `limits.md`, section 8.
- A page refused busy: `../../client/pages/README.md`, section 7.
- A party's fair share of the watch rooms: `limits.md`, section 8.
- `views.`'s other names: `limits.md`, section 8.
- A Claude-only box on the small profile: `settings.md`, section 9.
- A message, a report or a person's spec past what a box takes: `settings.md`, section 9.
- A brief past the host part's room: `settings.md`, section 9.
- Requirements an application fixes: `settings.md`, section 9.
- A kept GitHub forge whose base ends in `/`: `../forge/engine.md`, section 9.
- **A repository renamed or moved after adoption.** Default: its
  resource path, which landing patterns match, and its remote keep the
  owner and name of its adoption; Forgejo redirects the old path, and
  git follows the redirect (skein's conformance, `skein/docs/design/testing/fakes/throwaway.md`,
  section 2: `moved-repository-is-a-redirect`;
  `crates/temper-git-transcripts-tool/facts.txt`,
  `git-follows-a-renamed-repository-redirect`).
- Profiles' numbers: `limits.md`, section 8.
- **A configuration's format version** (jig's `protocol/configuration.md`,
  section 8). Default: none; unknown members refuse the start.
- A configuration directory moved or copied: `directories.md`, section 2.
- The rest of an assignment's transcript and of a person's read: `limits.md`, section 8.
- **A turn past what temper's client takes.** The engine carries a
  turn of smith's largest size whole, but temper's clients take a
  document within their own bound, jig-client's `document_bytes` as
  each shell sets it (`../../client/README.md`, section 2), which a turn
  of the agent's largest size with its wrapper can pass. Default: such a
  turn, which no plan's model reaches in ordinary use, shows as too
  large to read (jig's `protocol/store/loads.md`, 3).
- Who writes the case's workflows: `reference.md`, section 5.
