# Heavy

Provisional, 2026-10-29. Host admission, containment and recovery for builds
and gates. Only `heavy` is installed from this directory; the landing and
check scripts beside it support isolated tests.

## 1. In one page

- `heavy [--gate | --land] [--tail N] [--] command...` runs a command in
  the build lane, an exclusive gate, or a landing gate. `cargo fmt` runs
  directly. A machine without a recorded reconciler installation runs the
  command directly (section 12).
- The installation is `~/src/fabrig/main/tools/heavy/heavy`. Storage is
  independent of the checkout: the workspace defaults to `~/src/rust`,
  temporary storage to its `tmp/`, and the state directory to `tmp/heavy/`.
- The state directory is the queue. Its canonical path's SHA-256 names
  both the queue and its systemd unit prefix. Moving the executable does
  not create another queue or replace its ledger.
- Admission counts durable task lifetimes. A task occupies its lane until
  its owned cgroup is proved empty. Builds have unlimited slots unless
  `HEAVY_BUILD_SLOTS` sets a limit; a gate lane admits one gate at a time.
- Systemd CPU weights order contended work: gates 10000, builds
  1000, `HEAVY_CLASS=background` 100, `HEAVY_CLASS=idle` idle cores only.
  `HEAVY_BUILD_JOBS` defaults to the machine's CPU count.
- A separate reconciler timer polls every five seconds. This is a recovery
  interval, not a payload timeout. Holds, release and completion evidence
  survive the submitter or wrapper ending.
- State and environment files are private. Events contain safe identities
  and phases, never task commands, environment dumps or payload output.

## 2. Ownership and admission

The Bash entrypoint resolves its own symlink before loading adjacent
`kit.sh` and `heavy-recovery.py`. `DEV_ROOT` overrides the workspace;
otherwise Bash and Python both use `~/src/rust`. `DEV_TMP` overrides
`<workspace>/tmp`. Python canonicalizes both paths, then computes
`BASE = TMP / "heavy"`, `QUEUE = sha256(str(BASE))` and
`PREFIX = "skein-heavy-" + QUEUE[:12]`. All callers of a queue share this
state directory. No path in the fabrig installation participates in that
identity. The helpers require Python 3.11 or newer and its standard library.

The host lock serializes admission, reconciliation and maintenance.
Submission saves a random token, a monotonically increasing task number,
context and a private environment file, then starts
`PREFIX-TOKEN-wrapper.service` in the submitting directory. The transient
service has closed stdin and umask 077, and belongs to the user manager,
so it can outlive the waiting client. Failed wrapper launch releases the
task as refused. A wrapper claims only its unreleased queued token,
recording its PID, process start tick and boot identity. A late wrapper
cannot replay a released token. A queued token with no running wrapper
is recovered with code 137.

Each lane admits older eligible tasks first. Live tasks are launching,
active, disk-retry, finishing or closing; their capacity remains occupied until
release. The build lane admits up to `HEAVY_BUILD_SLOTS` live builds (0
means unlimited), except while an exclusive gate or operator holds it.
The gate lane admits one gate while no other gate is live and the
operator has not held the lane. An exclusive `--gate` records a machine
hold, drains every live build and landing gate, then waits for load to
settle. `--land` keeps the same gate policy and candidate checks but
starts beside builds without taking that hold. Two landing gates never
overlap. Payload units with weight 10000 have `OOMScoreAdjust=0`; other
weights have `OOMScoreAdjust=500`.

`kit.sh` selects integer machine settings from the environment, then the
last valid entry in `DEV_TMP/machine.env`, then the caller's default.
Installed heavy checks the workspace's free disk at submission and inside
the owned payload cgroup immediately before command start, after any lane
wait and settling. The default `PLAN_MIN_DISK_GB` is 40. Ordinary builds
below it are refused; diagnostics name the floor, free space and largest
directories. Cancellation, recovery and plain execution skip that check.
No target-size estimate or extra target reservation is added.

A task labelled `land REPO`, `land REPO prebuild` or `land REPO regenerate`
is a landing step. Every task records its submitted `floor_gb` and
`room_gb`; a task lacking them uses the machine settings. A landing reserves
`PLAN_LAND_ROOM_GB` (default 30 GB) while it is unfinished and for two
minutes after release. Other builds require their own floor plus the
largest current reservation before admission, while their submission and
payload checks remain the floor alone. Builds waiting for reserved room
cannot block a landing that can run on its own floor, even with one build
slot. Submission order still holds among landings.

