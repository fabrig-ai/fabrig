# Settings in the store

How work is run leaves the document for the store: the document keeps
what a start needs before it can read its store, and what belongs to
the machine, and everything else that configures work is a setting,
changed while temper runs and kept in the store, as projects already
are. The mechanism is jig's (jig's `domain/engine/settings.md`
; `jig/docs/design/domain/connectors.md`, section 17): a setting is a kind, a scope
and a number; the current settings are the binary's defaults with the
stored changes over them; a change is one decision, checked whole and
refused whole; charters are pinned per revision, everything else is
read at its newest. This section says what temper adds: its own kinds,
how the forge judges them, its stock defaults, and the members that go.
The pages a person meets it on are `../../client/pages/settings.md`, 2.

## 1. What leaves the document

| Member today | Becomes | Its default (`stock.md`) |
|---|---|---|
| Models in the deployment and agent files, refused under passed defaults | jig's models | temper's stock models, numbered (`stock.md`) |
| jig's `engine.charters` | jig's charters | temper's stock charters: a chat's (1), a coding agent's (2), a reviewer's (4), a coordinator's (5), a foreman's (6), the status agent's (9), a trial's (10) and Ask temper's (11) |
| jig's `engine.templates` | jig's templates | jig's chat (1) and goal (2), a goal for a draft (4), and temper's Ask temper (5) |
| jig's `engine.policies` | jig's policies projects are made from | the stock policy (1) |
| jig's `engine.rules` | jig's rules | temper's stock rules, the landing's two requirements among them (3) |
| `application.landing` | temper's kinds 1 and 2: the deployment's landing rules, and each project's (2) | each project's rule 1: CI passed, up to date and gate 1's agent review on every branch of each of its repositories; no deployment rule |
| `application.charter` | temper's kind 3: the forge's charter (2) | charter 2 |

- **What stays** is `engine.md`'s other members and jig's other parts:
  the directories, the listeners, the workers and their secrets' files,
  the limits, the first owners and the sign-in providers, `machine`,
  `clients`, `sign_in`, `webhooks` and `demo`, and, for now, the LLM
  providers and their endpoints (`engine.llm`), which are configuration of
  the work too, and leave the document by a design of their own, later. Of their
  members `temper-box init` gives each `keep` and `drop` (jig's
  `protocol/configuration.md`, section 3): OpenAI's keep `gpt-*` and drop `*audio*`,
  `*image*`, `*realtime*`, `*search*`, `*transcribe*`, `*tts*` and the dated
  `*-20??-??-??`; Anthropic's keep `claude-*` and drop the dated `*-20??????`. The
  playground's scripted provider has neither, and no list. jig's
  `engine.projects`,
  the projects a first start seeds, stays jig's member; `temper-box init`'s
  is `[]` (../setup/init.md, 3), and a document's seeded project names its
  policy as `policy: 1` (jig's `protocol/configuration.md`, section 3),
  so the seed copies the stock's policy 1 and no document carries a
  copy of it (5, step 1).
- **A project's own policy** is not a setting: it is in the store
  already, and its owners change it (jig's `domain/people/acts.md`, 2),
  part by part, or whole as an item of a change of settings (jig's
  `domain/engine/settings.md`, 2), so that one proposal sets a project up
  beside the deployment's settings: the charters and templates its
  work runs under, its landing rules, and its policy, with its roles,
  delegations, tasks and depth. Such a set is decided by one person
  who owns both the deployment and the project, in one decision; a
  restored version sets the policy back as it set the rest (jig's
  `domain/engine/settings.md`, 10). What a project adds to how its changes
  land is its landing rules, a setting of kind 2, decided by its
  owners.

## 2. temper's kinds

temper declares ten kinds of its own beside jig's, each owned by the
forge's connector, which judges them (jig's `domain/engine/settings.md`, 5):

| Kind | Scope | Numbers | Value | Decided by |
|---|---|---|---|---|
| 1, the deployment's landing rules | the deployment | a rule's, from 1; at most `forge.landing_rules` | a landing rule (below) | the deployment's owners |
| 2, a project's landing rules | each project | a rule's, from 1; at most `forge.landing_rules` per project | a landing rule | that project's owners |
| 3, the forge's charters | the deployment | 1, the change's; 2, its reviews'; 3, its foremen's | a charter's number, a u32 | the deployment's owners |
| 4, a project's landing | each project | 1 | how the project lands, by queue or by turn, and the turn's settings (`../../domain/landing.md`, 13.1) | that project's owners |
| 5, a repository's gates | each project | the repository's adoption, unique in the deployment: its forge's number shifted 16 bits left, with the forge's repository number below | a repository's test gates (`../../domain/landing.md`, 13.1) | that project's owners |
| 6, a project's questions | each project | 1 | the question route a change of the project takes when its maker names none: none, foreman or up (`../../domain/forge/change/questions.md`, 1) | that project's owners |
| 7, a project's ways | each project | 1 | what the project shows and the way a goal starts by, a `ProjectWays` (below) | that project's owners |
| 8, a project's classes | each project | 1 | what agents decide for a person and what reaches them, a `ProjectClasses` (below; `../../domain/forge/attention.md`, 6) | that project's owners |
| 9, a charter's words | the deployment | a charter's, 1 to `tasks.charters` | what the charter does, in a line, a `CharterWords` (below) | the deployment's owners |
| 10, a project's words | each project | 1 | How work runs' headline and lede for the project's own way, a `ProjectWords` (below) | that project's owners |

- **A project's ways' value** is a `ProjectWays` record in temper's
  codec, read by the client alone (`../../client/pages/settings.md`, 2):
  `drafts`, `goals` and `changes`, each a bool, whether chats
  offer drafts and whether the rail offers Goals and Changes; the Away
  switch shows by a project's classes (kind 8; `../../client/pages/README.md`
  ). An Ask temper chat offers a draft regardless, and Goals
  stays shown for a current Goals or goal page or a project holding a goal
  (`../../client/pages/README.md` and `../../client/view.md`, 3). The `way` is a template's number,
  the way a goal starts by, 0 for the first the project offers with
  `tracked` set. Its bound, the
  kind's, is its encoding; the engine decides nothing by it. A value of
  a project's questions is its route alone, a `ProjectQuestions` record.
