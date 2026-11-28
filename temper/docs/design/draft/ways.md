# Ways of working, several at once

Draft, 2026-11-26, from sessions with the user on 2026-11-25 and 2026-11-26. It started from how agents' prompts are assembled and what a repository's `.temper/` holds, and what a good story for both would be. The user added hooks named by the moment they run at, a ladder from a person working with one agent, where "temper adds only the tool calls", to a whole milestone, and then the frame this draft takes: "I'd like temper to support various 'ways' of working at the same time for the same project", as in "for this change I want to pair directly with an agent, for this other I want to delegate and I want a automatic review, for this other one I want a milestone". temper offers a few stock ways that a project uses as they are, changes by copying, removes, or adds to with ways of its own. The settings only owners change are "overridable per-way and possibly per-change", as in "milestone way does no require manual approval", but "this one milestone requires approval". Path rules still hold for any change, whatever its way. Standing roles "sit between the person and ways". Settings get a name beside their number, "mostly an internal detail", and the name maps to `.temper/`.

Second pass, 2026-11-27, from a session with the user that day, over a survey of temper's primitives and a gallery of ways drawn from them. The user's frame: make the whole gallery possible, "eventually (it can be a series of incremental drafts)", built into temper, each way tested, shown in the web UI with the note that every variation is possible, and every primitive explained to Ask temper "so he can propose anything that is possible". A new project picks its ways, from the gallery or from scratch, and they populate its `.temper/`: "no magic, expecially around prompts". `.temper/` is "all or nothing": present, it's authoritative in full; absent, the stock ways run. Its formats are versioned, so that what is decided now keeps being supported. After an upgrade, Ask temper reviews a project's ways and proposes changes "in an intelligent way", "not mechanically", since "prompts are very sensitive". Sharing ways across projects and reconciling drift are left for later. The landing part goes last. The hooks moved to a draft of their own, [hooks.md](hooks.md).

Third pass, the same day: a way's file is written in jig's own terms. The user found `shape` and fixed role names to be the former approach, "we have fixed ways that you can only slightly tweak", against "we have primitives that you can fully compose", and asked that "no role or shape should be 'assumed'": the file should be "as close as possible to the underlying model (that's what makes it flexible), while preserving our light 'opinionated' takes and 'guardrailes'", jig's mechanics with temper's own as its connector. Shapes go: a way is a set of task recipes, and the stock ways are compositions like any other. Then the same rule for the words (the user's yes, 2026-11-27): a repository with its own ways holds its charters whole, the stock's text written into its files, and Ask temper's review after an upgrade is what keeps them current. A second review that evening found the admission, gate, revision and read contracts underspecified; this pass specifies them. On the user's question, the project's cross-cutting defaults, rules by path among them, move into `ways.toml`, and the floor stays in settings.

The first pass covered the mechanics and the whole picture, so that each piece is built in turn and they still fit together; the second settles the vision the increments build towards (`ways.execution.md`). How a person meets it, in pages, words and the terminal, is the next pass (section 12). It revises a project's single stock way (`../protocol/configuration/settings.md`, section 8), standing roles brought by a way (`standing-roles.md`, section 10), kind 7's one preferred goal template (`../protocol/configuration/settings-kinds.md`, section 1) and the charter assembly's sources (`stock-charters.md`, sections 3 and 4). It takes up `stock-charters.md`, section 11's project overlay from `.temper/`, and adds names to `settings-numbers.md`'s numbered kinds. It wins where they differ until its plans fold it in.

## 1. In one page

- **Three homes, one rule** (section 2). `temper.toml` holds what a machine runs. The store's settings hold what grants and bounds: people, accounts, models, budgets and the rules that always hold. A repository's `.temper/` holds how work is done on that code: its ways, its charters' words and its hooks. Nothing lives in two of them.
- **A way is a set of task recipes in jig's terms** (`ways-recipes.md`): which task a person starts, and for each of its tasks who executes it (an agent under a charter and a model, temper's change procedure, a gathering, a person), what counts as done, what it holds (a branch of its own, a copy), whether its requester talks with it, which of the way's tasks it may start and how many, when it wakes, its budget, deadline and priority; for a change, who produces it, which gates judge it, where it lands and what landing asks; and what reaches its requester. No shape or role is assumed: the stock's Pair, Hand it over and Milestone are compositions like any other. temper keeps a few guardrails over every way (`ways-recipes.md`, 1).
- **New work picks its way when it starts and records it** (section 4). Its children take their part of it. Work started before ways runs as it did.
- **Values in four layers** (section 5): the floor, the project's defaults, the way, and the piece of work. The floor is the landing rules, rules by path among them. It always holds, and no way or piece of work lowers it. Below the floor, the work's own value wins over its way's, and its way's over the project's default. One calculation gives a change's criteria wherever they are used.
- **A charter's words come from five sources** (`ways-charters.md`): smith's harness; the charter itself, whole from `.temper/charters/` where the repository has its own ways, else the stock's; the deployment's lines about its machines; the repository's `AGENTS.md`; and the task's words. A run records which it read.
- **Standing roles start work, as people do** (section 7). A kind is a named charter. An instance is a live task, with its person, area, budget and Now, kept in the store and never in `.temper/`. A role's grant says which ways it may start. Ways stop bringing standing roles.
- **`.temper/`** (`ways-temper-dir.md`): `ways.toml`, `ways/<name>.toml`, `charters/<name>.md` and `hooks/` ([hooks.md](hooks.md)). temper reads it whole from one commit of the home repository's landing branch, checks a change to it before it lands, and, where a project's owners turn it on, asks one of them to sign off on it. All or nothing: a repository with `ways.toml` runs its own ways alone, and one without runs the stock's. Each file names its format, and every format shipped keeps reading (`ways-temper-dir.md`, 8).
- **A gallery** (`ways-recipes.md`, 5): ways built into temper as references, each tested and drawn by temper's own path, shown on the web and known to Ask temper, which explains every primitive. A new project picks from it, or starts on the stock ways, and temper writes what it picked into `.temper/` by a change (`ways-temper-dir.md`, 7).
- **Names** (section 9): every template, charter and policy gets a lowercase name. The number stays the store's key. A way's name is its file's. A name is also a path in `.temper/`.
- **fabrig and dev.fabrig.ai move onto it** (section 10), and upgrades keep what each build already runs (section 11). A release's changes reach a project's own ways and words through Ask temper's review, never by rewriting them.

