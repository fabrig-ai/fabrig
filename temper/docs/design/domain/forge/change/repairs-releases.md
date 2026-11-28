# A change's holds and releases

This document holds a change's holds and releases, split from [`repairs.md`](repairs.md): what holds a change, for each step (1), the one record a hold is (2), the one function that releases it (3), the judgements a release with nothing changed holds again and the act that measures them afresh (4), and how a hold is stored and read from the previous build (5).

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
  again with its hold's cause and the words its record keeps (2), unless a party released the
  task: that release's step lifts the change as ever (A release, below).
- **A result that arrives on a held change is kept.** A delegate ending
  while its change is held on a stall at that delegate's wait ends the
  hold, by the engine's release (3) and no party's, as a green head does (above): the change
  goes on from the state it was held at, its stall counted from the result,
  the deployment releases its task (jig's `domain/tasks/changes.md`),
  and the result is taken as any is: a head and a title open the pull
  request, a no-change ends the change. Under any other hold, the result
  stays on the change and a release takes it, as the hold's own words say;
  a no-change under one is said by the hold, since nothing else would.
- **A dependency's end ends a stall at producing.** A change held on a
  stall at producing with no producer asked for, whose task's last
  pending dependency ends done, is released by the engine's release (3)
  and no party's, as
  a result ends a stall at its delegate's wait (above): the change goes
  on from producing, its stall counted from that end, the deployment
  releases its task, and its producer starts as at any producing. This
  clears a hold placed during a dependency's wait before the stall
  stopped counting there (`stalls.md`, 1). A dependency that ends
  failed or cancelled ends no hold: it holds the change as any failed
  dependency does (jig's `domain/tasks/lifecycle-holds.md`, 1), as before.
  A hold of any other cause, or a stall at producing with its producer
  asked for, stays for a party's release.
- **A release** lifts what it releases, by the one function of 3: its
  cause's allowance alone counts again from it (2), whoever released it,
  the deployment ending a hold at a green head among them, its stall is
  counted from it, and the change decides afresh. **Deciding afresh reads afresh** (2026-11-13): the change
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
     result that arrives on a held change*, above). Then it settles what
     is owed and forgets what is stale, in that order (3, steps 1 and 2).
  3. **Read afresh.** It reads its branch, its base and its pull
     request afresh (`../keeping-up.md`, 3), through the record of
     temper's own writes (`../writes.md`, 4).
  4. **Decided** on those facts alone, at the earliest step they
     support (3, step 4), by the rules for the step it
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
  the repairs it has taken, since a failure resets no allowance (2).
  Held because its red stands after a repair that changed nothing, its
  release follows 4; held on a previous build's flake or no-test verdict,
  read as a red that stands with its one check owed (5), it asks none and
  names its regate instead (`repairs.md`, 1, *The previous build's holds*). The
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
  against the resolutions taken, a failure resetting none (2), and held by the rule above
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

A close job ends one of four ways, as the close check's verdict does
(`../../../protocol/tooling.md`, 4.2; `../../../draft/archive/whose-red.md`, 4),
since 2026-11-25, and outdated since 2026-11-27:

- **Settled:** it finished with `settled`. A head pushed opens the pull
  request, and nothing staged ends the change with no change.
- **Refused:** it finished with the close check's refusal, taken as one
  only where its job asked for it, its `head` is the commit the job's
  checkout started at, the goal's branch's tip the change read as it
  made the job, and nothing was pushed. The change holds at once at
  producing, its cause `close_refused` (2), spending no try of the
  job's, and escalates to the goal at once. Its words, cut to the hold's
  bytes at a character's end: `the close check refused the goal's
  branch at G, against main at B with counts C: F1, F2 and N more; it
  checks again once the goal's branch or main moves.`, G the head's
  first 12 hex digits, B the refusal's base's or `no base`, C its
  counts' or `none`, each F a failure's
  name as the refusal lists it, `and N more` present where the refusal
  or the bytes left some out (N the refusal's `more` and those cut).
  The change keeps the refusal's counts beside the tips it read (below).
  On the fresh refusal it also sends the goal one `Words` message, via
  its closer's relay where the closer requested the change: `the close
  check refused the goal's branch at G; every failure its result names,
  in the check's order: F1, F2, …, F9.` The names are joined by `, `,
  and ` and N more` comes before the period if the result says `more: N`.
  The message fits `tasks.message_bytes`, ending in `...` if cut. The
  hold account remains 512 bytes. Neither a restart nor a release on
  unchanged tips resends the list. The requester decided on 2026-11-27
  that the goal should hear all the failures a result can carry for one
  repair, keeping 512-byte workers eligible and taking up to 8,192 bytes
  from newer ones.
- **Outdated:** it finished with the close check's finding that it could
  not judge the goal against main's published counts: the goal's branch
  holds none of the commits their window covers (`window`), or its
  checkout's ancestry was too incomplete to tell (`ancestry`)
  (`tools/pre-pr/design.md`, 4.1; `../../../protocol/tooling.md`, 4.2).
  It is taken as one only where its job asked for a typed refusal, its
  `head` is the commit the job's checkout started at, and nothing was
  pushed, as a refusal is: every close job this build makes asks for
  both, and one an earlier build made asked for the refusal alone, its
  command never told of the finding. The job finishes, spending no try
  of its class, and the change ends at once with no change, holding and
  escalating nothing, its words `The close check didn't judge the goal's
  branch at G: F; it asks for the goal's catch-up from main.`, G the
  head's first 12 hex digits, F `it holds none of the commits main's
  published counts cover` or `its ancestry was incomplete`. The change
  keeps the finding beside the tips it read (below), and its closer,
  reading its end, catches the goal up from main and closes again, once
  (`../branches-closer.md`, 1, *A close found outdated*). A close run by a
  worker that can't carry the ask (`../../../protocol/wires.md`, 8) has
  its command not told, and breaks as before.
- **Broke:** the close check didn't judge the goal. A job that fails for
  good, its command having exited otherwise, past its deadline, or
  with a result missing or past its bound, has been tried within the
  run's class by jig; and a job that finished with any other result (one
  neither `settled`, this job's refusal nor its outdated finding, a
  refusal or a finding its job didn't ask for among them) is the
  command's failure at once, since a finished job has no tries left and
  the same command gives the same result.
  Either way the change holds at producing, its cause `failed`, its words `the close check
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

- **While its closer waits on its goal's new work** (`../branches-closer.md`,
  1, *New work waits for, and reopens, the steps*: a repair or a
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
  checks again once either moves.`, as a judgement released with nothing
  changed holds again (4), a blank line, then the refusal's
  words, cut to the hold's bytes; G and M are the two commits' first 12
  hex digits. Main's tip stands for main's published counts, which
  follow its landings: the engine reads no counts of its own, and keeps
  the refusal's for its words. This departs from
  `../../../draft/archive/whose-red.md`, 4.2, which compares the counts too
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

The two commits reach the root on the change's own decision, the one
that makes its close job, and come back on that job's admission, as any
delegate's admission comes back, for its row to keep (2026-11-27): the
forge's change decision and its delegate's admission each carry them as
an optional member, none for any other delegate and none where the
decision read no pair. Neither is stored or sent on a wire: both live
within the engine's own steps. Turned down: a close decision and a close
admission of their own, each the change's with the commits added, two
names for one thing.

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
release makes a fresh job, which asks. Beside them, `outdated`, an
`option CloseOutdated` defaulting to none, set when that job's end was
outdated: `enum CloseOutdated { window, ancestry }`, the finding's
`why`. An outdated end clears `refused`, and a refused or broke end
clears `outdated`. It is added with its default, so no version moves,
and a `CloseAt` stored before reads as not outdated.

*Why* (2026-11-25, `../../../draft/archive/whose-red.md`, 4): goals 15519 and
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

*Why* (2026-11-27, `../../../draft/test-budgets.md`, 3): a goal that
holds none of main's last twenty landings can't be judged against main's
published counts, and its close check broke at once asking for an update
from main, as the draft says it should be asked once. Nothing read the
ask: the job was tried again in its class, broke the same way, and held
for a person, as two goals on the reference deployment did at 15:19Z
and 15:23Z that day. A typed finding lets the closer make the update the
check asks for. Turned down: reading the check's words, which a
project's command chooses and no contract fixes; a refusal, which
judges the goal's own failures and holds until its branch or main moves,
which here only the catch-up would do; holding for the goal to catch
up, which is the person's wait again, one level lower; and an ask a
worker of an earlier build can't keep, whose closes would wait for a
newer worker rather than run as before.

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

## 2. One hold, with a cause

A held change keeps one record (`../../../draft/archive/whose-red.md`, 6.1), written in the decision that holds it:

- **the step it held at,** its state as it held, kept whole beneath the hold, which its release re-enters from (3);
- **its cause,** one of fifteen, each of one of four kinds (below);
- **its words,** its account (1, *Every hold reaches its task*), made in the decision that holds from the facts that hold it, within jig's `HOLD_ACCOUNT_BYTES`, and stored with it, so that a change found held again holds its task with the same words, byte for byte, never with words made again from what its row still keeps. The root makes them, in one place for every cause, a judgement held again among them (4): the change's step and the forge decide the hold and keep the facts it is made from (its cause, a judgement's inputs, the account a red, a refusal or a failed delegate leaves on the row, a read's evidence), and write no words of their own;
- **its since,** the instant of the decision that holds it.

Its **way out** follows from its cause, as the table says, and its words name it; every hold's way out includes a cancel. The task's hold carries the words as its account, and its revision is the one a release names (jig's `domain/tasks/lifecycle-holds.md`, 3).

| Kind | Cause | What holds it | Way out |
|---|---|---|---|
| A bound spent | `repairs`, `resolutions`, `updates` | another would pass that allowance (1, *Bounded apart*) | a release, which resets that allowance alone |
| | `runner` | CI's runner's faults past `reruns` at its head (1, *Bounded apart*) | a release, its re-runs counted afresh; or a green read at its head |
| | `questions` | another question would pass its bound (`questions.md`, 1) | a release, which resets none: questions are never reset |
| | `authority` | its next delegate beyond its authority (1, *A step its authority no longer covers*) | an amendment giving what it lacks, then a release; a bare release holds again |
| | `stalled` | a wait past its bound (`stalls.md`) | a release, its wait counted afresh; or, at a delegate's wait or at producing, that delegate's result or its last dependency's end (1) |
| The forge disagrees | `branch_moved`, `branch_missing` | its branch moved by no write of temper's, or gone (`drift.md`, 1; `../writes.md`, 4) | a release, which takes the branch as it reads it, or recreates it |
| | `pull_closed`, `retargeted` | its pull closed, or retargeted, by another (`drift.md`, 1) | a release, which reopens or retargets it |
| A judgement | `rejected` | a person's rejection of a required gate (`gates.md`, 1) | the person answering their gate again, then a release; held again while the rejection stands (4) |
| | `close_refused` | its close check refused the goal's branch (1, *A close job ends*) | a release once the goal's branch or main has moved, or a fresh measurement (4) |
| | `red_stands` | its red stands after a repair that changed nothing (`repairs.md`, 1) | an amendment, then a release, asking a fresh repair; a release once its head has moved; or a fresh measurement (4) |
| A failure | `failed` | a delegate or an effect failed for good (1), a close check that broke among them | a release, which retires what failed and asks afresh |

- **Held once.** A held change takes no step that writes, and a cause found while it is held changes nothing in its record: what that cause would hold for is found again from fresh facts after the release. A change ending while held keeps its cause as its ending's (`../branches-closer.md`, 1).
- **Projected:** the panel names the step and the cause (`../../../client/forge/records.md`, 4). The client's and the public wire read `runner`, `red_stands` and `close_refused` as `failed`, their account saying which (`../../../protocol/wires.md`, 9).

*Why* (`../../../draft/archive/whose-red.md`, 6): fourteen hold arms each grew a release of its own, and blocked changes were cancelled and cut again by hand. A cause of a kind keeps the differences in data (what a release resets, whether it judges), not in code paths. The words are kept, not made again, because what made them (a read's evidence, a delegate's failure) is gone by a later step, and words rebuilt from what survived could say less. They are made in one place, the root's, which reads every fact they name (a read's evidence, the row's accounts, the bounds), since words composed in the change's step, the forge and the root apart said the same judgement three ways (2026-11-27). One record, not a stack: a release reads every fact afresh, so a second cause found while held is found again, and a stack only made one release lift one cause to show the next. Turned down: storing the way out, which the cause gives; and the kind as a type each cause nests in, which every reader would match twice.

## 3. One release

One function releases a held change, whatever its cause and whoever releases it: a party releasing its task, a fresh measurement (4), or the engine. It is taken once per hold revision and fenced against reads asked before it (1, *In order*, steps 1 and 2), and goes in this order, whatever restart falls between two of its steps:

1. **Settle what is owed, and keep it.** Before anything is forgotten: its landing's `Landed` ask and its other asks (`../../landing-members.md`, 3.3), and a merge whose outcome is unknown, read back first (`../../landing-turn.md`, 1); an uncertain write, read back through the record (`../writes.md`, 3); a delegate's durable result kept on the held change (1, *A result that arrives on a held change is kept*); and an accepted decision it hasn't taken, an answer to relay among them. Each is taken by its own rules and kept: a release never drops a result, an ask, an answer or a write in flight.
2. **Forget what is stale:** the held step's reads, pins and verdicts (its gather, its pinned head, its recorded gate failures and clean heads, an update's backoff), but for a judgement's verdict and the inputs it judged (4); and the spent allowance of its cause alone (below). Its pull, its identity, its write record, its delegates' records, its questions and its other allowances are kept. A failed delegate is retired (1, *In order*, step 2).
3. **Read afresh** its branch, its base and its pull, through the record of temper's own writes, so a view behind them waits within its caller's bound rather than holding again (`../writes.md`, 4 and 5).
4. **Re-enter at the earliest step the fresh facts support,** from the step it held at (`README.md`, 2): producing with no branch; opening with no pull, reopening a closed one and retargeting a moved one; checking at a head CI hasn't judged, a moved head among them; gating at the same checked head, its asks cleared; the held step itself where its facts stand, a delegate asked afresh after a failure, under the current authority, and a merge that failed for good retired and asked afresh from its turn. A judgement re-enters only as 4 says.

| Cause | Its release resets |
|---|---|
| `repairs`, `resolutions`, `updates` | that count, from the release |
| `runner` | the jobs' re-runs, counted from their attempts at the release (1, *Bounded apart*) |
| `stalled` | its wait, counted from the release (`stalls.md`) |
| `questions`, `authority` | nothing: questions are never reset, and authority comes by an amendment |
| the forge's causes, the judgements, `failed` | nothing |

- **The engine's releases** keep their triggers: a green read at the head ends a CI hold (`repairs.md`, 1, *A hold ends too*), a delegate's result ends a stall at its wait, and a done dependency's end ends a stall at producing (1). Each goes through the same function, in the decision that sees its trigger, taking the hold's revision as a party's release would and resetting its cause's allowance as a party's does, and releases the task as the deployment (jig's `domain/tasks/changes.md`), so the task's release that follows finds that revision taken. No trigger releases a person's rejection or a hold a party placed.
- **Nothing is cut again to unblock it.** A change blocked at any step is moved on by its release, so none is cancelled and made again for that.

*Why* (`../../../draft/archive/whose-red.md`, 6.2): the bugs fixed one at a time (a stale resolution, a lost delivery, a closed predecessor's pull) came from a release that kept an observation the fresh read contradicted. Some facts can't be read again (a producer that ended with no change looks like one that never ran; a landing may still owe its result), so those are settled first and kept, and only what the fresh read replaces is forgotten. A release resets its cause's allowance alone, so a release for a moved branch leaves the repairs taken; the engine's releases reset as a party's do, since a cause the forge's own fact ended (green at the head) starts its next episode afresh.

## 4. Judgements and fresh measurements

- **A judgement released with nothing changed holds again at once** (`../../../draft/archive/whose-red.md`, 6.3), a new hold of the same cause, its words saying it was not measured again and what it judged, a blank line, then the judgement's own account, cut to the hold's bytes: the red's or the refusal's, as the row keeps it, or the rejection's, as its first hold said it; never the words of the hold it follows, so that every release with nothing changed, however many, holds with the same words:
  - `close_refused`: the goal's branch's tip and main's tip, compared as 1, *A refusal where nothing it judged has moved*, says.
  - `red_stands`: the head its repair left and the change's spec revision, both kept with the hold. With its head moved, it re-enters at checking (3); amended since it held, it asks a fresh repair at its head, with the amended spec and its reason, counted against the repairs taken; with neither, and its one check of the same content taken (`repairs.md`, 1), it holds again, its words `not measured again: its red at H stands at the same content and spec; amend this change to ask a fresh repair, measure it again, or cancel it.`, H the head's first 12 hex digits. With its check still owed (none could be made, or a previous build's hold read so: 5), the release takes it, as `repairs.md`, 1 does, and holds at once where it still can't be made.
  - `rejected`: the person's verdict at the change's head, read afresh as gating reads it (`gates.md`, 1). While it stands, the change holds again at once, whoever released it; once its person has answered their gate again, the release goes on. No engine release and no forge read ends it.
- **A hold a party placed** is the party's to lift (jig's `domain/tasks/lifecycle-holds.md`, 2): no engine release ends it, and the change's record doesn't hold it.
- **A fresh measurement** is an act of its own: jig's `Measure` (jig's `domain/people/acts.md`, 1), in ctl `task measure TASK --hold REV` (`../../../client/ctl/tools-acts.md`, 1), and a run's `measure` (`../../agent/declarations.md`, 1). It names the hold's revision, as a release does, and is taken by whoever may release the held task or decide its escalation, the goal for its own changes among them. Accepted, it releases the task as a release does, its due reason `Measured`; the change takes it by the function of 3 and keeps, with the revision it took, that it was a measurement, so a second delivery of the act, or a restart, takes none again. One accepted act takes one measurement:
  - `red_stands`: one check of the same content, as after a repair that changed nothing (`repairs.md`, 1, *One check of the same content*), asked afresh though one was taken, its identity kept: green goes on, red holds again, its words naming the measurement's failures, and where the same content can't be checked it holds at once saying so.
  - `close_refused`: a fresh close job at the goal's branch's tip as it reads it, as for a close that broke (1, *Otherwise*).
  - any other cause: the measurement is the release it carries, since its re-entry reads afresh anyway; a person's rejection still holds again while it stands.
- **`task release --again`** stays jig's, for an effect held uncertain alone; a change refuses it, as a procedure with no run of its own to repeat (1).

*Why* (`../../../draft/archive/whose-red.md`, 6.3): re-measuring the same inputs gives the same verdict where checks replay, as fabrig's do, so a release that ran the check again spent a worker's minutes to say the same thing; a project whose checks can flake has its decider's act instead, revisioned so that one request takes one measurement. A red that stands has two ways on without one: new content (its head moved) or new instructions (an amendment, so a fresh repair isn't the same brief again). A person's rejection is theirs, so only their own answer lifts it. A judgement held again says its own account, not the words it held with, since those carry the line of the release before: a red's account behind three such lines was cut away by the fourth release (2026-11-27). A measurement is a release asking one check more, so whoever may release the task may take it, and so may a person an escalation passed up to, who decides it without the task in their tree.

## 5. Stored, and the previous build's holds

- **Stored** with the change's row (`ChangeHot`, `../../../protocol/store.md`, 2): the step and its cause as its stored state's live step and its one hold (`ChangeState`'s `live` and `holds`, which holds at most one); the since, the words and a held `red_stands`'s inputs beside them, as `ChangeState`'s added `since`, `words` and `judged` (the head and the spec revision), each an option defaulting to none; `ChangeHold`'s added arm `close_refused`; and a measurement taken, as `TaskRelease`'s added `measure: bool = false` beside the revision it took. Each is a registered layout's added field or arm, so no store version moves (`../../../protocol/store-versions.md`, 1); jig's due reason `Measured` is its tasks family's added arm, likewise.
- **The previous build's holds** are read by arm as causes of 2, as its row is read, with nothing released: each arm as its cause of the same name; `failed`, where its row keeps a close refusal (`CloseAt`'s `refused`), as `close_refused`; `flake` and `no_test` as `red_stands` with its one check still owed, so that its release regates once, as before (`repairs.md`, 1, *The previous build's holds*); a stored state holding more than one hold as its outermost, the newest, the others found again from fresh facts after its release. A hold read so has no words and no inputs: its since reads as its step's, its words are made, as any hold's are, from what its row keeps, the next time it holds its task, and stored, and a `red_stands` with no inputs has its check owed. An arm this build would explain, drift the write record now explains among them, still holds once, and its release follows 3.
- **`flake` and `no_test` stay** `ChangeHold`'s readable arms, which no step writes, until the reference deployment's store has run this build (`../../../draft/archive/whose-red.execution.md`, 3).

## 6. Tests

All in the focused suites, but the property, which is the change policy's fuzzy sweep:

- **The record:** each cause holds with its words and since stored and kept across a restart; a change found held again holds its task with the same words; a cause found while held changes nothing; a close's refusal holds `close_refused` and its broken check `failed`; its outdated finding, asked, unpushed and at its job's head, ends it with no change and keeps the finding, across a restart, and one unasked, pushed or at another head is the command's failure.
- **One release:** for each cause, a release from each step it can hold at settles what is owed first (a kept result, a landing's ask, an uncertain write read back), re-enters where 3 says and resets its cause's allowance alone; the engine's three releases go through it, and the task's release that follows takes nothing again.
- **Judgements:** each released with nothing changed holds again at once, saying so; a `red_stands` re-enters at a moved head and asks a fresh repair once amended; a person's rejection survives the forge agreeing and every release while it stands.
- **A measurement:** refused at a moved revision and to a party who may neither release the task nor decide its escalation; accepted, one check of a red that stands and one close job of a refused close, none again across a restart or a second delivery; on any other cause, a release.
- **The previous build:** a store it wrote reads each arm as its cause, `flake` and `no_test` regating once, a refused close's `failed` as `close_refused`, a stack of holds as its outermost, none released.
- **The property** (`../../../draft/archive/whose-red.md`, 10): from any held step and cause, a release followed by the forge agreeing (its branch at its head, its pull open, CI and gates passing, delegates ending with their work) reaches ready or its end within a bounded number of steps without a person, unless a judgement holds it, which holds again until what it judged changes; an `authority` hold's agreeing includes the amendment its way out names (2), and its bare release holds again; and no release resets another cause's allowance.

## 7. Open questions

None.
