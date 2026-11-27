# A way's recipes

Split from [ways.md](ways.md), its section 3, on 2026-11-27, and part of that draft: how a way is written, as task recipes in jig's terms, with its changes and landing, the stock three as compositions, and the gallery. Its status is its own, and its execution is [ways.execution.md](ways.execution.md)'s.

## 1. What a way's file holds

| Member | What it says |
|---|---|
| its name | its identifier (`ways.md`, section 9): the file's name in `.temper/ways/`, or a stock way's name where the repository has no `ways.toml` |
| `format` | the file's format (`ways-temper-dir.md`, 8) |
| `headline`, `line` | its words: a heading of at most 120 bytes and a sentence |
| `start` | the task a person, a script or a standing role starts with this way, by its name among `tasks` |
| `[tasks.NAME]` | each kind of task the way's work is made of (2), named by the project |
| `[reaches]` | what reaches its requester, written out (3) |

`ways.toml` holds, beside the list of ways, the project's `[defaults]` across them (`ways.md`, 5.3).

- **In jig's terms** (jig's `domain/model.md`, section 3): a task has a spec, a result contract, a requester, an executor, an authority, a wake policy and the resources it holds. A way's tasks say those, and temper's connector adds its own: branches and copies to hold, the change procedure, the closer, landing's requirements, and what reaches a person.
- **Names are the project's.** `start = "milestone"` and `[tasks.lead]` name nothing temper knows. temper knows only the executors, contracts, resources and procedures a task names.
- **What temper keeps, over every way:**
  1. **Only temper's procedures write a landing branch:** an agent's work reaches one only as a change (3), and an agent writes only what it holds.
  2. **The floor always holds** (`ways.md`, section 5): a way adds requirements to a change, never removes the floor's.
  3. **Authority only narrows:** what a task may start, its tools and its spend are checked against the project's policy and the starter's role at every start, as jig checks any delegation (jig's `domain/authority.md`). A way can't grant.
  4. **Agents judge, people approve:** a gate is a task with a verdict, and an approval is a person's requirement on the forge, which no agent meets.
  5. **Words in charters, figures in settings:** a charter's words live in its files (`ways-charters.md`), models come from the catalogue, ceilings from the policy.
  6. **Bounded:** at most 8 kinds of task in a way, and a tree at most 4 deep, its starting task the first level.
- **A file that breaks one** fails the landing check (`ways-temper-dir.md`, 6), naming the file, its line and the rule.

## 2. A task's recipe

| Member | For | What it says | Left out |
|---|---|---|---|
| `executor` | every task | who carries it out: `{ agent = CHARTER, settings, model, effort, speed, sub_agents, later }`, `"change"`, `"gathering"` or `"person"` | required |
| `contract` | an agent, a person | what counts as done: `report`, `verdict` or `change` | an agent's `report`, a person's `verdict` |
| `labels` | a verdict | its closed list | `approve`, `request-changes` |
| `lens`, `advisory`, `freshness` | an agent judging as a gate (3) | what it judges by; whether its verdict only advises; `exact` or `clean` | none; blocking; `clean` |
| `holds` | an agent, a gathering | `branch`, a branch of its own; `copy`, a writable copy of its own, an agent's alone | none |
| `conversation` | an agent | whether its requester talks with it | `false` |
| `delegates` | an agent | which of the way's tasks it may start, each with `at_most` | none; `at_most` left out is the policy's task bound |
| `wakes` | an agent | when it runs: `results`, `questions`, `news`, `words`, `timers` | the charter's wake policy |
| `budget`, `deadline`, `priority`, `tracked` | every task | its spend asked for, a span from its start in whole seconds, its place in its queue, and whether it is tracked | the template's |
| `lands`, `into`, `landing`, `gates`, `share`, `catch_up` | a task holding a branch | how its branch lands (3) | `"whole"`, `"base"`, the defaults (`ways.md`, 5.3), `"when-needed"` |
| `gathers` | a gathering | which of the way's tasks it holds | required |
| `into`, `producer`, `gates`, `landing`, `questions`, `share` | a change | 3 | 3 |

- **Left out, false, empty:** a member left out takes the layer below it (`ways.md`, section 5). `false`, `[]` and `0` are values, the way's own.
- **Derived, not written:** what jig needs that a recipe implies. A task holding a branch is tracked and ended by its closer, as a goal's template is today (`../domain/forge/branches.md`, sections 1 and 2); one in a conversation is listed; a gathering is ended by its requester; a contract's bounds are the stock's for its kind (jig's `domain/tasks/README.md`, section 3). The landing check refuses a recipe whose members contradict: a `copy` on a gathering, `lands` without a branch, `gathers` on an agent.
- **An agent's charter** is a charter's name (`ways.md`, section 9), or one composed in place, as jig holds a charter: `agent = { instructions = ["common", "lead"], parts = [{ on = ["first"], file = "designer" }, { on = ["question", "result"], file = "coordinator" }] }`, each file a charter file of the project's (`ways-charters.md`, 2). It runs by the settings `settings` names, else those of its first file's `settings`. The stock goal charter's designer and coordinator are its parts so: one agent, its first run and its later wakes.
- **Its model** is the catalogue's, by name, with an effort and a speed (`model-speeds.md`); one the deployment doesn't hold is ignored, and the preview says so (`ways-charters.md`, 4).
- **Resolved as templates:** the engine makes each of a way's tasks a template of the project's own, at its repository revision (`ways-temper-dir.md`, 5), whose executor, contract, budget, deadline, tracking, flags, model choice and resources are the recipe's (jig's `protocol/configuration-templates.md`, section 1). Work started in the way is made from those templates.
- **Admitted by recipe** (the review's first finding): jig's templates are defaults a client fills, and its admission checks executors, charters, resources, spend, count and depth, which can't tell two recipes of one charter apart (jig's `domain/authority.md`, sections 3 and 6). So the engine stamps each task made in a way with its way, its recipe's name and its repository revision, and admits a creation only along an edge of the way: its creator's recipe names the new one's in `delegates`, within `at_most`, counted over the creator's children of that recipe, live and ended. The same check holds for a batch, an accepted proposal, an amendment that adds children, and a tool's call; the `goal` and `change` tools name a recipe. A creation along no edge is refused, naming the way and the recipes.
- **A procedure's own children aren't edges:** a change's producer, gates, repairs, resolutions and catch-ups, and a closer's close change, are the procedure's, made as today and counted toward the task and depth bounds as today. The way's bound of 4 deep (1) counts its own tasks alone.
- **Which revision a piece of work reads** (the review's fourth finding): the repository revision it started with, for its recipes and its charters, in every task it starts. An amendment moves it to the newest, as jig moves a task to a newer settings revision (jig's `domain/engine/settings.md`, section 4), refused where the newest lacks a recipe or a charter the work uses. Authority is checked live at every start.
- **What a run can start** is what its tools can: an agent starts a change with `change` and another of the way's tasks with `goal` where its charter grants that tool (`../domain/agent/tools.md`); a change's procedure starts its producer and gates itself.
- **What isn't a way's:** a start that changes no repository is no way's and keeps its template: a chat with no copy of its own, Ask temper, a design session, a standing role.

