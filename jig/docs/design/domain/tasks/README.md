# Tasks

Provisional, 2026-10-07, revised 2026-11-07. The core's tasks: what a
task is, how tasks are made, how each moves from asked to ended, the
resources it holds, what it hears and when it wakes, and how proposals
and escalations travel the tree. It is the first primitive of ../model.md
(3.1) in depth, kept by the tasks child, the center of the core
(../engine/README.md, section 3).
Authority, which every action here is checked against, is ../authority.md;
what an agent's run is, ../engine/executors.md; a connector's procedures,
../connectors.md, section 6; what a party asks, ../people/acts.md; a task
that stands, its budget per period, its sessions and its Now,
`standing.md`. What
is still open is listed in `README.md`, section 8.

## 1. In one page

- **The tasks child domain is the center of the core.** The other
  children serve it. It keeps every live task: its place in the tree,
  its dependencies, its lifecycle, its holds, its inbox, its
  subscriptions to other tasks and to timers, its budget's numbers, and
  the proposals and escalations waiting for a decision. It knows nothing
  of runs, connectors or clients: an executor is a kind and an opaque
  reference, a connector's news a message it is handed.
- **Batches, whole or not at all.** Tasks are made in batches, checked
  whole (no circle of waits, within the limits, within the creator's
  authority as ../authority.md answers) and committed at once.
- **Parties and agents ask alike.** A party's request and an agent's tool
  make the same batches, amendments, cancels and proposals, against the
  same checks; only the source of their authority differs. Nothing
  beyond authority becomes a proposal unasked (`proposals.md`).
- **Several operations, one decision.** A party's compound request
  cancels, moves, amends, releases, messages and stops in one decision,
  all or none (`changes.md`).
- **One lifecycle for every executor** (skein/docs/foundation/programming-model.md, 5.2): a
  task waits on its dependencies and its holds, is active while its
  executor works, closes (its run ended, its delegates closed, its
  effects settled, its holds released), and ends done, failed or
  cancelled. It may be held for a decision at any point before it ends.
- **A dependency may be added to a live task.** It stops the task going
  on and never sends it back: while it is pending, an active task is
  kept, starts no run, asks no effect that waits for dependencies, and
  ends no way but failed or cancelled (`lifecycle.md`, 8).
- **Holds are taken whole, and no waits circle.** A task takes every
  resource it writes at once, or waits with none. A task holding some
  may still wait, for its delegates or, kept, for its dependencies, so
  every new wait is checked to close no circle (`holds.md`, 7).
- **Decisions go up the tree.** What a task may not do alone, and what
  holds it, waits for the nearest task or party above it with the
  authority to decide, and wakes them.
- **News is kept; only some wakes.** Each task has a durable, bounded
  inbox and a wake policy over what arrives.
- **The tree is the plan's history.** Every task made, amended,
  cancelled, released or held, and why, is kept in the store as it
  commits.

## 2. Structure

The tasks child domain is a child of the core (../engine/README.md, section 3),
and its center: it keeps every live task, and the other children serve
it. Its state is a slab of live tasks, a slab of pending proposals,
their inboxes, subscriptions and holds, and the timers it arms. It is a
step machine of its own (skein/docs/foundation/programming-model.md, 4.5): its own
vocabulary, limits and world.

What it does not know, and how it is told:

- **Executors** are a kind (agent, procedure, person) and a reference
  the core gives it: for an agent, a charter's number; for a procedure,
  the connector that owns it (or the core) and an opaque code; for a
  person, a party or a role, the task's project's role (../people/README.md,
  section 1). It asks for a task to be activated, and hears when the
  activation ends.
- **Authority** is a value of its own type, which it carries and keeps
  the numbers of (budgets spent and reserved, tasks made), as it keeps
  every other funder's (../authority.md, section 7), and never judges: every
  check goes to the authority child domain, whose answers say what the
  numbers become.
