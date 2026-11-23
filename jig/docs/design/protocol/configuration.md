# Configuration

Provisional, 2026-10-10, revised 2026-10-28. What an engine and a worker
are told when they start, as far as jig defines its shape: the parts of
`../domain/engine/README.md`, section 8 that are jig's, the templates an
application defines its shapes of work with, how an application's
configuration composes them with its own, where secrets come from, and
when configuration is read. What is still open is listed in section 8.

## 1. In one page

- **Read once, before the loop.** A process's shell reads its
  configuration file whole, within a bound, with skein's startup kit
  (skein's `runtime/shell-startup.md`, 1.1), and parses it before io starts. Nothing is
  read again while the process runs: a change of the document is a
  restart. Settings change at runtime, in the store (below).
- **JSON, strict.** The file is a JSON document read with skein's JSON
  reader (skein's `data/json.md`). Unknown fields, missing fields, values past
  their bounds and duplicate names refuse the start, saying which, as
  smith's agent does with its own (smith's `protocol/agent.md`, section
  4).
- **One document per process, composed like its root.** The application's
  configuration is one object, with a member for jig's part and one for
  each of its connectors and its own children. `jig-protocol-config`
  parses jig's part; the application parses its own; its startup code
  hands each child its part, as its root hands each child its limits
  (`../domain/root.md`, section 6).
- **Typed, then checked whole.** Parsing yields plain values of jig's
  domain and protocol vocabularies. A check then runs across them: names
  unique, numbers that refer to something configured, limits whose edge
  bounds are representable (skein's programming-model.md, 6.3). A
  configuration that fails it never starts the loop.
- **Paths are typed.** Each path the document names is a configuration
  path, a data path or a cache path, by its member: relative to the
  application's directory of that kind, or absolute. The parsers resolve
  them against the directories their caller gives, touching no disk, or
  against the working directory where it gives none (section 9).
- **No secret in the file.** A secret is a file of its own, which the
  configuration names, read at startup as a private file (skein's
  `runtime/io/files.md`, 3): owned by the process's user and readable by nobody else,
  or the start is refused.
- **What lives in the store is not configuration.** Projects, their
  names, policies and roles, parties and their pools, and LLM accounts
  with their keys and records are the store's, changed by parties'
  requests. Configuration seeds projects, their names, their policies
  and their parties' roles on a deployment's first start only;
  resources and homes come by adoption after it (`configuration-engine.md`, section 1).
- **Settings are the binary's defaults with the store's changes over
  them** (`../domain/engine/settings.md`): charters, templates, the
  policies projects are made from and the rules. The application passes
  the defaults as values when it builds the engine; one that passes
  none has this document's `charters`, `templates`, `policies` and
  `rules` read as the defaults, and the store's changes lie over them
  (`configuration.md`, section 7). One that passes its own has those four members refused
  as unknown, each naming its path and that its setting lives in the
  store, so that no document seems to configure what it does not.
- **An application may fix members** as its constants (`configuration-templates.md`, 4): the
  deployment's directory and seed, the identity providers, the
  connectors, the acts, the seeded projects, the clients' heartbeat,
  link and approval, and a worker's graces and programs. A document
  naming one is refused, naming its path.
- **Templates are data the engine serves and never expands.** The
  application's shapes of work (jig's chat and goal among them) are
  numbered templates among the settings; a project's policy may override some
  of their defaults; the engine serves each project's merged templates
  to the client, which fills them into ordinary requests (`configuration-templates.md`, 1).
- **The server is the application's.** The clients' listener, its
  certificate and key, and whatever serves a browser are the
  application's service's own members; jig's part keeps what the clients'
  component takes: the heartbeat and the printed link (`configuration-engine.md`, section 1).

## 2. Where it sits

```
the application's main (its shell)
├── reads the file, and each secret file it names            skein's startup kit
├── parses: jig's part (jig-protocol-config), its own parts   plain values
├── checks the whole
└── makes the root, the protocol layer's components, io       then the loop
```

- **`jig-protocol-config`** is a crate of jig's, step code: functions from
  a parsed JSON value to jig's typed configuration and back to the
  problems it found, each naming its path in the document. It reads no
  file: it resolves each path member against the directories its caller
  gives, by joining their names (section 9). It yields jig's own plain
  values only, and links neither smith nor skein's LLM crates, so an
  application without agents never links them
  (`../README.md`, section 9); an application with agents turns the
  endpoints and charters it yields into smith's and skein's types in its
  startup (`llm.md`, section 4).
- **The application's startup** is ordinary code of its main: it reads,
  parses with `jig-protocol-config` and its own parsers, checks, and
  constructs. The reference binary's, `jig-test`'s, is the reference
  (`service.md`, section 7).

## 3. jig's part for an engine

Moved to [`configuration-engine.md`](configuration-engine.md), section 1; its subsections moved to [`configuration-templates.md`](configuration-templates.md), sections 1 to 4.

### 3.1 Templates

Moved to [`configuration-templates.md`](configuration-templates.md), section 1.

### 3.2 Ending authority in templates

Moved to [`configuration-templates.md`](configuration-templates.md), section 2.

### 3.3 A template's own resources

Moved to [`configuration-templates.md`](configuration-templates.md), section 3.

### 3.4 Members an application fixes

Moved to [`configuration-templates.md`](configuration-templates.md), section 4.

## 4. jig's part for a worker

| Member | What it gives |
|---|---|
| engine | the engine's address, and its TLS server name and trusted roots; or plaintext to a loopback address; or an in-process name, in one process (`channel/worker-link.md`, 1.1; `service.md`, section 10) |
| name, secret | the worker's name, and the file holding its secret, a data path |
| slots | how many runs it hosts at once |
| graces | the grace after losing the engine, `grace_ms`; the drain's bound, `drain_ms`, how long a worker told to stop lets its runs end their turns before it cancels them (`../domain/hosts/hub.md`, 7), a member that may be absent, 180,000 then (the user, 2026-11-12); the drain's grace, `drain_grace_ms`, how long a run asked for parts lets the calls in flight at its answer end before its agent cuts them (`../domain/hosts/hub.md`, 7; smith's `domain/turn-parts.md`, section 4), which may be absent too, 10,000 then, and is within the drain's bound, adding nothing to the stop bound; the stop graces of an agent's tree, `agent_stop_ms`; a delivery's deadline, `delivery_ms`; a save's, `save_ms`: with the link's ping interval and silence deadline, what its stop bound is made of, max(ping + silence + grace, drain) + max(agent stop, delivery) + save (`channel/worker-link.md`, 1.1; `../domain/hosts/hub.md`, 6) |
| agent | the agent program, which the worker's protocol layer spawns for each run, and which may be the application's own binary run as its `agent` command, smith's agent shell (`service.md`, section 6), with its arguments, which the worker passes as written: neither is a path of the document's; and its configuration, a configuration path, smith's agent configuration (smith's `protocol/agent.md`, section 4), which declares endpoint limits for the engine's charter models, by the same endpoint names as the engine's (`llm.md`, section 4). The worker's routing decodes charters against its endpoints, and its hello states its agent's bounds from it (`channel/README.md`, 4.3). A worker that composes `smith-inline-agent` gives it the same configuration, and runs no agent program |
| programs | `check_interval_ms`, a positive interval for the host's local login checks. The executable and private source-directory base are the agent configuration's `programs.claude_code.path` and `directory` (smith's `protocol/agent.md`, section 4), supplied with its explicit command environment below the worker root; no directory under a person's home is invented. Without `claude_code`, each source reports failed, no program configured. A model on an endpoint naming `program` takes a program source there, and a credential source on its provider's `direct` endpoint where the provider names one (`configuration-engine.md`, section 1); one on an endpoint without it takes a credential source |
| limits | a profile, and overrides |

`agent` and `limits` may be absent. An application whose worker takes
its figures from its engine names neither, and refuses them by name
(`channel/figures.md`). jig's reference binary requires both, and
refuses a worker part without them, naming the member.

The worker's workspace's configuration is the application's. Workers are
alike: every worker's configuration is the same but for its name and
secret, and every credential its runs need comes with their assignments
(`channel/README.md`, section 7). A program's login is not configuration: its
holder signs it in on each host, in the programs directory, and the
host's half reports it (`llm.md`, 5.5).

## 5. Periods

- **The core keeps no calendar** (`../domain/authority-budgets.md`, section 1). A
  project's funding period reaches it from the application's root, saying
  when the next begins if its source knows.
- **A period's source is the application's:** a child of its root that
  keeps a calendar from the application's configuration (a monthly
  period starting on a day, at a fixed offset from UTC), arms its deadline and says
  each period, or an operator's request that ends one. It is a decision,
  so it is a domain's, never the protocol layer's, which only fires the
  deadline. jig's configuration names none, and `ops`'s shows one
  (`../examples.md`, section 4).
- **The smallest source** is the reference service's: at a
  deployment's first start (section 7), before the core admits a
  party's request, its root opens one period for each configured
  project, its budget the project's spend per period, with no next
  start, since it keeps no calendar. A later start says none: the store
  keeps the current period (`../domain/engine/decisions.md`, 1.4).
- **A project made after the first start** has its first period opened
  by the core, in the decision that makes it, its budget the policy's
  spend per period, with no next start (`../domain/people/acts.md`, 2);
  the root hears that it was made, and a source that keeps a calendar
  says its periods from then on.

## 6. Secrets

| Secret | Kind | Configured as |
|---|---|---|
| a worker's secret | shared between the engine and that worker | on the worker, a file holding it; on the engine, its digest in the worker's setting (`../domain/engine/workers.md`, 1) |
| an LLM account's record or key | a signed-in source's record (its tokens, the client id the provider issued, the scopes granted, the ID token), or a key | never configured: kept in the secrets' store, from a sign-in or from a request carrying it (`store/secrets.md`; `llm.md`, 5.4). A registration is public, with no client secret; a confidential one is refused |
| the workers' listener's TLS key | a private key | a file beside its certificate |
| the clients' listener's TLS key; the secrets of the application's sign-in (an OAuth client's) | the application's server's | its own members' files; jig's part names none (`service.md`, section 7; `client/sign-in.md`) |
| the OpenID provider's signing key, and each of its confidential clients' secrets | the clients' component's (`client/openid.md`) | files `clients.provider` names (`configuration-engine.md`, section 1), the key 0600 and the service's user's own; a key replaced is a file replaced, made anew on its host, which its key set's `kid` names anew |
| a connector's credentials | the application's | the application's files, or its own secrets' records |
| an application's credential, for runs' workspaces | the application's | its own member's source: a file, or an exchange it refreshes (`configuration-engine.md`, section 1) |

