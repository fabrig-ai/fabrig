# Branch pre-ready check

Provisional, 2026-10-29. The lead owns a claimed, idle slot and runs
`precheck [--base origin/main] [--min-free-bytes INTEGER] SLOT TIP` before
marking its branch ready.
The source executable is `tools/queue/precheck`; `tools/install` installs
`precheck` from directory `queue`.

## 1. Meaning and selection

The lead fetches first. This tool never fetches: it resolves local BASE and
TIP once, to full commit objects using `--end-of-options`, and prints both.
It detaches BASE and merges TIP with `--no-ff --no-commit`, without moving
any original branch ref or making a commit. Already-contained tips have an
empty candidate diff. Logs include merge diagnostics.

Selection uses the merged index and worktree paths against BASE, captured
before writing ratchets. Generated lower baselines do not widen selection;
baseline changes supplied by the branch retain the queue's ordinary policy.
Ratchet-only conflicts retain their baseline paths for selection even when
choosing BASE and recounting leaves no baseline diff. Recounts still finish
before checks run. Metadata and formatting ownership come from the candidate;
Cargo.lock selection compares BASE's blob with the candidate file through
`mq.lock_changes`. `mq.selection`, its always/read-by-tests lists,
`mq.check_commands`, `mq.tools_commands`, `mq.ratchet_commands`, and
`mq.run_step` supply queue policy. Code and policy lists are loaded before
switching any slot, including the checkout containing the running tool.
Candidate suites and ratchet scripts come from the candidate. The queue's
graph-free docs/tools/full cases avoid unnecessary metadata requests.

Only ratchets, selected tools suites, fmt, clippy and nextest **build** run.
Only the queue's leading heavy `--gate` is removed; build scope and arguments
are retained. Every build uses ordinary `heavy cargo ...`. Nextest build
exit 4 passes. No default/fuzzy Rust tests execute. Tools suites retain the
queue's existing timeout, interpreter/shebang discovery, and exact exclusion
of `tools/heavy/test-heavy-recovery`; every discovered suite runs after an
ordinary suite failure, before stopping Cargo checks. No new budget or
command timeout is introduced.

Tools suites must keep their Git operations and temporary outputs outside
the caller's checkout. `test_mq.py` uses disposable sibling repositories,
including when exercising real ratchet stage/commit behavior.
`test_precheck_tools_isolation.py` runs that suite against an uncommitted
merge with a staged candidate and an unstaged lower baseline, compares the
caller's exact index, HEAD, refs, merge state and tracked contents/modes,
and verifies precheck restoration after the real suite finishes.

Immediately before each heavy clippy or nextest build command, the helper
checks `shutil.disk_usage(SLOT).free >= --min-free-bytes`. The default is
80000000000 bytes, enforcing Ops' temporary night admission policy. The
value must be a positive integer; zero, negative, nonfinite and malformed
values refuse before any checkout operation. Ops can supply a different
explicit value when that policy changes. This is helper scheduling: heavy's
own 40 GB floor and the queue's 45 GB floor, caps and timeouts are unchanged.
Fmt, metadata and docs-only checks need no build admission. A refusal
defers the command without launching heavy, sleeping, retrying, lowering
the threshold, cancelling work or changing heavy state. It records the
chosen threshold and observed free bytes in the command log and result.
Earlier completed checks retain their results, but the run reports an
infrastructure error, restores through the same lifetime protocol and
cannot certify readiness.

Exit 0 means all requested steps passed and restoration was verified. A
docs-only candidate can pass with no commands. This is a compile/lint check,
including tools tests when selected; it promises neither test execution nor
success against a later main or a queue batch. There is no periodic full-run
schedule here. Launch/metadata errors, capacity refusals, incomplete plans
and unknown completion are infrastructure failures, never success.

## 2. Ownership and preparation

SLOT names the checkout root. Symlink path components, Git environment
location overrides, dirty/staged/untracked files, hidden index flags, sparse
checkouts, submodules, and merge/rebase/cherry-pick/revert/sequencer/bisect
state are refused before source mutation. Linked Git worktrees are supported.
Git must support `merge-tree --write-tree` (Git 2.38 or later): a merged-tree
preview detects ignored-file collisions, including paths introduced by
directory renames. Source preparation still uses the ordinary uncommitted
merge. Ignored targets and unrelated files are never cleaned or trimmed.