- **Resources** are names (a connector's number and a path) and the kind
  of hold each takes; what they are is their connector's.
- **Connectors' news** reaches it as messages already classified (`messages.md`, 3).
- **The store** is the root's: this domain's saves and erases are part of
  the decision the core's step is making (../engine/README.md, 5.6).

## 3. A task

What the store keeps of a task, and the tasks child domain holds while
it is live:

- **Its number,** given by the core, never reused, and its project.
- **Its spec:** words, bounded; its typed parameters, numbers and texts,
  each text UTF-8 as the words are, never other bytes; the resources it
  reads and writes, by name; the ended tasks whose results it takes as
  **inputs**; and what its executor needs: a charter's number for an
  agent, the question for a person. A procedure's parameters are its
  connector's, kept by the connector under the task's number (../model.md,
  section 5). A value that is not text, such as a commit's id, goes in
  as text its application spells (`../../client/domain/contract/documents.md`, 3).
- **Its result contract:**
  - *a report:* words, bounded;
  - *a verdict:* one of a closed list the spec gives, each with a
    contract of its own: which fields it requires, how many follow-up
    tasks it may propose and of which kinds;
  - *a connector's result:* what a connector's procedure produces, such
    as a change landed or an environment ready; the connector keeps it
    under the task's number, and the task keeps words that summarise it. A
    change-shaped result also keeps an explicit title as opaque bytes,
    independently bounded by the contract's words cap and the deployment's
    result-byte limit. The title metadata is boxed to keep inline result
    carriers small. An oversized title is refused before charging its
    attempt. Tasks accounts for both allocations and persistence
    retains the title; inbox result notices carry the summary words.
    A change contract also accepts a successful `NoChange` result: an
    explanation within both word bounds, with no resulting resource. It
    ends done and its requester receives a distinct no-change result notice;
  - *a choice:* one of the options the spec lists, or an answer in
    words, for a person;
  - *an approval,* for a person, of a state a procedure names: the
    state as bytes its connector gave, which the core carries and
    compares but never reads, and the part its connector shows it by.
    The result names the state it approves, which must equal the
    contract's, byte for byte (../people/README.md, section 7).

  Every result may also be a failure, with a reason: the executor's
  verdict that the task cannot be done as asked (`lifecycle.md`, 7). Declining an
  approval is that failure.
- **Its requester:** a task, a party, or the deployment, for what its
  configuration or a connector starts on a project's behalf (a recurring
  task, a standing watch). Its **proposer,** when an accepted proposal
  made it requested by the accepter (`proposals.md`).
- **Its dependencies:** tasks it waits for, each done before it goes
  on: those its batch named, which it starts after, and those an
  amendment added (`changes.md`), each kept with when it was added and by
  whom.
- **Its references:** tasks it may message beyond its requester and its
  delegates (`messages.md`, 5).
- **Its executor** (2) and, for an agent, its **charter**: the role's
  instructions, the models it allows, in order, its resolution in the
  catalogue (../engine/settings.md, 11), the tools, the outcome it must produce, its default wake policy
  and waiting time, whether a run resumes its transcript or starts
  fresh, and which hosts may run it (../engine/executors.md, 1); and its **parts**, in
  order, each with instructions of its own for some of the reasons a run
  runs, which a run those reasons meet reads after the charter's (`messages.md`, 6).
  Each model declares its efforts, in order, in its `model` setting
  (../engine/settings.md, 1); a charter's model is that row but for its `effort`,
  which selects one of them or none, for the row's own default.
- **Its revision:** the settings' revision it was made in, the newest
  then, whatever its parent's (../engine/settings.md, 3). An agent task's runs
  read its charter from it, until a move sets it to a newer one
  (../engine/settings.md, 4); each claim keeps the revision its run was prepared
  under. Every task keeps it, live and ended; only an agent task's holds
  its revision (../engine/settings.md, 3).