## 3. Changes, landing and what reaches

A task with `executor = "change"` is temper's change procedure, code and no agent:

- `into`: `"parent"`, the branch its requester holds, or `"base"`, the repository's landing branch.
- `producer`: which of the way's tasks writes it, an agent with `contract = "change"`; or `"handed"`, its requester's own edits, handed over, as a pair's "Land this" hands its copy over (`../domain/forge/change/opened.md`).
- `gates`: which of the way's tasks judge it, each an agent recipe with `contract = "verdict"` and its `lens`, `advisory` and `freshness` (2). `[]` is none.
- `landing`: `ci` and `up_to_date`, booleans, and `approvals`, a list of `{ role, people, freshness }`, as a landing rule's own fields without the pattern and path sets that say which changes a rule covers (`../protocol/configuration/settings-kinds.md`, section 1). With `gates`, it makes the work's own rule (`ways.md`, 5.4).
- `questions`: `up` or `none`, how its producer's questions travel.
- `priority`: its place in its queue. A queue orders changes by their goal's priority today (`../domain/forge/queues.md`); a change with a recipe's `priority` is ordered by its own, and one with no goal by its own too.
- **`share`, built last** (`ways.execution.md`): how the change shares a landing, in a project that lands by turn (`../domain/landing.md`, section 1): `alone`, a turn of its own; or `with`, the ways whose changes may share a turn with its own, and `at_most`, how many; and `combine`, `merge` or `squash`. A turn takes the waiting units in queue order while every change of the next unit allows the turn's ways, at most the smallest `at_most` among them; a unit that doesn't fit ends the turn's packing, and the next turn takes it. Each member's `combine` reaches the prepare, temper's and a project's own. Queue against turn stays the project's; a project landing by queue lands every change alone.

A task holding a branch lands it whole, by temper's closer, as a goal's branch lands today:

- `lands = "whole"`, `into` `"parent"` or `"base"`, and its own `landing`, `gates` and `share` for the branch's landing, which is a change as any other.
- **`catch_up`, built late:** `when-needed`, the default, a coordinator asking for a catch-up for a change that needs what the base gained (`../domain/forge/updates.md`, section 1); or `after-each-landing`, temper making one itself, a merge of the base's new tip into the branch after each landing into the base. A conflict in it goes to the merge hook ([hooks.md](hooks.md)), then to a resolution.
- **A gathering may hold a branch** and land whole (the user, 2026-11-27): its gathered tasks land `into = "parent"`, its branch. It then gets a closer of its own, as a goal does, and jig's gathering waits for it before it concludes, where today it concludes once its last task ends (jig's `domain/tasks/gatherings.md`). What landed into the branch lands, a failed or cancelled task's work never having reached it. Built late. A gathering holding none, the default, lets each of its tasks land on its own, which keeps drift from the base small.

