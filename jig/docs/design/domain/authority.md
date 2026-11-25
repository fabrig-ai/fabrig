# Authority

Provisional, 2026-10-07, revised 2026-11-18. What a task may do, and how
the core holds it there: authority as a value, its order and how
delegation narrows it, where it comes from, budgets and what is spent,
the checks every action passes, what a proposal needs and what accepting
it funds, and the requirements no task loosens. It is the fourth
primitive of model.md (3.4) in depth. Proposals travel the tree as
tasks/proposals.md says; parties' roles are people/README.md, section 4. What
is still open is listed in `authority.md`, section 12.

## 1. In one page

- **Authority is a value with an order.** It names tools, the models
  its runs may use, effects and reads on resources, what may be
  delegated and what may be spent. One
  authority is at most another when every part of it is: a preorder,
  decidable from the two values alone. Whether a delegate's authority
  **fits** under its creator's is that order, checked against what the
  creator has left.
- **Resources are named by paths, grants by patterns.** Each connector
  names its resources as paths of segments; an effect is granted on
  every resource a pattern covers. Patterns order themselves, so the
  authority child domain knows no connector, and the deployment's rules
  are written once for all of them.
- **Budgets are carved, not copied.** What a task gives a delegate of
  its budget is reserved from its own; what the delegate leaves unspent
  goes back when it ends, and what it spent is counted up the chain that
  funded it. A run of a task that may start delegates holds back at
  most half of what the task has left, so what it starts is funded,
  unless half is too little for a run, when it takes what is left
  (`authority-budgets.md`, section 1). Spend is counted in one unit of the deployment's: LLM
  completions as runs price them, and effects as their connectors price
  them.
- **Recorded spend never exceeds a budget.** A run reserves each
  completion's maximum before making it, or, on a program backend, where
  it cannot, holds one more turn in its reserve; a priced effect is
  charged its maximum when decided. The only spend beyond a budget is
  what a worker spent on turns it lost before they were committed, which
  is bounded (`authority-budgets.md`, section 1).
- **Every action is checked** before it commits: a batch, an effect, a
  read, a tool call, a run starting, a party's request, an amendment, a
  note. A check answers allow, wait, propose, or refuse.
- **A party and an agent pass the same check.** A party's batch and an
  agent's reach one check; only the source differs: the party's role and
  pool, or the creating task's authority and what it has left.
- **Requirements are verdicts.** The policy attaches requirements to
  kinds of effect; each names a connector, which judges it from its own
  facts: met, wait, or refuse. The connector making the effect need not
  be the one judging. A guarded requirement holds when the effect is
  applied; an observed one, when it is decided.
- **Beyond its authority, a task or a party proposes, explicitly.** An
  action beyond authority is refused, naming what it lacked; asked for
  as a proposal, the nearest holder above whose authority covers what the
  action needs accepts and funds it, or rejects it.
- **The root is the deployment's rules and the project's policy,** as
  they stand when a decision is made. No task, and no party, loosens
  them; a narrowing reaches every task's later decisions.
- **Pure policy.** The authority child domain decides over values it is
  given and keeps nothing between calls but the rules and policies in
  force, so it is tested by its step tests.

## 2. Structure

The authority child domain is a child of the core (engine/README.md,
section 3). It is policy: a decision over data. It holds the
deployment's rules and each live project's policy, which change only by
a commit (people/acts.md, 2). The numbers kept against each task's
authority (what it spent, what its ended delegates spent, what it
reserved, who funded it) are the tasks child domain's (tasks/README.md,
section 3); the core gives them with each question, and authority's
answer says what they become. Each child domain has its own type for
authority, which the core translates.

`stored_finding_limit(&Limits)` bounds the findings retained in one
stored denial value. It is the largest of the batch, run and effect
finding counts, with every independent reason included; arithmetic that
cannot represent the count refuses the limits. The core's stored-value
checks and the store codec use this bound. A check's output queue is
uncapped, so this count reserves no room between authority and its caller.

Its questions, each a pure function of what the core gathers:

- is this authority at most that one (section 5)?
- does this batch fit under its creator, a task or a party, what is a
  budget given `AsAllowed` (`authority-budgets.md`, section 1), and what do the numbers become
  (`authority-checks.md`, 1.1)?
