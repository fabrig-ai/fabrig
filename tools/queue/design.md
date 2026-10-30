# The merge queue

Provisional, 2026-10-28. How a branch reaches `main`: a script on the gate machine batches ready branches, checks them once, and pushes. No agent merges.

## 1. In one page

- **Input:** remote branches `ready/<lead>/<task>` on `origin`, each pushed by its lead.
- **A batch:** from `origin/main`, the ready branches not yet tried at their current tip, oldest tip first, at most `batch` (8) of them, each merged with `git merge --no-ff -m "Merge <branch name>"`; ratchet-only conflicts are resolved by recounting against HEAD (section 3.3); any other conflict is aborted, reported, and left out. After merging, the queue writes and commits lower old-form baselines for touched trees before checks. Branches need not change those baselines.
- **The checks run on a selection** (section 2), not always the whole workspace: touched trees' `scripts/old_forms.py --write` (the ratchet, section 3.3), then `cargo fmt --check`, clippy with `-D warnings`, a nextest `build`, and the focused (`default`) and `fuzzy` nextest profiles. The gate skips pipeline tools suites (section 2.3). Cargo build and test commands run through `heavy --gate` on the gate machine, which nothing else builds on. Lockfile and member manifest changes select affected packages (section 2.1); formatting covers changed Rust owners (section 2.2). Step durations are recorded separately (section 3.6).
- **Green:** save the checked candidate and coordinator routes in a durable publication intent, push that saved candidate (fast-forward only), confirm containment and the exact journal row, then delete ready branches under expected-tip leases and notify leads (section 3.11).
- **Red:** a batch of one is rejected (its lead gets the failing step and the log's path, and the branch isn't retried until its tip changes); a larger batch is halved and each half tried from `main`, recursively, so the green branches still land.
- **Capacity refusal:** a heavy exit 2 containing `below the disk floor` is an error, with no rejection or bisection. Before every batch, free space below `--disk-floor` (45 GB by default) defers work to the next cycle.
- **Error:** an exception logs the batch as `error`; its tips remain eligible. `run` reconciles pending publications before restoring `origin/main`, reports each distinct error to the operator (`ops`) once per uninterrupted error episode, waits a cycle, and continues. A completed cycle, including an idle check, clears the persisted episode so recurrence alerts again. `once` raises to its caller.
- **Batches selecting no packages** skip Cargo checks; touched trees run their ratchets. Tools changes require author checks before Ready (section 2.3). Unread Markdown outside package source directories selects no package, even beside code or tools changes.
- **Every batch is logged** in `~/src/fabrig/notes/queue.jsonl`: start and end, branches, outcome, selection and step durations. A published batch has one immutable publication row and subsequent `event` diagnostics for notifications and cleanup (section 3.11). Other attempts retain their ordinary row.

## 2. The selection

1. **Changed paths:** compare the checked parent and candidate trees with `git diff --name-only --no-renames -z <parent> <candidate>`. `changed_paths` accepts an explicit `base` and `candidate`, defaulting to `origin/main` and `HEAD` for the one-gate queue. Include both sides of renames and deleted paths. The unmerged `plan` uses `preview_changed_paths` instead: compare each ready tip with its merge base with `origin/main`, preserving `origin/main...<tip>` preview semantics.
2. **Everything** when a changed path is outside every package's `src/`, `tests/`, `benches/`, `examples/`, `build.rs` and member `Cargo.toml`, except the root `Cargo.lock` (section 2.1), `tools/`, and unread Markdown (`*.md`). The root `Cargo.toml`, `.config/`, `rust-toolchain*`, `rustfmt.toml`, `clippy.toml`, docs read by tests, and other scripts and fixtures force everything. Also run everything every `full_every` (5) batches, when the last full run is older than an hour, or when the gate machine has been idle for 10 minutes and main has changed since the last full run.
3. **Otherwise,** the packages that own a changed path (the longest manifest directory that prefixes it, from `cargo metadata --no-deps`), their reverse dependencies' closure (normal, build and dev dependencies, from `cargo metadata`'s resolve graph), and the packages listed in `tools/queue/always.txt` (tests that read files by path).
4. **Docs read by tests** are globs in `tools/queue/read-by-tests.txt`; a matching path forces everything. A path under `tools/` contributes no package. Otherwise package source ownership still selects its package, including Markdown under `src/`, `tests/`, `benches/` and `examples/`. Unread Markdown outside those directories contributes no package. A batch with no package inputs and no full input selects `docs`: no Cargo checks or `always.txt` packages, plus touched trees' ratchets as applicable.
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

