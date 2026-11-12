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
  (skein's `runtime/shell.md`, 6.1), and parses it before io starts. Nothing is
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
  resources and homes come by adoption after it (section 3).
- **Settings are the binary's defaults with the store's changes over
  them** (`../domain/engine/settings.md`): charters, templates, the
  policies projects are made from and the rules. The application passes
  the defaults as values when it builds the engine; one that passes
  none has this document's `charters`, `templates`, `policies` and
  `rules` read as the defaults, and the store's changes lie over them
  (`configuration.md`, section 7). One that passes its own has those four members refused
  as unknown, each naming its path and that its setting lives in the
  store, so that no document seems to configure what it does not.
- **Templates are data the engine serves and never expands.** The
  application's shapes of work (jig's chat and goal among them) are
  numbered templates among the settings; a project's policy may override some
  of their defaults; the engine serves each project's merged templates
  to the client, which fills them into ordinary requests (3.1).
- **The server is the application's.** The clients' listener, its
  certificate and key, and whatever serves a browser are the
  application's service's own members; jig's part keeps what the clients'
  component takes: the heartbeat and the printed link (section 3).

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

| Member | What it gives | Used by |
|---|---|---|
| deployment | the deployment's directory, a data path, which holds the store, transcripts and the secrets' store; retention's horizons (`store/README.md`, 6.1); the seed's source, for tests | the store (`store/README.md`); the domain's retention; the shell |
| owners | the deployment's first owners, each a provider and a subject (`../domain/people/README.md`, section 3), each an owner of every project seeded with it and of the deployment's roles | the people child, on a first start |
| providers | the numbers and kinds that identities name (`../domain/people/README.md`, section 3), and nothing else: jig runs no provider's exchange. Of kind *link*, the printed link's; of kind *deployment*, exactly one, the deployment's own, for services' credentials; of kind *application*, each of the application's ways of signing in, whose identities its server verifies on its own routes and asks the clients' component to mint a credential for (`client/sign-in.md`). The settings and secrets of the application's sign-in are its own members | the people child; `jig-protocol-engine-clients`, which mints a printed link's credential for the first owner it names, a service's under the deployment's provider, and a `Mint`'s only for an identity whose provider is of kind application |
| rules | the deployment's rules, as the default of that setting when the application passes none (`../domain/engine/settings.md`, 7): the ceilings every project's policy is under (`../domain/authority.md`, section 6). A ceiling, the rules' or a project policy's, names in `models` the configured models it allows, from the llm member's, or every one without it; a role's authority names its models the same way: they seed the role on a first start, and the store keeps them by name (`store/records.md`, 1) | authority |
| projects | each project seeded on a first start: its number, from 1, since 0 is the deployment's roles' and refused here (`What::Bound` at the number; `../domain/people/README.md`, section 3); its name, which may be left out for "Project N", N its number, the seed's names unique and in shape or the start refused (`../domain/model.md`, 6.1); its policy, either inline, with its spend per period, its roles (each one's authority, the spend a party in it may allot per period, the kinds of proposal it decides; `../domain/authority.md`, section 6) and its overrides of templates' defaults (3.1), or as `policy: N`, the number of a policy among the settings, which the seed copies from the current settings as `MakeProject` copies one (`../domain/people/acts.md`, 2), so that an application's stock policy reaches its seeded projects with no copy of its own; its parties' roles. Its resources and homes come by adoption (below) | the core, on a first start |
| policies | each policy a project made after the first start starts from (`../domain/model.md`, 6.1), as defaults when the application passes none (`../domain/engine/settings.md`, 7), with the store's changes over them: numbered from 1 and never renumbered, as templates are (3.1); each a whole policy in a seed's format: its ceiling, its spend per period, its roles, its escalation role, its requirements, its projections' authority, its permissions' roles for adoption and its overrides of templates' defaults. Checked at every start as a seed's policy is: within the rules, its roles well formed, its overrides' templates configured, and its owner role able to be a root's requester (`../domain/people/acts.md`, 1); one that fails refuses the start, at its path. A project made from one keeps a copy, which no later change or removal here reaches (`../domain/people/acts.md`, 2). With none, no project is made after the first start | the core, at each `MakeProject` |
| charters | each charter the deployment offers, as defaults when the application passes none (`../domain/engine/settings.md`, 7), with the store's changes over them, numbered: its instructions (a file it names, a configuration path), its documents (`../domain/engine/settings.md`, 9: each a resource's name, a path and an optional heading, `{"connector": n, "name": [s, ...], "path": s, "heading": s}`, `heading` optional, and none when the member is absent), its models, named from the catalogue's (its main model, and those its sub-agents may use), and `rest`, whether every other model of the catalogue follows them (`../domain/engine/settings.md`, 11; a charter with no `models` member takes them all), its tool families: smith's (inspect, modify, shell, agents) and the host families its runs are offered (`tools`, a `ToolFamilies` mask, offered where the task's authority holds them too: `../domain/engine/executors.md`, 1), the hosts that may run it, its waiting time, its default wake policy (`../domain/tasks/messages.md`, 3: `wake`, below), which a task made under it takes when its batch entry names none, a template's task among them (3.1), whether its runs resume, its parts (`parts`, below; none when the member is absent), its default budget (turns, spend and time, and a reserve of turns and time), which applies to a task whose batch gives none (`../domain/hosts/charters.md`). Its models are those it allows, in order; a task that names none takes the first whose LLM provider has a usable account serving its payer (`../domain/tasks/README.md`, section 3). It names no account: the core chooses one with each run's check (`../domain/accounts.md`, 4.3) | the core; `jig-protocol-charter` |
| templates | each template the application defines, as defaults when the application passes none (`../domain/engine/settings.md`, 7), with the store's changes over them, numbered and never renumbered, with its defaults; jig's chat and goal among them unless the application drops them (3.1) | the core, only to serve each project's merged templates in `me` (`../client/domain/contract/documents.md`, 1); the client, which fills them in |
| hosts | the engine's own slots, if it runs agents in the engine (shape 2), and their `smith-inline-agent`'s limits; the workers' names, each with the file of its secret's digest, a data path; the workers' listener: its addresses, socket addresses or an in-process name (skein's `runtime/io/README.md`, section 3), and its certificate and key files, data paths, but for an in-process name (`channel/README.md`, section 9); a listener on an in-process name tells the core its workers are in its process, and that no worker outlives it (`../domain/engine/restart.md`) | the fleet; `jig-protocol-engine-workers`; `smith-inline-agent`, through the application's startup |
| llm | the LLM providers (`../domain/providers.md`), each numbered, with its name for showing and the `endpoint` its added models take, and, where that endpoint names a program, an optional `direct` endpoint, naming none, that its sign-ins and keys take (below); its endpoints, each with an optional `program` named among its provider's `programs`; where the application passes no defaults, the models, each a default of the `model` setting (`../domain/engine/settings.md`, 1), numbered by position from 1, and each provider's `models`, naming them, both refused where it passes them: each model's window, output, the efforts it takes, low to high, or none (`efforts`), and the one of them a task given no LLM choice takes (`effort`, the first when left out; `../domain/tasks/README.md`, section 3), its name for showing (`shown`, optional), whether a picker shows it (`picker`, true when left out; `../client/domain/contract/documents.md`, 1), and the rest of what smith's agent configuration declares of it (smith's `protocol/agent.md`, section 4), its prices (input, cached input, output, and cache writes when priced apart), its prompt cache's life in milliseconds (`cache_life_ms`, optional, 0 or left out for a model with no prompt cache; `../domain/tasks/standing.md`, 5.3) and its optional `description`, what it is good at, for the brief's catalogue (`../domain/engine/briefs.md`, section 1); the sources it offers: a sign-in, with its public registration (its entry client id, its name hint, its authorization, token, revocation and published keys' addresses with their transport, its issuer, its scopes and those that must be granted, its resource, its redirect's path, and its port where fixed, an ephemeral loopback port for each sign-in otherwise), a key, a program, named `claude_code` where the program is Claude Code (smith's `protocol/programs.md`, section 3), or none; a sign-in waits for its redirect within the profile's sign-in deadline, as a limit of the driver (`limits.md`; skein's `oauth/README.md`, section 3); and the facts its terms ask an application to show: whether a signed-in source is a plan, and its manage-usage address (`../domain/providers.md`, 2.6). No account, key or record: accounts are made at runtime and kept in the store (`../domain/accounts.md`, 3.1; `llm.md`, sections 4 and 5) | the core; smith's and skein's components and skein's accounts driver, through the application's startup; `jig-protocol-accounts` |
| clients | the heartbeat of the clients' streams (`client/wire.md`, section 2); whether the clients' component gives a printed link's one-time secret at start, whose URL the application's startup prints (`client/sign-in.md`), only where its clients reach it on loopback, which that startup checks against its own listener (`service.md`, section 3); `approval`, a boolean, false when absent, whether it holds sign-ins approved by a signed-in browser (`client/sign-in.md`), apart from `link` and only on loopback too, which the startup checks the same way; `provider`, the OpenID provider's (`client/openid.md`), or `null` for none: `key`, the file of its signing key, an RSA key of 3072 bits as PKCS#8 PEM (`-----BEGIN PRIVATE KEY-----`), made on the host that uses it by the application's setup (`openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:3072`), mode 0600 and owned by the service's own user, never copied, printed or logged; the start refuses, each with its reason, a key of another form or size, a file readable or writable by group or others, reached through a link, or owned by another than the process's effective user; `clients`, each its `id` (1 to 255 visible ASCII bytes, unique), the file of its `secret` or `null` for a public client (skein's `oauth/issuer.md`, section 2), and its one `redirect_uri` (`https`, or `http` on loopback with a port), at least one; and `addresses`, the providers, each of kind application or link, whose subjects are verified addresses, given as the claims' `email`, a link's being the deployment's own word on its one owner. A key skein's issuer refuses, a client off its rules, or a provider in `addresses` of another kind refuses the start at its path. The provider's codes and access tokens are counted by the limits `provider_codes` and `provider_tokens` (`limits.md`). The listener, its certificate and key, and what serves a browser are the application's service's own members (`service.md`, section 7) | `jig-protocol-engine-clients` |
| connectors | the application's connectors' numbers, as the core knows them (`../domain/model.md`, 3.5); none takes the number the core keeps for its own procedures (below) | the core |
| acts | the deployment's acts each connector may make in its owners' name (`../domain/people/acts.md`, 3), none without the member: a list of `{"connector", "may"}`, each `connector` one of `connectors`, at most once, and `may` the kinds it is granted, each at most once, among `close`, `add_key`, `remove_account`, `adopt`, `make_project` and `remove_project`. A connector not configured, or named twice, and a kind unknown, or named twice, refuse the start at their path. The validated entry count becomes people's `act_grants` memory bound, checked by its constructor; each entry charges at most one of every act kind. Every act is refused while `owners` names none | the people child, at every start |
| limits | a profile, and any limit overridden by name (`limits.md`) | every child and component of jig's |

The LLM provider's `name` is nonempty UTF-8 without control characters,
at most 256 bytes. Bidi controls U+061C, U+200E–200F, U+202A–202E,
U+2066–2069 and invisible formatting U+00AD, U+180E, U+200B–200D,
U+2060–2065, U+206A–206F, U+FEFF, U+FFF9–FFFB are refused.
`signed_in_plan` is a required boolean for an offered sign-in. Without a
sign-in it may be absent or false; true requires an offered sign-in. `manage_usage`, when present, is a
whole public HTTPS address, at most 2,048 ASCII bytes, with a nonempty
authority and no user information, query, fragment, whitespace or markup.
The address check is syntactic: private and loopback hosts are accepted;
uppercase `HTTPS://` is refused.
A signed-in plan or an offered program requires that address. Other
providers may omit it. These are public facts, independent of a sign-in's
registration. A program source remains a plan even when `signed_in_plan`
is false; account classification is recomputed from its remaining sources
on removal and from current configuration on restore. No separate plan
display name is configured.

**A charter's parts** (`../domain/tasks/messages.md`, 6) are a list, in order, each
`{"on": [s, ...], "instructions": s}`: `on` the reasons it is for, each
`first` or a kind that chooses parts, named as jig's store names it, at most
once; `instructions` a file it names, read as the charter's own instructions
are, whose text a run its reasons meet reads after the charter's. The start
is refused at the path of what fails: a name that is no kind, an
always-offered kind or a name given twice (`charters[i].parts[j].on[k]`); an
empty `on` (`charters[i].parts[j].on`); and a part past `tasks.charter_parts`
(`charters[i].parts[j]`). A charter whose runs resume may hold parts
(`../domain/tasks/standing.md`, 5.4). The charter's stored policy, its parts with it, stays
within `core.run_bytes` (`limits.md`), as its instructions do alone.

A provider's `keep` and `drop` say which names of its list are models for this
work (`../domain/accounts.md`, 3.7; `../domain/engine/settings.md`, 12). A name is kept
when a `keep` pattern matches it and no `drop` pattern does. Each is a list of at
most 16 patterns of 1 to 64 bytes of a model name's own bytes (`valid_name`,
`../domain/engine/settings.md`, 11), `*` standing for any run of them, `?` for exactly one,
matched against the whole name. `drop` is optional and absent is none. A provider
with no `keep` has no list, and none is read; with `keep`, `endpoint` is required,
since a read goes there, or to its `direct` endpoint where it names one. A bad pattern, or one past its bound, refuses the start at
its path: `llm.providers[i].keep[j]`.

