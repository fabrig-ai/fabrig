# The merge queue

Provisional, 2026-10-28. How a branch reaches `main`: a script on the gate machine batches ready branches, checks them once, and pushes. No agent merges.

## 1. In one page

- **Input:** remote branches `ready/<lead>/<task>` on `origin`, each pushed by its lead.
- **A batch:** from `origin/main`, the ready branches not yet tried at their current tip, oldest tip first, at most `batch` (8) of them, each merged with `git merge --no-ff -m "Merge <branch name>"`; ratchet-only conflicts are resolved by recounting against HEAD (section 3.3); any other conflict is aborted, reported, and left out. After merging, the queue writes and commits lower old-form baselines for touched trees before checks. Branches need not change those baselines.
- **The checks run on a selection** (section 2), not always the whole workspace: touched trees' `scripts/old_forms.py --write` (the ratchet, section 3.3), then the tools' suites (section 2.3), `cargo fmt --check`, clippy with `-D warnings`, a nextest `build`, and the focused (`default`) and `fuzzy` nextest profiles. Cargo build and test commands run through `heavy --gate` on the gate machine, which nothing else builds on. Lockfile and member manifest changes select affected packages (section 2.1); formatting covers changed Rust owners (section 2.2). Step durations are recorded separately (section 3.6).
- **Green:** `git push origin HEAD:main` (fast-forward only); the batch's `ready/` branches are deleted only under an explicit expected-tip deletion lease; each lead is told which commit holds its task.
- **Red:** a batch of one is rejected (its lead gets the failing step and the log's path, and the branch isn't retried until its tip changes); a larger batch is halved and each half tried from `main`, recursively, so the green branches still land.
- **Capacity refusal:** a heavy exit 2 containing `below the disk floor` is an error, with no rejection or bisection. Before every batch, free space below `--disk-floor` (45 GB by default) defers work to the next cycle.
- **Error:** an exception logs the batch as `error`; its tips remain eligible. `run` restores `origin/main`, reports each distinct error to the operator (`ops`) once per uninterrupted error episode, waits a cycle, and continues. A completed cycle, including an idle check, clears the persisted episode so recurrence alerts again. `once` raises to its caller.
- **Batches selecting no packages** skip Cargo checks. Tools changes run the tools step; touched trees run their ratchets. Unread Markdown outside package source directories selects no package, even beside code or tools changes.
- **Every batch is logged** as one JSON line (`~/src/fabrig/notes/queue.jsonl`): start and end, branches, outcome, the selection's packages, each step's duration, and test executable cleanup counts and GB.

## 2. The selection

1. **Changed paths:** compare the checked parent and candidate trees with `git diff --name-only --no-renames -z <parent> <candidate>`. `changed_paths` accepts an explicit `base` and `candidate`, defaulting to `origin/main` and `HEAD` for the one-gate queue. Include both sides of renames and deleted paths. The unmerged `plan` uses `preview_changed_paths` instead: compare each ready tip with its merge base with `origin/main`, preserving `origin/main...<tip>` preview semantics.
2. **Everything** when a changed path is outside every package's `src/`, `tests/`, `benches/`, `examples/`, `build.rs` and member `Cargo.toml`, except the root `Cargo.lock` (section 2.1), `tools/`, and unread Markdown (`*.md`). The root `Cargo.toml`, `.config/`, `rust-toolchain*`, `rustfmt.toml`, `clippy.toml`, docs read by tests, and other scripts and fixtures force everything. Also run everything every `full_every` (5) batches, when the last full run is older than an hour, or when the gate machine has been idle for 10 minutes and main has changed since the last full run.
3. **Otherwise,** the packages that own a changed path (the longest manifest directory that prefixes it, from `cargo metadata --no-deps`), their reverse dependencies' closure (normal, build and dev dependencies, from `cargo metadata`'s resolve graph), and the packages listed in `tools/queue/always.txt` (tests that read files by path).
4. **Docs read by tests** are globs in `tools/queue/read-by-tests.txt`; a matching path forces everything. A path under `tools/` contributes no package. Otherwise package source ownership still selects its package, including Markdown under `src/`, `tests/`, `benches/` and `examples/`. Unread Markdown outside those directories contributes no package. A batch with no package inputs and no full input selects `docs`: no Cargo checks or `always.txt` packages, plus tools and touched trees' ratchets as applicable.
5. A full run that fails where the selections before it passed points at a gap in `always.txt` or `read-by-tests.txt`: the queue reports it to the operator.

### 2.1 Lockfiles and member manifests

A member's `Cargo.toml` selects its owner and reverse dependencies just as a
source change does. For `Cargo.lock`, read `<parent>:Cargo.lock` and
`<candidate>:Cargo.lock` with `git show`, without checking out the parent.
`Queue.choose` accepts explicit `base` and `candidate` inputs with the same
`origin/main` and `HEAD` defaults. Checked package selection requires the
checkout to be at the candidate before loading its metadata; an explicit
candidate that differs from HEAD is an error. Compare
entries by name, version, source, and sorted dependencies; checksums and entry
order do not select packages. Added and removed entries count as changes.
Match changed entries to the candidate's resolved package IDs by name,
version, and source. Follow reverse edges through all resolved dependencies,
including external packages, and retain workspace members plus `always.txt`.
A removed dependency selects the parent whose lock entry drops that edge;
an unreachable removed entry selects no HEAD package. Normal, build, and dev
edges all participate. Periodic, hourly, and idle full checks remain unchanged.
Python 3.11 or later supplies the standard library TOML parser.
`plan` keeps a conservative `full` preview for lockfile or manifest changes:
it does not merge, so its checkout cannot supply the batch HEAD's resolved
graph. An attempt makes the narrower selection after merging the batch.

### 2.2 Formatting changed Rust owners

For a selected batch, collect the workspace packages owning changed `.rs`
paths, using the longest manifest directory from `cargo metadata --no-deps`.
Run `cargo fmt --check -p <name>` with each owner once, sorted by name.
Reverse dependencies and `always.txt` do not expand formatting. Both sides
of renames and deleted paths count. Without changed Rust paths, including
docs-only, manifest-only, and lockfile-only batches, skip formatting. A full
run keeps `cargo fmt --all --check`. A batch with no checks to run is green.

### 2.3 Tools suites