- **Its LLM choice,** for an agent task, what its runs' LLMs are:
  - *main:* a configured model, and an effort among those the model
    declares (`../../protocol/configuration.md`, section 3); its runs use
    it, and their check holds its most costly completion (../authority.md,
    8.3);
  - *sub-agents:* the models its runs' sub-agents may use, each with an
    effort, among which smith chooses within a run (smith's
    `domain/run.md`, 5.3);
  - *account:* optionally, one LLM account its creator wants in
    particular, within the account's uses (../accounts.md, 4.3);
  - *later:* optionally, a second main model with its effort and the
    sub-agents' models with theirs, which every run whose reasons hold no
    `first` takes (`messages.md`, 6), the first choice going to the runs of its first
    activation. It names no account: the task's preference covers both.
    Without it, every run takes the one choice. A task whose charter resumes
    takes none (`messages.md`, 6): a batch or an amendment giving one there is refused as
    `Llm` with the fault `Charter`, naming its main model, and a run of a task
    whose charter has come to resume since takes the first choice.

  Its creator gives it, or a template does
  (`../../protocol/configuration.md`, 3.1), each model within the task's
  authority (../authority.md, section 3). A task given none keeps none: each
  of its runs' checks takes its models from its charter, within its
  authority, the first it can use its main and the others its sub-agents'
  (../accounts.md, 4.3); each takes the effort its charter selects, or else
  its model's default (`effort`, `../../protocol/configuration.md`, section 3),
  or none where it declares none, which leaves its LLM provider's default. Its document shows what its next run would take
  (`../../client/domain/contract/documents.md`, 1). A batch or an amendment that
  leaves such a task with an authority allowing none of its charter's
  models is refused, naming the charter's first.
  A `later` is checked as the first choice is, model by model: within the
  task's authority at its batch or amendment, within its charter's models
  and their declared efforts at each run's check, and again on a restore.
  A task keeps the choice it was given, and only an amendment changes it,
  whole, its `later` with it: one giving a choice without `later` leaves the
  task none (`changes.md`).
  Configuration templates choose models and efforts; they never name an
  account. Runtime requests supply the optional account preference.
- **Its account:** the LLM account its runs use for its main model's LLM
  provider, chosen at its first run's check and kept from run to run,
  so that a conversation stays on one account's cache and replay, until
  a change of its payer or of that LLM provider chooses again
  (../accounts.md, 4.3), as its first run to take a `later` on another LLM
  provider does. The accounts its sub-agents use are each run's,
  recorded in its claim (../engine/executors.md, 1, step 3). This kept number is separate
  from the creator's optional preference. Only a successful claim changes it,
  in the same decision as its reservation and phase; a refused claim leaves
  it unchanged, and clearing the optional preference alone retains it.
- **Its authority** (../authority.md, section 3), and the numbers kept
  against it: spent by itself, spent by the ended tasks it funded,
  reserved for the live tasks it funds, and who funded its own budget.
  Its budget is an amount, whether its creator gave one or `AsAllowed`
  (section 4).
- **Its wake policy** (`messages.md`, 3) and **its subscriptions** (`messages.md`, 4).
- **Whether it is tracked,** and its priority if so: a goal (../model.md,
  6.2). The engine acts on the mark: it orders the queues of write holds
  by it (`holds.md`, 2), subscribes the goal to its connectors' goals' topics
  (`messages.md`, 4), has it projected (../connectors.md, section 8), and gives it a
  scope for notes (../engine/briefs.md, section 2).
- **Its label:** an opaque 32-bit number the application gives, through
  a template or by hand (`../../protocol/configuration.md`, 3.1). jig gives
  it no meaning, and no rule of the engine's keys on it: task queries
  filter on it, and the indexes of ended tasks are keyed by it
  (`../../protocol/store/README.md`, 4.4).