Decision0022 disables pipeline tools suites in every merge-gate batch,
including tools-changing, periodic, idle and forced repair full runs.
The shared `mq.gate_tools_required` predicate governs the gate runner and
worker's exact plan. Its policy bytes live in `mq`, frozen by both worker
runtime identity and restart's external-runtime validation. Ordinary ratchet,
fmt, clippy, build, default and fuzzy selection remains unchanged.

Authors run affected tool tests before Ready. Author precheck retains
`mq.tools_required` as its compatibility predicate: changed `tools/` paths
require author checks independently of Cargo scope. `mq.author_tools_commands`
is the deterministic owner selector. Each path under `tools/<tool>/` selects
only tracked candidate suites under that directory, including new tools with
suites. Adds, edits and deletes count; both rename sides select their owners.
Multiple tools select the sorted, deduplicated union. Discovery queries only
those literal owner paths, never all unrelated suites.

The top-level installer, manifest and installer suite (`tools/install`,
`tools/installed`, `tools/test_install.py`) select only `tools/test_install.py`.
Other top-level paths refuse with an unknown-owner diagnostic. An owner with
no eligible candidate suite also refuses, including deletion of its last
suite or whole directory; the author must supply an explicit safe mapping
rather than receive an empty success. A rename across owners requires suites
for both owners. These refusals restore through the ordinary precheck protocol
and cannot certify readiness.

The following discovery and execution rules apply within selected owners.
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
60-second timeout per suite. Record each suite's command, output, exit code
and elapsed time in the author check evidence. Run every selected suite even if one
fails; any nonzero exit, missing interpreter, or timeout makes the author
tools check red and stops later checks.

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
tree, in tree-name order, before Cargo checks. Each command is step
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
Ratchets run before Cargo checks (section 3.3). Docs-only batches
run only touched trees' writing ratchets.

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

#### 3.7.1 Pinned gate repair priority

Ops may name one branch repairing a recurring gate failure with the paired
options `--repair-ready ready/<lead>/<task> --repair-tip <full lowercase OID>
--repair-reason <printable nonempty text>` on `run`, `once`, or `plan`.
All three are required together; Git validates the full ready ref before
checkout mutation. There is no priority inferred from branch names or files.
After fetching, selection requires the exact name and tip both in fetched
ready refs and advertised by origin. A missing or moved pin refuses the cycle;
it never falls through to ordinary work. `plan` previews the singleton as full.

The repair runs first as a singleton through the ordinary attempt path with
`force_full=True`: writing ratchets, workspace fmt, clippy, build, default and
fuzzy checks remain unchanged; pipeline tools suites are skipped
(section 2.3). Ordinary merge
conflicts, red rejection, fast-forward-only main push, pinned cleanup and
notifications apply. Exceptions, capacity refusals and failed process launches
are errors, preserving tried tips without bisection or rejection. A completed
attempt consumes the priority; the existing durable tried name/tip suppresses
the completed repair after restart, including after its ready ref is deleted.
Normal batch capacity and full-check counters/cadence then resume. Use explicit
`--batch 16` for the approved operational batch capacity; the default remains 8.
No successor is selected before the repair attempt completes.

For a repair, a heavy client returning a negative signal status or 137 leaves
completion **unknown**, never red or consumed. A per-attempt status observer
blocks ownership in memory and publishes both receipts before log processing
or checker return. It uses the known prepared candidate without another Git
read; log I/O, decoding, checker and receipt-publication exceptions cannot
authorize cleanup or recovery. Failed receipt encoding also retains ownership
until publication can be retried. The queue retains the candidate
and records the exact repair, parent, candidate, steps, log and runtime location
in immutable private `queue.unresolved` receipts in the shared authority root
and checkout's `.git/`. Shared authority and checkout guards refuse reuse,
including another clone or a worker. There is no recovery/reset, pruning,
bisection or further admission. A daemon retains both leases, checking stop at
the existing 60-second cadence. It honors stop only with both receipts durable;
publication failure retains ownership and retries publication at that cadence.
Stopping is not job-completion evidence: the receipts still block restart.
There is no supported unknown-heavy reconciliation procedure or clear switch.
Process absence, stop, a released flock or a held daemon cannot establish
quiescence. Leave the queue blocked until supported completion evidence and
reviewed reconciliation exist; do not remove receipts to resume work. A dry
attempt also preserves unresolved inputs rather than restoring its checkout.
This does not provide part 5's supervision or arbitrary crash durability.

