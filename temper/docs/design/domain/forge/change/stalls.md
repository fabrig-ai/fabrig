# Stalls

Every wait of a change on the forge is bounded: past its bound the
change is held, and the hold says where it waited and why. This
document is the stall's: what it counts and what it does not (1), how
checking counts CI by what its runners do (2), what the hold says (3,
4) and how it holds (5). A repair's, a resolution's and a release's
rules are `repairs.md`'s; the landing turn a hold gives up is
`../queues.md`'s.

## 1. What a stall bounds

Each wait on the forge (CI that never reports, a mergeability never
known, an update or a merge whose answer or head never shows, a
forge's merge queue that neither lands nor ejects it, a gate's task
stuck) is bounded by a stall, the deployment's
`forge.change_policy.stall_ms` (`../../../protocol/configuration/limits.md`),
counted from the head's push, the write it waits on or the last
release; past it, the change is held. No step of a change's way waits
past it with no hold. At checking, the stall counts CI's running, not
its wait for a runner, and that wait has its own bound (section 2). The
default, 30 minutes, outlasts an ordinary CI run so a healthy change
is never held just for waiting, yet tells the person of a check that
never reports within the same sitting.

- **A wait for access is not one:** while its forge is not connected,
  or its repository not reachable (`../adoption.md`, 1, 3), the stall
  does not count.
- **Nor is a wait for its agent:** while the change's producer, repair,
  resolution or merge job is a live task, running, between its tries or
  waiting for a host, the stall does not count. An agent working for an
  hour is working, and jig bounds what it does (its tries, its
  executor's stall, its budget: jig's `domain/tasks/lifecycle.md`, 6).
  The stall counts from that delegate's end, as it does while a
  question of that agent's is open (`questions.md`, 1), from the step
  that relays its last answer down.
- **Nor is a wait for its dependencies:** while the change is at
  producing and its task has a dependency that has not ended (jig's
  `domain/tasks/lifecycle.md`, 8), the stall does not count, whatever
  the change waits on there: a producer not yet started (`questions.md`,
  1, *Early starts*), an early producer that ended giving nothing, a
  read of the forge. It counts from the last dependency's end, as it
  counts from a delegate's end, and a step taken while the wait lasts
  counts it afresh, as a live agent's does. At any other step a
  dependency changes nothing: a change done while kept is checked,
  gated and held there as any change is.
- **A gate's task is a wait on the forge** as this section names it,
  and stays bounded.
- **A head behind its base is no wait:** a range that lists a commit,
  or one too large to list, says the head lacks the base's tip
  (`../keeping-up.md`, 3), so the change takes its base, or in conflict
  goes to its merge job and then a resolution, however many commits
  behind it is.
- **What a step reads is waited on too** (2026-11-13): the reads a
  change makes to take its step (its branch, its base, its pull
  request, its CI, the ancestry of its head and its base's tip) are a
  wait on the forge as any is. A read with no answer, or one the forge
  client keeps refusing for its own capacity, counts the stall from
  its first read, and past it the change is held at that step, its
  words that step's (section 3). That count is the reads' own
  (2026-11-14): kept while the reads start over on news or are retried
  for capacity, counted afresh from a release or from an exemption's
  end, and gone once the reads decide. It moves nothing the step's own
  wait counts from, so the step's waits keep their rules: a conflict's
  settling, an approval's wait, CI's alarm, checking's execution clock
  (section 2).
- **An exemption stands only while its cause does:** its agent a live
  task, that agent's question open, a dependency not ended, its forge
  not connected or its repository out of reach. Once the cause ends,
  the stall counts from that end, and no record left of a cause that
  has ended (a question whose asker ended, a writer slot whose writer
  ended) keeps it from counting.

## 2. Checking counts CI's running

(2026-11-16, `../../../draft/ci-runner-wait.md`, 1.) A change at
checking whose CI at its head is pending or unknown may be waiting for
a runner, not for its CI: on one shared runner a run can wait longer
than the stall before it starts. Commit statuses say only `pending`,
so checking reads what the head's runs are doing and counts the stall
by an **execution clock**: the time CI runs, not the time it waits for
a runner.

### 2.1 A reading

Each step of a change at checking whose CI at its head is pending or
unknown reads the head's checks afresh (skein's `Read::Checks`, which
links a pending check to its job's run, job and attempt where the
forge says), then each run its pending checks link (skein's
`Read::Run`, one read a run, at most 8 runs a step), and folds them
into one **reading**:

- **running,** where a job a pending check links is running;
- else **waiting,** where every pending check links a job that its run
  lists and says is waiting for a runner or blocked by the jobs it
  needs (a run's read says both as waiting);
- else **unknown:** no pending check at all (none reported yet, or a
  required context missing), a pending check that links no job (CI
  another system reports, a forge whose checks name no job), a run not
  read (its read failed, or past the 8), a linked job its run does not
  list, or a linked job completed while its check is still pending.

A reading also names the newest run and the highest job attempt it
read, 0 where it read none. While CI stays pending or unknown, the
change steps again within the forge client's `poll_max` (30 seconds on
the stock; skein's `forge/client.md`), so readings are at most one poll
apart. That is the "one poll" below. These are reads of the periodic
path, one checks read and a run read a linked run each poll; the
job-state reads that explain a hold stay the hold's alone (section 4).

### 2.2 The clock

The clock keeps three times from its readings: running, waiting and
unknown. Each interval between two readings counts to what the earlier
reading saw. So:

- **It starts at the first observed running state,** the first reading
  that saw a job running, and repeated readings never restart it.
  Counting from the first observed running state undercounts by up to
  one poll, the interval before that reading counting as waiting; it
  counts up to one poll past a stretch's end, to the reading that sees
  no job running.
- **With several jobs, it runs while any of them is running:** a
  reading that sees one linked job running and another waiting is
  running; one that sees the first completed and the other waiting is
  waiting, and the clock stops until a job runs again.
- **It starts afresh** where the stall does: the clock belongs to the
  head and the change's `since` it began under, so a new head, a
  release, a re-run temper writes, or an exemption's end (section 1),
  each counting the stall afresh, start it afresh. So does a reading
  that names a newer run or a higher attempt than the clock has seen: a
  new attempt or a re-run, by temper or by anyone.
- **It is kept across a restart,** on the change's record
  (`Change.clock`), and the first reading after counts the interval the
  engine was down to what the clock last saw.

### 2.3 Its bounds, and which applies when

After each reading, the clock as it then stands decides:

- **Running:** once its running time reaches the stall
  (`forge.change_policy.stall_ms`), the change is held (section 3).
- **Unknown, where the forge gives no execution state:** the first
  `forge.change_policy.unknown_grace_ms` of unknown time (30 minutes on
  the stock) counts nothing; past it, unknown time counts toward the
  stall with running time. A change whose CI never says what it does is
  held at the grace and the stall after its push: the stall as today,
  after a grace.
- **Waiting or blocked for a runner** counts nothing toward the stall.
  It has a bound of its own, `forge.change_policy.runner_wait_ms` (4
  hours on the stock), past which the change is held, saying it waits
  for a runner (section 3), so a job whose labels no runner has does
  not keep the change unheld, and its landing turn, for ever.

The stall holds once running time and unknown time past the grace,
together, reach it; the runner wait holds once waiting time reaches it.
Between readings the change waits until the earlier of the next poll
and the instant the clock would reach a bound if what the last reading
saw went on. A settling window (`../keeping-up.md`, 2) still waits
first, as it does. A step at checking that read no execution (CI read
at another head than its branch's) waits as section 1 counts, from
`since`.

### 2.4 Its turn

A change waiting for a runner is waiting, not held: it keeps the
landing turn it owns (`../queues.md`, *The first owns the queue*), as
it keeps it through CI on its head. A bound past holds it, and the
hold passes the turn on, as every hold does.

### 2.5 The contract

`temper-engine-domain-forge-change`'s boundary owns the clock, and a
landing's gate counts the same one (`../../landing.md`, 7.1):

```rust
/// What a reading saw of the pending CI at a head (2.1).
pub enum Execution { Unknown, Waiting, Running }

/// One reading: what it saw, the newest run and the highest job attempt it read (0 for none).
pub struct Reading { pub execution: Execution, pub run: u64, pub attempt: u32 }

/// The execution clock (2.2): the wait it counts (`since`, `head`), the newest run and
/// attempt seen, the three times counted up to its last reading `at`, and what that saw.
pub struct Clock {
    pub since: Wall,
    pub head: Commit,
    pub run: u64,
    pub attempt: u32,
    pub ran: Duration,
    pub waited: Duration,
    pub unknown: Duration,
    pub at: Wall,
    pub seen: Execution,
}

impl Clock {
    /// The clock after a reading at `now`: afresh where `clock` is `None`, its `since` or
    /// `head` differ, or the reading names a newer run or a higher attempt; otherwise the
    /// interval since `at` counted to `seen`, then `at = now` and `seen` the reading's.
    pub fn observe(clock: Option<Clock>, since: Wall, head: Commit, reading: Reading, now: Wall) -> Clock;
    /// Each time as it stands at `now`, the interval since `at` counted to `seen`.
    pub fn ran(&self, now: Wall) -> Duration;
    pub fn waited(&self, now: Wall) -> Duration;
    pub fn unknown(&self, now: Wall) -> Duration;
}
```

The change policy takes the reading as `Facts.execution:
Option<Reading>` (`None` where the top read none this step) and keeps
the clock as `Change.clock: Option<Clock>`; its `Limits` gain
`unknown_grace` and `runner_wait`. The store keeps the clock as an
added field of the registered `temper-store` family, `none` by
default, so no version moves (`AGENTS.md`, 2).

## 3. A stall's hold says where

Its words name the step the change was held at and what it waits on,
commits at the forge's short length, in one shape: `no progress at
STEP for N minutes: waiting for WHAT`, N the stall in whole minutes,
or at checking the bound that held it. For instance:

| Held at | WHAT |
|---|---|
| opening | `its pull request to open` |
| checking | `CI at 0123456789ab` |
| checking, past the runner wait (2.3) | `a runner for CI at 0123456789ab`, N the runner wait's minutes |
| gating | `its gates at 0123456789ab` |
| first | `the forge to settle whether 0123456789ab merges` |
| updating | `its update of 0123456789ab with main at 89abcdef0123` |
| landing | `its merge of 0123456789ab` |

A step not listed says its own wait in the same shape. A change held
on its step's own reads (section 1, *What a step reads is waited on
too*) says that step's WHAT where the table gives one, and otherwise
`the forge to answer its reads` (2026-11-14): `no progress at producing
for 30 minutes: waiting for the forge to answer its reads`.

## 4. A stall on CI says why

(`../../../draft/ci-on-forgejo-17.md`, section 3.) Before a change is
held because CI at its head never reported, or ran or waited past its
bounds (section 2), it reads that head's checks afresh (skein's
`Read::Checks`) and the state of each pending check's job (skein's
`forge/job-state.md`), one read per job, only then: the periodic path
reads runs for the clock (2.1), never a job's state. The hold's words
follow the stall's, one clause per pending check, in the checks'
order, after `; `: `CONTEXT: waiting for a runner labelled L1, L2` (no
runner with its labels took it, `labelled` and its list left out when
the job names none), `CONTEXT: blocked by the jobs it needs`,
`CONTEXT: running at step N, NAME` (`running` alone when no step says
so), `CONTEXT: pending, no job linked` (a check whose status names no
job: CI another system reports, or a forge whose pending checks name
none), `CONTEXT: pending, its job unread` (the job's read failed or
answered `MissingJob`), and `CONTEXT: completed` (a job ended since the
stall's last read). At most 8 clauses, then `; and K more pending`;
clauses kept whole within the hold's bytes, those that do not fit
counted in K. For instance, `no progress at checking for 31 minutes:
waiting for CI at 0123456789ab; ci / test: waiting for a runner
labelled docker; ci / lint: blocked by the jobs it needs`. A checks
read that fails holds with the stall's words alone. The clauses live
in the hold's words alone, never in the store: a change held again
after its restoration (`repairs.md`, *Every hold reaches its task*)
holds with the stall's words alone.

## 5. A stall holds as any hold does

The change and its task are held together, never one without the
other: its escalation waits for its holder, which it wakes (jig's
`domain/tasks/lifecycle.md`, 6 and
`jig/docs/design/domain/tasks/proposals.md`): the goal above it where
that goal may decide, otherwise its person or the role the project's
policy names. A decision to hold reaches the task whatever step the
procedure is at. A held change gives up the landing turn it owns
(`../queues.md`), and its release, its result arriving on the hold and
its dependency's end are `repairs.md`'s.

## 6. Open questions

- **A gate's live run past the stall:** a review's task stays a wait
  the stall bounds, though a review working 30 minutes is as normal as
  a producer. Its verdict is kept if it arrives on the hold, and a
  release goes on from it. Default: as it stands, its wait being named
  by section 1; the request `parallel-faults` of 2026-11-06 met no such
  hold.
- **A dependency amended away** (section 1): the stall counts from a
  dependency's end, and an amendment that removes the last pending one
  ends the wait with no end. The stall then counts from the last step
  taken while the change waited, so a change at producing whose
  producer gave nothing, with no step between, can be held at the step
  the amendment brings. Default: so; its requester amended it, and
  releases it.
- **An authoritative start time** (`../../../draft/ci-runner-wait.md`,
  3): the clock counts from readings a poll apart, since skein's reads
  give no job's start time. Reading one would need skein's reads
  extended. Default: the clock, within a poll.
- **CI a gate waits on** (2): under the forge's queue, a change gating
  with CI pending at its head counts its stall from `since`, as section
  1 does, not by the clock. Default: so; the draft names checking.