- **Its flags,** each with the engine's semantics:
  - *conversation:* its turns are entries for its requester, while that
    requester is a party, with their read position and an unread count
    (../people/README.md, section 5), and its run is asked for its text as it is
    written (../engine/executors.md, 1). Only an agent's run has turns, so the flag is
    refused on any other executor (`README.md`, section 4);
  - *listed:* once ended, it is indexed by its project and its label, so
    its project's lists find it. Every ended task a party requested is
    indexed by that party and its label, listed or not
    (`../../protocol/store/README.md`, 4.4);
  - *its retention class,* what becomes of it once it has ended: *kept*,
    to the deployment's retention horizon, and whole while none is set
    (`../../protocol/store/README.md`, 6.1); or *forgotten once read*, erased once
    its requester, a party, has read it. When and how is ../people/README.md,
    section 5. Whatever its class, a task of a project being deleted is
    erased whole once every task of the project has ended, with no
    horizon waited for, as its retention would erase it at the horizon,
    its transcript's file with it (../people/acts.md, 2;
    `../../protocol/store/transcripts.md`, 5, and `../../protocol/store/README.md`, 6.1).

  The label and the flags are what a task is made as: no amendment
  changes them (`changes.md`).
- **Its times:** when it is made, when it last changed, when it ended,
  and its **last activity**: when it last heard words, its run last
  committed a turn, or one of its delegates last ended, and until then
  when it is made. Task queries order by them (../engine/views.md).
- **Its delegates' endings:** how many of its direct delegates have ended
  done, failed and cancelled, kept as each ends, so that a list of tasks says
  how far a task has gone without reading its delegates. A delegate moved
  away is not counted, and a retained delegate's erasure takes nothing back.
- **Its closer,** while one lives: its number (`lifecycle.md`, 11).
- **Its holds:** the resources it writes and holds alone, or the pool
  slots it holds; or, while it waits for them, its place in their queues
  (`holds.md`).
- **Its saved resources:** those a run of it left saved state on, which
  its next run starts from (../hosts/README.md, section 6).
- **Its phase** (`lifecycle.md`) and why, if held; its **attempt number,**
  which only grows, fencing stale runs; and its **tries** per class of
  failure, which a release resets.
- **Its reasons** (`messages.md`, 6): why its activation runs, `first` until a run of it
  parks idle or finishes, with the kinds that woke its runs since; and, with
  its claim, the kinds its attempt is offered. They are kept on the task
  across its attempts, so a retry and a continuation run for what the
  attempt before them ran for.
- **Its session,** for a standing task: the record of its open session,
  or none (`standing.md`, 5.1).
- **Its result,** once it has one.

What the store keeps beside it, and the engine loads only when wanted:
its transcript (../engine/executors.md, 2), its history (every change made to it,
by whom, and why), and the results of its ended delegates. While a task
that ended is named by a live one (a dependency's result not yet read,
an input, a reference), the core keeps a stub of it: its number, its
phase, and where its result is.

What a task **waits on** is not kept: the engine computes it when a
document shows the task, since only the engine holds the tree and the
inbox together (../engine/views.md). It is a party, and who, for a
question, a proposal, an escalation or a person task in its tree; its
dependencies, for a kept task (`lifecycle.md`, 8); its delegates; news; or nothing.

## 4. Batches

A batch is what one decision makes: one task or many, with the
dependencies among them.

- **Made by** an agent's `delegate` tool, a procedure's step, a party's
  `Create` (../people/acts.md, 1), an accepted proposal, or a recurring task at
  a period. Its tasks' requester is the task or the party that made it,
  or the accepter of the proposal that asked for them to be its
  (`proposals.md`).
- **A party's batch is an agent's.** Each task names the same things,
  whoever creates it: its executor, spec, result contract, authority,
  budget, deadline, wake policy, dependencies and inputs, the resources
  it will write, whether it is tracked and its priority, its label and
  its flags, and an agent task's LLM choice (section 3). Both pass the
  same checks; only the source of the creator's authority differs: a
  task's authority and what it has left, or a party's role and pool
  (../authority.md, 8.1 and 8.4).
- **A party's root has someone to escalate to.** Every task a party
  requests is the root of a tree, whose escalations wait for that party
  or past them (`proposals.md`). A party becomes a root's requester only
  while its escalations have a holder (../people/acts.md, 1); otherwise the
  request is refused, whatever authority it fits.