A landing below its floor queues instead of refusing immediately. Its
`disk_wait` records the start of the current period below the floor and is
cleared when space becomes sufficient, even under a lane hold. After 20
minutes below the floor, reconciliation durably refuses it with code 2,
refusal `disk-floor`, and the usual release and receipt. The waiting client
prints the disk diagnostic and `heavy: disk floor refused before payload
start`. A landing whose space falls after admission still refuses at its
payload check; it does not wait inside that cgroup. Reservation and disk
wait times use wall-clock timestamps.

`HEAVY_MIN_FREE_BYTES=<decimal bytes>` opts one submission into an exact-byte
wait. Only ASCII digits are accepted: no signs, whitespace, separators, units
or fractions. Leading zeros are normalized before conversion. At most 640
significant decimal digits are supported; longer values refuse before
persistence, independently of `PYTHONINTMAXSTRDIGITS`. Python 3.11+ permits
conversion limits down to 640 digits, so accepted byte minima remain numeric
JSON readable even by that strictest supported setting. This representation
limit changes no disk floor, slot cap or timeout. Outside the slot default
below, absent or empty retains the disk behavior above, including the rounded `df -BG`/GiB observations and landing timeout. Zero opts in with no
extra caller floor. Invalid input refuses with exit 2 before creating a task.
Opting in requires both the operator deployment proof in section 7 and an
installed reconciler. The installation check proves containment/recovery only,
not exclusion of older writers. Plain execution cannot enforce this wait and
refuses instead. No caller floor is lowered.

For ordinary command submissions physically below
`$HOME/src/fabrig/slots/<ASCII decimal>/`, `HEAVY_SLOT_BYTE_ADMISSION=1`
(default 0) enables the same byte path without a caller declaration. Nested
cwd paths qualify; a symlink escaping the slot tree does not. This opt-in is
operator policy, not a hostname rule. It sets the submitted byte minimum to
`max(80000000000, explicit HEAVY_MIN_FREE_BYTES)` and the saved PLAN floor
still participates in the effective maximum below. Invalid explicit minima
refuse even when the automatic floor would be larger.

`machine_setting` resolves these settings fresh for each future ordinary
invocation, including invocations by already-running workers. It retains its
environment, machine.env, default precedence. The selected declaration crosses
the Bash/helper handoff as private `HEAVY_SUBMISSION_DECLARATION` JSON and is
removed from the payload environment; no automatic caller minimum or cap is
exported globally. Pending tasks use their saved declaration. Control commands,
record/check helpers and the direct `cargo fmt` path receive no automatic
opt-in. Explicit declarations retain their usual validation.

For an opted-in task, free bytes are exactly
`os.statvfs(command cwd).f_bavail * f_frsize`. Its effective minimum is
`max(submitted PLAN_MIN_DISK_GB * 1073741824, HEAVY_MIN_FREE_BYTES)`: the
legacy floor is converted to whole GiB without rounding the observation.
The command filesystem can differ from DEV_ROOT's filesystem. Landing
reservations still add their GiB converted to bytes to a non-landing build's
admission requirement; FIFO, landing priority, capacity and exclusivity remain.
Submission observes space, every eligible admission attempt after lane waiting
observes it again, and contained entry checks after settling and input capture,
immediately before command start. All these checks use the submitted value,
never a wrapper caller's or reconciler's current override.

An opted-in task below its minimum waits with the same token,
number, wrapper claim, command and environment. The waiting client reports one
line per observed waiting episode. The landing floor's 20-minute timeout does
not apply to these byte waits. Without the cap below their wait is unbounded.
A miss at contained entry marks `disk-retry` and exits the guard without starting the command. Reconciliation kills that owned
cgroup, proves it empty, clears its exclusive hold, and returns the task to
waiting with a fresh payload unit name. Capacity stays owned through cleanup;
the byte wait then holds no payload cgroup or build slot. Repeated attempts
retain distinct admission/scope evidence. Cancellation and recovery work even
when their caller inherits an invalid or too-high override.

A failed cwd `statvfs` observation is unavailable evidence, never permission to
admit or a fallback to DEV_ROOT. That task remains in its byte wait;
contained failure uses the same `disk-retry` cleanup as a low observation.
`byte_observation_failed` records this condition, and its waiting event has a
null observation. A successful observation clears it. One task's unavailable
filesystem does not abort shared reconciliation, unrelated cancellation or
cgroup cleanup. The waiting client can report unavailable space without
exiting.