- **Read at startup,** each within its bound, from a private file, with
  skein's `read_private_file` (skein's `runtime/shell-startup.md`, 1.1): a file readable
  by anyone but the process's user, or reached through a link, refuses
  the start, naming it.
- **Held by the protocol layer,** never the domain: the domains name
  secrets by number (`../domain/engine/README.md`, section 6).
- **What changes while the engine runs** is kept by its owner, never
  written back to configuration: a rotated refresh token in the
  secrets' store, as skein's driver asks it kept (`llm.md`, 5.1); a
  minted credential's digest in the store's secrets' records
  (`store/secrets.md`).

## 7. First start, and every start

- **A deployment's first start** is the first with an empty store: the
  shell draws the deployment's id and the installation's identifier, a
  `urn:uuid:…` sent to LLM providers as `ext_agent_host_id`
  (`store/keys.md`, 1.4), and the core seeds owners, the deployment's roles
  and projects from configuration into its first commit
  (`../domain/engine/decisions.md`, 1.4, and `../domain/engine/README.md`, 8). The admitted projects seed as a whole; fixed
  insufficient capacity is rejected by the whole-config check after
  overrides (`limits.md`), never handled by partial or incremental seed.
  Configuration's tests put two configured projects, with the
  deployment's roster three rosters, below, at and above the people
  capacity (capacities 4, 3 and 2), asserting `What::Bound` at the
  projects path for the last. The core's seed/restart stories use
  capacity equal to and above the rosters' count
  (`../domain/testing.md`, section 4).
