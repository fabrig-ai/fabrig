# The store: hearth's own state

## 1. hearth's own state

### 1.1 The filesystem is the only truth

- **Subvolumes and the records beside them are the builder's state,**
  read back at each builder's start: no kv store, no index file to
  disagree with the disk.
- **Only in memory:** builds in flight (their staging is deleted) and
  leases. The queue of jobs is the steward's (hearth/docs/design/domain/steward.md, 3.4). A build
  asked for again finds or redoes its layers.
- **The in-memory index** holds one entry per published layer (key,
  kind, parent, tree digest, uuid, counts, when), bounded by the store's
  layer limit (4,096 by default); one per resolution, by key, bounded by
  its resolution limit (1,024); and one per cache file, by digest,
  bounded by its cache limit (262,144 files). The three limits are
  configured (`skein/docs/foundation/programming-model.md`, section 7). Tree files are never
  held whole, and the scan reads no cache file's record: it pairs files
  and records by name, and a record is read when its file is used.
- **A store at a limit is full.** A scan that counts more resolutions or
  cache files than their limits stops there: the builder holds no index,
  and answers every request that reads or adds to the store with an
  operational error of kind `store` naming the store, the limit and what
  passed it (hearth/docs/design/domain/render/errors.md, 2). Only a cache file with its record counts
  (1.2, step 6). A resolution is not begun without room for one more
  resolution and for every index its source lines may keep (128 cache
  files by default, `records.md`, 1), so a full store runs no mmdebstrap; a
  resolution or a cache file that would still pass its limit while the
  builder runs is refused the same way, and its file is removed. Nothing
  is evicted to make room in M1: collection is M2's (`README.md`, 5.5). From M2 a
  store at a limit collects first, one run (`README.md`, 5.2), and is full only if it
  is still at the limit after it. A store without
  layers that is full is deleted whole and made again (`README.md`, 2.4).
- **A store that cannot be opened ends the call.** A lock or a
  `store.record` that fails, or a `store.record` the builder refuses
  (`records.md`, 6), keeps the builder from starting: nothing renders, and the
  client says so on standard error, with the code 75 (hearth/docs/design/client/README.md,
  3.2 and 3.3).
- **A store whose upkeep fails is refused as a full one.** A page of a
  listing, a removal, a record's load, a `staging/` tree past the walk's
  bounds (`tree-helper.md`, 6) or an index's keeping (`records.md`, 8) that fails leaves the builder
  with no index: it answers every request that reads or adds to the
  store with an operational error of kind `store`, naming the store and
  the step that failed, and leaves the store as it is for the next
  start's scan.
- **A store's error is the call's,** given once, not each target's: a
  resolution the store would not find or keep leaves its key needed
  (hearth/docs/design/domain/render/spec.md, 2.7), and the call's code is 75 (hearth/docs/design/client/README.md, 3.3).

### 1.2 The startup scan

It runs at every builder's start, and the steward spawns a builder for
work (hearth/docs/design/domain/steward.md, 4.2), so what a crash, a kill or a power cut left is
settled before the first job of every pass.

1. **The lock:** an exclusive `flock` on the store's root directory,
   taken without waiting (`skein/docs/design/runtime/io/files.md`: `Lock`) and held until the
   builder exits. A second builder over the same store, a daemon's or a
   client's in-process on a developer's store, waits for it
   (hearth/docs/design/domain/steward.md, 3.4): while the lock is busy it asks again every 200 ms,
   and says once, after its first second of waiting, that it waits (on
   the client's standard error, hearth/docs/design/client/README.md, 3.5; as an event of
   its job [M3]). It sets no limit of its own: the person at the client
   may interrupt it, and a daemon's job has its steward's deadline
   (hearth/docs/design/domain/steward.md, 5.2). The root, not `store.record`, which `records.md`, 8 may write
   again by rename onto another inode.
2. **The store's record** (`records.md`, 6): version, layout, uid and range.
3. **Every subvolume under the store,** with
   `BTRFS_IOC_GET_SUBVOL_ROOTREF` on its root: the child subvolumes'
   tree ids and the directories holding them, 255 a call, continued from
   the last tree id. `btrfs subvolume list` needs root; this does not.
4. **Each one's path,** with `BTRFS_IOC_INO_LOOKUP_USER`, which checks
   read and search permission on each component: all the builder's.
5. **Each one's identity,** with `BTRFS_IOC_GET_SUBVOL_INFO`: uuid,
   parent uuid, read-only flag, generation.
