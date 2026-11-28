# Configuration: templates

## 1. Templates

- **A template is the application's recipe for a shape of work:** the
  defaults that turn "a chat with these words" or "a goal for this"
  into a complete `Create` (`../domain/tasks/README.md`, section 4). An
  application's own shapes (one for `ops`'s incidents, say) and jig's
  defaults are alike.
- **Numbered, never renumbered.** A number, above zero, is the
  template's identity across the configuration, projects' policies and
  clients; a template dropped leaves its number unused.
- **In two ranges,** as charters, policies and models are: the
  application's templates, the defaults, take 1 to 32, and a
  deployment's own, added by a change, 33 to 64; a start refuses
  anything else (`../domain/engine/settings.md`, 1). The document's
  charters, templates, policies and `llm` models are defaults when the
  application passes none (`../domain/engine/settings-store.md`, 1), so they
  take 1 to 32 too.
- **What each carries,** every field required (`configuration.md`, section 9):
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
    (`../domain/authority-budgets.md`, section 1); or *given*, the creator's,
    within their allotment;
  - the deadline: none; a span from creation, which the client sends
    as a span and the engine makes a wall when it makes the task
    (`../domain/authority-budgets.md`, section 1); or *given*, the creator's,
    or none;
  - tracked, with a default priority, or untracked;
  - the flags: *conversation*, *listed*, and the retention class,
    *kept* or *forgotten once read* (`../domain/tasks/README.md`, section 3);
  - the label: an opaque 32-bit number, which jig gives no meaning;
  - the grace its helpers give a close or a deletion, or none;
  - the LLM choice: none, for the first of the charter's models each run
    can use (`../domain/accounts-uses.md`, 1.3); or a main model
    with its effort and its speed, and the models its run's sub-agents may use, each
    with an effort and a speed (`../domain/tasks/README.md`, section 3), and optionally
    `later`, the same again for every run after its first activation's
    (`{"main": ..., "sub_agents": [...], "later": {"main": ...,
    "sub_agents": [...]}}`). It names no account.

  Its flags are checked against the executor it admits: a named
  charter's, as that charter's agent; a chosen executor's ended by
  `Standing`, or by `Requester` as a conversation, as an agent's; a
  chosen executor's ended by `Requester` and no conversation, as jig's
  gathering's, the only task of that shape, which its creator fills as
  the gathering's executor (`../domain/tasks/gatherings.md`, section 2);
  any other chosen executor's, as a person task's.

  It names no wake policy, parameters, inputs or dependencies, and
  resources only by its connectors' symbols (3): a task filled from it wakes by its charter's default wake
  policy, as the settings hold it when the task is made, and parks as
  its executor's defaults say (`../domain/tasks/messages.md`, 3), and its helper names none
  of the rest (`../client/domain/client/helpers.md`).
- **jig's defaults** are two, which an application keeps, changes or
  drops, by writing them in its `templates` member or leaving them out;
  none is implied (`configuration.md`, section 9). jig's helpers find them by number:
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
  standing for each it leaves `null` (`configuration.md`, section 9). Never another field,
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
  it declares none, and a speed the model declares in that charter's
  resolution, `standard` or none, and a `later` only under a charter whose runs don't
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
  account (`../domain/accounts-uses.md`, 1.3), in charter order,
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
  none; each speed one its model declares, by its own name, or
  `standard`, or left out; no `later` under a charter whose runs resume; each seeded
  policy's overrides naming configured templates, each at most once,
  and no more of them than the limit on templates.

## 2. Ending authority in templates

Templates optionally name `ended_by` as `result` (the default),
`requester`, `standing` (`../domain/tasks/standing.md`, section 2), or a
closer, `{"closer": {"connector": C, "code": N}}`, the
procedure that ends the task once nothing is left to await
(`../domain/tasks/lifecycle-endings.md`, 3), its connector by number and its code within
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

## 3. A template's own resources

- **A template names the resources a task filled from it holds** as its
  connectors' symbols: each a connector's number and a symbol that
  connector offers (`../domain/connectors.md`, 3.1), such as a forge's
  *its own goal branch*. A symbol stands for a resource named after the
  task, which no creator can name before the task has its number.
- **In the format** (`configuration.md`, section 9), `resources` is optional: a list of
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

## 4. Members an application fixes

An application may fix some of jig's members as its own constants, so
that its documents stop carrying values nobody changes (temper's
`protocol/configuration/moved.md`). It passes them to the parser, as it
passes its settings' defaults (`configuration.md`, section 1); jig's reference binary passes
none and reads every member as the tables above and `configuration.md`, section 4 give them.

- **What may be fixed,** each on its own: of the engine's part,
  `deployment` (its directory and its seed), `providers`, `connectors`,
  `acts`, `projects` (fixed as none: the first start seeds no project,
  and projects are made at runtime, `../domain/people/acts.md`, 2), and
  `clients`' `heartbeat_ms`, `link` and `approval` together; of the
  worker's part, `graces` and `programs` (its `check_interval_ms`).
- **A fixed member is refused** where a document names it, naming its
  path, as an unknown member is (`configuration.md`, section 1), with `What::Fixed`: the
  application's words say where it went. A part left with no member, the
  worker's `programs`, is refused whole; `clients` keeps `provider`, and
  is optional once its other members are fixed: absent, it has no
  provider.
- **A fixed value is checked as a document's would be,** and joins the
  start's checks across the whole (`configuration.md`, section 7) as if read: an owner's
  provider among the fixed providers, an act's connector among the fixed
  connectors, the printed link's loopback check. A fixed value that
  fails one is the application's fault, refused at start by the path its
  member would have, so that a test of the application's constants finds
  it.
- **The contract,** in `jig-protocol-config`: a value `Fixed` given to
  `engine` beside `Defaults`, each member an `Option`, `None` read from
  the document: `deployment: Option<Deployment>`, `providers:
  Option<Box<[IdentityProvider]>>`, `connectors: Option<Box<[u16]>>`,
  `acts: Option<Box<[ActGrant]>>`, `projects: Option<Box<[Project]>>`
  (`Some` of none alone), and `clients: Option<FixedClients>`, a
  `FixedClients { heartbeat, link, approval }` of the types `Clients`
  has them; and a value `FixedWorker` given to `worker`, `graces:
  Option<Graces>` and `check_interval: Option<Duration>` of the types
  `Worker` has them, the hub's `source_interval` following
  `check_interval` as it does when read. `Fixed::NONE` and
  `FixedWorker::NONE` fix nothing. The typed `Engine` and `Worker` are
  unchanged: a fixed member's field holds the fixed value.