`HEAVY_DISK_WAIT_CAP_SECONDS` optionally declares a positive ASCII decimal
number of seconds, normalized and bounded to 640 significant digits like the
byte minimum. Zero, signs, whitespace, fractions and non-ASCII digits refuse
with exit 2 before queue or log changes. Absent or empty retains unbounded
byte waits, except when the slot policy supplies a cap. That policy reads
`HEAVY_SLOT_DISK_WAIT_CAP_SECONDS` (default 1200); its selected value must be
positive. An explicit positive caller cap overrides it. A cap alone does not
opt a non-slot command into byte admission.

The declaration is fixed at submission. Monotonic nanoseconds accumulate only
between observed low/unavailable space and the next sufficient observation or
terminal decision. Separate episodes add together across retries. Sufficient
space while CPU capacity, exclusivity or landing room is unavailable adds no
disk wait, and payload execution adds none. Retry cleanup retains an open low
space interval until recovery, cancellation or expiry; cgroup cleanup after a
terminal decision is excluded. Expiry is checked under the host lock on
submission/admission/reconciliation and the final contained guard. At elapsed
wait greater than or equal to the cap, the task closes with exit 2,
`outcome=capacity-refused`, `failure_class=operational-capacity` and
`refusal_reason=disk-wait-cap`. This is an operational capacity outcome, with
no payload start. Normal empty-cgroup proof, hold release and receipt publication
still precede terminal completion. Sampling can detect expiry after the cap;
it never grants a fresh full cap to a retry or to recovered space at the boundary.

## 3. Durable state and release

The version 1 snapshot records the queue identity, next task number,
tasks, gate hold, maintenance holds, installation and event outbox. Save
writes and fsyncs a sibling temporary file, renames it over the snapshot,
then fsyncs the directory. A leftover temporary snapshot is never state.

An opted-in task additionally records `min_free_bytes` (the submitted override),
`minimum_free_bytes` (the effective floor), `byte_wait` while below it,
`admission_free_bytes` after admission, and `payload_attempt` after a retry.
New byte submissions also save `disk_wait_accounting`, schema
`heavy-disk-wait-v1`: `host`, physical `slot` or null, the selected `declaration`
(`min_free_bytes`, `cap_seconds`, `slot`, `automatic`), `cap_seconds`, `episodes`,
`elapsed_ns`, active `since_ns` or null, `clock_boot` and `complete`. State keeps
the completed prefix separate from the active monotonic interval. Same-boot
replay resumes it. A boot change or regressing clock makes the total unknown,
retaining its known prefix rather than inventing elapsed time; ordinary orphan
recovery still closes the task. Existing tasks without this accounting are
never backfilled and their wait history is unknown, not zero.

Tasks without `min_free_bytes` keep their existing semantics even if a later
caller or reconciler has the variable set. These are additive version 1 fields.

Each full task records its token, number, queue and boot identities,
wrapper identity, wrapper and payload unit names, lane, label, repository,
tree, policy, log and command. Commands remain private metadata and appear
in the diagnostic TSV; events exclude commands and output.

Admission captures candidate commit `HEAD` and `HEAD^{tree}`. A versioned
gate requires clean index and worktree, with no merge or rebase state,
at admission, contained entry, after settling and at completion. Commit,
tree, index, worktree or merge/rebase drift refuses with an instruction to
rerun land. Submitted policy must match freshly prepared policy at
submission and contained entry; preparation cannot replace the expected
identity with a new one.

Payload units use `ExitType=cgroup`, `KillMode=control-group`, and no
delegated subtree. The payload guard is a child subreaper before it starts
any command. It adopts and reaps orphaned descendants; a change of session
does not escape containment. Under the host lock, it checks its launching
token, live wrapper, boot and owned populated cgroup, and durably marks the
scope active before invoking any policy version launcher or payload.
A delayed unit creation after release therefore starts no policy tool or
payload. Every token has unique unit names that are never reused.

Wrapper death, boot change and normal payload completion use the same
release path: save closing, kill the token's cgroup, check both the unit's
reported cgroup and its saved cgroup path, and release only when neither
contains processes. An old boot does not waive the current cgroup check.
Release clears a gate's hold in the same snapshot; operator holds remain.
The submitted environment file is removed after release, and that removal
replays without changing a sealed log or receipt.