A provider's `direct` is the second way to its models, for a provider whose
`endpoint` names a program and that offers a sign-in or a key: its program
sources reach its models through the program on `endpoint`, and its sign-ins,
keys and *none* reach the same models directly on `direct`
(`../domain/accounts.md`, 3.4). It names one of `endpoints` that names no
program and speaks `endpoint`'s dialect, so that each of the provider's models
is the same model either way; it may share `endpoint`'s address. The start is
refused at `llm.providers[i].direct` where it names no endpoint, one naming a
program or one of another dialect, or where the provider's `endpoint` names no
program or the provider offers neither a sign-in nor a key. A model keeps its
`endpoint`, the program's; the endpoint a run takes is its source's
(`../domain/accounts.md`, 4.3), and a list is read on `direct`
(`../domain/accounts.md`, 3.7). A provider with one way, as OpenAI, names no
`direct`.

- **A delegation's kinds,** in an authority the rules, a policy or a
  role gives (`../domain/authority.md`, section 3): a list of
  `{"kind", "number"}`, at most `authority.executors`, `kind` one of
  `charter`, `procedure`, `role` and `job`. A charter's number names one
  of `charters` and a role's one of its policy's roles, or the start is
  refused at the number; a procedure's number is its connector's and a
  job's its application's kind, neither checked here, since the core
  refuses a batch naming a job of a kind the application does not
  declare (`../domain/engine/executors.md`, 5). Any other `kind` refuses the
  start at its path.
