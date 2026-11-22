# temper's settings kinds

This document holds temper's settings kinds, split from [`settings.md`](settings.md).

## 1. temper's kinds

temper declares ten kinds of its own beside jig's, each owned by the
forge's connector, which judges them (jig's `domain/engine/settings.md`, 5):

| Kind | Scope | Numbers | Value | Decided by |
|---|---|---|---|---|
| 1, the deployment's landing rules | the deployment | a rule's, from 1; at most `forge.landing_rules` | a landing rule (below) | the deployment's owners |
| 2, a project's landing rules | each project | a rule's, from 1; at most `forge.landing_rules` per project | a landing rule | that project's owners |
| 3, the forge's charters | the deployment | 1, the change's; 2, its reviews' | a charter's number, a u32 | the deployment's owners |
| 4, a project's landing | each project | 1 | how the project lands, by queue or by turn, and the turn's settings (`../../domain/landing-settings.md`, 1.1), taken at the next start | that project's owners |
| 5, a project's gates | each project | 1 | each of the project's repositories' test gates and tiers (`../../domain/landing-settings.md`, 1.1), taken at the next start | that project's owners |
| 6, a project's questions | each project | 1 | the question route a change of the project takes when its maker names none: none or up (`../../domain/forge/change/questions.md`, 1) | that project's owners |
| 7, a project's ways | each project | 1 | what the project shows and the way a goal starts by, a `ProjectWays` (below) | that project's owners |
| 8, a project's classes | each project | 1 | what agents decide for a person and what reaches them, a `ProjectClasses` (below; `../../domain/forge/attention.md`, 6) | that project's owners |
| 9, a charter's words | the deployment | a charter's, 1 to 64 (jig's `domain/engine/settings.md`, 1) | what the charter does, in a line, a `CharterWords` (below) | the deployment's owners |
| 10, a project's words | each project | 1 | How work runs' headline and lede for the project's own way, a `ProjectWords` (below) | that project's owners |
| 21, the agents' figures | the deployment | 1 | the agents' profile figures and each endpoint's limits, an `AgentFigures` (`figures.md`, section 5), taken at each worker's next start | the deployment's owners |
| 22, temper's limits, `temper-limits` | the deployment | 1 | a list of overrides of temper's limits, each a name, a value and a reason (`limit-settings.md`, 2.3) | the deployment's owners |

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
  names the eleven kinds in `Kind::ALL`'s order, `findings` last and
  appended, so every earlier kind keeps its encoding, and `ReachClass` the four classes, `decide`, `ask`,
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
    (`settings.md`, 8), whatever its own; the stock has none.
  - *Their bounds:* each UTF-8 holding no control character, a `line` and
    a `headline` of 1 to 120 bytes, a `lede` of 1 to 480. A kind's bound
    is its record's encoding at them.
- **A landing rule's value** is a `LandingRule` record in temper's
  codec, `engine.md`'s rule member for member but `connector` and `kind`,
  which are always the forge's and its merge: `pattern`, `enforce`,
  `ci`, `up_to_date`, `gates`, `approvals` and `sets`, each as `engine.md`
  says, `sets` last. Its largest bytes, the kind's bound, are its encoding
  with a pattern at jig's pattern bounds, `forge.change_policy.gates`
  gates, `authority.roles` approvals and 6 sets; `forge.landing_rules`, a limit of the
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
  judged for its own merges alone (`settings.md`, 3), so that a project's way is data
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
    `authority.roles`; with more than 6 sets, a set's number outside 1
    to 6 or named twice, or, of kind 1, any set ("a deployment's rule
    names no path set": sets are a project's). A set the project's
    classes don't hold is no refusal: each item is judged alone, and the
    rule covers every change until the classes hold it (`engine.md`);
  - *the forge's charter* outside 1 to 64, a charter's numbers (jig's
    `domain/engine/settings.md`, 1), or removed: the forge always has all three charters, and an owner who
    wants the stock's back resets it;
  - *a project's questions* that is not one of the three routes;
  - *a project's classes* that does not decode as a `ProjectClasses`,
    or that `Classes::check` refuses (`../../domain/forge/attention.md`, 6), the
    refusal's words saying which of its faults;
  - *words* of kind 9 or 10 that are empty, past their bound, not UTF-8
    or holding a control character, or of kind 9 at a number outside 1
    to 64, a charter's numbers; a removed or reset item of either is
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
  person linked on a forge is known there (`../../domain/forge/adoption-person.md`, 1).
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
  longer waited on (`../../domain/forge/change/gates.md`, 1), and a change
  a rule's sets now cover waits for what the rule asks. A commit that
  changes a project's classes' sets steps, likewise, each live change of
  the project that a rule naming a set covers by its branch, so a set's
  patterns changed reach the changes they now cover or no longer do
  (`../../draft/what-reaches-you.md`, section 7). At a start the forge is given the stock
  defaults, tells jig which changed, and holds the current values
  before its other records (`../../domain/forge/effects.md`, section 2).
- **Erased with their project.** A project deleted (jig's
  `RemoveProject`, jig's `domain/people/acts.md`, 2) has its settings
  erased by the connector owning each kind: the forge erases its kinds
  2, 4, 5, 6, 7, 8, 10, 12 and 13 of that project's scope, their current changes and the
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
  7, 8, 10, 12 and 13) in that scope alone. How work runs reads a project's words in
  its `ways` read (`settings.md`, 8), which any party of the project reads. A
  `Setting` read's answer, `SettingAnswer { value, revision }`, names
  the revision its value was set or removed at, read from the same
  state as the value, as jig's `settings` document names it in that
  setting's line (`last`), and none where it stands at its default; so
  a client reads it again only for a newer one
  (`../../client/pages/settings.md`, 1, *What the client holds*).
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
  change's procedure delegation to its producer's charter and its
  review charter. Its producer's charter is the one its `change` call
  names (`../../domain/agent/tools-change.md`, 1), or, with none, the change's
  charter (number 1), its default. The change keeps their numbers on
  its row: its producing, its repairs, its resolutions and its checks'
  gates run under the producer's charter it was made with, its agents'
  gates, the reviews, under the
  review charter it was made with (number 2; the stock's charter 4),
  and a newer charter of either reaches the
  changes made after it (`../../domain/root-translation.md`, section 1).
- **The foreman is retired** (`../../draft/stock-charters.md`, 3): kind 3
  has no number 3 and kind 6 no `foreman`. Their codecs keep each
  remaining value's encoding and reuse none: a stored kind 3 number 3,
  or a stored kind 6 whose value is the retired route, is refused, not
  read, naming its setting as ctl names it (as a start names
  `charter 1.hosts`), with no compatibility before v0.1.0: at a start,
  which then does not start, and at a restore of a settings revision
  holding it, which then changes nothing. A proposal or a change setting
  either is refused as an unknown value is.
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
    64, the stock's 1 to 32 and a deployment's own 33 to 64 (jig's
    `domain/engine/settings.md`, 1); decided by the deployment's owners.
    Read at a number 1 to 64 where none is held, it answers "none kept",
    as kind 9 does at a charter's number 1 to 64;
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
    else refused for its scope; a number 0 or past 64, for its number; a value not a `TemplateGoal`, for its value; a
    removal or a reset always good;
  - *named* `template-goal`, with the template's `number`, in a settings
    file and a run's `settings`, `see` and `propose_settings`
    (`../../client/ctl/tools-acts.md`, 1; `../../domain/agent/agents.md`, 1), its value
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
    domains, each refusal `../../domain/forge/follows.md`, section 2 names; a removal or a
    reset always good, and either cancels the follow's task;
  - *named* `follow`, with the `project` and its `number`, in a settings
    file and a run's `settings`, `see` and `propose_settings`, its value
    `follows.md`'s settings-file form; read by a party with a role in the
    project, as kinds 6 to 8 are;
  - *the stock's:* none.
- **A project's dependencies, kind 13**
  (`../../domain/forge/dependencies.md`, section 2): each another project
  of the deployment the project builds on, acted on by nothing. A kind of
  temper's beside the table's, owned by the forge's connector and kept,
  erased, read and never pinned as they are:
  - *its scope and numbers:* each project's; its number a dependency's,
    1 to `forge.dependencies`; decided by that project's owners;
  - *its value,* a `Dependency` record in temper's codec, its one member
    `project`, the number of the project depended on; its bound, the
    kind's, is its encoding;
  - *judged* by the forge's structure first, then by the root across
    domains, each refusal `../../domain/forge/dependencies.md`, section 2 names; a removal
    or a reset always good;
  - *named* `dependency`, with the `project` and its `number`, in a
    settings file and a run's `settings`, `see` and `propose_settings`,
    its value `{"project": Q}`; read by a party with a role in the
    project, as kinds 6 to 8 and 12 are;
  - *the stock's:* none.
- **A project's areas, kind 14, and its triager, kind 15**
  (`../../domain/areas.md`; `../../draft/standing-roles.md`, section 4):
  kinds of temper's beside the table's, owned by the forge's connector
  and kept, erased with their project, read and never pinned as kinds 7
  and 10 are; two kinds, so that a way sets the triager and leaves the
  areas as they are (`settings.md`, 8):
  - *their scope and numbers:* each project's, number 1; decided by that
    project's owners;
  - *an owner,* `AreaOwner` in temper's codec, `task` (u64), `party` (u64),
    `role` (u32) or `charter` (u32) (`../../domain/areas.md`, section 3);
  - *kind 14's value,* a `ProjectAreas` record: `areas`, at most 16, each
    `Area { name, words, owner }`, its owner an `AreaOwner`, its name 1 to 32 bytes of lowercase
    ASCII letters, digits and `-`, starting with a letter, unique in the
    record, its words 1 to 480 bytes of UTF-8 holding no control
    character; *kind 15's,* a `ProjectTriager` record, `owner`, an
    `AreaOwner`. Each kind's bound is its record's encoding at them;
  - *with none,* no areas, and the project's owners triage
    (`../../domain/areas.md`, section 4);
  - *judged by the forge,* as the table's kinds are: a value that does
    not decode, a name or words outside their bounds, a name twice; and
    by the root, from the core, as kind 7's way is, an owner that is not
    of the project: a `task` not a live task of it, a `party` with no
    role in it, a `role` past jig's `authority.roles`, a `charter` 0 or
    past 64, a charter's numbers; the refusal names the area and the
    owner. A removal or a reset is always good;
  - *named* `project-areas` and `project-triager`, with the `project`,
    in a settings file and a run's `settings`, `see` and
    `propose_settings`; kind 14's value `{"areas": [{"name": NAME,
    "words": TEXT, "owner": OWNER}, ...]}`, kind 15's `{"owner":
    OWNER}`, OWNER one of `{"task": N}`, `{"party": N}`, `{"role": N}`
    and `{"charter": N}`; read by any party with a role in the project,
    as kinds 2, 6, 7, 8 and 10 are;
  - *read by* the root's owner rule (`../../domain/areas.md`, section 4)
    and the forge's `findings` subject (`../../client/forge/records.md`);
    a save of either moves the findings it concerns, and tells their new
    owners (`../../domain/areas.md`, section 5).
- **The deployment's welcome, kind 20** (`../../draft/config.md`, 6.5;
  `moved.md`, section 4), which was the document's `application.welcome`:
  a kind of temper's beside the table's, owned by the forge's connector
  and kept, read and never pinned as kind 9 is. Kinds 16 to 19 are left to
  the configuration's other milestones, which run beside the one that
  made it (`../../draft/config.execution.md`):
  - *its scope and number:* the deployment's, number 1; decided by the
    deployment's owners;
  - *its value,* a `Welcome` record in temper's codec, `shown`, a bool:
    whether the welcome shows (`../../client/pages/README.md`) and the
    deployment offers the playground (`../setup/playground.md`). Its
    bound, the kind's, is its encoding;
  - *its default,* from how the deployment was made: shown under the box,
    whose `temper-box internal serve` is given `--box` at every start
    (`../setup/box.md`, 1), as a deployment `temper-box init --box` made
    is; not shown otherwise. The root gives it with the stock's defaults
    at each start (`settings-kinds.md`, 1), so a deployment made by `temper-box init` and
    served by a hand-written unit shows none until an owner sets it;
  - *judged by the forge,* as the table's kinds are: a value that does not
    decode. A removal or a reset is always good, back to the default;
  - *named* `welcome`, with no scope, in a settings file and a run's
    `settings`, `see` and `propose_settings`; its value `{"shown": true}`;
    read by any signed-in party, as the deployment's read is
    (`../../client/forge/records.md`);
  - *read at once* by the deployment's read, whose `welcome` is the
    setting's current value, so a change reaches each client as any change
    of that read does, and by `StartPlayground`, refused *not offered*
    while the welcome isn't shown (`../setup/playground.md`). The
    playground is configured at start whatever the welcome says, so that
    turning it on offers the playground with no restart.

## 2. Open questions

- **Settings refusal positions** (`settings-kinds.md`, 1; jig's `domain/engine/settings.md`, 2):
  the core's `at` counts from 0, and temper's form prints it as given,
  while a settings file and temper's translation refusal count from 1.
  Consistent person-facing numbering is later work. Default: keep the
  protocol's position and the current words until both temper paths agree.
