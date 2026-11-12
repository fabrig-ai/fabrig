# A task's lifecycle

## 1. Phases

```
waiting ──► active ──► closing ──► done, failed or cancelled
   │          │           │
   └──────────┴───────────┴──► held ── released ──► back where it was
```

- **Waiting:** some dependency is not done, or some hold it needs is
  not yet free (`holds.md`). A procedure its connector declares to start
  kept is not waiting on its dependencies: it enters active and kept
  (../connectors.md, 6.1).
- **Active:** its executor is engaged (2 to 5). It enters active in
  the commit that makes it ready, which takes its holds. An active task
  with a pending dependency, which an amendment gives it, or its
  creation where its procedure starts kept, is **kept** (8).
- **Closing:** its executor has finished, failed, or the task was
  cancelled, and it is settling before it ends. In order:
  1. its run, if one is live, has ended: cancelled, and its answer
     heard;
  2. its delegates have closed and ended, deepest first, the follow-ups
     its result made among them, and the proposal its result made, if
     any, has been decided (7);
  3. every effect it asked for that went out has settled, made or
     failed, and those not sent are withdrawn if it was cancelled
     (../connectors.md, 4.2);
  4. its connectors release the resources it held, as effects of their
     own, after all its others (`holds.md`, 5).

  Then it ends, and its result, or why it failed or was cancelled, goes
  to its requester. So a task that depends on another never starts
  before what that task did is visible and settled.

  What the core knows of a closing task is what it waits for: which of
  its delegates are still closing, and which connectors it waits on to
  settle its effects, by number. It never knows their outbox entries,
  which are each connector's, kept by the connector and settled by it
  (../connectors.md, 4.2; ../model.md, section 5).
- **Done, failed, cancelled:** ended. Its result is a message to its
  requester, and kept. An ended task leaves the engine's working set once
  that message is committed, leaving a stub while live tasks name it;
  the store keeps it all, until its retention class erases it, or its
  project's deletion does (`README.md`, section 3).
- **Held:** stopped for a decision, from waiting, active or closing. It
  says why (6). A release returns it where it was, the reason it was
  held judged afresh.

## 2. Agent tasks

An active agent task is one of:

- **idle:** nothing to do until a wake (`messages.md`, 3);
- **due:** a run is wanted: the first activation, or a wake;
- **preparing:** its brief is gathering and its saved transcript window,
  when present, is loading for a resume or a fresh brief's tail, including
  a transient retry's tail (../engine/executors.md, 1). An amendment or
  a cancel that comes meanwhile drops what was gathered and decides
  again; a brief that cannot be gathered sends it back to due after a
  backoff, pausing it or counting a failure by why, as no attempt was
  made (../engine/briefs.md, 1.1);
- **claimed:** an attempt is committed, naming the run and its attempt
  number, the writer slots of the holds its workspace writes (`holds.md`, 3), and
  the messages its start carries (`messages.md`, 2), before the run is assigned;
- **running:** a run is live on a host. Its start carried the messages
  waiting; it hears those its policy lets through as it reads (`messages.md`, 2);
- **backing off:** an attempt failed and the next is due after a backoff
  (6).

A run ends by **finishing**, with a result the core checks against the
contract (7), or by **parking**, its transcript kept, the task idle
until its next wake. A task has one run at most at a time. Whether a
waiting run holds its host's slot or parks is ../engine/executors.md, 4. A claim
that no host reports after a restart, its attempt having committed no
turn and no call, counts as not started: no try is spent on it. Nor is
one on a host in the engine's own process when that engine stops
(../engine/restart.md, 1): its attempt ends interrupted, as a recheck ends, with
no try spent and no backoff, keeping the turns and calls it committed,
and the task is due again. Nor is a run its host cancelled for lost
contact or for the host's own shutdown (../hosts/hub.md, 6), whose answer
ends its attempt interrupted in the same way, with the saved work it
names kept for the next attempt. A kept
task is never due, and a run of it that finishes done parks instead
(8). A standing task's run that finishes parks too (`standing.md`,
section 2).