An effective kache wrapper requires an already active shared
`kache.service`. Heavy never lazily starts it inside a task. Selection
follows Cargo configuration precedence, `[env]` force/relative settings,
PATH, generic overrides, then dedicated wrapper overrides. An empty
dedicated override disables only its wrapper slot. Masked global settings
do not require kache. Refusals print the service/override remedy without
printing configuration values.

A released task shrinks once its TSV row is durable, release is complete,
its receipt exists and its landing reservation has ended. It retains token,
number, state, code, row durability and release time for late-wrapper
refusal and replay. It leaves the snapshot 24 hours after release. History
remains in events, TSV and receipts, and the next number never decreases.
Waiting clients read the snapshot at most once per second; a launcher can
instead block until its unit ends and then read it once.

## 4. Independent recovery and maintenance

`heavy --install-reconciler [INTERVAL]` defaults to five seconds, probes
real cgroup containment including a `setsid` child, and installs an enabled
user service and timer under the queue's prefix. Installation is recorded
before units are written or enabled; a partial installation is installed
and refusing until repaired. The service uses the adjacent Python helper's
absolute path and a private `reconcile.env` containing required workspace
configuration. Reinstallation rewrites these paths without changing the
queue, task history or operator holds (section 7).

Admission requires an active timer and a containment probe for the current
boot. Reconciliation and admission repeat the probe after reboot. A missing
user bus or unsupported containment refuses visibly. `heavy --reconcile`
recovers ended payloads and dead wrappers, and cancels unclaimed tokens;
the timer survives a killed wrapper or submitter.

Waiting clients report failed/missing recovery services, inactive timers
and unreadable snapshots. Service/timer failure keeps them waiting with
ownership retained; an unreadable snapshot returns nonzero without
cancelling the task. Task age and delayed ticks never establish failure.
For storage failure, free space, restore the same service, then reconcile
with the same `DEV_TMP`. Retain the ledger. A recovered code 137 requires
rerunning the command or landing.

`--maintenance-pause LANE` holds `build` or `gate` in the ledger while
running tasks continue. `--maintenance-resume LANE` clears that operator
hold, including an obsolete lane name. `--group-held LANE` returns 0 for
an operator hold, or a gate's machine hold when LANE is not `gate`.
Operator and exclusive gate holds never clear each other.

`heavy --cancel TOKEN` accepts only a known token submitted from the
caller's current directory, and refuses inside a heavy task. It skips the
disk floor and releases unfinished work on the same path with code 130.
It returns 0 when cancelled, 1 if already ended, and 2 on refusal. After
60 seconds it leaves unfinished release to the reconciler. A waiting
client's SIGINT or SIGTERM also cancels between lock sections; a second
signal leaves release to the reconciler and exits 130.

`--remove-reconciler` refuses unfinished tasks, then removes only its own
timer and service. Removing the reconciler is not part of installation
relocation: the queue, holds and recovery ownership must be retained.

## 5. Events and compatibility

`DEV_TMP/heavy/events.jsonl` records submit, wrapper start, admission,
scope entry, drain/settle/payload/release/collection phases, completion,
cancellation, refusal, recovery and migration. It carries unique event ID,
token, time, boot and queue, lane, label, repository, tree, safe policy,
capacity and entrypoint. Admission adds weight, jobs and build slots.
The snapshot outbox precedes journal appends. Appends are flushed and
fsynced, replay deduplicates event IDs, and an interrupted final JSONL
line is truncated before replay. The TSV deduplicates by per-token log
path. Reconciliation adds neither duplicate completion nor duplicate row.

The eleven TSV columns are task number, lane, UTC completion, wait seconds,
run seconds, exit code, free memory GiB, working directory, log, command
and label. Wait ends at payload start; run includes completion cleanup.
Optional cache collection shares one 60-second monotonic deadline across
service readiness and the GC RPC. Gate payloads collect every two minutes;
every payload collects at completion. Failure preserves the payload exit and
kills only its own client. Release still proves the owned cgroup empty.

