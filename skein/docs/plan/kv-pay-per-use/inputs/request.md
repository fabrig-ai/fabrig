# skein-kv pays for what it holds: a store's start, snapshots and recovery sized by use, not by its limits (the user, 11-03)

## Why

The user's principle for temper: a deployment pays only for what it uses. A small deployment (a few
shallow tasks, brief reports) keeps a store of a few MB and memory of a few MB. Both grow about
linearly with real use, up to configured upper bounds, never at them.

skein's programming model already says so: "Limits at the edges, dynamic memory inside… a reservation
is never resident" (`skein/docs/foundation/programming-model.md`, section 6). It holds almost
everywhere. A fresh box measured 11.8 MB of anonymous memory and 44 KB on disk, and its store grows
1.1 KB per settings change.

A read-only survey of main (11-03) found three places in skein-kv where configured limits, not use,
size what is paid. Each grows with a bigger profile: plan fabrig-scale raises jig's `standard`
profile, which takes the chunk from 2.2 MB to 59.6 MB and `snapshot_after` from 355 MB to 902 MB.
1. **A store's open reads in profile-sized chunks:** `skein-io/src/file_layer.rs:523` and a second
   near-chunk read at `:1227`, asked by `skein-kv/src/store.rs:676,688,711,724`. Each read is the
   full `chunk` whatever the file holds.
2. **Snapshots wait for a fixed amount of log** (`store.rs:877,896`): `snapshot_after` bytes,
   whatever the live map's size. A busy box keeps up to that much stale log on disk, beside a live
   map of a few MB. kv.md, section 5 allows the bound to be "a multiple of the snapshot's size, or a
   fixed size". The counter also seems to restart at 0 after recovery (`store.rs:155`, `:756`), and
   closing takes no snapshot, so old segments may pile up across restarts. This is unconfirmed; the
   tests settle it.
3. **Recovery reads the whole retained log into memory before replaying it** (`store.rs:714-724`).
   A restart's peak memory then grows with the retained log, up to `snapshot_after`.

## What to change

1. **The open reads what is there:** each read asks `min(chunk, len − at)`. The file's length is
   known from the open (`Opened { len }`). No read past the end; a small store opens with reads of
   its own size.
2. **Snapshots by the live size:** a snapshot starts once the log since the last one passes a
   multiple of the live map's encoded size, with a floor so a tiny store doesn't snapshot on every
   commit. `snapshot_after` stays as a ceiling only. Recovery counts the log it replayed toward the
   next snapshot, or takes one after a replay past the bound, so restarts don't let segments pile
   up. Closing may snapshot if the bound has been passed.
3. **Recovery frame by frame:** replay streams the retained log a chunk at a time (frames already
   carry whole values), holding at most a chunk and the frame being applied, not the whole log.
   kv.md 7's budget already lists recovery's buffers.
4. **The design:** kv.md, sections 5 and 7, and 10.1 if it names sizes. Write the rule plainly: what
   the store holds on disk and in memory follows its live data, and the limits are ceilings and
   admission.

## Tests (in skein, the lowest tier that shows each)

- **A small store opens without large reads:** an open of a store of N KB under a profile whose
  chunk is far larger asks reads totalling about N KB. Use skein-io's test layer or a counting file
  layer.
- **Disk follows the live map:** a store whose live map stays small while it churns (many puts over
  the same keys) keeps its log within the multiple of the live size plus one segment, not up to
  `snapshot_after`.
- **Restarts don't pile up segments:** churn, restart, repeat. The retained log stays bounded as
  above.
- **Recovery's memory** stays within a chunk plus a frame, whatever the retained log's length: a
  counting allocator or the domain's byte meter.

## Out of scope

- jig's profiles and temper's limits: plan fabrig-scale.
- New explicit deployment budgets (a configured memory and store ceiling from which limits are
  derived) are a later design.
- Layout changes across restarts.
