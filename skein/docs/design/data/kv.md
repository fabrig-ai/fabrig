# A key-value store on Skein's loop

Provisional, 2026-10-05. A built-in store for a service whose durable
dataset fits in memory: ordered keys and values held in memory, an
append-only commit log, and snapshots, all as a Skein machine on Skein's
file I/O. Large append-only payloads go beside it, in logs of their own
whose committed lengths the store records (section 6).
Read it with [the programming model](../../foundation/programming-model.md),
[the testing strategy](../../foundation/testing-strategy.md), and
[the file design](../runtime/io/README.md).

## 1. In one page

- **What it offers.** Ordered byte keys to byte values. A commit is a
  batch of puts and erases, numbered, applied in order, and answered
  once it is durable. Reads (a key, or a page of a key range) are served
  from memory and see only durable commits.
- **What it assumes.** The dataset fits a stated memory budget; one
  service owns the store's directory; values are opaque, and the service
  versions its own record shapes.
- **What it pays for.** What the store holds on disk and in memory
  follows its live data: a read asks what its file holds, a snapshot's
  trigger follows the live map's size, a snapshot's chunk packs to a
  small target whatever the largest row the limits admit, and recovery
  holds a read and the frame it applies, never a whole file. Its limits
  are ceilings and admission, never the sizes it starts at (sections 3,
  5 and 7).
- **Why not redb here.** Keeping redb on the loop means an in-memory
  virtual file and a persistence machine that replays redb's storage
  calls (`../draft/redb.md`, sections 1 to 4): all the data in memory and a
  persistence protocol of our own anyway, with crash-safety resting on
  redb's internal call patterns. With the data in memory, a log and
  snapshots are the whole of what is needed, and their recovery rule is
  simple enough to test by crashing at every operation.
- **A Skein machine.** Unlike redb (`../draft/redb.md`, section 6), the store is an
  ordinary `no_std` step machine: its state is the map and its
  persistence state, and its disk work is io's file requests, with one
  terminal event each.
- **Not offered:** secondary indexes, queries, several processes, reads
  of what is not durable, a dataset larger than the budget.

## 2. Requests and events

The owner's vocabulary (`skein-kv::Request` and `Event`):

| Request | Terminal event |
|---|---|
| `Commit { ops }`, each op a `Put { key, value }` or an `Erase { key }` | `Committed { number }`; `Refused` (too large, past the budget, too many queued); `Failed` (the store stopped; outcome uncertain) |
| `Get { key }` | `Got { value }` |
| `Load { range, max }` | `Loaded { rows, next }` |
| `Open { root }` | `Opened { last }` or `OpenFailed` |
| `Close` | `Closed` |

After a fatal file failure, repair reports `Recovered { last }` or
`RecoveryFailed` as a store state event. It does not answer the original
`Open` request a second time.

- **Numbers are contiguous,** from one, and survive restarts.
- **A commit is applied to the map when its sync completes,** so a read
  never sees what a crash could lose, and the owner's outputs that
  depend on a commit wait for `Committed`.
- **Refusals happen at the entrance,** before anything is written: a
  commit whose bytes would take the map past its budget, an op past the
  key or value limit, a queue full.