A change under `tools/`, or any full run, runs step `tools` before Cargo.
Discover suites from the checkout's tracked files: every `tools/**/test_*.py`
runs with `python3 -I -B`, including `tools/test_install.py`; every executable
`tools/**/test-*` runs directly as `./tools/.../test-*`, honoring its shebang.
Pass the path as one argument without a shell or PATH lookup, so Python and
Bash suites both use their declared interpreters.

Exclude exactly `tools/heavy/test-heavy-recovery`: it installs real user
systemd units and requires manual execution on a builder with isolated
state and unit prefixes (`tools/heavy/design.md, 6`). It must not run
automatically on the gate machine. The three offline heavy suites
`test-heavy-plain`, `test-gate-policy` and `test-receipt-inputs` remain in
the tools step. Other executable suites, including the same recovery
basename in another directory, remain eligible.

Run serially from the checkout root with stdin from `/dev/null` and a
60-second timeout per suite. Each suite's command and output go to the batch
log; its exit code and elapsed time go in the tools step's `suites` records.
The step records its total elapsed time. Run every suite even if one fails;
any nonzero exit, missing interpreter, or timeout makes the step red and stops
later checks. Rejection and bisection follow section 1.

Queue tests allocate private disposable fixtures beside the tested checkout,
under its resolved root's parent, never inside the checkout. In operational
clones under `~/src/fabrig/`, the fixtures stay under that same hierarchy.
Train fixtures use `.mq-train-test-` plus a random suffix and mode 0700;
queue/restart/worker fixtures follow the same external-parent rule. Hard
termination can leave only external test artifacts, which are not queue input.
Cleanup removes only the fixture directory owned by that test. Python entries
and child processes use `-I -B`; shared controller imports also disable bytecode
before loading source so they cannot dirty the tested checkout with caches.

The controller's 63 cases run as three cohesive automatic entries:
`test_train.py` (19 preparation/evidence/policy cases),
`test_train_decisions.py` (22 ordering/conflict/bisection cases) and
`test_train_recovery.py` (22 ownership/replay/drain cases).
`train_fixture.py` is a test-free mixin outside automatic discovery.
`test_train_isolation.py` adds a pipe-synchronized real-fixture SIGKILL check
and a discovery check. `train_cases.json` fixes all 63 reviewed case names and
AST fingerprints; the discovery check proves every case is in exactly one
queue-discovered entry with an unchanged body and no inherited duplicates.
Each entry retains the fixed 60-second limit; gate timings require separate
operator validation with margin for overhead on the gate machine.

## 3. Running it

- `tools/queue/mq run` loops (a cycle every 60 seconds, or at once after a batch); `mq once` runs one cycle; `mq plan` prints the next batch and its selection without merging or pushing. Python 3, standard library only, plus `git`, `cargo`, `heavy` and `post` on `PATH`.
- It works in its own clone (`~/src/fabrig/queue`), whose target stays warm; its state (tips tried, batches since a full run, the last full run's time) is a JSON file beside it.
- **Leads are told by `post`:** `tools/queue/leads.txt` maps a branch's `<lead>` to a post address; it sends as `queue`.
- **Stopping:** a file `~/src/fabrig/queue.stop` makes `mq run` finish its batch and exit.

### 3.1 Cycle errors

`run` catches exceptions per cycle, aborts any merge in progress, and switches
to detached `origin/main`, discarding tracked changes left by a check. Ignored
`target/` stays in place; the root `.gitignore` and each tree's ignore file
cover build output. An errored attempt logs its error without marking its
branches tried. Recovery diagnostics and the operator's delivery are logged as a
separate `event: "recovery"` record; errors outside an attempt get their own
error record. Distinct error texts already reported to `ops` in `leads.txt`
are kept in state for the current uninterrupted error episode, so a repeated
error does not post every cycle. A completed cycle, including an idle cycle,
clears that persisted episode; recurrence alerts again. Recovery
failures are logged and the loop waits 60 seconds before trying again. If
recovery itself cannot append its record, it reports that failure on stderr
and keeps the loop running.

### 3.2 Capacity refusals

Space is measured on the queue checkout's filesystem in decimal GB. Before
every attempt, including bisection halves and idle full checks, less than
`--disk-floor` GB raises a capacity error without running a batch. `run`
posts the operator with the host and free GB once for that host's queue
floor refusal, then waits for the next cycle. A heavy step that exits 2 and
prints `below the disk floor` also raises a capacity error, preserving the
step's output (including the builder's host and space), plus the queue host
and free GB. Other exit codes or messages remain red. No refused batch
marks tips tried, rejects a branch, or bisects. `once` surfaces these errors.

### 3.3 Old-form baselines and ratchet conflicts

After all batch merges and any conflict recounts below, run
`python3 -I -B <absolute tree>/scripts/old_forms.py --write` from each touched
tree, in tree-name order, before tools or Cargo checks. Each command is step
`ratchet:<tree>` with its duration recorded. A raised count makes that step
red and stops later checks; rejection or bisection follows section 1. For
each changed `<tree>/scripts/old-forms.txt`, commit it on top of the batch's
merges as `Lower <tree> old-form baselines`, using the queue's git identity.
An unchanged baseline gets no commit. The green push carries these commits;
each bisection half recounts from its own merges. The selection uses the
batch's changed paths before these baseline commits, so a recount does not
expand a docs-only batch into Cargo checks. Batches touching no tree write
no baseline, including idle full checks with no changed paths.

Tasks do not lower baselines: the queue keeps them tight. Branches may still
change `old-forms.txt`; those changes merge normally, with the following
resolution for conflicts confined to baselines.

If every conflicted path is `<tree>/scripts/old-forms.txt` for one of the four
trees, take HEAD's version of each baseline and complete the merge. From
each such tree run `python3 -I -B <absolute tree>/scripts/old_forms.py --write`.
This compares the combined sources against the batch's baseline, then lowers
it. Amend the merge with all rewritten baselines. Exit 1 rejects that branch
with failing step `ratchet:<tree>` and restores the batch's prior commit,
including any earlier successful merges. Other command errors are queue
errors. A conflict in any other path follows the ordinary conflict path;
no ratchet resolution is attempted for a mixed conflict.

### 3.4 Merge names

