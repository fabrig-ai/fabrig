# Configuring temper by talking to it

Draft, 2026-10-23, from a session with the user that day, whose turns
shaped it; revised 2026-11-02 with the user, who set its acceptance
(section 8) and its first surface, `temper ctl` (5.4). A person
describes how they want work to go, and an agent configures temper to
run it: settings that live in the store and never in the
configuration document, a change that new work follows while running
work keeps what it started with, and a split between what temper
enforces and what agents follow. Its mechanisms are designed in jig and
in temper, as its status says; this draft keeps what that design still
cites it for, its acceptance and its order (section 9).

## 1. In one page

- **The person never learns the words.** They describe how they work,
  in their own words or by pointing at a document, as
  `reference-code-production.md` describes ours. temper answers with a
  proposal, "here is how I'll run that", in their words with the
  settings underneath (section 5).
- **Structure becomes settings; rules stay text** (section 2). What
  changes what temper wakes, schedules, permits, pays, lands or holds is
  structure, and the agent turns it into settings. What only changes how
  an agent behaves is a rule, and stays in the documents runs read.
- **Settings live in the store,** never in `temper.json` (section 3).
  A start loads them from the store, a setting the store does not hold
  takes the binary's default, and a change is saved in the store, as
  projects already are. The document keeps what the process needs before
  it can read its store, and what belongs to the machine.
- **A change is one decision** on the engine, the store's only writer:
  a set of settings checked whole against the revision it was made
  against, committed with all it touches (3.3).
- **New work follows a change; running work keeps what it started
  under** (section 4). Each change makes a revision of the settings; a
  task runs under the revision it was made in, and only its charter is
  read from it. A conversation moves to the newest at its next run, and
  a change may move running work. What a task may do is never pinned:
  rules and policies apply to every decision.
- **The agent proposes; an owner accepts** (5.2), on a review page in
  plain words, or with `proposal show` and `proposal decide` (5.4), with what temper enforces apart from what agents follow,
  and what the change does to running work.
- **It explains, both ways** (section 6): it answers from the settings
  and the documentation built into the binary
  (`../protocol/documentation.md`), and it describes the settings back
  as a workflow.
- **`temper ctl` first** (5.4): every act of the person's way is a
  command, so a pass of QA drives all of it. The web client's pages
  (5.1, 5.2) say the same, and are built separately.
- **Its acceptance** (section 8): on a box that holds no recipe of the
  reference, QA, through `temper ctl` alone, asks the configuring agent
  to set a project up as `reference-code-production.md` describes,
  accepts its proposal and checks it against that draft's section 14.
  First the settings, as far as temper can say them, the rest named as
  gaps (8.1); then, once the reference case walks, a draft carried from
  intake to landing under them (8.2).

## 2. Structure and rules

- **The test, for each sentence of a description:** would temper's
  engine act differently without it? If the sentence changes what
  temper wakes, schedules, permits, pays for, lands or holds, it is
  structure, and becomes settings. If only the agent reading it would
  act differently, it is a rule, and stays text that runs read.
- **The reference process, sorted** (`reference-code-production.md`):

  | From the reference | Kind | In temper |
  |---|---|---|
  | The coordinator on Opus 5.5 at xhigh, its surveys at high; producers on gpt-6.1-sol | structure | the charters' models, and each change's LLM choices (`notes.md`, N19) |
  | A goal per draft, holding a branch of its own | structure | the goal template; its own branch is a gap (`notes.md`, N3) |
  | Every change of the goal lands into its branch, with no CI | structure | a landing rule for goal branches (`notes.md`, N4) |
  | The coordinator woken at once by questions and results, by a tier's red at most every 30 minutes | structure | its charter's wake policy (`notes.md`, N23) |
  | At most four questions per change, through a foreman | structure | the change's question bound, which no design has yet (`notes.md`, N1) |
  | Designers write the design under `docs/design`, a status file beside each document | rule | the designer's charter's instructions |
  | A producer pushes early and finishes with a Left list | rule | the coding agent's charter's instructions |
  | A producer never fixes what it does not own | rule | authority is per repository, not per path |
  | `.temper/pre-pr` passes before delivery | rule | the project's checks, which the producer runs (`../domain/agent.md`, 4.5); the engine runs none of it |

- **Rules say how agents use the structure.** "Designers first" is a
  rule; the dependency it asks the coordinator to make is the engine's.
  The agent configures the mechanisms, and the rules stay where the
  person wrote them, so most changes to a process need no configuring at
  all.
