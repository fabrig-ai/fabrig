# Holds

**A configured-bound hold** stops the task under `lifecycle-holds.md`, 1; the resource holds
below stay with it as for any held task. An authorized
`Release` lifts `Hold::Exceeded`, returning it to Due with tries reset.
If the same bound is reported again, its first report holds the task
again. Words alone do not release it (../people/acts.md, 1).

## 1. What a task holds

- **A write hold** is a resource a task writes alone: a branch a change
  is pushed to, a service being remediated, an environment it provisioned.
  Its connector says which kinds of resource take holds (../connectors.md,
  3.3).
- **A pool slot** is one of a pool's counted slots: an environment from a
  pool of five, a share of a quota. Its connector names the pool, a
  resource of its own, and says how many slots it has, which may change
  as its facts do (6).
- **Delegates write under their holder's hold.** A procedure's producing
  and repair runs write the resource their procedure's task holds.
- **Shared resources are not held:** writing into them is ordered by
  their connector's mechanics (a landing queue), not by a hold.
- **A held resource a queue orders:** a resource may be both held and
  written through its connector's queue, as a goal's own branch is, held
  by its goal and landed into by its goal's changes one at a time. The
  hold says who may write it, the holder's delegates alone; the queue
  orders their writes, each taking the writer slot as it writes (3). A
  write into it by a task outside the holder's subtree is refused at its
  batch, naming the holder (../connectors.md, 3.3).

## 2. Taking holds

- **All at once.** A task takes every hold it needs in the commit that
  makes it active, or takes none and waits, so a task in a queue holds
  nothing. That alone does not keep waits from circling: a task holding
  a resource may wait for its delegates or, kept, for its dependencies.
  The check of 7 does.
- **Refuse or wait.** For each kind of resource, its connector's
  declaration says whether a hold that is taken refuses or waits
  (../connectors.md, 2), the rule the core installs in this child as it
  is built; a resource's report below says it for that resource and wins
  over its kind's. A batch whose task needs
  a hold that refuses is refused, naming the holder. A task whose hold
  waits is made, and waits in the resource's queue.
- **What a task holds is its connector's to name,** asked once the task
  has its number: from its executor and specification, and from the
  connectors' symbols its batch member names (../authority.md, section 4;
  `../../protocol/configuration-templates.md`, 3), each resolved to the name the
  task holds, and whether the task is tracked. A symbol the connector
  does not offer that task refuses the batch, naming the symbol.
- **Reported before it is taken.** For each resource a batch member
  holds, its connector reports the resource's role and hold kind
  (../connectors.md, 3.3), before or with its holdings answer, as many
  reports in a row as the batch has names. The tasks child keeps the
  latest report of each name in a table of `tasks.resource_reports`
  entries, in memory alone: a restart's connectors report again
  (../connectors.md, 14). A batch is made once every resource its
  members hold has a report; until then it waits whole, making no task
  and reserving no funding, its reply kept, and each report that comes
  tries it again against the state as it is then.
- **Kept while named or pinned.** An entry is kept while a live task's
  holdings name it or a waiting batch pins it. The core pins a batch's
  names as it takes the batch to be made, before it makes it or parks
  it, and unpins them once the batch is made, refused or dropped. A
  report of a name the full table lacks takes the place of the entry
  reported least recently among those not kept, its report's order
  counted afresh at each report; with none, it is dropped. Least
  recently, not first by name, so that a batch's members, reported one
  after another before its holdings answer, never push one another out
  while the table has room for them all.
- **Counted, or refused.** A batch is pinned only where the table holds
  it: the names kept, those other waiting batches pin, reported or not,
  and the batch's own, each counted once, together at most
  `tasks.resource_reports`. A batch past that is refused whole, making
  nothing: `Busy`, its capacity's cause `ResourceReports` with the
  table's bound (`README.md`, section 4), its words saying that the
  table cannot hold its resources (temper's are in
  `temper/docs/design/domain/agent/tools.md`).
- **Waited for within `tasks.hold_wait`.** A batch parked for a report
  waits at most `tasks.hold_wait` from when it parked, the figure that
  bounds a task's wait in a resource's queue (below). Then
  it is refused whole, `Unreported`, `blocked_by` naming each member, in
  the batch's order, some resource of whose has no report, and `task`
  the first of them; its pins go with it. Its connector is not asked
  again: a pinned name's report is never pushed out, so one still
  missing then is one its connector never sent, and asking again would
  wait on the same connector. Nothing is kept of a refused batch, and
  its caller may call again. So a call that makes tasks always has an
  answer: made, refused as it comes, or refused at its deadline.