- may this task, or this procedure, make this effect, given the verdicts
  its requirements received (`authority-checks.md`, 1.2)? Which verdicts does it need
  (section 10)?
- may this run start, and with what budget (`authority-checks.md`, 1.3)?
- may this party do this, in this project (`authority-checks.md`, 1.4)?
- what does this proposal need, and which of these holders covers it
  (section 9)?

## 3. What authority is

```
Authority
  tools       the tool families an agent's run may call
  models      the configured models its runs may use, and so their LLM providers
  grants      effects and reads, each a connector's kind on a pattern of resources
  delegation  executor kinds and charters it may delegate to; how many tasks; how deep
  budget      what it may spend, in the deployment's unit; by when
  notes       the scopes it may write notes in
```

- **Tools:** families, as an agent's charter names them (hosts/charters.md
  ): smith's own (inspecting the workspace, modifying it, the
  shell, sub-agents), each connector's reads, and the engine's tools
  (engine/executors-tools.md, 1), in families of their own: delegate; message, with
  `introduce`; amend and cancel, with `release`; decide, with
  `withdraw`; propose; subscribe, with `unsubscribe`; effect; note;
  recall; and asking, the tools an application declares to a run that
  explains to its person, whatever each reads or writes, decided as one
  family (engine/executors-tools.md, 1). A procedure has no tools; its authority's tools are empty. On
  a program backend, the program's own tools are these families too,
  each call asked of smith's policy before it runs (`authority-checks.md`, 1.5).
- **Models:** the catalogue's models a task's LLM choice may name, bit `i`
  the setting numbered `i + 1` (engine/settings-catalogue.md, 1), for
  its main model and its sub-agents' (tasks/README.md, section 3), and so the
  LLM providers whose accounts may pay for its runs (accounts-uses.md, 1.3);
  every configured model, unless the deployment's rules, the project's
  policy or a role names fewer (`authority.md`, section 6). A procedure runs no model;
  its models are those its delegates may name.
  A permission selector is omitted, literal ALL, or a named set of at
  most 64 unique configured names (each 4,096 UTF-8 bytes at most).
  A named empty set grants no model; a named set containing all 64
  remains frozen, distinct from ALL. Create omission inherits the
  creator's models; Amend omission keeps the task's. A role edit's
  omission keeps its models, or takes the project ceiling for a new role.
  Explicit role edits replace models under both ceilings and refuse
  atomically if they exceed either. Retained identity and effective
  restore mapping follow `../protocol/store/records.md`, 1.

- **Grants:** each is a connector, a kind of effect or read that
  connector defines, and a pattern of resource names (section 4):
  "restart the services under `staging`", "create environments in the
  pool `staging`", "read every service's logs". A task may make an effect
  or a read only if some grant covers it.
- **Delegation:**
  - *kinds:* which executors its delegates may have: agents with which
    charters, which procedures, people in which roles, jobs of which
    kinds (tasks/lifecycle.md, 3.1): a kind of its own, so that a policy grants
    its landings their jobs' kinds, never borrowed from a procedure's;
  - *tasks:* how many tasks its subtree may make over its life;
  - *depth:* how deep below it its subtree may go.
- **Budget:**
  - *spend:* how much its subtree may spend;
  - *deadline:* by when it must end, in wall time, if at all.
- **Notes:** the scopes it may write in (engine/briefs.md, section 2), a goal's
  meaning, for a root conversation, its own. It may read every scope its
  project reaches. A party's scope, in no project, is no note scope of an
  authority's: the asking family reaches it for the task's requester alone
  (engine/briefs.md, 2.1).

## 4. Resources and their names

- **A resource's name is a connector's number and a path** of segments,
  given by the connector: for `ops`'s infrastructure, `env`, the
  environment, `service` and the service's name; for a forge, the host,
  the repository, `branch` and the branch's own segments. Names are
  bytes, compared segment by segment.
- **A grant names a pattern:** segments matched exactly, then a last
  segment that is either exact or open, an open one covering every
  segment that begins with it, and every name beneath. One pattern is at
  most another when every name the first covers, the second covers too,
  which is decided segment by segment.
