# Ways of working, several at once

Draft, 2026-11-26, from sessions with the user on 2026-11-25 and 2026-11-26. It started from how agents' prompts are assembled and what a repository's `.temper/` holds, and what a good story for both would be. The user added hooks named by the moment they run at, a ladder from a person working with one agent, where "temper adds only the tool calls", to a whole milestone, and then the frame this draft takes: "I'd like temper to support various 'ways' of working at the same time for the same project", as in "for this change I want to pair directly with an agent, for this other I want to delegate and I want a automatic review, for this other one I want a milestone". temper offers a few stock ways that a project uses as they are, changes by copying, removes, or adds to with ways of its own. The settings only owners change are "overridable per-way and possibly per-change", as in "milestone way does no require manual approval", but "this one milestone requires approval". Path rules still hold for any change, whatever its way. Standing roles "sit between the person and ways". Settings get a name beside their number, "mostly an internal detail", and the name maps to `.temper/`.

Second pass, 2026-11-27, from a session with the user that day, over a survey of temper's primitives and a gallery of ways drawn from them. The user's frame: make the whole gallery possible, "eventually (it can be a series of incremental drafts)", built into temper, each way tested, shown in the web UI with the note that every variation is possible, and every primitive explained to Ask temper "so he can propose anything that is possible". A new project picks its ways, from the gallery or from scratch, and they populate its `.temper/`: "no magic, expecially around prompts". `.temper/` is "all or nothing": present, it's authoritative in full; absent, the stock ways run. Its formats are versioned, so that what is decided now keeps being supported. After an upgrade, Ask temper reviews a project's ways and proposes changes "in an intelligent way", "not mechanically", since "prompts are very sensitive". Sharing ways across projects and reconciling drift are left for later. The landing part goes last. The hooks moved to a draft of their own, [hooks.md](hooks.md).

Third pass, the same day: a way's file is written in jig's own terms. The user found `shape` and fixed role names to be the former approach, "we have fixed ways that you can only slightly tweak", against "we have primitives that you can fully compose", and asked that "no role or shape should be 'assumed'": the file should be "as close as possible to the underlying model (that's what makes it flexible), while preserving our light 'opinionated' takes and 'guardrailes'", jig's mechanics with temper's own as its connector. Shapes go: a way is a set of task recipes, and the stock ways are compositions like any other. Then the same rule for the words (the user's yes, 2026-11-27): a repository with its own ways holds its charters whole, the stock's text written into its files, and Ask temper's review after an upgrade is what keeps them current.

The first pass covered the mechanics and the whole picture, so that each piece is built in turn and they still fit together; the second settles the vision the increments build towards (`ways.execution.md`). How a person meets it, in pages, words and the terminal, is the next pass (section 12). It revises a project's single stock way (`../protocol/configuration/settings.md`, section 8), standing roles brought by a way (`standing-roles.md`, section 10), kind 7's one preferred goal template (`../protocol/configuration/settings-kinds.md`, section 1) and the charter assembly's sources (`stock-charters.md`, sections 3 and 4). It takes up `stock-charters.md`, section 11's project overlay from `.temper/`, and adds names to `settings-numbers.md`'s numbered kinds. It wins where they differ until its plans fold it in.

## 1. In one page

- **Three homes, one rule** (section 2). `temper.toml` holds what a machine runs. The store's settings hold what grants and bounds: people, accounts, models, budgets and the rules that always hold. A repository's `.temper/` holds how work is done on that code: its ways, its charters' words and its hooks. Nothing lives in two of them.
- **A way is a set of task recipes in jig's terms** (section 3): which task a person starts, and for each of its tasks who executes it (an agent under a charter and a model, temper's change procedure, a gathering, a person), what counts as done, what it holds (a branch of its own, a copy), which of the way's tasks it may start and how many, when it wakes, its budget, deadline and priority; for a change, who produces it, which gates judge it, where it lands and what landing asks; and what reaches its requester. No shape or role is assumed: the stock's Pair, Hand it over and Milestone are compositions like any other. temper keeps a few guardrails over every way (3.1).
- **New work picks its way when it starts and records it** (section 4). Its children take their part of it. Work started before ways runs as it did.
- **Values in four layers** (section 5): the floor, the project's defaults, the way, and the piece of work. The floor is the landing rules, rules by path among them. It always holds, and no way or piece of work lowers it. Below the floor, the work's own value wins over its way's, and its way's over the project's default. One calculation gives a change's criteria wherever they are used.
- **A charter's words come from five sources** (section 6): smith's harness; the charter itself, whole from `.temper/charters/` where the repository has its own ways, else the stock's; the deployment's lines about its machines; the repository's `AGENTS.md`; and the task's words. A run records which it read.
- **Standing roles start work, as people do** (section 7). A kind is a named charter. An instance is a live task, with its person, area, budget and Now, kept in the store and never in `.temper/`. A role's grant says which ways it may start. Ways stop bringing standing roles.
- **`.temper/`** (section 8): `ways.toml`, `ways/<name>.toml`, `charters/<name>.md` and `hooks/` ([hooks.md](hooks.md)). temper reads it whole from one commit of the home repository's landing branch, checks a change to it before it lands, and, where a project's owners turn it on, asks one of them to sign off on it. All or nothing: a repository with `ways.toml` runs its own ways alone, and one without runs the stock's. Each file names its format, and every format shipped keeps reading (8.8).
- **A gallery** (3.5): ways built into temper as references, each tested and drawn by temper's own path, shown on the web and known to Ask temper, which explains every primitive. A new project picks from it, or starts on the stock ways, and temper writes what it picked into `.temper/` by a change (8.7).
- **Names** (section 9): every template, charter and policy gets a lowercase name. The number stays the store's key. A way's name is its file's. A name is also a path in `.temper/`.
- **fabrig and dev.fabrig.ai move onto it** (section 10), and upgrades keep what each build already runs (section 11). A release's changes reach a project's own ways and words through Ask temper's review, never by rewriting them.

## 2. The picture

### 2.1 Three homes

| Home | Holds | Changed by | Read |
|---|---|---|---|
| `temper.toml` | what a machine runs: its places, its figures, its endpoints (`config.md`, section 3) | whoever runs the machine | at a start |
| settings, in the store | what grants and bounds: people and their roles, accounts, models, budgets and ceilings, the floor (section 5), the project's defaults, the deployment's charters and its lines for them | the deployment's or the project's owners, by a decision (`configuring.md`, section 3) | always |
| `.temper/`, in a repository | how work is done on this code: its ways, its charters' words and its hooks | whoever lands on the repository's landing branch, under the floor, and an owner's sign-off where the project turns it on (8.6) | whole, from one commit of the landing branch (8.5) |