`--repair-hold` requires the complete triplet on `run`, `once`, `plan` and
`restart`. After that repair's green/red/conflict adjudication, or when its
exact name/tip is already tried, it admits no ordinary, replacement or idle
attempt and performs no further fetch. The daemon stays alive with authority
held, using the existing idle/stop cadence. One `event: "repair-hold"` policy
record per invocation carries the repair identity and reason; it has no batch
outcome and certifies no completion. Ops requests the next official restart
with the next explicit repair and hold; an official restart without repair or
hold resumes ordinary batches (explicit `--batch 16`). Without hold the normal
post-repair behavior is unchanged. Unknown-heavy ownership is a separate blocker
and never enters this adjudicated hold.

Attempt records add a `repair` object containing `name`, `tip` and `reason` to
the existing external `queue.jsonl` stream. Conflicts and errors retain it;
configuration or selection refusals use `event: "repair-refusal", outcome: "error"`.
No additional persistent state format or in-checkout audit file is introduced.

`restart` forwards the same triplet and explicit batch size. Its optional
`--runtime DIRECTORY` selects a complete queue runtime directory outside the
live checkout, containing regular `mq`, `restart`, `worker.py`, `train.py`,
`authority.py`, `recovery.py`, `durable.py`, `publication.py`, `always.txt`,
`read-by-tests.txt`, `leads.txt` and `coordinators.txt` files. This manifest covers all imported policy/effect modules
and lists, matching the worker's frozen runtime. It validates Python syntax,
repair and shared-authority support, lists and the advertised pin before
requesting stop. Missing or changed files during drain refuse before checkout
update; the manifest is checked again before launch. The replacement loads
its code, authority dependencies and lists from that directory, with bytecode
writes disabled, while operating on `--repo`. Without `--runtime`, the installed
path stays unchanged. Section 3.10's shared lock, persistent startup guards,
dual-lock effect inheritance and acknowledged same-inode handoff apply to the
external runtime too. The pidfd drain and 1800-second default wait remain in
use. A failed launch restores stop. Stop is observed between whole bisection
cycles; restart cannot interrupt heavy.
The external runtime is an interim Ops choice until this option is installed.

Restart reads the private adjacent queue state to distinguish pending and
consumed name/tip pairs, then revalidates under shared authority and checkout
exclusion after drain and before update. Consumed repairs require no ready ref
(cleanup may delete it or a new tip may replace it); pending repairs still
require the exact advertised pin at both checks. Malformed, duplicate-key,
nonfinite, incompatible or untrusted state refuses, with no normal fallback.
No consumption state field or outcome vocabulary changes. PATH preflight,
plan fetch and other triplet refusals retain the exact identity/reason in the
external audit; audit failure cannot return success.

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
| `plan` | Complete ordered plan from `check_plan`: command entries `{name, command, cwd}`. Decision0022 selects no tools entry. |

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

Checks retain the same order and gate tools skip policy from section 2.3,
nextest exit-4 interpretation and serial
`heavy --gate` commands from sections 2.3 and 3.6. The small shared
`run_checks(check_only=True)` mode omits ratchets' `--write` and never stages
or commits a baseline. Its default remains the one-gate writing/committing
behavior. Candidate preparation must lower baselines before dispatch; the
read-only invocation uses each candidate ratchet script's existing comparison
semantics, including lower-count advisory output. It cannot certify a
rewritten replacement commit. The same candidate metadata supplies formatting
ownership, avoiding a second metadata graph during checking.

Runtime identity freezes the loaded bytes of `worker.py`, `mq`, `restart`,
`train.py`, `authority.py`, `recovery.py`, `durable.py`, `publication.py`,
`always.txt`, `read-by-tests.txt`, `leads.txt` and `coordinators.txt`, the Python major/minor/micro version, disk
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
ready refs, or promotes a candidate. Section 3.9 supplies a disabled controller
library around this contract. Production transport, authoritative effects,
shared startup guards, supervision, reconciliation tooling and train-aware
restart remain absent. The worker cannot activate or supervise a train alone.

### 3.9 Controller library (disabled, unready)

`train.py` has no CLI, production transport or authority implementation. Neither
`mq` nor `restart` imports it. Ordinary Queue behavior remains unchanged; both
live train gates remain off. **Production activation is prohibited** until the
authority/startup and recovery obligations below, followed by supervised
transport and rollout validation, are implemented and reviewed.

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
  part 4's only production adapter accepts local empty-plan jobs. Part 5 must
  supply supervised heavy/SSH evidence. Process/SSH absence, a terminal client
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
  groups remain durable. There is no production resume/restart integration.