- **Names after a task.** A connector may name resources after the task
  that makes them (a branch per task, an environment per request), under
  a prefix of the deployment's. A batch names them symbolically ("its own
  environment", "its runs' branches") and resolves them once the task has
  its number, each with its grant, so a creator can narrow a delegate to
  its own resources without knowing its number.
  A batch member may also name a connector's **symbol**, a resource the
  connector names after the task and its tree (a forge's *its own goal
  branch*), which the connector resolves into the name the task holds
  (`connectors.md`, 3.1; tasks/holds.md, 2). A symbol adds a hold, never a
  grant: what the task may do to it is its authority's.
- **Kinds are a connector's,** with an order of their own where one
  implies another: scaling a service may imply restarting it, while
  landing into a branch does not imply pushing to it. A kind's order is a
  table the connector gives, and the authority child domain holds as
  data.

## 5. Order and fitting

- **The order.** Authority A is at most B when:
  - A's tools, models, kinds of delegate and note scopes are subsets of
    B's;
  - every grant of A is covered by one of B's: the same connector, a kind
    at most B's, a pattern at most B's;
  - A's tasks, depth and spend are at most B's, and A's deadline no later
    than B's (no deadline being the latest).

  It is reflexive and transitive, a preorder: two authorities with the
  same grants written differently may each be at most the other. The
  step tests check its laws (section 11).
- **Fitting.** A delegate's authority A fits under its creator B, given
  the numbers kept against B, when A is at most B with B's depth less
  one and B's tasks and spend replaced by what B has left. Fitting is
  what a batch, an amendment and an acceptance are checked against.
- **Narrowing** is giving a delegate an authority that fits. It is
  checked, never applied silently: a batch asking for more than fits is
  refused, naming what it lacked, and may be proposed (section 9); it is
  never trimmed to fit.
- **What is consumed and what is not.** Tools, models, grants, kinds,
  depth, deadlines and scopes are shared: a delegate given a grant does
  not take it from its creator. Spend and tasks are consumed: what a
  delegate is given is reserved from its creator (`authority-budgets.md`, section 1).

## 6. Where it comes from

```
deployment's rules       the most any project may have; requirements on effects
  project's policy       the most any task in the project may have; its roles; its requirements
    a party's role       what a party may give, accept and do in the project
      a root task        what its creator gave it, fitting under the creator
        its delegates    narrowed at every level
```

- **The deployment's rules** are a setting (engine/settings.md): the
  binary's default, with the change an owner of the deployment stored
  over it, read afresh by every decision: the connectors and the kinds
  of effect any project may be granted, the models any project may use,
  the deployment's spend per period, the charters and procedures it
  has, the limits, and the requirements every project's effects meet. A
  rule may forbid what no project may grant at all. A change of them is
  a narrowing or a widening below.