- **A taking has its effects,** as a release does (5): as the commit
  that makes a task active takes its write holds, each hold's connector
  hears of it with an outbox entry of its own, and does what its kind's
  taking does (../connectors-effects.md, 1.7). One that fails for good holds the
  task, saying why (`lifecycle-holds.md`, 1).
- **Queues are ordered** by the priority of the goal each task serves
  (../model.md, 6.2), then by when it began to wait, and bounded per
  resource. A task waiting past its bound is held (`lifecycle-holds.md`, 1), and its
  requester hears why.
- **A queue is joined once the dependencies are done.** A waiting task
  given another (`changes.md`) leaves its queues, keeping when it began to
  wait, and comes back at its place once its dependencies are done
  again, its bound counted afresh from then.
- **Freed holds go to the queue** in the commit that frees them: the
  first waiter whose every hold is now free takes them all and becomes
  active. A pool slot is free only while the pool has fewer holders than
  the slots it is known to have.

## 3. Writer slots

Each write hold has one writer slot: the one run that may write it at a
time.

- **A run takes the slot of each held resource its workspace writes**
  (../engine/executors.md, 1) when it is claimed, whether its task holds the
  resource or writes under a task above it that does, and none of a
  resource it only reads: a goal's run that inspects its goal's branch
  takes none of it, while the merges into it take it in turn. It frees
  each when its answer is committed, or, when its attempt is settled as lost past
  the engine's grace (../engine/fleet.md), once the connector has read
  the resource afresh, so that what a lost attempt wrote is known before
  anyone writes again.
- **The run writer's task and its hold owner stay live** while the slot is
  taken, including when the hold was handed down. Neither advances to
  settling its connector effects and releasing resources until the run's
  answer frees the slot, or, after loss, the connector confirms its fresh
  read. The confirmation erases the writer and resumes closing in the
  same decision, so a restart sees no writer without its live task and hold
  owner.
- **A claim that finds the slot taken waits,** which is no failure.
- **A connector's own effects** on a held resource take the slot too,
  while they are in flight, and wait for it like a claim.
- **Each use checks its writer** (skein/docs/foundation/programming-model.md,
  10.4). A slot is taken only where its claim's or its effect's admission,
  in the same step, found it free and the room for it; a slot taken and
  room full are answered as the waits and refusals above, as values.
  Taking it asserts that no writer holds the resource, naming the
  resource and its new writer; an effect whose own entry already holds
  it is answered as done before anything is taken. Each release names
  the writer it expects (the run's task and attempt at its committed
  answer, the lost attempt's at the connector's fresh read, the effect's
  entry as it settles) and asserts that the slot is that writer's,
  naming the resource and both writers, and frees it once. A loss marks
  only the slots its attempt holds, each asserted as its. A trigger
  that may find no slot of its writer (an effect that took none, a fresh
  read of a resource no lost writer holds, an answer of a run that
  wrote nothing) finds that from its writer's own slots, as a value,
  before anything is freed.

## 4. Handing down

A creator may hand a resource it holds to a task it makes: the hold
moves in the batch's commit, and the writer slot stays with a run that
has it until that run's answer, so the new holder's first writer waits
for it. A chat whose run wrote to a resource hands it to the procedure
task that finishes the job.

## 5. Release

Holds are released as the task closes (`lifecycle.md`, 1), after every other effect of
the task has settled. What becomes of the resource is its connector's to
say, per kind and per way the task ended, as effects of its own, as its
taking's are (2): an
environment provisioned for a task that ended is torn down; a branch a
change landed from is deleted; one a failed change left is kept for a
person to read until its tree's root closes.

## 6. Pools that shrink

A pool's slots are its system's to say, and may fall below the number of
tasks holding it: a quota cut, a machine gone.

- **Holders keep their slots.** Nothing the core admitted is taken back
  by a fact: the tasks holding the pool keep their holds.
- **The pool is overcommitted,** and admits no holder, from its queue or
  from a batch, until its holders are fewer than its slots.
