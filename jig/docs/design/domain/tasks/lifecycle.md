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
  (../connectors-procedures.md, 1.1).
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
     (../connectors-effects.md, 1.2);
  4. its connectors release the resources it held, as effects of their
     own, after all its others (`holds.md`, 5).

  Then it ends, and its result, or why it failed or was cancelled, goes
  to its requester. So a task that depends on another never starts
  before what that task did is visible and settled.

  What the core knows of a closing task is what it waits for: which of
  its delegates are still closing, and which connectors it waits on to
  settle its effects, by number. It never knows their outbox entries,
  which are each connector's, kept by the connector and settled by it
  (../connectors-effects.md, 1.2; ../model.md, section 5).
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
(../engine/restart.md, 5): its attempt ends interrupted, as a recheck ends, with
no try spent and no backoff, keeping the turns and calls it committed,
and the task is due again. Nor is a run its host cancelled for lost
contact (../hosts/hub.md, 6), whose answer ends its attempt interrupted
in the same way, with the saved work it names kept for the next
attempt. Nor is an attempt its worker's stop cut: a run or a job its
worker answers `Cancelled(Shutdown)`, a job only before its delivery
began (../hosts/hub.md, 7 and 9), or a claim its worker, back as a new
process, no longer lists. Its attempt is **suspended**
(../engine/executors.md, 6): released as an interrupted one is, its
end read `suspended`, no clock counting the gap, and its next attempt
sent back to that worker within a window; a job's next runs it from the
start. A kept
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
wakes. Such a task is stuck, with no wake at all (`stuck.md`, 4): past
its run tries it is held, as a task past its tries is (6), for
`Failures` of the run's class, its hold's words `Its runs parked with
nothing left to await, past its N run tries.`, N the run tries its
class allows, cut as a hold's reason is (`REASON_BYTES`). It is not
failed: whoever its hold reaches decides what follows (`stuck.md`, 5). A task ended by its requester may remain idle (9), and so may a standing one (`standing.md`, section 2). A task ended by its closer is
not due by this rule: where it would be, jig makes its closer instead,
and numbers no notice (11).

**A question left open wakes it again.** After the ordinary
unread-message wake rules, an agent task whose run parks idle while a
question asked of it is still open (`messages.md`, 2) is due again at
once, `question` among its reasons (`messages.md`, 6), when all of
these hold:

- its wake policy lets a question wake it (`messages.md`, 3);
- its runs start from a brief, never resuming (../engine/executors-transcripts.md,
  1), since a resumed run's window already holds the question and a wake
  would bring it nothing new;
- it is no conversation, does not stand (`standing.md`), and is not
  ended by its requester (9): each waits on its person or its session;
- it is not kept (8).

A question is read in a run, never settled by it: its delivery leaves
the inbox with the run's committed read (`messages.md`, 2), and this
rule is what brings it back until it is answered or ends. Its next
run's brief shows each open question (../engine/briefs.md, 1.2). No
notice is numbered: the open question is the reason. Whether it wakes
the task is whether the task is stuck (`stuck.md`, 4), judged in the
decision that parks it:

- **A stuck task,** every wake of which is dead or which has none, is
  woken at each such park. Each consumes a run try, retained across
  these wakes as Nothing left to await's are. Past its run tries the
  task is held, as a task past its tries is (6), for `Failures` of the
  run's class, its hold's words naming the oldest open question and the
  chain that leaves it stuck, by task and question, cut to the 128
  bytes a hold's reason keeps (`REASON_BYTES`; `stuck.md`, 5), so that
  whoever the hold reaches sees what waits. A task held so is not
  failed: its delegates go on, and a release counts its tries again from
  it.
- **A task with a live wake,** a timer, an entry waiting at a party, or
  a task that can still move without its answer (`stuck.md`, 2 and 3),
  stays idle, spending no try, until that wake comes. It is judged again
  when a task its walk visited changes (`stuck.md`, 6), so a watch that
  turns dead wakes it then. A goal that means to answer once a change
  lands costs no run until then: a watch that can still bring the answer
  leaves the task alone (`temper/docs/design/draft/bounded-waits.md`,
  5.2).