- **Dependencies** name tasks in the same batch, or live tasks the
  creator reaches: for a task, its own delegates, or a task it is
  introduced to (`messages.md`, 5); for a party, a task in their tree, or any task of
  the project for a maintainer (../people/acts.md, 1). An amendment may add
  more once a task is made (`changes.md`). Admission checks the batch's new
  waits with the live ones (`holds.md`, 7): its tasks' dependencies, the queues
  they join and the holds handed down to them, and its creator's wait
  for them. So no new task depends on an ancestor or waits in the queue
  of a resource an ancestor holds, and no two subtrees join into a
  circle of waits. A result of a task that has ended reaches
  a new task as one of its inputs, not as a dependency, within the same
  reach.
- **A budget as `AsAllowed`.** A creator gives each task's budget as an
  amount, or, for one task of a batch, as `AsAllowed`, which the
  authority child resolves as it checks the batch (../authority.md,
  section 7); the task keeps the amount, and the batch's answer names
  it.
- **A deadline as a span.** A creator gives each task's deadline as a
  wall, as a span from the decision that makes the task, or none;
  authority makes a span a wall as it checks the batch (../authority.md,
  section 7), and the task keeps the wall.
- **Checked whole,** before anything of it exists:
  - every task well formed: a known executor kind, a charter or
    procedure the deployment has, a result contract its executor can
    produce, its spec within its limits, its words and text parameters
    UTF-8 (section 3), the conversation flag on an agent's task only,
    and an LLM choice on an agent's task only, its models configured
    and its efforts among those its models declare, and each of its
    models, main and sub-agent, one its charter allows at the newest
    revision (section 3), as a template's and a move's are checked
    (`../../protocol/configuration.md`, 3.1; ../engine/settings.md, 4). Whoever
    makes the batch, a party, a run's delegation, a procedure's step,
    an accepted proposal or a recurrence, the check is the same. The
    tasks child answers it alone too, for a charter and an authority's
    models, to a root that checks a choice before the batch that will
    carry it is asked for (a procedure's parameters naming one);
  - at most one task giving `AsAllowed`;
  - no circle of waits, with the live ones (`holds.md`, 7);
  - within the limits: its tasks' count and their bytes in all, live
    tasks in the deployment, tasks in the tree, depth below its root,
    delegates of the creator, goals in its project (section 6);
  - within authority: each task's authority fits what the creator has
    left, its budgets carved from what it has left (../authority.md, 8.1),
    or, for a party, fits their role and their pool (../authority.md, 8.4);
    its LLM choice names models its authority allows, and an account
    only one whose uses serve its payer (../authority.md, 8.1).

  A batch that fails a check other than authority is refused, saying
  which task and why, and nothing of it is made. One that fails only on
  authority is refused naming what it lacked, and whether a proposal
  could be accepted; its creator may propose it instead (`proposals.md`). It
  never becomes a proposal on its own, a party's no more than a run's.
- **Made at once.** Its tasks, their dependencies, the creator's carved
  budgets and the holds handed down commit together; its tasks with
  nothing left to wait for become active in the same commit.
- **Resources named for it.** A creator names the resources a task will
  write: resources it holds and hands down, the hold moving with the
  commit (`holds.md`, 4); resources named after the new task, which the batch
  resolves once the task has its number (../authority.md, section 4); or
  any other its authority covers, which the task takes when it is ready
  (`holds.md`). Which of those a task will hold, besides what is handed
  down, each connector says from its executor and spec before the batch
  commits, whoever creates it. So a party's batch names its tasks'
  resources by their specs and the grants named after them, and hands
  down none, since a party holds nothing (`holds.md`, 1).
- **Procedure tasks' parameters** go to their connector in the same
  decision, under the numbers the batch gives (../connectors.md, 6.1).

## 5. Core procedures

Procedures the core owns, stepped by this child domain:

- **Recurring.** A task whose procedure makes the batch it was made
  with, its recurring batch (never one of the configuration's
  templates, ../model.md, 7.3): at each period, as the period reaches the
  core from the application's root, it makes its recurring batch as its
  own delegates, within its authority, its budget carved again each
  period from the project's spend for that period (../authority.md,
  section 7). If the last batch is still live, the period passes, or the
  new batch waits, as its maker chose. A recurring task ends when it is
  cancelled.

