# Slot tools

Provisional, 2026-10-28. Builder disk cleanup and local slot bookkeeping.

## 1. In one page

- **Trim on the builder:** `tools/slots/trim [--older MINUTES] [SLOT ...]`
  deletes old test executables and prints each slot's count, decimal GB of
  file sizes removed, and filesystem free GB. Default age is 120 minutes;
  default slots are `~/src/fabrig/slots/*`.
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
file's hard links are protected too. Vanishing processes and descriptors
are skipped; unreadable same-user descriptors fail that slot before deletion.
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

`hosts` lists a host followed by its slot numbers. Only configured pairs
can be claimed or released. Owner and work description are nonempty text;
quote a description containing spaces. Records hold owner, what, and an
ISO 8601 UTC creation time. Claims use `O_CREAT|O_EXCL` with mode 0600;
an existing claim exits 1 and prints the holder. Release checks the owner
and refuses another owner's claim. Missing claims also exit 1.

List prints every configured pair, with `free` or the owner, description,
UTC creation time, and elapsed age. All operations coordinate through a
local advisory lock so readers and release cannot race a record's creation
or replacement. This lock also lives under the claims directory. There is
no expiry or automatic stealing: a holder releases its claim explicitly.

## 4. Later

- Coordination of claims across machines if leads move away from hel8.
- Finding executable mappings after all descriptors have been closed.
