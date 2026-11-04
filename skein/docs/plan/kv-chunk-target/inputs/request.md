# skein-kv: a snapshot chunk packs to a small target, not to the store's largest-row ceiling (the user, 11-04)

## Why

The user's principle: a deployment pays for what it uses, and limits are ceilings
(`skein/docs/design/kv.md`, 1 and 7; plan kv-pay-per-use). One limit-driven cost is left in the store.

`limits.chunk` does two jobs: it is the ceiling that must hold the largest row, and it is the size
each snapshot chunk is packed to (`skein-kv/src/store/compaction.rs`, `chunk`). jig's `standard`
profile raises it to 59.57 MB to hold a large row, where small's is 2.17 MB. So on standard, a store
past about 25 MB of live data writes 59.57 MB chunks. A restart's transient reaches up to 4 × chunk
over the snapshot, about 238 MB (`skein/docs/design/kv.status.md`), and a snapshot's write about
119 MB. That is memory paid because a row may be large, not because rows are. It passes the user's
bound of about 100 MB of memory on a store whose live data is small next to it.

A deployment memory budget, which the user wants next, also needs this split: the row ceiling must
stop setting the working buffers.

## What to change

1. **Two figures:** a snapshot chunk packs to a target, one small constant the store owns or a limit
   of its own (about 1 MiB; the design says which and why), and a chunk grows past the target only
   to hold one row, up to `limits.chunk`, the ceiling. So a chunk is the target's size, or one row's
   size when that row is larger.
2. **Reading:** recovery reads snapshot chunks of either size in place of a fixed read; a store
   written before this change still opens if that costs nothing, and otherwise is refused at open
   as a pre-release format may be.
3. **The bounds:** recovery's and a snapshot's transients follow the target and the largest row
   actually held, not `limits.chunk`. Update `kv.status.md` and `kv.md`, 5 and 7: its `3 × chunk +
   commit` and `4 × chunk` lines, and say that chunk is a ceiling.
4. **Replay:** the kv-pay-per-use tests keep passing unchanged.

## Tests (in skein, the lowest tier that shows each)

- **A store with many small rows under a large `chunk`:** snapshots are written in chunks of about
  the target; the snapshot's and recovery's peak heap stays near the target, not `limits.chunk`
  (the measure kv-pay-per-use uses).
- **A row larger than the target** goes through a snapshot and recovery alone in its chunk.
- **A row at `limits.chunk`** still round-trips.

## Out of scope

- jig's and temper's profiles and limits; the budget knobs; layout migration.