Each pinned branch is merged with `git merge --no-ff -m "Merge <branch name>"`,
for example `Merge ready/jig/admission-ops`. The message names the ready
branch even though the merge uses its pinned commit, and a ratchet amendment
keeps that message. Landing notifications name the resulting merge commit.

### 3.5 Target space

After every attempt (green, red, conflict, or error), if free space is below
`--disk-floor` plus 40 GB, remove old test executables from
`target/debug/deps`, oldest mtime first, until that space is free or no eligible
files remain. Eligible files are regular executable files with mtime strictly
before the batch started, excluding `.rlib`, `.rmeta`, `.so`, and `.d` files.
Symlinks, directories, nonexecutables, batch outputs, libraries, and incremental
state stay. A symlink in the target/deps directory path skips cleanup. Recheck
each candidate's identity and mtime before deletion to protect rewritten files.
A later batch relinks any removed executable. The batch's `target_cleanup`
records the count and decimal GB of file sizes removed; deletion errors are
logged without changing a gate result. Cleanup also runs after dry checks.

### 3.6 Build and test steps

After formatting and clippy, run `heavy --gate cargo nextest run <scope>
--no-fail-fast --no-run` as step `build`, then the same command without
`--no-run` as step `default`. Scope is `--workspace` for a full run or sorted
`-p <name>` arguments for selected packages. Record each command's elapsed
monotonic wall time in its own step record. A failed build stops checks and
the rejection names failing step `build`. Nextest exit 4 (no tests) is green
for build, default, and fuzzy, as for an empty test selection. Fuzzy remains
one step with `--profile fuzzy`; the build step already built its binaries.
Ratchets run before tools and Cargo checks (section 3.3). Docs-only batches
run only touched trees' writing ratchets, plus tools suites when applicable.

### 3.7 Restarting and the login shell

Discovery changes take effect when the running queue restarts. Land and
activate a discovery change before queuing suites that require it.

`bash -lc 'tools/queue/restart'`, from the queue clone, checks `cargo`,
`heavy`, `post`, and `git` on PATH before changing anything. A missing tool
exits 2 and names it; `mq run` performs the same check before its first git
command. `once` and `plan` keep their existing behavior.

Restart serializes invocations with the shared layout's `restart.lock`. It
finds the checkout lock's holder through `/proc/locks`; there is no broad
process-name fallback. It writes `~/src/fabrig/queue.stop` and waits through a
pidfd when a holder exists. `--wait` bounds this wait (default 1800 seconds).
A timeout retains stop and performs no fetch or replacement. Process absence
permits only an attempt to acquire authority and inspect persistent ownership.

After the old process exits, restart acquires the common authority lock and
checks section 3.10's persistent guards before any update. It locks the
checkout, fetches origin and switches to detached `origin/main`. The Git
children inherit both locks. A competing direct writer or unresolved ownership
causes refusal with stop retained. Restart does not hold main authority while
waiting for the old controller to drain.

It then removes stop and starts `python3 -B -u tools/queue/mq run --repo <clone>`
in a new session, stdin `/dev/null`, both outputs appended to
`~/src/fabrig/notes/queue.out` (0600). Internal arguments pass the same open
flock description and an acknowledgement pipe to the child. The child adopts
that inode, acquires checkout exclusion and checks persistent guards before
acknowledging startup. Restart waits up to the existing three-second startup
window for that acknowledgement and checks for early exit. Unconfirmed startup
retains stop and reports failure. Closing the parent's duplicate does not
unlock the child's authority. `--repo` defaults to the current clone root.

`restart --batch N` sets the replacement queue's positive batch size; the
default remains 8. Pass the option on each restart to retain a nondefault
size. It changes batch capacity, not checks, rejection or bisection policy.

### 3.8 Local immutable-candidate worker

`worker.py` is the synchronous checker primitive. Section 3.11 supplies its
supervised local/SSH endpoint and section 3.12 explicit executable train mode.
Production rollout remains OFF pending review and dedicated real-tool proof.
Existing one-gate behavior remains the default.

Run the policy from a frozen runtime copy **outside** the worker checkout.
The configured checkout must be an independent clone, its root, with no
symlink in its directory path; linked worktrees are refused. The configured
artifact directory must already exist, outside the checkout, owned by the
worker's user and mode 0700. Invocation is explicit:

```text
python3 -I -B /frozen/tools/queue/worker.py identity
python3 -I -B /frozen/tools/queue/worker.py run --enable-local \
  --repo /gate/checkout --artifacts /gate/artifacts \
  --request /gate/request.json --gate gate-name
```

`--disk-floor` is a global option, default 45 decimal GB, as in `mq`. It is
part of the runtime identity; a request cannot change it. `identity` prints
JSON without touching the checkout. `run` prints the result path and exits
0 for green, 1 for red, 2 for INFRA/refusal. A malformed request, invalid
configuration, occupied checkout, or existing attempt directory is refused
before admission and has no new terminal result. Diagnostics go to stderr.
There is no implicit path, host, fetch, or request discovery.

The version-1 request has exactly these fields:

| Field | Contract |
|---|---|
| `version` | Integer 1. Unknown fields and duplicate JSON keys are refused. |
| `attempt`, `generation`, `gate` | Nonempty ASCII names, starting with an alphanumeric, then alphanumerics, `_`, `.`, `-`; at most 128 characters. `gate` must match local configuration. Attempts are unique within the configured artifact directory, across generations. |
| `parent`, `candidate` | Full lowercase commit OIDs (40 or 64 hex digits), already transferred into this clone. |
| `branches` | Ordered list of exactly `{name, tip}` objects: unique valid `ready/<lead>/<task>` refs and full pinned commit OIDs. |
| `selection` | `"full"`, `"docs"`, or a sorted unique list of package names. An empty list is distinct from docs. |
| `runtime` | Exact object printed by this runtime's `identity` command. |
| `plan` | Complete ordered plan from `check_plan`: command entries `{name, command, cwd}` and a tools entry `{name: "tools", suites: [command entries]}`. |

The future preparer computes the plan with this frozen policy against its
prepared candidate. Commands are full argv arrays; cwd is checkout-relative
(`.` at the root). Absolute ratchet script paths normalize to
`./<tree>/scripts/old_forms.py`, so host paths do not affect plan identity.
The worker independently recomputes and compares the entire plan. It never
executes commands supplied by the request. There is no shell interpolation.