- **Each connector's own configuration** (its system's address, its
  credentials' files, the address of its webhooks' listener, its
  procedures' settings and texts) is the application's, in its own
  member.
- **An application's credentials** for its runs' workspaces (a token to
  fetch and push) are its own members: each with its number, as the
  domain names it, and its source, a file or an exchange it refreshes.
  jig's part names none; the accounts child hears each generation from
  the application's root (`../domain/engine/README.md`, 6.1; `channel/README.md`,
  section 7).
- **The application's own children** take their configuration from the
  application's members, as its root gives them.
- **What the core takes that no member names:**
  - *a project's current period and its budget,* from the application's
    root as each period begins (section 5), the first at the
    deployment's first start, before the core admits a party's request;
    the store keeps the current one across restarts
    (`../domain/engine/README.md`, 5.4), and the policy's spend per period
    bounds its budget;
  - *a party's pool* for a period: the allotment their role gives, from
    the project's policy;
  - *a task's authority and budget, as a template gives them:* in the
    request, never in configuration the core reads. The client fills a
    template's authority (the creator's role narrowed) and its budget
    into a `Create`, and the core checks them as it checks any request
    (`../domain/authority.md`, sections 5 and 6), a budget given as
    `AsAllowed` resolved at admission (`../domain/authority.md`, section
    7);
  - *the core's recurring procedure* (`../domain/tasks/README.md`, section 5):
    the core's own, under a number the core keeps for its procedures,
    which no connector takes;
  - *the engine's tool families' bits* in an authority: jig's own
    numbering, the same in every deployment; a charter names the
    families it allows;
  - *a fresh brief's transcript text:* `core.brief_core_budgets.tail`,
    2048 bytes in the small profile, separate from the task section's budget;
  - *a standing task's Now:* `core.brief_core_budgets.now`, 4,096 bytes in
    both profiles, the room its brief places it whole in
    (`../domain/tasks/standing.md`, section 6);
  - *a resumed conversation's bytes and the hub's turns:* limits, from
    the profile (`limits.md`).
