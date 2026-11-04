# kv-pay-per-use: the plan

skein-kv pays for what it holds, from `inputs/request.md` (the user, 11-03), cut for fab by one
lead call (2026-11-03). The shapes that cross tasks are `notes/contracts.md`; the standing
rules `notes/common.md`.

## Design written by this cut

Compatible detail, in `skein/docs/design/kv.md`, with **Not yet** entries in
`kv.status.md` for what the tasks build:

- 1: "What it pays for": what the store holds on disk and in memory follows its live data;
  its limits are ceilings and admission.
- 3: recovery reads each file a chunk at a time, never past its end (each read asks
  `min(chunk, len − at)`), applies the snapshot a chunk and the log a frame at a time, and
  skips segments named before the snapshot's start, which the next snapshot removes.
- 5: "When": the bound is `min(snapshot_after, max(SNAPSHOT_FLOOR, SNAPSHOT_MULTIPLE × live))`,
  the floor 64 KiB and the multiple 2, `live` each row's key, value and 8 bytes; "Across
  restarts": recovery's replayed frames count toward the next snapshot, and closing takes no
  snapshot. A crash between a snapshot's rename and the old segments' removal leaves them
  for the next snapshot.
- 7: recovery's buffers are one read and the chunk or frame being applied; "Limits are
  ceilings"; `SNAPSHOT_FLOOR` and `SNAPSHOT_MULTIPLE` among the sizes for the layer above.
- 9: "What it pays for": the world's checks of reads, the log's bound and recovery's heap.
- 10.1: a restart replays at most twice the map in log. 12: `snapshot_after` is a ceiling.

### Decisions

- **The floor is a constant, 64 KiB, not a limit:** a limit-derived floor (a segment, a
  chunk) is profile-sized again (jig's small profile: a 710 MB segment, a 2.2 MB chunk), and a
  new `Limits` field would change every literal in jig and temper, whose limits are another
  plan's. 64 KiB keeps a fresh box's store near its measured 44 KB, and a snapshot of a tiny
  map costs a few file operations.
- **The multiple is 2:** a store churning in place keeps at most its snapshot and twice its
  map in log, writing each byte of log about 1.5 times.
- **`snapshot_after` as the ceiling** keeps every test that sets it to 1 (a snapshot after
  every commit) as it is; a test that set it huge to mean "never" now snapshots once its log
  passes 64 KiB.
- **Closing takes no snapshot** (the request allowed it, "may"): recovery counts what it
  replays, so the next open snapshots at once when past the bound; a close stays quick, and
  the golden files (`skein/tests/kv/golden/`), made by a close past the bound, stay as they
  are.
- **The snapshot streams too:** the request names the log; holding the whole snapshot and its
  decoded rows beside the map makes a start about three times the map, against kv.md 7's
  budget, so recovery reads both a chunk at a time.
- **A crash's leftover segments** below a snapshot's start are dropped from recovery's list
  today and never removed: the restart task keeps them for the next snapshot's removal.

## Parts and tasks

| Part | Task | Needs | Batch |
|---|---|---|---|
| open-reads | `kv-open-reads`: an open's reads sized by its files; the world counts reads | | 1 |
| snapshot-bound | `kv-snapshot-bound`: the bound by the live map; disk follows the map | | 1 |
| recover-stream | `kv-recover-stream`: recovery a chunk and a frame at a time; its heap | kv-open-reads | 2 |
| recover-count | `kv-recover-count`: restarts count what they replay; leftovers removed | kv-recover-stream, kv-snapshot-bound | 2 |
| status | `kv-status`: kv.status.md once built | every task | 2 |

Batch 2 is written and released by this call after batch 1.

## Critical path

`kv-open-reads` (45 min) → `kv-recover-stream` (120) → `kv-recover-count` (75) → `kv-status`
(20): about four and a half hours of implementer time. `kv-snapshot-bound` (75) runs beside
the first two.

## One file, several tasks

Every code task changes `skein/crates/skein-kv/src/store.rs`, the store's one transition
file: `notes/contracts.md`, 6 gives each its functions and arms. `kv-open-reads` and
`kv-snapshot-bound` run side by side on disjoint hunks (the recovery reads; `is_ready` and
`fire`), and `kv-recover-stream` beside `kv-snapshot-bound` likewise; the test files and
`main.rs` lines are apart by construction (`notes/contracts.md`, 5). This departs from one
writer per crate, for the parallelism a one-crate plan has.

## Overlaps with the drafts beside this one

None: no other draft lists a file under `skein/crates/skein-kv`, `skein/tests/kv` or
`skein/docs/design/kv*`. The readers' tests the bound and the restart count run (jig's and
temper's stores on files) are run, not edited; jig's `protocol/store.md` reads kv.md 7 as
"about three times each map's budget", which the streamed recovery lowers: an upper bound
that stays true, jig's to tighten.