**Nothing left to await.** After the ordinary unread-message wake rules,
a result-ended agent task that parks still idle is due again if it is not
kept, owes no requester question, has no open question, live delegate or
pending proposal, and has no executor-taken timer or task watch. Topic
subscriptions do not count: news is a hint (`messages.md`, 2). The tasks child asks the
core to number a `NothingAwaited` notice whose words are: "Nothing more
will come: your delegates have ended and no question waits. Finish with
your result." It always wakes and merges in the dependency-met notice
slot: an unread dependency-met notice already wakes it before this rule
is reached. Each such park consumes a run try, retained across these
wakes; exhausting the run tries ends the task failed. A task ended by its requester may remain idle (9), and so may a standing one (`standing.md`, section 2). A task ended by its closer is
not due by this rule: where it would be, jig makes its closer instead,
and numbers no notice (11).

**A question left open wakes it again.** After the ordinary
unread-message wake rules, an agent task whose run parks idle while a
question asked of it is still open (`messages.md`, 2) is due again at
once, `question` among its reasons (`messages.md`, 6), when all of
these hold:

- its wake policy lets a question wake it (`messages.md`, 3);
- its runs start from a brief, never resuming (../engine/executors.md,
  2), since a resumed run's window already holds the question and a wake
  would bring it nothing new;
- it is no conversation, does not stand (`standing.md`), and is not
  ended by its requester (9): each waits on its person or its session;
- it awaits nothing that would bring the answer: it is not kept (8), and
  has no pending proposal, no live person delegate, and no
  executor-taken timer or task watch naming a live task.

Its next run's brief shows each open question (../engine/briefs.md,
1.2). No notice is numbered: the open question is the reason. Each such
park consumes a run try, retained across these wakes as Nothing left to
await's are. Past its run tries the task is held, as a task past its
tries is (6), for `Failures` of the run's class, its hold's words
`Task A's question N is still open: LINE`, A, N and LINE the oldest open
question's asker, number and first line, cut at a character's edge to
the 128 bytes a hold's reason keeps (`REASON_BYTES`), so that whoever
the hold reaches sees what waits. A task held so is not failed: its
delegates go on, and a release counts its tries again from it.

## 3. Procedure tasks

An active procedure task is stepped by the part of the engine that owns
its procedure: a connector (../connectors.md, section 6), or this child
domain for the core's (`README.md`, section 5). The tasks child hears a step's
decisions (effects asked, tasks made, amended or cancelled, proposals,
messages up, the result) as one decision, committed with the
procedure's state.
It steps when it is first active, when a message its policy lets through
arrives, and whenever what it reads changes, which its owner knows
(level-triggered). Kept, it steps as ever and reads which of its
dependencies are pending; a step that asks an effect waiting for them,
or ends the task done, is refused whole (8).

A connector's due step may also replace its own task's specification
words through the optional `words` field on `ProcedureStep` and
`ProcedureDecision`. The root admits it only for the due task's connector
and procedure code. Words and byte-valued parameters together must fit
`Limits::spec_bytes`, as at creation. Parameters, inputs and `asks` stay
unchanged. This saves with the step's decision; it creates no revision,
amendment, inbox message, wake or amendment history, and cannot target
another task or a person, agent or job task.

## 3.1 Job tasks

A job task (../model.md, 3.2; ../hosts/hub.md, 9) is made by a procedure's step,
as its delegate, with the job's kind, its spec, its workspace's items
and its deadline. It is due once its dependencies are
done, claimed by the fleet on a host that runs its kind, active while
it runs, and ends done with its result; past its tries it is held, as
any task is (6), and its procedure, hearing it, decides what that
means for its own work. It
holds what its items write as any task does (`holds.md`), or writes
under its requester's hold. It takes no messages: words, questions and
amendments to it are refused, and a cancel ends its job at once. It
costs nothing from a budget but the host's time, which its deadline
bounds.

- **Its record:** the executor `Job { kind }`; its spec, bytes of the
  application's own within the kind's declared spec bound (../hosts/hub.md,
  9), never words or parameters, which are UTF-8; its items, prepared
  by its procedure's connector as a run's are (../connectors.md, section
  10); and its deadline.
