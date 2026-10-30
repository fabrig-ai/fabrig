# Codex runs

Provisional, 2026-10-28. Detached task workers and persistent owners on the fleet.

## 1. In one page

- `codex-run task [OPTIONS] TASK DIR REPORT` reserves a report and starts a
  detached user unit. TASK, DIR and REPORT are absolute paths; REPORT must
  not exist, and its parent must exist. The task runs in DIR.
- `codex-run owner [--model M] [--effort E] NAME BRIEF` starts an owner that
  receives post between turns. BRIEF is an absolute readable file.
- `codex-run stale [--post]` lists unfinished inactive runs and refused
  completion messages; `--post` retries notification.
- Runs live under `~/src/rust/tmp/codex-run/`; `DEV_ROOT` replaces
  `~/src/rust`. Writable slot tasks trim their own slot before Codex starts.
- Dependencies are Bash, Python 3, jq, Perl, GNU coreutils, flock, systemd
  user units, Codex and `post` on PATH. Units start through a login shell.
  No adjacent agents checkout or sibling post executable is required.

## 2. Tasks and arguments

Task options are `--model M`, `--effort E` (medium, high or xhigh),
`--read-only`, `--to ADDRESS`, `--resume RUN`, and `--on-stop FILE`.
Defaults are `gpt-6.1-sol`, high effort, writable execution, and the recipient
in `POST_NAME`. Paths and model names must not contain tabs or newlines.

A resume names a previous run under the same run root. It reuses that
thread and inherits model, effort, read-only mode and working directory
unless supplied anew. With a resume, `TASK REPORT` can omit DIR.
A resume inherits its recipient unless `--to` or `POST_NAME` supplies one,
with `--to` taking precedence.
Each resume gets its own run directory and report. A thread missing from
its saved file can be recovered from its events.

Writable tasks use Codex's bypass-approval mode. Read-only tasks use its
read-only sandbox and return their complete report as their final message;
the worker publishes it outside the sandbox. Both receive the task path
and the restriction to search only their worktree and task-named paths.
Prompts reach Codex on stdin from a private file.

`--on-stop` records an absolute callback path for the unit's stop step.
A regular executable callback runs from its own directory with closed stdin,
logging stdout and stderr in `on-stop.log` and its result in `on-stop`.
It is killed after 60 seconds; absence and failure do not fail the unit.
Resume does not inherit the callback. Unsafe installation paths are refused
when building the systemd stop command.

## 3. Runs, reports and completion

Task launch prints `ID RUN_DIRECTORY`; the unit is `codex-run-ID`. The ID
combines a UTC timestamp and random suffix. The directory holds `meta`,
`prompt`, timestamped `events.jsonl`, `stderr.log`, `thread`, and `last.md`.
Directories are private and files are mode 0600 through umask 077.

Launch reserves REPORT exclusively. If the task leaves the reservation or
no report, completion publishes `last.md` or a diagnostic, retaining the
fallback at `report.md` in the run directory. Completion posts the report
path, run directory, thread, elapsed minutes and attempts to its recipient.
It appends settings, exit status, usage and post status to `runs.tsv` under
an advisory lock and writes `ended`. A failed completion step is recorded
in `finish-errors` without skipping later steps.

Capacity failures with a saved thread retry after 120, 300 and 900 seconds;
`CODEX_RUN_RETRY_DELAYS` overrides those delays. Other failures finish.
TERM and INT finish with their signal status. A killed worker can be
reported by `stale`; that command considers metadata aged at least two
minutes and less than one day. Post exits 0 and 3 count as delivered;
refused delivery remains retryable. Stale notification records `told`
without replaying report publication or accounting.

### 3.1 Reaper producer hooks

After saving a new task run's metadata and resume thread, the launcher calls
`reap record start --kind codex-run --owner OWNER --id ID --path RUN_DIRECTORY`.
OWNER is the launching `POST_NAME`, falling back to the recipient when unset.
Only that invocation's new run directory is registered. Task files, reports,
callbacks, slot checkouts and targets, primary clones, credentials, persistent
owner resume state, claims, leases and heavy state are outside its authority.
Arguments retain their boundaries, including paths with spaces.