Before checkout mutation the worker verifies a clean checkout, all requested
objects are commits, the parent and each pinned input tip are ancestors of
the candidate, disk capacity, exclusive ownership, and runtime identity.
Local replacement/graft histories and index flags that hide worktree changes
are refused. It does not consult moving ready refs or `origin/main`. Object import and
merge-chain preparation are the future controller/transport's job; ancestry
and pinned identities here do not attest the controller's merge construction.
An idle full check may have parent equal to candidate and no branches.

Under ownership, it detaches to exactly the requested candidate, then
computes changed paths from parent to candidate. Candidate metadata is loaded
at that HEAD for full/package selections, including the resolved dependency
graph. Root, members, manifests and graph completeness are checked; a digest
of package identities, relative member manifests and dependency edges removes
host-specific Cargo IDs and paths. Docs selections need no Cargo metadata.
Package/docs selection must equal the recomputed selection using the frozen
lists and explicit parent/candidate lock blobs. Full is always an allowed
escalation; deciding when a full run is due belongs to the future controller.

Checks retain the same order, tools discovery, manual recovery-suite
exclusion, suite timeout, nextest exit-4 interpretation and serial
`heavy --gate` commands from sections 2.3 and 3.6. The small shared
`run_checks(check_only=True)` mode omits ratchets' `--write` and never stages
or commits a baseline. Its default remains the one-gate writing/committing
behavior. Candidate preparation must lower baselines before dispatch; the
read-only invocation uses each candidate ratchet script's existing comparison
semantics, including lower-count advisory output. It cannot certify a
rewritten replacement commit. The same candidate metadata supplies formatting
ownership, avoiding a second metadata graph during checking.

Runtime identity freezes the loaded bytes of `worker.py`, `mq`, `always.txt`,
`read-by-tests.txt` and `leads.txt`, the Python major/minor/micro version, disk
floor and check-only ratchet policy. Those files are checked against their
startup bytes before admission and after every command and final cleanup.
This identity binds the **running** checker and lists independently of
checkout paths. Candidate code, tools suites and ratchet scripts are inputs
from the certified tree; their presence does not make them the running worker
policy. PATH tools (`git`, Cargo, heavy, interpreters), machine configuration,
installed tool versions and heavy's supervision are operational prerequisites,
not attested by these source digests. Runtime copies must not be edited while
in use. Injectable Python checker/metadata arguments are test seams, not a
production plugin or a request capability.

Each attempt creates `<artifacts>/<attempt>/` (0700), `checks.log` (0600) and
`result.json` (0600). Existing attempts are never overwritten or rerun. A
result is fsynced to a private temporary file, atomically replaced, then its
directory fsynced. The log is fsynced and its path, byte count and SHA-256 are
recorded. Logs with completion unknown are explicitly marked incomplete.
The version-1 result includes:

- The complete `requested` object and `observed` gate/runtime identities,
  start/end times, computed changed paths, selection, plan and metadata digest
  when available. Unreached preflight fields are absent or null.
- `initial`, `before`, `after` snapshots with exact HEAD, tree, clean flag and
  porcelain status. Before checking, after every command (including individual
  tools suites), and after cleanup, HEAD/tree/worktree must remain identical.
  Ignored build output is outside the tracked/untracked cleanliness check.
- Ordered `steps` with commands, cwd, return codes, green flags and durations;
  tools steps include every suite. `events` preserves completed command/suite
  evidence if an observer aborts the enclosing step. A red may stop the plan
  at its first failed step; later planned steps are unrun. A tools red still
  runs every suite. A green requires the entire exact plan, including all
  suites. Only a recomputed empty plan can certify an empty step list.
- `target_cleanup` evidence using the existing floor-plus-40-GB and batch-start
  protection rules, `completion_known`, `reconciliation_required`, `outcome`
  (`green`, `red`, `INFRA`) and `error` (null only on ordinary success/red).
  Cleanup deletion errors are recorded without changing check outcome, as
  in the one-gate queue.

Ordinary nonzero test/check exits are red. Metadata failures, missing objects,
identity/selection/plan errors, dirty or rewritten candidates, launch errors,
checker exceptions, malformed/incomplete evidence and recognized heavy disk
refusals are INFRA. They provide no basis for rejection or bisection. A
signal-terminated heavy client (also exit 137) is INFRA with completion unknown.
The worker never resets or repairs unexpected writes; it preserves them and
records the final identity. Any result publication failure retains ownership
ambiguity rather than certifying success.

Ownership uses the same adjacent `.queue.lock` flock as one-gate `mq`, held
for the whole invocation through commands, cleanup and result publication.
Before detaching, it durably writes `.git/mq-worker-unresolved.json`, naming
the attempt, generation, candidate and artifacts. A second worker cannot
enter even after process death: this marker refuses reuse until explicit
operational reconciliation. A normal complete invocation removes and fsyncs
the marker only after publishing its result and confirming unchanged inputs.
An unexpected write also retains the marker, even if the command returned.
Checker exceptions or interruption leave completion unknown and skip target
cleanup. A crash after result publication but before marker removal still
requires reconciliation. Process absence, a lost client, or a released flock
never proves that heavy is quiescent.

Section 3.11 supplies supervised evidence and explicit fail-closed recovery.
Standalone unresolved workers without that supervisor retain their existing
operational recovery boundary; no process-absence or marker-deletion shortcut
is supported. Workers never fetch/push origin, post, delete ready refs or
promote a candidate.

### 3.9 Controller library

`train.py` supplies the reviewed decision library. Section 3.12 selects it only
with an explicit private train configuration; plain Queue behavior remains
the default. Production activation awaits separate review and rollout proof.

`Controller` receives `Preparer`, `StateStore`, transport, authority, time,
generation, two gate names, frozen worker runtime and batch/full cadence.
The metadata loader is mandatory: preparation has no implicit Cargo command.
The disposable harness injects candidate metadata and fake checkers; real Git
merges, bundles, ratchet fixtures and local-origin pushes exercise the boundary.

Preparation and reservations:

- Preparation owns an independent clone root, separate from both gates.
  Discovery retains `list_candidates`' oldest-tip order and tried-tip semantics.
  Gate endpoints/configuration, controller source digest, worker/list runtime
  identity and policy are frozen in the generation's state. No runtime edits
  or changed configuration can continue that generation silently.
