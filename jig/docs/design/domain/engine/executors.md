# Agent executors

## 1. From a task to a run

When an agent task is due (../tasks/lifecycle.md, 2), the core does as below. A
kept task is never due (../tasks/lifecycle-endings.md, 1), and one kept while it prepares
has its claim refused, its preparation abandoned with no try spent; it
is due again once its dependencies are done.

1. **Checks** that the run may start, and with what budget
   (../authority-checks.md, 1.3), takes its task's LLM choice: its `later` when the
   task's reasons hold no `first` and its charter doesn't resume, its first
   choice otherwise (../tasks/README.md, section 3, and ../tasks/messages.md, section 6); takes its models where
   its task was given none, the first it can use its main (../accounts-uses.md,
   1.3), and chooses the
   accounts that pay for it: one
   for its main model's LLM provider, kept by its task, and one for each
   other LLM provider its sub-agents' models use, each with its first
   source usable on a host with room (../accounts-uses.md, 1.3).
2. **Prepares it,** the task preparing over as many steps as it takes:
   takes its reasons, `first` and the kinds that woke it, which the tasks
   child gives with the activation (../tasks/messages.md, 6), and chooses its
   charter's parts those reasons meet, at the revision its preparation
   began under (`settings.md`, 3), which make its instructions (step 4) and the kinds
   it is offered (a message or an entry that would add to those reasons,
   arriving before the claim, prepares it again: ../tasks/messages.md, 6); gathers its brief (`briefs.md`, section 1), for why it runs; has each
   connector that owns a resource the task
   names gather its workspace items, and name the application's
   credentials they need (../connectors.md, section 10); and loads its
   transcript's saved window when it has one, for resuming or for the
   tail of a fresh brief after its window closed (2). An activation
   that needs this load stays due while the root is busy; one with no
   saved window can prepare its brief then. Either way it is given the calls
   committed after its task's last committed turn: a resumed run as a
   note after the transcript, a fresh one as a section of its brief.
3. **Commits the claim:** the task claimed, with its new attempt number,
   the revision its preparation began under (`settings.md`, 3), the host kind it is placed on, the accounts and sources its check
   chose, the task's kept account among them (../accounts-uses.md, 1.3), the
   endpoint its main model runs on, which names its backend (2,
   *Resumed only where it ran*), the
   writer slots of the holds its workspace writes (../tasks/holds.md, 3), its
   reasons and the kinds it is offered (../tasks/messages.md, 6), and the messages
   its start carries, those it is offered (../tasks/messages.md, 2).
