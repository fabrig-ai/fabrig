# Landing queues

Under a project's queue; a project that lands by turn has none
(`../landing.md`, sections 8 and 9).

One per landing branch that has changes ready. A repository belongs to
one project (another may read it as context), so its goals' priorities
are comparable.

- **In the queue:** a change is in the queue once it is ready (CI passed
  where its landing branch's rules ask it (`change/gates.md`, 2), and every blocking gate
  holds, at its head, and its task is not kept:
  no dependency it was given is pending, jig's `domain/tasks/lifecycle.md`, 8),
  and no person's approval it waits on is short at its head (below).
  It keeps the time it
  first became ready while it stays live, so leaving and coming back
  does not send it to the back.
- **Ordered by priority,** its goal's (jig's `domain/model.md`, 6.2), a
  change outside any goal taking its project's default, **then by that
  time.** Priorities move with the goals'. **Aging bounds starvation:**
  a change ready for longer than the deployment's queue window
  (`forge.queue_window_ms`, `../../protocol/configuration/limits.md`)
  goes before every change that became ready after it, whatever their
  priorities.
- **The first owns the queue.** Only the first is brought up to date and
  merged; the rest wait. A change owns the queue's turn from the step it
  becomes first until it leaves the queue or lands, and while it owns it
  no other change into the branch is first, whatever its priority or its
  age: the order above, aging included, chooses only who takes the turn
  next. It keeps its turn through a clean update, CI on its new head (a
  re-run of the runner's failures there included, `change/README.md`, 2,
  step 3), its gates carried over and its merge, so no CI run is wasted
  by a change overtaking it. It **leaves the queue** on a conflict, on a
  check that fails (CI, a gate asked again), on a gate added, when held
  or closing, when its task is kept by a dependency added to it, when a
  person's approval it waits on is short at its head (below), or once
  the forge's own merge queue holds it (1), and the next becomes first:
  a change that gives up its turn, or lands, wakes the ready changes into
  its branch in the same step, so the next takes the turn with no wait of
  its own. It comes back when it is ready again, at its place by its
  first time. A kept change keeps its head, its checks and its gates'
  verdicts: once its last dependency is met, it is ready again at once
  if they still hold. A resolution never holds the queue. A ready repair
  of the branch's own CI (below) goes first even so: the owner keeps its
  turn and waits for it, as every change does while the branch's CI
  fails.
- **Out of the queue while a sign-off waits**
  (`../../draft/what-reaches-you.md`, section 7): a sign-off can wait a
  night, and the branch's other changes shouldn't. A ready change whose
  review facts at its head (`change/gates.md`, 2) show an approval of an
  enforcing rule covering it short, and none of them refused, is out of
  the queue, as a kept change is: it stays `Queued`, keeping its head,
  its checks, its gates' verdicts and its first time, and gives up the
  turn if it held it; the next becomes first and the rest of the branch
  lands meanwhile, while Needs you shows it to whoever's review it waits
  on (`change/questions.md`, 2).
  - *Its facts* are a step's: each step of a ready change that an
    enforcing rule covering it asks an approval of hands the root its
    reviews, which the root judges and keeps (`change/gates.md`, 2), and
    a merge that waits on an approval keeps them too. A change out of
    the queue is stepped on a review's delivery and once a queue window
    (`forge.queue_window_ms`), as a waiting merge is asked again.
  - *Back in the queue* once its approvals count at its head: ready again
    at its place by its first time, after the change then first, which
    keeps its turn through its update, CI and merge. A person's review
    asking for changes at its head makes the repair a merge's refusal
    makes (`change/gates.md`, 1), from the step's facts, without waiting
    for a turn.
  - *A change with no facts at its head,* one just ready or every change
    after a restart, counts as in the queue until its step's facts are
    kept; a first whose merge then waits on an approval leaves as above,
    so the queue's turn is never held for one.
  - *An `exact` approval* asks again at each new head, a clean update's
    among them, so a change signed off and then updated as first leaves
    again for a sign-off at its new head: a rule by path on a busy branch
    wants `clean` (`../../protocol/configuration/engine.md`).