- **A rule may become structure** where temper's model has it, and the
  agent offers it: "I can make each family's changes wait for its
  designer, as dependencies temper holds; 'push early' stays a rule your
  producers follow". The person chooses what is guaranteed. The producer's last row is the honest answer the other
  way: it looks enforceable, and is not.
- **Rules must reach the runs.** A run reads `AGENTS.md` in its
  workspace's repositories (`../domain/agent.md`, 4.5), and a run with no
  repository has no workspace, so a chat cannot read the process's
  document. A
  charter's instructions may therefore name documents, each a
  repository, a path and its landing branch, which the engine reads into
  each run's brief: a rule that lands reaches the next run. A
  description typed in the chat, with no document behind it, puts its
  rules in the charter's own instructions.
- **The proposal shows the split** (5.2): *temper enforces*, the
  structure, and *your agents follow*, the rules and where they come
  from. What is guaranteed is never confused with what depends on agents
  behaving.

## 3. Settings in the store

### 3.1 Which

The user's rule: `temper.json` keeps only what the process needs before
it can read its store, and what belongs to the machine; everything else
is a setting in the store. Projects already left it: jig makes them at
runtime, and the first run makes them as the person goes
(`archive/first-run.md`, 2.4). This draft moves the rest of what
configures work:

| Setting | In the document today | Read | So |
|---|---|---|---|
| charters | jig's `engine.charters` | at each run's claim | kept per revision (section 4) |
| templates | jig's `engine.templates` | by clients, when they make a task | the newest |
| policies a project is made from | jig's `engine.policies` | at `MakeProject` | the newest |
| the deployment's rules, the ceilings | jig's `engine.rules` | at every decision | always current |
| landing rules, the deployment's and the projects' | temper's `application.landing` | at every landing | always current |
| the forge's charter | temper's `application.charter` | when a procedure makes agent work | the newest |

- **A project's own policy** is in the store already, changed by its
  owners (jig's `domain/people.md`, 5.2), and stays as it is.
- **Only charters are read after a task is made.** A template's values
  are copied into the task it fills (jig's `protocol/configuration.md`,
  3.1), a project keeps a copy of the policy it was made from (jig's
  `domain/people.md`, 5.2), and the rules are guards that every decision
  reads afresh. That is why only charters need revisions.
- **What stays in the document:** the directories, the listeners, the
  workers and their secrets' files, the limits (they size memory at the
  start), the first owners and the sign-in providers (needed before
  anyone signs in), and, for now, the LLM providers, which follow the
  same rule later, a design of their own (section 10): models are bit
  positions in authorities. Forges are records in the store already
  (`archive/playground-reset.md`, section 3).

### 3.2 Defaults and changes

- **The binary holds the defaults:** jig's chat and goal templates,
  temper's stock charters (today `temper init` embeds a chat's and a
  coding agent's, `../protocol/setup.md`, 8.2), the stock policy and the
  stock landing rule. The store holds what was changed, setting by
  setting, each by its number.
- **The current settings are the defaults with the stored changes over
  them.** A setting never changed follows the binary: a release that
  improves a stock charter reaches every deployment that kept it, and
  its release notes say so. "Reset to default" removes a stored change,
  as a change of its own.
- **A start loads them from the store** before the core admits
  anything. A start whose binary's defaults differ from the newest
  revision makes a new revision, recorded as the release's, with no
  proposal: new work takes the new defaults, and running work keeps its
  own (section 4).
- **The document stops naming them.** A document that names one is
  refused, as any unknown member is (`../protocol/configuration.md`,
  section 1); from the baseline on, a removed member is first accepted
  for one release with a warning (`upgrades.md`, section 10). Until the
  baseline a store whose format changes is made again, so nothing needs
  importing; an operator who keeps settings in a file proposes it
  whole with `settings propose` (`../client/tools.md`, 5.3).
- **Bounds stay the start's.** A change is checked as the start checks
  a document today: charters within jig's `tasks.charters`, a charter's
  instructions within the journal's run bytes, templates within jig's
  `templates`, a policy within its bytes. The limits stay in the
  document, so a change allocates nothing beyond what they priced.

### 3.3 A change

- **One decision.** The engine is the store's only writer, and a
  decision commits all it changes at once (jig's `domain/core.md`,
  section 4): the settings changed, the revision they make and whatever
  they touch. After a crash the old revision holds or the new one does,
  never a mix.
