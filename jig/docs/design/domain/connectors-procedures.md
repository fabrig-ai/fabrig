# Connectors: procedures

## 1. Procedures

### 1.1 What a procedure is

A step machine a connector owns, one per task whose executor it is:
remediate, provision, a change, a standing watch. Its parameters come
with the batch that makes its task, in the connector's terms, and the
connector keeps them under the task's number. Its state is a plain
value, kept in the store with the connector's records and committed with
each step, including the effects it has asked for that are not yet
settled and the wall times of what it waits for. It knows its task's
number, its delegates by number, which of its task's dependencies are
pending (tasks/lifecycle-endings.md, 1), and the facts it reads. A projection
(`connectors.md`, section 8) is not a procedure: no task asked for it.

**A procedure that starts kept.** A connector may declare, per
procedure, that a task of it made with a pending dependency enters
active and kept (tasks/lifecycle-endings.md, 1) rather than waiting: it steps and
delegates while its dependency goes on, under 5.7's refusals, ending no
way but failed or cancelled while kept and asking no effect that waits
for dependencies (`connectors-effects.md`, 1.1). It is for work whose producing may run beside
its dependency's, and whose landing or result is what the dependency
orders: temper's change procedure, whose dependent change across
repositories is coded beside its dependency and merged after it. The
declaration is the connector's, given with its configuration to the
core, which keeps it for each procedure it names by connector and code.

**A dependency started from.** A step of such a procedure may mark
pending dependencies of its task as started from, when what its task
makes builds on their unfinished work (temper's early start, whose
producer starts on its dependencies' pushed heads). The mark is kept
with the task's dependencies and never lifted: an amendment that leaves
a marked dependency out is refused, naming it (tasks/changes.md),
since the task's work cannot drop what it was built on; its requester
cancels the task and makes it again instead. A mark naming a task that
is not among its task's dependencies refuses the step whole (1.3).

### 1.2 When it steps

- when its task becomes active;
- when its task is kept or goes on again: an amendment adding a
  dependency, and the notice that its last pending one is done or
  removed (tasks/lifecycle-endings.md, 1, and tasks/messages.md, 1);
- when a message its owner lets through reaches its task: a delegate's
  result, an amendment, a cancel, an answer, news it subscribed to; a
  cancel's grace, asking it to finish, comes as an amendment does
  (tasks/changes.md);
- when its last step was refused, with why (1.3);
- when room it was refused for frees: a step whose batch was refused
  for a count that frees on its own (the live tasks in the deployment or
  in its project, tasks/README.md, section 4; never its tree's allotment, which
  counts every task the tree made and never frees) is told so, and
  its task waits for that room as a condition (1.4): a task ending that
  frees a place wakes the procedures waiting on it, one per place freed,
  in the order they were refused, each stepping again on the facts as
  they are then; the waiters are its live procedure tasks, so their
  number is bounded by theirs, and a restart rebuilds them from the
  refusals their states keep;
- when an effect it asked for settles, made or failed (`connectors-effects.md`, 1.2);
- when a fact it reads changes, which its connector knows from its
  working set;
- when a deadline its state names passes: a stall, a backoff, the end of
  what its task was asked to keep. The connector's top arms one deadline
  per procedure task, from the wall time its state keeps; the procedure
  arms none.

### 1.3 What a step decides

One decision, committed with its new state:

- optional specification words for its own task: `ProcedureStep.words`
  reaches `ProcedureDecision.words`, replaces only its specification
  words, and saves in this decision (tasks/lifecycle.md, 3). Its connector and
  code must match the task's due step; words and byte-valued parameters
  share `Limits::spec_bytes`. Parameters, inputs and `asks` do not change;
  this adds no revision, amendment, message, wake or amendment history;
- effects, described and kept for the outbox (`connectors-effects.md`, 1.1);
- batches of delegates: agents for what needs judgement (a triage, a
  diagnosis, a repair), people for what needs a decision, each task
  with what a party's `Create` names (tasks/README.md, section 4);
- amendments of its delegates and the tasks below them, each naming the
  revision of the spec it was told, but for a change of priority alone
  (tasks/changes.md);
- cancels of its delegates and their trees, with a reason, and a grace
  when the step gives one, in which the task is asked to finish first
  (tasks/changes.md);
- proposals, for what its task's authority does not cover, while the
  rest of the step commits: a batch, an amendment, an effect, or a
  widening of its task's authority (authority.md, section 9);
- messages: words to its delegates and to the tasks its task references
  (tasks/messages.md, 5), and an answer to a delegate's question; and up, a
  question, an escalation, news its requester should have;
- marks on pending dependencies of its task, started from (1.1);
- a hold, when it needs a decision it cannot make. Its connector, finding
  the hold's cause gone while the task is held, may ask the root to release
  the task as the deployment's (tasks/changes.md): the procedure then steps
  as after any release, and no person is asked;
- its result, which ends its task: in jig's terms (a report, a verdict),
  or a connector's result it keeps, with words summarising it (tasks/README.md,
  section 3).