- `admit` persists pinned inputs, explicit parent and a preparation intent
  before mutation. At most two records have adjudication authority; a completed
  successor occupies its place until its predecessor lands. Branch names are
  reserved across those records, pending bisection groups and invalidated jobs
  still draining. Advancing or deleting a ready ref does not change its pinned
  selection or prevent that selected commit landing.
- `prepare_candidate` calls `build_batch` on explicit P, then only writing
  ratchets. It commits lower baselines before freezing C, paths, metadata,
  selection and the normalized complete check plan. Successor P is exact C of
  its predecessor while origin/main may still lag. Workers check the original
  C; they cannot rewrite or generate a substitute.
- The record preserves ordered input/merge identities, each merge's previous
  commit and complete message (including Git's conflict annotations), explicit
  already-contained inputs and baseline commits. `verify_chain` validates the
  input partition/order, exact merge parents/messages and baseline-only suffix.
  Parent-relative paths and lock blobs and candidate-resolved metadata drive
  selection. Train paths include generated baseline commits; these may cause a
  conservative full escalation rather than the one-gate selection in 3.3.
- Parent, inputs and candidate stay pinned under local `refs/mq/jobs/<id>/`.
  `export_bundle` writes a self-contained immutable bundle, recording size,
  SHA-256 and every ref/OID. `import_bundle` checks bytes, heads, prerequisites
  and requested objects without changing checkout HEAD or publishing refs to
  origin. Transport must hold ownership while importing. Pins/artifacts have
  no automatic deletion in this part.
- Conflicts block successors throughout preparation, checking and adjudication.
  When some inputs merge, dispatch that exact merged subset in the same attempt
  against its pinned P/C; the conflicts remain provisional. A valid green lands
  C and records merged inputs' tried marks and conflict rejections in one
  durable decision. A valid red rejects the conflicts and bisects only the
  checked merged inputs, preserving the current full requirement. Preparation,
  transport, capacity, checker or result INFRA retains no branch rejection or
  tried mark from that attempt, and uncertain gate ownership stays unresolved.
  When all inputs conflict and the merge chain is empty, an immediate conflict
  decision needs no worker check. An unchecked partial-conflict decision is
  refused. Ratchet exit 1 is a preparation red; other exits, missing objects,
  metadata or preparation errors are INFRA. Preparation red conservatively
  requires full checks for its later bisection groups.

Transport and evidence:

- `identity()` supplies the frozen gate configuration; each entry names `repo`
  and optionally `host` (default `local`). Gate endpoints are distinct and a
  local endpoint cannot be the preparation clone. Adapters must establish that
  their endpoints are independent clones, not aliases for shared storage.
- `start(request, bundle)` must be idempotent for generation/attempt, always
  naming the same supervised job. `dispatch` persists `dispatch-intent` and
  gate ownership first. Transfer/start/status exceptions are INFRA, never red,
  and never establish that the gate is free. Retrying start uses the same
  request, candidate and attempt identity.
- `status(request)` returns `running`, `unknown`, or `terminal`. Terminal
  includes its exact request, full result/log bytes, `quiescent` and
  identity-bound `evidence`. A boolean alone cannot release ownership. The
  frozen completion adapter must verify that evidence independently of outcome;
  the standalone local adapter accepts only empty plans. Section 3.11 supplies
  supervised heavy/SSH evidence. Process/SSH absence, a terminal client
  and a released flock are insufficient.
- `ingest` stores immutable returned bytes, including malformed or incomplete
  evidence, in controller-owned 0700/0600 storage. `validate_result` separately
  recomputes objects, chain, paths, metadata graph, selection and complete plan
  from the preparation clone and controller-owned request. It compares request,
  generation/attempt/gate/runtime, P/C/tree, ordered inputs, exact normalized
  commands/cwd, clean initial and unchanged before/after snapshots, observer
  snapshots after every command/suite, and full log SHA-256/length/completeness.
- Step validation retains the worker's first-red-prefix rule, all-tools-suite
  accounting, and nextest exit-4 interpretation. Green needs the entire plan;
  only an expected empty plan permits empty green evidence. INFRA, malformed
  or mismatched evidence grants no rejection/bisection authority and retains
  gate ownership until explicit reconciliation. Valid terminal evidence with
  established quiescence can release a gate; an invalidated result cannot
  regain adjudication authority.

Adjudication and effects:

- `adjudicate_head` holds out-of-order results without counters or effects.
  Non-INFRA decisions require fresh main equal to certified P. A red head
  immediately revokes descendants' authority; their physical jobs drain.
  No replacement is admitted while invalidated or adjudicated uncertain jobs
  own gates. Sequential bisection persists left-before-right groups, prepares
  each group on actual main after earlier groups land, rejects only singleton
  check failures, and inherits full requirements until recovery ends.
- `full_due` charges unresolved predecessors as attempts without assuming
  an unresolved full resets state. It preserves the fifth-batch/hourly policy;
  idle full checks retain the ten-minute/main-change rule and use P == C with
  no push. Adjudicated green/red checks update cadence, including checks of
  partial-conflict merged subsets; all-conflict decisions, INFRA and invalidated
  speculation do not. Full red after selected successes enqueues the
  selection-gap event.
- Green first persists `promotion-intent` with explicit P, C and stable id.
  `authority.guard()` must establish one sole main authority and startup guard
  across legacy, train and restart. `main()` reads fresh authoritative main.
  `promote(P, C, id)` must recheck P under that same authority and push explicit
  certified C with a normal fast-forward push. There is no main force-push or
  HEAD/reconstruction fallback. After uncertain push, main == C finishes the
  recorded landing; main == P retries the identical intent; unexpected main
  blocks adjudication/admission for reconciliation.
- `apply_decision_once` atomically records the applied decision, tried tips,
  counters, pending bisection and outbox in one state snapshot. Replaying a
  recorded landing changes these once. `flush_outbox` delivers separately;
  failures cannot change the gate decision. Posts are at-least-once, with the
  stable event id in every message body; adapter post exits 0 and 3 mean
  delivered. A crash after acceptance but before state acknowledgement may
  deliver that same id again.
- `authority.cleanup(name, expected_tip, id)` performs expected-tip deletion
  only. A changed/deleted/recreated ready ref is a harmless cleanup disposition;
  it cannot change a landing. The disposable adapter exercises explicit
  expected-OID leases on deletions only. Section 3.10 supplies the concrete
  adapter; workers still have no main writer.

Persistence and remaining recovery boundaries:

- `StateStore` uses version 1, refuses unknown versions and does no migration.
  Its private JSON snapshot is the recovery source; any future JSONL audit is
  descriptive only. State publication uses a private temporary file, file
  fsync, rename and directory fsync. Requests and returned evidence use private
  durable files; repeated identical evidence is accepted without overwriting,
  and different bytes under an existing identity require reconciliation.
- A publication failure poisons the in-memory controller: reload/reconciliation
  is required before more work. The fake suite injects failures at preparation,
  pre/post dispatch publication, returned-evidence publication, decision after
  push, lost push acknowledgement, and post acknowledgement. These prove model
  replay properties, not arbitrary-machine-crash or remote-job safety.
- `recovery_inventory` exposes main, gate owners, unresolved preparation,
  promotion intents and pending outbox. `reconcile_job` requires explicit
  quiescence and operational evidence; it does not reset a checkout or infer
  heavy completion. An interrupted preparation remains reserved and blocks
  admission. Section 3.10 hardens these boundaries; interrupted preparation
  without a completed snapshot remains blocked for supervised reconciliation.
- `stop` durably closes admission. `drained` requires no active records and no
  gate owners, including invalidated/uncertain owners. Existing jobs adjudicate
  in order; stop starts no replacements or pending bisection groups. Those
  groups remain durable. Section 3.12 integrates explicit resume/restart.

Section 3.10 implements shared local authority and concrete effects. Sections
3.11–3.12 supply supervised transport and operation; reviewed rollout remains
a separate operator action. No local test asserts real heavy quiescence, real-service
authority, arbitrary machine-crash durability or remote transport safety.

### 3.10 Shared authority, effects and fail-closed recovery

These authority primitives are used by the explicit train mode in 3.12.
No production queue, second prover or installation is activated by this code.
All queue modes on the same production host must use one authority directory,
including alternate preparation/worker clone names. The default is
`~/src/fabrig/queue-authority`; `--authority-dir ABSOLUTE` on `mq`/`restart`, or
`MQ_AUTHORITY_DIR`, selects an isolated queue layout. An override denotes a
queue identity, never a per-clone lock. It must name an owned 0700 directory
without symlink components; its parent must exist. Tests pass private fixture
layouts and never use the production default.

`authority.Layout.acquire()` holds `authority.lock` (0600); `restart_lock()`
holds a separate `restart.lock`. `Lock.guard()` checks the owning process and
inode. Locks are never unlinked or explicitly unlocked during handoff. Legacy
`run`, `once`, dry cycles and `plan` acquire common authority before fetching,
then their checkout's sibling `<clone>.queue.lock`. `Queue` mutations require
an injected authority/startup guard; the CLI supplies it for the entire run,
including error recovery, bisection and pruning. Local Git/check/post children
inherit both the common authority and checkout locks, so controller exit
cannot permit another main writer or local worker to reuse the checkout while
those effect clients still run. This proves no remote heavy lifetime.

`Layout.legacy_guard(repo)` refuses any `train/state.json` or
`train-owner.json`, even corrupt state or a drained-looking snapshot. The
concrete train adapter binds its state to `<layout>/train/state.json` and
publishes an immutable generation/configuration ownership receipt before any
train mutation. Missing state cannot erase that startup refusal. Returning to
legacy requires explicit supported retirement in 3.12, never deleting a
snapshot to infer safety.

Checkout exclusion recognizes the existing `.git/mq-worker-unresolved.json`
and `.git/precheck.unresolved`, plus preparation ownership at
`.git/mq-preparation-unresolved.json`. It checks `.git/precheck.lock`, sibling
`<clone>.precheck.lock` when present, and the shared sibling queue lock.
Markers block without parsing their contents; invalid markers and missing
processes cannot grant reuse. Worker and preparer use the same checkout guard.
Preparation holds checkout exclusion through its completed state publication;
a durable marker survives a hard exit or failed publication. Recovery refuses
fetch/reset/switch/cleanup while unresolved. Ordinary one-gate notifications
and check/outcome policy retain their existing behavior.

`authority.GitAuthority(layout, lock, repo, origin, leads, runner=...)` supplies
`guard`, `identity`, `bind`, `main`, `promote`, `cleanup` and `post`. Caller
configuration freezes the explicit origin and copied area/address mapping;
`bind` requires the shared train state and an independent authority clone
(which may be the preparation clone). Origin must
be an absolute local path or validated SSH/HTTPS address; subprocesses use
argv, full refs and full lowercase OIDs, with no shell interpolation.

- `main()` freshly reads exactly `refs/heads/main` through `ls-remote`.
- `promote(P,C,id)` loads and matches the durable head's promotion intent,
  freshly requires main == P, imports the original verified bundle into a
  separate controller clone when needed, verifies commit types and P ancestry
  of C, freshly compares P again under the same authority, then performs `git push -- ORIGIN C:refs/heads/main`. There is no force, main lease,
  HEAD substitution, reconstruction, recount or rebase. P/C/unexpected replay
  stays in `Controller.promote`: C applies the landing once, P retries the same
  intent, unexpected main blocks. Main authority must still be unique among
  compliant writers; this is no server-side expected-parent extension.
- `cleanup(name,tip,id)` validates the full ready ref, then pushes only its
  deletion with `--force-with-lease=refs/heads/ready/...:TIP`. A fresh differing
  tip or absence completes cleanup harmlessly; unreadable state/transport
  remains pending. The lease compares OIDs; ref recreation at the same OID is
  indistinguishable under this approved contract. No main update uses a lease.
- `post(event)` routes landed/rejected to the frozen branch area and
  infrastructure/selection-gap to ops. Every body carries `Event: ID` and its
  controller-owned evidence directory. Content goes through stdin. Exits 0/3
  acknowledge delivery; other exits/exceptions leave the durable outbox
  pending. Replays retain ID and never change a landing or rejection.

Legacy `push_and_cleanup` uses the same deletion-only expected-tip lease after
its observation. A lead can advance a ready tip between observation and push;
the server rejects that deletion and preserves the replacement. Its main push
remains an ordinary fast-forward with no force or lease.

`durable.py` implements owned 0700 directories and 0600 regular files, strict
JSON parsing, temp-file fsync, atomic rename and containing-directory fsync.
Immutable publications serialize check/rename on the directory's stable flock
inode and reject different existing bytes; identical replay also syncs the
directory. State/marker replacement refuses unsafe targets. Logs are written
as private partial files and published before terminal results. Bundles and
requests are immutable. Validation refuses missing, public, symlinked,
truncated or digest-mismatching referenced evidence. Pins use create-only
expected-old-OID updates and cannot silently change identity.

`StateStore.load()` and `recovery.validate_state()` refuse unknown versions,
wrong structural types, changed exact generation/runtime/helper/configuration
identity, malformed P/C/input/pin identities, phases and gate ownership,
request/result mismatches, decision/tried/outbox/cadence inconsistencies and
invalid sequential bisection frontiers/full inheritance. Version 1 is retained;
there is no automatic migration or acceptance of an old runtime's generation.
The frozen runtime includes train, authority, recovery, durable and restart
sources as well as mq, worker and policy lists.

`validate_artifacts()` checks every completed record's request, bundle,
preparation log, pins, merge chain and returned evidence before replay effects.
Accepted certificates are independently recomputed at decision/promotion.
A failed snapshot/evidence publication or failed certificate recheck poisons
that controller instance. Reload/reconciliation is required before further
effects; malformed evidence is INFRA, never a test red. Missing snapshots with
retained evidence/pins refuse new-generation initialization. Snapshot JSON is
the recovery source; JSONL summaries have audit meaning only.

`Controller.startup_inventory` / `recovery_inventory()` expose fresh main,
P/C/unexpected promotion dispositions, preparing records, dispatch/physical
owners (including adjudicated or invalidated draining jobs), evidence and
pending cleanup/post events. Artifact inventory lists orphan/partial and
unrecorded returned publications without following symlinks or deleting data.
A reloaded controller blocks new preparation until each recorded gate has an
identity-matching status observation or supported completion reconciliation.
Every preparation/dispatch also requires fresh main consistent with recorded
ancestry. A prepared record is dispatched/reconciled before another admission.

`reconcile_preparation(attempt)` can clear only the exact completion marker
backed by a validated completed snapshot. Incomplete preparation remains
reserved, without reconstructing certified C or changing checkout/target.
`reconcile_job(..., quiescent=True, evidence=...)` requires the frozen completion
adapter to return its verified identity-bound evidence object. A generic
boolean/string/PID fails. The built-in `local-empty-plan-v1` evidence names the
exact private worker result path and SHA-256, matches the durable request,
checks complete result/log and absence of checkout markers while holding its
lock, and requires an empty plan. All heavy/nonempty/remote plans remain
blocked in this standalone adapter; 3.11 supplies supervised lifetime evidence. Tests inject an
explicit fake-completion adapter whose events attest synchronous fake checks;
this is not a production supervisor.

Snapshot validation replays each admitted bisection group as consumption of
the first pending group, checks its ordered inputs and inherited full
requirement, then applies any red split or INFRA requeue. Thus an admitted
left group is absent from pending across reload and concrete promotion; its
right group retains the accepted left ancestry. Nested reds and INFRA retries
follow the same replay rule.

Applied INFRA decisions retain `decided` through late unknown/status failures
while their gate remains owned. Verified reconciliation releases that owner
into `discarded`, preserving the applied outcome, counters and outbox. A
terminal reply has the same release behavior. `drained()` requires a valid
loadable snapshot matching memory in addition to no active/physical ownership
or unresolved preparation; an unloadable snapshot cannot authorize drain.
Restart restores stop if unlink, directory fsync or replacement launch fails.

Authority/recovery/worker/mq tests place their disposable repositories and
publication files beside the checked source, so a hard test-process exit leaves
no fixture inside the candidate checkout. Test suite timeouts are unchanged.

No pin, artifact or snapshot GC is provided. Retain all objects/evidence;
removal requires a later proof covering active, draining, bisection, decision,
outbox and recovery references. There is no age-only removal or retention
budget. Disposable processes inject crashes around file fsync, rename,
dispatch, push/decision and post acknowledgements; tests prove those local
boundaries and shared flock lifetime, not arbitrary machine power-loss or
remote-supervisor guarantees.

### 3.11 Supervised gate transport (opt-in; rollout OFF)

`transport.py` uses ordinary local argv or OpenSSH with one doubly quoted
`bash -lc 'exec ...'` command. Only configured host/runtime/config paths enter
that command; requests, refs, bundle metadata and bytes travel in `wire.py`'s
binary envelope. SSH requires an explicit validated host alias, BatchMode,
StrictHostKeyChecking and a private known-hosts file with a frozen digest.
Configuration and Python/runtime/list identity are frozen for the generation.

The wire version is 1: `MQ-WIRE-1\n`, an unsigned network-order 8-byte JSON
header length, exact JSON header, then exact length/SHA-256 blobs. JSON headers
and results are bounded at 8 MiB, bundles/supervision at 512 MiB, worker logs at
128 MiB; diagnostics have a separate bounded pipe. Duplicate/nonfinite JSON,
truncation, digest mismatch, trailing banners and version mismatch refuse.
RPC timeout is explicitly configured (positive, at most the unchanged restart
wait of 1800 seconds); an enclosing drain clamps RPC and authority-effect clients
to its remaining deadline without changing the frozen configuration. Expired
drain starts no additional RPC/effect. Terminating a client never releases a gate.

`supervisor.py --config PATH --config-sha256 SHA start|status|reconcile-reboot`
consumes an envelope containing version/request/bundle. `start` has one bundle
blob. Status has none. Terminal replies return result, complete checks log and
supervision receipt as three blobs. Reconciled replies return one receipt blob.
Transport binds every reply to exact request and endpoint digest; receipt digest
is independently checked before returning it. CLI diagnostics use stderr only.

The endpoint configuration has exact version-1 fields: gate, incarnation, repo,
artifacts, jobs, runtime, disk_floor, heavy, heavy_sha256, heavy_root, heavy_tmp.
All paths are explicit absolute paths; runtime/checkout/stores are disjoint.
Private stores/runtime/config use 0700/0600 and reject symlinks. Heavy's six
runtime files are hashed separately; queue code uses its supported read-only
`--completion-query`, never its state files. See `transport-operations.md`.

Acceptance writes immutable request, self-contained bundle, exact claim with
request digest/nonce/boot/incarnation/config, endpoint owner and a checkout
`mq-transport-unresolved.json` marker before a launch intent and detached start.
The accepted attempt is never rerun, including a crash before spawn. Same-job
start/status are idempotent. Checkout exclusion passes the same open flock to
the supervisor, so startup has no ownership gap. Legacy, preparation, workers
and restart refuse that marker. The owner covers all checking and cleanup.

The Linux supervisor becomes a child subreaper. Each command runs serially in
its own process session. A returned direct child is insufficient: waitpid must
reach ECHILD, including orphan descendants, before the next command or cleanup.
The existing tools timeout remains red only after killing the owned group and
reaping all descendants; uncertain completion remains INFRA. Every command has
an immutable intent before spawn and completion journal after proof. Heavy
commands use a unique request-digest/index label; the supported query must
identify exactly one matching repo/argv/candidate task and immutable release,
empty-containment receipt and sealed log. Token/id/queue and complete heavy log
are persisted before the next command. A killed worker/client before this
identity or completion is known retains UNKNOWN.

After all children are reaped, the worker result/log, exact observed command
sequence, runtime and final immutable checkout snapshot are checked. A terminal
supervision receipt is published after closing checkout exclusion, while the
persistent owner marker still blocks writers. Status independently verifies
receipt/journals/heavy evidence/artifacts and removes only that exact marker
under endpoint plus checkout exclusion, with a retained release audit. Result,
checks log and full supervision/receipts/heavy logs are returned into immutable
controller-owned storage before adjudication. Replay validates their hashes.
Corruption cannot establish red or green; it retains ownership for explicit
reconciliation. No client PID, released lock or missing service is release proof.

There is no automatic safe recovery from a lost subreaper on the same boot.
The explicit `reconcile-reboot` path requires a different kernel boot, then
supported heavy recovery evidence for every journaled label: each match must
be released and contained; ambiguity/unreleased jobs block. Zero matches is
allowed only here, because the old boot cannot have a pending submit client.
It writes a distinct immutable reconciliation receipt, retains all old bytes,
checks exact markers and releases physical ownership without a check
certificate. The controller records INFRA and never rejects those tips. Dirty
inputs are retained for inspection; no reset, heavy unit stop or handwritten
receipt is supported. A dedicated-host reboot is a separate operator/user
decision, never an action performed by this tool.

This is cooperative ownership, not a sandbox for hostile candidate code or
external writers. Supported heavy launches are the exact planned direct heavy
commands; arbitrary tools must not create detached external service jobs or
independently submit heavy work. Same-boot recovery of a lost subreaper and
incomplete preparation remain blocked. No arbitrary power-loss guarantee or
real SSH/systemd/heavy proof is claimed by the offline tests.

### 3.12 Train operation, drain and audited retirement (rollout OFF)

`mq run|once --train-config PATH` selects `operation.py`; no option retains
legacy Queue. `plan` remains legacy. Train config owns paths/policy; legacy
path/policy overrides are refused. Exact private version-1 config fields are
in the operator runbook. `train-operation.json`, train owner and snapshot freeze
mode/config/batch/endpoints/runtime. Unknown versions/config changes refuse
without migration. Snapshot version remains 1, with optional hashed returned
supervision artifacts for this completion adapter.

`mq train-status --train-config PATH` reads a validated snapshot without taking
main authority or guessing fresh main/job status; unobserved main is `unknown`.
`train-reconcile` takes authority, polls both physical owners, reconciles
uncertain promotion intents and delivers the existing outbox, admitting
nothing. It may explicitly release a proved physical owner whose check
certificate was INFRA. It never manufactures a check outcome.

`train-stop` durably writes the configured stop file and works while the
controller holds authority. The run loop notices it through inotify while
idle. Stop closes admission, starts no replacement/bisection, and finishes
ordered admitted decisions plus invalidated physical owners. Pending bisection
remains durable. `train-drain` runs that process explicitly; `train-resume`
requires proved drain, resolved promotions/preparation and delivered outbox,
then reopens the same configuration with pending bisection preserved. A completed
prepared record interrupted before dispatch resumes its exact existing admission,
including its validated completed-preparation marker; it admits no new group. The
existing 60-second idle/error cycle and five-batch/hour/idle cadence remain.
Cycle-error delivery has a separate private durable episode record and stable
IDs; completed cycles clear the episode so recurrence alerts again.

`restart --train-config PATH [--wait 1800] [--batch 16]` preserves the exact
configured mode/batch/runtime; an explicit batch must match. It serializes
restart, writes stop, waits for the common authority inode's actual holder via
pidfd, then acquires authority and reconciles/drains BOTH gates. A dead
controller grants inspection only. Unknown/unreachable ownership or timeout
retains stop and performs no fetch, runtime update or launch. Replacement is
the same frozen runtime/config, receives the same open authority description
and acknowledges only after recovery; the existing three-second startup window
is unchanged. Failure restores stop, including failures after resume but before opening the
replacement output/launch. Runtime upgrades cannot mutate a live
frozen generation: drain and explicitly retire it before deploying replacement
code through the separately reviewed rollout. This helper does not fetch code.

`train-retire` requires a valid matching snapshot, no active/unknown physical
owners or incomplete preparation, no pending bisection, and every outbox item
delivered. Under authority it stops admission and publishes immutable
`retired.json` binding generation and snapshot/owner/operation hashes and UTC.
All evidence remains. The same layout can then admit legacy writers only while
that audit still matches. A retired generation cannot bind or start again;
state deletion or process absence is never retirement. New train generation
installation in a retired layout is deliberately unsupported; it needs a
separately reviewed succession protocol. There is no retention policy or GC.


## 4. Later

- Separate authorized real SSH/systemd/heavy validation and reviewed activation
  on dedicated hetzner8 and hetzner2; production remains OFF until then.
- Same-boot containment recovery for a lost supervisor, incomplete preparation
  reconciliation, and a reviewed generation succession/runtime upgrade protocol.
- Reference-aware pin/artifact retention and GC without weakening ownership.
- Partitioned full checks and learning selection gaps from full runs.
