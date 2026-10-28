# Configuring temper by talking to it

Draft, 2026-10-23. A person describes how they want work to go, and an
agent configures temper to run it. It takes `first-release.md`'s
section 7 further, from a session with the user that day, whose turns
shaped it: settings that live in the store and never in the
configuration document, a change that new work follows while running
work keeps what it started with, and a split between what temper
enforces and what agents follow. It crosses jig and temper on purpose;
an owner splits it. Nothing here is adopted.

## 1. In one page

- **The person never learns the words.** They describe how they work,
  in their own words or by pointing at a document, as agents'
  `roles/owners.md` and `roles/director.md` describe ours. temper answers with a
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
  plain words, with what temper enforces apart from what agents follow,
  and what the change does to running work.
- **It explains, both ways** (section 6): it answers from the settings
  and the documentation built into the binary (section 7), and it
  describes the settings back as a workflow.
- **Its test** (section 8): given `owners.md` and `director.md`, it
  should produce the encoding `temper-on-temper.md` made by hand, as far
  as that draft's gaps (its 6.8) let temper say it.

## 2. Structure and rules

- **The test, for each sentence of a description:** would temper's
  engine act differently without it? If the sentence changes what
  temper wakes, schedules, permits, pays for, lands or holds, it is
  structure, and becomes settings. If only the agent reading it would
  act differently, it is a rule, and stays text that runs read.
- **`owners.md`, sorted:**

  | From `owners.md` | Kind | In temper |
  |---|---|---|
  | Owners on Opus, testers on Codex by default, Claude for taste | structure | the charters' models, and the effort a parent picks within them |
  | An owner runs on the machine its work runs on | structure | its charter's hosts |
  | A fresh session at each milestone's end | structure | the charter's resume; by phase is a gap (`temper-on-temper.md`, 6.8) |
  | Every change lands through `land`, its gate passed | structure | the landing rule |
  | A milestone's cap is 4 hours | nearly | a deadline on the sub-owner's template, but jig's deadline holds what runs too, where the cap lets it finish (6.8 there) |
  | At most four milestones per mission | rule | the owner counts; the engine has no such bound |
  | A tester reads the design only, never the code under test | rule | authority is per repository, not per path |
  | Upstream first | rule, using structure | the owner makes the dependency; the engine releases it |
  | Notes, the closing note, small tasks, a world's test cost | rule | text |

- **Rules say how agents use the structure.** "Upstream first" is a
  rule; the dependency it asks the owner to make is the engine's. The
  agent configures the mechanisms, and the rules stay where the person
  wrote them, so most changes to a process need no configuring at all.
- **A rule may become structure** where temper's model has it, and the
  agent offers it: "I can make the 4-hour cap a deadline temper holds,
  though it holds what runs too; 'at most four milestones' stays a rule
  your owners follow". The person chooses what is guaranteed. The
  tester's row is the honest answer the other way: it looks enforceable,
  and is not.
- **Rules must reach the runs.** A run reads `AGENTS.md` in its
  workspace's repositories (`../domain/agent.md`, 4.5), and a run with no
  repository has no workspace, so a chat cannot read `owners.md`. A
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
(`first-run.md`, 2.4). This draft moves the rest of what configures
work:

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
  anyone signs in), and, for now, the LLM providers and the forges'
  addresses. Those two follow the same rule later, each its own design
  (section 10): models are bit positions in authorities, and a forge's
  number keys its records.

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
  importing; an import of a file as a proposal is later (section 10).
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
  at its next run, when its charter is still there; otherwise it stays.
- **A change may move running work:** each task at its next run,
  checked the same way. A task whose charter the change removed cannot
  move, and finishes where it is.
- **What a task may do is never pinned.** Its authority, from the
  rules, its project's policy and the landing rules, is always current;
  what its runs are given is its charter's, pinned. A change that must
  reach running work now moves it at its next run; a live run goes on
  until it ends, or an owner cancels it.