- **Resources and homes come by adoption,** after the first start, by an
  owner's request (`../domain/people/acts.md`, 1): a resource is a
  project's once its connector has read what the application may do
  there (`../domain/connectors.md`, section 12), which configuration
  cannot say. A home is a resource its adoption names as the project's
  home for that connector. A project seeded with none has none until
  then.

Every model of the catalogue belongs to exactly one numbered LLM provider,
including providers whose source is none (`Model.provider`). Where the
application passes defaults, `endpoint` is required of each provider. Omitted
or empty providers therefore require an empty model catalogue. Endpoint backend names do
not select accounts or infer provider membership. The public JSON refuses
`llm.accounts` and an endpoint's `account` member at their paths.

### 3.1 Templates

- **A template is the application's recipe for a shape of work:** the
  defaults that turn "a chat with these words" or "a goal for this"
  into a complete `Create` (`../domain/tasks/README.md`, section 4). An
  application's own shapes (one for `ops`'s incidents, say) and jig's
  defaults are alike.
- **Numbered, never renumbered.** A number, above zero, is the
  template's identity across the configuration, projects' policies and
  clients; a template dropped leaves its number unused.
- **What each carries,** every field required (section 9):
  - a name, for showing;
  - the executor: an agent under a charter the template names, by its
    number among the charters member's; or *chosen*, a charter or
    procedure the creator's role may create with, the creator's choice;
  - the result contract: a report, a verdict or a connector's result,
    each with its bounds (`../domain/tasks/README.md`, section 3); never an
    approval, whose state only a procedure gives;
  - the authority: the creator's role narrowed, by as many levels of
    delegation as its `shallower` says;
  - the budget: an amount; `AsAllowed`, with a cap or none
    (`../domain/authority.md`, section 7); or *given*, the creator's,
    within their allotment;
  - the deadline: none; a span from creation, which the client sends
    as a span and the engine makes a wall when it makes the task
    (`../domain/authority.md`, section 7); or *given*, the creator's,
    or none;
  - tracked, with a default priority, or untracked;
  - the flags: *conversation*, *listed*, and the retention class,
    *kept* or *forgotten once read* (`../domain/tasks/README.md`, section 3);
  - the label: an opaque 32-bit number, which jig gives no meaning;
  - the grace its helpers give a close or a deletion, or none;
  - the LLM choice: none, for the first of the charter's models each run
    can use (`../domain/accounts.md`, 4.3); or a main model
    with its effort, and the models its run's sub-agents may use, each
    with an effort (`../domain/tasks/README.md`, section 3), and optionally
    `later`, the same again for every run after its first activation's
    (`{"main": ..., "sub_agents": [...], "later": {"main": ...,
    "sub_agents": [...]}}`). It names no account.

  It names no wake policy, parameters, inputs or dependencies, and
  resources only by its connectors' symbols (3.3): a task filled from it wakes by its charter's default wake
  policy, as the settings hold it when the task is made, and parks as
  its executor's defaults say (`../domain/tasks/messages.md`, 3), and its helper names none
  of the rest (`../client/domain/client/helpers.md`).