4. **Places it** through the fleet, once the claim is durable, on a host
   its charter allows, where its sources are usable and the host rule
   allows it (`fleet.md`), with an assignment the root puts together
   (../root.md, section 7):
   - its run and attempt, the core's names for them, and its
     conversation's first attempt, from which the engine's protocol layer
     derives the run's identity, for the LLM provider's prompt cache,
     the same for every attempt of a conversation and never shared by
     two (`../../protocol/channel/README.md`, section 6). No domain holds it;
   - its workspace: the connectors' items, and a workstream key, the
     number of the task that holds the resources written, so that the
     runs of one piece of work find their workspace cached;
   - its start's messages, those it is offered, named, oldest first; the
     rest it is offered are relayed as the run reads (4);
   - its **ask** for live content (`../hosts/README.md`, section 2): `text` set when
     its task is flagged conversation (../tasks/README.md, section 3), and `input`
     set by the same rule; `reasoning` is false. The core sets it with the
     run's prepared parts (step 2), from which the root puts the assignment together;
     the root copies it, kind by kind, and no host decides it. A task a
     conversation delegates is not flagged, so its run is not asked
     (`../hosts/README.md`, section 10);
   - its charter, in the core's terms: the task's instructions, its own
     or else its charter's followed by each part its reasons meet, in the
     charter's order, each after a blank line (`\n\n`; ../tasks/messages.md, 6); the
     brief, the tools of each host family its charter names and its
     authority holds (the charter's `tools`, a `ToolFamilies` mask, the
     same type as an authority's: a family neither names is not offered,
     so a run's host part spends nothing on it), the outcome its result
     contract asks for; its budget for the run: turns, spend and time,
     with a reserve of turns and time for finishing, its spend reserved
     from what its task has left in the claim's commit (../authority-budgets.md,
     section 1); and its models, from its task's LLM choice (../tasks/README.md,
     section 3): the main model with its effort, and those its
     sub-agents may use, each with its effort, its window (the input
     tokens a conversation's window may hold), its output (the most
     output tokens a completion may have) and its prices (../authority-budgets.md,
     section 1).
     The engine's protocol layer translates it into smith's (`protocol/charter.md`,
     section 2);
   - its transcript's current window, when it resumes, and the calls
     committed since its last turn;
   - its credential grants: for each account its check chose, its
     source's (../accounts-sources.md, 1), a value's current generation with how
     it is sent, or a program source's name, which has its agent run the
     program backend (smith's `protocol/programs.md`, section 3); and
     the application's credentials its connectors named (`README.md`, 6.1).
     Each names its form; the form selects the table the protocol layer
     fills its value from (`../../protocol/channel/README.md`, section 7).

The check's allowance carries its checked spend, retained models, a concrete
channel and kind, and one provider/account/source binding per retained provider.
Main and same-provider sub-agents share their binding. A provider with no usable
serving account contributes no optional models; a usable optional provider's
retained models constrain the same host as main. Fleet preserves affinity and
free-slot preference among those hosts. An undeclared unused alternative does
not block the run; a declared model with inadequate bounds still refuses hello.

Preparation facts are read again after asynchronous work and before the claim.
The successful task claim keeps main's account in the same commit as the core's
complete bindings. A changed selection abandons preparation without a retry
timer. Start and queued placement retain the selected channel; loss of room
returns the claim for a fresh check, releasing its reservation without a failed
try. Source failures are authenticated against that claim, and their ordinary
terminal charges actual spend once and releases writers. A recovered refresh
cannot turn an unrelated later failure into an account wait.

Restoration refuses duplicate provider bindings and a surviving source whose
account or provider disagrees. Removed rows remain legitimate history. Adoption
offers grants only from the recorded surviving sources, never replacements;
an unavailable recorded grant can renew on its original source, and an absent
worker settles through the fleet's grace (`fleet.md`). A new assignment rechecks the
recorded bindings and current coverage.

The check joins each candidate host's kept program logins to the accounts' issued
names and their original holders. Every bound account must satisfy that host's
rule, including retained providers. Deployment-held logins restrict no run;
a deployment with at most one person is unaffected. A program source is chosen
only where its login is usable; the engine never takes one. A program source's
run is granted its source's name and runs on smith's program backend, for the
program its endpoint names (`../../protocol/configuration-engine.md`, section 1).

## 2. Transcripts

Moved to [`executors-transcripts.md`](executors-transcripts.md), section 1.

## 3. The engine's tools

Moved to [`executors-tools.md`](executors-tools.md), section 1.

## 4. Answers, waiting and parking

- **A run answers once:** finished, with its result; parked; or failed,
  with a typed failure. The core commits it, once it has every turn the
  answer counts, and acknowledges it; a finished task closes, a parked
  one follows the wake rules (../tasks/lifecycle.md, 2), a failed one retries or is
  held (../tasks/lifecycle-holds.md, 1). What the
  answer says the run left on resources goes to their connectors
  (../connectors.md, section 10).
- **Finished done while kept, it parks.** A run whose task is kept
  (../tasks/lifecycle-endings.md, 1) and finishes done, with a result its contract takes,
  is committed as parked: its turns first, then its saved state, kept,
  with no try spent, and a line in its task's activity naming the
  dependencies it waits for. Woken once they are done, its next run goes
  on from what it left, its brief or its start saying what it waited
  for (`briefs.md`, section 1), and finishes again. Kept by its own question, a run
  that finishes with a failure parks the same way (../tasks/lifecycle-endings.md, 1). The tasks child decides it, as it
  decides every reason a task may not go on, so the run and its host
  know nothing of it.
- **Finished while it stands, it parks.** A run of a standing task
  that finishes, with any result, is committed as parked, its result
  discarded and its words a line in its activity
  (`../tasks/standing.md`, section 2).
- **Finished while its delegates are live, it parks.** A run that
  finishes, its finish not asking to cancel them, while its task has live
  delegates (../tasks/lifecycle.md, 7), is committed as parked, as a kept one is: its
  turns first, then its saved state, with no try spent and its finish not
  taken. Its run has ended by the time its finish is judged, on a program
  backend and a direct one alike, so a refusal naming the delegates would
  reach no one, and failing the attempt would cost a whole run and tell
  the next nothing. The task waits for its delegates, whose results wake
  it (../tasks/messages.md, 3), and a later run finishes once they are done. The
  finish put off leaves no trace for the next run: telling it would need
  a value stored for that run, and the delegate's result that wakes it,
  with its delegates' state in its brief (`briefs.md`, section 1), says what it needs
  (the plan `wake-faults`'s answer of 2026-11-08; ../tasks/README.md, section 8). The tasks child decides
  it.
- **Waiting holds the slot; parking frees it.** A run that calls `wait`
  keeps its host's slot while it waits, so a run that is talked with
  answers at once. Idle
  past its charter's threshold, it parks: every turn it took is committed
  first, and it ends. A coordinator's threshold is zero: it parks as soon
  as it waits.
- **Woken again,** a parked task is due. Its charter says whether the
  next run resumes the transcript or starts fresh from a brief, and a
  standing task's resumes only within its session (`../tasks/standing.md`,
  section 5); by
  default a charter for conversations resumes and a coordinator's starts
  fresh, its brief
  carrying its goal, its plan's state and its notes, and its start the
  messages that woke it (1).
- **Messages to a live run,** those it is offered (../tasks/messages.md, 6), are
  relayed as the run reads, named, in order, never more relayed and
  unread than the run's inbox takes; a turn that reads one takes it (2;
  ../tasks/messages.md, 2).