- **A project's classes' value** is a `ProjectClasses` record in
  temper's codec, the table of `../../domain/forge/attention.md`, 6, member for
  member: `items`, at most 20 `ClassItem { subject, words, class }`, its
  `subject` a `ClassSubject`, `kind` with a `ReachKind` or `set` with a
  set's number, its `words` at most 120 bytes, its `class` a
  `ReachClass`; and `sets`, at most 6 `PathSet { number, name, patterns
  }`, its name at most 40 bytes and at most 6 patterns of 80. `ReachKind`
  names the ten kinds and `ReachClass` the four classes, `decide`, `ask`,
  `ask_away` and `wait`, in 5.10's order. A judgement of the engine's
  reads it: the forge's `decided` (`../../domain/forge/attention.md`, 5) and no
  other. Its bound, the kind's, is the encoding of a table full at every
  bound. A project with no value has Agents review's table
  (`../../domain/forge/attention.md`, 6). The same codec's `Reach`, `{ kind, class,
  reason, under, sets, matched, away }` with `reason` a `ReachReason`,
  `kind`, `set` with a set's number, or `always`, is 5.10's reach.
- **A setting's words,** kinds 9 and 10: words a person or Ask temper
  writes with a setting, kept beside it, which the pages that draw the
  setting say (`../../draft/whole-story.execution.md`, P3e). Numbered past
  kind 8, which a project's classes take (`../../draft/what-reaches-you.md`,
  section 2). The engine decides nothing by them.
  - *A charter's words,* kind 9, a `CharterWords` record in temper's
    codec, its `line`: what the charter's runs do, in a line of the
    person's terms ("reviews each change at its head, and holds it until
    it approves"), the line How work runs' Roles gives the role beside
    when it runs (`../../draft/surfaces.md`, section 7). Its number is the
    charter's. A charter with none is said by its instructions' first
    sentence there; words of a number the settings offer no charter at are
    kept and said nowhere. The stock's are `stock.md`'s.
  - *A project's words,* kind 10, a `ProjectWords` record, its `headline`
    and its `lede`: How work runs' heading and lede while the project runs
    no stock way (`../../client/pages/settings.md`, 2), "Agents review everything,
    then one of you signs off" over a sentence or two of how its changes
    go. A project that runs a stock way is said by that way's words
    (8), whatever its own; the stock has none.
  - *Their bounds:* each UTF-8 holding no control character, a `line` and
    a `headline` of 1 to 120 bytes, a `lede` of 1 to 480. A kind's bound
    is its record's encoding at them.
- **A landing rule's value** is a `LandingRule` record in temper's
  codec, `engine.md`'s rule member for member but `connector` and `kind`,
  which are always the forge's and its merge: `pattern`, `enforce`,
  `ci`, `up_to_date`, `gates` and `approvals`, each as `engine.md` says.
  Its largest bytes, the kind's bound, are its encoding with a pattern
  at jig's pattern bounds, `forge.change_policy.gates` gates and
  `authority.roles` approvals; `forge.landing_rules`, a limit of the
  forge's (`limits.md`), is 4 in `small`, as jig's requirements per scope
  bound the rules today.
- **A project's rule may name a branch** (`../../draft/stock-ways.md`,
  3.2): a pattern of kind 2 may begin at `branch` in place of `forge`,
  the host, the owner and the name, `{"segments": ["branch"], "last":
  "exact", "value": "main"}` for `main`, or `"last": "open", "value": ""`
  for every branch, and covers that branch in each repository of the
  project, those adopted after the rule among them. It is judged against
  a merge's landing branch past its repository's segments, the forge's
  resource path but its first four (`engine.md`), and a project's rules are
  judged for its own merges alone (3), so that a project's way is data
  naming no repository. A pattern of kind 2 beginning at `forge` keeps
  its meaning; one of kind 1 begins at `forge` alone.
- **Judged by the forge, in the change's decision.** The root hands the
  forge each item of its kinds with the value it would leave (jig's
  `domain/connectors.md`, section 17); the forge reads only its own
  state, and refuses an item, naming it by kind, scope and number and
  saying what fails:
  - *a rule* that does not decode; whose pattern's first segment is not
    `forge`, or, of kind 2, `branch`; whose pattern passes jig's pattern
    bounds; with more gates than
    `forge.change_policy.gates`, or a gate's number twice; with an
    approval of no people, or a role twice, or one past
    `authority.roles`;
  - *the forge's charter* outside 1 to jig's `tasks.charters`, or
    removed: the forge always has all three charters, and an owner who
    wants the stock's back resets it;
  - *a project's questions* that is not one of the three routes;
  - *a project's classes* that does not decode as a `ProjectClasses`,
    or that `Classes::check` refuses (`../../domain/forge/attention.md`, 6), the
    refusal's words saying which of its faults;
  - *words* of kind 9 or 10 that are empty, past their bound, not UTF-8
    or holding a control character, or of kind 9 at a number outside 1
    to jig's `tasks.charters`; a removed or reset item of either is
    always good;
  - *a project's ways* with a `way` neither 0 nor a template the project
    would offer after the whole decision, read by the root from the core
    through its active settings decision as an approval's eligible people
    are read from its people; a removed or reset item of kind 6, 7 or 8 is always good,
    the project going back to the stock (`stock.md`). A proposal may install a
    template and select it as the project's way in the same acceptance.
    A refusal names the item and the rejected way number;
  - *any item* whose value is not its kind's (a rule for
    kind 1 or 2, a charter's number for kind 3, a `CharterWords` for kind
    9, a `ProjectWords` for kind 10), or whose kind, scope or number
    the table above does not have.
  An item that leaves its setting as it is (a value equal to the
  stored change's, a removal of a setting already removed, a reset of
  one with no change) the forge answers as the same, which jig records
  as no change of it (jig's `domain/connectors.md`, section 17).
  The root also checks a set rule's enforcing approvals against the
  core's project memberships and known forge user ids, including at
  proposal acceptance (`../../domain/forge/change/gates.md`, 3). For each affected
  project and each forge of the repositories the rule covers, a rule of
  kind 2 beginning at `branch` covering each repository of its project,
  too few
  people eligible on that forge refuses the whole change, naming project,
  forge, role, required and eligible counts with the remedy; a rule
  covering the playground's fake forge is refused in its own words. An
  asking charter's proposal is judged so at its call too, for its chat's
  project alone (`../../domain/agent/ask-temper.md`, 2). A
  deployment login or display name establishes no forge identity; a box's
  person linked on a forge is known there (`../../domain/forge/adoption.md`, 6).
  Later membership changes can leave an accepted rule waiting for
  eligible approvers. Kind 3 declares
  that its value names one charter (jig's `domain/engine/settings.md`, 5): the
  forge says which when it judges or reports it, and jig's check of
  references refuses a change that would leave the forge's charter
  unoffered, naming kind 3, as it refuses one that would leave a
  template's.
- **Kept by the forge,** in its family `Setting` (`../store.md`, section
  2), written in jig's decisions (jig's `domain/engine/settings.md`, 5): the
  stock defaults of its kinds the binary gave it at the last start, each
  setting's current change, the items of pending proposals of settings
  that name its kinds, and, beside each change it saves, the value the
  setting held before, by the revision that made the change, for as long
  as jig's history keeps that revision's record, so that a restore
  proposes it again (jig's `domain/engine/settings.md`, 10). jig keeps only
  their states and sizes. Once a change is committed, the forge saves
  its items and replaces its rules, the deployment's or the project's,
  whole; each live change into a repository it serves is stepped after
  the commit, so a gate a rule added is asked for and one it dropped no
  longer waited on (`../../domain/forge/change/gates.md`, 1). At a start the forge is given the stock
  defaults, tells jig which changed, and holds the current values
  before its other records (`../../domain/forge/README.md`, section 9).
- **Erased with their project.** A project deleted (jig's
  `RemoveProject`, jig's `domain/people/acts.md`, 2) has its settings
  erased by the connector owning each kind: the forge erases its kinds
  2, 4, 5, 6, 7, 8 and 10 of that project's scope, their current changes and the
  items of pending proposals among them, in the deletion's steps, and
  the deployment's kinds 1, 3 and 9 stay. A project made later, its number
  never the deleted one's, starts from the stock defaults.
- **Read from the forge.** A client reads a landing rule's value, the
  forge's charter, a charter's words, a project's questions, ways and
  words, and a pending
  proposal's items of temper's kinds through the forge's family
  (`../../client/forge/served.md`); jig's documents give their states
  and sizes alone. A read asks in a scope: the deployment's, by a party
  with a role in the deployment, for any kind; or a project's, by a party
  with a role in that project, for a kind of each project's scope (2, 6,
  7, 8 and 10) in that scope alone. How work runs reads a project's words in
  its `ways` read (8), which any party of the project reads.
- **Never pinned.** Every merge is judged by the landing rules the
  forge holds then, the deployment's and its project's, whatever
  revision its change was made in: what work may do is always current,
  as a task's authority from jig's rules and its project's policy is
  (jig's `domain/engine/settings.md`, 4), and only what a run is given, its
  charter, keeps the revision its task was made in. A change of the
  rules reaches a change already in flight at its next merge; a merge
  already made stays as it was decided.
- **The forge's charters are read when a change is made.** The forge
  reads its own kind 3, as it holds it, and the root grants the
  change's procedure delegation to its producer's charter, its
  review charter and, when requested, its foreman's charter. Its producer's charter is the one its `change` call
  names (`../../domain/agent/tools.md`, 1), or, with none, the change's
  charter (number 1), its default. The change keeps their numbers on
  its row: its producing, its repairs, its resolutions and its checks'
  gates run under the producer's charter it was made with, its agents'
  gates, the reviews, under the
  review charter it was made with (number 2; the stock's charter 4), its
  foremen under the foreman's charter it was made with (number 3; the
  stock's charter 6), and a newer charter of any reaches the
  changes made after it (`../../domain/root.md`, section 5). A change whose
  questions take no foreman is granted no delegation to the foreman's.
- **A project's questions are read when a change is made,** as its
  charters are: a `change` that names no route takes its project's
  kind 6 (`none` on the stock), kept on the change's row as its
  questions parameter (`../../domain/forge/change/README.md`, 1), and a later change
  of the setting reaches only the changes made after it.
- **A goal template's work, kind 11** (`../../draft/stock-ways.md`, section
  4; `../../draft/whole-story.execution.md`, P3f): whether the coordinator
  of a goal started from a template designs first and whether it makes a
  close-out, said by a setting of the template rather than guessed from
  its charter's instructions, so that a goal's path draws "design" or
  "plan", and a "close-out" where one is made (`../../domain/forge/change/branch-path.md`
  ). A kind of temper's beside the table's, owned by the forge's
  connector and kept, erased, read and never pinned as they are;
  numbered past kind 8, a project's classes
  (`../../draft/what-reaches-you.md`, section 2), and kinds 9 and 10, a
  setting's words (`../../draft/whole-story.execution.md`, P3e):
  - *its scope and numbers:* the deployment's, as templates are (jig's
    `protocol/configuration.md`, 3.1); its number a template's, 1 to
    jig's `core.templates`; decided by the deployment's owners;
  - *its value,* a `TemplateGoal` record in temper's codec: `design`, a
    bool, whether the template's charter designs the goal first,
    delivering the design into the goal's branch as a change the goal's
    other changes start from and merge after; and `close_out`, a bool,
    whether it makes a close-out once the goal has landed, one change
    into the landing branch bringing what the goal leaves up to date: its
    status files, its designs' Later sections and its draft's fate. Its
    bound, the kind's, is its encoding. The engine decides nothing by it:
    the charter's instructions say what its runs do, and the setting says
    it for the pages, so that the two agree is the owner's, as a
    charter's words are;
  - *with none,* the template's coordinator plans and makes no
    close-out, the value `{design: false, close_out: false}`; a removal
    or a reset where the stock gives none leaves it so. A value at a
    number the settings offer no template at, or at a template no goal
    starts by, is kept and drawn nowhere;
  - *judged by the forge,* as the table's kinds are: at scope 0 alone,
    else refused for its scope; a number 0 or past `core.templates`,
    for its number; a value not a `TemplateGoal`, for its value; a
    removal or a reset always good;
  - *named* `template-goal`, with the template's `number`, in a settings
    file and a run's `settings`, `see` and `propose_settings`
    (`../../client/ctl/tools.md`, 3; `../../domain/agent/agents.md`, 1), its value
    `{"design": BOOL, "close_out": BOOL}`, both members required; read
    by a party with a role in the deployment, as kinds 1 and 3 are;
  - *the stock's,* `stock.md`'s: template 4's, neither; the reference's,
    `reference.md`'s: template 4's, design alone, since its goals close by temper's
    closer (`../../domain/forge/branches.md`, 2), whose close step settles what a
    close-out wrote. A template's closer is no part of this kind: it is
    the template's `ended_by` (jig's `protocol/configuration.md`, 3.2),
    which every way of making a goal from the template carries, while
    this kind is read by no way that makes a task.
- **A project's follows, kind 12** (`../../domain/forge/follows.md`,
  section 2): each a follow of a branch of a repository the project
  adopts as context, into a repository it owns. A kind of temper's beside
  the table's, owned by the forge's connector and kept, erased, read and
  never pinned as they are:
  - *its scope and numbers:* each project's; its number a follow's, 1 to
    `forge.follows`; decided by that project's owners;
  - *its value,* a `Follow` record in temper's codec, its members
    `follows.md`'s table; its bound, the kind's, is its encoding with
    every list at its limit;
  - *judged* by the forge's structure first, then by the root across
    domains, each refusal `follows.md`, section 2 names; a removal or a
    reset always good, and either cancels the follow's task;
  - *named* `follow`, with the `project` and its `number`, in a settings
    file and a run's `settings`, `see` and `propose_settings`, its value
    `follows.md`'s settings-file form; read by a party with a role in the
    project, as kinds 6 to 8 are;
  - *the stock's:* none.

## 3. How the landing rules are enforced

- **Two standing requirements** in temper's stock rules, on the forge's
  merge (kind 4) over `{"segments": ["forge"], "last": "open", "value":
  ""}`, every landing branch of every forge: the forge's requirement 5,
  *the landing rules*, guarded and marked as one that must be guarded;
  and its requirement 6, *up to date by the landing rules*, observed
  with a freshness of 0. Their parameters are 0. They replace the
  requirements 1 to 4 that each rule's criteria became, one per
  criterion, in the rules and in each seeded project's policy
  (`../../domain/forge/change/gates.md`, 2).
- **A rule that does not enforce is advice:** its gates, blocking or
  not, are asked for and their verdicts shown on the change's panel,
  none holding the merge, and its approvals are not asked; only rules
  that enforce hold a merge, below. Built with 5's second step, when
  the forge holds the rules itself.
- **Judged over the rules the forge holds then.** At a merge, the forge
  takes every rule covering its branch that enforces, the deployment's
  and the project's, a project's rule beginning at `branch` covering the
  branch of that name in each of the project's repositories (2), and
  its rule 1 the stock's while its owners have set none (`stock.md`):
  requirement 5 meets when each one's CI, gates and
  approvals are met, waits when one waits and none refuses, and refuses
  when one refuses; requirement 6 likewise for each one's up to date. A
  branch no enforcing rule covers meets both. The forge keeps on the
  change the rule and the criterion it last waited on or refused, which
  the change's panel shows and its hold says (`../../domain/forge/change/gates.md`,
  2): the words are the forge's, kept with its change, whatever a
  verdict carries through jig.
- **So a change of landing rules touches nothing of jig's:** no rule,
  no project's policy. A project is covered from its making by its own
  rule 1, the stock's (`stock.md`), and by the rules a deployment's owners
  add, which every project's merges meet beside its own; a project's
  rules changed reach its next merge. So a project's owners may land
  with less than an agent's review in their own project, and a
  deployment that wants a floor sets one in kind 1
  (`../../draft/stock-ways.md`, 3.1).
- **jig's rules are the deployment owners' to change,** the two
  requirements among them. A change that removes one leaves landing to
  the change procedure's waits alone; the review page says so under
  "temper enforces" (`../../client/pages/settings.md`, 2).

## 4. The members gone

- **Refused.** temper passes its defaults, so `jig-protocol-config`
  reads no `engine.charters`, `templates`, `policies` or `rules` (jig's
  bridge, `jig/docs/design/domain/engine/settings.md`, 7, serves an application that passes
  none), and temper reads no `application.landing` or `charter`. A
  document naming one is refused as any unknown member is (`README.md`, section 1),
  naming its path and saying where it went: "`engine.charters`:
  settings live in the store; read them with `temper ctl settings show`".
  `engine.md`'s `landing` and `charter`, and `Config`'s `landing` and
  `forge_charter` fields, go with them, in 5's second step.
- **A seeded project names its policy by number.** jig's
  `engine.projects` stays, but temper refuses a project whose policy is
  inline, naming its path: "`engine.projects[0].policy`: name a policy,
  `policy: 1`; change it with `temper ctl settings apply`". A policy is
  then written in one form alone, the settings file's
  (`../../client/ctl/tools.md`, 3), and the document's older forms of an
  authority, a pattern and a role's permissions (milliseconds, text
  segments) meet nothing.
- **Before the baseline** (`../../draft/upgrades.md`, 4.1): the settings change the store's families, so a
  deployment is made again, and nothing is imported. A document `temper-box
  init` wrote carried the stock values, now the binary's; one an
  operator edited is told again, after the start, as a proposal ("Tell
  temper how you work").
- **From the baseline,** were the members to go after it
  (`../../draft/upgrades.md`, section 10): the release that removes them
  accepts each for one release with a warning naming its setting, and
  at that start takes its values as stored changes where the store has
  none, in a revision recorded as the release's and as from the
  document, so nothing configured is lost; the next release refuses it.
- **Across jig and temper:** jig reads the four members for an
  application that passes no defaults (its bridge, which stays jig's),
  and refuses them for one that passes its own. temper passes its own
  from 5's first step, which needs nothing new of jig's settings;
  its second needs jig's application settings.
- **Their writers name none of them:** `temper-box init`'s document and
  files (`../setup/init.md`, 2 and 3); the box, which has no document of
  its own but `temper-box init`'s (`../setup/box.md`);
  `docs/examples/owned.json` and its instructions; the cluster
  fixture's engine documents, whose stories apply their own values as a
  settings file through ctl after sign-in; the fake backends' document
  rewriter (`temper-fake`'s backends); the tests' documents and
  builders (`../../testing/checks.md`, 4). A generated document
  elsewhere, fabrig's, drops the six members; what it set differently
  from the stock values becomes a proposal after the deployment's first
  start.

## 5. In steps

The members leave in three steps, each landing whole, so that temper's
side never waits on the part of jig it does not use:

1. **jig's four kinds.** The `stock` module passes jig's builder
   charters 1 and 2, templates 1 and 2, policy 1 and the rules as
   `Config`'s defaults (`stock.md`), the rules delegating to charters 1 and 2
   and the change procedure until step 3's charter comes, and the document's
   `engine.charters`, `templates`, `policies` and `rules` are refused
   (4). The landing rules stay `application.landing` until the
   second step, so policy 1 and the rules keep the requirements 1 to 4
   each rule's criteria become today (`../../domain/forge/change/gates.md`, 2),
   derived at each start from the document as now: a document whose
   landing rules change between starts gives new defaults, which make
   a revision recorded as the release's (jig's `domain/engine/settings.md`,
   7). Until step 2, the rules' default carries those requirements,
   so once the rules hold a stored change, a landing rule edited in the
   document reaches no decision until the rules are reset or changed
   again; step 2 ends that, the rules' standing requirements reading
   the forge's rules (3). The document's checks that read the four members (`engine.md`:
   `application.charter` among the charters, a landing rule's
   approvals' roles in the policies) read the stock defaults instead,
   which the binary knows before the store is read. A change that
   leaves the forge's charter unoffered (dropping the rules' delegation
   to it too, which jig's references require) is accepted in this step,
   and the forge's changes are refused until it is offered again: step
   2 refuses such a change, kind 3 naming its charter (2). The
   forge's review charter is the stock's charter 4 until then, fixed by
   the binary, with no document member of its own: its settings change
   as any charter's, and step 2 makes it kind 3's number 2. `temper-box
   init`'s document
   loses the four members and its `charters/`, and every other writer
   of them (4) moves with it. A seeded project names `policy: 1`
   in place of an inline policy, so the stock reaches its first chat;
   the interim inline copy with tools 513, and the test that holds it
   equal to the stock, go with it. A document may seed no project at
   all, its first owners the deployment's alone, as jig's seed allows,
   holding no escalation role until a project's policy gives them one;
   `temper-box init`'s seeds none, its projects made at runtime on policy 1
   (../setup/init.md, 3); a world or a person that needs a wider project
   raises the rules and policy 1 with one change of settings, then makes
   it, or widens an existing project's ceiling within the new rules
   (jig's `domain/authority.md`, section 6). A deployment whose document carried
   the stock values starts on the same settings, and makes no
   revision.
2. **temper's kinds,** on jig's application settings (jig's
   `domain/engine/settings.md`, 5): kinds 1 to 3 (2), judged by the
   forge; `application.landing`'s rules and `charter` refused; 3's two
   standing requirements in the rules, and policy 1 without
   requirements, as `stock.md` has them. `application.landing` keeps its
   projects' ways, checks and tiers (`engine.md`) until kinds 4 and 5;
   the document's checks that read a landing rule's approvals' roles go
   with the rules, the forge judging a rule as 2 says. Its writers
   (4) move with it: their landing rules become stock or a world's
   defaults, or a settings file applied after the start.
3. **Ask temper:** charter 11 and template 5 (`stock.md`), with the run's
   asking tools, `see`, `document`, `remember` and `propose_settings`' asking form
   (`../../domain/agent/tools.md` and `../../domain/agent/ask-temper.md`), and the rules' and policy 1's
   delegation to charter 11. A deployment made before takes them as new
   defaults, by a revision recorded as the release's; a project made
   before keeps its own policy, a copy (jig's `domain/engine/settings.md`, 2),
   and gains charter 11 by a proposal its owner accepts, or is made
   again, as nothing before v0.1.0 is kept compatible.

## 6. A charter's documents

A rule a person wrote down in a repository, and that only changes how
an agent behaves, reaches the runs that should follow it from where it
was written, by jig's charter documents (jig's `domain/engine/settings.md`,
9): a chat has no workspace and reads no repository's `AGENTS.md`,
so its charter names the documents, and a rule that lands reaches its
next run.

- **A document** is a branch of an adopted repository, by its resource
  name (`../../domain/forge/README.md`, section 2: `forge`, the host, the owner,
  the repository, `branch` and the branch's segments), a path, and
  optionally a heading. temper's stock charters name none.
- **The forge gathers each** for a run of the charter, as a brief
  section of its own (`../../domain/root.md`, 6.2): the file at the
  branch's head as the forge resolves it when it gathers the section,
  one commit named in the section and kept for its whole life, read as
  `read_file` reads one (`../../domain/forge/README.md`, section 2). With a
  heading, the part from the first Markdown heading line whose text,
  after its `#`s and one space, equals the heading byte for byte, to
  the line before the next heading of its level or above. Cut to the
  section's allotment at the last line end within it, saying how many
  bytes were left out, as the brief's other cuts do. Its budget is
  `root.brief.budgets.document`, beside `ci`, `reviews` and `pull`
  (`limits.md`), 4,096 bytes in `small`, bounding each document section
  the forge prepares. The repository is the one adopted, in any
  project, under the name's host, owner and repository: the charter's
  owners chose its documents, and no authority is asked of the run
  (jig's `domain/engine/settings.md`, 9). A repository not
  adopted, a branch, file or heading not there, or a read that failed,
  is a section saying so.
- **In a settings file** (`../../client/ctl/tools.md`, 3), a charter's value
  carries `"documents": [{"forge": F, "repository": "owner/name",
  "branch": B, "path": P, "heading": H}]`, `heading` optional, `F` the
  forge's host as its resources name it, which ctl turns into the
  branch's resource name, `forge F owner name branch` and B's
  segments, and back.
- **Ask temper** (`../../domain/agent/agents.md`, 1) proposes a charter that
  names a document, where the person keeps a rule in an adopted
  repository, in place of copying it into the charter's instructions,
  the document's repository named `OWNER/NAME` and its forge written by
  `propose_settings` (`../../domain/agent/ask-temper.md`, 2); a change landed
  there reaches the next run.

A document section begins `At BRANCH's head COMMIT`, then the selected text
on the next line. A cut appends `[N bytes cut]`, or `[more than N bytes
cut]` when the file read itself was truncated; a later cut renders the
retained text again without reading another head. Missing resources read
`Repository OWNER/NAME on HOST is not adopted.`, `No branch BRANCH in
OWNER/NAME.`, `No file PATH in OWNER/NAME at BRANCH's head COMMIT.` (a
folder too), or `No heading "HEADING" in PATH at BRANCH's head COMMIT.`. A
failed or refused read is a failed brief read, rather than one of these
successful explanatory reads (2026-11-04).

A stored document whose resource name is no forge branch renders with
`"connector"` and `"name": [segments]`, its `path` and optional `heading`,
preserving what was stored. A settings file naming a document in that raw
form is refused as out of shape; the accepted input form is the forge,
repository and branch form above (2026-11-04).

## 7. A model with a larger output

`temper-box init`'s limits hold the stock's models, each a window of 1,048,576
tokens and an output of 8,192 (`stock.md`), as its agent's endpoints declare them. A
deployment whose model takes a larger output (gpt-6.1-sol at effort high, output
32,000, for one) raises its agent's endpoint and overrides its profile's turn limits, each derived
from the agent's largest turn, T, which the start checks name "the
agent's largest turn" (`temper-service`'s `startup/checks.rs`).