Every effect and batch is checked against its task's authority, as any
decision is (authority-checks.md, section 1); a procedure's authority is its
task's, given by its creator.

**Kept, it steps as ever.** While its task is kept (tasks/lifecycle-endings.md, 1), a
step may do all of the above but two: ask an effect of a kind that
waits for dependencies (`connectors-effects.md`, 1.1), and end its task done. A step that does
either is refused whole, its connector told which dependencies are
pending, and the procedure steps again once they are done. A step may
still hold its task, or end it failed.

**The same operations, by the same rules.** A step's batches,
amendments, cancels, proposals and messages are the operations a
party's requests and a run's tools make (people/acts.md, 1; engine/executors-tools.md,
1), and reach the same rules: who may amend or cancel a task, what an
amendment may change, a grace's bound (tasks/changes.md); who may
message whom (tasks/messages.md, 5). What a procedure never does is decide: it
releases no hold and decides no proposal or escalation, which pass over
it to the next holder (tasks/proposals.md); stopping a run and moving a
task are parties' alone.

**A step names the revisions it was told.** An amendment names its
task's spec's revision as the core last told the procedure's connector:
with what its step made or amended (below), or with a step refused as
moved. A step whose amendment names a revision that has moved is
refused whole, as a step that fails any check other than authority is:
nothing of it commits, its connector is told why, with what the task is
now, and the procedure steps again on that (1.2). So no procedure writes
over a change it was not told of.

**What its step made comes back** within the same decision. The core
numbers a step's batch as it admits it, so the step cannot know its
delegates' numbers; the core tells the procedure's connector, through
the root, the number of the first task the step made, or that it made
none, with the revisions of the specs it made or amended; or that the
step was refused, and why. A connector that must know what that task is
for (an incident's lead) keeps, in its procedure's state, what the step
made it for, and
records the number when it hears it, durably, in that decision, so a
restart keeps it. Nothing is inferred from the task's words or the
order tasks were made in.

### 1.4 Level-triggered

A step decides from the facts as they are and its own state, never from
the event that woke it, so stepping twice on the same facts decides the
same, and a step lost to a restart is made again from what is read. It
never asks for an effect while one for the same purpose is outstanding
in its state, so deciding the same twice asks for nothing twice. What it
waits for (a service healthy, an environment ready, a verdict met) is a
condition on the facts, armed with a deadline past which it stalls.

### 1.5 Bounded

Each procedure declares its limits: remediations, retries, the stall of
each wait. Past one, it holds its task, saying why, and its requester
hears (tasks/lifecycle-holds.md, 1). Its connector reports each wait its
task is in to the core, as the record every task's wait is, with that
stall as its bound (`engine/waits.md`, 6).

### 1.6 Standing procedures

A procedure may stand: subscribed to topics, it makes tasks as news
arrives, and ends only when cancelled. `ops`'s watch is one: woken by a
batch of alerts, it makes a triage task for them, and, when a triage's
verdict says so, an incident and the task that leads it
(`../examples.md`, 7.1). A standing procedure's tasks are its
delegates, within its authority, and its requester is usually the
deployment or a person on call.

### 1.7 Drained

A procedure in the scope of a draining hold
(tasks/lifecycle-holds.md, 2) is told so in its next step, and steps on as ever until
its connector answers that it is idle: at the first point where its
step would start something new, rather than finish what it began. The
core then holds it, by its step's end, `Idle`, which outside a drain
is a wait. What counts as new is the connector's to say, for each
procedure, as part of the procedure's own design. A procedure
already waiting for nothing of its own is idle at once. A drain lifted
before it is idle leaves it as it was.