- **Shown whole,** to the client's subject `changes`
  (`../../client/forge/records.md`): the change that owns the queue first, at
  whichever step of its turn, then the ready changes in this order.
- **Derived, not kept.** A queue is the ready changes into a branch,
  their priorities and their times; nothing of it is stored beyond the
  changes' own states, so a restart rebuilds it from them.
- **The branch's own CI,** for a branch no tier follows: a branch
  tiers follow is judged by their verdicts instead, and its queue never
  pauses for them (`../after-landing.md`, sections 2 and 7). A landing temper did
  not make may break the branch. Once CI fails on the branch's tip, the queue pauses and one
  repair is made: a change task into that branch, first in its queue,
  its words, and its producer's, opening with what it repairs, as a
  repair's do (`change/repairs.md`): the branch, its failed tip at the forge's length
  and each failed check's context and name, and its brief carrying `change/repairs.md`'s
  why, CI's failing output on that tip,
  requested by the deployment on the project's behalf with the
  authority its policy gives such repairs, funded from the project's
  spend (jig's `domain/authority.md`, section 6), and news for every goal
  with changes waiting there. Every change in the queue waits for it
  rather than repairing a failure that is not its own. The pause ends
  when CI passes on the tip; repairs are made one at a time, up to a
  bound, past which the queue is held for a person, who may also release
  it as it is (a flaky check).
- **Where temper may not merge,** the queue still orders what is ready,
  for people to see; each change's landing waits for someone else's
  merge (`README.md`, section 7).
- **Where the forge keeps its own merge queue** on the branch, temper's
  queue decides only what enters it and when (1).

## 1. A forge's own merge queue

A branch whose rules require the forge's merge queue (GitHub's; the
adoption says so, `adoption.md`, 4) lands through that queue. Forgejo keeps none.

- **Entering is the merge.** The first's merge (kind 4) is made as any
  merge is: at exactly its head, its requirements judged at that head
  (`change/gates.md`, 2). The forge answers `Answer::Queued`, final for the call and not
  for the entry: the pull request is in the forge's queue at that head,
  neither landed nor refused. temper names no other call for it: the
  forge's rules, not temper, send the merge into its queue (skein's
  `forge/README.md`, 3.3).
- **temper's queue decides only what enters and when.** A change in
  the forge's queue leaves temper's turn, and the next ready change
  becomes first, and enters behind it when its own turn comes; the
  forge's queue groups and orders what it holds, and runs the branch's
  CI on each group. While the branch's own CI fails, nothing enters
  (`queues.md`).
- **Landed or ejected comes later,** heard as any hint is, from a
  delivery or the delivery log (`../../protocol/forge/README.md`, section 4 and
  4.1), and found by the client's read of the pull request: the merge's
  entry stays open in the client until that read finds the pull request
  merged or out of the queue (skein's `forge/README.md`, 3.3;
  `skein/docs/design/forge/client.md`, 7.7).
- **Landed:** the change is landed (`change/README.md`, 2), and its landing's commit is
  the queue's group's (`Pull.merged`), not the head temper saw: the
  forge merged that head onto the queue's base with the changes ahead
  of it.
- **Ejected,** for any reason but its landing (its group's CI failed, a
  conflict with what entered before it, a person took it out): a
  repair, as a failed CI is (`change/repairs.md`), its brief carrying the forge's reason
  alone, the pull request's `dequeued` (`README.md`, section 11), which a green head
  does not end. The change takes
  its base before the repair starts (`change/README.md`, 2), so a conflict there is
  resolved. A head another hand moved is drift, as ever (`change/README.md`, 3).
- **Its place there,** `Pull.queued`, is read with the pull request and
  shown in the change's panel ("in GitHub's merge queue, n";
  `../../client/forge/README.md`, a change's panel). The wait there is bounded by
  the stall (`change/repairs.md`).
- **Cancelled while there,** the change waits for its merge's entry to
  settle, landed or ejected, and what landed counts (`change/README.md`, 4): temper never
  takes a pull request out of the forge's queue itself (`README.md`, section 11).