After release and durable events/TSV, recovery seals capture bytes into a
separate immutable private `TOKEN.log`, fsyncs and hashes it, and atomically
publishes `DEV_TMP/heavy/receipts/TOKEN.json`. A delayed service can only
append to its capture. Publication replays after crashes. The 17 required
`heavy-receipt-v1` fields are `schema`, `token`, `id`, `queue`, `lane`,
`tree`, `policy`, `args`, `outcome`, `code`, `payload_started`,
`payload_ended`, `release_complete`, `cgroup_empty`, `elapsed_s`, `log`
and `log_sha256`. Optional `min_free_bytes`, `minimum_free_bytes` and
`admission_free_bytes` carry the override, effective floor and bytes observed
at the most recent admission; the last is absent if never admitted. Admission
events and payload capture logs record the effective floor and observation for
each attempt. Optional `head` and `inputs` record commit and effective
worktree inputs. Input capture uses a temporary Git index and changes no
real index or refs. Cancelled, recovered and unstarted payloads have
nonzero codes. The client's last line reports receipt path and token,
outside the hashed log. Relocation leaves these formats and paths intact.

New byte receipts additionally contain `disk_wait_accounting`, including the
schema, host, slot, declaration, cap and episode count above, `complete`,
`elapsed_ns`, `known_elapsed_ns`, `elapsed_seconds` and `elapsed_minutes`.
Unknown totals have null elapsed fields and retain their known prefix. These
fields join the enclosing token/id/queue to count per-host, task and slot waits
for admitted, cancelled and capped work. The same summary appears once in a
`disk_wait_accounting` event and in the sealed log before hashing. Missing
accounting means unknown history. Capacity receipts include the optional
`failure_class` and `refusal_reason` above. The 17 required v1 receipt fields,
TSV columns and ordinary command status remain unchanged. The landing reader
accepts and validates this additive evidence without treating a capacity
receipt as a successful gate.

Gate admission and payload entry load adjacent `gate-policy.sh`. Policy
identifies Chromium 147.0.7727.101 and its resources, DejaVu fonts, selected
compiler/Cargo/wrappers, semantic environment and Cargo configuration,
gate executable and implementation digests. It exports validated runtime,
fonts, helper path and safe hash, without dumping configuration. Config
includes outside the declared policy refuse. Implementation content is
part of policy: updating heavy or its helper can invalidate a cached gate,
which must be rerun. It does not alter the queue's identity.

## 6. Verification

Run the offline suites directly; they build nothing:

```
python3 -B tools/heavy/test-heavy-plain
python3 -B tools/heavy/test-heavy-byte-repairs
python3 -B tools/heavy/test-heavy-slot-wait-cap
python3 -B tools/heavy/test-receipt-inputs
python3 -B tools/heavy/test-gate-policy
python3 -I -B tools/test_install.py
tools/install --dry-run
```

The queue's automatic tools step includes `test-heavy-slot-wait-cap`,
`test-heavy-byte-repairs`, `test-heavy-plain`,
`test-gate-policy`, `test-receipt-inputs` and the installer tests. It runs
executable `test-*` suites directly through their shebangs. Exactly
`tools/heavy/test-heavy-recovery` is excluded because it installs real
systemd units; it runs manually on a builder, never automatically on the
gate machine (`tools/queue/design.md, 2.3`).

The plain suite uses temporary homes and hand-written ledgers, and installs
no unit. Its two layouts are `home/src/rust/agents/scripts/devel` and
`home/src/fabrig/main/tools/heavy`. The former runs the unchanged agents
entrypoint retained in `fixtures/agents-heavy`. With no DEV overrides,
both go through symlink resolution and their Bash defaults; a temporary
Python launcher imports the selected recovery helper without running its
main, and compares its computed workspace, temporary/state paths, queue
and prefix. It reads or writes no ledger and adds no public CLI mode.

`test-heavy-slot-wait-cap` uses a fake monotonic clock and disk observations
with the repair suite's fake cgroups. It covers cumulative/exact-boundary caps,
retry finality, cancellation, unknown old/reboot history, CPU/payload exclusion,
invalid declarations without writes, immutable submission records, fresh local
machine settings, physical slot scope, symlink escape and floor composition.
It uses only disposable queue state, with no real service or long wait.

`test-receipt-inputs` checks dirty, unborn and foreign Git contexts,
best-effort failures, unchanged indexes/refs and optional receipt fields.
`test-gate-policy` uses fake runtime/toolchain/systemd-facing heavy and
local Git repositories for policy, cache, candidate and receipt faults.
Its agents check/landing scripts are test support, not installed tools.

