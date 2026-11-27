# A task's lifecycle: failures and holds

## 1. Failures and holds

- **Tries, within a task.** Except for `Exceeded` below, a run or a step
  that fails is tried again after a jittered backoff, within a number
  of tries per class of failure:
  - *transient:* a peer unavailable, the engine's own reads missing, a
    transcript the agent could not use (../engine/executors-transcripts.md, 1);
  - *infrastructure:* a preparation that failed for now because a system
    it needs did: a workspace its host could not prepare for now
    (../hosts/hub.md, 3), for a run or a job alike, or a brief's required
    section whose system failed for now or that was not gathered by the
    brief's deadline (../engine/briefs.md, 1.1), or a dependency's or
    input's result whose load the store answered busy (the same); or a run whose model's
    provider could not be reached, its model call failed unavailable or
    overloaded with no call reaching the model, its evidence unsent (an
    outage, ../hosts/hub.md, 3). Its allowance and backoff are its own
    (below);
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
  a sign-in: its task waits for its account, saying why (../accounts-sources.md, 2, and ../accounts-uses.md, 1.3
  ). Such a terminal, or a checked placement losing its room,
  returns the task to due without a retry counter or polling timer, retaining
  its account and committed conversation. But an account's terminal read
  once its source is usable again, its limit's reset zero or already past,
  is a refusal: the task claims again after the pause a refusal takes,
  spending no try, its retry notice saying why, so that a source that
  keeps ending runs never has one start assigned again without backoff
  (the user, 2026-11-19). Stale asynchronous preparation also
  returns to due without a timer. A preparation is stale when its task left
  preparing or its newest word changed: the newest of its inbox and of the
  proposals and escalations waiting on it, which never enter that inbox.
  Nor is a run whose program started without its tools a failure of its
  task's: the fault is its place's, the program source's login on its
  worker (../hosts/hub.md, 3). Its attempt ends once, refused as an
  untranslated assignment's is, with words naming the program, the
  source and the worker on its retry notice, spending no try; the task
  then claims again, never on that place until something changes there,
  and with no place left waits, saying why (../engine/fleet.md).
  Every attempt, a failed one included, takes a new attempt number.
  A standing task's tries start again as each of its sessions begins
  (`standing.md`, 5.1).