- **A set, checked whole:** a new charter, the template that starts
  it and the policy that funds it go together. It names the revision it
  was made against, and is refused as stale when another is current.
- **Who decides:** an owner of the deployment for charters, templates,
  the rules and the policies projects are made from; an owner of a
  project for its policy and its landing rules, as now. A set that
  touches both needs one person holding both.
- **A proposal of a new kind.** jig's proposals are batches and
  amendments of tasks (jig's `domain/tasks.md`, section 9). A change of
  settings proposed by an agent or a member is a proposal in the
  deployment's scope, decided by the roles above, judged when it is
  accepted. jig designs it.
- **jig's part and temper's in one decision.** jig holds settings in
  the store generically: its charters, templates, policies and rules,
  their revisions, the change and its proposal. temper holds its landing
  rules and its forge's charter as settings of its own, through the same
  mechanism, and its stock defaults. One change may touch both, through
  temper's root.
- **What follows:** clients re-read `me`, whose templates changed; the
  next `MakeProject` copies the changed policy; a narrowed rule reaches
  every later decision, the decisions of tasks already made included, as
  a policy's narrowing does (jig's `domain/people.md`, 5.2); charters
  as section 4 says.
- **History.** The store keeps the set as each change left it, within
  a bound, with who accepted it and the proposal it came from.
  "Restore this version" makes a new proposal of that set, never a
  silent rollback.

## 4. Running work keeps its charter

The user's idea: settings side by side, as an array of them. The first
is loaded at the start; a change adds the next; from then on, what is
made follows the new one, and what is in flight keeps the old.

- **A revision is the set as a change left it,** numbered from the
  first start's. Revisions share what a change left alone: a charter is
  kept once per revision that changed it, so a revision costs only the
  charters it changed.
- **A task runs under the revision it was made in,** recorded on the
  task: the newest, when it is made. Each of its runs reads its charter
  from that revision: its instructions, models, smith's tool families,
  hosts, waiting, resume and budget per run. Templates, the policies
  projects are made from and the forge's charter are only read when
  something is made, so the newest is the one.
- **New delegates take the newest revision,** whatever their parent
  runs under. A delegation to a charter the newest dropped is refused as
  not offered, and the agent adapts, as to any refused create. Taking
  their parent's revision instead, so that a goal's subtree keeps the
  settings its plan was made under, is a later option of a change
  (5.2).
- **A conversation moves at its next run.** A task its requester ends
  (a chat; `../domain/agent.md`, 4.6) never ends on its own, so it would
  never see a fix to its own charter. It moves to the newest revision
  at its next run, when the newest still offers its charter and allows
  its LLM choice (jig's `domain/engine.md`, 17.4); otherwise it stays.
- **A change may move running work:** each task at its next run,
  checked the same way. A task whose charter the change removed, or
  whose LLM choice it no longer allows, cannot move, and finishes where
  it is.
- **What a task may do is never pinned.** Its authority, from the
  rules, its project's policy and the landing rules, is always current;
  what its runs are given is its charter's, pinned. A change that must
  reach running work now moves it at its next run; a live run goes on
  until it ends, or an owner cancels it.
- **Removing is free.** A removed charter lives in the revisions that
  hold it while tasks use them. That answers jig's open question on
  parts removed at a restart (jig's `protocol/configuration.md`, section
  9), and two questions of a process that changes: retiring a charter
  (no new work under it; what runs finishes) and whether a charter's
  change reaches tasks already running (not unless they move).
- **Bounded.** At most a limit's number of revisions are in use, and
  the charters they hold within another. A change that would need more
  is refused, naming the tasks that hold the oldest, with moving them
  offered. A revision no task uses, other than the newest, leaves memory;
  its history stays (3.3).
- **In the store:** each task's revision (a change to jig's tasks
  family), and the revisions in use, whole. A restart restores them
  before the core admits anything, and an upgrade's patches carry them
  as any record (`upgrades.md`).
- **In the children:** jig's core holds charters, templates and models
  as one value today, its settings, documented as immutable. The tasks
  child's models per charter and the fleet's host needs per charter
  become per charter and revision, and lookups take the task's revision.
  How far that reaches is the survey's question (section 9).

## 5. The person's way

`temper ctl` is the first surface (5.4), the user's choice of
2026-11-02: the mechanisms first, exposed so that QA can run them. The
pages of 5.1 and 5.2 are the web client's (`../client/README.md`),
designed and built separately, and act through the same requests.