The core has no other procedure. A static plan is a batch; waiting for
several tasks is a dependency; anything that needs a condition over
results is a coordinator's judgement. A standing task that makes tasks
from news is a connector's procedure (../connectors.md, 6.6). Both kinds of
standing task have their allotment of tasks and spend renewed each
period (../authority.md, section 7), and so does an agent task made
standing (`standing.md`, section 4), which also ends only when it is
cancelled.

## 6. Limits

Each is configured, refused at the entrance, and part of the engine's
worst case (../engine/README.md, section 7):

- live tasks in the deployment, and per project, and the stubs of ended
  ones;
- goals per project: the live tracked tasks in one project, refused at
  the entrance like live tasks per project, whether a batch, an
  amendment or an accepted proposal tracks them;
- tasks per tree, and the depth of a tree;
- delegates and dependencies per task, and batch size;
- a batch's bytes: what its tasks hold in all, each task's record and
  what it owns (its spec's words and parameters, its contract's choices
  and their fields, its authority's grants, scopes and delegation, its
  dependencies, holdings and LLM choice), a recurring task's template's
  tasks among them. The store prices a batch by these bytes
  (`../../protocol/store/records.md`, 1). A batch past them, or past its size, is
  refused as past the batch's limit (`Refusal::Batch`: a run's tool
  answers `Refused: batch`, and a party hears the engine's batch limit)
  wherever a batch is checked: made, proposed, kept as a recurring
  task's template, and at restart against limits lowered since; a
  result whose follow-ups pass it breaks its contract (`lifecycle.md`, 7);