On a builder, from a disposable copy under `~/src/fabrig/`, run:

```
python3 -B tools/heavy/test-heavy-recovery --evidence NEW_DIRECTORY
```

This suite needs real user systemd and cgroup v2. Each case sets a disposable
HOME, config directory and DEV_ROOT/DEV_TMP, links only its private reconciler
units into the user manager at runtime, prints its prefix before installation,
and refuses the production queue prefix used by the authorized builder check. It uses
fake disk/cache/runtime/compiler version executables and temporary Git
repositories. Byte observations can use `HEAVY_DISK_FREE_BYTES_FILE` only with
the recovery-fixture flag, a non-account HOME containing DEV_ROOT and DEV_TMP,
and a file inside DEV_TMP. The offline suite checks exact cwd statvfs arithmetic
and refuses observations that could select the account HOME or another queue.
The byte cases cover invalid and empty values, floor composition, waiting and
recovery, admission after lane holds, independent reconciler persistence,
repeated payload retries, cancellation and landing reservation priority.
Cases cover admission/order, holds, migration, installation relocation and
rollback, descendant containment, subreaping, boot change, client/wrapper
loss, delayed launch, snapshot/journal/log ENOSPC, release and receipt
crashes, disk reservation and waiting, candidate/policy drift, cancellation,
cache reuse and overhead. Cleanup stops only its recorded units, removes
its reconciler and checks empty cgroups, no live wrappers and no unit files.
Evidence is retained until the operator verifies cleanup and removes the
copy. No production ledger is read or written and no production timer is
stopped. This suite does not run on hel8.

The relocation case installs from both paths against one isolated queue,
preserving an operator hold, receipt, queue and increasing task numbers,
and checks each generated reconciler command. It needs no agents Git
history. Runtime/cache cases use the fixture Chromium version command and
fixture fonts, without reading an installed browser package.

## 7. Installation and relocation

`tools/install` installs the manifest entry `heavy heavy` from a dedicated
clone at `~/src/fabrig/main` and links `~/.local/bin/heavy` to its executable.
The installer behavior and dry-run contract are
`tools/codex-run/design.md, 6`. The agents copy remains in place. The six
runtime files are `heavy`, `heavy-recovery.py`, `kit.sh`, `gate-policy.sh`,
`recovery_message.py` and `receipt_inputs.py`; helpers resolve adjacent to
the real executable, independent of the caller's working directory.

The byte wait deploys through the normal fabrig tools installation below and
is inert until a caller or the scoped slot machine policy opts in. Retain
queue identity, version 1 state, operator holds, task history, logs, receipts and `heavy-receipt-v1`. The new
helper can read pre-feature tasks without changing their legacy semantics.
That is one-way old-task compatibility, not safe coexistence of old and new
queue writers after opt-in. The cap/accounting and slot-default revision requires
the same writer inventory and ended-lifetime proof: an older byte-capable helper
can ignore the cap or publish a receipt without wait accounting. This source
change installs nothing and changes no machine setting. Ops enables slot policy
on the selected builders only after its separate rollout proof.

Before the first opt-in on each queue, Ops must record deployment proof and
explicitly approve enabling its callers. Reconciler installation or
`--check-recovery` alone is insufficient. The proof must identify the machine,
canonical queue path/identity, boot, deployed revision/helper digests, every
resolved writer entrypoint and its disposition, the drained wrapper/submitter
lifetimes and empty-cgroup/release evidence, and the reconciler's new helper
and environment. This task enables no production caller. Ops must:

1. Freeze new submissions from all producers. Drain or cancel all pre-feature
   work through its supported owner and reconciler, then prove every prior
   wrapper and in-flight submitter has ended, every task scope is empty, and
   releases/receipts are durable. Replacing files does not update Python code
   already running in a wrapper or reconciler. Preserve existing operator holds.
2. Update or disable every pre-feature writer for this queue: canonical fabrig
   heavy, retained fabrig/agents copies, sibling heavy invoked by agents land,
   direct helper invocations and the independent reconciler. Retained paths
   may remain only if they resolve to reviewed compatible code or cannot write
   this queue. A PATH symlink update alone does not cover sibling entrypoints.
   Prevent provisioning or `setup --fix` from restoring a pre-feature writer.