- **Infrastructure failures wait longer, within a bound of their own.**
  A forge, a network or a model's provider down for minutes, or a burst
  of preparations timing out together, holds no task, while a run's own
  failures keep
  their tries. Each class counts its own tries: an infrastructure
  failure spends none of another class's, and another class's failure
  none of its. Its defaults, both profiles alike: 12 tries, its backoff
  starting at 10 seconds and doubling up to 5 minutes (the same jittered
  backoff as every class's), so a task fails for its infrastructure for
  about half an hour, between some 20 and 40 minutes, before it is held.
  Every other class keeps 3 tries, from 1 second up to 30. They are
  configured as every class's are, `tasks.retries.infrastructure.retries`,
  `.base_ms` and `.max_ms`. Past them the task is held for `Failures` of
  the infrastructure class, as for any class, its hold's words its last
  failure's reason, which names what failed: the host's detail, cut as a
  failed job's is (../hosts/hub.md, 3), the brief's section and words
  (../engine/briefs.md, 1.1), or the model call's words, from its
  structured fields alone (../hosts/hub.md, 3). A preparation that always fails still
  holds, past that bound. A run whose account waits turns an
  infrastructure failure into a recheck as it does a transient one
  (../accounts-sources.md, 2).
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
- **A wait past its bound holds** (`../engine/waits.md`, 5;
  `temper/docs/design/draft/bounded-waits.md`, 4.2). At its kind's figure, a
  `news`, `host` or `account` wait holds its task for `Waited`; at the
  task's deadline, a task that isn't working (`../engine/waits.md`, 4),
  in any wait, a `dependency` or `delegate` one among them, is held for
  `Deadline`. Either hold's source is the words
  of the waits it was in, made by the core at the bound, naming each
  wait's kind, its object, its *since* and its figure or deadline:
  "Waited 24 h for news since 2026-11-24T06:07Z: subscriptions 3 and 5,
  a timer at 2026-11-25T06:00Z." The deadline is the one suspension
  moves (below), and the record's bound is that same instant.
  - *The core's bound alarm:* the core keeps one alarm for each live
    task, at the earliest bound among those of its waits whose outcome
    it takes at the alarm (these, and a question's pass up,
    `messages.md`, 2), and moves it whenever the task's waits may
    change, as the task is saved; a start arms it again for every live
    task. When it fires, the core computes the task's own waits at that
    instant, as a line's are (`../engine/waits.md`, 4), and in one
    decision takes the outcome of each wait past its bound, in their
    order: a wait that has moved since, or begun again with a later
    *since*, is left, and the alarm moves to the next bound. A crash
    before that decision commits makes it again at the next start, and
    a decision made once is not made twice.
  - *What it keeps:* the hold keeps whatever the task had, as every hold
    here does: its place in a queue, its reservations, its effects in
    flight (an uncertain write keeps its read-back as its wake, and its
    outcome is recorded whenever it comes), its subscriptions and timers.
    A live run that waits (`../engine/waits.md`, 3, `news`) is cancelled
    as parked, as a drain parks it (2), its turn kept and no try spent,
    and the task is held once it has parked.
  - *Its clocks:* a wait with a clock of its own keeps it and its
    owner's outcome: a resource's queue holds for `HoldsWaited`
    (`holds.md`, 2), an escalation and a proposal pass up at their
    stall (`proposals.md`), a person task at its own
    (`../people/README.md`, 7). A standing task's `news`, and a
    `dependency` or `delegate` wait in a task with no deadline, have no
    bound and never hold for it.
  - *Released,* the task starts a fresh wait, with a new *since*; the
    hold and the wait before it stay in its history. A deadline still
    past holds it again, as below.
- **Past its tries, a task is held.** So it is when its executor stalls
  past its bound, a dependency fails or is cancelled, drift breaks a
  resource it relies on (../connectors.md, section 11), an effect it asked
  for fails for good (a procedure's task hears it and steps instead,
  ../connectors-effects.md, 1.2), an effect it asked for is uncertain and of a kind
  that cannot be recovered (../connectors-effects.md, 1.3), its budget or its
  deadline runs out (../authority-budgets.md, section 1; the deadline moved
  later by the time the task spent suspended, ../engine/executors.md,
  6.3; for an agent task, its
  share for its next run, half of what it has left where it may start a
  delegate and that half is above a run's minimum, too little for that run's reserve and one completion more of its
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
  reason too, so a restart before task settlement retains it. A run
  that failed for a named model call carries the call's words so, in
  the infrastructure class for an outage and the run's otherwise
  (../hosts/hub.md, 3), and its hold names the call.
- **A hold is escalated** (`proposals.md`): it waits for the nearest task or
  party above it that may decide, which it wakes. Whoever it reaches
  decides it: releases the task from it, leaves the task held or passes
  it up. Releasing, amending or cancelling the held task is for those
  `changes.md` names, wherever its escalation waits.
- **A release lifts what it releases:** tries count again from it, and
  what held the task is judged afresh; a deadline still past holds it
  again, with a fresh escalation, and so does a dependency ended failed
  or cancelled, until an amendment removes it (1, `changes.md`).
- **A measurement is a release that asks for one** (`Measure`,
  `../people/acts.md`, 1): it lifts a procedure's own hold as a release
  does, at the revision it names, and the task is due as measured
  (`Measured`, beside `Released`), so that its procedure's next start
  reads that it was asked to measure afresh what its hold judged. What
  it measures is the procedure's (temper's
  `domain/forge/change/repairs-releases.md`, 4); jig only carries the
  ask, once per accepted act. It is offered for a procedure's task
  alone, since an agent's run judges nothing to measure, and never
  lifts a placed hold or settles an effect held uncertain, which keep
  `Lift` and `Release`'s choice.
- **A claim's cause stays readable while its attempt is placed,** so that
  an application can say why each assignment left (temper's
  `domain/worker/hosted-runs.md`, 1.3): the due reason the claim found, the
  retry notice it clears, and whether the attempt before it ended in a
  recheck, which no due reason names. The tasks child holds it in memory
  alone, beside the task's record and never in it, nor on a wire, from the
  claim until the task's next claim. A restart loses it, and the due
  reason the record keeps answers instead (the user, 2026-11-19).

## 2. Placed holds: owners, modes and scopes

A placed hold is one a party or a task places by an
act, to stop or pause work: a person pausing a goal, a coordinator
parking a delegate, a project drained to change its rules. The holds the
core places itself (6: tries, stalls, bounds, drift, effects, budget,
deadline, circles, a procedure's need) keep their rules.

- **It is a record of its own,** numbered in the deployment: its owner,
  its mode, its scope, what lifts it, its reason in words, when it was
  placed, its revision, and the tasks it exempts.
- **Its reason is never empty.** Every placement gives words saying
  why: a party's `Stop` and `Hold` each carry a reason
  (`../people/acts.md`, 1), and so does a task's or a procedure's
  placement. One with an empty reason is refused before anything
  changes, with the refusal a message's empty words take, a limit named
  empty, and nothing of it is held. A stored hold placed before this rule,
  its reason empty, reads as it is; its words say it was placed with no
  reason (3). Its revision moves
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
    steps on until its connector says it is idle (../connectors-procedures.md, 1.7);
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
    (../connectors-procedures.md, 1.7); a procedure mid-step learns it at its next;
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
  event that its connector classes as waking for the hold (../connectors-facts.md,
  1.3); a named task's end; or a time. The core subscribes the hold to
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

## 3. A hold's words

Every hold has words (`temper/docs/design/draft/bounded-waits.md`, 8):
one text, which the tasks child makes from the held task's record
whenever it is read, so that it is never stale and a hold stored before
it has words too.

- **What they say:** its cause, naming its object by number or value;
  since when the task has been held, the instant its present hold began;
  and its way out, the act that ends it: "Waited 24 h for news since
  2026-11-24T06:07Z: subscriptions 3 and 5. Held since
  2026-11-25T06:07Z. Release it to wait again, or amend or cancel it."
- **Its since is kept:** the task's record keeps the instant its
  present hold began, stamped with the hold's first line and left as it
  is while the held task's prior phase, or its cause's facts, move;
  each such move still writes its hold line and moves the phase's clock.
  A record stored before it has none, and reads the latest hold line's
  time. The words are made at read; only that instant is stored.
- **From what the record keeps:** the words name only facts the tasks
  child keeps. Where the table names a fact it does not keep, they say
  what it does: a dependency no longer retained is named by number as
  failed or cancelled, and a queue's holder is named where it is known.
- **Bounded:** at most 1,024 bytes of UTF-8. A source's words or a
  placed reason too long for the rest is cut at a character's boundary
  and marked cut. The task's and the chip's document bounds keep that
  room, as they keep a connector's account
  (`../../client/domain/contract/documents.md`, 3).
- **Where they are read:** the chip's hold, in the task's document and
  each line that carries it, as the hold's words
  (`../../client/domain/objects.md`, 5); a run's read of the task; and
  the escalation's entry its holder hears (`proposals.md`, *Waking*),
  so that a task holding an escalation reads why in the entry itself.

| Held for | Its cause and object | Its way out |
|---|---|---|
| a configured bound (`Exceeded`) | the bound, and the allowance it reached | lighten what was asked, or raise the allowance, then release |
| out of tries (`Failures`) | the class, and the last failure's words | release, to try again; amend first where its instructions failed it |
| stalled (`Stalled`) | its executor made no progress within its stall bound | release, to run again |
| a dependency (`Dependency`) | the dependency, and how it ended | amend it to drop the dependency, then release |
| stopped (`Stopped`) | that it was stopped | release |
| a placed hold (`Placed`) | the hold, its owner and its reason, or that it was placed with none | its owner lifts it, or releases this task from it (2) |
| drift (`Drift`) | the resource, and its connector's words | release, which does what the connector says |
| an effect (`Effects`, `EffectFailed`) | the effect's entry, and its connector's account | release, for its executor to decide afresh |
| an uncertain effect (`Uncertain`) | the entry, made or not on its system | look at the system, then release, making it again or not |
| its budget (`Budget`) | what it spent, and that its next run or effect can't be funded, with the need where it is given | widen its budget, then release |
| its deadline (`Deadline`) | its deadline, and the words of the waits it was in, where a bound made it | move its deadline, then release |
| a resource's queue (`HoldsWaited`) | the holdings it asked for, the task holding one where known, and since when it queued | release, to queue again; or amend to drop it |
| a resource gone (`ResourceUnavailable`, `ResourceAccess`) | the resource, and its connector's account | release once it is back, or amend to drop it |
| its pool (`PoolLost`) | the pool | release, to wait for a slot |
| its procedure (`Procedure`) | its connector's account | release, its counts reset |
| a circle (`Circle`) | the circle's tasks in order, cut where cut | it lifts once the circle breaks; amend a task on it to drop a dependency |
| a wait's bound (`Waited`) | its source's words: the wait, its object, its *since* and its figure | release, to wait again |

Every way out but a placed hold's also has amend and cancel, as
`changes.md` lets their party; the words name the one that ends the hold.