- **Every later start** reads the deployment from the store, and the
  installation's identifier, ignoring the one the shell drew. The
  configuration's projects are ignored, every project, seeded or made,
  restored from the store alone. The settings are the defaults with the
  store's changes over them (`../domain/engine/settings-store.md`, 1): where the
  application passes no defaults, the document's `rules`, `charters`,
  `templates` and `policies` are them, so a restart with the same
  document and no change behaves as before; a document edited since the
  last start gives new defaults, which a setting no change is over
  follows, recorded as the release's revision, while a stored change
  stays over its member and a reset returns to it. The policies matter
  only to the next project made. Its limits, hosts and LLM providers
  apply as configured, and a narrowing reaches every later decision, as
  a policy's does (`../domain/authority.md`, section 6). LLM accounts are the store's,
  made at runtime; an LLM provider whose source is *none*, configured
  for the first time, gets the deployment's account at that start
  (`../domain/accounts.md`, 3.1).
- **What the store holds and the configuration does not name** (a
  connector gone, an LLM provider removed) is the domain's to judge at
  restart, as it judges a live set past limits lowered since
  (`../domain/engine/restart.md`); what it decides is open (`configuration.md`, section 8
  ). A charter removed is not among them: it lives in the revisions
  that hold it while tasks use them, and work under it finishes there,
  while nothing new may name it (`../domain/engine/settings.md`, 3 and 4).
  Nor is a template dropped: no task depends on one (`configuration-templates.md`, 1).