Completion calls `reap record end --kind codex-run --id ID`, followed by
`systemctl --user start --no-block reap.service`. Without a callback this is
after accounting and `ended`, with the worker status known. With `--on-stop`
it is after `ended` and the callback result, including any attempt to mark a
failed callback's lease uncertain. A missing callback records `absent` first.
A failed launch closes its record before removing its private run directory
and report reservation. TERM, INT and worker failure use the same completion
path; a worker killed before `ended` emits no end. End records are idempotent.
An absent completion marker or stop result prevents the corresponding end.

These records describe run completion, never a landed or withdrawn task, nor
permission to reclaim a slot. Lease release remains the lifetime helper's
decision; failure and signals can retain protected inputs and uncertain holds.
Even then the sole registered path is the run directory. The reaper must
preserve metadata, shared `runs.tsv` accounting, pending resumes and sessions
referenced by an owner. It owns discovery, retention and deletion, including
older runs without producer records; missing records establish no deletion
authority. The launcher neither scans nor deletes retained evidence.

Each optional notification uses GNU `timeout` with the existing 60-second
stop bound and KILL signal, closed stdin and discarded output. Missing,
broken or hanging tools and units cannot change the caller's exit status,
report, delivery, accounting, callback result, lease decision or end marker.
The hooks can deploy before the `reap` CLI and unit, without a storage
migration or backfill by this producer. The reaper's deployment owns adoption.

Retention policy belongs to that deployment: runs and heavy logs seven days,
precheck evidence three days, landed or withdrawn notes and tasks seven days,
Codex sessions fourteen days, and ended Ops scratch three days. Heavy logs'
seven days takes precedence over generic old logs' three days. Keeps still
protect resumable and referenced evidence beyond these ages.

## 4. Slot trimming

Before its first Codex turn, a writable task resolves DIR and checks that
it is exactly `~/src/fabrig/slots/<n>`, with a numeric final component.
It runs `python3 -I <fabrig>/tools/slots/trim <slot>`, where fabrig is the
repository containing the resolved codex-run executable. This uses the
slot trim rules in `tools/slots/design.md, 2`.

The trim's one-line result and any diagnostic go to the run's `trim.log`.
A nonzero exit adds `trim failed: exit N` and the task continues. Missing
trim scripts are handled the same way. Read-only runs, owners, named
directories and directories nested inside a slot do not trim. `DEV_ROOT`
changes run storage, not the slot directory convention or the trim script.

## 5. Owners

Owner launch prints its escaped unit name. State lives under
`<root>/tmp/owners/<last-name-component>/codex/`, where root defaults to
`~/src/rust` and can be replaced by `DEV_ROOT`. A stored full owner name
refuses collisions. The owner registers its holder with post before its
first turn, reads BRIEF, then waits for mail for an hour between turns.
An idle hour prompts it to check its work. Failed turns wait five minutes;
pending mail is retained until a successful turn, including across restarts.
Replacement of the holder ends the loop. Turn accounting is in `turns.tsv`.
Neither a turn ending nor a holder replacement ends persistent owner state.
This producer emits no owner lifecycle record, including on final loop exit;
MC owns that lifecycle and its pending resume references.

## 6. Installation and verification

`tools/install [--dry-run]` uses its own repository's origin URL. It clones
into `~/src/fabrig/main` when absent, otherwise fetches that dedicated clone,
then switches it to detached `origin/main`. It refuses symlink checkouts,
dirty checkouts, different origins and directories inside another checkout.
The task's checkout and the agents repository are left in place.

`tools/installed` lists `TOOL DIRECTORY`: `codex-run codex-run` and `claim slots`.
The installer points `~/.local/bin/TOOL` at the corresponding executable in
the dedicated clone, printing each link's old and new target. Existing
non-symlink programs are refused. Dry-run prints proposed git commands and
links using the invoking checkout's manifest, without fetching, cloning,
switching, creating directories or changing links. A real install reads
the manifest from the updated clone.

`bash tools/codex-run/test-codex-run` tests workers, owners, notifications,
signals and trimming against fake Codex, post and systemd executables.
`bash tools/codex-run/test-codex-run --reap-only` runs the producer adoption
checks with the same isolated fixtures. Both selections assert CPU below
15 seconds and wall time below 60 seconds. Only private fixture copies
shorten the timeout when proving hanging tools and callbacks are killed.
`python3 -I tools/test_install.py` tests installation using local fixture
repositories and a temporary home. Neither command uses real fleet state.
These scripts run directly on hel8; they build nothing.

## 7. Later

- Move other fleet tools and add their entries to `tools/installed`.
- Move run and owner storage into fabrig with a separate migration that
  preserves resumable runs and accounting.
