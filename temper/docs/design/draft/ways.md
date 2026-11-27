# Ways of working, several at once

Draft, 2026-11-26, from sessions with the user on 2026-11-25 and 2026-11-26. It started from how agents' prompts are assembled and what a repository's `.temper/` holds, and what a good story for both would be. The user added hooks named by the moment they run at, a ladder from a person working with one agent, where "temper adds only the tool calls", to a whole milestone, and then the frame this draft takes: "I'd like temper to support various 'ways' of working at the same time for the same project", as in "for this change I want to pair directly with an agent, for this other I want to delegate and I want a automatic review, for this other one I want a milestone". temper offers a few stock ways that a project uses as they are, changes by copying, removes, or adds to with ways of its own. The settings only owners change are "overridable per-way and possibly per-change", as in "milestone way does no require manual approval", but "this one milestone requires approval". Path rules still hold for any change, whatever its way. Standing roles "sit between the person and ways". Settings get a name beside their number, "mostly an internal detail", and the name maps to `.temper/`.

Second pass, 2026-11-27, from a session with the user that day, over a survey of temper's primitives and a gallery of ways drawn from them. The user's frame: make the whole gallery possible, "eventually (it can be a series of incremental drafts)", built into temper, each way tested, shown in the web UI with the note that every variation is possible, and every primitive explained to Ask temper "so he can propose anything that is possible". A new project picks its ways, from the gallery or from scratch, and they populate its `.temper/`: "no magic, expecially around prompts". `.temper/` is "all or nothing": present, it's authoritative in full; absent, the stock ways run. Its formats are versioned, so that what is decided now keeps being supported. After an upgrade, Ask temper reviews a project's ways and proposes changes "in an intelligent way", "not mechanically", since "prompts are very sensitive". Sharing ways across projects and reconciling drift are left for later. The landing part goes last. The hooks moved to a draft of their own, [hooks.md](hooks.md).

The first pass covered the mechanics and the whole picture, so that each piece is built in turn and they still fit together; the second settles the vision the increments build towards (`ways.execution.md`). How a person meets it, in pages, words and the terminal, is the next pass (section 12). It revises a project's single stock way (`../protocol/configuration/settings.md`, section 8), standing roles brought by a way (`standing-roles.md`, section 10), kind 7's one preferred goal template (`../protocol/configuration/settings-kinds.md`, section 1) and the charter assembly's sources (`stock-charters.md`, sections 3 and 4). It takes up `stock-charters.md`, section 11's project overlay from `.temper/`, and adds names to `settings-numbers.md`'s numbered kinds. It wins where they differ until its plans fold it in.

## 1. In one page

