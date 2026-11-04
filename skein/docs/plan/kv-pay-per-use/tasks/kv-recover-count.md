+++
id = "kv-recover-count"
part = "recover-count"
kind = "feature"
needs = ["kv-recover-stream", "kv-snapshot-bound"]
files = ["skein/crates/skein-kv/src/store.rs", "skein/tests/kv/tests/suite/disk.rs"]
effort = "high"
minutes = 75
+++

# skein-kv: restarts count what they replay

## Goal

The frames a recovery replays count toward the next snapshot's bound, and segments named
before the snapshot's start, which a crash between a snapshot's rename and their removal
leaves, are removed by the next snapshot: restarts never let segments pile up
(`skein/docs/design/kv.md`, 3 and 5). Today `log_bytes` starts at 0 after every open, so a
store restarted before each bound is reached keeps every segment since its last snapshot; and
`apply_snapshot` drops the names below the snapshot's start from its list, so they stay on disk
for good.

Acceptance:

- `notes/contracts.md`, 4: after a recovery, `log_bytes` is the bytes of the frames it
  replayed; segments named below the snapshot's start stay in the segment list, unreplayed;
  closing takes no snapshot (unchanged).
- The tests below pass; every test of `skein-kv` and `skein-kv-tests` stays green, the golden
  files unchanged.
- The readers' tests have run (step 4), and any red among them is reported as that step says.

## Inputs

- `skein/docs/plan/kv-pay-per-use/notes/common.md` and `notes/contracts.md`, 3, 4, 5 and 6.
- `skein/docs/design/kv.md`, 3 (the last sentence of "Recovery reads each file"), 5 ("When",
  "Across restarts", "A crash during a snapshot") and 9 ("What it pays for").
- The code as `kv-recover-stream` and `kv-snapshot-bound` leave it: `store.rs`'s `recover`,
  the snapshot's and the log's recovery code, `next_log`, `start_recovery`,
  `finish_recovery`, the `Announcing` arm (where `log_bytes` resets when a snapshot starts),
  and `fire`'s `RemoveOld` stage; `skein/tests/kv/tests/suite/disk.rs` (its limits, model and
  helpers) and `store.rs`'s `crash_during_each_snapshot_step_replays_log`.

## Owns and may change

- `skein/crates/skein-kv/src/store.rs`: the region `notes/contracts.md`, 6 gives this task.
  `fire`, `is_ready` and the snapshot's stages stay as they are: the next snapshot's
  `RemoveOld` already deletes every listed segment below its start.
- `skein/tests/kv/tests/suite/disk.rs`: your tests appended, its helpers reused (a helper
  made shared may move to the file's top).
- Options: if counting what is replayed makes an existing skein test snapshot right after its
  open and that changes what it checks, it is a finding: report it, and change the test's
  limits only if its own purpose is unchanged by it, saying so.

## Steps

1. Count each frame recovery applies into `log_bytes` (after `recover` zeroes it; a torn
   tail's bytes are not counted, and the repair's snapshot resets it as today). Keep the
   names below the snapshot's start in the list, and start the replay at the first segment
   at or past the start.
2. Tests appended to `disk.rs`, with its limits:
   - `restarts_keep_the_log_within_its_bound`: eight rounds over 16 keys, each about three
     quarters of the bound in one-put commits (480 of 64-byte values), then a restart: odd
     rounds close and `world.reopen()`, even rounds `world.crash(round)` and open again.
     After each restart (the world fires a due snapshot), the retained log is at most the
     bound plus one segment; at the end every model row reads back. Each assertion names the
     round, the retained log and the bound.
   - `a_crash_in_a_snapshot_leaves_no_segment_behind`: with `snapshot_after: 4096` (a small
     ceiling, so a snapshot is cheap), for each cut from 0 until the world stops making
     progress, as `crash_during_each_snapshot_step_replays_log` does: a fresh world commits
     past the bound with `request_no_fire` (about 50 one-put commits), steps the snapshot
     `cut` times, crashes (`world.crash(cut)`) and opens again; then commits with
     `world.request` until a snapshot started after one of those commits stands: the
     `snapshot` file's start (its bytes 6..14, big-endian) is more than the reopen's
     recovered number plus one (a snapshot the open itself started does not count), at most
     200 commits. Then no `log-*` file is named below that start, and every model row reads
     back. Each assertion names the cut and the files.
   Check both fail on your base without step 1 (a quick local revert, not committed), and
   say so in the report.
3. The narrowest runs first (`heavy cargo nextest run -p skein-kv-tests --test suite -E
   'test(=disk::...)'`), then `heavy cargo nextest run -p skein-kv -p skein-kv-tests`.
4. The readers' tests: a store reopened past its bound now snapshots once open, where none
   did. Run, one command each:
   `heavy cargo nextest run -p jig-protocol-store -p jig-test-store-files -p jig-store-world -p jig-one-process-world -p jig-channel-world`
   and
   `heavy cargo nextest run -p temper-one-process-world -p temper-service -p temper-engine-domain-world -p temper`.
   A red there that a snapshot after an open explains (a file list, a segment's name, a count
   of operations or files): read its log, then report it under `## Left` with the test, the
   log's lines and why; change nothing outside skein. A red that a snapshot cannot explain (a
   recovered value, a lost or extra event) is a question: stop and ask.
5. `cargo fmt`; old forms; `heavy cargo clippy -p skein-kv -p skein-kv-tests --all-targets --
   -D warnings`; the citations check.
6. Commit (why: restarts count what they replay, and a crash's leftover segments go with the
   next snapshot); push your branch.

## Report

As the implementer's text says, and also: the retained log each round of the restart test
saw against its bound, the cuts at which leftovers appeared, that both tests failed without
step 1, each test's CPU time, and every reader's run with its count and result.