- **T,** the configured turn ceiling, must hold smith's largest measured
  turn (`smith-agent-service`'s `profile.rs`). That measurement uses the
  smallest shared room over every endpoint, so one set of bounds
  holds whichever model a charter names (smith's `protocol/limits.md`, 3.9).
  `temper-box init`'s agent takes 48 calls per response, 26
  conversations (`stock.md`'s table) and 64 models a charter, its endpoints an
  output of 8,192 and a `reasoning_item` of at most 65,536, and efforts
  move no figure of T, smith charging an effort's name at the charter
  codec's fixed bound; T is what smith measures for it,
  within smith's own ceiling of 33,554,432 bytes (smith's
  `protocol/limits.md`, 4, `turn-in-channel`), and init's document
  holds it, written by the test's writer (`../setup/init.md`, 1). A box
  without the demo is measured from its own endpoints and must declare
  every carrier at least as large as that measurement.
  `docs/examples/owned/agent.json` takes init's figures but its
  endpoints and programs, with its own T in `docs/examples/owned.json`.
  Init keeps the demo, its window `gpt-6-luna`'s, so the host part is
  that window's (`stock.md`; `../setup/init.md`, 3).
- **The store's ceiling:** jig's `standard` page holds a turn's frame of
  a transcript of up to 33,554,432 bytes, any turn smith admits, and the
  store's frame and the worker wire carry it (jig's
  `protocol/store/transcripts.md`, 1); small's page holds a loaded transcript of
  at most 2,168,915 bytes (its 2,168,979 less the frame's 64), so a
  deployment on `small` keeps T within that. The page ceiling admits
  the largest shipped ordinary transcript (jig's
  `protocol/limits.md`). A deployment with larger output or
  reasoning items derives T from its own agent and checks that its full
  frame fits the selected profile's page.
- **The limits T sets,** "both hubs" being the engine's and each
  worker's; init's values are its agent's T's, in its document:

| Limit | Relation |
|---|---|
| both `hub.turn_bytes` | T |
| the engine's `core.resume_bytes` | the larger of T + 4 and 4,194,304 (4 MiB), so that a coding run's whole window resumes after an interruption, not one turn's worth (`../../domain/worker.md`, section 2) |
| both `hub.transcript_bytes` | resume + 12 × ⌊resume / 4⌋ |
| both `hub.turn_queue_bytes` | 8 × T |
| the engine's and each worker's `io.output` | the same room, the greatest worker room derived from a turn of T's frame and the workers' configured rooms |
| `root.journal.transcript_bytes` | resume |
| `root.loads.transcript_bytes` | T |
| `root.loads.bytes`, `root.loads.reply_bytes` | T + a temper record's size |
| `root.loads.read_bytes` | init: 2 × T; holds T and a row and a record |

- **One call's payload:** skein bounds a call's decoded input by the
  completion's answer, eight bytes a token of the model's output, at
  six bytes a payload byte with its framing, so at init's output of
  8,192 a call carries at most about 10,662 bytes; the agent's
  `tool_payload` is the largest smith admits at init's models, and
  `checkout.message_bytes` equals it, so the checkout accepts the
  agent's largest call, and the engine's message and result bounds are
  the payload less two (`stock.md`). A deployment whose models' output is
  larger raises them with its T, up to 262,144, smith's own bound on a
  host's input and answer (smith's `protocol/limits.md`, 2.4). A note's body is not bound by it: a
  run builds a body past one call in parts, by line ranges, up to
  `notes.body_bytes` (jig's `domain/engine/briefs.md`, section 2;
  `../../domain/agent/agents.md`, 3). `propose_settings` can name that note and
  its revision instead of inline items: the whole settings file is resolved
  internally, checked for the proposing task's note access and matching
  revision, parsed and proposed as one value (`../../domain/agent/agents.md`, 1).
  One owner acceptance makes one revision, without increasing a model's
  output or a deployment budget. A decided call replays its saved proposal
  before resolving a note again.
- **Memory:** `engine.memory_bytes` is the engine's byte budget
  (`../../domain/root.md`, 4.1), independent of a model's edge bounds.
  The start checks those bounds, without comparing memory with a sum
  or naming a deficit. A pipe the engine cannot adopt is refused
  naming what it lacks, never a bare `fd N refused`.
  `docs/examples/owned.json`'s declarations are the user's budget for
  the owned deployment and do not move with a box's.
- **The budgets** follow the model's C (`stock.md`), so a larger output gives
  larger spends with no setting changed: output 32,000 gives C 1,081.
- **The reference's room** (`../../draft/goal-as-designer.md`, section 6):
  room for a whole spec in a `change`'s words. Its
  agent declares `tool_payload` 63,740 and, on every endpoint, an
  `output` of 48,000, and its charters' models take the same `output`
  (`reference.md`), the draft's fallback of about 64 KB. skein's rule between them
  is (payload + 260) × 6 ≤ output × 8. `output` isn't sent to the providers
  it uses, so it changes no model's behaviour.
  `docs/examples/reference/room.json` holds this pair. Smith refuses
  131,072 and 98,500 at `turn-in-transcript`: `session.said_text`,
  1,245,440 bytes, passes `transcript.said_text`'s 1,048,576-byte ceiling.
  The fallback's T is 18,431,842 bytes, within smith's 33,554,432-byte
  ceiling. The stock keeps its figures, 10,662 and 8,192, and
  `temper-box init`'s T of 16,803,810.
- **What moves with the payload,** P, each named by the start at its path
  against the agent's figure, beside the limits T sets above:

| Limit | Relation | The start names it against |
|---|---|---|
| `tasks.message_bytes`, `tasks.result_bytes` | P − 2 | the agent's message (`tool_payload` less 2) |
| `tasks.inbox_bytes` | 82 × (P − 2), one of each for the standard inbox | its own bound |
| each worker's `hub.event_bytes` | P − 2 | `tool_payload` less the 2-byte separator |
| `checkout.message_bytes` | P | the agent's `call_bytes` |
| `forge.client.op_bytes` | at least `tasks.result_bytes` | `tasks.result_bytes` |
| `forge_component.body_bytes` | 2 × `forge.client.op_bytes` + 21 | `forge.client.op_bytes` |
| `forge.client.answer_bytes` | at least 2 × `forge_component.body_bytes` + `forge_component.string_bytes` + skein's file-record bytes; the reference takes 255,570 | the forge client's validity check |
| both `hub.outcome_bytes` | at least P; `standard`'s 131,072 holds the reference's | the agent's outcome (`tool_payload`) |
| both `core.call_answer_bytes` | at least P; follows it unless overridden | the agent's call input |

- **A sub-agent's answer:** smith's `run.answer_bytes` follows P as the
  outcome does, the larger of 4,096 and P (smith's `protocol/limits.md`,
  3.10), so a survey a goal's run asks through `sub_agent` answers in up
  to 63,740 bytes on the reference and 10,662 on the stock. temper
  declares nothing for it: smith derives it at the agent's start, holds
  it under `render-in-result` and prints it among the agent's effective
  limits. A longer answer is cut, its words saying how much was left out
  and to ask again, narrower (smith's `domain/run.md`, 5.3).
- **A batch:** `standard`'s `authority.batch` is 32, so a `change` of up to
  32 changes passes (jig's `domain/authority.md`, section 8), still
  within `tasks.batch`, `tasks.batch_bytes` and the task room a batch
  needs. Each model's figures don't enter it. It costs a retained denial
  300 findings at most where it took 92, and a batch's aggregate 512
  grants and 544 executors where it took 144 and 153: bounds on what a
  value may hold, none of them allocated ahead. It stands in jig's profile
  and not the reference's, because `tasks.batch`, 160, already sizes
  `standard` for it and a batch's count isn't a model's figure.
- **A brief:** `standard`'s `hub.charter_bytes` is 262,144, and smith's
  charter allowance is the larger of 65,536 and twice the payload (smith's
  `protocol/limits.md`, 3.10). The reference's is 127,480, so this receiving
  allowance binds before the hub's 262,144: the engine fits a charter to
  the smaller of the two where its process has the agent, and to 65,536
  where it doesn't know its worker's (`../../domain/agent/tools.md` and `../../domain/agent/README.md`, 8
  ). Its coding charter holds a 63,740-byte spec beside its
  instructions and tools. A section past the room those leave is cut with
  its `[cut N bytes; whole: READ]` line, including a 131,072-byte section,
  whatever the hub admits. The box's charter allowance stays at 65,536.
- **A report:** a change's `body` follows the call's payload
  (`../../domain/agent/charter.md`, 3).

## 8. The stock ways

The ways a project can work, pre-made settings of its own scope that it
opts into (`../../draft/stock-ways.md`, sections 2 and 5): data in the
`stock` module, given to the engine at each start beside the forge's
defaults (`stock.md`), so that a test world gives its own or none.

| Way | Its headline | Its line | Its items, in the project's scope |
|---|---|---|---|
| You approve | "You approve every change" | "An agent makes each change; nothing reaches main until you approve it." | kind 2's rule 1: every branch (`branch`, open: 2), enforced, CI, up to date, no gate, one approval by role 0, `clean`; rules 2 to 4 removed; kind 8: You approve's table (`../../domain/forge/attention.md`, 6) |
| Agents review | "Agents review every change, and code lands it" | "An agent makes each change, another reviews it, and code lands it." | kind 2's rule 1, the stock's (`stock.md`); rules 2 to 4 removed; kind 8: Agents review's table |
| Draft first | "Draft first, then agents build it on its own branch" | "You and an agent write a design draft; agents build it on its own branch, and it lands whole." | kind 2's rule 1: `branch`, exact `main`, enforced, CI, up to date, no gate, no approval; rule 2: `branch`, open `temper`, enforced, no CI, not up to date, no gate, no approval; rules 3 and 4 removed; kind 7: the project's own, or the stock's with none, with `drafts` and `goals` on and `way` 4; kind 8: Draft first's table; the project's policy, where its delegation lacks charter 5, the goal's closer (procedure 6), role 0, the merge job (job 3) or the close job (job 4), with them added to its ceiling's and role 0's delegation |

- **In this order,** from more of the person to more of their agents, as
  the page's tabs are. Each way's name, headline and line are its words,
  the stock's, carried by the read (below), so that the page and `see
  ways` say the same. Its headline is How work runs' heading while the
  project runs it (`../../client/pages/settings.md`, 2): the words saved with a
  stock way (`../../draft/whole-story.execution.md`, P3e), at most 120 bytes.
- **A way's items** are what jig's `ChangeSettings` carries (jig's
  `domain/engine/settings.md`, 2), computed by the root for one project from
  the way and that project's settings: a rule *set* to its value, each
  rule the way names none of *removed*; kind 7 *set* to the project's
  value with the way's members laid over it; kind 8 *set* to the way's
  table, whole (`Classes::stock`, `../../domain/forge/attention.md`, 6), so that a
  way sets what reaches the project's people
  (`../../draft/what-reaches-you.md`, section 2); a project's policy item
  only where its delegation lacks what the way needs. Every value is
  set whole, never reset, so that a way's values become the project's
  own and a release's new stock leaves them.
- **Nothing else:** charters, models, questions (kind 6), kind 7's
  `changes`, a charter's or the project's words (kinds 9 and
  10), a repository's gates and tiers, and the project's
  landing stay as they are, so that a person's choices of models survive
  a change of way. You approve and Agents review set no kind 7: a project
  that ran Draft first keeps its goals' way, and their paths over it
  start with a goal.
- **Each way's path** over the project is `../../domain/forge/change/branch-path.md`'s
  derivation over the project's settings with the way's items laid over
  them. The way the project runs is the first whose path equals the
  project's own, stop for stop; none, where none does.
- **The forge's `ways` read** (`../../client/forge/records.md`;
  `../../client/forge/served.md`) gives each way
  with its words, its path and its items, the way the project runs or
  none, the project's own path, the project's words (kind 10) or none,
  and whether the reader may change the
  project's settings: an owner of the project, as jig's `ChangeSettings`
  asks (jig's `domain/people/acts.md`, 1). "Use this" and "Keep it" send
  its items (`../../client/pages/settings.md`, 2).