`[reaches]` is the classes table of what reaches the requester, written out (`../domain/forge/attention.md`, section 6): each movable kind, `wording`, `retry`, `implementation`, `tests`, `dependency`, `departure` and `findings`, at `decide`, `ask` or `ask-away`; `[[reaches.narrow]]` items, each a kind, words of at most 120 bytes and a class, narrow one ("fixes that keep the wire format"). Left out, the way takes the defaults' table (`ways.md`, 5.3). Present, the table is whole, and a kind it leaves out asks even when away. Money, irreversible acts and rules always wait, and the path sets' classes stay the floor's. The kinds are temper's, the same for every project and tied to no way of working: how a project designs, drafts or records its work is its charters' business, which temper doesn't read. A project's own nuance goes in `narrow` words.

## 4. The stock three, as compositions

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
gates = []                           # no review: you're there
landing = { ci = true, up_to_date = true }
[reaches]                            # present and empty: everything asks, even when you're away
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
gates = []                           # Draft first's landing: no review
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

- **Their `[reaches]`:** Pair's table is present and empty, so every decision asks even when away, as You approve's table does today. Hand it over's and Milestone's write out today's Agents review and Draft first table (`../domain/forge/attention.md`, section 6): `wording`, `retry` and `tests` at `decide`, the rest at `ask`, and `findings` narrowed to `decide` for "a repair goal for a routine red after landing".
- **You approve becomes a value,** an approval with one owner in a change's `landing`, which any layer sets (`ways.md`, section 5), rather than a way of its own. Agents review becomes Hand it over, and Draft first becomes Milestone, with the same landing.
- **A goal alone is a milestone of one,** started the same way, or a way whose `start` is its `goal` task.
- **Offered only without `ways.toml`** (`ways-temper-dir.md`, 1): a repository with its own ways runs them alone, so a stock way's name is free for a project's own file.
- **The words are placeholders.** The next pass names and words them (`ways.md`, section 12).

## 5. The gallery

- **Ways built into temper as references:** each a way's file, written as 1 to 3 say, a `ways.toml` default (`ways.md`, 5.3), or for a floor's recipe the settings it asks of the project's owners, with its words, its drawing and its tests. None runs anywhere until a project picks it (`ways-temper-dir.md`, 7). The stock three are its first entries.
- **Its entries' full files** are [ways-gallery.md](ways-gallery.md)'s.
- **The first ten** (the user's gallery of 2026-11-27):

  | Entry | What it shows | Needs |
  |---|---|---|
  | `pair` | you and one agent, "Land this", no review since you're there; one in place, by the agent's own sub-agent, if you ask | Pair's draft, for `copy` and `handed` |
  | `hand-it-over` | you describe it, an agent builds it, review and CI land it | this draft |
  | `owner-approved-milestone` | design first, goals on their own branches, each landing on an owner's approval; one milestone stricter | this draft |
  | `docs-fast-lane` | a `ways.toml` default across ways: a change touching only documentation skips CI, any other waits for it | a default rule `unless_only` a set (`ways.md`, 5.3), which path sets can't say today |
  | `sensitive-paths` | the floor's: any change touching named paths waits for two owners, which no landing changes; or a `ways.toml` default, where the project accepts that a landing can | today's rules by path |
  | `second-opinion` | the stock review and a second, by a lens of its own | a change's `gates` with `lens` |
  | `hotfix` | straight into main, first in line, at Ultrafast, decided and told after, CI still required | an executor's model and speed, `priority` |
  | `night-batch` | many small goals on a cheaper model, at low priority, landing in turns of four | an executor's model, `priority`, `share`, landing by turn |
  | `careful-newcomer` | every decision waits for its requester, away or not | `reaches` |
  | `standing-triage` | a standing triager starts hand-it-over work, and milestones only through a person | standing roles' grants |
  | `one-landing-milestone` | a milestone on a branch of its own, each goal landing into it and the branch caught up with main after each, the whole landing at once | a gathering holding a branch, `catch_up` |

- **Each entry is tested:** its files as a fixture, a world test running work through it in the focused and fuzzy suites, and its stops checked against temper's derivation of a way's path (`../domain/forge/change/branch-path.md`), so that the drawing shown is the derivation's, never drawn by hand.
- **Shown on the web,** on How work runs and at a project's creation, with the note that every variation of an entry is possible, and how.
- **Known to Ask temper:** a reference of every primitive a way is made of, what it does, its values, who sets it and where (`ways.md`, section 5), from which Ask temper proposes any way that is possible, an entry of the gallery or not.