- **Its result:** a contract of its own, `Job`, the application's bytes
  within the kind's declared result bound, never decoded as an agent's;
  its requester hears it as any delegate's result (7), and its own
  failures (its deadline passed, its delivery lost) are a job's, held
  past its tries.
- **Its authority:** a creator's delegation names the job kinds it may
  make (../authority.md, section 3), as it names charters and procedures.

## 5. Person tasks

An active person task is in the inbox of the party, or of the parties
holding the role, its executor names (../people/README.md, section 7). Its result
is the answer one of them gives, checked against its contract. A person
task waiting past its stall (`README.md`, section 6) is news for its requester,
which may amend, cancel or re-address it. Kept, it stays in the inbox,
shown waiting on its dependencies, and an answer is refused, naming
them (8).

## 6. Failures and holds

- **Tries, within a task.** Except for `Exceeded` below, a run or a step
  that fails is tried again after a jittered backoff, within a number
  of tries per class of failure:
  - *transient:* a peer unavailable, the engine's own reads missing, a
    transcript the agent could not use (../engine/executors.md, 2), a brief's
    required section late or its system failing for now (../engine/briefs.md, 1.1);
  - *permanent until something on a system changes:* a resource missing,
    an identity refused, an assignment its host found invalid, a brief
    whose system answers wrongly or whose required section cannot fit
    (../engine/briefs.md, 1.1);
  - *the run's own:* it reported failing;
  - *the agent's:* it crashed, broke its channel's rules, stalled;
  - *lost:* its host went away;
  - *invalid:* a result that broke its contract. A start its host
    refused before the run's first turn gave no result, so it is never
    invalid: one refused for its transcript is transient, one refused as
    any other invalid assignment permanent, and one refused busy, or for
    a version its host does not read (a charter, a turn or the
    application's items of another version), no failure (below).

  A refusal before anything ran (a host busy, a writer slot taken, no
  room to gather a brief, a version mismatch, its notice naming the wire
  and both versions: `../../protocol/channel/wires.md`, 6) is no
  failure: the task claims again after a
  pause, with no bound, its retry notice saying why. Nor is a run ended
  because its LLM account's source turned limited, refused or in need of
  a sign-in: its task waits for its account, saying why (../accounts.md,
  3.5 and 4.3). Such a terminal, or a checked placement losing its room,
  returns the task to due without a retry counter or polling timer, retaining
  its account and committed conversation. Stale asynchronous preparation also
  returns to due without a timer. A preparation is stale when its task left
  preparing or its newest word changed: the newest of its inbox and of the
  proposals and escalations waiting on it, which never enter that inbox.
  Every attempt, a failed one included, takes a new attempt number.
  A standing task's tries start again as each of its sessions begins
  (`standing.md`, 5.1).
- **A configured bound holds at once.** `End::Exceeded(Exceeded)` becomes
  `Hold::Exceeded(Exceeded)`, with prior lifecycle `Active(Due)` and no
  source. The tasks child's own `Exceeded { bound: Bound, value: u64 }`
  and nine `Bound` tags have the meaning and units of ../hosts/hub.md, 3.
  They are `Copy` and inline. No try is counted and no backoff is set.
  Refusals reset to zero and `retry_notice` is cleared to `None`, so an
  earlier retry notice cannot stand as the current cause. The child
  emits `Fact::Held` and publishes as for any hold.
  Terminal admission still checks the attempt, price, saved resources
  and read fences. Charged spend, `last_answer`, `RunEnded` and committed
  replies are kept. This end bypasses the account Recheck rewrite. A
  requester-ended conversation keeps its requester ending (9).
- **Past its tries, a task is held.** So it is when its executor stalls
  past its bound, a dependency fails or is cancelled, drift breaks a
  resource it relies on (../connectors.md, section 11), an effect it asked
  for fails for good (a procedure's task hears it and steps instead,
  ../connectors.md, 4.2), an effect it asked for is uncertain and of a kind
  that cannot be recovered (../connectors.md, 4.3), its budget or its
  deadline runs out (../authority.md, section 7; for an agent task, too
  little left for its next run's reserve and one completion more of its
  main model, and on a program backend one more again, with nothing to
  free more), a party stops it, it waits for a hold past its
  bound (`holds.md`, 2), its joining a queue would close a circle of waits (`holds.md`, 7),
  its pool lost its allocation (`holds.md`, 6), or its procedure says
  it needs a decision. A hold keeps whatever the task had: its result
  while closing, its inbox, its attempt number. A failure that came from
  preparing keeps its reason on the retry notice and, past its tries, on
  the hold, whose words say it (../engine/briefs.md, 1.1). A classified activation
  failure may carry `End::FailedWithReason { class, reason }`: the same
  retry policy as `End::Failed(class)`, with a UTF-8 reason within
  `REASON_BYTES` (128 bytes), retained on its retry notice and its
  `HoldSource::Preparation` after exhaustion. That source covers both
  preparation and activation causes. The terminal proof stores this
  reason too, so a restart before task settlement retains it.
- **A hold is escalated** (`proposals.md`): it waits for the nearest task or
  party above it that may decide, which it wakes. Whoever it reaches
  decides it: releases the task from it, leaves the task held or passes
  it up. Releasing, amending or cancelling the held task is for those
  `changes.md` names, wherever its escalation waits.
- **A release lifts what it releases:** tries count again from it, and
  what held the task is judged afresh; a deadline still past holds it
  again, with a fresh escalation, and so does a dependency ended failed
  or cancelled, until an amendment removes it (1, `changes.md`).

## 7. Results

- **Checked twice.** An agent's run checks its result against the
  contract before it finishes (smith's `domain/run.md`, section 7); the
  core checks it again as it commits, against the contract and the state
  as it is now. A result that breaks its contract is a failed attempt,
  its reason given to the next run.
- **What a result may carry** beyond its words: effects, through the
  tools that ask for them; follow-up tasks its verdict's contract allows,
  as one batch of the task's own delegates, made in the commit that takes
  the result. Follow-ups beyond the task's authority become one
  proposal, made as the result commits; the result stands either way.
- **Follow-ups are waited for.** The task, closing, waits for its
  follow-ups as for any delegate (1), and for the proposal its result
  made to be decided: accepted, its tasks are delegates like the others;
  rejected, there is nothing more to wait for. Closing does not withdraw
  that proposal (`proposals.md`); cancelling the task does. So its requester
  hears its result, and its dependents start, once what the result asked
  for has been done or refused.
- **A task ends after its delegates.** An executor that finishes while
  delegates of its task are live is refused, naming them, unless its
  finish asks for them to be cancelled, so a task that depends on another
  waits for that task's whole subtree. An agent's run has ended by the
  time its finish is judged and could hear no refusal, so its finish
  parks it instead, with no try spent, and the task waits for its
  delegates' results (../engine/executors.md, 4). Its result goes to its requester
  once it has closed (1).
- **A task ends done after its dependencies.** A done result while one
  of them is pending is not taken (8), so a task's requester and
  dependents never see it done before what it depends on is done.
- **Delivered once,** as a message to the requester, committed with the
  task's end. A party requester sees it in their inbox.
- **Kept.** A result is read by every task that depends on its task, or
  names it among its inputs, in the brief of its next run (../engine/briefs.md,
  section 1), and by its requester as a message. A connector's result is
  read through its connector's section of that brief.
- **Failure is a result.** An executor that finds the task cannot be done
  as asked ends it failed, saying why. A failed task's dependents are
  held (6); its requester decides what follows.

## 8. Kept by a dependency

A task its requester ends is not kept: it may start and run as ever,
and only its ending differs (9).

An active task with a pending dependency is kept: it stays active, with
its holds, its delegates and what its executor has done, and goes no
further until each is done. The tasks child keeps the promise, not the
executor: a charter or a procedure may also behave, but the refusals
hold if they don't. While kept, a task:

- **starts no run.** An agent task is never due. A run already live
  hears the amendment that kept it (`changes.md`), and may finish its turn.
  Its last pending dependency done or removed is a notice that wakes it
  (`messages.md`, 1), and its next run goes on from what the last one left, its brief
  saying what it waited for (../engine/briefs.md, section 1).
- **asks no effect that waits for dependencies.** Its connector says
  which kinds of effect do (../connectors.md, 4.1). One asked while a
  dependency is pending is refused, naming it.
- **ends no way but failed or cancelled.** An agent's run that finishes
  done, with a result its contract takes, parks instead: its saved state
  is kept, no try is spent, and the task's activity names what it waits
  for (../engine/executors.md, 4). A procedure's step that ends it done is refused
  whole (../connectors.md, 6.2). A person's answer is refused, naming the
  dependencies (../people/README.md, section 7). A failure or a cancel ends it as
  ever.

What reaches it:

- **Messages wait in its inbox,** as for a waiting task, and wake it, as
  its policy says, once it is no longer kept. A live run hears what its
  policy lets through, as ever, and a procedure steps as ever (3).
- **Decisions wait above it.** A proposal or an escalation that would
  wait at a kept task waits at the nearest holder above it instead,
  since a kept task starts no run to decide it (`proposals.md`).
- **A cancel's grace** reaches a live run, as ever. A kept task with no
  live run, a kept person task, and one whose run parks during the grace
  cannot finish on their own, and are cancelled at once (`changes.md`).

What it keeps:

- **Its holds and pool slots.** Releasing them would run their
  connectors' release effects, and taking them again would mean queueing
  whole. So a kept task keeps them, and the waits they make for others
  are checked to close no circle (`holds.md`, 7).
- **One rule for going on.** Whether an active task may go on, and why
  not, is decided in one place of the tasks child, its reason typed: a
  pending dependency is one. A further reason to keep a live task from
  going on, its state and holds kept, is one more reason there, not
  another rule.

**Owing a question keeps it too.** A task its requester made owing it
a question (its spec's `asks`, set at creation and never amended, as
an application marks work to be agreed before it is done) owes one until
its first question-form message to its current requester is admitted.
Admission sets the durable `TaskRecord.asked_requester` latch, which is
never cleared: answering or retiring that question does not make the
obligation owed again. Before it has asked, its run that finishes done
parks with its saved state, consumes a run try, and hears a notice
naming what it owes that wakes it at once. Its next run goes on from
what the last one left with that notice to read. Exhausting its run
tries ends it failed; each run also spends within the task's budget.
Once it has asked, its own question keeps it, as below.

**Its own question keeps it too.** A task that asked its requester a
question, in the question form, still unanswered, is kept as by a
pending dependency, the reason naming the question: it ends no way but
failed or cancelled, and its run that finishes parks without spending a
try, whatever the form of its finish, `failure` among them, its activity
naming the question it waits for. A run that has asked has no reason to
finish until it is answered, so its finish is not taken: its result goes
with its attempt, and its next run hears the answer. This is a rule of
any task with its own question open, and of the run's own finish alone:
a run that fails without one (its tries, its budget or its account
spent), a cancel and a procedure's step that fails end it as ever. Only an
answer in that question's number releases this keep and wakes it (`messages.md`, 1),
letting its next run go on from what the last one left. An instruction
amendment, a note, or the person's answer to someone else is not that
answer. Its delegates, its holds and its other messages go on as for
any kept task. A question that ends unanswered lifts the keep: its asked
task ending with it open, or its cancel, wakes the asker with a notice
naming it. A move of the asker to a new requester (`changes.md`) carries its
open questions to that requester in the move's decision, each keeping
its number and activity line, as a successor's transfer does, so a goal
moved to its party asks that party directly and stays kept. A question
it asked through a person task, in words, is that task's, and detaches
as `changes.md` says; it keeps nothing.

**A credit names the question's parties and line.** Each open question
keeps a `QuestionCredit { number, answerer, asked, line }` in its asker's
record: `answerer` is the party allowed to use its reserved answer room,
`asked` the logical addressed party, each a `Party`; `line` is the
position of the question's activity line in the asker's record. Words
and time are read from that line, never copied into the credit; the
line does not name the addressed party.

**Question notices say which words they carry.** `QuestionEnded` carries
empty words, its kind naming the question's number. `QuestionAnswered`
names the asker in its word's `from` party and carries the person's
answer words within 256 bytes, including a
`[N bytes cut]` line when cut, with the prefix ending at a UTF-8 boundary.
Each received question reserves those 256 bytes beside its notice slot.
Stored notices with empty words still restore. `QuestionOwed` carries the
instruction: "You owe your requester a question: ask your plan with message, as a question, and wait."
Unanswered questions retired together produce one `QuestionEndedDue`
batch per asker; the core numbers each notice once.

## 9. Ended by its requester

A task is ended by its result, by its requester or by its closer (11),
or it stands, ended only by a cancel (`standing.md`, section 2):
its `ended_by`, fixed at its making. A result ends it by default. Its requester may end it
only if it is an agent's conversation with a person, its requester;
making it refuses any other shape, and a restart takes what was made
(../engine/restart.md). It changes no right
to cancel. It travels with the task from the client's making, through
storage and preparation, to its run's charter (../hosts/charters.md, 1),
whatever the charter the task's agent shares with others.

A task its requester ends:

- **starts and resumes as ever.** Only a pending dependency keeps it
  from a run (8).
- **takes no run's finish.** A run that finishes, with any result,
  a failure among them, or with follow-ups, parks before its result is
  judged. The result and its follow-ups are discarded, and no try is
  spent. What the run spent, saved and committed stays, with its turns,
  its read fences, its delegates and its history. The discard writes no
  line of its own: it is the run's ordinary turn end, and the committed
  reply is the history's line. The fences of saved work, of charge and
  of messages are checked first, as for any finish.
- **ends as its requester says,** or by cancel. While it closes, a
  cancel keeps its partial result. A narrowing run goes Due, as ever,
  and a held task stays held.

The kept rule of 8 holds both facts, the dependency and this one,
apart: a task that is held from active keeps its requester's ending,
and one that is held from closing does not. A failure that is not a
finish, the run's own, keeps its retry policy whoever ends the task.
`End::Exceeded` is such a failure, not a finish: it holds at once by
6, retaining the requester ending through the hold.

## 10. Placed holds: owners, modes and scopes

A placed hold is one a party or a task places by an
act, to stop or pause work: a person pausing a goal, a coordinator
parking a delegate, a project drained to change its rules. The holds the
core places itself (6: tries, stalls, bounds, drift, effects, budget,
deadline, circles, a procedure's need) keep their rules.

- **It is a record of its own,** numbered in the deployment: its owner,
  its mode, its scope, what lifts it, its reason in words, when it was
  placed, its revision, and the tasks it exempts. Its revision moves
  when it is lifted, never when it exempts a task, so that accepting
  one task's release leaves its siblings' proposals standing. A lift
  names the hold by its number alone: the core finds its project and
  judges the party's role there, and a hold unknown to them is refused
  as unknown. Each task it holds is held by `Placed { hold }`,
  naming it. It replaces `StoppedBy`: a party's `Stop` is a placed
  hold.
- **No escalation.** A task held by a placed hold raises none: it waits
  for its hold's owner, who knows it, and never passes up or stalls. A
  task held by a placed hold and by a cause of 6 has that cause's one
  escalation, routed as `proposals.md` says; deciding it lifts that cause
  alone, and the placed hold stays.
- **Its owner** is the party or the task that placed it. Only these may
  lift it, or release a task from it:
  - its owner;
  - a party whose role in the project stands above the owner's: the
    project's policy ranks its roles, owner over maintainer over member
    over observer by default, a role of the policy's own with member
    unless it says otherwise (../people/README.md, section 4);
  - for a hold a task placed: that task, any task above it, and the
    parties in whose tree it is.

  Anyone else who could otherwise release the task (`changes.md`) is
  refused, naming the owner and whether a proposal could be accepted,
  and may propose the release (`proposals.md`), which waits for the owner. A
  task above a party's hold never lifts it. A release attached to an
  amendment (an amended task released in the same request) takes the
  same rule.
- **Two modes:**
  - **Stop.** Each live run in scope is cancelled at once, and each task
    in scope is held.
  - **Drain.** What runs finishes, and nothing new starts. A live run in
    scope is cancelled as parked at its next committed turn, its turn
    kept and no try spent, so it ends at a turn's boundary and resumes
    from its transcript; a procedure in scope is told to drain, and
    steps on until its connector says it is idle (../connectors.md, 6.7);
    no due task in scope gets a run; and each task in scope is held as
    it goes idle: an agent's once its run has parked or ended, a
    person's at once, a procedure's once its connector says so.
- **Two scopes:**
  - **A task's subtree:** the task and every task below it, those made
    later included.
  - **A project:** every task of the project, those made later
    included, except each conversation a party requested directly, so
    that a person can still talk to their chat.
- **A drain, in four rules:**
  - a run parked by a drain ends due, not idle, as a narrowed run does,
    no try spent: held while the drain stands, run again on its release
    with no wake;
  - placing a drain steps each idle procedure in scope once, told it is
    draining, so that its connector says whether it is idle
    (../connectors.md, 6.7); a procedure mid-step learns it at its next;
  - a closing task finishes under a drain: what runs finishes; a stop
    holds it;
  - a drain lifted before a task is idle clears its hold and the task
    goes on, a park already asked standing (its run ends due and goes
    on); the tasks it already held stay held, released one by one or
    with the lift's tasks.

  The attempts asked to park are kept in memory, rebuilt at a restart
  from the placed holds and the live attempts, so that a standing drain
  over a live attempt asks it to park again.
- **Nothing new in scope.** A task's batch whose tasks would be in a
  placed hold's scope is refused, naming the hold: a run finishing its
  turn under a drain delegates nothing. A party's batch, or a
  procedure's, is made, its tasks held by the hold in the commit that
  makes them, so that a person may start one task and release it alone.
- **Lifted as one, released one by one.** Lifting a placed hold, naming
  its revision, ends it: nothing more is held by it, and a task it held
  stays held, so that the person resumes goals one at a time, each
  released as `changes.md` says, by those this section names. A subtree
  hold's root is the exception: released by one who may lift its hold,
  or by an accepted proposal of its release, it lifts the hold with
  every task it holds that nothing else holds, so that a stopped goal
  resumes whole. A lift may
  ask to release with it every task it holds that nothing else holds.
  The record of a lifted hold stays, ended, while a task it held is
  still held by it, and goes with the last one's release. A task
  released from a placed hold while that hold stands is exempt from it
  from then on: a revert released under a project's stop lands while
  the stop stands.
- **Parked until an event:** a hold a task places on a task below it may
  name the event that lifts it, with every task it holds: a connector's
  event that its connector classes as waking for the hold (../connectors.md,
  5.3); a named task's end; or a time. The core subscribes the hold to
  it, as it subscribes a task (`messages.md`, 4), and lifts the hold in the commit
  that delivers the event. A party's hold names no event: a person lifts
  their own.
- **Proposing a release:** a task or a party proposes to release a task
  from a placed hold, naming the hold's revision; the proposal waits for
  the hold's owner, whatever the tree's authority, a party's in their
  inbox and a task's among its entries, and only those who may lift the
  hold accept it, which releases the task as their own release would. A
  hold lifted, or its revision moved, withdraws it. Accepting one is one
  step: the release judged as the accepter's own, then made (a root's
  lifting the hold whole), then the proposal accepted and the hold's
  other proposals withdrawn; a release refused refuses the acceptance,
  and changes nothing.
- **One at a time:** a task is held by one placed hold, the first
  standing one that covers it; released from it, it is judged afresh
  against the others.
- **Once per decision:** in one decision, each task's placed hold
  changes at most once. A compound request's operation that would place
  a hold on a task, or let it go from one, after an earlier operation of
  the same request changed it, refuses the request at its place, naming
  the hold; so a decision's room is bounded by the live tasks and the
  placed holds (../engine/README.md, section 7).
- **Moving a held task:** a task a standing placed hold holds, whose
  scope's root lies outside what is moved, is moved by no one (`changes.md`
  ): the move is refused, naming the hold, so that a move never
  releases an owner's hold unseen. Moving the hold's whole scope, its
  root with it, is allowed.
- **Ended with its root:** a subtree hold whose root ends is erased, its
  tasks released from it.
- **Bounded:** placed holds in the deployment, per project and per
  party, the exemptions one keeps, and their subscriptions among the
  subscriptions per task (`README.md`, section 6).
- **Restart** restores placed holds and their subscriptions with the
  tasks they hold, resolving each hold's owner once the people child's
  parties are restored; a drain in progress goes on draining.

## 11. Ended by its closer

A closer ends a task when nothing is left for its agent to judge: jig
decides when, by 2's rule, and the application's procedure decides
what, so that a model wakes for judgement alone and code does what is
mechanical once the task's work is done (temper's `domain/forge/branches.md`,
2, a goal's landing).

- **Named at its making:** `ended_by` `Closer { connector, code }`, a
  procedure executor (3), given by its batch entry or its template
  (`../../protocol/configuration.md`, 3.2), and fixed: no amendment changes
  it. Only an agent task that is no conversation takes one; any other
  shape is refused as `ended_by` is (`README.md`, section 4). Its making is refused,
  beyond authority, where the task's own delegation does not name the
  procedure (../authority.md, section 3), as a delegate's would be.
- **A result-ended task besides:** a run that finishes with a result
  ends it, as for a task its result ends. The closer only takes the
  place of 2's `NothingAwaited` notice.
- **Made where the notice would be.** When the nothing-left-to-await
  rule (2) finds the task due again, and no closer of it is live, the
  tasks child asks the core to make its closer instead: one procedure
  task, the task's own delegate, made by the task as one batch with no
  run: its executor the named procedure, its words the fixed `Close
  your requester: nothing more will come.`, no parameters or inputs,
  its contract the task's own, its authority the task's own whole, one
  level shallower in delegation, and its budget `AsAllowed` within the
  task's (../authority.md, section 7). The task keeps the closer's number
  while it lives, stays idle, numbers no notice and spends no run try.
  A refused making (a budget or a limit reached) holds the task, its
  words naming the closer's refusal (6).
- **Also when its last awaited thing ends while it is idle.** A
  delegate's end, an answer or a watch's end that leaves an idle task
  nothing to await, by the same conditions, makes its closer too, with
  no run between: a last result under `last_or_failure` (`messages.md`, 3) does not
  wake it, where a failure still does, and the result stays in its
  inbox, offered to its next run if one comes.
- **Its funds follow its task's runs.** The closer's budget grows by
  what its task has left whenever the task is idle or waiting and the
  closer is not closing: after a delegate the task funds ends, after a
  run of the task answers, and after an amendment of the task. A run of
  the task, as it is claimed, takes back what the closer holds
  unreserved, the closer's budget becoming what it and its ended
  delegates spent and what it holds reserved, so that the task's
  judgement is never starved by its closer. Each of those growths makes
  the closer due (`Changed`, as a procedure's changed inputs do) where
  it is idle, whether it gave anything or not, so that a step the
  closer could not fund while a run held its funds is decided again
  once that run has answered (temper's `domain/forge/branches.md`, 2).
- **Its success ends the task:** the closer ending done, its result is
  the task's: checked against the task's contract as a run's finish is
  (7), and the task closes with it (1), no run claiming it. A
  result the contract refuses holds the task, `Contract`, its words
  naming the closer. Its requester hears the task's result, never the
  closer's.
- **Its failure is the task's to judge.** A closer held, by a failure
  past its tries or a hold of its own (6), escalates to the task as
  any delegate does (`proposals.md`), and the escalations of the closer's
  own delegates pass the closer, a procedure, to the task. A closer
  that ends failed or cancelled is a delegate's result, which wakes the
  task by its policy; one that ends failed spends a run try, as a park
  with nothing to await does, so that a closer failing every time
  ends the task failed. The task's next park with nothing to await
  makes a closer again.
- **New work keeps it waiting.** A delegate the task makes while its
  closer lives (by a run, an accepted proposal or a batch) becomes a
  dependency of the closer in the same commit, added by jig as an
  amendment adds one (`changes.md`), so the closer is kept (8) until
  that work has ended: it steps, and cannot end done, nor so end the
  task, before the work. Its procedure reads the work as any kept
  procedure reads its dependencies, and decides what its own steps need
  again. A batch that would make a circle of waits through the closer is
  refused as any is.
- **Closed with the task:** a task that closes otherwise (a run's
  finish, a cancel, its requester's) closes its closer as any delegate
  (1). Restart takes the closer as any task, and the task's record its
  number.
