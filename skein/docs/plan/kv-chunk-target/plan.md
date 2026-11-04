# kv-chunk-target: the plan

A snapshot chunk packs to a small target and `Limits::chunk` is its ceiling, from
`inputs/request.md` (the user, 11-04), cut for fab by one lead call (2026-11-04). The shapes
that cross tasks are `notes/contracts.md`; the standing rules `notes/common.md`.

## Design written by this cut

In `skein/docs/design/kv.md`, with **Not yet** entries in `kv.status.md` for what the tasks
build:

- 1: a snapshot's chunk packs to a small target, among "What it pays for".
- 3: a recovery read asks at most the target; no read assumes a chunk's size; a chunk or frame
  longer than a read is carried, its length reserved once; any chunk up to `Limits::chunk` is
  read alike, and one past it is corrupt.
- 5: "A chunk packs to a target" and "Why a constant, and why 1 MiB".
- 7: the buffers are a read of the target; "`Limits::chunk` is a ceiling too"; `CHUNK_TARGET`
  among the sizes for the layer above.
- 9: the checks under a large `Limits::chunk`. 12: `CHUNK_TARGET` stays the store's own unless
  a deployment's budgets need to move it.

### Decisions

- **The target is a constant, 1 MiB, not a limit:** it sizes a working buffer, not what a
  deployment admits; no profile has a reason to set it; a `Limits` field would change every
  literal in jig and temper, which are out of scope. The same reasoning as `SNAPSHOT_FLOOR`.
- **The effective target is `min(CHUNK_TARGET, Limits::chunk)`:** no chunk passes the ceiling.
  So every test and golden file whose `chunk` is at or below 1 MiB packs and reads as it did,
  and replay (the kv-pay-per-use tests) holds unchanged.
- **The format does not change, and old stores open:** a chunk's lengths already say its size,
  and `snapshot::Reader` already checks it against `limits.chunk`, not against a read. So a
  store written before this change opens at no cost, the version stays 1, and the golden files
  stay.
- **Reads are a fixed target, not a read of each chunk's exact size:** an exact read of a
  59 MB row would hold it three times over in the file layer and the carry; a read of the
  target carries it once.
- **The log's reads take the target too,** so its `3 × chunk + commit` falls as well. A frame
  longer than a read would then grow `bytes` a read at a time, copying a large frame once per
  read, so the carry reserves the frame's whole length once its header arrives, as
  `snapshot::Reader` already does.
- **`snapshot::chunk` builds its bytes once:** its `body` copy made a snapshot's step
  `3 × chunk`, where the request's figure is two. With one buffer the design's "rows and their
  encoding" is what runs, for the target and for a large row alike.
- **The measures:** recovery's is the whole world's `peak − net`, as kv-pay-per-use's. A
  snapshot's is a `Span` around `fire` alone: around `drive`, the fake disk's own growth and
  its pending writes would count.

## Tasks

| Task | Part | Needs | Builds |
|---|---|---|---|
| `target-pack` | pack | | `CHUNK_TARGET`, `Limits::chunk_target`, the packing, one encoding, `target.rs`: the chunks and a snapshot's peak |
| `target-read` | read | `target-pack` | reads of the target, `reserve_frame`, `target.rs`: recovery's peaks, a chunk up to the ceiling |
| `target-status` | status | both | `kv.status.md`: **Not yet** closed; the bounds; 11's figures |

One writer per crate holds the two code tasks in sequence: they share `skein-kv`, and
`target-read`'s peaks rest on `target-pack`'s chunks and on `chunk_target`.

## The request, mapped

- 1, two figures: `target-pack`; design 5.
- 2, reading: `target-read` (reads of the target; a chunk up to the ceiling opens, which is the
  store written before this change); design 3.
- 3, the bounds: both code tasks' measures and `target-status`; design 7.
- 4, replay: the kv-pay-per-use tests, unchanged, in each code task's acceptance.
- The tests: many small rows, a row past the target and a row at the ceiling, in `target.rs`.

## Left outside

`jig/crates/jig-protocol-store/src/generations.rs` copies a generation's files in reads of
`limits.store.chunk`: a patch's copy still pays the row ceiling, a figure the deployment
budget will meet. jig is out of this request's scope; no task touches it.

## Critical path

`target-pack` (50 minutes), `target-read` (60), `target-status` (20): one chain, about two
hours and ten minutes of implementers.

## Overlaps

None: neither `v0.1.0-ladder` nor `debt-7` touches `skein/crates/skein-kv`, `skein/tests/kv`
or `skein/docs/design/kv*`.
