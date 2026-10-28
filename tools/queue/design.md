# The merge queue

Provisional, 2026-10-28. How a branch reaches `main`: a script on the gate machine batches ready branches, checks them once, and pushes. No agent merges.

## 1. In one page

- **Input:** remote branches `ready/<lead>/<task>` on `origin`, each pushed by its lead.
- **A batch:** from `origin/main`, the ready branches not yet tried at their current tip, oldest tip first, at most `batch` (8) of them, each merged with `git merge --no-ff -m "Merge <branch name>"`; ratchet-only conflicts are resolved by recounting against HEAD (section 3.3); any other conflict is aborted, reported, and left out.
- **The checks run on a selection** (section 2), not always the whole workspace: `cargo fmt --check`, clippy with `-D warnings`, a nextest `build`, the focused (`default`) and `fuzzy` nextest profiles, and each tree's `scripts/old_forms.py` (the ratchet). Build and test commands run through `heavy --gate` on the gate machine, which nothing else builds on. Lockfile and member manifest changes select affected packages (section 2.1); formatting covers changed Rust owners (section 2.2). Build and test durations are recorded separately (section 3.6).
- **Green:** `git push origin HEAD:main` (fast-forward only); the batch's `ready/` branches are deleted; each lead is told which commit holds its task.
- **Red:** a batch of one is rejected (its lead gets the failing step and the log's path, and the branch isn't retried until its tip changes); a larger batch is halved and each half tried from `main`, recursively, so the green branches still land.
- **Capacity refusal:** a heavy exit 2 containing `below the disk floor` is an error, with no rejection or bisection. Before every batch, free space below `--disk-floor` (45 GB by default) defers work to the next cycle.
- **Error:** an exception logs the batch as `error`; its tips remain eligible. `run` restores `origin/main`, reports each distinct error to the coordinator once, waits a cycle, and continues. `once` raises to its caller.
- **Docs-only batches** whose files no test reads skip formatting and builds; only the touched trees' ratchets run.
- **Every batch is logged** as one JSON line (`~/src/fabrig/notes/queue.jsonl`): start and end, branches, outcome, the selection's packages, each step's duration, and test executable cleanup counts and GB.

## 2. The selection

1. **Changed paths:** `git diff --name-only origin/main...HEAD` for the batch.
2. **Everything** when a changed path is outside every package's `src/`, `tests/`, `benches/`, `examples/`, `build.rs` and member `Cargo.toml`, except the root `Cargo.lock` (section 2.1). The root `Cargo.toml`, `.config/`, `rust-toolchain*`, `rustfmt.toml`, `clippy.toml`, scripts, docs read by tests, and fixtures force everything. Also run everything every `full_every` (5) batches, when the last full run is older than an hour, or when the gate machine has been idle for 10 minutes and main has changed since the last full run.
3. **Otherwise,** the packages that own a changed path (the longest manifest directory that prefixes it, from `cargo metadata --no-deps`), their reverse dependencies' closure (normal, build and dev dependencies, from `cargo metadata`'s resolve graph), and the packages listed in `tools/queue/always.txt` (tests that read files by path).
4. **Docs read by tests** are globs in `tools/queue/read-by-tests.txt`; a Markdown-only batch that matches none of them is docs-only (section 1).
5. A full run that fails where the selections before it passed points at a gap in `always.txt` or `read-by-tests.txt`: the queue reports it to the coordinator.

### 2.1 Lockfiles and member manifests

A member's `Cargo.toml` selects its owner and reverse dependencies just as a
source change does. For `Cargo.lock`, read `origin/main:Cargo.lock` and
`HEAD:Cargo.lock` with `git show`, without checking out the base. Compare
entries by name, version, source, and sorted dependencies; checksums and entry
order do not select packages. Added and removed entries count as changes.
Match changed entries to the batch HEAD's resolved package IDs by name,
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
branches tried. Recovery diagnostics and coordinator delivery are logged as a
separate `event: "recovery"` record; errors outside an attempt get their own
error record. Distinct error texts already reported to `coord` in `leads.txt`
are kept in state, so a repeated error does not post every cycle. Recovery
failures are logged and the loop waits 60 seconds before trying again. If
recovery itself cannot append its record, it reports that failure on stderr
and keeps the loop running.

### 3.2 Capacity refusals

Space is measured on the queue checkout's filesystem in decimal GB. Before
every attempt, including bisection halves and idle full checks, less than
`--disk-floor` GB raises a capacity error without running a batch. `run`
posts the coordinator with the host and free GB once for that host's queue
floor refusal, then waits for the next cycle. A heavy step that exits 2 and
prints `below the disk floor` also raises a capacity error, preserving the
step's output (including the builder's host and space), plus the queue host
and free GB. Other exit codes or messages remain red. No refused batch
marks tips tried, rejects a branch, or bisects. `once` surfaces these errors.

### 3.3 Ratchet conflicts

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
Ratchets follow tests. Docs-only batches run only touched trees' ratchets.

## 4. Later

- Two gate machines with nextest's `--partition` once the full run's time matters.
- Learning the selection's always-list from full runs' misses.