## 2. The picture

### 2.1 Three homes

| Home | Holds | Changed by | Read |
|---|---|---|---|
| `temper.toml` | what a machine runs: its places, its figures, its endpoints (`config.md`, section 3) | whoever runs the machine | at a start |
| settings, in the store | what grants and bounds: people and their roles, accounts, models, budgets and ceilings, the floor (section 5), the deployment's charters and its lines for them | the deployment's or the project's owners, by a decision (`configuring.md`, section 3) | always |
| `.temper/`, in a repository | how work is done on this code: its ways, its defaults across them, its charters and its hooks | whoever lands on the repository's landing branch, under the floor, and an owner's sign-off where the project turns it on (`ways-temper-dir.md`, 6) | whole, from one commit of the landing branch (`ways-temper-dir.md`, 5) |

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

## 4. A piece of work's way

- **A start names a way, or takes a default.** `temper ctl chat start` on a repository, `change start`, `goal start` and `milestone new` take `--way NAME`, and a way's `start` task must fit the command: a conversation for `chat start`, a change for `change start`, an agent for `goal start`, a gathering for `milestone new`. A person's New offers the project's ways (section 12). A standing role starts a way within its grant (section 7). A start refuses a way that doesn't fit, and a way the project doesn't offer (`ways-temper-dir.md`, 1), naming why.
- **The default, by command,** for a `temper ctl` start without `--way` in a project with `ways.toml`: its `default` for that command (`ways-temper-dir.md`, 1), else the first offered way that fits. Without `ways.toml`, such a start runs as today, from its template and today's settings, recorded with no way; `--way` names a stock way that fits. Kind 7's `goals` and `changes`, which say what the rail offers, go to the next pass (section 12).
- **Starts from the web and the TUI** stay as today until their pages change (`ways.execution.md`, W14): they fill a template and send it, and the work is recorded with no way. Today's stock ways and their "Use this" keep writing the project's rules and classes until then, which reaches new work as any floor's change does (5.4).
- **`--template` alone** still starts work as today, recorded with no way. With `--way`, the way's templates are used, and a `--template` beside it is refused.
- **What is recorded** with the work, at its start, in one step with its admission: the way's name, its source (the stock at this release, or the project's repository revision, `ways-temper-dir.md`, 5), and its tasks' recipes as templates, each value resolved through section 5's layers. A change records its producer and gates' charters today (`../domain/forge/change/README.md`, section 1), and this records the rest beside them.
- **What the work starts takes its recipe from that record,** never by looking the way up again: a task the starting task delegates is made from the recorded template its `delegates` names, and a change's producer and gates from the change's recipe. A child may set a value of its own at its start, within section 5. Admission checks the caller's and the parent's authority and funds, as today. A task's `budget` is a request at its own start.
- **Running work keeps its way.** A change to a way, landed or decided, reaches new work only. An amendment of a parent's value reaches its children not yet started. Started ones keep theirs and are amended themselves. A floor's change reaches running work at its next step, as a landing rule's change does today (`../domain/forge/change/gates.md`, section 3).
- **Work started before ways** keeps running from what it stored: its targets, charters, routes and authority. It is shown as having no way. It is never matched to one after the fact.