- **Not offered, a message stays** in the task's inbox, never relayed to
  the run, and is judged again when the run ends, waking the task if its
  kind does (../tasks/messages.md, 2 and 6).
- **An entry left undecided** when a run ends, a proposal or an
  escalation still waiting for the task's decision, wakes the task again,
  its reasons that entry's kind, unless the run was offered its kind: then
  it waits for a change, and is offered at the next wake (../tasks/messages.md, 6).
- **The first activation ends** when a run parks idle or finishes: the
  task's next runs hold no `first` among their reasons and take its
  `later` (../tasks/messages.md, 6). A run parked kept, and a failed attempt, leave it
  as it was.
- **Refused or unread, a message stays** in the task's inbox, untaken,
  and wakes the task again when the run ends (../hosts/README.md, section 7;
  ../tasks/messages.md, 2 and 3). A turn's commit never takes a message the run
  refused, whatever its fence covers (2).
- **A cancel** ends the run: the host cancels it and answers; what it
  committed before stays committed.
- **Nothing left to await.** After normal wakes, tasks asks the core to
  number the always-waking `NothingAwaited` notice for a result-ended
  task that parks idle with nothing pending (../tasks/lifecycle.md, 2). Tasks spends
  a run try on each such park and ends failed when those tries run out.
- **Saved state.** A run that parks, fails or is cancelled with unfinished
  work has it saved by its host, where its connectors say, and the next
  run of its task starts from it (../connectors.md, section 10).

## 5. Jobs

A job task (../model.md, 3.2; ../hosts/hub.md, 9; ../tasks/lifecycle.md, 3.1) becomes a
hosted run with no agent:

- **Its check** needs no LLM account, no charter, no model and no
  program's login: its kind among those the application declares, its
  items within the workspace's bounds, its spec and deadline within the
  tasks child's bounds, and its requester's authority over what its
  items write.
- **Its kinds are the application's.** It declares each to the core
  as it makes it (`jig-domain-core`'s `Config`), with the most bytes a
  spec and a result of it may hold and its longest deadline. A batch
  naming a job of a kind not declared, or a spec or deadline past its
  kind's, is refused; the kind's result bound is the job's `Job`
  contract. A job's executor is named by its kind in a delegation's
  kinds (../authority.md, section 3), and no party creates one
  (`../../client/domain/contract/documents.md`, 7).
- **Its claim** goes to a host whose hello declared its kind, whose
  bounds for that kind (its spec's and result's bytes, its deadline)
  cover the job's, and that has a free slot, as a run's does (`fleet.md`
  ), fenced by its attempt. Nothing of a run's placement applies: no
  charter's hosts, no model, no account's source, no login. With no such
  host it is not claimed: it stays due, its wait naming the host and
  what it lacks (`fleet.md`, 2, *A job is claimed only where a host can
  take it now*).
- **Its answer** commits its result as a delegate's result (../tasks/lifecycle.md,
  7), which its procedure reads at its next step; a failure is tried
  again within its tries, then holds it, its procedure hearing why.
- **Nothing of it is a transcript:** its spec and result are its
  record; a restart finds a claimed job's attempt as a run's (`restart.md`
  ) and settles it lost or answered.

The application's declaration of job kinds is ascending by number, each
once, as the worker's hello is. A batch naming an undeclared job kind is
refused as `Executor`; a job's spec or deadline beyond its declared bound is
refused as `Spec` (2026-11-04).

