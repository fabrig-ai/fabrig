# Landing queues

Under a project's queue; a project that lands by turn has none
(`../landing.md`, sections 8 and 9).

One per landing branch that has changes ready. A repository belongs to
one project (another may read it as context), so its goals' priorities
are comparable.

- **In the queue:** a change is in the queue once it is ready (CI passed
  where its landing branch's rules ask it (`change/gates.md`, 2), and every blocking gate
  holds, at its head, and its task is not kept:
  no dependency it was given is pending, jig's `domain/tasks/lifecycle.md`, 8).
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
  merged; the rest wait. It keeps its turn through a clean update, CI on
  its new head and its merge, so no CI run is wasted by a change
  overtaking it. It **leaves the queue** on a conflict, on a check that
  fails (CI, a gate asked again), on a gate added, when held or
  closing, when its task is kept by a dependency added to it, or once
  the forge's own merge queue holds it (1), and the next becomes
  first; it comes back when it is ready again, at its place by its
  first time. A kept change keeps its head, its checks and its gates'
  verdicts: once its last dependency is met, it is ready again at once
  if they still hold. A resolution never holds the queue.
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

## 2. Open questions

- **A queue's throughput:** landings into one branch wait on CI one after
  another. If that binds, landings are batched: tested together, and
  bisected when they fail.

- **A repair of the branch's own CI at a tip since passed** (`queues.md`): the
  rule of `change/repairs.md` asks of a change's own head. That repair is a change of its own,
  made for a failed tip: CI passing on the tip ends the queue's pause, and that
  change goes on. Whether the rule reaches it. Default: as it stands.

- **Several changes in a forge's merge queue** (`queues.md`, 1): a change there
  leaves temper's turn, and the next enters behind it. The alternative
  is the first keeping its turn until it lands or is ejected, one at a
  time, as temper's own queue does. Default: several, the forge's queue
  grouping them.

- **The update before entering a forge's merge queue** (`queues.md`, 1): the first
  takes its base as any first does (`README.md`, section 6), though the queue merges
  the base itself. The alternative enters it as its head is, saving a CI
  run. Default: as any first.

- **Taking a change out of a forge's merge queue** when it is cancelled:
  the vocabulary has no call for it. Default: none; the cancel waits for
  the landing or the ejection (`queues.md`, 1). The alternative is closing the
  pull request, which the forge's queue drops.