- **jig's defaults** are two, which an application keeps, changes or
  drops, by writing them in its `templates` member or leaving them out;
  none is implied (section 9). jig's helpers find them by number:
  template 1 is the chat they fill, template 2 the goal
  (`../client/domain/client/helpers.md`). What the table leaves
  unnumbered (the chat's charter, a report's bound, the chat's grace)
  the application writes, and the start checks (below).

| Field | chat (template 1) | goal (template 2) |
|---|---|---|
| name | chat | goal |
| executor | an agent under a charter the template names: the application's charter for chats | chosen: a charter or procedure the role may create with |
| contract | a report | a report |
| authority | the role's, one level shallower in delegation | the role's, one level shallower in delegation |
| budget | `AsAllowed`, with no cap | given: the creator's, within their allotment |
| deadline | none | given: the creator's, or none |
| tracked | no | yes, with a default priority |
| conversation | yes | no |
| listed | no | yes |
| retention | kept | kept |
| label | 1 | 2 |
| grace | a span, which the helpers closing a chat or deleting a project give | none |
| LLM choice | none | none |

- **A project's policy may override,** per template number, the
  executor, the budget and its cap, the deadline and the default
  priority (`../domain/authority.md`, section 6), the template's own
  standing for each it leaves `null` (section 9). Never another field,
  and never the flags, the label or tracked: they are the shape's
  identity, which lists and the engine's behaviour key on. An override
  naming a template the configuration does not define, or one another
  override of the same policy names, is refused when the policy is
  changed (as unknown, or as a repeated identity), and a seeded
  policy's refuses the start.
  The effective executor must name a configured charter, or be chosen.
  A template's retained LLM choice must fit that executor: its main and
  every sub-agent model, its `later`'s among them, must be allowed by a
  named charter, each with an effort the model declares, or none only when
  it declares none, and a `later` only under a charter whose runs don't
  resume (`../domain/tasks/README.md`, section 3). An
  override that invalidates the retained choice is refused atomically
  at configuration, policy change and restore, before `me` is served.
- **Merged for `me`:** a project's merged templates are the
  configuration's, in its order, each with that project's override
  applied: its executor, budget and deadline where the override gives
  them, and its priority where the template is tracked. An override of
  the default priority of an untracked template is kept, and shown with
  the policy, but has no effect: the merged template stays untracked.
- **A served preview is separate from configuration.** `me` adds
  `resolved_default: Option<LlmChoice>` beside each effective template's
  configured intent, what a run's check would take now for a task keeping no
  account (`../domain/accounts.md`, 4.3), in charter order,
  under the effective creating authority and current receiving-party payer
  (`../domain/engine/views.md`, 1). Explicit intent previews itself; omitted
  intent with a chosen executor or no usable model gives `None`. It is not a
  configuration member or a reservation, and shapes never turn it into
  explicit intent. A task given none keeps none, and each of its runs'
  checks takes its models (`../domain/tasks/README.md`, section 3).
- **The engine never expands one into work.** The core validates the
  merged defaults above and keeps the
  configuration's templates, and each policy's overrides, only to serve
  each project's merged templates in `me` (`../domain/engine/views.md`, 1;
  `../client/domain/contract/documents.md`, 1). The client fills a template
  into an ordinary request, which the core checks as it checks one
  written by hand. A template grants nothing: whatever it makes, a
  `Create` could make within the creator's authority. A recurring task
  repeats a batch of its own, its recurring batch, never a template
  (`../domain/tasks/README.md`, section 5).