## 5. Values in four layers

### 5.1 The layers

| Layer | Holds | Set by | Lowered from below? |
|---|---|---|---|
| the floor | the enforcing landing rules, kinds 1 and 2, rules by path among them; the path sets and their classes (kind 8's `sets`); the fixed `Wait` kinds; the policy's ceilings | the deployment's and the project's owners | never |
| the project's defaults | a value each way takes where it says none: `ways.toml`'s `[defaults]` (5.3), and today's kinds for the rest (5.2) | a landing on the home repository (`ways-temper-dir.md`, 6); the deployment's owners, for kind 3 | yes |
| the way | its members (`ways-recipes.md`) | the stock, or a landing on the home repository (`ways-temper-dir.md`, 6) | yes |
| the work | its own values, set at its start or by amendment | its requester, within its grant (section 7) | — |

- **Below the floor, the most specific wins,** member by member: the work's own value, else its way's, else the project's default. Over the floor, nothing wins: it is judged as well, every time.
- **What reaches a person** takes the strictest class, as today (`../domain/forge/attention.md`, section 6): the work's classes from its layers, the path sets' classes, the fixed waits and Always ask. A way's `reaches` replaces the classes alone, never the path sets.
- **A budget is a request,** reserved against the live ceilings and funds at the work's start, as a start's `--budget` is today.
- **The user's example:** the project's default asks an owner's approval, Milestone's way sets none, and one milestone sets it again. That milestone's goals then wait for an owner at the landing branch, and every other milestone lands on its own. A rule by path asking an owner for any change touching `.temper/hooks/` holds for all three, as for any change.

### 5.2 Each value, by layer

| Value | Floor | Project default | Way | Work, today |
|---|---|---|---|---|
| CI, up to date | a rule's `ci`, `up_to_date` | `[defaults]` | a change's `landing` | — |
| gates | a rule's blocking gate | `[defaults]` | a change's `gates` | — |
| a person's approval | a rule's `approvals`, by branch or by path | `[defaults]` | a change's or a branch's `landing.approvals` | — |
| what reaches the requester | the path sets' classes, the fixed waits | `[defaults.reaches]`, else kind 8's classes | `[reaches]` | Always ask, a goal's |
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

### 5.3 The defaults, in `ways.toml`

- **Cross-cutting, and the project's method** (the user, 2026-11-27): `ways.toml`'s `[defaults]` holds what every way takes where it says none, so that a rule meant for all of a project's work is written once, beside its ways, and changed by a landing as they are.
- **Its members:**
  - `landing.base` and `landing.held`: `ci`, `up_to_date` and `approvals`, as a change's `landing` (`ways-recipes.md`, 3), for changes into a repository's landing branch and into a branch a task holds;
  - `gates.base` and `gates.held`: the gates such changes take, each a recipe in `ways.toml`'s own `[tasks]`, written as a way's are;
  - `[defaults.reaches]`, a classes table as a way's;
  - `questions`;
  - `[[defaults.sets]]`, path sets of the defaults' own, each a name and patterns, and `[[defaults.rules]]`, rules by path across ways: a rule adds `ci`, `up_to_date`, gates or approvals to a change that touches one of its `sets`, or, with `unless_only`, to every change but one touching only its sets ("CI unless only documentation"), a form the floor's rules don't have. A default rule joins every change's criteria, as the floor's rules do, and no way or piece of work lifts it: it changes only by a landing, where the floor changes only by its owners (the gallery's validation, 2026-11-27).
- **The stock's,** where `ways.toml` says none: CI and up to date into the landing branch, the stock review, no approval, as stock rule 1 asks today; into a held branch, neither `ci` nor `up_to_date`; Agents review's classes; `questions = "up"`.
- **Defaults, not the floor:** a way's value, or a piece of work's, wins over a default, as section 5 says, and a later landing can change them. What must hold whatever any landing says is the floor's, in settings: an owner puts it in a kind 2 rule.