A job's transient preparation uses the infrastructure retry class
(../tasks/lifecycle-holds.md, 1), and its failed delivery the transient one;
permanent preparation, stale or refused delivery, and an invalid
assignment use the permanent class. Its own failure and deadline use the
run's-own class; failure to start uses the agent's class; an oversized
result uses invalid; lost contact uses lost. A cancel for its host's
shutdown, which the hub answers only before the job's delivery began
(../hosts/hub.md, 9), ends the attempt suspended, as a run's does,
spending no try and with no backoff (6). Engine cancellation
ends it as cancelled. Busy before starting is a refusal, spending no try. A
finished job becomes its task's Job result, without asking to cancel
delegates (2026-11-04). A failed job's nonempty detail accompanies its
class as a bounded `FailedWithReason` (../tasks/lifecycle-holds.md, 1); an invalid start's
reason does too. The engine retains it through retries into the hold.

## 6. Suspended attempts

An attempt its worker's stop cuts is **suspended**, neither failed nor
refused: no try, no backoff, no clock over the gap and no blame, and the
task's next attempt goes back to that worker while it returns within a
window (temper's `draft/archive/worker-restarts.md`, sections 4, 5 and 7). The
attempt ends; its task goes on, due, or idle where it is kept
(../tasks/lifecycle-endings.md, 1).

### 6.1 What suspends an attempt

- **A run or a job its worker answers `Cancelled(Shutdown)`**
  (../hosts/hub.md, 7 and 9): the worker's drain or stop cut it. Its
  cause is `Stopped`.
- **A claim that its worker, back as a new process, does not list**
  (6.5): the process is gone, and the run with it. Its cause is
  `Restarted`. A task whose previous attempt ended suspended
  `Restarted` ends this one as lost instead (`fleet.md`, *Losing
  contact*), spending a try of the lost class
  (../tasks/lifecycle-holds.md, 1), so that a run that brings its worker down
  is not tried for ever. Any other end of an attempt clears that mark.
- **Nothing else.** A run its worker cancels for lost contact,
  `Cancelled(Contact)`, ends interrupted as before
  (../tasks/lifecycle.md, 2), and so do the engine's own slots at its
  stop (`restart.md`, 5); a claim no host reports after a restart, with
  nothing committed, counts as not started.
- **The worker** is the one whose channel answered, or whose hello
  omitted the claim: the fleet names it to the core with the answer
  (`Request::Answered`'s `worker`) or the end (`Request::Suspended`'s).
  An answer with no worker, from an engine host, never suspends: its
  `Cancelled(Shutdown)` ends `End::Recheck`, as before. The fleet
  names a worker by its position, the tasks child, the core's documents
  and the client by its setting number, the position plus one
  (`workers.md`, 2).

```rust
// jig-domain-tasks
pub enum SuspendWhy { Stopped, Restarted }
pub struct Suspended {
    pub worker: u32,      // its setting number
    pub why: SuspendWhy,
    pub window: Duration, // the window's length at the end (6.4)
}
pub enum End { /* as before */ Suspended(Suspended) }

// jig-domain-fleet
pub enum Request {
    /* as before */
    Answered { to: ReplyTo, run: Token, attempt: Token, answer: Answer, payload: Token, worker: Option<u32> }, // worker: the answering channel's position, none on an engine host
}
```

### 6.2 What it records

- **As interrupted, with no try:** the tasks child releases the attempt
  with no try spent and no backoff, keeping the turns and calls it
  committed; the next attempt resumes from its transcript and from the
  saved work its answer named (2), and a job's next runs it from the
  start, as `End::Recheck` does.
- **Its end reads `suspended`,** never `refused`, in the activity's
  `RunEnded` line and in the attempt's row (../tasks/lifecycle.md,
  12.1). The tasks family takes it as added parts, its version unmoved
  (`../../protocol/store/layouts.md`, 2 and 6):

  ```
  enum SuspendWhy { stopped, restarted }
  record SuspendedEnd { worker: u32 why: SuspendWhy }
  enum EndKind { parked, finished, cancelled, failed: FailedEnd, refused, suspended: SuspendedEnd }
  record Suspension { worker: u32 since: u64 until: u64 }
  # TaskRun, after open_attempt:
  #   suspension: option Suspension = none
  #   suspended: u64 = 0
  #   restarted: bool = false
  ```