- **Permissions stay primitive:** which charters and executors a role may
  create with, its spend and its delegation (`../domain/authority.md`,
  section 6). No role is given or refused a template.
- **A task keeps the values it was made with.** A template changed or
  dropped at a later start touches no task already made; a policy's
  override naming a template the configuration does not define at that
  start serves nothing, and the next change of that policy must leave
  it out, as above.
- **Checked at start:** no more templates than the core's limit on them
  (`limits.md`); numbers unique and above zero; each name within the
  documents' bound on a text; each charter a template's executor names
  configured; each contract within the tasks child's bounds; each grace
  within the tasks child's limit on a grace's span
  (`../domain/tasks/README.md`, section 6); each model an LLM choice names,
  its `later`'s too, configured and one its template's charter allows,
  and each effort one its model takes, `null` only for a model that takes
  none; no `later` under a charter whose runs resume; each seeded
  policy's overrides naming configured templates, each at most once,
  and no more of them than the limit on templates.

### 3.2 Ending authority in templates

Templates optionally name `ended_by` as `result` (the default),
`requester`, `standing` (`../domain/tasks/standing.md`, section 2), or a
closer, `{"closer": {"connector": C, "code": N}}`, the
procedure that ends the task once nothing is left to await
(`../domain/tasks/lifecycle.md`, 11), its connector by number and its code within
the connector's procedures. Requester requires a named agent charter and
`conversation: true`; a closer requires a named agent charter and no
conversation, and is checked at the start as an executor is: a connector
configured, and a code within `u32`; standing requires a named agent
charter, no conversation and no tracking. Effective executor overrides
must preserve each. The delivered client template carries this flag into
the NewTask record, without a template identity in the request, so that
every way of making a task from the template names its closer.
Tool-created and deployment tasks use Result explicitly; the tools take
no `ended_by` key.

### 3.3 A template's own resources

- **A template names the resources a task filled from it holds** as its
  connectors' symbols: each a connector's number and a symbol that
  connector offers (`../domain/connectors.md`, 3.1), such as a forge's
  *its own goal branch*. A symbol stands for a resource named after the
  task, which no creator can name before the task has its number.
