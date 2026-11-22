# A change's holds and releases

This document holds a change's holds and releases, split from [`repairs.md`](repairs.md).

## 1. Holds and releases

- **A verdict whose post is refused** (`gates.md`, 1) is no repair and no stall:
  the gate counts as temper's verdict says, nothing is held for the
  post, and its `Forbidden` narrows what temper may do in that
  repository, as any refused write does (`../adoption.md`), so no later
  verdict is posted there.
- **Bounded apart:** repairs past their limit, resolutions past theirs,
  updates past theirs, hold the change, saying why, and its requester
  hears.
- **A step its authority no longer covers holds it too.** A change's
  delegates (its producer, a review, a review asked again, a repair, a
  resolution) are made under the change's own authority, its tasks
  consumed by each (jig's `domain/authority.md`, section 5). A step whose
  delegate that authority no longer covers (the tasks its subtree may
  make all made, its depth, its spend below a run's reserve) holds the
  change, naming what it lacks and what it had ("its 6 tasks are
  made"), and its requester hears; it never waits due with nothing to
  run. Released, it asks again: a release after an amendment that gives
  what it lacks (more tasks or budget, a widening its giver funds:
  `../../../client/ctl/README.md`, area 2's `task amend`) lets it go on, a bare one
  holds it again, saying so. The requester and panel's recovery words
  offer `task cancel TASK REASON`, or asking for the change afresh, and
  explain that bare release holds again. A task allowance hold also offers
  `task amend TASK --rev REV --tasks N REASON`, followed by
  `task release TASK --hold HOLD`, with room for a repair and review in
  the replacement lifetime allowance. Cancellation comes before these
  commands so a bounded task summary retains a way out. They advertise
  only commands the client takes.
- **Stalls:** every wait of a change on the forge is bounded, and past
  its bound the change is held. What a stall counts and what it does
  not, how checking counts CI by its execution, and what a stall's hold
  says are `stalls.md`'s.
- **Every hold reaches its task.** A change whose step fails for good
  (its producer's, a resolution's, a repair's, a job's or a write's)
  ends failed or holds, and its hold is its task's hold, never its
  row's alone, so the escalation wakes its requester with the hold's
  words (2026-11-12). The hold does not ride on its step's fence: a
  step carrying it that turns stale, or is refused whole (its words to
  the requester refused, say), still holds the task, with the same
  cause and words. A change found held at any later step or once
  restored, its task live and neither held nor ending, holds its task
  again with its hold's cause and words, unless a party released the
  task: that release's step lifts the change as ever (A release, below).
- **A result that arrives on a held change is kept.** A delegate ending
  while its change is held on a stall at that delegate's wait ends the
  hold, with no party's release, as a green head does (above): the change
  goes on from the state it was held at, its stall counted from the result,
  the deployment releases its task (jig's `domain/tasks/changes.md`),
  and the result is taken as any is: a head and a title open the pull
  request, a no-change ends the change. Under any other hold, the result
  stays on the change and a release takes it, as the hold's own words say;
  a no-change under one is said by the hold, since nothing else would.
- **A dependency's end ends a stall at producing.** A change held on a
  stall at producing with no producer asked for, whose task's last
  pending dependency ends done, is released with no party's release, as
  a result ends a stall at its delegate's wait (above): the change goes
  on from producing, its stall counted from that end, the deployment
  releases its task, and its producer starts as at any producing. This
  clears a hold placed during a dependency's wait before the stall
  stopped counting there (`stalls.md`, 1). A dependency that ends
  failed or cancelled ends no hold: it holds the change as any failed
  dependency does (jig's `domain/tasks/lifecycle.md`, 6), as before.
  A hold of any other cause, or a stall at producing with its producer
  asked for, stays for a party's release.
- **A release** lifts what it releases: repairs, resolutions and updates
  count again from it (not from the deployment's, ending a hold at a green
  head, above), its stall is counted from it, and the change decides
  afresh. **Deciding afresh reads afresh** (2026-11-13): the change
  reads its branch, its base and its pull request again, a read still
  outstanding from before abandoned, its answer changing nothing when
  it comes; and it reads its delegate's end and result as its task's
  record keeps them, its pull request's title among them, but for a
  delegate the release retires (*In order*, below). So a change
  whose producer's result had already arrived moves on at the release.
  **In order** (2026-11-15): a release takes effect once, in four steps,
  in this order, whatever restart falls between two of them:
  1. **Once.** A party's release names the hold revision of the
     change's task (jig's `domain/tasks/holds.md`). The change keeps,
     on its row, the last revision it took, and takes a release only
     above it: a second delivery of one release, or an older one,
     changes nothing, before a restart or after it.
  2. **Fenced.** Before anything is read, the release abandons the
     gather outstanding and fences the change's branch and pull
     request: the answer of a read of either asked before the release,
     a gather's, a watch's poll or a hint's, changes nothing when it
     comes (`../keeping-up.md`, 3). Released from a delegate that
     failed or was cancelled, its producer, a repair, its resolution or
     its merge job, it retires that delegate: its status, its failure's
     words and its result are never read again, from the change's row,
     its task's record or an archived one, before a restart or after
     it. A result kept on another hold is no failure, and is taken (*A
     result that arrives on a held change*, above).
  3. **Read afresh.** It reads its branch, its base and its pull
     request afresh (`../keeping-up.md`, 3).
  4. **Decided** on those facts alone, by the rules for the step it
     was held at, below and in `README.md`, 2.
  A party's release of the change's task (a party's stop is a placed
  hold: jig's `domain/tasks/lifecycle.md`, 10) does the same, though
  the change itself was not held. A producer that fails for good holds the change at
  producing, whatever it delivered first: its pushed work stays on its
  branch, no pull request opens, and the hold carries its words whole
  within the hold's bytes,
  so its requester hears why (its notice fitted to the requester's
  message bytes on its own, marked when cut) and its person decides:
  cancel it, or release it if the producer was wrong; a chat holds no
  decision of its own here. At producing (its producer failed for good),
  deciding afresh makes a fresh producer, one more of its tasks, whatever
  the last one pushed, holding by the rule above when its authority no
  longer covers one. With a head pushed, its workspace starts from that
  branch, and only its success gives the pull request its title (`README.md`, 1). A
  repair or a resolution that fails holds the same way, whatever it pushed
  first: its head is not taken as the change's. A repair that fails for good
  holds at its attempted count; this is a delegate failure, not repair
  exhaustion. Released at a head CI has not passed at (a green head ends
  its CI repair, above), it asks for a fresh repair with the same reason
  and current forge facts, under its current authority, counted against
  the reset repair allowance; held on its repair's flake verdict, it asks
  none and names its regate instead (*A landing repair that finds only
  flakes*, above). The
  replacement's result supersedes the failed delegate's, including
  after a restart; a pending replacement is not asked twice.
  Released from a resolution or a merge job that failed or was
  cancelled, the change decides afresh as it does once its producer
  ends (`A resolution`, above), on what step 3 of *In order* read:
  never the ended delegate's status, words, result or head. It keeps
  the head it read as its last head, the release taking it as a
  drift's release takes a moved head (`drift.md`, 1), the branch's
  known head with it, so that head raises no drift later; the failure
  alone never takes a head. With no conflict left, it goes on at that
  head, checked again (`README.md`, 2, step 3). With a conflict left,
  it asks exactly one fresh resolution, directly, pinned to that head
  and its base as now read, with no merge job before it (below);
  under `turn`, the preparation that judges the conflict is made at
  that head and main's tip as now read. The resolution is counted
  against the allowance the release reset, and held by the rule above
  when its authority no longer covers one. Since the ended delegate is
  retired, the change itself keeps, on its row, that it was released
  from a resolution or a merge job, so that a restart before the fresh
  resolution is asked still asks it directly. A task still live that
  pushes its branch is waited for, as there. The fresh resolution is
  recorded as asked in the step that asks it, so it is asked once, a
  restart included, and the change never holds again on the old
  delegate's status, words or head.
  A delegate a person cancels holds at the same count and is released
  the same way, and its hold says what happened: `its repair, task N,
  was cancelled`, with the cancel's reason where it gave one, within the
  hold's bytes (`its resolution`, `its merge job`, `its producer`). A
  cancelled repair, resolution or merge job holds as a decision of the
  procedure, not as a failed effect; a cancelled producer, a catch-up's
  first merge job among them, keeps a failed producer's hold, carrying
  those words.
  A change is a procedure, with no run of its own to repeat: a
  release asking to run it again is refused saying so, never as
  missing authority.

A catch-up's first merge job is its producer and spends no resolution
allowance. If it leaves conflicts, its following resolution spends one. This
differs from a conflict after production, whose merge job spends one and
whose following resolution spends none. A resolution asked again after
release is a resolution directly, with no repeated merge job before it
(2026-11-04).

A close job held for its failures stays its change's delegate while it
lives (2026-11-12). Its change holds at producing on that failure, as on
a producer failed for good, carrying its words; the job's escalation
reaches the goal above as any held task's, so either task may be
released:

- **The job released** runs again as its change's close job, in the
  change's checkout as on its first attempt: the change's repository,
  its own branch writable, made at the goal's branch's tip
  (`../../../protocol/tooling.md`, 4.2; `../../worker/jobs.md`). Its end is
  the change's producer's result, and it ends the change's hold on that
  failure with no release of the change, as a delegate ending ends a
  stall at its wait (a result that arrives on a held change, above): a
  head pushed opens the pull request, nothing staged ends the change with
  no change, and a failure for good holds the change again with the new
  words.
- **The change released** decides afresh as at producing, with a fresh
  close job; the held one is cancelled once the fresh one is made and
  before it starts, its reason `superseded by its change's fresh close
  job, task M`, M the fresh job's task, so that no close job its change
  no longer names runs.

A close job reserves none of its change's spend: it runs no model and
makes no children, so a held one leaves its change's budget whole for a
fresh one (2026-11-12). A merge job's reservation is unchanged.

## 2. Open questions

None.