### 5.4 One set of criteria per change

- **The work's own rule:** a change's recorded `landing` values make one more enforcing rule, covering that change alone. It joins the floor's covering rules everywhere they are computed: the gates scheduled, the checks and reviews selected, the queue's and a turn's admission, requirements 5 and 6 at the merge, and the sign-offs counted (`../protocol/configuration/settings.md`, section 3; `../domain/forge/change/gates.md`, sections 1 to 3; `../domain/landing-turn.md`, section 1). It is the plan-added gates' generalisation, filled from the way.
- **It can only add.** Rules combine as today: any refusal wins, else any wait wins (`../domain/forge/change/gates.md`, section 2). The work's rule adds a gate, a wait or an approval, and never removes one the floor asks for.
- **A floor's review is the owners':** an agent review a floor rule requires runs kind 3's reviewer, the deployment's charter, never a charter from the repository, whatever the way's gates say. A way's own gate judges only the review the way adds.
- **A goal's direct writes:** the goal's agent writes its design straight into the goal's branch, outside any change (`../domain/forge/branches.md`, section 3). The `landing` of the changes into that branch covers them. The floor judges what the branch brings at its landing whole.
- **Who may set a work's value:** its requester, for its own work. A person may set any value the floor leaves free, lower than the way's included (the user, 2026-11-27). A standing role may set only what its grant allows (section 7). An amendment re-steps a change that is queued or being proved, and the evidence it had counts only where it was gathered for the same criteria at the same head.

### 5.5 The stock floor, and upgrades of it

- **For a new project,** stock rule 1 asks CI and up to date into each repository's landing branch, and the review lives in the defaults (5.3). A pattern naming a repository's landing branch, whatever it's called, is new: a pattern names a branch by its name today.
- **For a project that exists** (the review's seventh finding): today's stock rule 1, every branch with CI, up to date and gate 1, is the fallback at rule number 1 for a project with no rule 1 of its own. The upgrade writes it as that project's rule 1, at that number, only where it applies: a project with its own rule 1 has already replaced it, and nothing is added beside its rules. Nothing it enforced loosens. An owner who wants ways to vary a value moves it to the defaults by a decision and a landing.

## 7. Standing roles

### 7.1 Between a person and ways

- **A requester, as a person is:** a standing role starts work in a way, and the work's questions reach it. It decides, or asks its person, as today (`../domain/agent/standing.md`, section 3). A person talks with it as with any standing role (`ways-charters.md` of that document).
- **It never signs off:** a way's `approval`, the work's or a floor rule's, asks people holding a project role, on the forge. A standing role's work waits for them as any work does (`../domain/forge/change/gates.md`, section 2).
- **Pairing on a person's behalf,** with a standing role as the requester of a pair's conversation, follows from the model and isn't designed here (section 14).

### 7.2 Kinds and instances

- **A kind is a named charter:** `standing-role`, `triager`, `prover` and `porter` in the stock (`../domain/agent/standing.md`, section 2), `drafter` for the design session's, and a deployment's own. Areas name a kind today as `charter N`, meaning the project's lowest-numbered live role of it (`../domain/areas.md`, section 2). They take its name too.
- **An instance is a live task:** its person, its words, its area, its budget a period and its Now, in the store. Instances come and go and are never in `.temper/`.
- **A kind's words in `.temper/charters/`,** `.temper/charters/drafter.md` for one, would be the project's words for a standing role, as for any charter. The user takes that up later (section 14). This draft keeps the place free.

### 7.3 Its grant

- **What it holds:** the ways it may start, by name, which authorizes their recipes as they resolve at each start, later revisions included, since a landing on the home repository changes them (`ways-temper-dir.md`, 6); which of a work's values it may set and in which direction; and its spend, as today. It is kept with its task's authority (`../domain/agent/standing.md`, section 5). Whoever starts the role grants within their own. Today nothing lists what a role may start.
- **Checked at every start,** delegation and amendment the role makes, against the way as it resolves then. A way's file changes only by a landing on the home repository (`ways-temper-dir.md`, 6), so whatever lets a change land there decides what every grant naming the way allows from then on. A project that wants an owner to see each such change turns the sign-off on.
- **Until a role has a grant** (`ways.execution.md`), it starts work in the default way of the command it uses and sets no value.
- **A value outside its grant** isn't set: the role asks its person, as for a decision outside its rules.
- **A way that starts with a gathering** is started by a party: jig admits a gathering only as a party's root (jig's `domain/tasks/gatherings.md`). A role proposes one to its person, naming the way and the repository revision, and the person's acceptance makes it, as their own, funded by them (the review's fifth finding).

