# Slot tools

Provisional, 2026-10-28. Builder disk cleanup and local slot bookkeeping.

## 1. In one page

- **Trim on the builder:** `tools/slots/trim [--older MINUTES] [SLOT ...]`
  deletes old test executables and prints each slot's count, decimal GB of
  file sizes removed, filesystem free GB, and skipped processes. Default age
  is 120 minutes; default slots are `~/src/fabrig/slots/*`.
- **Claim on hel8:** `tools/slots/claim claim HOST N OWNER WHAT` records a
  holder atomically, `release HOST N OWNER` removes its own claim, and
  `list` shows every configured slot and its age. No network operations.
- **State stays local:** claim records live in `~/src/fabrig/claims/`, named
  `<host>-<n>`, with mode 0600. `hosts` is the inventory; a claim does not
  grant remote access or start a build.

## 2. Trimming

### 2.1 Eligible files and open descriptors

Only regular files with the user-execute bit in `target/debug/deps` qualify.
Files ending in `.so`, `.rlib`, `.rmeta`, or `.d`, symlinks, directories,
nonexecutables, and files modified at or after the cutoff stay. A symlink
in a slot's deps path is an error; an absent deps directory removes nothing.
Immediately before deleting a slot's candidates, scan `/proc/*/fd` metadata
for processes owned by the current uid. Compare device and inode, so an open
file's hard links are protected too. A process is skipped if its metadata,
fd directory, or an fd link raises `PermissionError`, `FileNotFoundError`, or
`ProcessLookupError`. Count each such process once per slot's scan. Continue
scanning its readable fd links, whose inodes remain protected. Other errors
fail the slot before deletion.
Slots without candidates skip the scan and report zero skipped processes.
Never read descriptor contents or another process's environment.

Recheck each candidate's identity, mode, size, mtime and ctime before unlinking.
The descriptor scan is a snapshot: callers trim idle slots, keeping a build's
inputs still, rather than opening old executables during a trim. File sizes
removed and filesystem free space are reported separately, since hard links
and open mappings can affect when blocks become free.

### 2.2 Invocation

Use trim as a task's first step on its builder. When calling from hel8,
send the standalone standard-library script through stdin:

```sh
ssh HOST 'bash -lc "python3 -I - [ARGS]"' < tools/slots/trim
```

The script imports no repository modules. `--older` accepts finite,
nonnegative minutes. Errors report the slot, exit 1, and permit remaining
slots to be inspected. Trimming does not touch caches or incremental state.

## 3. Claims

### 3.1 Ownership and inventory

`hosts` lists `HOST N=OWNER ...`, one host per line; each slot's share is
a post name such as `lead/jig`, or `pool`. Blank lines and `#` comments
are ignored; malformed entries, duplicate pairs and empty owners fail.
Only configured pairs can be claimed or released. Owner and work
description are nonempty text; quote a description containing spaces.
Slots on a draining builder belong to `ops`: leads start no new work there,
while existing holders can still release their claims. Replacement slots
and shares are announced before use. Remove the drained builder from the
inventory after its last task finishes. Build admission remains capped by
the machine's heavy setting, independently of its slot count.
A claim may take its owner's share or a pool slot. Another owner's share
requires that owner's agreement and `--lent-by SHARE_OWNER`; the value
must equal the configured share owner. Refusal exits 1 and names the
share owner and the lend option. Records hold owner, what, and an
ISO 8601 UTC creation time, plus `lent_by` when supplied; records without
it stay valid. Claims use `O_CREAT|O_EXCL` with mode 0600;
an existing claim exits 1 and prints the holder. Release checks the owner
and refuses another owner's claim. Missing claims also exit 1.

List prints every configured pair and its `[share]`, with `free` or the
owner, description, UTC creation time, elapsed age and lender when present.
`list --free OWNER` prints only free slots claimable without a lend,
the owner's share first, then the pool, in inventory order within each;
one `HOST N` per line, exit 0 even when empty.

Run `claim` from PATH on hel8: `tools/install` links it to the dedicated
main clone (`tools/codex-run/design.md, 6`), so its adjacent `hosts` comes
from main rather than the caller's clone. All operations coordinate through a
local advisory lock so readers and release cannot race a record's creation
or replacement. This lock also lives under the claims directory. There is
no expiry or automatic stealing: a holder releases its claim explicitly.

## 5. Disk guard

`tools/slots/guard [--below GB] [--dry-run]` checks the filesystem holding
`~/src/fabrig/slots`. Below 80 decimal GB by default, it trims numeric slot
directories, largest first by allocated regular-file bytes, until free
space is above the threshold plus 40 GB or every idle slot was inspected.
Directory and file symlinks are not followed when measuring size; a numeric
slot symlink is an error. Non-numeric names are ignored.

An idle slot has no active or activating user unit whose name contains
`-heavy-` and whose `WorkingDirectory` is the slot or a descendant. Read
these two properties with `systemctl --user show` after listing units.
Check again immediately before trimming each slot. No heavy state or
process environment is read. A Codex run between builds is idle; its next
build relinks what it needs. Unit properties are snapshots, so a build
starting after the last check can overlap the trim.

The guard loads the sibling `trim` by resolved file path, including under
Python's isolated mode. Both files must travel together; `trim` remains
standalone for task-start use through ssh stdin. The guard calls its trim
function with the current time as cutoff (age zero), sharing eligibility,
descriptor protection and candidate identity checks. Dry-run uses the same
checks and reports counts and sizes without unlinking. It checks every idle
slot while space remains low, without assuming that candidate file sizes
would become free blocks.

Each sweep prints one UTC line with free GB before and after, slots trimmed
(count and GB) or skipped as busy, and `ok` or `still below GB`. It appends
the same line to `~/src/fabrig/notes/slot-guard.log`, creating it as 0600
and enforcing that mode on an existing file. Symlink logs are refused.
Exit 0 means space is at least the threshold or there are no numeric slots;
exit 1 means an error; exit 2 means space remains below the threshold after
all idle slots, including a dry-run. Once triggered, a sweep continues
toward the extra 40 GB even after crossing the threshold.

`guard --timer` replaces the transient user timer `fabrig-slot-guard` using
`systemd-run --user --unit fabrig-slot-guard --on-calendar '*:0/5' --collect`.
It runs the guard's resolved path through `bash -lc` every five minutes,
with the requested threshold, collecting the service when inactive.
`guard --timer --stop` stops its timer and service. Timer control lists and
stops only these named units; it does not trim slots. Start the timer from
`~/src/fabrig/main/tools/slots/guard` so periodic runs use the installed
clone. The install manifest requires identical link and file names, so
there is no `slot-guard` link in `tools/installed`.

`python3 -I -B tools/slots/test_guard.py` uses disposable slots and fake
systemd commands; `test_trim.py` checks the shared deletion policy and
`test_claim.py` checks bookkeeping. They build nothing and run on hel8.

## 4. Later

- Coordination of claims across machines if leads move away from hel8.
- Finding executable mappings after all descriptors have been closed.
