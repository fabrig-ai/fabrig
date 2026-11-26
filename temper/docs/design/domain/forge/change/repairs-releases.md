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
  hears. So does CI whose runner's faults outlast `reruns` at a head
  (`README.md`, 2, step 3): its cause the runner's, `Runner`, its words
  naming each job, its check and its fact (`set-up failed`, `completion
  failed`, `no step failed`, `cancelled`) and that the change wasn't
  judged. Its release counts the jobs' re-runs afresh from their
  attempts at the release, as a repairs hold's counts repairs afresh.
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
  dependency does (jig's `domain/tasks/lifecycle-holds.md`, 1), as before.
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
  hold: jig's `domain/tasks/lifecycle-holds.md`, 2) does the same, though
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
  the reset repair allowance, as it does when held because its red
  stands after a repair that changed nothing (`repairs.md`, 1, *A repair
  that changes nothing is checked once more*); held on a previous
  build's flake or no-test verdict, it asks none and names its regate
  instead (`repairs.md`, 1, *The previous build's holds*). The
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

A close job ends one of three ways, as the close check's verdict does
(`../../../protocol/tooling.md`, 4.2; `../../../draft/whose-red.md`, 4),
since 2026-11-25:

- **Settled:** it finished with `settled`. A head pushed opens the pull
  request, and nothing staged ends the change with no change.
- **Refused:** it finished with the close check's refusal, taken as one
  only where its job asked for it, its `head` is the commit the job's
  checkout started at, the goal's branch's tip the change read as it
  made the job, and nothing was pushed. The change holds at once at
  producing, as on a producer failed for good, spending no try of the
  job's, and escalates to the goal at once. Its words, cut to the hold's
  bytes at a character's end: `the close check refused the goal's
  branch at G, against main at B with counts C: F1, F2 and N more; it
  checks again once the goal's branch or main moves.`, G the head's
  first 12 hex digits, B the refusal's base's or `no base`, C its
  counts' or `none`, each F a failure's
  name as the refusal lists it, `and N more` present where the refusal
  or the bytes left some out (N the refusal's `more` and those cut).
  The change keeps the refusal's counts beside the tips it read (below).
- **Broke:** the close check didn't judge the goal. A job that fails for
  good, its command having exited otherwise, past its deadline, or
  with a result missing or past its bound, has been tried within the
  run's class by jig; and a job that finished with any other result (one
  neither `settled` nor this job's refusal, a refusal its job didn't ask
  for among them) is the command's failure at once, since a finished
  job has no tries left and the same command gives the same result.
  Either way the change holds at producing, its words `the close check
  broke and didn't judge the goal: R; its release runs it again.`, R the
  job's failure words, or `it finished with ` and the result's first 64
  bytes.

A close job that fails for good ends with that failure (2026-11-24,
replacing the held job of 11-12). Its change holds at producing on it,
as on a producer failed for good, carrying its words, and as it takes
that hold cancels the job, its reason `its change holds on its
failure`, and names no delegate after it; the job's cancel neither ends
the hold nor replaces its words. So one task is held and escalates, the
change, and only its release closes again. A change held before this
build with its close job still held cancels that job as its release is
taken, before it decides.

The change released decides afresh at producing, and a close change
makes a close job only for what is new to check:

- **While its closer waits on its goal's new work** (`../branches.md`,
  2, *New work waits for, and reopens, the steps*: a repair or a
  catch-up the goal made after its close step), it makes no job and
  waits, with no hold, until its closer wakes it once that work has
  ended, and then decides afresh. A release meanwhile is taken, never
  refused, and nothing is held or escalated for it.
- **A refusal where nothing it judged has moved:** the goal's branch's
  tip and the repository's landing branch's tip, each read afresh
  before it decides, against the two it read as it made that job, its
  request being its own and fixed. It makes no job and holds again at
  once on that refusal, its words `not measured again: the goal's
  branch is at G and main at M, as when its close check refused; it
  checks again once either moves.`, a blank line, then the refusal's
  words, cut to the hold's bytes; G and M are the two commits' first 12
  hex digits. Main's tip stands for main's published counts, which
  follow its landings: the engine reads no counts of its own, and keeps
  the refusal's for its words. This departs from
  `../../../draft/whose-red.md`, 4.2, which compares the counts too
  (decided by the goal's requester, 2026-11-25): a forge read of
  `refs/temper/counts` would take a new call across skein's forge kit
  for the one case of counts published while both tips stand, and the
  counts keep one entry per main commit in their window, so an entry
  once published doesn't change. The gap that remains, a close checking
  before its base's entry is published, is not judged: the check breaks
  rather than refuses (`tools/pre-pr/design.md`, 4.1), so no refusal is
  held on counts that weren't there yet.
- **Otherwise,** a close that broke among them whether or not anything
  moved, it makes a fresh close job, in the change's checkout as
  on its first attempt: the change's repository, its own branch
  writable, made at the goal's branch's tip as it read it, fetched by
  that commit, the job's spec asking for a typed refusal
  (`../../../protocol/tooling.md`, 4.2; `../../worker/jobs.md`, 2). It
  keeps the two commits it read, for the next release to compare. The
  job's end is the change's producer's result, as above.

A close change never asks an agent producer: a head on its branch at
producing, which only a close job's success pushes, opens its pull
request as that job's push does.

The two commits a close change read as it made its last close job are
stored with its row, as `ChangeHot`'s `close_at`, a `CloseAt` of the
goal branch's and the landing branch's commits, a field of the store's
registered family that defaults to none, so no version moves. A row
stored before reads as none, and its next release makes a fresh job.
Beside the two commits, `CloseAt` keeps `asked`, a `bool` defaulting to
`false`, true where the job made with them asked for a typed refusal,
and `refused`, an `option CloseRefused` defaulting to none, set when
that job's end was a refusal: `CloseRefused { counts: option fixed 32 }`,
the refusal's `counts`, none where it named none. A broke end clears
`refused`. Both fields are added with defaults, so no version moves: a
`CloseAt` stored before reads as not asked and not refused, so a
refusal its job made by exit 3 is the command's failure, and its
release makes a fresh job, which asks.

*Why* (2026-11-25, `../../../draft/whose-red.md`, 4): goals 15519 and
15520's close checks refused on test failures and each ran three more
times in full before holding, about 23 and 8 minutes of a worker, the
goal hearing that much later, since any exit but 0 failed the job. A
refusal is a verdict the same inputs give again, so it finishes the job
and holds at once; a broken check judged nothing, so its job is tried
within its class and its release runs it again. The head, not the tree,
checks a refusal against its attempt: the engine never sees the tree
settlement makes. Turned down: a new job kind for the ask, which every
project's policy would have to grant before any close could run
(`../../../protocol/wires.md`, 8).

*Why* (2026-11-24): goals on the lean template spent 28% of their agent
time closing, seven of them 16 to 52 close jobs each, every one
`superseded by its change's fresh close job`. Each goal released its
held close change as soon as it had made the repair, as its charter
says; the release made a fresh job at once, at a goal's branch the
repair had not reached, which failed as the last had and woke the goal,
which released again, until the repair landed. Each failure reached the
goal twice, the job's escalation and its change's, and a job released
ran again with no wait to hold it. The job's own replay
(`tools/pre-pr/design.md`, 4.1) is each worker's and keeps no test
failure, so each fresh job measured again. Turned down: replaying in
the job alone (each attempt still a job, a checkout and a worker's
minutes), refusing a release while the work runs (the release the
charter asks for would be an error, waking the goal), and keeping both
tasks releasable (two escalations for one failure, and a release no
wait holds).

A close job reserves none of its change's spend: it runs no model and
makes no children, so a failed one leaves its change's budget whole for
a fresh one (2026-11-12). A merge job's reservation is unchanged.

## 2. Open questions

None.