- **A holder whose allocation is gone** is told by the connector, as
  drift (../connectors.md, section 11): its task is held, saying so, and its
  release frees its slot.
- **The promise is about admission:** no holder is admitted beyond the
  slots the pool was known to have when it was admitted.
- **A shrink is judged for circles.** Fewer slots make each waiter need
  more holders to end, so each is judged as if it joined again (7).

## 7. Circles of waits

A live task may wait for others to end. No set of tasks waits in a
circle, each for another of the set, so that none of them could end.

- **The waits:**
  - *a dependency:* a task, waiting or kept, waits for each pending
    dependency;
  - *a delegate:* a task waits for each live delegate, since it ends
    after them (`lifecycle.md`, 7);
  - *a queue:* a task waiting for holds waits for the holders of each
    resource it needs that is not free. A write hold needs its holder to
    end. A pool needs as many of its holders to end as bring them below
    its slots, so a wait for it is stuck once as many of its holders are
    stuck as it has slots, one at least.

  A held task waits for a decision, not a task, and a closing one for
  its delegates alone, so neither waits for a dependency or a queue: a
  release judges again the waits it gives back.
- **Checked where a wait is added,** with the live ones:
  - *a batch:* its tasks' dependencies, the queues they join, the holds
    handed down to them and its creator's wait for them. One that would
    close a circle is refused whole, naming the circle's tasks (`README.md`, section 4
    ).
  - *an amendment adding a dependency* is refused so (`changes.md`).
  - *a queue joined on its own,* by a task whose dependencies are done,
    and *the waits a release gives back:* a task that would close a
    circle is held instead, naming the circle (`lifecycle-holds.md`, 1). One held from a
    queue leaves it, keeping when it began to wait. Its hold lifts on
    its own, back where it was, once a task the circle names ends, is
    held or loses a wait (a pending dependency done or removed, a queue
    left), or a pool on it changes its slots; one whose names were cut
    at the limit, at every such change. A release judges it afresh.
  - *a pool that shrinks* (6): each waiter, as a join.
  - *a failed task's hold retained by its root* (5): each waiter, as a join.

  Taking holds adds no wait: a task becomes active with its dependencies
  done and no delegates yet.
- **How.** A new wait from a task can close a circle only among what
  waits for that task, directly or through others: any other circle was
  one already. The check takes two steps over them.
  - *It marks them,* walking back from the task along the live waits:
    what depends on a marked task, the requester of a marked delegate,
    and a resource's waiters once one of its holders is marked. It
    counts each resource's marked holders.
  - *It judges them as a restart does,* with the task's new wait: in
    turn it frees a marked task none of whose waits is stuck, counting
    down what waits for it. A wait for a task is stuck while that task
    is marked and not freed; one for a resource, while as many of its
    holders are left as it has slots, one at least. The new wait closes
    a circle when the task is left.

  Marking only what would be stuck were the task stuck is not enough: a
  pool's holders can each be stuck only through the circle they close
  together. The circle named is its knot: from the task, each named
  task's first wait still stuck (its dependencies, then its delegates,
  then its resources), and for a resource as many of its holders left
  as it has slots, so that a change to any wait the circle needs
  changes a task it names. It costs two passes over what waits for the
  task, for most its ancestors and their dependents, within the limits
  (`README.md`, section 6; ../engine/README.md, section 7). The tasks child
  keeps, per resource, its holders and waiters, and, per task, its
  dependents, so that the walk reads them without a scan.
- **Restored, judged whole.** A restart judges every wait at once, ending
  in turn what can end and counting down what waits for it. A queue's
  waiter left on a circle an older rule allowed is held, as at a join.
- **The bound stays the backstop.** Writer slots, effects in flight and
  a connector's own queues, such as a landing queue, are not among these
  waits. A task waiting for holds past its bound is held all the same
  (2), and a connector keeps its own queue from circling: a kept task
  leaves it (../connectors.md, 3.3).
- **Wakes are not these waits.** A circle that runs through a parked
  task's wakes, its watches, its delegates' results and the questions
  asked of it, is found when the task parks, not when an edge is made
  (`stuck.md`).

## 8. Later

- **A client's words for an unreported batch.** A person's batch refused
  `Unreported` reaches a client in the form `ResourceUnavailable` takes
  there, a required connector resource unavailable, naming no member,
  until the client's wire carries `blocked_by` for it; an agent's call
  names them (2).