### 7.4 Ways stop bringing standing roles

- **A recipe for one piece of work can't own a project's agent.** Draft first's triager (`../protocol/configuration/settings.md`, section 8; `standing-roles.md`, section 10) becomes a project's own standing role, started with `role start` or Team's "Hand an area to a role". A way names none. A project's triager stays as it is.
- **The next pass** decides where the stock suggests a triager, for one to a project that first runs Milestone (section 12).

## 9. Names

- **Every template, charter and policy has a name,** and a way the name of its file: 1 to 32 bytes of lowercase letters, digits and `-`, as an area's name (`../protocol/configuration/settings-kinds.md`, section 1). A display name stays beside it, for a person.
- **The number stays the store's key,** and what records keep, with `settings-numbers.md`'s ranges. People, `temper ctl` and files use names, and a number still reads wherever one does today (`--template 34`).
- **Reserved and unique:** a stock item's name is the stock's, and a deployment's own item can't take it. A deployment changes a stock item at its number and keeps its name. Within a kind, a name is unique in the deployment. A way's name is free: the stock ways are offered only where a repository has no `ways.toml` (`ways-temper-dir.md`, 1). So is a project's charter's: a file named as a deployment's charter is the project's whole charter of that name, run by that charter's settings unless its `settings` names another (`ways-charters.md`, 2).
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
- **A project's own items take no number:** a way and a project's charter are keyed by project and name. A record of work that ran by one keeps the project, the name, the repository revision and its `settings` charter's number (`ways-charters.md`, 3).

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
- **A release's changes reach a project's own files by review** (the user, 2026-11-27): the store keeps, per project, the release its files were last reviewed against. After an upgrade past it, the project's next Ask temper session, or an owner's ask, reads the project's `.temper/` beside what the stock ways, the gallery and the stock charters changed since that release, infers what the project would want of it, and opens a change of its proposed edits on a branch, its words listing each edit with its reason. A person decides edit by edit, dropping what they don't want from the branch before it lands; it lands by the project's rules, its base moving as any change's does, and its landing records the release as reviewed. Nothing rewrites a project's ways or words by itself: prompts are sensitive, and a file the project changed says what it wanted.
- **Files' formats** keep reading, as `ways-temper-dir.md`, 8 says.
- **Clients** at another layout are refused by name and reload or upgrade, as `../protocol/wires.md`, section 9 says.

## 12. For the next pass, how a person meets it

What this pass leaves to the pages, the terminal and the words:

- **Picking a way:** New's menu as the project's ways (`template-places.md`'s offers become ways'), the terminal's `--way`, and Ask temper suggesting one from a request.
- **How work runs:** the project's ways, its floor and its defaults, in place of one displayed way and its path (`../client/pages/settings.md`, section 2). `see ways` and Ask temper's picture change with it.
- **The preview** of a charter's words (`ways-charters.md`, 4), and a way's file shown beside what it changes.
- **The stock ways' names and words** (`ways-recipes.md`, 4), You approve as a switch, and where the stock suggests a triager (7.4).
- **The gallery's page** (`ways-recipes.md`, 5) and a new project's step (`ways-temper-dir.md`, 7): the drawings, the starter, and how a picked entry is changed before it's written.
- **Ask temper's review after an upgrade** (section 11): how its proposal reads, edit by edit.
- **The rail's entries** that kind 7's `goals` and `changes` say today (section 4), and how a project with work of no way shows it.

## 13. Open questions

None left from the second pass.

**Settled by the user, 2026-11-27:** whoever starts a piece of work may set any value the floor leaves free, lower than its way's included, as no approval for one milestone whose way asks for one (5.4); `ways.toml` is the all-or-nothing switch (`ways-temper-dir.md`, 1); the sign-off on `.temper/` is off by default (`ways-temper-dir.md`, 6); the stock ways a repository without `ways.toml` runs are Pair, once its draft lands, Hand it over and Milestone (`ways-recipes.md`, 4); a deployment's own ways, a milestone across repositories and a hotfix cancelling a turn wait (section 14). A change first in line waits for a turn in progress.

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
