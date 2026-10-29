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
active, finishing or closing; their capacity remains occupied until
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

## 3. Durable state and release

The version 1 snapshot records the queue identity, next task number,
tasks, gate hold, maintenance holds, installation and event outbox. Save
writes and fsyncs a sibling temporary file, renames it over the snapshot,
then fsyncs the directory. A leftover temporary snapshot is never state.

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
and `log_sha256`. Optional `head` and `inputs` record commit and effective
worktree inputs. Input capture uses a temporary Git index and changes no
real index or refs. Cancelled, recovered and unstarted payloads have
nonzero codes. The client's last line reports receipt path and token,
outside the hashed log. Relocation leaves these formats and paths intact.

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
python3 -B tools/heavy/test-receipt-inputs
python3 -B tools/heavy/test-gate-policy
python3 -I -B tools/test_install.py
tools/install --dry-run
```

The queue's automatic tools step includes `test-heavy-plain`,
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

`test-receipt-inputs` checks dirty, unborn and foreign Git contexts,
best-effort failures, unchanged indexes/refs and optional receipt fields.
`test-gate-policy` uses fake runtime/toolchain/systemd-facing heavy and
local Git repositories for policy, cache, candidate and receipt faults.
Its agents check/landing scripts are test support, not installed tools.

On a builder, from a disposable copy under `~/src/fabrig/`, run:

```
python3 -B tools/heavy/test-heavy-recovery --evidence NEW_DIRECTORY
```

This suite needs real user systemd and cgroup v2. Each case sets fresh
DEV_ROOT/DEV_TMP, prints its prefix before installation, and refuses the
production queue prefix used by the authorized builder check. It uses
fake disk/cache/runtime executables and temporary Git repositories.
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

Transient wrapper and payload services already in flight retain their
entrypoint paths, and their environment files preserve the submitted roots.
Keep the agents copy available for them. They share the same ledger,
formats and queue with fabrig's reconciler. Receipt paths and logs depend
on the retained state directory, not the script directory. Gate policy
uses adjacent helpers and content digests; new submissions recompute it.
No receipt rewrite, forced policy ID or cache deletion accompanies install.
An agents landing tool still invokes its sibling agents heavy; moving
those tools is separate. Existing and new callers continue to share the
same host queue, although their implementation policies can differ. The
agents provisioning command `setup --fix` still relinks its sibling heavy;
when using it, run fabrig's `~/src/fabrig/main/tools/install` afterward to
restore the manifest links.

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