- **In the format** (section 9), `resources` is optional: a list of
  `{"connector": c, "symbol": s}`, each `c` a connector's number and each
  `s` its symbol's, at most the tasks child's limit on holdings, no pair
  twice; left out, or `[]`, the template names none. jig's chat and goal
  name none.
- **Filled as any default:** the client copies the symbols into the batch
  member it fills, its `symbols`, which a member written by hand may name
  too (`../domain/tasks/holds.md`, 2). Served in `me` with the rest of the
  template, and kept by the settings with it; never overridden by a
  project's policy, since they are part of the shape.
- **A symbol grants nothing.** Its connector resolves it once the task has
  its number, into the name of the resource and the hold the task takes
  on it, or refuses the batch, naming the symbol: a symbol the connector
  does not offer, or one it offers only to tracked tasks named for an
  untracked one. What the task may then do to the resource is its
  authority's, as for any resource (`../domain/authority.md`, section 4).
- **Checked at start:** the list's shape and bound, and no pair twice.
  Whether a connector offers a symbol is the connector's to say, at the
  batch.

## 4. jig's part for a worker

| Member | What it gives |
|---|---|
| engine | the engine's address, and its TLS server name and trusted roots; or plaintext to a loopback address; or an in-process name, in one process (`channel/README.md`, 9.1; `service.md`, section 10) |
| name, secret | the worker's name, and the file holding its secret, a data path |
| slots | how many runs it hosts at once |
| graces | the grace after losing the engine, `grace_ms`; the drain's bound, `drain_ms`, how long a worker told to stop lets its runs end their turns before it cancels them (`../domain/hosts/hub.md`, 7), the one member that may be absent, 180,000 then (the user, 2026-11-12); the stop graces of an agent's tree, `agent_stop_ms`; a delivery's deadline, `delivery_ms`; a save's, `save_ms`: with the link's ping interval and silence deadline, what its stop bound is made of, max(ping + silence + grace, drain) + max(agent stop, delivery) + save (`channel/README.md`, 9.1; `../domain/hosts/hub.md`, 6) |
| agent | the agent program, which the worker's protocol layer spawns for each run, and which may be the application's own binary run as its `agent` command, smith's agent shell (`service.md`, section 6), with its arguments, which the worker passes as written: neither is a path of the document's; and its configuration, a configuration path, smith's agent configuration (smith's `protocol/agent.md`, section 4), which declares endpoint limits for the engine's charter models, by the same endpoint names as the engine's (`llm.md`, section 4). The worker's routing decodes charters against its endpoints, and its hello states its agent's bounds from it (`channel/README.md`, 4.3). A worker that composes `smith-inline-agent` gives it the same configuration, and runs no agent program |
| programs | `check_interval_ms`, a positive interval for the host's local login checks. The executable and private source-directory base are the agent configuration's `programs.claude_code.path` and `directory` (smith's `protocol/agent.md`, section 4), supplied with its explicit command environment below the worker root; no directory under a person's home is invented. Without `claude_code`, each source reports failed, no program configured. A model on an endpoint naming `program` takes a program source there, and a credential source on its provider's `direct` endpoint where the provider names one (section 3); one on an endpoint without it takes a credential source |
| limits | a profile, and overrides |

The worker's workspace's configuration is the application's. Workers are
alike: every worker's configuration is the same but for its name and
secret, and every credential its runs need comes with their assignments
(`channel/README.md`, section 7). A program's login is not configuration: its
holder signs it in on each host, in the programs directory, and the
host's half reports it (`llm.md`, 5.5).

## 5. Periods

- **The core keeps no calendar** (`../domain/authority.md`, section 7). A
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
  keeps the current period (`../domain/engine/README.md`, 5.4).
- **A project made after the first start** has its first period opened
  by the core, in the decision that makes it, its budget the policy's
  spend per period, with no next start (`../domain/people/acts.md`, 2);
  the root hears that it was made, and a source that keeps a calendar
  says its periods from then on.

## 6. Secrets

| Secret | Kind | Configured as |
|---|---|---|
| a worker's secret | shared between the engine and that worker | on the worker, a file holding it; on the engine, a file holding its digest, per worker |
| an LLM account's record or key | a signed-in source's record (its tokens, the client id the provider issued, the scopes granted, the ID token), or a key | never configured: kept in the secrets' store, from a sign-in or from a request carrying it (`store/secrets.md`; `llm.md`, 5.4). A registration is public, with no client secret; a confidential one is refused |
| the workers' listener's TLS key | a private key | a file beside its certificate |
| the clients' listener's TLS key; the secrets of the application's sign-in (an OAuth client's) | the application's server's | its own members' files; jig's part names none (`service.md`, section 7; `client/sign-in.md`) |
| the OpenID provider's signing key, and each of its confidential clients' secrets | the clients' component's (`client/openid.md`) | files `clients.provider` names (`configuration.md`, section 3), the key 0600 and the service's user's own; a key replaced is a file replaced, made anew on its host, which its key set's `kid` names anew |
| a connector's credentials | the application's | the application's files, or its own secrets' records |
| an application's credential, for runs' workspaces | the application's | its own member's source: a file, or an exchange it refreshes (section 3) |