### 5.1 Where they meet it

- **The stock defaults are enough to start.** The first run's wizard
  (`archive/first-run.md`, 2.3) grows no step. After its Ready, and on
  an empty "How work runs" page, one offer: "Tell temper how you work".
- **The conversation is a chat** from a stock template whose charter is
  the configuring agent's: nothing particular in jig, and a setting
  itself, so it can be reconfigured.
- **"How work runs"** lists the settings in the person's words: the
  roles (charters), the ways to start work (templates), the rules (the
  project's policy and landing rules). Each says whether it is the
  default or changed. The page also shows the revisions still in use,
  with their tasks and "Move them", and the history (3.3). It is where
  proposals are reviewed, and it is needed with or without the agent.

### 5.2 A proposal

- **In the chat,** the agent asks until it can propose, then posts a
  card: its title, one line per change, and "Review". Nothing applies
  from the chat.
- **The review page:**

  ```
  Proposal 12 · from "How we work" · against revision 7

  temper enforces
    Testers            GPT-5 high → GPT-5 medium
    Sub-owners (new)   a deadline 4 h from start; it holds running work too
  your agents follow
    Owners             acme/handbook process.md, "Owners" (main)

  Effect
    New work follows this from now. Nothing restarts.
    9 running tasks are on revision 7:
      (•) Let them finish on revision 7
      ( ) Move them at their next run
    2 chats move at their next run.
    2 advisors cannot move: this removes their role. They finish on 7.

  [Show the settings]            [Reject…]  [Change something]  [Accept]
  ```

  Changes are grouped by what the person knows (roles, ways to start
  work, the project's rules, landing), each as before and after in
  words; the settings' own diff is folded away.
- **Who accepts:** an owner (3.3). A member talking to the agent gets the
  same card, marked "Waiting for an owner", and the proposal waits in an
  owner's inbox, as jig's proposals do.
- **Reject and Change something** go back to the chat as the person's
  words, and the agent revises.
- **Out of date:** when another change was accepted since, the page says
  "Made against revision 7; now 8", Accept is off, and the agent is
  woken to redo it.
- **Accepted:** a line in the chat, "Accepted at 14:03: new work takes
  it now", and "How work runs" shows the new revision.
- **Later, a delegates' option** under "Let them finish": "and their
  new delegates too" (section 4).

### 5.3 What it cannot do

- **What temper cannot model,** it says, and offers the nearest thing:
  a goal's own branch, a coordinator's wake policy, the other gaps
  `reference-code-production.status.md` records. Each such answer is
  design input.
- **What needs a restart,** it says too: a provider the document does
  not name, a listener, a worker, a limit (3.1).

### 5.4 Through `temper ctl`

The same way, a command for each act (`../client/tools.md`, section 5):

| The person's way | The command |
|---|---|
| Tell temper how you work | `chat start --project PROJECT --template 3 TEXT` |
| the conversation | `task message TASK TEXT`; `chat show TASK` |
| the card, and an owner's inbox | the chat's answer naming the proposal; `inbox --kind proposals` |
| the review page | `proposal show PROPOSAL`: the values now and proposed, the revision it was made against, and its reason, under *temper enforces* and *your agents follow*, with its effect on running work (`../domain/agent.md`, 8.1) |
| Accept, Reject | `proposal decide PROPOSAL accept\|reject [REASON] [--running stays\|moves]`; a rejection's reason reaches the chat as the person's words |
| How work runs | `settings show`, `settings show charter NUMBER [--revision REVISION]`, `revision show REVISION`, `revision list` |
| Move them, Restore this version | `revision advance REVISION`, `revision restore REVISION` |

- **No act is a page's alone.** Whatever a page does, a command does,
  so that a person at a terminal, and a pass of QA, which drives temper
  as one (`../../development/qa.md`, section 4), can do all of it.

## 6. The configuring agent

- **Its tools are the requests a person may make,** through the same
  role and proposal checks: reads of the settings and of the
  documentation, and a change as one proposal. It changes nothing an
  owner is not asked about.
- **Its input** is the conversation, a pasted text, or a document in an
  adopted repository it reads by its path.
- **It explains:** "What happens when I start a goal?", "Why did the
  tester run on GPT?", answered from the settings, a task's revision and
  the documentation. And it describes the settings back as a workflow,
  which is also how its test reads (section 8).
- **Its charter** is a stock one, a default like the others (3.2).
- **Later, it is woken by a landing** of a document a charter names,
  when the landing touches structure: "process.md changed; here is
  what it means for your setup".

## 8. Its acceptance

The user's story, 2026-11-02: on a box that holds no recipe of the
reference, QA, driving temper through `temper ctl` alone, asks it to
configure itself the reference's way, and it works: a draft is carried
from intake to landing as the reference says. It passes in two stages,
the second once the reference case walks.

- **In the gate,** scripted stories on the fake LLM hold the
  mechanisms (`../domain/testing.md`, 6.4): a person alone; two
  people, one approving landings (a landing rule's approvals,
  `../protocol/configuration.md`, section 3); a proposal gone stale;
  running work moved; a document a charter names, changed. The
  revisions' own: a change during a run, a restart with old revisions in
  use, the bound reached. They accept nothing: a fake model says only
  what it is scripted to.

### 8.1 Stage 1: the proposal

- **The box:** `temper box` with its owned forge
  (`../protocol/setup.md`, section 7). Its settings are the stock
  defaults, and nothing in it is the reference's: neither the stock
  settings nor the documentation in the binary carry a recipe of it
  (section 10), so the agent works from the description alone.
- **What the box brings, not the agent:** what needs a restart (5.3)
  and what lives in repositories. The pass's brief lists it and the pass
  confirms each through `temper ctl` before the chat starts:
  - the providers and models the reference names (Opus 5.5 for the
    coordinator and designers, gpt-6.1-sol on smith's direct backend
    for producers, `notes.md`, N20), with the accounts they run on, and
    a real model for the configuring chat;
  - the limits: the stock's for this stage, which the proposal needs no
    more than; the reference's, sized for a goal of a hundred changes
    (`notes.md`, N21), once `reference-case.execution.md`'s M10 gives
    them, for 8.2;
  - the project, with rung C's repository adopted, holding what section
    14 asks of a repository: its `AGENTS.md`, its code and design rules,
    `.temper/pre-pr`, its merge style; and, at a path the brief names,
    `reference-code-production.md` and `reference-landing.md` as the
    person's process documents.
- **The pass plays the person** (`../../development/qa.md`, section 2),
  through 5.4's commands. Its first words name the two documents, as a
  person would: "set this project up to work the way these describe".
  It answers the agent's questions from them, and from its brief for
  what they leave out (which account the producers use, for one); it
  reads the proposal with `proposal show` and accepts it, running work
  staying as it is.
- **The golden:** `reference-code-production.md`, section 14, cut into
  obligations, a row's each field its own, held by the acceptance and
  never by the binary. Each is of one kind:
  - *a setting*: what the proposal sets, field by field: each charter's
    instructions, models, tool families, waiting and documents; each
    template; the landing rules for main and for goal branches; the
    forge's charters; the project's policy, its delegations and its
    tasks and depth; the question bound;
  - *the box's*: a prerequisite above, which the agent proposes nothing
    for, and names where the box lacks it;
  - *a gap*: one `reference-code-production.status.md` records, which
    the agent says temper cannot model (5.3), offering the nearest
    thing.
- **The check, by code,** of the accepted revision and the project's
  policy against the golden, obligation by obligation: each setting
  holds its value, and each gap is named in the proposal or the chat. A
  setting wrong fails it, though its row also holds a gap; so does a
  setting no obligation asks for, a rule made structure without the
  person's choice (section 2), or anything that needs a restart
  proposed rather than said.
- **The check, by judgement:** an LLM reads the accepted charters'
  instructions, and the documents they name, against the reference's
  rules (section 2's table): each rule reaches the runs of the role it
  concerns, from where the person wrote it. It reads too the agent's
  description of the accepted settings back as a workflow (section 6),
  asked once the proposal is accepted, against the reference's section 1.
- **It moves with the reference.** Each milestone of
  `reference-case.execution.md` names the settings of
  `reference-code-production.md`, section 14 it makes sayable; as one
  lands, its obligations move from *a gap* to *a setting*.
- **A walk,** a pass of QA after landing, run again when the agent's
  instructions, the documentation in the binary or a milestone of the
  reference change; never in the gate.

### 8.2 Stage 2: the reference case under it

- **v0.1.0's rung D** (`v0.1.0.md`, 2.1): 8.1 with no gap left, on
  the box with the reference's limits, then a person's draft started as
  a goal under the accepted settings, and carried from intake to landing
  as rung C carries `reference-case.md`, acts 1 to 5.
- **It differs from C by the agent alone.** C runs under
  `reference-code-production.md`, section 14's settings as the
  reference's milestones make them (`reference-case.execution.md`,
  M10); D runs under what the agent proposed, on a box that never held
  them.
- **It waits for C,** the user's choice of 2026-11-02: stage 1 runs as
  this draft's code lands, stage 2 once rung C passes.

## 9. Order

The first order's survey and designs are done: jig's
`domain/engine.md`, section 17; temper's `../protocol/configuration.md`,
section 13, `../domain/agent.md`, 8.1 and `../client/tools.md`, section
5. What is left, revised 2026-11-02:

1. **The design's own words** for what it still cites this draft for,
   and for what it says otherwise (this draft's status): among them, a
   change touching a project's own policy (3.3), which the acceptance
   needs, so that the draft can go once 8.2 passes.
2. **The code, against that design,** as the designs' statuses record
   it:
   - jig's: charters read at their task's revision, and a charter's
     documents (17.9), jig's and temper's both
     (`../domain/agent.status.md`, `../protocol/configuration.status.md`),
     which the reference's charters name too; no milestone of
     `reference-case.execution.md` builds them;
   - temper's: charter 3 and template 3, the configuring agent, with
     the `settings` family in the stock's policy and rules; the
     `settings` family offered only where a charter's `tools` names it;
     each charter leaving its brief room, checked at a start and at a
     change's entrances (`../domain/agent.status.md`,
     `../protocol/configuration.status.md`); `chat start --template`,
     the items of temper's kinds in a settings file and in a proposal's
     review, `settings show landing-rule` and `forge-charter`, and
     `revision restore` (`../client/tools.status.md`); the reference
     pages `configuration`, `configuration-engine` and `limits` the
     agent reads (`../protocol/documentation.status.md`);
   - not here: temper's kinds as settings, `application.landing` and
     `charter` gone from the document, which
     `reference-case.execution.md`'s M3 builds; this draft's execution
     takes them from there, its ctl and its agent following.
3. **The configuring agent's stories in the gate**
   (`../domain/testing.md`, 6.4).
4. **Stage 1** (8.1): the golden as settings items, its check in code
   and its judge, and QA's walk; designed beside the gate's stories in
   `../domain/testing.md` unless the execution finds a better home.
5. **Stage 2** (8.2), once rung C passes.
6. **The web client's pages** (5.1, 5.2): "How work runs", the card and
   the review page, their mockups first, designed and built separately
   (`../client/README.md`).

## 10. Open questions

- **Settings in a repository,** a source
  synced into the store's settings. What was said:
  - a local git repository in the deployment's data directory, written
    by temper: history, diffs and rollback, with no forge;
  - a repository on the connected forge, changes landed through
    temper's own gates: configuration as code, reviewed and public. Its
    hard parts: a start never depends on the forge (only on the last
    landed copy), credentials never live in it, and a forge that is down
    blocks changes, not running;
  - the fake forge, grown a real-git bridge, as an internal store; the
    bridge is designed but not built, and the fakes are test doubles,
    held in memory, with test-grade promises;
  - a split to weigh: what concerns working on a product (charters,
    gates, landing rules) in the product's own repository, as
    `AGENTS.md` is today; what concerns the deployment (workers,
    accounts, budgets, capacities) with the deployment.

  Charters naming documents (section 2) take part of it. Default: the
  store alone.
- **The reference as a stock recipe** in the documentation in the
  binary, which the agent tailors to a project (its repository, models,
  accounts, a tier's cadence) rather than deriving it each time; its
  settings are 8.1's golden. The user wants it (11-02). Default: once
  stage 1 passes without it. The documentation is compiled into the
  binary, so a box cannot leave it out; how the acceptance keeps
  running without it, a build without the recipe or a configuring
  agent told not to read it, is the recipe's design.
- **LLM providers** in the store too, by 3.1's rule. Default: later,
  a design of their own.
- **Delegates on their parent's revision,** as a change's option.
  Default: later.
- **Hand edits** on "How work runs", by an owner who is also the one to
  decide. Default: proposals only at first.
- **The bounds' numbers:** revisions in use, the charters they hold, the
  history kept. Default: from the first deployments, as the profiles'.
- **A release's new defaults** reaching settings never changed, with no
  proposal. Default: yes, said in its release notes and its history
  line.