**Words left unrecorded wake it once.** After the ordinary
unread-message wake rules and the question rule above, a task that keeps
words (`messages.md`, 2) whose run parks idle, and that those rules
left idle, is due again at once, `words` among its reasons
(`messages.md`, 6), when it keeps an unrecorded words message newer than
its `words_woken` that its wake policy lets wake it (`messages.md`, 3: a
person's words always, a task's by the policy's kind for words).
`words_woken` is the newest unrecorded message that has woken it so, 0
for none, never lowered; the wake sets it to the newest such message's
number. It spends no run try and numbers no notice: the unrecorded
words are the reason, and the next run's brief shows them
(../engine/briefs.md, 1.3). A park leaving only words at or below
`words_woken` leaves the task idle, spending nothing, until its next
event. So words a run read and left unrecorded cost one more run, never
a poll, and a task that rules below would close or tell that nothing
more will come is woken for its words first: this rule comes before
Nothing left to await and before a closer is made (11), which a later
park reaches.

## 3. Procedure tasks

An active procedure task is stepped by the part of the engine that owns
its procedure: a connector (../connectors-procedures.md, section 1), or this child
domain for the core's (`README.md`, section 5). The tasks child hears a step's
decisions (effects asked, tasks made, amended or cancelled, proposals,
messages up, the result) as one decision, committed with the
procedure's state.
It steps when it is first active, when a message its policy lets through
arrives, and whenever what it reads changes, which its owner knows
(level-triggered). A changed fact while a step is pending is coalesced
into one further step after a wait; the pending step cannot consume that
wake. The owner receives the step's acknowledgment before that further
activation, so it records the decision and its made tasks first. This
marker is transient: a restart reoffers a due step against fresh facts. Kept, it steps as ever and reads which of its
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

Moved to [`lifecycle-holds.md`](lifecycle-holds.md), section 1.

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
- **A failure always says why.** A failed task's reason, and so its
  summary's (`../../protocol/store/retention.md`, 3), is never empty. An
  executor's declared failure with no reason ends with `Its EXECUTOR
  declared failure without a reason.`, EXECUTOR `run` for an agent's
  run, `person` for a person's answer and `procedure` for a procedure's
  step. Each failed ending jig makes itself carries
  its words, N the run tries its class allows (6):
  - past its tries owing its requester a question (8): `Its runs
    finished without asking its requester the question it owes, past
    its N run tries.`;
  - past its tries with its closer failing (11): `Its closer failed,
    past its N run tries: REASON`, REASON the closer's own failure's,
    the colon and REASON left out where that is empty.

  A task past its tries parked with nothing left to await is held, not
  failed (2), its hold's words those of Nothing left to await.
  Any other ending jig makes failed with no reason of its own takes the
  nearest of these, never none. Each is cut at a character's edge to
  the bound every failed reason has, `tasks.result_bytes`, as a reason
  past it would be: a deployment whose bound is 0 keeps no reason at
  all, its own choice, and one whose bound is smaller than the words
  keeps their beginning (the standard's holds them whole).

## 8. Kept by a dependency

Moved to [`lifecycle-endings.md`](lifecycle-endings.md), section 1.

## 9. Ended by its requester

Moved to [`lifecycle-endings.md`](lifecycle-endings.md), section 2.

## 10. Placed holds: owners, modes and scopes

Moved to [`lifecycle-holds.md`](lifecycle-holds.md), section 2.

## 11. Ended by its closer

Moved to [`lifecycle-endings.md`](lifecycle-endings.md), section 3.

## 12. What is kept of it

A task's phase changes are not kept one by one. What people read of a
lifecycle is per attempt and per hold: when a run was claimed, when it
started, when and how it ended, and the holds between (temper's
`draft/store-shape.md`, section 6). A wake that moves a task through
idle, due, preparing, claimed, running and backing off writes nothing
of its own beyond the task's record. The tasks child keeps, beside the
task's record (`README.md`, section 3):

- **Its count of phase changes,** `milestone` in its record: one more
  at each phase change the tasks child publishes, its first phase and
  its end included. A phase change's **position** is the count it
  brought, from 1. No row is written per phase change.
- **Its attempts** (12.1): a row per attempt.
- **Its holds** (12.2): a row per held phase.
- **Its activity's phase lines,** as before: a line naming the
  position at its first phase, at each held phase (whose hold row is at
  that position) and at its end. The lines of the first phase and the
  end name no row.

Times are the engine's wall clock at the commit that made the change,
in nanoseconds.

### 12.1 Attempts

An attempt is a claim of a run and what follows until that run ends,
named by its attempt number (2). A phase **names** attempt N when it is
`Active::Claimed { N }` or `Active::Running { N }`, a hold whose `was`
is one of those, or closing at stage `Run(N)`.

```
# the tasks family (jig-store-tasks); a row at 01 01 16 <task> <attempt>
record AttemptRow { task: u64 attempt: u64 claimed: option u64 started: option u64 ended: option u64 end: option EndKind phases: option u32 }
# kept in the live task's record, TaskRun.open_attempt: option OpenAttempt
record OpenAttempt { attempt: u64 claimed: option u64 started: option u64 from: u64 end: option EndKind }
```

- **Opened** at the first phase change after which the phase names it:
  the task's record keeps it as `open_attempt`, `claimed` that change's
  time, `started` that time too where the phase is running, `from` its
  position; and its row is put, with no end.
- **Started** at the first phase change after which the phase is
  `Running { N }`: `started` takes its time, and the row is put again.
- **Its run's end:** where the tasks child accepts its run's end and
  writes the activity's `RunEnded` line, `open_attempt.end` keeps the
  end's kind: `suspended`, with its worker and cause, for an attempt its
  worker's stop cut (../engine/executors.md, 6.2).
- **Closed** at the first phase change after which the phase no longer
  names it, the task's end among them: its row is put whole, `ended`
  that change's time, `end` its run's end where one was heard, `phases`
  that change's position less `from`; and `open_attempt` clears. An
  attempt costs three puts of one row.
- **Its sources:** an agent task's attempt records what its run was
  told from, `sources`, from its opening on, so that what a run read can
  be read back while its row is kept (temper's
  `temper/docs/design/domain/agent/project-charters.md`, 6):
  - `build`: the application's build name, at most 64 bytes, as the
    application gives it at the core's construction and the store's
    floor keeps it (`../../protocol/store/opening.md`);
  - `settings`: the settings revision its run is prepared under
    (`../engine/settings.md`, 3);
  - `charters` and `source`: the project's set its task records, its
    revision and its `source`, or 0 and empty
    (`../engine/project-charters.md`, 4);
  - `guides`: none until its host reports them, then each guide its
    agent read, at most one per workspace directory: the repository's
    name, the commit its checkout held, the guide's path, each at most
    256 bytes, the SHA-256 of the bytes read, their count, and whether
    the read was whole. A report is taken once, for the attempt still
    open; another, or one for an attempt closed, is dropped. The host
    sends its report just before the attempt's answer
    (`../hosts/hub.md`, 2, step 9), so the report steps first, then the
    end. An attempt
    whose host never reported keeps none, which reads as unreported,
    never as no guide.
  The row carries them as an optional field, its default none, so a row
  written before them reads with none.
- **A task restored** whose phase names an attempt its record keeps no
  `open_attempt` for takes `open_attempt { attempt, claimed: none,
  started: none, from: 0, end: none }`; at its close its row is put
  whole with what it knows, `claimed`, `started` and `phases` none.

### 12.2 Holds

```
# the tasks family; a row at 01 01 17 <task> <position>
record HoldRow { task: u64 position: u64 held: Held at: u64 }
```

- **A row per held phase:** each phase change after which the task is
  held puts a row at its position, with the held phase whole (where it
  was, why, its source) and the change's time: a hold made, its reason
  or source changed, or its `was` moved, as a run's end heard while
  held moves it. The row is never written again: its release is the
  task's history (`Released`, `changes.md`), and the next phase change
  says where the task went.
- **The task's `hold`** is the position of its latest held phase, as
  before, so the row at its `hold` is the task's latest hold. The hold's
  revision (`Revisions.hold`), which escalations and releases name
  (6, `proposals.md`), is unchanged.

### 12.3 Who reads them

- **Escalations:** an escalation's decision is read with the hold row
  at its task's `hold` (`../../protocol/store/loads.md`, 3).
- **History pages:** a task's history page holds its changes, then its
  attempts, then its holds, each in key order; a subtree's, each task's
  in turn, by the index of its delegates (`../../protocol/store/loads.md`,
  3). The client's wire carries them as a history page's rows
  (`../../client/domain/contract/reads.md`, 1).
- **Goals' projections** are fed holds and ends alone
  (`../connectors.md`, 8): each hold row as it is saved, as the lifecycle
  identity `{ task, position }` with its held phase; and a task's end,
  as its ended record is saved, as `{ task, position }` its record's
  `milestone`, with its ended phase. Attempts are not fed. The
  identities are stable phase-change positions, so what a projection
  has already written of a task is not written again.
- **A task's erasure** erases its attempts and holds with its other
  rows.

### 12.4 Stores at the floor

No store this build opens holds milestone rows. The tasks family's
`Row` has no milestone variant, no value type declares one, and the
range `[01 01 0a, 01 01 0b)` lies outside the family's keys. Its value
type's number is never given again (`../../protocol/store/layouts.md`,
2). Attempts and holds are the records of 12.1 and 12.2; projections
use 12.3's identities.

No job converts milestones at open. A store without the floor, below
it or behind the build's levels is refused, naming `FLOOR_BRIDGE`
(`../../protocol/store/opening.md`, 1.8); this build upgrades none of it.

## 13. Failed by its application

An application may end a live task failed, as a cancel ends one
cancelled, where a rule of its own says the task's time is up:
temper's repair goal past its deadline (temper's
`domain/red-repairs.md`, 4.3). A deadline of jig's holds a task
(`lifecycle-holds.md`, section 1); this ends it.

- **Asked** by `tasks::Event::Fail { reply_to, by: Party, operation:
  Fail }`, `Fail { task: u64, reason: Box<[u8]> }`, the reason within
  the limits' failure reason bytes, cut as a failure's is. Only the
  application may ask it, as `Party::Deployment` of the task's project
  or an ancestor task of it; a person's controls and an agent's tools
  have no such act.
- **What it does:** what a cancel does to the task's tree (its live
  attempt stopped, its live delegates and its closer cancelled with the
  reason), then the task ends `Failed { reason }`, its result reaching
  its requester as any failure's, and its effects settling as any
  ending's.
- **Refused** for a task that has ended or is closing, naming it, as a
  cancel of one is; nothing is written.