Ownership holds nonblocking flocks for the slot's sibling `.queue.lock` and
`.worker.lock`, and its Git directory's `worker.lock` and `precheck.lock`,
through restoration. These coordinate with the queue's existing lock and
workers using those paths; they cannot fence an uncooperative writer. The
lead must own the slot and have no concurrent task writer. Durable
`mq-worker-unresolved.json` (the queue worker's existing marker),
`precheck.unresolved`, `worker.unresolved`, or `queue.unresolved` markers in
the Git directory, or sibling `<slot>.<kind>.unresolved`, refuse reuse,
including dangling symlinks. Refusal precedes preparation, logging and
restoration; existing markers are never removed or migrated.
No lock is reclaimed by PID. When a user systemd bus is available, active
heavy services' WorkingDirectory values are checked using the slot guard's
unit naming convention. Listing requests `--type=service`; non-`.service`
names are ignored defensively, so reconcile timers need no WorkingDirectory.
A service collected between listing and inspection is ignored only after
a successful show explicitly reports `LoadState=not-found` and
`ActiveState=inactive`; this observation is recorded, not treated as build
completion. Failed inspection, missing/unknown properties, or a potentially
running service without an absolute WorkingDirectory fail closed. A stopping
service in the slot still blocks reuse. An unavailable bus is recorded
explicitly, not reported as proof of heavy quiescence. No heavy state or
process environment is read.

Before mutation, a private, fsynced `precheck.unresolved` in the worktree's
Git directory points at the durable run log and records original branch or
detachment, HEAD, branch refs, pinned inputs and ownership. This marker is
not a daemon or a slot claim protocol. A hard death leaves it behind.

For conflicts confined to the four old-form baseline files, choose BASE's
versions in the index and recount touched trees against those versions.
Preserve conflicted paths for recount even when choosing BASE leaves no
diff. Recounts may lower working baselines; a raise fails. They neither
stage their outputs nor commit. Every ordinary touched-tree ratchet also
runs before planning. Other conflicts fail with diagnostics and restore.

## 3. Evidence and restoration

Each run uses a 0700 sibling `<slot>.precheck-logs/run-*` directory, outside
the source checkout. Files are 0600: `commands.log`, `result.json`,
`original.index`, `original.worktree.json` (raw file/type/mode hashes), and
`original.files/` (checksummed raw contents of tracked regular files that
preparation/restoration can rewrite, including baseline files).
Results include pinned inputs, paths, selection, planned commands, argv,
step return codes, durations, heavy observation, outcome and restoration.
`selection_paths` records candidate paths captured before recounts,
including reconciled baseline conflict paths; `paths` retains the complete
post-recount candidate diff against BASE, including generated baselines.
The latter remains available to consumers verifying candidate freshness.
`min_free_bytes` records the chosen threshold; `admissions` records each
heavy command's name, `free_bytes`, `min_free_bytes` and `admitted` decision.
A deferred command remains in the plan without a completed step or an
unknown active-command record of its own.
The active command is durably marked unknown before launch and completed
only after its child exits. All execution uses argv and `/dev/null` stdin.

Normal success, failed steps, launch errors and ordinary merge conflicts
restore original HEAD and branch/detachment, verify all local branch refs,
clean worktree/index and absence of merge state, compare raw tracked contents
and modes, restore saved raw contents and permissions (including clean CRLF
files and modes affected by the private umask), and replace the index with
its exact original bytes. The marker
is removed and its directory fsynced only after verification and known
command completion. Logs, targets and ignored files stay. A restoration
failure makes the result nonzero and retains the marker. An unexpected
commit, ref movement, index mutation, new untracked file or tracked content
change outside the prepared plan retains the candidate for reconciliation.
Ratchets may change only their own baseline's working file.

## 4. Interrupted work and reconciliation

INT/TERM/HUP record interruption. Commands run in a separate session so the
known child can drain without forwarding the signal to heavy. No reset or
checkout happens while that child is running. A drained, normally exited
client allows restoration and reports interruption as infrastructure error.
A negative or signal-style heavy client exit (including 137), a launch with
unknown outcome, or a wait exception retains candidate inputs and durable
ownership. Client disappearance alone never establishes build completion.

Do not reuse, reset or remove the marker in an unresolved slot. From its
recorded submitting directory, inspect status with heavy's supported interface
and cancel with `heavy --cancel TOKEN` when needed; the operator uses
`heavy --check-recovery` / `heavy --reconcile` where required. Never kill
heavy units or edit heavy state. Establish completion first, then restore
the recorded original HEAD and index and verify clean source and original
branch refs before removing `precheck.unresolved`. There is deliberately no
automatic recovery/force option. A worker's `mq-worker-unresolved.json`
belongs to its separate operational reconciliation: leave it intact, and
do not use precheck restoration or process absence to clear that ownership.

## 5. Before-ready freshness

`ready-check RESULT --repo REPO --tip EXACTTIP` is a standalone check of
owner-supplied precheck evidence. `tools/install` installs its executable
link from `queue`. RESULT is the completed `result.json`; REPO is the lead's
candidate checkout, never the dedicated primary clone. EXACTTIP must be the
full lowercase commit OID, with no revision expressions or abbreviated IDs.
The check runs no precheck, build, test, push, reset, merge or checkout.
It preserves HEAD, local and remote branch refs, source files and index bytes.
It downloads Git objects with a destination-free fetch, an empty refmap,
no tags, no submodule recursion and no FETCH_HEAD write. Hooks are disabled
for its Git commands. It accepts dirty candidate sources because all decisions
use pinned commit objects and the completed report, never working contents.

The evidence must say `outcome=passed` and `restored=true`, contain full
base/tip/original OIDs, the slot, a timezone-bearing start time, a heavy
observation, exact literal file paths, selection, plan and completed steps.
Duplicate JSON keys, duplicate or unsafe file paths, incomplete records,
errors or unresolved evidence, unsuccessful steps and disagreement between
planned and completed commands (including executed working directories)
are refused. Completed ratchet steps precede
the plan, with a completion for each touched tree. A nonempty compile/lint
selection needs ordinary-heavy clippy and test-build commands. Build exit 4
is accepted as in precheck; every other accepted step exited 0. The last
active command must explicitly be completed, match the last step, its cwd
and return code. A run with no commands has no active command and requires
an empty/docs selection. Older or injected reports that cannot establish
these inputs require fresh usable evidence. No arbitrary age limit is added:
freshness is evaluated against pinned commits and remote history.

The report is owner-supplied evidence, not authentication. Its `paths` list
is the exact reported touched-file scope, not a crate or dependency closure;
ready-check validates its shape and consistency but does not recreate the
uncommitted merge, ratchet outputs or command selection to authenticate it.
A lead must use the unedited report from the exact-tip precheck. This check
proves no compatibility with semantic changes in other files.

The check observes `origin`'s `refs/heads/main` with `ls-remote`,
fetches its objects, verifies the observed full commit and requires the
checked base to be its ancestor. Missing objects, unavailable refs, shallow
history, replace refs and Git location overrides fail closed. It collects
literal filenames with NUL-delimited `diff-tree` output for every commit
since the checked base, comparing merges against each parent. Both sides of
renames, deletions, merge resolutions and changes later reverted count;
this can conservatively require a new precheck even when final trees match.
Any intersection with reported paths requires another precheck. A second
remote observation after comparison must equal the first. An advance or
rewrite observed during the check is a refusal, including a disjoint advance.

Stdout is one JSON decision with `verdict` (`passed` or `rerun-precheck`),
`tip`, `checkedbase`, `currentmain`, `touchedscope`, and `diagnostic`. Once
known, it also includes `overlap` and the second observation `finalmain`.
Unavailable values are null. Exit 0 means passed; a refusal exits nonzero
and writes a human diagnostic to stderr naming tip, precheck base and current
main as known, and concrete overlapping filenames when present. The lead
may redirect stdout to a private verification record outside source.

Immediately before publishing readiness, pin the candidate OID and use:

```sh
ready-check RESULT --repo REPO --tip "$exact_tip" &&
    git -C REPO push origin "$exact_tip:refs/heads/ready/lead/task"
```

`exact_tip` is the full OID supplied to the successful precheck, not a moving
branch expression. A refusal means fetch current main, rerun precheck of
that exact tip in the owned idle slot, and retry this check with its new
report. A pass is bound to `currentmain`, the exact remote OID observed twice.
It cannot fence origin: main may advance after the final observation, before
return, or before/after the ready push. A transient advance and return to the
same OID between observations also cannot be detected. The queue gate remains
the final check of the eventual combined candidate. This tool publishes no
refs; the lead owns the conditional exact-tip ready push.

## 6. Later

- Builder validation and fleet installation by Ops after review/landing.
- Coordination with a future common checkout ownership protocol, if workers
  adopt a different lock/marker convention. Noncooperating writers cannot
  be excluded by this tool.
- Durable heavy receipts could establish completion after a lost client;
  this tool currently retains ownership for operator reconciliation.
