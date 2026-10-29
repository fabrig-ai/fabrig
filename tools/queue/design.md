# The merge queue

Provisional, 2026-10-28. How a branch reaches `main`: a script on the gate machine batches ready branches, checks them once, and pushes. No agent merges.

## 1. In one page

- **Input:** remote branches `ready/<lead>/<task>` on `origin`, each pushed by its lead.
- **A batch:** from `origin/main`, the ready branches not yet tried at their current tip, oldest tip first, at most `batch` (8) of them, each merged with `git merge --no-ff -m "Merge <branch name>"`; ratchet-only conflicts are resolved by recounting against HEAD (section 3.3); any other conflict is aborted, reported, and left out. After merging, the queue writes and commits lower old-form baselines for touched trees before checks. Branches need not change those baselines.
- **The checks run on a selection** (section 2), not always the whole workspace: touched trees' `scripts/old_forms.py --write` (the ratchet, section 3.3), then the tools' suites (section 2.3), `cargo fmt --check`, clippy with `-D warnings`, a nextest `build`, and the focused (`default`) and `fuzzy` nextest profiles. Cargo build and test commands run through `heavy --gate` on the gate machine, which nothing else builds on. Lockfile and member manifest changes select affected packages (section 2.1); formatting covers changed Rust owners (section 2.2). Step durations are recorded separately (section 3.6).
- **Green:** `git push origin HEAD:main` (fast-forward only); the batch's `ready/` branches are deleted; each lead is told which commit holds its task.
- **Red:** a batch of one is rejected (its lead gets the failing step and the log's path, and the branch isn't retried until its tip changes); a larger batch is halved and each half tried from `main`, recursively, so the green branches still land.
- **Capacity refusal:** a heavy exit 2 containing `below the disk floor` is an error, with no rejection or bisection. Before every batch, free space below `--disk-floor` (45 GB by default) defers work to the next cycle.
- **Error:** an exception logs the batch as `error`; its tips remain eligible. `run` restores `origin/main`, reports each distinct error to the operator (`ops`) once, waits a cycle, and continues. `once` raises to its caller.
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
are kept in state, so a repeated error does not post every cycle. Recovery
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

Restart finds the queue lock's holder through `/proc/locks`, falling back
to `pgrep -f` for exactly one `tools/queue/mq run`. An absent or ambiguous
process is an error. It writes `~/src/fabrig/queue.stop`, prints the pid and
stop path, and waits for that process to exit, using a pidfd so pid reuse
cannot change the wait. `--wait` bounds this wait in seconds (default 1800).
A timeout retains the stop file and does not fetch or start a replacement.

After the process exits, restart fetches origin and switches the clone to
detached `origin/main`. Only then does it remove the stop file and start
`python3 -u tools/queue/mq run --repo <clone>` in a new session with stdin
from `/dev/null` and both outputs appended to
`~/src/fabrig/notes/queue.out` (0600). After three seconds it checks for an
early exit, then prints the new pid and main's commit. `--repo` defaults to
the current directory and must name the clone root. Run only one restart
at a time; a failed update leaves the stop file for inspection.

`restart --batch N` sets the replacement queue's positive batch size; the
default remains 8. Pass the option on each restart to retain a nondefault
size. It changes batch capacity, not checks, rejection or bisection policy.

### 3.8 Local immutable-candidate worker (disabled rollout)

`worker.py` is an opt-in, synchronous local entry point and a review proposal
for the request/result contract. It is not installed, dispatched by `mq`, or
started by `restart`. Ops review, coordinator contract review and a dedicated
gate validation precede activation. Existing one-gate behavior remains the
default. No second prover is activated by this code.

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

There is deliberately no automatic reconciliation command. Ops must establish
heavy job completion through its supported recovery procedure, inspect the
candidate/worktree and result/log evidence, then explicitly reconcile the
ownership marker before reusing this dedicated checkout. Do not remove heavy
state or stop its units. Do not run legacy queue/restart or any other checkout
writer on a worker clone with unresolved ownership: persistent controller-wide
startup guards are a later obligation. The flock is cooperative ownership,
not a sandbox against arbitrary external writers or hostile candidate code.

The worker never fetches/pushes origin, posts, changes queue counters, deletes
ready refs, or promotes a candidate. Transport, durable controller state,
reservations and full-check cadence, result/log ingestion and validation,
merge-chain evidence, ordered adjudication, promotion, notification recovery,
conditional ready cleanup, supervised remote job lifetime, controller startup
guards, draining and train-aware restart remain unimplemented. This proposal
cannot activate or safely supervise a merge train by itself.

## 4. Later

- A merge-train controller and supervised transport around the opt-in local
  worker, with pinned objects, independent result validation, durable decisions,
  ordered promotion, ownership recovery guards and train-aware restart.
- Two gate machines with nextest's `--partition` once the full run's time matters.
- Learning the selection's always-list from full runs' misses.