- a spec's and a result's bytes, and inputs per task; a verdict's
  fields are counted apart, at most 512 (`VERDICT_FIELDS`, the store's
  schema's ceiling), never from a result's bytes, so a result's byte
  bound may grow without moving what the store can hold of a verdict;
- messages and bytes per inbox, and a message's bytes, within what a run
  reads (`messages.md`, 2);
- subscriptions, references, holds and saved resources per task;
- waiters per resource's queue, and how long a task may wait for holds;
- the tasks a circle of waits names, in a refusal or a hold (`holds.md`, 7), and
  those a change of settings refused as full names (../engine/settings.md, 2);
- pending proposals per task, and in the deployment; open questions per
  task;
- tries per class of failure, and their backoff;
- the stall of a person task, of a pending proposal or escalation, and
  of an agent task idle with delegates still live, but a standing one
  (`standing.md`, section 2);
- a grace's span: the longest grace a cancel may give;
- placed holds per project and per party, live at once, and the
  exemptions one holds (`lifecycle.md`, 10), which a start refuses past the live
  tasks: a profile that shrinks its live tasks shrinks the exemption
  bound it inherits.

A live task is never evicted: when the deployment is full, new batches
are refused, and the creator hears why.

A compound request is bounded by the people child's operations per
request (../people/acts.md, 1), and its worst case is ../engine/README.md, section 7's.

The models each charter's tasks may choose are kept per charter
version, for the charters and the versions held together, the core's
limits (../engine/settings.md, 8); this child's live tasks per revision are
counted within the revisions held.

## 7. The world

The tasks child domain has a world of its own: its parent, the
executors, the connectors' news, the holds' kinds and the authority
checks scripted, its expectations in a referee (../testing.md, section 3).
Its stories: a batch made whole or refused whole; a plan of reports, a
choice and procedure tasks, run in dependency order; a delegate failing
past its tries, held, escalated two levels to a person, released and
finished; a watch woken once by a burst of news; a cancel closing a tree
of three levels with runs live and effects in flight; an amendment
reaching a live run; a person's words left unread as a run ended, waking
its task again; a proposal routed past a procedure to a person and
accepted, and one that stalls and passes up; a chat's goal accepted as
its person's, outliving the chat; an inbox filling, its news merged,
words refused, a result still taken; two tasks wanting the last slot of
a pool, the second waiting and taking it as the first closes; two tasks
each wanting two holds the other wants, neither waiting in a circle; a
pool shrinking while its slots are held and a task waits, admitting
nobody until it drains; a standing task making delegates across many
periods, its allotment renewed each time; a verdict whose follow-ups
go beyond its task's authority, their proposal accepted and the
follow-ups run before the result reaches the requester, and another
whose proposal is rejected, its result delivered at once.

Its stories of placed holds (`lifecycle.md`, 10): a person's stop
of a goal, its runs and its delegates' runs cancelled, a maintainer's
release refused for it as the person's, the coordinator above it
proposing the release, the person accepting it in their inbox, and the
goal resuming from its transcript; a project drained while one run is
mid-turn and a procedure mid-step, the run parking at its turn's end,
the procedure going on until its connector says it is idle, a due task
getting no run, a delegation by the parking run refused naming the hold,
the person's chat answering throughout, then the drain lifted and the goals
released one at a time; a project stopped, a task made under it held,
that one task released and ending while the stop stands, and the stop
lifted with its tasks; a coordinator parking a delegate until a news
event, the hold lifting itself as the event's commit delivers it, and
another parked until a time; a restart in the middle of a drain,
draining on.

Its stories of added dependencies (`lifecycle.md`, 8; `holds.md`, 7; `changes.md`): the
reordering, two tasks waiting for one write hold, the first given a
dependency on the second, leaving the queue, the second taking the
hold and ending, and the first back at its place, ahead of a third that
queued after it; a procedure task kept, its effect that waits for
dependencies refused and its result refused, stepping again when its
dependency is done and ending done; a person task kept, its answer
refused naming the dependency, then taken; a dependency on a person
task, answered, and its dependent going on; a held task given one and
released, kept; circles refused, naming their tasks: A after B, then B
after A, a task after its ancestor, a holder after B while a task under
B waits in the holder's queue, and a batch whose task waits in the queue
of a resource its grandparent holds; a waiting task whose dependencies
are done joining a queue onto a circle, held, and back at its place
once the circle breaks; a pool shrinking onto a circle, its waiter held;
a failed task's hold retained by its root onto a circle, its waiter held;
a restart restoring a circle an older rule allowed, its waiter held; a
pool's two holders each given a dependency on a task that waits, through
its delegate, for that pool: the first accepted, the second refused,
naming the knot; a waiter joining a pool onto a circle through a holder's
delegate, held, and lifted when that delegate ends; a prerequisite
already done, met at once, one failed, refused, and one that fails
later, its dependent held, and released, held again; one removed, its
task woken; a grace given to a kept task with no live run, cancelled at
once; a proposal from below a kept task waiting above it.

Its stories of what parties and agents ask alike: a party's batch of an
agent's task and a person task, made as an agent's would be; a party's
batch beyond their role refused, naming what it lacked, then proposed
and accepted; another party's proposal rejected, the rejection reaching
that party, not a task; a party refused a root while their escalations
have no holder, and made once the policy names one; a budget given as
`AsAllowed` beside named amounts, resolved to what the pool has left
after them and named in the answer, one that comes to nothing refused,
and a batch where two tasks give it refused; an agent's amendment and a
party's release each named with a revision that has moved, refused as
moved; a party withdrawing their proposal; a goal tracked past goals per
project, refused at the entrance, and one untracked, its projection's
last write made; a conversation closed in one compound request, one
delegate moved to its party, another cancelled, and the conversation
cancelled with a grace in which its run finishes, ended done; another
whose grace ends first, cancelled; one compound request refused whole
when its third operation fails, nothing of it made; a project's goals
reordered in one compound request, its priorities naming no revision; a
task's last activity moving with words, a turn and a delegate's end; a
conversation forgotten once its party has read it, and one kept.

Its stories of revisions (../engine/settings.md, 3 and 4): a task made by a
batch, a delegation, a procedure's step, a recurring task and an
accepted proposal, each on the newest revision, a delegate on the
newest whatever its parent's; a task's LLM choice amended, checked
against its own charter version's models, not the newest's; a move
refused for a charter the newest drops or an LLM choice it no longer
allows, the task staying; live and ended records restored with their
revisions and claims; the count of live tasks per revision reaching
zero as the last one ends.

Its stories of a task that stands are `standing.md`, section 8's.

Its referee: a task starts only after its dependencies are done and its
holds taken; batches are made whole or not at all, and compound requests
too; one run at most per task; a resource has one holder, and a pool
admits no holder beyond the slots it is known to have;
no task in a queue holds a resource, and no circle of waits forms; no
kept task starts a run, asks an effect that waits for dependencies, or
ends done; every result
reaches its requester once, after the task has closed; a cancel ends
every task below it, deepest first, and one with a grace ends its task
by the grace's end; no amendment or release is made against a revision
that moved; nothing becomes a proposal unasked but a result's
follow-ups; no committed message is lost, and none is refused once
committed; every proposal and escalation waits where a holder can decide
it, and keeps across a restart the holder, entry and time it was routed
to, since a restart decides nothing again (../engine/restart.md): only
a new escalation, a pass or a decline routes one; nothing beyond a
task's limits is held, goals per project
included; every task's revision was the newest when it was made, or
the newest when it moved.

## 8. Open questions

- **Replacing at the deployment's proposal bound** (`proposals.md`): whether
  the fresh settings proposal may count the set it replaces as room.
  Default: the core checks the bound before withdrawing anything; a full
  bound refuses the fresh proposal and leaves the old one pending
  (`../engine/README.md`, 10, and `../engine/settings.md`, 6).

- **Wake defaults:** which kinds wake each charter's tasks, and how
  batching is tuned; whether a cheap model should triage news before a
  coordinator wakes. Tuned from use.
- **Later, for parts:** a part given as a message when a resumed run first
  meets its kind, so that a charter whose runs resume may have parts;
  documents chosen per part (../engine/settings.md, 9); an LLM choice per kind of
  wake, if `later` proves too coarse; and offering a run a message with its
  part rather than keeping it for the next run.
- **Dependencies on a failure:** whether a task should be able to depend
  on another's ending, whatever its result, for clean-ups; until wanted,
  a requester makes the clean-up task itself.
- **Priorities among tasks that are not goals,** when hosts or pools are
  scarce: they are ordered by when each became due.
- **Pools across projects:** whether a pool's slots are shared fairly
  between projects, or by goal priority alone.
- **A pending proposal whose holders go:** one that waits for a policy's
  roles when the policy stops naming any for its kind, or that stalls
  past its last holder: who may decide it then, or whether it is
  withdrawn, saying why.
- **A finish refused at the call** (`lifecycle.md`, 8): a run with its own question open
  that calls `finish` parks, the run hearing nothing live, and its next run's
  transcript shows that finish as accepted. Refusing it at the call needs a
  new crossing from jig through the worker to smith, since the run serves
  `finish` with no call to its host (../engine/executors.md, 3 and 4), and costs a turn;
  parking stays underneath it, as a refusal can't cover every path to a finish.
  Default: it parks, an application's charter saying beforehand to call `wait`.
- **A finish put off for live delegates, told** (`lifecycle.md`, 7; ../engine/executors.md, 4):
  whether the next run is told that its task's last run finished while
  its delegates were live, a bounded note the tasks store keeps for its
  brief. Default: it isn't, the plan `wake-faults`'s answer of
  2026-11-08; revisited if a model is seen finishing again and again while
  its delegates run.