- **Three homes, one rule** (section 2). `temper.toml` holds what a machine runs. The store's settings hold what grants and bounds: people, accounts, models, budgets and the rules that always hold. A repository's `.temper/` holds how work is done on that code: its ways, its charters' words and its hooks. Nothing lives in two of them.
- **A way is a recipe for one piece of work** (section 3): its shape (pair, change, goal or milestone), the charter, model, effort and speed each of its agents runs by, its priority and deadline, how its work lands, with the reviews and approvals it adds, what reaches its requester, and its budget. Last, it can say how its changes share a landing. The stock has three: Pair, Hand it over and Milestone.
- **New work picks its way when it starts and records it** (section 4). Its children take their part of it. Work started before ways runs as it did.
- **Values in four layers** (section 5): the floor, the project's defaults, the way, and the piece of work. The floor is the landing rules, rules by path among them. It always holds, and no way or piece of work lowers it. Below the floor, the work's own value wins over its way's, and its way's over the project's default. One calculation gives a change's criteria wherever they are used.
- **A charter's words come from six sources** (section 6): smith's harness, temper's method, the deployment's lines, the repository's `AGENTS.md`, the project's words from `.temper/charters/`, and the task's words. A run records which it read.
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
   the way: shape · charters · landing · what reaches · budget (section 3)
                    │
   each agent's words: harness · temper's method · the deployment's lines
                       · AGENTS.md · .temper/charters · the task's words (section 6)
                    │
   landing: the floor, always, and the work's own rule
            (the project's defaults ◀ the way ◀ the work)      (section 5)
                    │
   the repository's hooks: deliver · finish · merge · close    (hooks.md)
```

- **Orthogonal by construction:** a way doesn't know who started its work, and a standing role starts work in any way its grant allows. The floor holds whoever the requester and whatever the way.
- **temper's code stays the mechanism** (LLM for judgement, code for mechanisms): a way chooses one of temper's shapes and fills it in. It can't make a new flow.

### 2.3 What there is today

From a read-only survey of main at `de275134b0`, 2026-11-26:

- **One displayed way per project.** The stock ways, You approve, Agents review and Draft first, are pre-made settings a project applies with "Use this": landing rules, what reaches its people and, for Draft first alone, the goal template it prefers and the triager. Settings a way names none of keep their values (`../protocol/configuration/settings.md`, section 8). How work runs, `see ways` and Ask temper show the first stock way whose path equals the project's, or none. That is a display, not a choice any running work reads.
- **Per piece of work already:** a change's producer, its models and its `questions` route (`../domain/agent/tools-change.md`, section 1). A goal's template, models and budget at `goal start`, and the goal page's Always ask, kept as the spec's line `Asks: always` (`../client/pages/goal.md`, section 3). A milestone's budget, deadline and priority, as its task's, and no landing or reaching settings of its own (`../domain/milestones.md`, section 3).
- **Landing rules cover branches and paths.** Kinds 1 and 2 match the branch a change lands on. Every matching enforcing rule holds, and the project's add to the deployment's. A project's rule naming path sets covers only the changes that touch them, whatever the change, and its approval is a person's review on the forge (`what-reaches-you.md`, section 7; `../domain/forge/change/gates.md`, section 2). A plan may add gates to one change (`../domain/forge/change/gates.md`, section 1).
- **Charters are deployment-wide.** Kind 3 names the producer and the reviewer for every project. A change records their numbers, and jig resolves each new delegate's charter at the newest settings revision (jig's `domain/engine/settings.md`, section 3). A deployment sets a charter whole or lays an overlay over a stock one (jig's `domain/engine/overlays.md`). No `.temper/` directory is imported as charters or settings. A run reads repository text from the documents its charter names (`../protocol/configuration/settings.md`, section 6) and from the workspace's guide (`../domain/agent/charter.md`, section 1).
- **`.temper/` holds scripts.** `pre-pr` is the delivery check, by a convention temper's code gives smith (`../domain/agent/charter.md`, section 4). `merge` and `checked-close` run because the project's `merge_job` and `close_job` name them, and those settings' presence also switches the merge job and the close change on (`../domain/landing-settings.md`, 1.1). `close` is called by `checked-close`. `goal-pre-pr` and `goal-child-pre-pr` are wrappers for CI.
- **Numbers, no names:** a template has a display name, and a charter has none. A record refers to each by number (`settings-numbers.md`, section 3).

## 3. A way

### 3.1 What it holds

| Member | What it says |
|---|---|
| its name | its identifier (section 9): the file's name in `.temper/ways/`, or a stock way's name where the repository has no `ways.toml` |
| `headline`, `line` | its words: a heading of at most 120 bytes and a sentence, as the stock ways' are today |
| `shape` | which of temper's flows its work runs (3.2) |
| `charters` | for each agent its shape runs, the charter by name (section 9) |
| `models` | for each agent its shape runs, a model, its effort and its speed, by name (`model-speeds.md`); one the deployment doesn't hold is ignored, and the preview says so (6.4) |
| `priority`, `deadline` | the work's place in the queue and its deadline, a span from its start, within its template's and policy's bounds |
| `landing` | how its work lands: `ci`, `up_to_date`, `review`, the `reviews` and `approvals` it adds, and, last, how its changes share a landing, for each branch its shape lands on (3.3) |
| `reaches` | the classes of what reaches its requester: a stock table by name, or classes of its own (`../domain/forge/attention.md`, section 6) |
| `questions` | how its changes' questions travel, `up` or `none`, as a change's own choice says today |
| `budget` | the budget its piece of work asks for at its start |
| `format` | the file's format (8.8) |

- **Every member but `shape` is optional.** One left out takes the layer below it (section 5). One set, `false` and `none` included, is this way's value.
- **Per member, not whole:** `landing.review` left out inherits while `landing.approval` set is the way's.

### 3.2 The shapes

| Shape | Its work | Its agents | Where it lands |
|---|---|---|---|
| `pair` | a chat whose one agent works in its own copy of the repository, with its requester there; "Land this" opens its change | `pair` | the landing branch |
| `change` | one change, from its requester's words, written unattended | `implementer`, `reviewer` | the landing branch |
| `goal` | a goal whose agent designs, then has changes made into the goal's own branch, which lands whole | `goal`, `implementer`, `reviewer` | each change into the goal's branch, then the branch into the landing branch |
| `milestone` | a milestone of goals, each run as `goal` is | as `goal` | as `goal`, for each goal |

- **A shape is code:** `pair` is the chat working in its own copy, designed in a draft of its own that hasn't landed yet. `change` is the forge's change (`../domain/forge/change/README.md`), `goal` is a goal on its own branch with its closer (`../domain/forge/branches.md`), and `milestone` is the gathering (`../domain/milestones.md`).
- **`implementer` is kind 3's producer slot,** named for what the agent does.
- **A person starts a `change` from words,** which `temper ctl` can't do today: a person reaches a change through a chat that delegates one, or by pushing a branch (`../domain/forge/change/opened.md`). A change opened from a pushed branch has no producer, and takes the `change` way's landing alone.
- **A `goal` table** carries kind 11's two values, `design` and `close_out`, which describe the goal's path for the pages and Ask temper (`../protocol/configuration/settings-kinds.md`, section 1). The goal's charter directs the work, as it does today.
- **What isn't a way's:** a start that changes no repository is no way's and keeps its template: a chat with no copy of its own, Ask temper, a design session, a standing role. Whether every chat on a repository is a `pair` is the Pair draft's question.

### 3.3 Its landing

A way's `landing` is a landing rule's own fields, without the pattern and path sets that say which changes a rule covers (`../protocol/configuration/settings-kinds.md`, section 1). Its work covers itself:

- `ci` and `up_to_date`, as a rule's;
- `review`: whether the agent review gate blocks, the stock's gate 1, with its freshness;
- `approval`: none, or `{ role, people, freshness }`, as a rule's `approvals`: how many people holding a project role sign off on the forge. It is always people's review, never an agent's word.
- `reviews`: review gates the way adds beside the stock's, each a lens and the charter that judges by it, blocking or advisory, as a gate is (`../domain/forge/change/gates.md`, section 1): `{ lens = "data safety", charter = "reviewer", blocking = true }`.
- `approvals`: approvals the way adds beside `approval`, each as it is: two roles, or more people.
- **`share`, built last** (`ways.execution.md`): how the work's changes share a landing, in a project that lands by turn (`../domain/landing.md`, section 1): `alone`, a turn of its own; or `with`, the ways whose changes may share a turn with its own, and `at_most`, how many; and `combine`, `merge` or `squash`. A turn takes the waiting changes whose ways allow one another, in queue order. Queue against turn stays the project's, since who merges and how CI is triggered are the repository's; a project landing by queue lands every change alone.
- **Added, never removed:** `reviews` and `approvals` only add to what the floor asks (5.4). `share` changes how a landing is formed, never what it is judged by.

A shape with two branches gives two tables: `landing`, into the repository's landing branch, and `landing.goal`, a change into a goal's own branch. A member `landing.goal` leaves out takes the layer below's `landing.goal`, never the way's own `landing`.

### 3.4 The stock three

| Way | Shape | Headline | Landing | Reaches |
|---|---|---|---|---|
| `pair`, Pair | `pair` | "You and one agent, in its own copy" | `review = false`, `approval = none`: its requester is there and says "Land this" | You approve's table: everything asks |
| `hand-it-over`, Hand it over | `change` | "An agent writes it, another reviews it, and code lands it" | `review = true`, `approval = none` | Agents review's table |
| `milestone`, Milestone | `milestone` | "Design first, then agents build it on its own branch, and it lands whole" | Draft first's: `landing` with `ci` and `up_to_date`, no review and no approval; `landing.goal` with neither `ci` nor `up_to_date` | Draft first's table |

- **You approve becomes a value,** `approval` with one owner, which any layer sets (section 5), rather than a way of its own. Agents review becomes Hand it over, and Draft first becomes Milestone, with the same landing.
- **A goal alone is a milestone of one,** started the same way. A project wanting a `goal` way of its own writes one (8.2).
- **Offered only without `ways.toml`** (8.1): a repository with its own ways runs them alone, so a stock way's name is free for a project's own file.
- **The words are this pass's placeholders.** The next pass names and words them (section 12).

### 3.5 The gallery

- **Ways built into temper as references:** each a way's file, or for a floor's recipe the settings it asks of the project's owners, with its words, its drawing and its tests. None runs anywhere until a project picks it (8.7). The stock three are its first entries.
- **The first ten** (the user's gallery of 2026-11-27):

  | Entry | What it shows | Needs |
  |---|---|---|
  | `pair` | you and one agent, "Land this", the agent review | Pair's draft |
  | `hand-it-over` | you describe it, an agent builds it, review and CI land it | this draft |
  | `owner-approved-milestone` | design first, goals on their own branches, each landing on an owner's approval; one milestone stricter | this draft |
  | `docs-fast-lane` | a floor's recipe: Markdown alone skips CI, code waits for it | today's rules by path |
  | `sensitive-paths` | a floor's recipe: any change touching named paths waits for two owners | today's rules by path |
  | `second-opinion` | the stock review and a second, by a lens of its own | `reviews` |
  | `hotfix` | straight into main, first in line, at Ultrafast, decided and told after, CI still required | `models`, `priority` |
  | `night-batch` | many small goals on a cheaper model, at low priority, landing in turns of four | `models`, `priority`, `share`, landing by turn |
  | `careful-newcomer` | every decision waits for its requester, away or not | `reaches` |
  | `standing-triage` | a standing triager starts hand-it-over work, and milestones only through a person | standing roles' grants |

- **Each entry is tested:** its files as a fixture, a world test running work through it in the focused and fuzzy suites, and its stops checked against temper's derivation of a way's path (`../domain/forge/change/branch-path.md`), so that the drawing shown is the derivation's, never drawn by hand.
- **Shown on the web,** on How work runs and at a project's creation, with the note that every variation of an entry is possible, and how.
- **Known to Ask temper:** a reference of every primitive a way is made of, what it does, its values, who sets it and where (section 5), from which Ask temper proposes any way that is possible, an entry of the gallery or not.

## 4. A piece of work's way

- **A shaped start names a way, or takes its shape's default.** `temper ctl chat start` on a repository, `change start`, `goal start` and `milestone new` take `--way NAME`. A person's New offers the project's ways (section 12). A standing role's `goal` and `change` tools take a way within its grant (section 7). A start refuses a way whose shape isn't its own, and a way the project doesn't offer (8.1), naming why.
- **The default, by shape:** `ways.toml`'s `default` for that shape (8.1), else the first offered way of the shape. With no `ways.toml`, the stock's way of that shape. Kind 7's `way`, today a template's number, is read as the `goal` and `milestone` default until the project has a `ways.toml`. Its `goals` and `changes`, which say what the rail offers, go to the next pass (section 12).
- **`--template` with `--way`:** the way's shape picks the template, and a `--template` naming another shape's is refused. `--template` alone still starts work as today, recorded with no way.
- **What is recorded** with the work, at its start, in one step with its admission: the way's name, its source (the stock at this release, or the project's repository revision, 8.5), and each of its values resolved through section 5's layers. A change records its producer and reviewer charters today (`../domain/forge/change/README.md`, section 1), and this records the rest beside them.
- **Its children take their part of it,** projected from the parent's record, never by looking a way up again:
  - a milestone's goal takes the milestone's `goal` values, `charters`, `reaches`, `questions` and both landing tables;
  - a goal's change into its branch takes `landing.goal` and the goal's `charters`;
  - the goal's own landing into the landing branch, its close change and its catch-ups take `landing`;
  - a child may set a value of its own at its start, within section 5.
  Admission checks the caller's and the parent's authority and funds, as today. `budget` is the top piece's request. A child's budget is chosen at its start, as today.
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
| CI, up to date | a rule's `ci`, `up_to_date` | work defaults | `landing` | — |
| the agent review | a rule's blocking gate | work defaults | `landing.review` | — |
| a person's approval | a rule's `approvals`, by branch or by path | work defaults | `landing.approval` | — |
| what reaches the requester | the path sets' classes, the fixed waits | kind 8's classes | `reaches` | Always ask, a goal's |
| how questions travel | — | kind 6 | `questions` | a change's `questions` |
| the charter each agent runs by | a floor's review: kind 3's reviewer (5.4) | kind 3's producer and reviewer, and the templates' charters | `charters` | a change's producer |
| models, efforts, speeds | the catalogue, and the policy's models | the charter's and the template's choices | `models` | a start's `--model`, `--effort` and `--speed`, a change's choices |
| priority, deadline | the template's and the policy's bounds | the template's | `priority`, `deadline` | a start's `--priority`, `--deadline` |
| added reviews and approvals | the floor's own, which nothing removes | work defaults | `landing.reviews`, `landing.approvals` | — |
| sharing a landing | the project's `by` and `batch` | the project's `combine` | `landing.share` | — |
| budget | the policy's ceilings | the template's budget | `budget` | a start's `--budget` |

Each row already has its default and, mostly, its per-work choice. A way adds the middle layer, and the work gains the landing rows.

### 5.3 Work defaults, a new project kind

- **Its value:** `landing` and `landing.goal`, as a way's (3.3), for the project. The project's owners decide it, as they decide kind 2.
- **The stock's:** CI and up to date into the landing branch, the agent review on, and no approval, as stock rule 1 asks today. Into a goal's branch, neither `ci` nor `up_to_date`.
- **The floor or the defaults:** an owner who wants a value to hold whatever the way says puts it in a kind 2 rule. One who wants it as a starting point puts it here.

### 5.4 One set of criteria per change

- **The work's own rule:** a change's recorded `landing` values make one more enforcing rule, covering that change alone. It joins the floor's covering rules everywhere they are computed: the gates scheduled, the checks and reviews selected, the queue's and a turn's admission, requirements 5 and 6 at the merge, and the sign-offs counted (`../protocol/configuration/settings.md`, section 3; `../domain/forge/change/gates.md`, sections 1 to 3; `../domain/landing-turn.md`, section 1). It is the plan-added gates' generalisation, filled from the way.
- **It can only add.** Rules combine as today: any refusal wins, else any wait wins (`../domain/forge/change/gates.md`, section 2). The work's rule adds a gate, a wait or an approval, and never removes one the floor asks for.
- **A floor's review is the owners':** an agent review a floor rule requires runs kind 3's reviewer, with no project charter of `replaces` (6.2), whatever the way's `charters` say. A way's own reviewer judges only the review the way adds.
- **A goal's direct writes:** the goal's agent writes its design straight into the goal's branch, outside any change (`../domain/forge/branches.md`, section 3). `landing.goal` covers the changes into that branch. The floor judges what the branch brings at its landing whole.
- **Who may set a work's value:** its requester, for its own work. By default a person may set any value the floor leaves free (section 13). A standing role may set only what its grant allows (section 7). An amendment re-steps a change that is queued or being proved, and the evidence it had counts only where it was gathered for the same criteria at the same head.

### 5.5 The stock floor, and upgrades of it

- **For a new project,** stock rule 1 asks CI and up to date into each repository's landing branch, and the review lives in the work defaults. A pattern naming a repository's landing branch, whatever it's called, is new: a pattern names a branch by its name today.
- **For a project that exists,** today's stock rule 1, every branch with CI, up to date and gate 1, is written as the project's own kind 2 rule at the upgrade, beside its other rules, so that nothing it enforced loosens. An owner who wants ways to vary a value moves it to the work defaults by a decision.

## 6. Charters and their words

### 6.1 Six sources

| Source | Says | Read from | Changed by |
|---|---|---|---|
| the harness | tools, the brief's frame, the checkout, delivery, the budget | smith's and temper's code (smith's `domain/run-start.md`) | a release |
| temper's method | how temper's agents work, in that charter | the stock charter's files (`stock-charters.md`, section 2), or the deployment's charter where it sets one whole | a release, or the deployment's owners |
| the deployment's lines | what its machines are, and its own rules | a deployment overlay of the charter (jig's `domain/engine/overlays.md`) | the deployment's owners |
| the repository's guide | how this code is written, for every agent and person | `AGENTS.md` at the root of each workspace directory, as smith finds it; others as it says (smith's `domain/run-start.md`, 1.3) | the repository |
| the project's words | what this project wants of that charter's agents alone | `.temper/charters/<name>.md` of the home repository (6.2) | the home repository (8.6) |
| the task | its words, its brief and its messages | the task (`../domain/agent/charter.md`, section 1) | its requester, and the run |

- **Sources, not an order:** smith lays out the prompt in its own order, instructions and brief before workspace guides (smith's `domain/run-start.md`). The table says where each word comes from.
- **Each source says what only it can say.** A rule said in two of them is a fault, which the preview shows (6.4).
- **A run with no workspace** has no guide.

### 6.2 The project's words

- **A file per charter, named for it** (section 9): `.temper/charters/implementer.md` adds to the implementer's charter, and `.temper/charters/reviewer.md` to the reviewer's. The goal's charter keeps `stock-charters.md`, section 2's three files: `goal.md` for both its runs, `designer.md` for its first, and `coordinator.md` for its later wakes.
- **Its form is an overlay's charter file** (`stock-charters.md`, section 4): text before any part heading joins the shared instructions, after the stock's and the deployment's. A part heading, `## part, on: REASONS`, adds a part after theirs, selected by its reasons as a stock part is.
- **Words alone.** A file can't set models, tools, wakes or budgets, which stay settings (2.1).
- **A front matter**, optional, between `+++` lines, in TOML:
  - `base = "implementer"` makes a charter of the project's own, with a name no stock or deployment charter has. It runs by its base's settings with these words over it, and a way's `charters` names it as it names any charter.
  - `replaces = true` drops temper's method, keeping the harness, the deployment's lines and the base's settings. Nothing else can unsay a stock line (`stock-charters.md`, section 11). The landing check refuses it for a charter that judges, the reviewer's (5.4).
  - `format`: the file's format (8.8).
- **No `common.md`:** a repository's words for every agent are its `AGENTS.md`.

### 6.3 Carried as a project's overlay

- **jig's overlay gains a project scope** beside the deployment's, applied after it. A charter then resolves for a project as its stock or whole charter, the deployment's overlay, then the project's.
- **A charter's identity gains the project:** the effective charter is its number, the settings revision, and the project's repository revision (8.5). A charter of the project's own (`base`) is its base's number with the project's name for it. Two projects' variants of one charter are then two keys, and a record keeps all three.
- **Revisions as jig's are:** each new task takes the project's newest repository revision at its creation, and keeps it, as it keeps its settings revision (jig's `domain/engine/settings.md`, sections 3 and 4). A running task's words don't change. A goal's later changes take what has landed since.
- **The worker gets assembled text,** as today, and needs nothing new to run a project's words. The engine and jig resolve the scope.
- **Bounded at the read:** the project's words count in the charter's size (jig's `domain/engine/settings.md`, section 8), with their parts among the charter's. A file that would take a charter past a bound fails the landing check (8.6), naming the file and its bytes. A release, a settings change or a limit's change that puts a charter over a bound refuses new tasks of that charter, naming it, until it fits.

### 6.4 Seen and recorded

- **A preview:** `temper ctl settings show charter NAME --project PROJECT [--reason REASON]` prints a charter's assembled instructions, each source headed by where it came from and its size. A page draws the same in the next pass.
- **Each run records what it read:** the release, the settings revision, the project's repository revision, and the guide's file and content hash, so that what a run was told can be read back while the run's records are kept.

## 7. Standing roles

### 7.1 Between a person and ways

- **A requester, as a person is:** a standing role starts work in a way, and the work's questions reach it. It decides, or asks its person, as today (`../domain/agent/standing.md`, section 3). A person talks with it as with any standing role (section 6 of that document).
- **It never signs off:** a way's `approval`, the work's or a floor rule's, asks people holding a project role, on the forge. A standing role's work waits for them as any work does (`../domain/forge/change/gates.md`, section 2).
- **Pairing on a person's behalf,** with a standing role in the requester's seat of a `pair` shape, follows from the model and isn't designed here (section 14).

### 7.2 Kinds and instances

- **A kind is a named charter:** `standing-role`, `triager`, `prover` and `porter` in the stock (`../domain/agent/standing.md`, section 2), `drafter` for the design session's, and a deployment's own. Areas name a kind today as `charter N`, meaning the project's lowest-numbered live role of it (`../domain/areas.md`, section 2). They take its name too.
- **An instance is a live task:** its person, its words, its area, its budget a period and its Now, in the store. Instances come and go and are never in `.temper/`.
- **A kind's words in `.temper/charters/`,** `.temper/charters/drafter.md` for one, would be the project's words for a standing role, as for any charter. The user takes that up later (section 14). This draft keeps the place free.

### 7.3 Its grant

- **What it holds:** the ways it may start, by name; the shapes among them; which of a work's values it may set and in which direction; and its spend, as today. It is kept with its task's authority (`../domain/agent/standing.md`, section 5). Whoever starts the role grants within their own. Today nothing lists what a role may start.
- **Checked at every start,** delegation and amendment the role makes, against the way as it resolves then. A way's file changes only by a landing on the home repository (8.6), so whatever lets a change land there decides what every grant naming the way allows from then on. A project that wants an owner to see each such change turns the sign-off on.
- **Until a role has a grant** (`ways.execution.md`), it starts work in its shape's default way and sets no value.
- **A value outside its grant** isn't set: the role asks its person, as for a decision outside its rules.

### 7.4 Ways stop bringing standing roles

- **A recipe for one piece of work can't own a project's agent.** Draft first's triager (`../protocol/configuration/settings.md`, section 8; `standing-roles.md`, section 10) becomes a project's own standing role, started with `role start` or Team's "Hand an area to a role". A way names none. A project's triager stays as it is.
- **The next pass** decides where the stock suggests a triager, for one to a project that first runs Milestone (section 12).

## 8. `.temper/`

### 8.1 Its layout

```
.temper/
  ways.toml            which ways the project offers, in what order, and each shape's default
  ways/<name>.toml     a way of the project's: new, or picked from the gallery
  charters/<name>.md   the project's words for a charter (6.2)
  hooks/               the repository's hooks and their timeouts (hooks.md)
```

- **`ways.toml`:** `format` (8.8), `offer`, the ways' names in order, each a file in `ways/`, and `[default]`, one offered way per shape (`change = "hand-it-over"`). Work already running keeps its way (section 4).
- **All or nothing** (the user, 2026-11-27): a repository with `ways.toml` offers its own ways alone, and the stock ways aren't offered there, so none of their names is reserved. One without `ways.toml` offers the stock ways, each its shape's default. `ways.toml` is the switch, not `.temper/` itself, so that a repository with hooks alone still runs the stock ways.
- **Ways and charters are the project's,** from its home repository, the repository its path is drawn over (`../domain/forge/change/branch-path.md`, section 1). Hooks and the guide are each repository's own. The landing check refuses `ways/`, `ways.toml` and `charters/` in another of the project's repositories, so that a project's method has one home. A context repository's `.temper/` is never read.
- **What temper doesn't read leaves `.temper/`,** so that the directory says exactly what temper does.

### 8.2 Use, copy, remove, add

- **From the gallery:** picking an entry writes its file into `ways/`, whole, and names it in `offer`. It is the project's own from then on. `temper ctl way copy NAME` does the same from the terminal, and `temper ctl charter copy NAME` writes a stock charter's method into `charters/` with `replaces = true`, for a project that wants to own those words.
- **Change one:** edit its file.
- **Remove one:** delete its file and its name from `offer`.
- **Add one:** a file with a new name, which `offer` names.
- **No link back:** a file doesn't say where it came from. What a release changes in the gallery or in temper's method reaches a project's files through Ask temper's review (section 11), for a person to decide.

### 8.3 A way's file

```toml
# .temper/ways/milestone.toml: the project's Milestone, as picked from the gallery and changed
format = 1
headline = "Design first, then agents build it on its own branch, and it lands whole"
line = "You and an agent write a design; agents build it on its own branch; it lands whole."
shape = "milestone"
reaches = "draft-first"
budget = 1000

[charters]
implementer = "careful-implementer"   # .temper/charters/careful-implementer.md, base = "implementer"

[models]
implementer = { model = "gpt-6.2-sol", effort = "high", speed = "ultrafast" }

[goal]
design = true

[landing]
approval = { role = "owner", people = 1, freshness = "clean" }

[landing.goal]
ci = false
up_to_date = false
```

- **People's roles by name:** `role` names a project role as the project's policy does. Its form is the next pass's to settle with `temper ctl`'s.
- **Budget in the deployment's unit,** as a start's `--budget` takes it.

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

- **Checked before it lands:** a change touching `.temper/` waits for a check of temper's own, by code, of the tree that will land: the change's head merged onto the landing branch's tip, and again at a turn's candidate. The files must parse; names must be valid and unique (section 9); every way named must exist and be offered; shapes must be known; charters must fit their bounds; no `replaces` may sit on a judging charter; a hook may be a symbolic link only to another hook beside it ([hooks.md](hooks.md), 3.1); every file must name a format the build reads (8.8); and no `ways/`, `ways.toml` or `charters/` may sit outside the home repository. A failure refuses with the file and its line, as a gate's verdict does.
- **Signed off, where the owners turn it on** (off by default, the user, 2026-11-27): a change touching `.temper/` then also waits for one owner's approval on the forge, a sign-off of temper's own beside the floor's rules, taking none of a project's rule or path set slots. It is a project setting, refused where no owner can sign, a playground's forge or a project with no owner on its forge.
- **Off, the floor alone guards `.temper/`:** ways can ask for less review than the project's defaults, charters shape what agents do, and hooks run code on workers, so a change an agent writes can edit them and land under the project's rules alone. A rule by path naming `.temper/` is how owners ask for a person meanwhile.
- **The floor stays in settings,** where landing can't reach it.

### 8.7 A new project's ways

- **At its creation, or its adoption,** a project whose home repository has no `ways.toml` is shown the gallery, each entry drawn, and the stock ways it would run meanwhile.
- **It may start on the stock ways,** writing nothing: they don't ask a person to buy in yet. Or it picks one or more entries, or writes a way from scratch, and may then change them before going on.
- **temper writes them by a change** into the home repository, "Add .temper/ with your ways": `ways.toml` and the files it names. It lands by the project's own rules (8.6), so that the first thing temper does in a repository is a change a person can read. Until it lands, the stock ways run.
- **A starter is offered,** Hand it over preselected, so that the step is one click, with the advice to start small and grow, the whole gallery in sight.

### 8.8 Versioned files

- **Each file names its format:** `format = 1` in `ways.toml`, in each way's file and in `hooks/config.toml`, and in a charter file's front matter, where leaving it out reads as 1.
- **Every format shipped keeps reading:** a build reads each format temper has released, converting it as it reads. A file lives in a repository temper doesn't control, so this is longer than the store's rule, which keeps no code for a layout older than the deployment runs ([layouts-both-ways.md](layouts-both-ways.md)).
- **A newer format than the build knows** fails the read, naming the file and the format, and the last good revision holds (8.5).
- **Rewriting a file to the newest format** is a change a person asks for, or Ask temper proposes (section 11); nothing rewrites it by itself.
- **temper.toml and the other files a person writes,** the workers' among them, take the same rule in a draft of their own: today they name no format ([../protocol/configuration/file.md](../protocol/configuration/file.md), section 9).

## 9. Names

- **Every template, charter and policy has a name,** and a way the name of its file: 1 to 32 bytes of lowercase letters, digits and `-`, as an area's name (`../protocol/configuration/settings-kinds.md`, section 1). A display name stays beside it, for a person.
- **The number stays the store's key,** and what records keep, with `settings-numbers.md`'s ranges. People, `temper ctl` and files use names, and a number still reads wherever one does today (`--template 34`).
- **Reserved and unique:** a stock item's name is the stock's, and a deployment's own item can't take it. A deployment changes a stock item at its number and keeps its name. Within a kind, a name is unique in the deployment. A project's own charter may not take a name the deployment uses. A way's name is free: the stock ways are offered only where a repository has no `ways.toml` (8.1). A charter file named for a stock or deployment charter holds the project's words for it, over it.
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
- **A project's own items take no number:** a way and a `base` charter are keyed by project and name. A record of work that ran by one keeps the base charter's number, the project and the name, and the repository revision (6.3).

## 10. fabrig and dev.fabrig.ai

- **fabrig's repository:**
  - `temper/docs/charters/fabrig/`, fabrig's overlay (`stock-charters.md`, section 5), moves its words to `.temper/charters/`, a file per charter. Its settings fields, models among them, stay the deployment's.
  - Its common file's rules move to `AGENTS.md`, which already says most of them. Its rule on stored and wire versions contradicts `AGENTS.md`'s fingerprinted layouts and goes.
  - Its scripts move to hooks as [hooks.md](hooks.md), section 6, says.
  - Its own `ways.toml` and ways, once the project can run them (`ways.execution.md`): fabrig picks its ways from the gallery as any project does.
- **dev.fabrig.ai,** by hand, ops' steps, with the release that carries each increment:
  - its machines' lines stay a deployment overlay, applied today by a script kept outside this repository;
  - its lean goal, a deployment template and charter that copy charter 5 whole with edits a script applies, becomes fabrig's own `milestone` way and charter files, so that it follows the stock. Before that, the step checks the deployment's template and charter as they stand, and keeps as settings whatever the files can't say: models, efforts, wakes and budgets;
  - the project's `merge_job` and `close_job` give way to the hooks ([hooks.md](hooks.md)).

## 11. Upgrades

The rules every stored value and wire follows (`../protocol/wires.md`):

- **The store:** work recorded without a way reads as having none (section 4). Settings without names read with the stock's names at the stock's numbers (section 9). A project's existing rules stay its floor, today's implicit stock rule 1 written out as its own (5.5).
- **The engine and jig:** a charter with a project's scope resolves in the engine. The worker gets assembled text, as today, so an older worker runs it. A worker whose layout lacks something a run needs is refused by name, and the run waits for one that has it, as `../protocol/wires.md`, sections 5 and 8 say.
- **Hooks:** [hooks.md](hooks.md), sections 4 and 5.
- **A release's changes reach a project's own files by review** (the user, 2026-11-27): after an upgrade, Ask temper reads the project's `.temper/` beside what the release changed in the stock ways, the gallery and temper's method, infers what the project would want of it, and proposes it as a change, each edit with its reason. A person decides, edit by edit. Nothing rewrites a project's ways or words by itself: prompts are sensitive, and a file the project changed says what it wanted.
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

- **Lowering a way's value for one piece of work:** its requester may set any value the floor leaves free, lower than its way's included, as no approval for one milestone whose way asks for one, the default, since the floor is what owners protect; or a piece of work may only tighten its way, as an approval for one milestone whose way asks for none, and lowering needs an owner.

**Settled by the user, 2026-11-27:** the sign-off on `.temper/` is off by default (8.6); the stock ways a repository without `ways.toml` runs are Pair, once its draft lands, Hand it over and Milestone (3.4); a deployment's own ways, a milestone across repositories and a hotfix cancelling a turn wait (section 14). A change first in line waits for a turn in progress.

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