- **A refusal says which,** as bounded data and never a key's or a
  value's bytes, so an owner can say why without asking again.
  `Refused`'s `Refusal` is one of:
  - `TooLarge(Large)`, `Large` naming what passed its limit: `Key` (an
    op's key, a get's, or a load's range end past `Limits::key`),
    `Value` (a put's value past `Limits::value`), `Ops` (more ops than
    `Limits::ops`), `Frame` (the commit's encoded frame past
    `Limits::commit`, or past what a frame can encode), `Page` (a load's
    `max` zero or past `Limits::page`, or its first row past the `max`
    it asked), `Range` (a load's range ending before its start).
  - `Full(Exhausted)`, `Exhausted` naming what the commit would
    exhaust: `Budget { used, budget, projected }`, the map's bytes,
    where `budget` is `Limits::budget`, `used` the bytes admission
    counted before this commit, with the commits queued ahead of it
    (section 7), and `projected` the bytes this commit would leave,
    past `budget` (a charge past `u64` makes it `u64::MAX`);
    `Entries { capacity }`, more entries than the map's `capacity`
    holds; `Numbers`, the commit numbers at their end.
  - `Busy` (the queue or its bytes full) and `Unavailable` (not open, or
    closing), which carry nothing.

  The checks keep their order; only what a refusal carries is new.
  `Large`, `Exhausted` and `Refusal` are `Copy`.
- **An in-memory store** (`Store::memory`) has no directory: it takes
  commits without an `Open` and applies them at once, for an owner's
  tests that need no files.

### 2.1 Keys

Owners build composite keys whose byte order is their logical order
(temper's `task/<n>/msg/<seq>`, `proj/<p>/ended/<rtime>/<n>`), with the
crate's `KeyWriter`, bounded by the key limit, and read them back with
`KeyReader`:

- `tag(u8)`: a key space;
- `u64(n)`: big-endian, fixed width; `rev_u64(n)`, its complement, newest
  first;
- `bytes(b)`: `0x00` escaped as `0x00 0xff`, ended by `0x00 0x00`, so a
  shorter string sorts before its extensions;
- `finish()`: the key, or none past the limit.

### 2.2 The census

The store counts what its map holds, by keyspace, as it applies what
changes it, so that its owner can say what the store holds and what
its log writes without walking either. Counting is off until the owner
gives a rule, and an owner that gives none pays nothing.

- **A rule names the keyspaces:** `CensusRule { prefix: [u8; 256] }`,
  given by `Store::count(rule)` before `Open` (or before the first
  commit of an in-memory store). A key whose first byte is `b` belongs
  to the keyspace of its first `prefix[b]` bytes, at least 1 and at most
  `CENSUS_PREFIX` (8); a key shorter than that, to the keyspace of its
  whole bytes. An owner's rule follows its key writer's tags: jig's
  component counts its own rows by two bytes, jig's by three, and the
  application's by two (jig's `protocol/store/README.md`, 6.3).
- **What each keyspace counts:**
  - `rows`, `keys` and `values`: the rows the map holds, and their keys'
    and values' bytes;
  - `heads`: its rows by head, the value's first two bytes read as a
    big-endian `u16` (a versioned record's version, skein's
    `data/codec/README.md`), each head with its rows, in ascending
    order; a value under two bytes counts under no head;
  - `puts`, `erases` and `written`: the operations applied since the
    log's period began and the bytes they carried (a put's key and
    value, an erase's key), whether or not the erased key was held.
- **The log's period** begins at the last snapshot's start, S (section
  5), and the census's `since` says S. A snapshot's start empties every
  keyspace's `puts`, `erases` and `written` and sets `since` to its S;
  with no snapshot yet, `since` is the first commit the log holds, 1 for
  a new store.
- **Kept as the map changes,** in the same place the map changes: a
  commit's operations as it is applied (section 2), a snapshot's rows as
  recovery loads them (rows, bytes and heads only), and the frames
  recovery replays after it (as commits). So a store reopened counts,
  as it recovers, what it held when it stopped, and its writes since
  the snapshot it recovered from, with that snapshot's S as `since`. A
  recovery that starts the map again starts the census with it. Nothing
  of the census is written to disk.
- **Bounded by constants,** as `CHUNK_TARGET` is: at most
  `CENSUS_KEYSPACES` (128) keyspaces with a prefix, and the empty
  prefix's beside them, 129 in all; at most `CENSUS_HEADS` (8) heads in
  each, no head among them, so 7 with a `u16` beside it. A key of a
  keyspace past the bound counts under the empty prefix, which sorts
  first, and a head past the bound under no head. A head with no row
  leaves at once; a keyspace with no row leaves at the next snapshot's
  start. A new keyspace or head is admitted only while no row folded
  under its overflow for the bound is left there, so an erase never
  takes a row from an entry that didn't count it. An apply's step finds each operation's keyspace by
  a search of at most `CENSUS_KEYSPACES` entries, and the census's memory
  grows with the keyspaces it holds, up to its bound.
- **Read whole,** at once and with no event: `Store::census()` gives the
  `Census`, its `since` and its keyspaces in prefix order, each with its
  prefix and counts, as of `Store::last()`, as `used()` gives the map's
  bytes. A store whose owner gave no rule gives an empty census.

## 3. On disk

The store owns a directory, a root (../runtime/io/files.md):

- **`snapshot`:** the map in key order, with the commit number its replay
  starts from (section 5), in checksummed chunks and a trailer;
  replaced atomically.
- **Log segments,** `log-<first number, 20 decimal digits>`: frames
  appended in commit order. New segments are announced with a directory
  sync before any commit in them is acknowledged.

| A frame's field | Bytes |
|---|---|
| magic `SKVC`, format version | 4, 2 |
| commit number | 8 |
| op count, body length | 4, 4 |
| ops: kind (1), key length (4), key, and for a put value length (4), value | body |
| CRC-32C of everything before it | 4 |

**Recovery** reads the snapshot, then applies the frames of the segments
from the snapshot's start number, accepting the longest valid prefix: it
stops at the first frame that is short, fails its checksum, or is not the
next number. Nothing after that point was ever answered as committed. A
new segment starts after recovery; a segment with a torn tail is never
appended to, and is deleted with the others the next snapshot covers.

Recovery reads each file from its start, never past its end: each read
asks the smaller of the chunk target (section 5) and what the file holds
past the read's offset (the length its open reports), so a small store
opens with reads of its own size and a large one with reads of the
target, whatever `Limits::chunk`. No read assumes a chunk's size: a
chunk's and a frame's own lengths say how long it is, and one longer
than a read is carried across reads, its whole length reserved once
those lengths have arrived, so that its reads do not each move it. It
applies the snapshot a chunk at a time and the log a frame at a time as
the reads arrive, holding one read and the chunk or frame being applied
(section 7); a snapshot found corrupt part way fails the start, as a
whole one would. Any chunk up to `Limits::chunk` is read alike, however
its rows were packed, and one past it is corrupt. Segments named before
the snapshot's start, which a crash between a snapshot's rename and
their removal leaves (section 5), are not replayed; the next snapshot
removes them with its own.

## 4. Commits

- **Group commit.** While a sync is in flight, commits queue; the next
  write carries every queued frame, followed by one sync, and each is
  answered once that sync completes. Queued bytes are bounded.
- **A failed sync is fatal to the open store.** After a failed `fsync`
  the kernel may have dropped the dirty pages and will not report the
  error again, so a retry proves nothing. The store stops admitting,
  answers what is in flight `Failed`, settles its I/O, and recovers from
  the disk as at a start. The owner learns an uncertain commit's outcome
  from the recovered number.
- **A failed or short write** is continued or treated as a failed sync;
  either way no commit is answered unless every byte before its sync
  barrier is written and synced.

## 5. Snapshots

- **When:** the log since the last snapshot's start passes a bound that
  follows the live map: `SNAPSHOT_MULTIPLE` (2) times the map's encoded
  size, each row's key and value and 8 bytes of lengths, as a snapshot's
  chunks hold them; at least `SNAPSHOT_FLOOR` (64 KiB), so that a tiny
  store does not snapshot on every commit; and at most
  `Limits::snapshot_after`, which is a ceiling only. The bound holds both
  the disk a store keeps, its snapshot and less than the bound of log,
  and the replay a restart does: a store of a few KB keeps under 64 KiB
  of log, and a larger one whose map churns in place never more than
  twice its map, whatever its limits. While a snapshot runs, the log
  before its start stays until it finishes.
- **Across restarts:** the frames a recovery replays count toward the
  next snapshot's bound, so a store reopened past its bound snapshots
  once it is open, and restarts never let segments pile up. Closing
  starts no snapshot, and finishes one already running: a close past
  the bound stays quick, and the next open counts what it replays.
- **Fuzzy, so nothing is frozen.** Frames carry whole values, puts and
  erases, so applying a frame twice is harmless. A snapshot records the
  next commit number as it starts, S, then writes the live map in key
  order, a chunk per step, resuming after the last key written, while
  commits go on. A key that changes behind the cursor is put right by
  replaying the log from S; one ahead of it is written as it then is.
  There is no frozen copy, and memory does not double.
- **A chunk packs to a target:** `CHUNK_TARGET` (1 MiB), and grows past
  it only to hold one row. Rows go in key order until the next would
  take the chunk past the target; a row that alone passes the target
  has a chunk of its own, up to `Limits::chunk`. So a chunk is the
  target's size, or one row's when that row is larger: it falls short of
  the target by less than the row after it, and by more only at the
  map's end. `Limits::chunk` is the ceiling: it is there to hold the
  largest row the limits admit, and packs nothing. Below `CHUNK_TARGET` the ceiling is the target, since no
  chunk passes it.
- **Why a constant, and why 1 MiB:** the target sizes a working buffer,
  not what a deployment admits, so the store owns it, as it owns
  `SNAPSHOT_FLOOR`: no profile has a reason to set it, and a `Limits`
  field would be one more figure each must derive. A chunk is one write
  and one step's copying and checksum (section 7), so the target trades
  a step's length and its transient heap for the file requests a
  snapshot makes: at 1 MiB, a thousand rows of a kilobyte, a step of a
  millisecond or two, and a snapshot of a gigabyte in about a thousand
  writes.
- **Finishing** follows ../runtime/io/README.md's replacement sequence, with the writes a
  chunk at a time: create the temporary, write, sync, close, rename over
  `snapshot`, sync the directory. Only then are the segments wholly
  before S deleted.
- **A crash during a snapshot** leaves the previous snapshot and every
  segment since it, so recovery is unchanged. A crash after the rename,
  before the old segments are deleted, leaves them beside the new
  snapshot: recovery skips them and the next snapshot deletes them
  (section 3).

## 6. Payload logs beside the store (planned)

For values that are large, appended to, and read rarely (temper's
transcripts), and so are not held in memory:

- **One log file per owner key,** frames each with a sequence number and
  a checksum, in a directory beside the store's.
- **Data first, then the commit.** The service appends the frames and
  syncs them, then commits a store record holding the log's committed
  extent (its length and last sequence), with whatever else that decision
  changes. The store's commit is the point of truth.
- **Past the extent is garbage:** bytes of a write that never committed.
  Readers stop at the extent, and the next append writes at the extent,
  over them, so nothing is ever truncated.
- **Removal:** a log whose extent record is erased is removed once that
  commit is durable; at recovery, a log with no extent record is removed.
- **Bounds:** a size per log, the open logs, the bytes in flight.
- **In the vocabulary:** two ops, `Extend { log, length }` and
  `Drop { log }`, which the store keeps in a key space of its own; and
  two requests, `Append { log, bytes }`, written at the committed extent
  and synced, answered with the length to commit, and
  `ReadLog { log, offset, max }`, never past the extent.
- **What it needs of io:** opening an existing file to write, in the
  kernel's records, the ring, the simulator and the conformance suite.

## 7. Memory and bounds

- **The budget** covers the map's keys, values and per-entry overhead,
  queued commits, the frames being written, a snapshot's chunk, and
  recovery's buffers: one read of at most the chunk target, and the
  snapshot chunk or frame being applied (section 3), whatever the
  retained log's length. Recovery that would pass it fails the start,
  saying so.
- **Limits are ceilings:** the store pays for what it holds, not for its
  limits. A read asks what its file holds, up to the chunk target
  (section 3), a snapshot's chunk packs to that target and the
  snapshot's trigger follows the live map up to `snapshot_after`
  (section 5), and recovery holds a read and a frame, not the log it
  replays.
- **`Limits::chunk` is a ceiling too:** the largest chunk the store
  writes or reads, which holds the largest row the limits admit. It sets
  no working buffer: a snapshot's step holds its chunk's rows and their
  encoding, and recovery a read and a chunk or frame, so both transients
  follow the target and the largest row the store holds. A store pays
  for a chunk past the target only while it holds a row that large.
- **`Limits::is_usable` checks the limits:** key and value sizes, operations and
  bytes per commit, queued bytes, segment and chunk sizes and derived
  capacities remain checked at the entrance. The map's byte budget is
  admission, not a sum of all the process's allocations.
- **Loop latency** stays bounded: a commit applies its ops to the map, a
  load returns a page, a snapshot writes a chunk per step.

- **Sizes for the layer above:** `FRAME_BYTES`, `put_bytes`, `erase_bytes` and `chunk_bytes` are the frame's and a snapshot chunk's encoded sizes, the numbers a user's limits are derived from, written once here; `chunk_bytes` of the largest row is `Limits::chunk`, the ceiling. `SNAPSHOT_FLOOR` and `SNAPSHOT_MULTIPLE` are the snapshot's trigger (section 5), from which the disk a store keeps is estimated. `CHUNK_TARGET` is what a chunk packs to and the most a recovery read asks (sections 3 and 5), from which a store's working buffers are estimated.

## 8. What it needs of io

../runtime/io/README.md's whole-file operations hold a file's content in one request. The
store needs more of io, which its file requests (`skein_io::file`) give:

- **an open file written at its end and synced,** kept open across
  requests: the log segments and the payload logs;
- **a file written a chunk at a time,** then replaced over its target as
  ../runtime/io/files.md orders it: the snapshot;
- **reads by offset** up to a stated length: recovery, and payload logs
  read back;
- **creating a file with given permission bits** (a store of secrets is
  `0o600`), and removing and listing.

The fake machine behind the simulator models what a crash keeps: synced
data; of what is not synced, any writes in any order, the last torn at
any byte; a directory entry only once its directory is synced; a rename
whole or not at all. Its faults include failed and short writes, and
failed syncs.

## 9. Verification

- **The machine's world** runs commits, loads and snapshots against a
  model map, on the simulated disk, crashing at every operation and every
  sync boundary, then recovering. Its referee: the recovered map is the
  last acknowledged commit, or that plus a whole uncertain one in flight,
  never part of a commit; numbers contiguous; every load equal to the
  model's.
- **Snapshots under load:** commits racing a snapshot at every chunk,
  crashes at each phase of its replacement.
- **Payload logs, later:** a crash between the data's sync and the
  commit leaves the extent as it was, and the next append overwrites the
  tail.
- **The format's golden bytes:** a log segment and a snapshot made from
  one fixed sequence of commits (puts and erases, an empty commit, keys
  and values at their bounds, a segment's roll), kept as files beside
  the crate's tests and never regenerated by the test that reads them.
  The test makes the same commits on the simulated disk and compares
  the files byte for byte, then recovers the kept files and compares
  the map with the model's. A change to a frame's or a snapshot chunk's
  bytes fails it, naming the format: from the first release on, a
  deployment's store holds these bytes, and changing them needs an
  upgrade of the store's own, run before any records are patched
  (temper's `draft/upgrades.md`, 6.5). The test says so, and the change
  moves the format's version.
- **What it pays for:** an open asks reads totalling the bytes of the
  files it reads, whatever the chunk; a store whose map churns in place
  keeps its log within its bound and a segment (section 5), across
  restarts too; a crash at each step of a snapshot leaves no segment
  past the next snapshot; recovery's transient heap stays within a read
  and a frame or snapshot chunk, whatever the retained log's length;
  under a large `Limits::chunk`, many small rows are written in chunks
  of about the target, and a snapshot's and a recovery's heap peaks stay
  near the target, not the ceiling; a row past the target goes through a
  snapshot and a recovery alone in its chunk, and a row at the ceiling
  round-trips.
- **Over the real ring:** a scratch-directory integration test commits,
  closes, reopens on the same root descriptor and verifies the value. A
  killed process leaves the page cache, so the ring shows integration,
  not power loss: that is the crash model's (section 8), which no test
  against a kernel can check.

## 10. For temper

- **The engine's store** (jig's `protocol/store/README.md`, which temper's
  `protocol/store.md` extends): the children's records and keys as
  values and keys, the root's commit as one `Commit`, loads by key range
  at restart; secrets in a second store, in a root of their own,
  `0o600`.
- **Transcripts** as payload logs: a turn's bytes appended and synced,
  then committed with its spend and the messages it took; the worker's
  acknowledgement after that commit.
- **Retention is temper's:** ended tasks moved out of the map, or into
  payload logs, past a horizon, so the budget holds for years.

### 10.1 How large, roughly

Back of the envelope, for a busy small team's deployment: a few people,
tens of repositories, about **300 tasks a day**, two thirds of them
agents' (bounded in practice by LLM spend: a few hundred dollars a day
at one or two dollars a task), the rest procedures and people's.

| What | Per task | Per year (~110k tasks) |
|---|---|---|
| the task: spec, contract, lineage, state, result | 2–4 KB | |
| messages kept (most change tasks hear none; chats and coordinators many) | 1–5 KB | |
| effects and their keys, connector state for its resources | 1–3 KB | |
| funding and spend | ~0.5 KB | |
| the map's per-entry overhead (~15 entries) | ~1.5 KB | |
| **the map, no retention** | **5–15 KB** | **0.5–1.7 GB** |
| **the map, ended tasks cut to a ~1 KB summary after 30 days** | | **~0.1–0.2 GB live, plus ~0.1 GB of summaries a year** |
| transcripts: 20–100 turns of 5–30 KB, bounded by the context window | ~0.2–1 MB an agent run | **~20–40 GB**, ~5–10 GB compressed |

**Room for what comes later.** Connectors beyond the forge (test
environments, deployments, CI systems, issue trackers, chat, cloud
accounts) add resources with state of their own (an environment, a
release, a rollout: 1–5 KB each), news from their events, subscriptions,
and procedure tasks (provisioning, deploying, rolling back, tearing
down), perhaps doubling the tasks a day. Sizing for **three to five
times the forge-only figures** gives:

| What | Per year |
|---|---|
| the map, no retention | ~2–8 GB |
| the map, with the 30-day horizon | ~0.5–1 GB live, plus ~0.3–0.5 GB of summaries a year |
| transcripts | ~50–150 GB, ~15–40 GB compressed |

So a budget of **1–2 GB for the map** covers a busy deployment with
several connectors and a retention horizon, for years; a store that is
designed to reach about 10 GB in memory before its limits bite leaves
room for the cases this estimate is wrong about. Past that, retention
tightens first, then `../draft/redb.md`'s approach or an on-disk index.

- **The map fits in memory comfortably** with a retention horizon, and
  even without one for the first year or two on a server; the live
  working set (tasks not ended) is tens of megabytes, a few hundred with
  several connectors.
- **Transcripts do not,** which is why they are payload logs: a few
  hundred megabytes a day, on disk, with a retention of their own.
- **Write rate is low:** tens of thousands of commits a day (task
  changes, turns' extents, effects), under one a second on average and
  tens a second at peaks; with group commit, one sync of a few
  milliseconds covers a burst. The log grows by roughly 10–20 MB a day
  before snapshots fold it.
- **Restart** reads a snapshot of a few hundred megabytes, about a
  second from an SSD, and a log of at most twice the map (section 5).

## 11. Where the risk is

The store itself is small: a map, a frame codec, group commit, a
recovery rule, a chunked snapshot and payload logs' extents. The risk is
around it:

- **a faithful crash model** in the simulator, since the tests are only
  as good as what it says a crash keeps (section 8);
- **sync failures** handled as fatal, everywhere a sync is awaited
  (section 4);
- **the fuzzy snapshot's argument** (section 5), the one part that is not
  obvious;
- **memory accounting** close enough to the allocator's to make the
  budget mean something (section 7).

## 12. Settled choices and later work

- Frames and snapshot chunks use version 1 and CRC-32C. Snapshot files
  have a checksummed trailer; recovery rejects one without it.
- `Limits` supplies a fixed segment size, the snapshot's ceiling
  (section 5) and a timeout for each file request. A hung kernel
  operation is cancelled; its owner receives one terminal failure after
  the operation settles.
- One store owns one directory and one file driver.
- Later, when a user needs them: payload logs (section 6); several
  stores sharing a sync; reads at one commit across pages, for a
  consistent view of many; `Load` in reverse order, where reversed keys
  do not serve.
- A rollover's `Close` answered `Failed` goes as a failed write does
  (section 4), but the simulator fails no `Close`, so no test reaches
  it. Later: that fault, beside `io_error`.
- A deployment's budgets are a later design: a configured ceiling on its
  memory and its store, from which `Limits` are derived, beside
  section 7's rule that a store pays for what it holds. `CHUNK_TARGET`
  stays the store's own unless that design needs to move it, and that
  design meets what still pays `Limits::chunk` beside the store: jig's
  patch, whose copy and switch work in reads of the row ceiling (jig's
  `protocol/store/README.md`, section 11).
- When the dataset outgrows memory, retention comes first, then values
  on disk: keys in memory with where their values lie, values read by
  offset, compaction in place of snapshots, and the index rebuilt at
  start from hint files. `Get` and `Load` then answer after their reads,
  not in the step that asks; the rest of the vocabulary stays. The
  approach in `../draft/redb.md` is the other way past memory.