6. **Matched against the records:** `README.md`, 4.4's cases, `README.md`, 4.5's uuids, each key
   recomputed, a resolution's among them (`records.md`, 4), and each cache file with
   its record. A subvolume inside a layer (`GET_SUBVOL_ROOTREF` on the
   layer) is refused, and the layer deleted. A store without layers runs
   this step for its resolutions and its cache alone (`README.md`, 2.4). `cache/`,
   `resolutions/` and `layers/` are listed page by page (
   `skein/docs/design/runtime/io/files.md`: `ListPage`), 1,024 entries a page, so a directory of any
   size is scanned in bounded memory. Resolutions count against their
   limit (1.1) as they come. `cache/` is listed twice: its records
   first, counted against the cache limit as they come, then its files,
   each paired with its record; a file with none is deleted (`README.md`, 4.4) and
   never counts, so what a crash left cannot fill a store.
7. **Settled:** deletions done (`README.md`, 5.3), strays (any name `README.md`, 2.1 does not
   list) reported once, by their count and the first found, one count
   for the whole scan, M1's directories and the layers' alike, and the
   index built. `state/` is the steward's, read at its own start (1.3).

What the store says while it opens, that it waits for the lock and that
it holds strays, goes to the client's standard error as progress does,
as text or as records with `--json` (hearth/docs/design/client/README.md, 3.2; as events
of its job [M3]); its error is part of the answer, unless it kept the
store from opening (1.1).

The three ioctls have been unprivileged since Linux 4.18, and the
builder makes them itself (`tree-helper.md`, 8).

### 1.3 The steward's records

What the steward must keep across restarts and reboots lives in
`state/`, which only the steward writes and no builder opens
(hearth/docs/design/domain/steward.md, 2.2), each file named by `H("state", name)`, each bounded:

| Record | Holds | Bound | Written | Of a version it cannot read | Defined in |
|---|---|---|---|---|---|
| desired | target, repository, a commit or a branch with its interval, `trial: auto \| manual`, who set it and when, a generation | one | by rename, when the client changes it | reported; the running slot stays until the client sets a revision again | hearth/docs/design/protocol/standalone.md, 1 |
| refused | output keys refused, each with why and when | the last 16 | by rename | discarded, reported; a fall back is refused again as the activator reports it | hearth/docs/design/protocol/standalone.md, 4 |
| hold | who held (a peer's uid), when, the reason; or none | one | by rename | held, reported, until `hearth release` | hearth/docs/design/protocol/standalone.md, 5 |
| pass | the last pass: its generation, commit, stage, output and prepare keys, and its outcome | one | by rename, at each stage | discarded; the bless's render then takes the desired revision, its commit or the branch's head in the clone | hearth/docs/design/protocol/standalone.md, 2; hearth/docs/design/domain/steward.md, section 6 |
| orders [M4] | each order's key, attempt and stage, then its outcome, until the engine acknowledges it; a restart's unit `InvocationID` | 64 outcomes waiting (to start) | by rename, before the effect is asked | discarded; the engine sends again what it has not seen answered, and orders are keyed or conditional | hearth/docs/design/protocol/wire.md, 4.4 and 4.5 |
| signals [M5] | CPU, memory, pressure, load, disk use per mount | three rings: 10 s for 6 hours, 1 minute for 3 days, 15 minutes for 60 days | in place, slot by slot | discarded, started afresh | hearth/docs/design/domain/operating.md, section 5 |
| check history [M5] | each check's results | 1,024 per check, a ring each | in place, slot by slot | discarded, started afresh | hearth/docs/design/domain/operating.md, section 5 |

- **Records are written by rename** (`records.md`, 8), as every record is. An order
  is recorded before the activator is asked for its effect, so a
  restart finds it begun and resumes it under its key or its condition.
- **Rings are written in place.** A ring is a file of fixed-size slots,
  its size computed at startup, with a header in the frame of `records.md`, 1 (kind
  8). Rewriting a whole ring every 10 seconds would cost more than the
  sample, so each slot carries its sequence number and the first 8 bytes
  of its SHA-256, and a torn slot is skipped on reading: a power cut
  loses at most the sample being written.
- **Nothing here can be made again,** unlike the rest of the store, so
  `records.md`, 8's rule matters most here: a newer release writes these in versions
  the previous one reads, and an unreadable one is settled as the table
  says, never by stopping. The fall back that brings an older steward
  back is when convergence matters most.
