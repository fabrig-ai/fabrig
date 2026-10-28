# The merge queue

Provisional, 2026-10-28. How a branch reaches `main`: a script on the gate machine batches ready branches, checks them once, and pushes. No agent merges.

## 1. In one page

- **Input:** remote branches `ready/<lead>/<task>` on `origin`, each pushed by its lead.
- **A batch:** from `origin/main`, the ready branches not yet tried at their current tip, oldest tip first, at most `batch` (8) of them, each merged with `git merge --no-ff --no-edit`; ratchet-only conflicts are resolved by recounting against HEAD (section 3.3); any other conflict is aborted, reported, and left out.
- **The checks run on a selection** (section 2), not always the whole workspace: `cargo fmt --check`, clippy with `-D warnings`, the focused (`default`) and `fuzzy` nextest profiles, and each tree's `scripts/old_forms.py` (the ratchet), all through `heavy --gate` on the gate machine, which nothing else builds on.
- **Green:** `git push origin HEAD:main` (fast-forward only); the batch's `ready/` branches are deleted; each lead is told which commit holds its task.
- **Red:** a batch of one is rejected (its lead gets the failing step and the log's path, and the branch isn't retried until its tip changes); a larger batch is halved and each half tried from `main`, recursively, so the green branches still land.
- **Capacity refusal:** a heavy exit 2 containing `below the disk floor` is an error, with no rejection or bisection. Before every batch, free space below `--disk-floor` (45 GB by default) defers work to the next cycle.
- **Error:** an exception logs the batch as `error`; its tips remain eligible. `run` restores `origin/main`, reports each distinct error to the coordinator once, waits a cycle, and continues. `once` raises to its caller.
- **Docs-only batches** whose files no test reads skip the build (formatting and the ratchet only).
- **Every batch is logged** as one JSON line (`~/src/fabrig/notes/queue.jsonl`): start and end, branches, outcome, the selection's packages, and each step's duration.

## 2. The selection

1. **Changed paths:** `git diff --name-only origin/main...HEAD` for the batch.
2. **Everything** when a changed path is outside every package's `src/`, `tests/`, `benches/`, `examples/` and `build.rs` (manifests, `Cargo.lock`, `.config/`, scripts, docs read by tests, fixtures), or every `full_every` (5) batches, or when the last full run is older than an hour, or when the gate machine has been idle for 10 minutes and main has changed since the last full run.
3. **Otherwise,** the packages that own a changed path (the longest manifest directory that prefixes it, from `cargo metadata --no-deps`), their reverse dependencies' closure (normal, build and dev dependencies, from `cargo metadata`'s resolve graph), and the packages listed in `tools/queue/always.txt` (tests that read files by path).
4. **Docs read by tests** are globs in `tools/queue/read-by-tests.txt`; a Markdown-only batch that matches none of them is docs-only (section 1).
5. A full run that fails where the selections before it passed points at a gap in `always.txt` or `read-by-tests.txt`: the queue reports it to the coordinator.

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
failures are logged and the loop waits 60 seconds before trying again.

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

## 4. Later

- Two gate machines with nextest's `--partition` once the full run's time matters.
- Learning the selection's always-list from full runs' misses.
