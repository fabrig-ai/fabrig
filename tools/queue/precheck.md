# Branch pre-ready check

Provisional, 2026-10-29. The lead owns a claimed, idle slot and runs
`precheck [--base origin/main] SLOT TIP` before marking its branch ready.
The source executable is `tools/queue/precheck`; `tools/install` installs
`precheck` from directory `queue`.

## 1. Meaning and selection

The lead fetches first. This tool never fetches: it resolves local BASE and
TIP once, to full commit objects using `--end-of-options`, and prints both.
It detaches BASE and merges TIP with `--no-ff --no-commit`, without moving
any original branch ref or making a commit. Already-contained tips have an
empty candidate diff. Logs include merge diagnostics.

Selection uses the merged index and worktree against BASE, after writing
ratchets. Metadata and formatting ownership come from this candidate;
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

## 5. Later

- Builder validation and fleet installation by Ops after review/landing.
- Coordination with a future common checkout ownership protocol, if workers
  adopt a different lock/marker convention. Noncooperating writers cannot
  be excluded by this tool.
- Durable heavy receipts could establish completion after a lost client;
  this tool currently retains ownership for operator reconciliation.