- **A project's policy** is in the store, whole, from its seed or, for
  a project made after the first start, copied from a configured policy
  (`../protocol/configuration-engine.md`, section 1); changed by the parties
  whose role allows it, within the deployment's rules:
  - its ceiling: the authority no task in the project exceeds, which
    follows from its resources' roles (owned, fork, context) and from
    what the application was given on each at adoption (connectors.md,
    section 12), and names the models its tasks may use. Its owners edit
    it whole, within the deployment's rules, refused past them: the way a
    rule widened by a change of settings (engine/settings.md, 2) reaches a
    project made before it, which otherwise keeps its copy. A ceiling
    below a role's authority is refused whole, naming the role: its
    owners narrow the role first. Any policy edit past a ceiling (the
    project's above the rules', a role's above the project's, a cut
    below a role's) is refused as lacking that ceiling, typed with its
    holder, dimension and value (`../client/domain/contract/answers.md`, section 1),
    never as a shape; the ceilings are judged before the open period's
    room (`authority-budgets.md`, section 1), so a raise past both names the ceiling first;
  - its spend per period, from which every root task's budget is carved;
  - its roles: for each, the authority a party in it has, the spend a
    party in it may allot per period, and the kinds of proposal it may
    decide, escalations among them (people/README.md, section 4);
  - its escalation role, if it names one: the role whose parties hold an
    escalation that passes every holder in its tree, and the party who
    requested it (tasks/proposals.md). A party becomes a root's
    requester only when their own role decides escalations or the policy
    names this role (people/acts.md, 1);
  - its overrides of the application's templates, per template number,
    which no check reads, since a template grants nothing: what they may
    override is `../protocol/configuration-templates.md`, 1, and how `me` serves
    them merged, engine/views.md, 1;
  - its requirements: per kind of effect and pattern of resources, what
    an effect there needs (section 10);
  - its projections' authority: the grants its goals' projections are
    made under in the project's homes, apart from any goal's own
    authority, since no task asks for a projection (connectors.md,
    section 8). Each projection effect is checked against it, its
    requirements included, as any effect is (`authority-checks.md`, 1.2), and it costs no
    task's budget.
- **The public party's role,** `public`, is fixed in code and in no
  policy: it grants the public reads of a public project and nothing
  else (people/public-reading.md). It has no number, and a policy, the
  rules and a seed name roles by number alone, so none can name it. No
  authority, spend or proposal reaches it.
- **A party's authority** in a project is their role's, and their
  allotment is a pool per period with its own amount left. A party who
  creates a task (a chat, a goal), or accepts a proposal, funds it from
  that pool, which is itself carved from the project's spend for the
  period.
- **A person's creating offer,** read by the application through
  `Core::creating_offer`, is the role's authority and models, its task
  allowance within the tree limit, its depth, and the spend its pool
  allows, before a root takes its own slot and level. The read admits
  nothing: a `Create` still checks the batch whole.
- **Permissions stay primitive.** A role says which executors and
  charters a party in it may create with (its delegation's kinds), which
  models its tasks may use, how deep and how much (its depth and spend),
  and which kinds of proposal it decides; never which template. A task
  filled in from a template is checked as any batch is (`authority-checks.md`, 1.1), so a
  template grants nothing a party could not create by hand.
- **The deployment** is the requester of what configuration or a
  connector starts on a project's behalf (a recurring task, a standing
  watch), with the authority the project's policy gives such tasks,
  funded from the project's spend. A procedure task
  it starts that runs no agent (a forge's landing or tier run, which
  only make effects) is made with a budget of zero, so that a period its
  people have reserved whole never stops it; an effect it asks for that
  has a price is funded at the effect from the period's unreserved
  spend, and waits while there is none. A deployment-started task that
  runs an agent (a queue's repair, whose producer is one) is funded as
  before, and cannot be made while nothing of the period is
  unreserved. A make the tasks child refuses for any such task is told
  to every connector as `Ask::RepairRefused`, naming the task the
  refusal names where it names one (`repair`), the make's own number
  (`number`) and the refusal (`why`), so a connector knows which of its
  makes failed and why.
- **Effective authority.** What a task may do when it decides is its own
  authority within the deployment's rules, the project's policy and what
  adoption found the application may do, as all three stand then:
  - *a narrowing* applies to every decision after it, existing tasks'
    included: a grant beyond the new ceiling is unusable, not removed,
    and a check that meets it answers refuse;
  - *a widening* never widens an existing task: its authority is what it
    was given;
  - *effects already committed* are made: they were decided under the
    policy that stood then. A party who wants them stopped cancels their
    task, which withdraws what has not been sent (tasks/changes.md);
  - *pending proposals* are judged again when accepted, against the
    policy as it stands then;
  - *a live run* whose workspace writes something the narrowing forbids
    is cancelled, and its task's next run starts within the new ceiling;
    its other calls are checked as they come.

### 6.1 A person's task in no project

An application may need one short agent task for a person who has no
project yet: to find out whether an account answers as a chat of theirs
would. Project 0 names no project and has no policy (people/README.md, section
3; model.md, 6.1), and this is the one kind of task it holds.

- **Who makes it:** the application, for a person it has authenticated:
  `Core::person_seed`, then `Event::PersonTask`. Its creator is
  `Party::Deployment { project: 0 }`, so what selects by a person
  requester (Chats, results, updates, the inbox) never selects it.
- **Its LLM choice** is the application's, in `Event::PersonTask`'s
  `llm`: none, so that it takes what a chat of theirs takes, or one from
  `Core::account_llm`, the seed's `models()` its models, which holds it to
  one account serving them (accounts-uses.md, 1.3).
- **Who pays:** the person's pool in project 0, `Funder::Pool { project:
  0, person, period: 0 }`. Its payer is the person, as a chat's is, so it
  takes what a chat of theirs takes (accounts-uses.md, 1.3). Only a key whose
  `uses` name a project alone doesn't cover it: it names no project.
- **Its money:** period 0's budget is the rules' `period_spend`, the
  pool's the rules' ceiling's spend, each opened at the first such task;
  the task's budget is the rules' `maximum_run_spend`, at most what the
  pool has left. No calendar renews them: what it spends stays spent, and
  a seed past the pool's room is refused.
- **Its authority:** the rules' ceiling with no tools, grants, notes or
  delegation, and its spend its budget, whatever its charter names.
- **Who sees it:** the person who pays may watch its Task document. No
  other document or read selects it but the deployment's attempts, where
  it is one more attempt.
- **A hold** has no role to escalate to, so it stays with its task until
  the application cancels it.
- **A restart** restores it, as any live task: whether it runs again is
  the application's, which cancels it first if it shouldn't.

### 6.2 A person's automatic work in a project

An application may start, in a project, work done for one person
rather than for the project: temper's brief, written for the person who
looks (temper's `domain/forge/attention.md`, 3). It is that person's
work, paid as theirs, where a connector's repair is the project's
(section 6, *The deployment*).

- **Who makes it:** the application, for a party with a role in the
  project: `Core::pool_seed(project, person) -> Option<RepairSeed>`,
  then `Event::ConnectorRepair` with that seed, as a project-funded
  repair is made from `Core::repair_seed`. Its creator is
  `Party::Deployment { project }`, unlisted and untracked, so what
  selects by a person requester (Chats, results, updates, the inbox)
  never selects it.
- **Who pays:** the person's pool in the project, `Funder::Pool {
  project, person, period }`, the period's and the pool's opening
  (`OpenPeriod`, `CarvePool`) in the decision that makes it where
  either is missing, as a person's goal opens them. Its payer is the
  person (accounts-uses.md, 1.1), so it takes what a chat of theirs in the
  project takes (accounts-uses.md, 1.3), a plan of theirs included, though
  the project has other parties.
- **Its money:** the project's ceiling's authority, its spend the least
  of the ceiling's spend, what the person's pool has left (its role's
  unopened allotment where the pool isn't open yet, as
  `Core::person_pool` reads it) and the rules' `maximum_run_spend`. A
  run returns what it leaves to the pool when it ends (`authority-budgets.md`, section 1).
- **None** where that spend is zero, the person has no role in the
  project, the project is not open or has no policy, or no task number
  is left; nothing is written. A make that the tasks child refuses (a
  pool the period can't carve) is refused as a repair's is, with
  `Ask::RepairRefused`.
- **The rest** is a repair's: the procedure's steps, its delegates
  funded from it, its deadline and holdings, a restart restoring it.

### 6.3 A task's goal made by its application

An application may make, for a live agent task, a goal that task
would make with a batch, from a decision of the application's own
rather than the task's run: temper's triager's repair goal, made by an
act of its tool that also writes the application's rows (temper's
`domain/red-repairs.md`, 4.3). It is the task's delegate, as if the
task had made it.

- **Who makes it:** the application, with a number allocated as a
  connector's repair's is (`Core::repair_seed`, else
  `Core::procedure_seed`), then `RepairSeed::goal(self, task: u64,
  context: &tasks::DelegationContext, goal: SeededGoal) ->
  Result<tasks::Event, tasks::Refusal>`, the `tasks::Event::Make` it
  returns sent in the same decision as the application's own writes.
  `SeededGoal { charter: u32, spec: tasks::Spec, contract:
  tasks::Contract, authority: tasks::Authority, llm:
  Option<tasks::LlmChoice>, budget: u64, tracked: Option<u32>, label:
  u32, flags: tasks::Flags, holdings: Box<[tasks::Holding]> }`.
- **Its shape:** an agent of `charter`, its creator and requester the
  task (`Party::Task`), funded by it (`Funder::Task`), its deadline
  `Named`, its wake the default; `flags` may name a closer
  (`tasks/lifecycle-endings.md`, section 3), and `holdings` are the
  application's, translated as a creation's are.
- **Its money and room:** its spend `budget`, refused `Funding` where
  the task's numbers leave less (budget less spent, spent below and
  reserved), never cut to fit; refused `Tree` where the task has no
  task left to make, and `Depth` where its delegation has no level left.
  `authority` is the application's narrowing of the task's own, one
  level shallower; the tasks child checks it within the task's, the
  closer's procedure among its delegation's kinds, and `llm` as any
  making's (`tasks/README.md`, section 4).
- **Checked before it is sent:** an application whose own writes stand
  only if the making does checks the `Make` first with
  `Core::check_make(&self, make: &tasks::Event) -> Result<(),
  tasks::Refusal>`, which changes nothing and runs the checks the
  making runs (its places, the funder's accounts, `authority`, the
  closer, `llm`), in its order, refusing as the making would. In the
  same decision the making then gives the same answer, so a refusal
  writes nothing of the application's and is answered in jig's words
  (temper's `domain/red-repairs.md`, 4.2).
- **The rest** is a delegate's: its questions and result reach the
  task, its reservation is the task's, a restart restores it.

## 7. Budgets and spend

Moved to [`authority-budgets.md`](authority-budgets.md), section 1.

## 8. Checks

Moved to [`authority-checks.md`](authority-checks.md), section 1.

## 9. Proposals

What may be proposed, what it needs, and what an acceptance does; where
proposals wait and how they move is tasks/proposals.md.

- **What may be proposed:** a batch beyond its creator's authority; an
  effect beyond its grants or its budget; a widening of the proposer's
  own authority (more budget, more depth, another grant, a later
  deadline); an amendment beyond the amender's authority. Never what the
  project's policy or the deployment's rules refuse, nor an effect a
  requirement refused. A party proposes a batch or an amendment, a
  widening of a task's authority among them: a party makes no effect,
  and their own authority is their role, which the policy sets
  (people/acts.md, 1). A change of settings may be proposed by a party
  with a role or by a run its charter lets (engine/settings-proposals.md, 1); no task's
  authority covers one, so only owners accept it, and accepting it
  funds nothing: a setting is spent from by what is made under it.
- **Proposing is explicit,** for tasks and parties alike. An action
  beyond authority is refused, naming what it lacked and whether a
  proposal could be accepted; who would accept it is the offer's to say
  (`../client/domain/contract/documents.md`, 3). It is proposed only when its
  proposer asks: a run's `propose` call, a procedure's step that
  proposes, a party's `Propose` (engine/executors-tools.md, 1; people/acts.md, 1). Nothing
  becomes a proposal by itself, so nobody is asked by accident, but a
  result's follow-ups beyond its task's authority, which its verdict's
  contract allows, and which become one proposal as the result commits
  (tasks/lifecycle.md, 7).
- **The proposer is whoever asked:** the task, or the party, never the
  task an amendment would change, so its decision, a rejection
  included, reaches the one who asked.
- **What it needs** is the least authority the action takes: for a
  batch, every tool, grant, kind and scope its tasks have, the sum of
  their tasks and budgets, their depth, their latest deadline; for an
  effect, the one grant that covers it, and its price; for an amendment
  or a widening, the authority it asks for its task. The holder that
  accepts funds all of it, whatever the proposer has.
- **An escalation needs** nothing more than standing above the held task,
  unless releasing it needs what the task lacks (budget, time, a grant):
  then it needs that, as a widening would.
- **Who covers it.** A holder covers a proposal when what it needs fits
  under the holder (section 5), its depth counted from the holder: a task
  above the proposer, or a party whose role decides that kind of
  proposal and whose pool has the budget left.
- **An acceptance funds the action** from the accepter: the budgets of a
  batch's tasks are reserved from the accepter, which is recorded as
  their funder; an amendment or a widening makes the accepter the funder
  of the task it changes, whoever proposed it, as `authority-budgets.md`, section 1 says
  ("Whoever widens a task funds it"), its grants covered by the
  accepter's; an effect is made with the accepter's grant, its price
  charged to the accepter. The action commits with the acceptance, as one
  decision; an effect's requirements are judged again then, as any
  effect's are.
- **An acceptance decides one action,** not a standing permission. A task
  that keeps needing more proposes widening its authority instead, once.
- **A rejection carries its reason** to the proposer, which may revise
  and propose again.

## 10. Requirements

Beyond what tasks may be given, the rules and the policy attach
**requirements** to kinds of effect, which hold whatever any task's
authority says.

- **A requirement** is attached to a connector's kind of effect on a
  pattern of resources, and names a judge: a connector and one of the
  requirements that connector defines, with the parameters it takes in
  that connector's terms, which the judging connector keeps as its own
  configuration. "Restarting a service in production needs observability's
  *healthy replica elsewhere*"; "scaling a service down needs
  observability's *load below 40% for 30 minutes*"; "landing into `main`
  needs the forge's *CI passed at this head*".
- **The authority child domain knows** only which judges each effect
  needs, whether each is guarded by the effect's kind, the freshness of
  each observed one, and what their verdicts were. It never knows what a
  requirement means.
- **Guarded or observed** (connectors.md, section 7). A requirement the
  effect's own system checks as it applies the effect is guarded, and
  holds when the effect is applied. Any other is observed: judged when
  the effect is decided, within a freshness the policy gives it, and it
  may stop holding before the effect is applied. A requirement that is
  safety-critical is marked as one that must be guarded, and an effect
  kind whose system cannot guard it is refused wherever the requirement
  applies.
- **Judged by its connector,** from that connector's own facts, for the
  exact state the effect names (connectors.md, section 7):
  - *met:* its facts, read afresh enough, hold;
  - *wait:* its facts are unknown or pending, saying what it waits for;
    it reads afresh, and the decision is asked again when they change;
  - *refuse:* a fact failed, saying which.

  The judge finds the subject of its facts from the effect's resource
  names, by its own configuration: the service a restart names is the
  service whose health observability reads.
- **Across connectors.** The connector judging a requirement need not be
  the one making the effect. The core asks each judge the effect needs,
  through the root, which routes each question to its connector and each
  verdict back (root.md, section 9).
- **Gates a plan adds** to one task (a review at a head, a person's
  approval) are its procedure's own, and add to these, never replace
  them.
- **Who decides what:** for each kind of proposal (a goal's budget past
  a threshold, an effect in production), the role that may accept it.
- **Requirements are checked twice:** by the procedure that waits for
  them, which is the mechanics, and by the core at the effect, which is
  the rule. A procedure that asks for an effect whose requirements do not
  hold is refused, which is a bug in the procedure, counted, and never an
  effect made.

## 11. The world

Authority, a decision over data, is tested by its step tests: the order
checked against an independent statement of it over generated values,
and its laws (reflexive, transitive, fitting never above the order);
carving and returning budgets over generated trees with funders, the
spend of a subtree never counted twice and never above what funded it,
with reservations of completions' maxima under many open sessions;
standing allotments renewed across periods with delegates live; every
check's answer over generated verdicts, freshness and prices, and a
run's budget against its reserve and its main model's most costly
completion, the strictest always winning; effective authority as policy
narrows and widens around live tasks; one batch checked under a task and
under a party with the same authority and numbers left, answering alike;
`AsAllowed` resolved over generated pools, roles and caps, never above
any of them, after a batch's named amounts; a party's root refused
whenever its escalations have no holder. The core's world exercises it
in place (engine/world.md, section 1), whose referee holds the first promise
of model.md, section 11.

## 12. Open questions

- **The deployment's unit,** and whether spend per period has a window
  that rolls or resets.
- **Costs over time:** an environment kept past what its effect was
  priced for, charged as it lasts rather than once.
- **Read grants:** whether reads within a project need grants at all, or
  are the project's for any of its tasks.
- **Delegated decisions:** whether a holder may give a delegate the right
  to decide some proposals from below it, as a coordinator might for its
  sub-coordinators.
- **Half short of the reserve:** whether a run in a task that may start
  a delegate, whose half is above the deployment's minimum but short of
  the run's reserve and the completions above, takes what the task has
  left as well, where that would fund the run (`authority-budgets.md`, section 1). The rules'
  figures and the charter decide how often it meets one: temper's stock
  minimum is two completions of the costliest stock model
  (`temper/docs/design/protocol/configuration/stock.md`), so it can meet
  only a run on a program backend, or one whose charter keeps more than
  one reserve turn.
