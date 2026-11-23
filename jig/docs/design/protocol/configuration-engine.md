# Configuration: jig's part for an engine

## 1. jig's part for an engine

| Member | What it gives | Used by |
|---|---|---|
| deployment | the deployment's directory, a data path, which holds the store, transcripts and the secrets' store; the seed's source, for tests. Retention's horizons are engine limits, overridden by name (`store/retention.md`, 2) | the store (`store/README.md`); the shell |
| owners | the deployment's first owners, each a provider and a subject (`../domain/people/README.md`, section 3), each an owner of every project seeded with it and of the deployment's roles | the people child, on a first start |
| providers | the numbers and kinds that identities name (`../domain/people/README.md`, section 3), and nothing else: jig runs no provider's exchange. Of kind *link*, the printed link's; of kind *deployment*, exactly one, the deployment's own, for services' credentials; of kind *application*, each of the application's ways of signing in, whose identities its server verifies on its own routes and asks the clients' component to mint a credential for (`client/sign-in.md`). The settings and secrets of the application's sign-in are its own members | the people child; `jig-protocol-engine-clients`, which mints a printed link's credential for the first owner it names, a service's under the deployment's provider, and a `Mint`'s only for an identity whose provider is of kind application |
| rules | the deployment's rules, as the default of that setting when the application passes none (`../domain/engine/settings-store.md`, 1): the ceilings every project's policy is under (`../domain/authority.md`, section 6). A ceiling, the rules' or a project policy's, names in `models` the configured models it allows, from the llm member's, or every one without it; a role's authority names its models the same way: they seed the role on a first start, and the store keeps them by name (`store/records.md`, 1) | authority |
| projects | each project seeded on a first start: its number, from 1, since 0 is the deployment's roles' and refused here (`What::Bound` at the number; `../domain/people/README.md`, section 3); its name, which may be left out for "Project N", N its number, the seed's names unique and in shape or the start refused (`../domain/model.md`, 6.1); its policy, either inline, with its spend per period, its roles (each one's authority, the spend a party in it may allot per period, the kinds of proposal it decides; `../domain/authority.md`, section 6) and its overrides of templates' defaults (`configuration-templates.md`, 1), or as `policy: N`, the number of a policy among the settings, which the seed copies from the current settings as `MakeProject` copies one (`../domain/people/acts.md`, 2), so that an application's stock policy reaches its seeded projects with no copy of its own; its parties' roles. Its resources and homes come by adoption (below) | the core, on a first start |
| policies | each policy a project made after the first start starts from (`../domain/model.md`, 6.1), as defaults when the application passes none (`../domain/engine/settings-store.md`, 1), with the store's changes over them: numbered from 1 and never renumbered, as templates are (`configuration-templates.md`, 1); each a whole policy in a seed's format: its ceiling, its spend per period, its roles, its escalation role, its requirements, its projections' authority, its permissions' roles for adoption and its overrides of templates' defaults. Checked at every start as a seed's policy is: within the rules, its roles well formed, its overrides' templates configured, and its owner role able to be a root's requester (`../domain/people/acts.md`, 1); one that fails refuses the start, at its path. A project made from one keeps a copy, which no later change or removal here reaches (`../domain/people/acts.md`, 2). With none, no project is made after the first start | the core, at each `MakeProject` |
| charters | each charter the deployment offers, as defaults when the application passes none (`../domain/engine/settings-store.md`, 1), with the store's changes over them, numbered: its instructions (a file it names, a configuration path), its documents (`../domain/engine/settings.md`, 9: each a resource's name, a path and an optional heading, `{"connector": n, "name": [s, ...], "path": s, "heading": s}`, `heading` optional, and none when the member is absent), its models, named from the catalogue's (its main model, and those its sub-agents may use), and `rest`, whether every other model of the catalogue follows them (`../domain/engine/settings-catalogue.md`, 1; a charter with no `models` member takes them all), its tool families: smith's (inspect, modify, shell, agents) and the host families its runs are offered (`tools`, a `ToolFamilies` mask, offered where the task's authority holds them too: `../domain/engine/executors.md`, 1), the hosts that may run it, its waiting time, its default wake policy (`../domain/tasks/messages.md`, 3: `wake`, below), which a task made under it takes when its batch entry names none, a template's task among them (`configuration-templates.md`, 1), whether its runs resume, its parts (`parts`, below; none when the member is absent), its default budget (turns, spend and time, and a reserve of turns and time), which applies to a task whose batch gives none (`../domain/hosts/charters.md`). Its models are those it allows, in order; a task that names none takes the first whose LLM provider has a usable account serving its payer (`../domain/tasks/README.md`, section 3). It names no account: the core chooses one with each run's check (`../domain/accounts.md`, 4.3) | the core; `jig-protocol-charter` |
| templates | each template the application defines, as defaults when the application passes none (`../domain/engine/settings-store.md`, 1), with the store's changes over them, numbered and never renumbered, with its defaults; jig's chat and goal among them unless the application drops them (`configuration-templates.md`, 1) | the core, only to serve each project's merged templates in `me` (`../client/domain/contract/documents.md`, 1); the client, which fills them in |
| hosts | the engine's own slots, if it runs agents in the engine (shape 2), and their `smith-inline-agent`'s limits; no worker: `workers`, naming them, is refused, "workers are settings in the store" (`../domain/engine/workers.md`, 7); the workers' listener: its addresses, socket addresses or an in-process name (skein's `runtime/io/README.md`, section 3), and its certificate and key files, data paths, but for an in-process name (`channel/worker-link.md`, section 1); a listener on an in-process name tells the core its workers are in its process, and that no worker outlives it (`../domain/engine/restart.md`) | the fleet; `jig-protocol-engine-workers`; `smith-inline-agent`, through the application's startup |
| llm | the LLM providers (`../domain/providers.md`), each numbered, with its name for showing and the `endpoint` its added models take, and, where that endpoint names a program, an optional `direct` endpoint, naming none, that its sign-ins and keys take (below); its endpoints, each with an optional `program` named among its provider's `programs`; where the application passes no defaults, the models, each a default of the `model` setting (`../domain/engine/settings.md`, 1), numbered by position from 1, and each provider's `models`, naming them, both refused where it passes them: each model's window, output, the efforts it takes, low to high, or none (`efforts`), and the one of them a task given no LLM choice takes (`effort`, the first when left out; `../domain/tasks/README.md`, section 3), its name for showing (`shown`, optional), whether a picker shows it (`picker`, true when left out; `../client/domain/contract/documents.md`, 1), and the rest of what smith's agent configuration declares of it (smith's `protocol/agent.md`, section 4), its prices (input, cached input, output, and cache writes when priced apart), its prompt cache's life in milliseconds (`cache_life_ms`, optional, 0 or left out for a model with no prompt cache; `../domain/tasks/standing.md`, 5.3) and its optional `description`, what it is good at, for the brief's catalogue (`../domain/engine/briefs.md`, section 1); the sources it offers: a sign-in, with its public registration (its entry client id, its name hint, its authorization, token, revocation and published keys' addresses with their transport, its issuer, its scopes and those that must be granted, its resource, its redirect's path, and its port where fixed, an ephemeral loopback port for each sign-in otherwise), a key, a program, named `claude_code` where the program is Claude Code (smith's `protocol/programs.md`, section 3), or none; a sign-in waits for its redirect within the profile's sign-in deadline, as a limit of the driver (`limits.md`; skein's `oauth/README.md`, section 3); and the facts its terms ask an application to show: whether a signed-in source is a plan, and its manage-usage address (`../domain/providers.md`, 2.6). No account, key or record: accounts are made at runtime and kept in the store (`../domain/accounts.md`, 3.1; `llm.md`, sections 4 and 5). Its providers, each with the endpoints that name it, are the defaults of the `provider` setting and the providers a start runs, where the application passes no providers' defaults; one that passes them has the member refused (`../domain/engine/llm-providers.md`, 6) | the core; smith's and skein's components and skein's accounts driver, through the application's startup; `jig-protocol-accounts` |
| clients | the heartbeat of the clients' streams (`client/wire.md`, section 2); whether the clients' component gives a printed link's one-time secret at start, whose URL the application's startup prints (`client/sign-in.md`), only where its clients reach it on loopback, which that startup checks against its own listener (`service.md`, section 3); `approval`, a boolean, false when absent, whether it holds sign-ins approved by a signed-in browser (`client/sign-in.md`), apart from `link` and only where the listener is on loopback too, which the startup checks the same way; where the application gives a public name, a listener behind its TLS proxy, it enables asks by that name too, under their kinds' rules (`client/sign-in.md`, *A sign-in approved by a browser*), with no second switch; `provider`, the OpenID provider's (`client/openid.md`), or `null` for none: `key`, the file of its signing key, an RSA key of 3072 bits as PKCS#8 PEM (`-----BEGIN PRIVATE KEY-----`), made on the host that uses it by the application's setup (`openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:3072`), mode 0600 and owned by the service's own user, never copied, printed or logged; the start refuses, each with its reason, a key of another form or size, a file readable or writable by group or others, reached through a link, or owned by another than the process's effective user; `clients`, each its `id` (1 to 255 visible ASCII bytes, unique), the file of its `secret` or `null` for a public client (skein's `oauth/issuer.md`, section 2), and its one `redirect_uri` (`https`, or `http` on loopback with a port), none or more, an application adding its own while it runs (`client/openid.md`); and `addresses`, the providers, each of kind application or link, whose subjects are verified addresses, given as the claims' `email`, a link's being the deployment's own word on its one owner. A key skein's issuer refuses, a client off its rules, or a provider in `addresses` of another kind refuses the start at its path. The provider's codes and access tokens are counted by the limits `provider_codes` and `provider_tokens` (`limits.md`). The listener, its certificate and key, and what serves a browser are the application's service's own members (`service.md`, section 7) | `jig-protocol-engine-clients` |
| connectors | the application's connectors' numbers, as the core knows them (`../domain/model.md`, 3.5); none takes the number the core keeps for its own procedures (below) | the core |
| acts | the deployment's acts each connector may make in its owners' name (`../domain/people/acts.md`, 3), none without the member: a list of `{"connector", "may"}`, each `connector` one of `connectors`, at most once, and `may` the kinds it is granted, each at most once, among `close`, `add_key`, `remove_account`, `adopt`, `make_project` and `remove_project`. A connector not configured, or named twice, and a kind unknown, or named twice, refuse the start at their path. The validated entry count becomes people's `act_grants` memory bound, checked by its constructor; each entry charges at most one of every act kind. Every act is refused while `owners` names none | the people child, at every start |
| limits | optional: a profile, and any limit overridden by name (`limits.md`); absent, the application's base. An application whose limits are settings refuses it (`limits.md`, limits as settings) | every child and component of jig's |

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
work (`../domain/accounts.md`, 3.7; `../domain/engine/settings-catalogue.md`, 2). A name is kept
when a `keep` pattern matches it and no `drop` pattern does. Each is a list of at
most 16 patterns of 1 to 64 bytes of a model name's own bytes (`valid_name`,
`../domain/engine/settings-catalogue.md`, 1), `*` standing for any run of them, `?` for exactly one,
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
    root as each period begins (`configuration.md`, section 5), the first at the
    deployment's first start, before the core admits a party's request;
    the store keeps the current one across restarts
    (`../domain/engine/decisions.md`, 1.4), and the policy's spend per period
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

A copied seed is checked against the effective rules and authority limits,
its references, and the byte price of its complete policy snapshot before
any store opens. A refusal at the policy's delegation names its executor
count and `authority.executors` limit; a snapshot refusal names its bytes
and `core.policy_bytes` limit. Unknown references name their member.