- **Read at startup,** each within its bound, from a private file, with
  skein's `read_private_file` (skein's `runtime/shell.md`, 6.1): a file readable
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
  (`store/README.md`, 4.4), and the core seeds owners, the deployment's roles
  and projects from configuration into its first commit
  (`../domain/engine/README.md`, 5.4 and section 8). The admitted projects seed as a whole; fixed
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
  store's changes over them (`../domain/engine/settings.md`, 7): where the
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
  Nor is a template dropped: no task depends on one (3.1).

## 8. Open questions

- **Long-answer figures on the sign-in page** (`limits.md`): carrying
  skein's `LongRedirect { length, bound }` through the accounts domain
  and client wire to the application's page. Default: `SignInAgain`,
  with the specific long-answer page at the listener; the driver's
  terminal names both figures. Showing them on temper's page requires
  a client-wire change and regenerated journey recordings, left for a
  later request.
- **Approval beyond loopback** (3, clients): enabling it at a public
  name waits for the between-hosts flow (`client/README.md`, section 12).
  The optional `approval` boolean enables the loopback flow alone.

- **Reloading without a restart:** a deployment's limits changed while
  it runs. Its rules, charters, templates and policies change at
  runtime as settings (`../domain/engine/settings.md`).
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
    shell resolves at startup (skein's `runtime/shell.md`, section 6);
  - `{"in_process": "<name>"}`, an in-process name within skein's bound
    (skein's `runtime/io/README.md`, section 3), in one process only (`service.md`,
    section 10).
- **A path** is a string of bytes, of one of three kinds, which its
  member fixes: a *configuration path*, a *data path* or a *cache path*,
  after the application's three directories (skein's `runtime/shell.md`, 6.5).
  Configuration is what the operator writes and may copy or publish (the
  document itself, charters' instructions, trusted roots, the agent's
  configuration); data is what the deployment keeps, secrets among it
  (the deployment's directory, a worker's secret and the digests of
  workers' secrets, the workers' listener's certificate and key, the
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
    them from its arguments (skein's `runtime/shell.md`, 6.5). A caller that
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
- **A template,** in the `templates` member (3.1), is one JSON object
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
  null}`, each `n` the application's (3.1). jig's defaults are written,
  never implied: the `templates` member holds each template the engine
  has, jig's among them, and an application drops one of jig's by
  leaving it out (3.1).
- **A policy's override of a template,** in its `templates` list, is one
  JSON object: `template`, its number; and `executor`, `budget`,
  `deadline` and `priority`, each in the template's form (`priority` a
  number), or `null` where the template's own stands:
  `{"template": 2, "executor": null, "budget": null, "deadline": null,
  "priority": 5}`.
- **A charter's default wake policy,** its optional `wake` member (section
  3; `../domain/tasks/messages.md`, 3), is one JSON object whose members are all
  required: `words`, `notices` and `news`, each a rule, `"never"`,
  `"immediate"`, `{"batch": {"count": n, "age_ms": n}}` or `{"paced":
  {"span_ms": n}}`, its count, age and span above zero; `results`,
  `"never"`, `"each"` or `"last_or_failure"`; and `questions`, `answers`
  and `timers`, booleans. Left out, the charter's default is jig's: every
  rule `"immediate"`, `results` `"last_or_failure"`, the three `true`. A
  coordinator's: `{"words": "immediate", "notices": "immediate", "news":
  {"paced": {"span_ms": 1800000}}, "results": "each", "questions": true,
  "answers": true, "timers": true}`.
