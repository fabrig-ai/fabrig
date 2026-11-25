# A parked task that is stuck

Provisional, 2026-11-25. A parked agent task that something has asked a
question, and that nothing left can wake but its own answer, or that
nothing can wake at all: what can wake a parked task, which of its wakes
are dead, the walk that judges it when it parks and again when what it
walked changes, and what follows. Written from
`temper/docs/design/draft/bounded-waits.md`, 5. A circle of
prerequisites is `holds.md`, 7's. A wait that has stopped with no
circle ends at its bound (`../engine/waits.md`, 5).

## 1. In one page

- **Wakes** (2): what jig keeps that can make a parked task due again:
  its timer, its watches, its delegates as its wake policy hears them,
  its own open questions, and its entries waiting at a party. Its
  prerequisites and its topics are not wakes.
- **Dead wakes** (3): a wake on a task that has ended, on the parked
  task itself, or on a task that only the parked task can move.
- **Stuck** (4): parked with a question asked of it open and every wake
  dead, or parked with no wake at all. A walk over the edges jig keeps
  judges it, within tasks per tree.
- **What follows** (5): a stuck task is due again at each park, a run
  try counted, and past its tries it is held, its words naming the
  question and the chain. A task with a live wake stays idle, spending
  nothing.
- **Judged again** (6): when a task its walk visited changes, when its
  own wakes change, and at each start.
- **Left to the bounds** (7): what the walk reads no edge for.

## 2. What can wake a parked task

A task is **parked** when it is an agent task, active and idle: its run
has parked, and no run is due or live (`lifecycle.md`, 2). Its
**wakes** are what jig keeps that can make it due again (the draft,
5.1):

- **its armed timer** (`messages.md`, 4);
- **each watch of a live task**, whatever states it asks to hear
  (`messages.md`, 4), one by reading among them (`messages.md`, 5);
- **each live delegate whose result its wake policy lets wake it now**
  (`messages.md`, 3): under `each`, every live delegate; under
  `last_or_failure`, the last live one alone, since another's result
  rides along; under `never`, none. A delegate's possible failure is no
  wake: the task plans for no failure;
- **each open question of its own** (`messages.md`, 2): one waiting at a
  task is that task's to answer, and its answer wakes the asker; one
  waiting at a person or a role is an entry at a party;
- **each entry of its own waiting at a party:** its questions there, its
  pending proposal (`proposals.md`), and each live person delegate.

A wake on a task (a watch, a delegate, a question waiting at a task) is
named by that task; a timer and an entry at a party are named by
themselves.

Not wakes:

- **Prerequisites:** its pending dependencies and the holds it queues
  for. They must be met whatever else wakes it, and they have their own
  circle check (`holds.md`, 7), the task's deadline and the queue's bound
  (`lifecycle-holds.md`, 1; `holds.md`, 2).
- **Topic subscriptions:** news is a hint (`messages.md`, 2), as Nothing
  left to await counts it (`lifecycle.md`, 2). A task that a connector's
  news would move waits at its `news` bound (`../engine/waits.md`, 5).

## 3. Dead wakes

A timer and an entry at a party are live: a timer fires, and a party may
act at any time, within the stall that passes its entry up
(`proposals.md`). A wake on a task S is **dead** when S cannot bring
it:

- S has ended: whatever it had to say has been said;
- S is the parked task itself;
- S can be moved only through the parked task: the walk (4) reaches no
  free task from it.

A task the walk reaches is, by what can move it:

- **free:** a task with a run live or coming (due, preparing, claimed,
  running or backing off); one held, which waits for a decision, not a
  task (`holds.md`, 7); one waiting, kept or closing, whose prerequisites
  and delegates have their own check and bounds; a person task or a job;
  a procedure with no open question of its own waiting at a task; a
  conversation, a standing task or one its requester ended, each of
  which waits on its person's words (`lifecycle.md`, 2; `standing.md`);
- **moved by its questions:** a procedure with an open question of its
  own waiting at a task, which the answer moves, as a procedure that
  relays a question waits on its answer;
- **parked:** any other idle agent task, moved by its wakes (2) as the
  parked task being judged is.

## 4. Stuck, and the walk

A parked task is **stuck** (the draft, 5.2) when:

- **a question is open and every wake is dead:** a question asked of it
  is open (`messages.md`, 2), it has no timer and no entry at a party,
  and the walk reaches no free task from its wakes on tasks. Only a task
  that the question rule wakes is judged so (`lifecycle.md`, 2): its
  policy lets a question wake it, its runs start from a brief, it is no
  conversation, it does not stand, its requester has not ended it, and
  it is not kept; or
- **it has no wake at all:** nothing asked of it, no delegate, no timer,
  no watch of a live task and no entry at a party, which is Nothing left
  to await (`lifecycle.md`, 2). It needs no walk.

**The walk** starts at the parked task T and goes breadth first. It
first takes the tasks T's wakes name: T's own open questions at tasks,
oldest first, then its watches in subscription order, then its delegates
by number. It then takes, in turn, the tasks that move each parked or
question-moved task it has reached, in the same order. Each task is
taken once. T and ended tasks end a path. The walk stops at the first
free task, and T is not stuck. If every task it reached has been judged
with none free, T is stuck. It reads each task's own record alone (its
phase, executor, wake policy, subscriptions, delegates, and its own
questions and where they wait), so it scans no other task.