- **The rule:** whatever grants, bounds or spends is in settings, and whatever describes method is in the repository. A way in `.temper/` can ask for more review than the floor does, never for less.
- **Why both, not one:** a repository's method travels with its code, is reviewed as code, and is the same on every deployment that serves it. Authority can't travel with the code, since whoever can land would then grant themselves more. A forge splits it the same way: workflow files live in the repository, and a branch's protection in the forge's settings.
- **`charters/`, not `roles/`:** in temper a role is a standing role or a person's role in a project, and an agent's rules are its charter.

### 2.2 The pieces

```
a person ─ talks with ─▶ a standing role                       (section 7)
   │                         │
   └──── starts work in a way, as its requester ◀──┘           (section 4)
                    │
   the way: its tasks · executors · contracts · holds · delegation
            · gates · landing · what reaches               (section 3)
                    │
   each agent's words: harness · the charter (.temper/charters, or the stock's)
                       · the deployment's lines · AGENTS.md · the task's words (section 6)
                    │
   landing: the floor, always, and the work's own rule
            (the project's defaults ◀ the way ◀ the work)      (section 5)
                    │
   the repository's hooks: deliver · finish · merge · close    (hooks.md)
```

- **Orthogonal by construction:** a way doesn't know who started its work, and a standing role starts work in any way its grant allows. The floor holds whoever the requester and whatever the way.
- **temper's code stays the mechanism** (LLM for judgement, code for mechanisms): a way composes jig's tasks and temper's procedures. It can't add a mechanism: an executor, a resource, a procedure or a requirement temper doesn't have.

### 2.3 What there is today

From a read-only survey of main at `de275134b0`, 2026-11-26:

- **One displayed way per project.** The stock ways, You approve, Agents review and Draft first, are pre-made settings a project applies with "Use this": landing rules, what reaches its people and, for Draft first alone, the goal template it prefers and the triager. Settings a way names none of keep their values (`../protocol/configuration/settings.md`, section 8). How work runs, `see ways` and Ask temper show the first stock way whose path equals the project's, or none. That is a display, not a choice any running work reads.
- **Per piece of work already:** a change's producer, its models and its `questions` route (`../domain/agent/tools-change.md`, section 1). A goal's template, models and budget at `goal start`, and the goal page's Always ask, kept as the spec's line `Asks: always` (`../client/pages/goal.md`, section 3). A milestone's budget, deadline and priority, as its task's, and no landing or reaching settings of its own (`../domain/milestones.md`, section 3).
- **Landing rules cover branches and paths.** Kinds 1 and 2 match the branch a change lands on. Every matching enforcing rule holds, and the project's add to the deployment's. A project's rule naming path sets covers only the changes that touch them, whatever the change, and its approval is a person's review on the forge (`what-reaches-you.md`, section 7; `../domain/forge/change/gates.md`, section 2). A plan may add gates to one change (`../domain/forge/change/gates.md`, section 1).
- **Charters are deployment-wide.** Kind 3 names the producer and the reviewer for every project. A change records their numbers, and jig resolves each new delegate's charter at the newest settings revision (jig's `domain/engine/settings.md`, section 3). A deployment sets a charter whole or lays an overlay over a stock one (jig's `domain/engine/overlays.md`). No `.temper/` directory is imported as charters or settings. A run reads repository text from the documents its charter names (`../protocol/configuration/settings.md`, section 6) and from the workspace's guide (`../domain/agent/charter.md`, section 1).
- **`.temper/` holds scripts.** `pre-pr` is the delivery check, by a convention temper's code gives smith (`../domain/agent/charter.md`, section 4). `merge` and `checked-close` run because the project's `merge_job` and `close_job` name them, and those settings' presence also switches the merge job and the close change on (`../domain/landing-settings.md`, 1.1). `close` is called by `checked-close`. `goal-pre-pr` and `goal-child-pre-pr` are wrappers for CI.
- **Numbers, no names:** a template has a display name, and a charter has none. A record refers to each by number (`settings-numbers.md`, section 3).

## 3. A way

### 3.1 What a way's file holds

| Member | What it says |
|---|---|
| its name | its identifier (section 9): the file's name in `.temper/ways/`, or a stock way's name where the repository has no `ways.toml` |
| `format` | the file's format (8.8) |
| `headline`, `line` | its words: a heading of at most 120 bytes and a sentence |
| `start` | the task a person, a script or a standing role starts with this way, by its name among `tasks` |
| `[tasks.NAME]` | each kind of task the way's work is made of (3.2), named by the project |
| `[reaches]` | what reaches its requester, written out (3.3) |

- **In jig's terms** (jig's `domain/model.md`, section 3): a task has a spec, a result contract, a requester, an executor, an authority, a wake policy and the resources it holds. A way's tasks say those, and temper's connector adds its own: branches and copies to hold, the change procedure, the closer, landing's requirements, and what reaches a person.
- **Names are the project's.** `start = "milestone"` and `[tasks.lead]` name nothing temper knows. temper knows only the executors, contracts, resources and procedures a task names.
- **What temper keeps, over every way:**
  1. **Only temper's procedures write a landing branch:** an agent's work reaches one only as a change (3.3), and an agent writes only what it holds.
  2. **The floor always holds** (section 5): a way adds requirements to a change, never removes the floor's.
  3. **Authority only narrows:** what a task may start, its tools and its spend are checked against the project's policy and the starter's role at every start, as jig checks any delegation (jig's `domain/authority.md`). A way can't grant.
  4. **Agents judge, people approve:** a gate is a task with a verdict, and an approval is a person's requirement on the forge, which no agent meets.
  5. **Words in charters, figures in settings:** a charter's words live in its files (section 6), models come from the catalogue, ceilings from the policy.
  6. **Bounded:** at most 8 kinds of task in a way, and a tree at most 4 deep, its starting task the first level.
- **A file that breaks one** fails the landing check (8.6), naming the file, its line and the rule.

### 3.2 A task's recipe

| Member | What it says | In jig and temper |
|---|---|---|
| `executor` | who carries it out: `{ agent = CHARTER, model, effort, speed, sub_agents, later }`; `"change"`; `"gathering"`; or `"person"` | an agent under a charter (jig's `domain/model.md`, 3.2); temper's change procedure (`../domain/forge/change/README.md`); jig's gathering (jig's `domain/tasks/gatherings.md`); a person task |
| `contract` | what counts as done: `report`, `verdict` with its `labels`, or `change` | the result contract (jig's `domain/tasks/README.md`, section 3) |
| `holds` | the resources it holds: `branch`, a branch of its own; `copy`, a writable copy of its own | temper's symbolic resources (`../domain/forge/branches.md`, section 1) |
| `conversation` | whether its requester talks with it | the conversation flag |
| `delegates` | which of the way's tasks it may start, each with `at_most` | its authority's delegation, narrowed to those templates |
| `wakes` | when its executor runs: `results`, `questions`, `news`, `words`, `timers` | the wake policy (jig's `domain/tasks/messages.md`, section 3), the charter's when left out |
| `budget`, `deadline`, `priority` | its spend asked for, a span from its start, its place in the queue, or `tracked = false` | the task's, within the template's and policy's bounds |
| `lands`, `into`, `landing`, `catch_up` | for a task holding a branch: `lands = "whole"`, into `"parent"` or `"base"`, under `landing`, kept near its base by `catch_up` | the closer and its close change (`../domain/forge/branches.md`, section 2) |
| `gathers` | for a gathering: which of the way's tasks it holds | the gathering's members |

- **An agent's charter** is a charter's name (section 9), or one composed in place, as jig holds a charter: `agent = { instructions = ["common", "lead"], parts = [{ on = ["first"], file = "designer" }, { on = ["question", "result"], file = "coordinator" }] }`, each file a charter file of the project's or the stock's (6.2). The stock goal charter's designer and coordinator are its parts so: one agent, its first run and its later wakes.
- **Its model** is the catalogue's, by name, with an effort and a speed (`model-speeds.md`); one the deployment doesn't hold is ignored, and the preview says so (6.4).
- **Every member but `executor` is optional.** One left out takes the layer below it (section 5): the project's default, else the charter's or the template's.
- **Resolved as templates:** the engine makes each of a way's tasks a template of the project's own, at its repository revision (8.5), whose executor, contract, budget, deadline, tracking, flags, model choice and resources are the recipe's, and whose authority lets it start only the way's tasks it names in `delegates` (jig's `protocol/configuration-templates.md`, section 1). Work started in the way is made from those templates.
- **What a run can start** is what its tools can: an agent starts a change with `change` and another of the way's tasks with `goal` where its charter grants that tool (`../domain/agent/tools.md`); a change's procedure starts its producer and gates itself.
- **What isn't a way's:** a start that changes no repository is no way's and keeps its template: a chat with no copy of its own, Ask temper, a design session, a standing role.

### 3.3 Changes, landing and what reaches

A task with `executor = "change"` is temper's change procedure, code and no agent:

- `into`: `"parent"`, the branch its requester holds, or `"base"`, the repository's landing branch.
- `producer`: which of the way's tasks writes it, an agent with `contract = "change"`; or `"handed"`, its requester's own edits, handed over, as a pair's "Land this" hands its copy over (`../domain/forge/change/opened.md`).
- `gates`: which of the way's tasks judge it, each an agent with `contract = "verdict"`, and `lens` where it judges by one, blocking unless `advisory = true`, with its freshness (`../domain/forge/change/gates.md`, section 1).
- `landing`: `ci`, `up_to_date` and `approvals`, each `{ role, people, freshness }`, as a landing rule's own fields without the pattern and path sets that say which changes a rule covers (`../protocol/configuration/settings-kinds.md`, section 1). It makes the work's own rule (5.4).
- `questions`: `up` or `none`, how its producer's questions travel.
- **`share`, built last** (`ways.execution.md`): how the change shares a landing, in a project that lands by turn (`../domain/landing.md`, section 1): `alone`, a turn of its own; or `with`, the ways whose changes may share a turn with its own, and `at_most`, how many; and `combine`, `merge` or `squash`. A turn takes the waiting changes whose ways allow one another, in queue order. Queue against turn stays the project's; a project landing by queue lands every change alone.

A task holding a branch lands it whole, by temper's closer, as a goal's branch lands today:

- `lands = "whole"`, `into` `"parent"` or `"base"`, and its own `landing` table for the branch's landing.
- **`catch_up`, built late:** `when-needed`, the default, a coordinator asking for a catch-up for a change that needs what the base gained (`../domain/forge/updates.md`, section 1); or `after-each-landing`, temper making one itself after each landing into the branch. Either takes the merge hook ([hooks.md](hooks.md)), and a conflict goes to a resolution.
- **A gathering may hold a branch** and land whole (the user, 2026-11-27): its gathered tasks land `into = "parent"`, its branch, and the branch lands once the gathering is whole and its tasks have ended. Built late. A gathering holding none, the default, lets each of its tasks land on its own, which keeps drift from the base small.

`[reaches]` is the classes table of what reaches the requester, written out (`../domain/forge/attention.md`, section 6): each movable kind, `wording`, `retry`, `implementation`, `tests`, `dependency`, `departure` and `findings`, at `decide`, `ask` or `ask-away`; a kind left out asks even when away; `[[reaches.narrow]]` items, each a kind, words of at most 120 bytes and a class, narrow one ("fixes that keep the wire format"). Money, irreversible acts and rules always wait, and the path sets' classes stay the floor's. The kinds are temper's, the same for every project and tied to no way of working: how a project designs, drafts or records its work is its charters' business, which temper doesn't read. A project's own nuance goes in `narrow` words.

### 3.4 The stock three, as compositions

```toml
# hand-it-over: you describe it, an agent writes it, another reviews it, and code lands it
start = "change"
[tasks.change]
executor = "change"
into = "base"
producer = "build"
gates = ["review"]
landing = { ci = true, up_to_date = true }
[tasks.build]
executor = { agent = "implementer" }
contract = "change"
[tasks.review]
executor = { agent = "reviewer" }
contract = "verdict"
```

```toml
# pair: you and one agent, in its own copy; "Land this" hands its edits over
start = "pair"
[tasks.pair]
executor = { agent = "pair" }
contract = "report"
conversation = true
holds = ["copy"]
delegates = { land = {} }
[tasks.land]
executor = "change"
into = "base"
producer = "handed"
landing = { ci = true, up_to_date = true }
```

```toml
# milestone: design first; each goal is built on its own branch, and lands when it's done
start = "milestone"
[tasks.milestone]
executor = "gathering"
gathers = "goal"
[tasks.goal]
executor = { agent = "goal" }        # the stock goal charter: designer on its first run, coordinator after
contract = "report"
holds = ["branch"]
lands = "whole"
into = "base"
landing = { ci = true, up_to_date = true }
delegates = { change = {} }
[tasks.change]
executor = "change"
into = "parent"
producer = "build"
landing = { ci = false, up_to_date = false }
[tasks.build]
executor = { agent = "implementer" }
contract = "change"
```

- **Their `[reaches]`:** Pair's leaves every kind out, so every decision asks even when away, as You approve's table does today. Hand it over's and Milestone's write out today's Agents review and Draft first table (`../domain/forge/attention.md`, section 6): `wording`, `retry` and `tests` at `decide`, the rest at `ask`, and `findings` narrowed to `decide` for "a repair goal for a routine red after landing".
- **You approve becomes a value,** an approval with one owner in a change's `landing`, which any layer sets (section 5), rather than a way of its own. Agents review becomes Hand it over, and Draft first becomes Milestone, with the same landing.
- **A goal alone is a milestone of one,** started the same way, or a way whose `start` is its `goal` task.
- **Offered only without `ways.toml`** (8.1): a repository with its own ways runs them alone, so a stock way's name is free for a project's own file.
- **The words are placeholders.** The next pass names and words them (section 12).

### 3.5 The gallery

- **Ways built into temper as references:** each a way's file, written as 3.1 to 3.3 say, or for a floor's recipe the settings it asks of the project's owners, with its words, its drawing and its tests. None runs anywhere until a project picks it (8.7). The stock three are its first entries.
- **The first ten** (the user's gallery of 2026-11-27):

  | Entry | What it shows | Needs |
  |---|---|---|
  | `pair` | you and one agent, "Land this", no review since you're there; one in place, by the agent's own sub-agent, if you ask | Pair's draft, for `copy` and `handed` |
  | `hand-it-over` | you describe it, an agent builds it, review and CI land it | this draft |
  | `owner-approved-milestone` | design first, goals on their own branches, each landing on an owner's approval; one milestone stricter | this draft |
  | `docs-fast-lane` | a floor's recipe: Markdown alone skips CI, code waits for it | today's rules by path |
  | `sensitive-paths` | a floor's recipe: any change touching named paths waits for two owners | today's rules by path |
  | `second-opinion` | the stock review and a second, by a lens of its own | a change's `gates` with `lens` |
  | `hotfix` | straight into main, first in line, at Ultrafast, decided and told after, CI still required | an executor's model and speed, `priority` |
  | `night-batch` | many small goals on a cheaper model, at low priority, landing in turns of four | an executor's model, `priority`, `share`, landing by turn |
  | `careful-newcomer` | every decision waits for its requester, away or not | `reaches` |
  | `standing-triage` | a standing triager starts hand-it-over work, and milestones only through a person | standing roles' grants |
  | `one-landing-milestone` | a milestone on a branch of its own, each goal landing into it and the branch caught up with main after each, the whole landing at once | a gathering holding a branch, `catch_up` |

- **Each entry is tested:** its files as a fixture, a world test running work through it in the focused and fuzzy suites, and its stops checked against temper's derivation of a way's path (`../domain/forge/change/branch-path.md`), so that the drawing shown is the derivation's, never drawn by hand.
- **Shown on the web,** on How work runs and at a project's creation, with the note that every variation of an entry is possible, and how.
- **Known to Ask temper:** a reference of every primitive a way is made of, what it does, its values, who sets it and where (section 5), from which Ask temper proposes any way that is possible, an entry of the gallery or not.

## 4. A piece of work's way

- **A start names a way, or takes a default.** `temper ctl chat start` on a repository, `change start`, `goal start` and `milestone new` take `--way NAME`, and a way's `start` task must fit the command: a conversation for `chat start`, a change for `change start`, an agent for `goal start`, a gathering for `milestone new`. A person's New offers the project's ways (section 12). A standing role starts a way within its grant (section 7). A start refuses a way that doesn't fit, and a way the project doesn't offer (8.1), naming why.
- **The default, by command:** `ways.toml`'s `default` for that command (8.1), else the first offered way that fits. With no `ways.toml`, the stock's way that fits. Kind 7's `way`, today a template's number, is read as the default for `goal start` and `milestone new` until the project has a `ways.toml`. Its `goals` and `changes`, which say what the rail offers, go to the next pass (section 12).
- **`--template` alone** still starts work as today, recorded with no way. With `--way`, the way's templates are used, and a `--template` beside it is refused.
- **What is recorded** with the work, at its start, in one step with its admission: the way's name, its source (the stock at this release, or the project's repository revision, 8.5), and its tasks' recipes as templates, each value resolved through section 5's layers. A change records its producer and gates' charters today (`../domain/forge/change/README.md`, section 1), and this records the rest beside them.
- **What the work starts takes its recipe from that record,** never by looking the way up again: a task the starting task delegates is made from the recorded template its `delegates` names, and a change's producer and gates from the change's recipe. A child may set a value of its own at its start, within section 5. Admission checks the caller's and the parent's authority and funds, as today. A task's `budget` is a request at its own start.
- **Running work keeps its way.** A change to a way, landed or decided, reaches new work only. An amendment of a parent's value reaches its children not yet started. Started ones keep theirs and are amended themselves. A floor's change reaches running work at its next step, as a landing rule's change does today (`../domain/forge/change/gates.md`, section 3).
- **Work started before ways** keeps running from what it stored: its targets, charters, routes and authority. It is shown as having no way. It is never matched to one after the fact.

## 5. Values in four layers

### 5.1 The layers

| Layer | Holds | Set by | Lowered from below? |
|---|---|---|---|
| the floor | the enforcing landing rules, kinds 1 and 2, rules by path among them; the path sets and their classes (kind 8's `sets`); the fixed `Wait` kinds; the policy's ceilings | the deployment's and the project's owners | never |
| the project's defaults | a value each way takes where it says none: the new kind *work defaults* for landing (5.3), and today's kinds for the rest (5.2) | the project's owners; the deployment's, for kind 3 | yes |
| the way | its members (section 3) | the stock, or a landing on the home repository (8.6) | yes |
| the work | its own values, set at its start or by amendment | its requester, within its grant (section 7) | — |

- **Below the floor, the most specific wins,** member by member: the work's own value, else its way's, else the project's default. Over the floor, nothing wins: it is judged as well, every time.
- **What reaches a person** takes the strictest class, as today (`../domain/forge/attention.md`, section 6): the work's classes from its layers, the path sets' classes, the fixed waits and Always ask. A way's `reaches` replaces the classes alone, never the path sets.
- **A budget is a request,** reserved against the live ceilings and funds at the work's start, as a start's `--budget` is today.
- **The user's example:** the project's default asks an owner's approval, Milestone's way sets none, and one milestone sets it again. That milestone's goals then wait for an owner at the landing branch, and every other milestone lands on its own. A rule by path asking an owner for any change touching `.temper/hooks/` holds for all three, as for any change.

### 5.2 Each value, by layer

| Value | Floor | Project default | Way | Work, today |
|---|---|---|---|---|
| CI, up to date | a rule's `ci`, `up_to_date` | work defaults | a change's `landing` | — |
| gates | a rule's blocking gate | work defaults | a change's `gates` | — |
| a person's approval | a rule's `approvals`, by branch or by path | work defaults | a change's or a branch's `landing.approvals` | — |
| what reaches the requester | the path sets' classes, the fixed waits | kind 8's classes | `[reaches]` | Always ask, a goal's |
| how questions travel | — | kind 6 | a change's `questions` | a change's `questions` |
| the charter each agent runs by | a floor's review: kind 3's reviewer (5.4) | kind 3's producer and reviewer, and the templates' charters | a task's `executor.agent` | a change's producer |
| models, efforts, speeds | the catalogue, and the policy's models | the charter's and the template's choices | a task's `executor` | a start's `--model`, `--effort` and `--speed`, a change's choices |
| priority, deadline | the template's and the policy's bounds | the template's | a task's `priority`, `deadline` | a start's `--priority`, `--deadline` |
| what a task may start | the policy's delegation | the template's | a task's `delegates` | — |
| when a task wakes | — | the charter's wake policy | a task's `wakes` | an amendment's |
| sharing a landing | the project's `by` and `batch` | the project's `combine` | a change's `share` | — |
| keeping a branch near its base | — | — | a branch's `catch_up` | a coordinator's catch-up |
| budget | the policy's ceilings | the template's budget | a task's `budget` | a start's `--budget` |

Each row already has its default and, mostly, its per-work choice. A way adds the middle layer, and the work gains the landing rows.

### 5.3 Work defaults, a new project kind

- **Its value:** two landing tables, as a change's `landing` (3.3): one for changes into a repository's landing branch, one for changes into a branch a task holds, for the project. The project's owners decide it, as they decide kind 2.
- **The stock's:** CI and up to date into the landing branch, the agent review on, and no approval, as stock rule 1 asks today. Into a goal's branch, neither `ci` nor `up_to_date`.
- **The floor or the defaults:** an owner who wants a value to hold whatever the way says puts it in a kind 2 rule. One who wants it as a starting point puts it here.

### 5.4 One set of criteria per change

- **The work's own rule:** a change's recorded `landing` values make one more enforcing rule, covering that change alone. It joins the floor's covering rules everywhere they are computed: the gates scheduled, the checks and reviews selected, the queue's and a turn's admission, requirements 5 and 6 at the merge, and the sign-offs counted (`../protocol/configuration/settings.md`, section 3; `../domain/forge/change/gates.md`, sections 1 to 3; `../domain/landing-turn.md`, section 1). It is the plan-added gates' generalisation, filled from the way.
- **It can only add.** Rules combine as today: any refusal wins, else any wait wins (`../domain/forge/change/gates.md`, section 2). The work's rule adds a gate, a wait or an approval, and never removes one the floor asks for.
- **A floor's review is the owners':** an agent review a floor rule requires runs kind 3's reviewer, the deployment's charter, never a charter from the repository, whatever the way's gates say. A way's own gate judges only the review the way adds.
- **A goal's direct writes:** the goal's agent writes its design straight into the goal's branch, outside any change (`../domain/forge/branches.md`, section 3). The `landing` of the changes into that branch covers them. The floor judges what the branch brings at its landing whole.
- **Who may set a work's value:** its requester, for its own work. A person may set any value the floor leaves free, lower than the way's included (the user, 2026-11-27). A standing role may set only what its grant allows (section 7). An amendment re-steps a change that is queued or being proved, and the evidence it had counts only where it was gathered for the same criteria at the same head.

### 5.5 The stock floor, and upgrades of it

- **For a new project,** stock rule 1 asks CI and up to date into each repository's landing branch, and the review lives in the work defaults. A pattern naming a repository's landing branch, whatever it's called, is new: a pattern names a branch by its name today.
- **For a project that exists,** today's stock rule 1, every branch with CI, up to date and gate 1, is written as the project's own kind 2 rule at the upgrade, beside its other rules, so that nothing it enforced loosens. An owner who wants ways to vary a value moves it to the work defaults by a decision.

## 6. Charters and their words

### 6.1 Five sources

| Source | Says | Read from | Changed by |
|---|---|---|---|
| the harness | tools, the brief's frame, the checkout, delivery, the budget | smith's and temper's code (smith's `domain/run-start.md`) | a release |
| the charter | how this agent works: its instructions and its parts | `.temper/charters/` of the home repository, whole, where it has `ways.toml` (6.2); else the stock charter's files (`stock-charters.md`, section 2), or the deployment's charter where it sets one whole | the home repository (8.6); else a release, or the deployment's owners |
| the deployment's lines | what its machines are, and its own rules | a deployment overlay of the charter (jig's `domain/engine/overlays.md`) | the deployment's owners |
| the repository's guide | how this code is written, for every agent and person | `AGENTS.md` at the root of each workspace directory, as smith finds it; others as it says (smith's `domain/run-start.md`, 1.3) | the repository |
| the task | its words, its brief and its messages | the task (`../domain/agent/charter.md`, section 1) | its requester, and the run |

- **All or nothing, for words as for ways** (the user, 2026-11-27): a repository with `ways.toml` holds every charter its ways' agents run by, whole. Nothing of the stock's text is read for it, and a charter a way names that has no file fails the landing check (8.6), never filled in from the stock. A repository without `ways.toml` runs the stock ways with the stock charters, as today.
- **What stays outside, on purpose:** the harness, which is code; the deployment's lines, which say what its machines are and differ from one deployment to the next, while the repository is the same on all; figures and authority, which are settings (2.1); and the reviewer behind the floor (5.4).
- **Sources, not an order:** smith lays out the prompt in its own order, instructions and brief before workspace guides (smith's `domain/run-start.md`). The table says where each word comes from.
- **Each source says what only it can say.** A rule said in two of them is a fault, which the preview shows (6.4).
- **A run with no workspace** has no guide.

### 6.2 The project's charters

- **A file per charter, named for it** (section 9): `.temper/charters/implementer.md`, `.temper/charters/reviewer.md`, `.temper/charters/lead.md`. A file holds the charter's whole text: its instructions, then its parts, each from a line `## part, on: REASONS` to the next, selected by its reasons as a stock part is (`stock-charters.md`, section 4).
- **Composed in place,** where a way's task names several files (3.2): their instructions joined in order, then their parts. A file shared by several charters, as the stock's `common.md` is, is one file named by each.
- **Words alone.** A file can't set models, tools, wakes or budgets, which stay settings (2.1).
- **A front matter**, between `+++` lines, in TOML:
  - `settings = "implementer"`: the deployment's charter whose settings it runs by, its tools, hosts, waiting, resume and default budget. Left out, the deployment's charter of the file's own name; a file with neither fails the landing check.
  - `format`: the file's format (8.8).
- **Written from the stock at the start:** picking ways at a project's creation (8.7), or `temper ctl charter copy NAME` (8.2), writes the stock's files whole. After that they are the project's, and a release's changes to the stock's text reach them only through Ask temper's review (section 11).

### 6.3 Carried as the project's charters

- **jig's charters gain a project scope:** a charter of the project's is its files' text at the project's repository revision (8.5), run by its `settings` charter's settings, with the deployment's overlay laid over it as over a stock charter.
- **A charter's identity gains the project:** the project, the charter's name, the repository revision, and the settings revision of its `settings` charter. Two projects' charters of one name are then two keys, and a record keeps them all.
- **Revisions as jig's are:** each new task takes the project's newest repository revision at its creation, and keeps it, as it keeps its settings revision (jig's `domain/engine/settings.md`, sections 3 and 4). A running task's words don't change. A goal's later changes take what has landed since.
- **The worker gets assembled text,** as today, and needs nothing new to run a project's words. The engine and jig resolve the scope.
- **Bounded at the read:** a project's charter counts as any charter does (jig's `domain/engine/settings.md`, section 8). A file that would take a charter past a bound fails the landing check (8.6), naming the file and its bytes. A release, a settings change or a limit's change that puts a charter over a bound refuses new tasks of that charter, naming it, until it fits.

### 6.4 Seen and recorded

- **A preview:** `temper ctl settings show charter NAME --project PROJECT [--reason REASON]` prints a charter's assembled instructions, each source headed by where it came from and its size. A page draws the same in the next pass.
- **Each run records what it read:** the release, the settings revision, the project's repository revision, and the guide's file and content hash, so that what a run was told can be read back while the run's records are kept.

## 7. Standing roles

### 7.1 Between a person and ways

- **A requester, as a person is:** a standing role starts work in a way, and the work's questions reach it. It decides, or asks its person, as today (`../domain/agent/standing.md`, section 3). A person talks with it as with any standing role (section 6 of that document).
- **It never signs off:** a way's `approval`, the work's or a floor rule's, asks people holding a project role, on the forge. A standing role's work waits for them as any work does (`../domain/forge/change/gates.md`, section 2).
- **Pairing on a person's behalf,** with a standing role as the requester of a pair's conversation, follows from the model and isn't designed here (section 14).

### 7.2 Kinds and instances

- **A kind is a named charter:** `standing-role`, `triager`, `prover` and `porter` in the stock (`../domain/agent/standing.md`, section 2), `drafter` for the design session's, and a deployment's own. Areas name a kind today as `charter N`, meaning the project's lowest-numbered live role of it (`../domain/areas.md`, section 2). They take its name too.
- **An instance is a live task:** its person, its words, its area, its budget a period and its Now, in the store. Instances come and go and are never in `.temper/`.
- **A kind's words in `.temper/charters/`,** `.temper/charters/drafter.md` for one, would be the project's words for a standing role, as for any charter. The user takes that up later (section 14). This draft keeps the place free.

### 7.3 Its grant

- **What it holds:** the ways it may start, by name; which of a work's values it may set and in which direction; and its spend, as today. It is kept with its task's authority (`../domain/agent/standing.md`, section 5). Whoever starts the role grants within their own. Today nothing lists what a role may start.
- **Checked at every start,** delegation and amendment the role makes, against the way as it resolves then. A way's file changes only by a landing on the home repository (8.6), so whatever lets a change land there decides what every grant naming the way allows from then on. A project that wants an owner to see each such change turns the sign-off on.
- **Until a role has a grant** (`ways.execution.md`), it starts work in the default way of the command it uses and sets no value.
- **A value outside its grant** isn't set: the role asks its person, as for a decision outside its rules.

### 7.4 Ways stop bringing standing roles

- **A recipe for one piece of work can't own a project's agent.** Draft first's triager (`../protocol/configuration/settings.md`, section 8; `standing-roles.md`, section 10) becomes a project's own standing role, started with `role start` or Team's "Hand an area to a role". A way names none. A project's triager stays as it is.
- **The next pass** decides where the stock suggests a triager, for one to a project that first runs Milestone (section 12).

## 8. `.temper/`

### 8.1 Its layout

```
.temper/
  ways.toml            which ways the project offers, in what order, and each command's default
  ways/<name>.toml     a way of the project's: new, or picked from the gallery
  charters/<name>.md   a charter of the project's, whole (6.2)
  hooks/               the repository's hooks and their timeouts (hooks.md)
```

- **`ways.toml`:** `format` (8.8), `offer`, the ways' names in order, each a file in `ways/`, and `[default]`, one offered way per start command (`change = "hand-it-over"`, `milestone = "milestone"`). Work already running keeps its way (section 4).
- **All or nothing** (the user, 2026-11-27): a repository with `ways.toml` offers its own ways alone, and the stock ways aren't offered there, so none of their names is reserved. One without `ways.toml` offers the stock ways, each the default for the command it fits. `ways.toml` is the switch, not `.temper/` itself, so that a repository with hooks alone still runs the stock ways.
- **Ways and charters are the project's,** from its home repository, the repository its path is drawn over (`../domain/forge/change/branch-path.md`, section 1). Hooks and the guide are each repository's own. The landing check refuses `ways/`, `ways.toml` and `charters/` in another of the project's repositories, so that a project's method has one home. A context repository's `.temper/` is never read.
- **What temper doesn't read leaves `.temper/`,** so that the directory says exactly what temper does.

### 8.2 Use, copy, remove, add

- **From the gallery:** picking an entry writes its file into `ways/`, whole, and names it in `offer`. It is the project's own from then on. `temper ctl way copy NAME` does the same from the terminal, and `temper ctl charter copy NAME` writes a stock charter's files into `charters/`, whole.
- **Change one:** edit its file.
- **Remove one:** delete its file and its name from `offer`.
- **Add one:** a file with a new name, which `offer` names.
- **No link back:** a file doesn't say where it came from. What a release changes in the gallery or in the stock charters reaches a project's files through Ask temper's review (section 11), for a person to decide.

### 8.3 A way's file

```toml
# .temper/ways/milestone.toml: the project's Milestone, as picked from the gallery and changed
format = 1
headline = "Design first; each goal is built on its own branch, and lands when it's done"
line = "You and an agent write a design; agents build each goal on its own branch, and each lands when it's done."
start = "milestone"

[tasks.milestone]
executor = "gathering"
gathers = "lead"

[tasks.lead]
contract = "report"
holds = ["branch"]
lands = "whole"
into = "base"
delegates = { change = { at_most = 40 } }
budget = 1000
landing = { approvals = [{ role = "owner", people = 1, freshness = "clean" }] }

[tasks.lead.executor]
model = "claude-opus-5-5"
effort = "xhigh"

[tasks.lead.executor.agent]           # a charter composed in place: one agent, its first run and its later wakes
instructions = ["common", "lead"]
parts = [{ on = ["first"], file = "designer" }, { on = ["question", "result"], file = "coordinator" }]

[tasks.change]
executor = "change"
into = "parent"
producer = "build"
gates = ["review", "safety"]
landing = { ci = false, up_to_date = false }

[tasks.build]
executor = { agent = "careful-implementer", model = "gpt-6.2-sol", effort = "high", speed = "ultrafast" }
contract = "change"

[tasks.review]
executor = { agent = "reviewer" }
contract = "verdict"

[tasks.safety]
executor = { agent = "reviewer" }
contract = "verdict"
lens = "data safety"

[reaches]
wording = "decide"
retry = "decide"
tests = "decide"
implementation = "ask"
dependency = "ask"
departure = "ask"
findings = "ask"
```

- **People's roles by name:** `role` names a project role as the project's policy does. Its form is the next pass's to settle with `temper ctl`'s.
- **Budget in the deployment's unit,** as a start's `--budget` takes it.
- **`careful-implementer`** is `.temper/charters/careful-implementer.md` with `settings = "implementer"`, and `common`, `lead`, `designer`, `coordinator` and `reviewer` are charter files of the project's (6.2), written from the stock's and changed.

### 8.4 Hooks

- **Moved to [hooks.md](hooks.md)** (the user, 2026-11-27): `deliver`, `finish`, `merge` and `close`, each run from its own branch's version, with timeouts in `.temper/hooks/config.toml` under a project ceiling. They are the repository's, for every way, and a way can't turn one off.
- **Found where they run,** in the job's checkout, not from the revision 8.5 keeps.

### 8.5 How temper reads it

- **Whole, from one commit, or not at all:** the forge reads `.temper/` of the home repository's landing branch at one tip, every file at that commit, within bounds of its own: files, bytes per file and in all. A read cut short, refused, or answered for another commit is no read. The CI workflows' directory is read at a landing tip in the same way, as a precedent for when, though not for how whole (`../domain/forge/adoption.md`, section 11).
- **When:** at adoption; at each move of the landing branch whose diff touches `.temper/`, or can't be told, an outside push and a rewind included; when the project's home changes; and at an engine's start, which reads again rather than trusts a mark held in memory.
- **Kept as the project's repository revision:** what a read gave, validated as the landing check validates it (8.6), with its commit, saved as a revision of the project's own that no person sets but by landing. A late answer for a commit older than the revision held is dropped.
- **The last good revision holds:** a read that fails or doesn't validate keeps the last good one, and the project's record says so, with the commit it couldn't take. Work started meanwhile takes the last good revision. A project with no good revision yet, or whose `ways.toml` was removed, runs the stock's ways, and one with no `charters/` no project words.
- **Hooks are found where they run,** in the job's checkout, not from the revision (8.4).

### 8.6 Guarded

- **Checked before it lands:** a change touching `.temper/` waits for a check of temper's own, by code, of the tree that will land: the change's head merged onto the landing branch's tip, and again at a turn's candidate. The files must parse; names must be valid and unique (section 9); every way named must exist and be offered; every task's executor, contract, resource and procedure must be temper's, and the guardrails of 3.1 must hold; every charter a way names must have its file, and each file its `settings` charter; charters must fit their bounds; a hook may be a symbolic link only to another hook beside it ([hooks.md](hooks.md), 3.1); every file must name a format the build reads (8.8); and no `ways/`, `ways.toml` or `charters/` may sit outside the home repository. A failure refuses with the file and its line, as a gate's verdict does.
- **Signed off, where the owners turn it on** (off by default, the user, 2026-11-27): a change touching `.temper/` then also waits for one owner's approval on the forge, a sign-off of temper's own beside the floor's rules, taking none of a project's rule or path set slots. It is a project setting, refused where no owner can sign, a playground's forge or a project with no owner on its forge.
- **Off, the floor alone guards `.temper/`:** ways can ask for less review than the project's defaults, charters shape what agents do, and hooks run code on workers, so a change an agent writes can edit them and land under the project's rules alone. A rule by path naming `.temper/` is how owners ask for a person meanwhile.
- **The floor stays in settings,** where landing can't reach it.

### 8.7 A new project's ways

- **At its creation, or its adoption,** a project whose home repository has no `ways.toml` is shown the gallery, each entry drawn, and the stock ways it would run meanwhile.
- **It may start on the stock ways,** writing nothing: they don't ask a person to buy in yet. Or it picks one or more entries, or writes a way from scratch, and may then change them before going on.
- **temper writes them by a change** into the home repository, "Add .temper/ with your ways": `ways.toml`, the ways' files, and the charter files their agents run by, with the stock's text (6.2). It lands by the project's own rules (8.6), so that the first thing temper does in a repository is a change a person can read. Until it lands, the stock ways run.
- **A starter is offered,** Hand it over preselected, so that the step is one click, with the advice to start small and grow, the whole gallery in sight.

### 8.8 Versioned files

- **Each file names its format:** `format = 1` in `ways.toml`, in each way's file and in `hooks/config.toml`, and in a charter file's front matter, where leaving it out reads as 1.
- **Every format shipped keeps reading:** a build reads each format temper has released, converting it as it reads. A file lives in a repository temper doesn't control, so this is longer than the store's rule, which keeps no code for a layout older than the deployment runs ([layouts-both-ways.md](layouts-both-ways.md)).
- **A newer format than the build knows** fails the read, naming the file and the format, and the last good revision holds (8.5).
- **Rewriting a file to the newest format** is a change a person asks for, or Ask temper proposes (section 11); nothing rewrites it by itself.
- **temper.toml and the other files a person writes** take the same rule, by [formats.md](formats.md).

## 9. Names

- **Every template, charter and policy has a name,** and a way the name of its file: 1 to 32 bytes of lowercase letters, digits and `-`, as an area's name (`../protocol/configuration/settings-kinds.md`, section 1). A display name stays beside it, for a person.
- **The number stays the store's key,** and what records keep, with `settings-numbers.md`'s ranges. People, `temper ctl` and files use names, and a number still reads wherever one does today (`--template 34`).
- **Reserved and unique:** a stock item's name is the stock's, and a deployment's own item can't take it. A deployment changes a stock item at its number and keeps its name. Within a kind, a name is unique in the deployment. A way's name is free: the stock ways are offered only where a repository has no `ways.toml` (8.1). So is a project's charter's: a file named as a deployment's charter is the project's whole charter of that name, run by that charter's settings unless its `settings` names another (6.2).
- **A task's name is its file's alone:** `[tasks.lead]` names nothing outside its way, and two ways may use the same task names.
- **A name maps to a path:** way `NAME` is `.temper/ways/NAME.toml`, and charter `NAME` is `.temper/charters/NAME.md`. A gallery entry's name is a suggestion, copied with its file.
- **Renamed by a decision,** for a deployment's item: records keep numbers, so a rename moves nothing that ran. A name given up can be taken again only by an item of the same kind.
- **The stock's names:**

| Kind | Names |
|---|---|
| templates | 1 `chat`, 2 `goal`, 4 `draft-goal`, 5 `ask-temper`, 6 `design-session`, 7 `standing-role`, 8 `milestone` |
| charters | 1 `chat`, 2 `implementer`, 4 `reviewer`, 5 `goal`, 9 `status`, 10 `trial`, 11 `ask-temper`, 12 `drafter`, 13 `standing-role`, 14 `triager`, 15 `prover`, 16 `porter`, and a new `pair` |
| stock ways | `pair`, once its draft lands, `hand-it-over`, `milestone`, offered without `ways.toml` |
| policies | 1 `stock` |

- **A deployment's own items,** at 33 to 64, take a name when set. One stored without a name reads with none until an owner names it, and is reached by its number meanwhile.
- **A project's own items take no number:** a way and a project's charter are keyed by project and name. A record of work that ran by one keeps the project, the name, the repository revision and its `settings` charter's number (6.3).

## 10. fabrig and dev.fabrig.ai

- **fabrig's repository:**
  - When fabrig picks its ways (`ways.execution.md`, section 2), its charters are written whole into `.temper/charters/`: the stock's text with `temper/docs/charters/fabrig/`, fabrig's overlay (`stock-charters.md`, section 5), merged in, a file per charter. Its settings fields, models among them, stay the deployment's.
  - Its common file's rules move to `AGENTS.md`, which already says most of them. Its rule on stored and wire versions contradicts `AGENTS.md`'s fingerprinted layouts and goes.
  - Its scripts move to hooks as [hooks.md](hooks.md), section 6, says.
  - Its own `ways.toml` and ways, once the project can run them: fabrig picks its ways from the gallery as any project does, a step of the user's (`ways.execution.md`, section 2).
- **dev.fabrig.ai,** by hand, ops' steps, with the release that carries each increment:
  - its machines' lines stay a deployment overlay, applied today by a script kept outside this repository;
  - its lean goal, a deployment template and charter that copy charter 5 whole with edits a script applies, becomes fabrig's own way and charter files, written whole, which Ask temper's review keeps current with the stock. Before that, the step checks the deployment's template and charter as they stand, and keeps as settings whatever the files can't say: models, efforts, wakes and budgets;
  - the project's `merge_job` and `close_job` give way to the hooks ([hooks.md](hooks.md)).

## 11. Upgrades

The rules every stored value and wire follows (`../protocol/wires.md`):

- **The store:** work recorded without a way reads as having none (section 4). Settings without names read with the stock's names at the stock's numbers (section 9). A project's existing rules stay its floor, today's implicit stock rule 1 written out as its own (5.5).
- **The engine and jig:** a charter with a project's scope resolves in the engine. The worker gets assembled text, as today, so an older worker runs it. A worker whose layout lacks something a run needs is refused by name, and the run waits for one that has it, as `../protocol/wires.md`, sections 5 and 8 say.
- **Hooks:** [hooks.md](hooks.md), sections 4 and 5.
- **A release's changes reach a project's own files by review** (the user, 2026-11-27): after an upgrade, Ask temper reads the project's `.temper/` beside what the release changed in the stock ways, the gallery and the stock charters, infers what the project would want of it, and proposes it as a change, each edit with its reason. A person decides, edit by edit. Nothing rewrites a project's ways or words by itself: prompts are sensitive, and a file the project changed says what it wanted.
- **Files' formats** keep reading, as 8.8 says.
- **Clients** at another layout are refused by name and reload or upgrade, as `../protocol/wires.md`, section 9 says.

## 12. For the next pass, how a person meets it

What this pass leaves to the pages, the terminal and the words:

- **Picking a way:** New's menu as the project's ways (`template-places.md`'s offers become ways'), the terminal's `--way`, and Ask temper suggesting one from a request.
- **How work runs:** the project's ways, its floor and its defaults, in place of one displayed way and its path (`../client/pages/settings.md`, section 2). `see ways` and Ask temper's picture change with it.
- **The preview** of a charter's words (6.4), and a way's file shown beside what it changes.
- **The stock ways' names and words** (3.4), You approve as a switch, and where the stock suggests a triager (7.4).
- **The gallery's page** (3.5) and a new project's step (8.7): the drawings, the starter, and how a picked entry is changed before it's written.
- **Ask temper's review after an upgrade** (section 11): how its proposal reads, edit by edit.
- **The rail's entries** that kind 7's `goals` and `changes` say today (section 4), and how a project with work of no way shows it.

## 13. Open questions

None left from the second pass.

**Settled by the user, 2026-11-27:** whoever starts a piece of work may set any value the floor leaves free, lower than its way's included, as no approval for one milestone whose way asks for one (5.4); `ways.toml` is the all-or-nothing switch (8.1); the sign-off on `.temper/` is off by default (8.6); the stock ways a repository without `ways.toml` runs are Pair, once its draft lands, Hand it over and Milestone (3.4); a deployment's own ways, a milestone across repositories and a hotfix cancelling a turn wait (section 14). A change first in line waits for a turn in progress.

## 14. Later

- **Sharing ways across projects** (the user, 2026-11-27): copying the files over, at first.
- **A milestone across repositories** (the user, 2026-11-27): its way from the home repository, and each goal's hooks and guide its own repository's, as a first thought.
- **A deployment's own ways,** offered beside the stock ones without a repository's `.temper/` (the user, 2026-11-27: not for now).
- **A hotfix cancelling a turn in progress,** at the cost of the turn's CI (the user, 2026-11-27: not for now).
- **Comparing and reconciling** a project's files with the gallery's, beyond Ask temper's review after an upgrade (section 11).
- **Who hears and when, per way:** who holds a work's escalations, its progress cadence and its push, until `what-reaches-you.md` settles them.
- **Moving between ways mid-work** (the user, 2026-11-26): Pair's "hand it over" starting a Hand it over, and an implementer proposing a milestone for a change too big.
- **Standing roles in `.temper/`** (the user, 2026-11-26): a kind's words in `charters/`, and saying that one instance is of a kind, with many of a kind coming and going.
- **A standing role pairing on its person's behalf** (7.1).
- **The forge's own protection** for a rule by path, and for `.temper/`'s sign-off where it is on, stays `what-reaches-you.md`, section 7's open question.