- **Removing is free.** A removed charter lives in the revisions that
  hold it while tasks use them. That answers jig's open question on
  parts removed at a restart (jig's `protocol/configuration.md`, section
  9), and two of `temper-on-temper.md`'s, 6.8: retiring a charter (no
  new work under it; what runs finishes) and whether a charter's change
  reaches tasks already running (not unless they move).
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

### 5.1 Where they meet it

- **The stock defaults are enough to start.** The first run's wizard
  (`first-run.md`, 2.3) grows no step. After its Ready, and on an empty
  "How work runs" page, one offer: "Tell temper how you work".
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
  parking an owner, a cap that holds new starts only, the other gaps of
  `temper-on-temper.md`, 6.8. Each such answer is design input.
- **What needs a restart,** it says too: a provider or a forge the
  document does not name, a listener, a worker, a limit (3.1).

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
  when the landing touches structure: "owners.md changed; here is what
  it means for your setup".

## 7. Documentation in the binary

- **One source of user documentation, three readers:** the site's
  documentation pages, the command line's help (`temper help <topic>`),
  and the agent's context, in `temper box` and in a deployment, through
  a tool that searches and reads it.
- **Written for people who run temper,** not in the design documents'
  register: the concepts (a task, a goal, a charter, a template, a
  policy, a landing rule), recipes ("review before landing", "a tester
  that cannot land", "a weekly budget") and the limits.
- **Where its source lives** is open (section 10).

## 8. Its test

- **In the gate,** real-loop stories on the scripted fake LLM: a person
  alone; two people, one approving landings (a landing rule's
  approvals, `../protocol/configuration.md`, section 3); a proposal gone
  stale; running work moved; a document a charter names, changed. The
  revisions' own: a change during a run, a restart with old revisions in
  use, the bound reached.
- **The encoding as a golden:** `temper-on-temper.md`'s section 4 turned
  into a set of settings. `owners.md` and `director.md` go through a
  real model, and its proposal is scored against the golden, within
  6.8's gaps. A lab experiment, run when the agent's instructions or the
  documentation change, not in the gate.

## 9. Order

1. **A survey,** read-only, on Codex: what moving jig's `charters`,
   `templates`, `policies` and `rules`, and temper's `landing` and
   `charter`, into the store takes, against jig's runtime projects, which
   moved projects' policies into it whole; what each child copies from
   the core's settings; where lookups would take a revision; the change
   to the tasks family; the stock defaults in the binary.
2. **The design in jig:** settings in the store, their defaults, a
   change and its proposal, revisions. Then temper's: its settings, its
   defaults, the document's members gone.
3. **Mockups:** "How work runs", the card and the review page.
4. **The code,** jig's, then temper's.
5. **The documentation in the binary.**
6. **The configuring agent,** its stories, then the golden and the
   experiment.

## 10. Open questions

- **Settings in a repository** (from `first-release.md`, 7.2), a source
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
    `AGENTS.md` is today (`factory.md`: product knowledge is data in the
    product's repository); what concerns the deployment (workers,
    accounts, budgets, capacities) with the deployment.

  Charters naming documents (section 2) take part of it. Default: the
  store alone.
- **An import:** a file of settings made into a proposal, for operators
  who keep configuration in files. Default: later.
- **LLM providers and forges' addresses** in the store too, by 3.1's
  rule. Default: later, each its own design.
- **Delegates on their parent's revision,** as a change's option.
  Default: later.
- **Hand edits** on "How work runs", by an owner who is also the one to
  decide. Default: proposals only at first.
- **The bounds' numbers:** revisions in use, the charters they hold, the
  history kept. Default: from the first deployments, as the profiles'.
- **A release's new defaults** reaching settings never changed, with no
  proposal. Default: yes, said in its release notes and its history
  line.
- **Where the documentation's source lives.** Default: temper's
  repository, beside its design.