## 8. Open questions

- **Long-answer figures on the sign-in page** (`limits.md`): carrying
  skein's `LongRedirect { length, bound }` through the accounts domain
  and client wire to the application's page. Default: `SignInAgain`,
  with the specific long-answer page at the listener; the driver's
  terminal names both figures. Showing them on temper's page requires
  a client-wire change and regenerated journey recordings, left for a
  later request.
- **Reloading without a restart:** a deployment's limits changed while
  it runs. Its rules, charters, templates and policies change at
  runtime as settings (`../domain/engine/settings.md`); its limits are a
  setting too, which the application takes at its next start, and of
  which temper takes a few figures' raises at once (`limits.md`, limits
  as settings).
- **Removed parts at restart:** whether a connector or an LLM provider
  that live work or accounts name may be removed, and what becomes of
  that work: a refused start, saying which, or the work's runs failing
  as not offered. A removed charter is answered: it lives while tasks
  hold it (section 7).
- **Profiles' numbers** beyond fabrig's development, which sizes
  standard's work (`limits.md`): small's, and each profile's machine
  concurrency, from the first deployments' measurements.
- **A configuration's format version,** once a deployment's configuration
  must survive a release that changes it.

## 9. Formats an operator writes

- **An address** is one of three objects:
  - `{"socket": "127.0.0.1:7100"}`, a socket address, IPv4 or IPv6;
  - `{"host": "issuer.example", "port": 443}`, a host's name, which the
    shell resolves at startup (skein's `runtime/shell-startup.md`, section 1);
  - `{"in_process": "<name>"}`, an in-process name within skein's bound
    (skein's `runtime/io/README.md`, section 3), in one process only (`service.md`,
    section 10).
- **A path** is a string of bytes, of one of three kinds, which its
  member fixes: a *configuration path*, a *data path* or a *cache path*,
  after the application's three directories (skein's `runtime/shell-startup.md`, 1.5).
  Configuration is what the operator writes and may copy or publish (the
  document itself, charters' instructions, trusted roots, the agent's
  configuration); data is what the deployment keeps, secrets among it
  (the deployment's directory, a worker's secret, the workers'
  listener's certificate and key, the
  programs directory); cache is what can be fetched again (jig's part
  names none yet). Every trusted roots' file, the engine's, an LLM
  endpoint's, a sign-in's and a registration's, is a configuration path.
  - *An absolute path,* one beginning with `/`, is taken as written.
  - *A relative path* is joined to its kind's directory when the
    document is parsed, by joining names, without reading the disk. It
    is non-empty, and none of its components is `..`, or the start is
    refused, naming its member.
  - *The directories* are the caller's: each absolute, given to the
    parsers with the document by the application's startup, which takes
    them from its arguments (skein's `runtime/shell-startup.md`, 1.5). A caller that
    gives none, as `jig-test` and tests do, has each relative path
    resolved against the working directory. Every file the
    configuration names (`files`, `worker_files`) is reported resolved,
    so a startup reads each where its row says.
  - *The agent's program and its arguments* are no kind: the worker
    spawns them as written (`service.md`, section 6), so a bare program
    name stays bare.
- **A digest file** holds one SHA-256 digest as 64 lowercase hexadecimal
  digits, and nothing else but a final newline.
- **A secret file** holds the secret's bytes as they are, and nothing
  else but a final newline, which is not part of the secret.
- **No LLM account's value is in a file** the configuration names: a
  key comes in a request carrying it (`client/README.md`, section 3), and a
  record from a sign-in (`llm.md`, 5.4).
- **A template,** in the `templates` member (`configuration-templates.md`, 1), is one JSON object
  whose members are all required, `null` where it has none: `number`;
  `name`; `executor`, `{"charter": n}` or `"chosen"`; `contract`,
  `{"report": {"words": n}}`, `{"verdict": {"choices": [{"code": c,
  "words": n, "followups": n, "fields": [{"name": s, "max": n}]}]}}`
  (each choice's code, its words' most bytes, the most follow-up tasks
  it may propose, and each field it requires, with its value's most
  bytes: `../domain/tasks/README.md`, section 3) or `{"change": {"connector":
  c, "kind": k, "words": n}}`, a connector's result (an approval names
  a state its procedure gives, so no template makes one); `shallower`,
  the levels of delegation its authority is below the creator's role; `budget`,
  `{"amount": n}`, `{"as_allowed": cap}`, the cap a number or `null`, or
  `"given"`; `deadline`, `"none"`, `{"span_ms": n}` or `"given"`;
  `tracked`, `null` or `{"priority": n}`; `conversation` and `listed`,
  booleans; `retention`, `"kept"` or `"forgotten_once_read"`; `label`;
  `grace_ms`, or `null`; and `llm`, `null` for none, or
  `{"main": {"model": m, "effort": e}, "sub_agents": [{"model": m,
  "effort": e}]}`, each `m` a configured model's name and each `e` an
  effort it takes. jig's chat: `{"number": 1, "name": "chat",
  "executor": {"charter": n}, "contract": {"report": {"words": n}},
  "shallower": 1, "budget": {"as_allowed": null}, "deadline": "none",
  "tracked": null, "conversation": true, "listed": false, "retention":
  "kept", "label": 1, "grace_ms": n, "llm": null}`; jig's goal:
  `{"number": 2, "name": "goal", "executor": "chosen", "contract": {"report":
  {"words": n}}, "shallower": 1, "budget": "given", "deadline":
  "given", "tracked": {"priority": 1}, "conversation": false, "listed":
  true, "retention": "kept", "label": 2, "grace_ms": null, "llm":
  null}`, each `n` the application's (`configuration-templates.md`, 1). jig's defaults are written,
  never implied: the `templates` member holds each template the engine
  has, jig's among them, and an application drops one of jig's by
  leaving it out (`configuration-templates.md`, 1).
- **A policy's override of a template,** in its `templates` list, is one
  JSON object: `template`, its number; and `executor`, `budget`,
  `deadline` and `priority`, each in the template's form (`priority` a
  number), or `null` where the template's own stands:
  `{"template": 2, "executor": null, "budget": null, "deadline": null,
  "priority": 5}`.
- **A charter's default wake policy,** its optional `wake` member (`configuration-engine.md`, section 1
  ; `../domain/tasks/messages.md`, 3), is one JSON object whose members are all
  required: `words`, `notices` and `news`, each a rule, `"never"`,
  `"immediate"`, `{"batch": {"count": n, "age_ms": n}}` or `{"paced":
  {"span_ms": n}}`, its count, age and span above zero; `results`,
  `"never"`, `"each"` or `"last_or_failure"`; and `questions`, `answers`
  and `timers`, booleans. Left out, the charter's default is jig's: every
  rule `"immediate"`, `results` `"last_or_failure"`, the three `true`. A
  coordinator's: `{"words": "immediate", "notices": "immediate", "news":
  {"paced": {"span_ms": 1800000}}, "results": "each", "questions": true,
  "answers": true, "timers": true}`.