**Its bound** is the one the views' tree walks take, tasks per tree
(`README.md`, section 6; `../engine/views.md`, 3): it reaches at most
`tasks.tree_tasks` tasks. A walk that would reach one more stops and
leaves T alone, since a wake past the bound may be live. It is judged
again as section 6 says.

## 5. What follows

- **Stuck with a question open:** the task is due again at once,
  `question` among its reasons (`messages.md`, 6). A run try is counted,
  and its tries are kept across these parks as the question rule keeps
  them (`lifecycle.md`, 2). Past its run tries it is held, as a task past
  its tries is (`lifecycle-holds.md`, 1), for `Failures` of the run's
  class. Its source is `Preparation`, with these words:
  - `Task A's question N is still open: LINE` where the walk reached no
    task but A;
  - `Task A's question N is still open, through tasks B (question M), C:
    LINE` otherwise.

  A, N and LINE are the oldest open question's asker, number and first
  line. B, C and the rest are each task the walk reached other than A,
  in the order it reached them, each with the number of its own open
  question where a question moves it. The words are written in that
  order and cut at a character's edge to `REASON_BYTES` (128 bytes), so
  a long chain cuts the line first. A task held so is not failed: its
  delegates go on, a release counts its tries again from it, and the
  hold escalates as any hold does.
- **Stuck with no wake at all:** as Nothing left to await says
  (`lifecycle.md`, 2), the task is due again with its `NothingAwaited`
  notice and a run try counted. Past its run tries it is held for
  `Failures` of the run's class, not failed. Its words are `Its runs
  parked with nothing left to await, past its N run tries.`, where N is
  the run tries its class allows.
- **Not stuck:** the task stays idle. No try is spent and no notice
  numbered until a wake comes or what its walk visited changes (6).
- **Every open question counts at every park.** No watermark passes an
  older question over because a newer one woke the task, and no question
  earns a wake by being newer: the record keeps no `question_woken`
  (`../../protocol/store/records.md`, 1). A task that answers stays
  under the ordinary rules, and its tries start again at its next park
  that is not stuck, as any park's do.

A stuck run may still answer: its brief shows each open question
(`../engine/briefs.md`, 1.2), so each try is a run that can answer.

## 6. Judged again

- **When what it walked changes.** The tasks child keeps, in memory
  alone, each parked task with an open question that the check left
  alone, with the tasks its walk visited, at most tasks per tree. It is
  judged again, in the decision that makes the change, when:
  - a visited task changes what the walk reads of it: its phase, its
    own open questions or where they wait, its subscriptions or timer,
    its delegates, its wake policy, its pending proposal or its person
    delegates; or
  - its own wakes change without waking it: a watch or a timer added or
    gone, a delegate made or ended, its policy amended, or an entry of
    its own decided.

  Found stuck, it takes section 5's outcome then, as at a park: due
  again with a try counted, or held past its tries. Found not stuck, it
  stays, with its new walk's tasks. A task that leaves idle, or whose
  last open question closes, leaves the memory.
- **In turn, after the change's own outcome.** A decision that parks a
  task takes that task's outcome first. It then judges again each task
  its change concerns, the oldest left alone first, each seeing the
  outcomes taken before it. So the tasks of one circle are never all
  judged against one another's idle phase, and the circle is held at one
  task (7).
- **A walk cut at its bound** is judged again at every such change to a
  task it reached.
- **At a start** the memory is rebuilt as the restore ends: every parked
  task with an open question is judged, and one found stuck takes
  section 5's outcome. So a task the previous build left idle, woken
  once for its newest question and then left, is caught at the first
  start of this one (`temper/docs/design/draft/bounded-waits.md`, 3.4).
- **One decision:** the outcome is taken in the decision that parks the
  task or makes the change, so a crash before it commits finds it again
  at the next start's judgment.

## 7. Left to the bounds

The walk reads only the edges jig keeps. A wait it doesn't read ends at
its bound (`../engine/waits.md`, 5; `lifecycle-holds.md`, 1):

- **Prerequisites,** a kept or waiting task's dependencies, and a
  closing task's delegates: `holds.md`, 7 checks their circles, and the
  deadline bounds them.
- **A held answer:** an application's procedure that holds an answer
  until a dependency of its own ends (the draft, 5.1) keeps that in its
  own state, not in jig's edges. To the walk it is a procedure with no
  open question of its own, and so free. A circle through one ends at
  its asker's `question` bound, which passes the question up, or at its
  watcher's `news` bound.
- **A task outside the question rule:** one whose runs resume, whose
  policy doesn't wake on questions, or that is kept. A conversation and
  a standing task wait on their person's words.
- **A wake that keeps coming:** a timer is live, so a task that keeps
  waking on its timer and parking again reaches its `news` bound
  instead (the draft, 5.2).
- **A held task in a chain** is free to the walk: once one task of a
  circle is held, the others wait behind its release, which a holder
  decides within its stall. So a circle is held once, at the task whose
  park closed it, and its hold names the rest.

## 8. Open questions

- A held answer as an edge the walk reads: a procedure's reported wait
  that names the task it waits behind (`../engine/waits.md`, 6) could
  move the walk on to that task.
- A held task's escalation waiting at a task, and a kept task's
  dependencies, as edges: either would catch more circles at once,
  where today their bounds end them.