Section 3.10 implements shared local authority and concrete effects. Part 5
supplies SSH/supervised job transport, operational drain/resume/restart and the
reviewed rollout. No local test asserts real heavy quiescence, real-service
authority, arbitrary machine-crash durability or remote transport safety.

### 3.10 Shared authority, effects and fail-closed recovery (train disabled)

The source adapters are production-capable primitives, with no executable
train mode, network job transport, second-prover activation or installation.
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
legacy requires explicit supported retirement in part 5, never deleting a
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
blocked until part 5 supplies supervised lifetime evidence. Tests inject an
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

### 3.11 Publication intent and coordinator feed

`tools/queue/coordinators.txt` is a separate required configuration file.
Its rows are `KEY COORDINATOR_ADDRESS`, with comments and blank lines;
keys use the same `[A-Za-z0-9_.-]+` grammar as lead keys and must be unique.
Addresses use the existing hosted-mailbox grammar, or exactly `-` for an
explicitly unassigned key. Malformed rows, controls, invalid addresses and
missing files refuse configuration. The coordinator and lead key sets need
not match. The checked-in coordinator file has no assignments; activation
requires reviewed addresses. No prefix, mission file or lead address supplies
an inferred coordinator. `leads.txt` remains two columns and retains its
notification destinations and sender.

Routes are flat fields on each merge entry. Resolved routes have exactly
`route_status="resolved"` and `coordinator=HOSTED_ADDRESS`. Unresolved routes
have exactly `route_status="unresolved"` and `route_reason="unassigned"` for
`-`, or `"missing_key"` when the key is absent. Omit the other state's field;
null, empty and sentinel coordinator values are invalid. The validator also
recognizes `historical_unknown` for separately reviewed backfill; ordinary
publication never generates that reason or changes historical journal rows.

Both worker and restart freeze the complete runtime list above, including
`publication.py` and exact coordinator bytes. Their identities include
`coordinators_sha256` as well as per-file digests. Restart compiles and validates
all source/list/route files before requesting stop, then checks their frozen
bytes after drain and before replacement launch. Default restart validates
`--repo`'s `tools/queue/` runtime and launches its validated absolute mq path, even
when the invoking script belongs to another directory. An explicit `--runtime`
binds validation, byte checks and launch to that directory instead. This validation admits no new
publication and does not replace saved intent routes. Plan and dry runs refuse pending publication/effect work without pushing or
cleaning refs; a nondry writer must reconcile it first. Queue startup reconciles
pending intents before loading coordinator configuration for new work; restart
reconciles under the same authority and checkout locks before fetch/reset.

For each successful nondry batch that has carrying merges, allocate a fresh
32-lowercase-hex publication identity once. Keep its evidence outside the
checkout at `<authority-root>/publications/PUBLICATION_ID/`, below the common
shared authority root of section 3.10. Directories are owned, nonsymlinked and
0700; files are owned regular nonsymlinked files, 0600. Retain this evidence:

- `coordinators.txt`: the exact loaded bytes, without reserialization.
- `intent.json`: the immutable version-1 publication intent.
- `journal-confirmation.json`: the durable exact-row receipt.
- `effects-confirmation.json`: version 1 and publication_id, marking completion
  of subsequent cleanup/notification work. This is not landing authority.

The intent has exactly `version` (integer 1), `publication_id`, validated
explicit credential-free `origin`, `ref` (`refs/heads/main`), `parent` (original
batch P0), `candidate` (checked C), ordered `merges`, `coordinators_sha256`,
`created` (fixed UTC), absolute `journal_path`, and complete `journal_row`.
OIDs are full lowercase 40- or 64-hex commit identifiers. Booleans cannot
satisfy integer fields. Each merge has `name`, pinned `tip` B, carrying `commit`
M, first `parent` P and the canonical route fields. Prove M's exact parents
are `[P,B]`, and that the entries cover the candidate interval's first-parent
carrying merges in order. Train preparation retains its contained-input evidence and stored chain shape;
that evidence is separate from the publication merge array. Record final
amended M after ratchet resolution;
baseline commits above M stay outside the array. A tip already reachable from
HEAD creates no carrying merge and no entry. A wholly already-reachable batch
is recorded as an `already-reachable` diagnostic and its pinned tips are marked
tried without a push; reachable inputs in a checked batch are consumed only
after its green result. Nested branch merges are below
B, and deletion of a ready ref does not remove the saved B/M evidence.