- **The task keeps its suspension:** its worker; `since`, the end's
  commit on the engine's wall clock, in nanoseconds; and `until`,
  `since` plus the end's `window` (6.4). It stands until the task next
  leaves due and preparing: its next claim committed, or the task held,
  idle or ended. Then its age, that phase change's time less `since`,
  is added to `suspended`, the nanoseconds the task has spent suspended,
  and it clears. A task has one at most, since a suspended task has no
  attempt left to cut.
- **`restarted`** marks a task whose latest attempt ended suspended
  `Restarted` (6.1): set by that end, cleared by any other.
- **Shown as suspended.** The client's wire carries the end as
  `RunEnd`'s `suspended` and the wait (6.4) as `TaskRunWait`'s
  `worker`, from version 40:

  ```
  enum SuspendWhy { stopped, restarted }
  record RunEndSuspended { worker: u32 why: SuspendWhy }
  enum RunEnd { parked, finished, cancelled, failed: RunEndFailed, refused, suspended: RunEndSuspended }
  record TaskWaitWorker { worker: u32 until: u64 }
  enum TaskRunWait { /* as before */ worker: TaskWaitWorker }
  ```

  A view names the worker as it names an untooled wait's
  (`../../client/domain/contract/documents.md`, 1): `Run 4 suspended:
  worker box-2 stopped`, `Run 4 suspended: worker box-2 restarted
  without it`, and while the task waits, `Waiting for worker box-2 to
  return, until 17:40`. An application's log line for the answer says
  suspended, never failed: temper's engine writes `task 812 run 812
  attempt 3 suspended: its worker stopped`.

### 6.3 No clock counts the gap

- **The gap** is a suspension's age (6.2): from the suspended end until
  the task leaves due and preparing.
- **The task's deadline** (../authority-budgets.md, 1) is its authority's,
  moved later by `suspended` and by its open suspension's age. The
  core's checks of a claim and of a job against the task's own deadline
  (../authority-checks.md, 1.3) and the tasks child's alarm at it
  (../tasks/lifecycle-holds.md, 1) read that moved deadline, which
  `tasks::Domain::deadline(task, wall) -> Option<Wall>` gives, reading
  the open suspension's age at the caller's wall, the same wall the
  check or alarm is made at. Its authority is not
  rewritten: an ancestor's deadline and a project's or deployment's
  ceiling move with nothing of this task's. So a task whose deadline is
  12:00, suspended from 11:30 until its claim at 11:50, is held for its
  deadline from 12:20 on.
- **A stall's timer** counts nothing while its task is live, and a
  suspended task is due, so live: temper's change stall restarts while
  its producer is live (temper's `domain/forge/change/stalls.md`,
  section 1), so a change whose producer waits for its worker is not
  held stalled, however long the window.
- **A run's own clocks start with the run:** its no-progress watchdog and
  wall bound (../hosts/README.md, 5.1), its budget's time (smith's
  `domain/budget.md`, section 1) and each check's bound (smith's
  `domain/run.md`, the checks' bound) are each run's from its start.
  The next attempt has them whole, and a check the stop cut runs again
  with its whole bound.

### 6.4 Back to its worker

- **The window** runs from the suspension's `since` for the restart
  wait, `fleet.restart_wait` (`fleet.md`, *A restart said holds its
  runs*), the wait the engine announces to its workers (temper's
  `draft/archive/redeploys.md`, 5.3): twenty minutes in jig's profiles. The
  core gives it with the end, as its `window`, from the fleet's limits
  at that moment. A restart wait of zero gives no window.
- **Within it, the claim goes to that worker alone.** The claim's check
  (1, step 1) takes only that worker's channel, and so does the queued
  assignment's recheck (`fleet.md`). Where that worker has no channel
  that may take the run (none connected, or its channel paused, draining,
  mismatched, refusing the task's versions, or full), the task waits
  with `RunWait::Worker { worker, until }`, shown as
  `TaskRunWait::Worker` (6.2), in place of the host wait it would
  otherwise have; an account's or a credential's wait on that worker is
  said as ever.
- **At its end, the claim goes anywhere,** from the saved-work branch
  (2), as before. The tasks child activates a due task again at its
  suspension's `until`, whatever else has changed, and with the window
  past the claim's check takes every host.
- **What the return gains** is the worker's warm store, its
  workstream's cache (`fleet.md`, the workstream affinity) and, on
  temper's worker, the cut run's workspace, kept for it with its build
  outputs and its checks' results (temper's `domain/worker/checkouts.md`,
  *A workspace kept for its run*), and with the parts kept since its
  last turn, the cut turn itself (`executors-transcripts.md`, 1).

```rust
// jig-domain-core
pub enum RunWait { /* as before */ Worker { worker: u32, until: Wall } } // worker: its setting number
// jig-domain-core-vocabulary
pub enum TaskRunWait { /* as before */ Worker { worker: u32, until: Wall } }
```

### 6.5 A worker back as a new process

- **Its hello says so.** The hello's `fresh` is true on each hello of a
  worker's process until the engine has admitted one, at the first
  record the engine sends after it, `sources`
  (`../../protocol/channel/README.md`, 4.3, and `../../protocol/channel/worker-link.md`, 1.1), and false on every
  later hello of that process. It has a default, so a worker whose
  layout lacks it is read `fresh: false` (`../../protocol/channel/layouts.md`,
  2.2).

  ```
  record Hello { slots: u32 stop_bound: duration workstreams: list 65536 u64 hosting: list 65536 Hosting jobs: list 256 JobBound bounds: Bounds fresh: bool = false }
  ```

- **At its admitted hello,** each claim the fleet holds for that worker
  that the hello does not list ends at once with `Request::Suspended`:
  one adrift from that worker's lost channel, and one restored at the
  engine's start whose proof names that worker, adopted or crossed,
  whether or not a restart said holds it (`fleet.md`). The core ends it
  `End::Suspended` with the cause `Restarted` (6.1). Its window is open
  and its worker back, so its next attempt goes there at once. A claim
  the hello lists goes on as before.
- **A claim's worker survives the engine's restart** in its proof:
  `RunProof.worker`, the setting number of the worker its claim was
  placed on, set with its `placement`, none on an engine host; the core
  family takes it as an added field, `worker: option u32 = none`, and
  passes it to the fleet with the restored claim. A restored job's
  claim names no worker, and keeps its grace.
- **Without `fresh`,** from a reconnect of the
  same process, the claims the hello does not list keep their grace, or
  their hold, as before.
- **Two processes under one worker's name** are a fault of the
  deployment's that the engine cannot see: the older's channel is
  replaced (`../../protocol/channel/README.md`, 4.2), and the newer's
  `fresh` is believed.

```rust
// jig-domain-link
pub struct Hello { /* as before */ pub fresh: bool }
// jig-domain-fleet
pub enum Event {
    /* as before */
    Adopt { /* as before */ worker: Option<u32> }, // its position, from the proof
    Cross { /* as before */ worker: Option<u32> },
}
pub enum Request {
    /* as before */
    /// To the parent, terminal for `Start`, `Adopt`, `Cross` and `StartJob`:
    /// the attempt's worker came back as a new process that does not list it.
    Suspended { to: ReplyTo, run: Token, attempt: Token, worker: u32 }, // its position
}
```

### 6.6 No blame for a stop

- **A run's agent that exits, faults or says its run was cancelled while
  its hub drains or shuts down** answers `Cancelled(Shutdown)`
  (../hosts/hub.md, 7), so its attempt suspends: the stop's own signal
  may reach the agent before the hub's cancel does (temper's
  `draft/archive/worker-restarts.md`, 2). A run that said in its turn how it
  finished keeps its ending, as before.
- **A job that fails while its hub drains or shuts down, before its
  delivery began,** answers `Cancelled(Shutdown)` too (../hosts/hub.md,
  9). One whose delivery began answers as its delivery ends, as before.
- **So nothing a stop cut is repaired or blamed:** a suspended attempt
  spends no try, adds to no failure count, holds nothing and tells no
  one it failed. An application's change whose producer is suspended
  sees its producer live (6.3) and asks no repair, as a CI runner's
  stop asks none (temper's `domain/landing-proof.md`, 2.4); a check the stop
  cut writes no results (temper's `domain/forge/reds.md`, 5.6), and the
  next attempt runs it again.

### 6.7 Open questions

- **Lost contact and the engine's own slots** end interrupted, `refused`
  in the activity, not suspended (6.1): whether they should suspend too.
  Default: no, as neither is a worker's stop.
- **A suspended task's delegates** are bounded by its authority's
  deadline, unmoved (6.3), while its own runs are bounded by the moved
  one. Default: unchanged.
- **A window of its own,** apart from the restart wait (6.4). Default:
  none, one figure serving both.
- **An agent's answer that reaches its hub before the hub hears its own
  stop** is not yet the stop's, and ends as the run's own failure
  (6.6). Default: it stays so until an agent with a live host leaves its
  stop to its host (temper's `draft/archive/worker-restarts.md`, 2).
