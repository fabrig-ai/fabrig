# A goal's close made once: contracts

2026-11-24. The shapes the changes of the goal building
[`change/repairs-releases.md`](change/repairs-releases.md), section 1's
close job and [`branches.md`](branches.md), section 2's held close step
share, and no other area reads (`../../draft/design-economy.md`, 4.3).
They bind as those documents do. The goal's last change deletes this
document once the code holds them.

## 1. The close step's change, as its closer reads it

The closer's route (`temper-engine-domain`, `closer_route.rs`, `child`)
reads its running close step's change as running no close job, the
closer policy's `Child::Idle { held: bool }`, in two states, whichever
change made them:

- **Held on its job's failure,** `held: true`: the change's state
  `State::Held { was: Producing { requested: true }, why: Hold::Failed }`
  with its close request set (`ChangeRow.close`), whatever its delegate.
  Until the single release path (3) lands, it still names its held close
  job; after, it names none. Cancelling the change cancels that job with
  it, as its descendant.
- **Waiting for its closer's work,** `held: false`: the change's state
  `State::Producing { requested: false }`, its close request set, no live
  delegate (`ChangeRow.delegate` none, or naming a task that has ended),
  and its task `tasks::Phase::Active(Idle)`. Only the wait of
  `change/repairs-releases.md`, section 1 leaves a close change so.

Every other state of a live close change reads `Child::Pending`, as now.

## 2. The closer's wake

The closer wakes a close change waiting for its work with
`Work::Tasks(tasks::Event::WakeProcedure { task })`, `task` its running
close step's change, at each of its steps where its goal's new work has
all ended (`waiting_on` empty) and none of the goal's changes landed into
its branch since its close step. The change, woken, reads its closer's
`waiting_on` again before it decides: a wake is no leave to make a job.
The change reads its closer as its requester (`tasks::Party::Task`), with
`domain.forge.closer(requester)` naming a row, and its pending work as
that closer's task's `waiting_on`; a close change whose requester is no
closer never waits so.

## 3. The held close change, as its release reads it

Once a close job's failure for good holds its change
(`change/repairs-releases.md`, 1), the change names no delegate
(`ChangeRow.delegate` none), its `delegate_status` stays `Failed`, and its
`producer_failure` holds the job's words, through its release too: a
close change stays exempt from `retire_failed_delegate`'s clearing. The
job is cancelled with the reason `its change holds on its failure`. A
held change that still names a live close job, one held before this
build, cancels it as its release is taken, before it decides. The release
decision reads the failure from these fields alone.