Under both locks, write/fsync coordinator bytes and commit intent through a
private sibling, fsyncing the file, publication directory and all newly created
containing directory entries. Only this durable boundary authorizes the first
push. Uncommitted staging cannot authorize publication. Push saved C to the
saved origin/ref, never HEAD. All Git children retain both lock descriptors,
so a surviving push excludes a replacement writer even after parent death.

Reconciliation fetches the saved explicit origin/ref and tests containment
of saved C, including when remote main advanced beyond it. Containment
confirms publication. Main still at P0 permits retry of that exact C under
the same guards; no rebuild, new identity, time or route lookup occurs.
An unavailable, unknown or divergent remote retains the intent and blocks
reset, replacement and new work; startup alerts the existing Ops destination.
The running loop retains its ordinary error-episode notification policy.

Only a confirmed published `outcome="green", dry=false` ordinary row carries
`publication_id`, `publication_time`, batch `parent` P0, `commit` C and the
exact saved `merges` array. Existing branches, selection, steps, posts and
conflicts retain their shapes. Red/conflict/dry/idle attempts and recovery or
policy diagnostics have no authoritative landing fields. Successful bisection
halves receive separate intents; the failed original batch is not a landing.

Append logically by replacing the journal through a fsynced private sibling,
preserving every previous byte. All writers, including refusal audits that
precede main-authority acquisition, serialize on the owned private sibling
`<journal>.lock` inode. This lock is never unlinked. Refuse incomplete tails, malformed/duplicate-key
JSON, nonobjects, duplicate publication identities and conflicting saved rows.
Both publication and audit writers check every publication identity in the
entire locked journal, including identities unrelated to the current intent.
The audit writer also validates the prospective appended row before replacement.
An identical complete row is replay, not another append. Fsync file and directory
before committing `journal-confirmation.json`, whose exact fields are integer
`version=1`, `publication_id`, `journal_path` and SHA-256 `row_sha256` of the
canonical saved row (sorted compact JSON with a final newline). A crash after
journal replacement but before confirmation revalidates the same row once.
Confirmation retires publication from pending; evidence remains retained.
Ready-ref cleanup follows confirmation with the existing expected-tip leases.
Its failures or later notification failures never revoke a landing. Interrupted
effects can replay until their confirmation; notification delivery keeps the
existing best-effort behavior and is not an exactly-once message outbox.

Operator metrics: the saved row's `end` and `publication_time` are the intent
observation, not the exact remote-push instant. They remain identical across
lost acknowledgements and UTC midnight. Push, post, target cleanup and ready
cleanup observations belong to later `event` rows (`publication-attempt`,
`publication-effects`, or `recovery`) that existing batch readers skip. Compare
step durations for check cost; publication-row elapsed time excludes later
effects. Diagnostic rows omit the authoritative publication_id/time/merges
fields and may refer to the intent with `publication` instead.

The offline command is `python3 -I -B tools/queue/test_publication.py`;
`test_mq.py`, repair, worker, authority and isolation suites exercise the
integration. Fixtures use private local origins and injected services; they
never run the live queue, a deployment or heavy recovery. Ordinary tools
discovery includes the publication suite with the unchanged 60-second bound.

## 4. Later

- CPU-per-suite policy is stage B of decision0016, owned by Green's shared
  schema, and is not implemented here. The launcher keeps its existing
  15-second CPU / 60-second wall budget and each tool suite keeps its
  60-second wall timeout; no CPU cap is inferred for other suites.

- Supervised SSH transport with stable job/incarnation identity, same-job
  start/status, verified bundles/results/logs, and lifetime evidence across
  client/controller/worker death and heavy recovery. Unknown ownership stays
  blocked; neither absence nor an SSH error is quiescence.
- Operational stop/drain/resume/restart and explicit generation retirement:
  retain configuration and sequential recovery, prove both gates quiescent,
  preserve stop on timeout, then permit legacy return/runtime updates.
- Authorized real-tool validation and reviewed rollout. Both train provers
  remain disabled; the executable train entry point belongs to part 5.
- Reference-aware pin/artifact retention and operator repair, preserving all
  immutable evidence until supported reconciliation permits removal.
- Two gate machines with nextest's `--partition` once the full run's time matters.
- Learning the selection's always-list from full runs' misses.