3. Install/repoint recovery to the reviewed helper on the same queue and verify
   it and all enabled writers against the recorded revision/digests. Record
   absence of old in-flight code as well as the on-disk paths. An already
   running pre-feature reconciler invocation must end; changing ExecStart
   does not replace that process. Keep opt-ins off
   until Ops has reviewed this evidence and issued the enable decision. Any
   reintroduction of an unverified writer invalidates that decision.

The restricted boundary is pinned to pre-feature commit
`b3dcc345c0796e2fd801c2e5b323fc5a2e5ff6ec`. That helper treats an opted-in
landing as a legacy timed wait, excludes `disk-retry` from LIVE capacity and
can publish a byte task's receipt without its byte evidence. Updating only the
reconciler cannot make older wrappers safe. The offline
`test-heavy-byte-repairs` executes those pinned functions against disposable
fake state to establish all three restrictions, with no live queue or service.
The recovery suite's `rollback` case copies the current helper between layouts;
it establishes relocation only and makes no pre-feature coexistence claim.

For rollback, freeze producers and stop new opt-ins, then complete or cancel
all unfinished opted-in tasks with the new helper. Prove empty scopes, durable
release and receipts, and ended writer lifetimes before restoring older code.
Do not reset/delete the queue or rewrite receipts. Older strict landing readers
can reject optional byte fields: retain that evidence as history and rerun gates
with the rolled-back tools when necessary. Ops records the rollback boundary
with the same writer inventory and lifetime proof.

After this branch has landed, run on each machine in a login shell:

```
~/src/fabrig/main/tools/install
```

On a build machine already using heavy's reconciler, then run:

```
env -u DEV_ROOT -u DEV_TMP ~/src/fabrig/main/tools/heavy/heavy --install-reconciler
env -u DEV_ROOT -u DEV_TMP ~/src/fabrig/main/tools/heavy/heavy --check-recovery
```

Installation rewrites the same queue's service `ExecStart` to fabrig's
`heavy-recovery.py`, refreshes `reconcile.env`, reloads/enables its timer
and reprobes containment. The default interval is five seconds; where an
operator selected a different interval, pass that same interval explicitly.
Use the same explicit DEV_ROOT/DEV_TMP for an intentionally non-default
queue. Do not copy, reset or retire state, remove the reconciler, or create
a fresh queue for this move. A machine without recovery installed does not
need these two management commands.

Entrypoint relocation preserves queue identity and submitted roots. For the
byte feature it is permitted only within the writer restrictions above:
pre-feature services must finish before opt-ins begin, and retained entrypoints
must be updated or disabled. There is no approved mixed-writer window with
byte tasks present. Compatible reviewed implementations may share a queue
through different paths after Ops proves the deployment boundary. Receipt and
log paths depend on the state directory, not the executable directory. Gate
policy uses adjacent helpers/content digests; rerun gates when that policy
changes. No forced policy ID or cache deletion accompanies installation.

A different recorded queue identity can be an older daemon-based ledger.
Heavy takes it over only once all its tasks are released, records migration,
sets the path-based identity, and numbers above all retained tasks. Until
then it refuses new work, recovers older tasks on the common release path
and leaves identity intact. The executable move itself triggers no such
migration. It never calls the former queue daemon or removes its data.

## 12. A machine without heavy

Installation in the operational sense means a ledger with a recorded
reconciler installation, rather than merely an executable on PATH.
`heavy --installed` returns 0 for installed, 1 for absent state/ledger or
no installation, and 2 when it cannot tell (unreadable or unsupported
ledger). It reads the snapshot without taking the host lock and writes
nothing. A partial installation remains installed and refusing.

Absent an installation, the command form prints
`heavy: not installed here; running it directly`, then executes the
requested command in its caller's directory and environment, with its
exit status and streams. There is no ledger, receipt, unit, disk floor or
policy preparation. Management commands retain their own semantics.
Installed but broken or unreadable never silently falls back to direct
execution. Plain execution has the caller's CPU share, rather than
heavy's weights.

## 13. Later

- Move other agents tools independently. Landing turns, dependency repins,
  landing sets and the after-landing runner are not installed from here.
  Their orchestration designs are outside heavy's host recovery contract.
- Migrate workspace/state storage only with a separate compatibility plan;
  fabrig installation alone retains `~/src/rust/tmp/heavy`.
- Historical rollback to the daemon-based generation and a real installed
  browser runtime proof need separate fixtures. This directory tests
  relocation/rollback of the current queue format and fixture runtimes.
