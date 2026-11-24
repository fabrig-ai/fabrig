# Checkpoints

2026-11-16. A deployment's state at one commit, taken by the serving
store at a short barrier in its ordered work, mostly by hard links,
never changed once made, and laid out as a stopped deployment's
directory (temper's `draft/archive/checkpoints.md`, sections 2 to 4 and 5.1,
milestone C1 of `temper/docs/design/draft/archive/checkpoints.execution.md`). The component takes it
through skein-kv's holds for both of its stores (skein's `data/kv.md`,
2.3), and takes the transcripts and the task logs itself. Restoring one
is `restoring.md`'s (temper's `draft/archive/checkpoints.md`, 5.3).

## 1. In one page

- **What it is** (2): `<deployment>/checkpoints/<C>-<16 hex>/`, holding
  `active`, generation 1's `store/`, `secrets/` and a fresh manifest,
  `transcripts/`, and its own record, `checkpoint`.
- **Asked for** (3) by `Request::Checkpoint`, answered `Checkpointed`
  with its name and figures, or `CheckpointRefused`.
- **Taken** (4) at a barrier in the component's ordered work: later
  commits wait while both stores are held and the sources are selected
  at C, the last commit; then commits go on while its own file lane
  links what nothing writes again and copies the prefixes of what is
  still appended to. Renamed into place once whole, then the newest
  `store.checkpoints.keep` are kept.
- **Over a stopped deployment** (5): an open that writes nothing but
  recovery's, then the same request.
- **Readers hold its lock** (6), which retention and a removal by name
  respect.
- **At open** (7): a first start refuses a `checkpoints/` that holds
  anything, and every open sweeps checkpoint temporaries.

## 2. What a checkpoint is

### 2.1 Its name

- **`<C>-<suffix>`:** C, the last main commit at the cut, as 20 decimal
  digits, then `-`, then the suffix as 16 lowercase hex digits, the
  first eight bytes of the request's `draw` read big-endian (3).
  `CheckpointName { store: u64, suffix: u64 }` formats and parses it; a
  name of any other form is no checkpoint's.
- **Two may record the same C** (one taken while serving and another at
  a stop with no commit between); the suffix keeps their names apart,
  and nothing orders by it (4.7).
- **Its temporary** is `<name>.skein-<16 hex>`, the last eight bytes of
  `draw`, in `checkpoints/` too (4.6). A name of that form is a
  checkpoint temporary, which a sweep removes (7).

### 2.2 Its layout

`<deployment>/checkpoints/` sits beside `generations/`, outside it, so
no open removes it as a stray (`opening.md`, 1.5), on the deployment's
file system, made `0o700` by the first checkpoint, durable as io makes
it. A checkpoint inside it, laid out as a stopped deployment's directory
(`README.md`, section 2):

| Path | Holds |
|---|---|
| `active` | `1` and a newline |
| `generations/1/store/` | the store's snapshot and the segments recovery reads through C; `logs/`, with each log the map at C names an extent of |
| `generations/1/secrets/` | the secrets' store's snapshot and segments through its own position at the cut, the directory `0o700` and its files `0o600`, each with one link |
| `generations/1/manifest` | the manifest of 2.4 |
| `transcripts/` | the transcript files the map at C names places of (`transcripts.md`, 1) |
| `checkpoint` | its record (2.3) |

- **Left out:** a snapshot still being written and the temporary of a
  durable replace (skein's `data/kv.md`, 5; skein's
  `runtime/io/files.md`, 2); a segment no hold names (skein's
  `data/kv.md`, 2.3); a transcript's or a log's bytes past what the map
  at C names; a log or a transcript no row at C names; every other
  generation and every other checkpoint.
- **Not in it:** the configuration, the cache, the forge, the programs'
  logins and the workers' own state, as a backup holds none of them.
- **Never changed once in place:** a file still appended to is copied
  up to its length at C, never linked (4.3). An open writes the
  directory it opens (recovery's next segment, the transcripts' cut), so
  a checkpoint is opened only through a copy of it: `upgrade --check`
  copies its input (`opening-rehearsal.md`, 1), and a restore stages one
  (`restoring.md`, 4.4 and 4.5).

### 2.3 Its record

`checkpoint`, written by io's durable replace (`Store`, expecting no
file), a versioned record of the component's own schema, added to
`component-v3.schema` in place beside `Manifest`, as `Manifest` was
(`records.md`, 1):

```
record CheckpointFile {
    part: u8          # 1 snapshot, 2 segment, 3 log, 4 transcript, 5 the secrets' snapshot, 6 a secrets' segment
    number: u64       # a segment's first number, a log's number, a transcript's task; 0 for a snapshot
    length: u64       # the bytes the checkpoint's file holds; for a log, its extent at C
    way: u8           # 1 linked, 2 copied, 3 missing (a transcript whose file was gone at C, 4.3)
}
versioned record Checkpoint {
    store: u64                         # C
    secrets: u64                       # the secrets' store's last commit at the cut
    made: u64                          # env.wall at the barrier, in Manifest.written's unit
    build: text 128                    # the component's build name, Store's build_name
    deployment: fixed 16               # the format row's deployment (records.md, 3)
    levels: Levels                     # the format row's levels at C
    files: list 65536 CheckpointFile   # in the order 4.5 makes them
}
```

- **Its path for each file** follows from `part` and `number`: the
  store's `snapshot` and `log-<20 digits>` under `generations/1/store/`,
  a log at skein-kv's path under `generations/1/store/logs/` (skein's
  `data/kv.md`, 6.1), a transcript at `transcripts.md`'s, 1, and the
  secrets' `snapshot` and segments under `generations/1/secrets/`.
- **A linked log's file** may be longer than its extent: bytes a crash
  left past it are garbage that recovery and every read skip (skein's
  `data/kv.md`, 6.2). Its record still says the extent.
- **`FILES`,** 65,536, bounds `files`, and so every checkpoint's files:
  one that would select more refuses `Files` (3) before it links
  anything.

### 2.4 Its manifest

`generations/1/manifest`, a `Manifest` encoded afresh (`README.md`,
section 2) and written by io's durable replace expecting no file:
`generation` 1, `from` none, `before` and `after` both the format row's
levels at C, `counts` empty, `store` C, `secrets` the secrets' store's
position at the cut, `written` the record's `made`. Retention orders by
its `written` (4.7), so it reads a small file, never the record.

## 3. The request and its answer

```rust
// jig_protocol_store, beside Request and Event (boundary.rs)
Request::Checkpoint { owner: Token, draw: [u8; 16] }
Event::Checkpointed { owner: Token, checkpoint: Checkpointed }
Event::CheckpointRefused { owner: Token, why: CheckpointRefusal }

pub struct CheckpointName { pub store: u64, pub suffix: u64 }     // Copy, Ord by (store, suffix)

pub struct Checkpointed {
    pub name: CheckpointName,
    pub secrets: u64,      // the secrets' store's position at the cut
    pub linked: u32,       // files linked
    pub copied: u32,       // files copied
    pub bytes: u64,        // bytes copied
    pub paused: u64,       // nanoseconds from holding later work to releasing it (4.2)
    pub took: u64,         // nanoseconds from the request to the answer
    pub held: u64,         // bytes only this checkpoint names at the answer (4.6)
    pub removed: u32,      // older checkpoints removed by retention (4.7)
    pub skipped: u32,      // older checkpoints past the number kept, left for their lock or an unreadable manifest
}

pub enum CheckpointRefusal {
    InProgress,                                   // "checkpoint in progress"
    Unavailable,                                  // not serving, closing, or a memory store
    Files,                                        // more than FILES (2.3)
    Unrecognized { path: Box<str> },              // a name under store/ or secrets/ it does not know (4.5)
    Vanished { path: Box<str> },                  // a selected file gone (4.5)
    Failed { path: Box<str>, error: file::Error },// a file request of its own failed (4.8)
    InUse,                                        // a checkpoint's lock is held (6)
    Missing,                                      // no checkpoint of that name (6)
}
```

- **One terminal per request,** tagged with `owner`, as every request's
  (`README.md`, section 3). `draw` is 16 bytes its asker draws from its
  randomness, as the shell draws `Open`'s `installation`: the name's
  suffix and the temporary's (2.1).
- **A path in a refusal** is relative to the deployment's directory,
  `generations/8/store/log-00000000000000001234`, at most 128 bytes.
- **`Unavailable`** answers a request made before `Opened`, after a
  close or a stop has begun, or to a store in skein-kv's memory mode,
  which has no files.
- **Its words,** for whoever prints a refusal: `InProgress`, "checkpoint
  in progress"; `Files`, "more than 65536 files to take"; `Unrecognized`,
  "an unrecognized file, PATH"; `Vanished`, "a selected file vanished,
  PATH"; `Failed`, "a file failed, PATH: ERROR"; `InUse`, "in use: its
  lock is held"; `Missing`, "no such checkpoint"; `Unavailable`, "the
  store is not serving". `CheckpointRefusal::words` gives them.

## 4. Taking one

### 4.1 Admitted

- **One at a time:** a request while another is taken, from its
  admission to its answer, refuses `InProgress`.
- **Before the barrier,** with nothing paused: `checkpoints/` made where
  it is missing, and every checkpoint temporary in it removed (7).

### 4.2 The barrier

- **In the ordered work:** the request takes its place in the queue
  that orders the component's commits with its reads (the view's), and
  its turn comes as a read's does: once each commit accepted before it
  is durable, its transcript frames written and synced and its drops
  done (`transcripts.md`, 2 and 5). From then until 4.3 ends, every
  later commit, and every read that queue orders, waits in it, in
  order. Gets, transcripts' reads and secrets don't queue there, and go
  on.
- **Both stores held:** at its turn, `Hold` to the store and to the
  secrets' store in the same step (skein's `data/kv.md`, 2.3). The
  store's `Held` names C, `last`, and the files recovery reads to reach
  it; the secrets' store's names its own position and files. A secret
  mutation asked after the barrier waits in skein-kv behind its hold,
  in order, until `Held`; secrets go on through the component as
  before. A hold refused is the checkpoint's `Unavailable`, the other
  hold released.
- **`paused`** runs from the barrier's turn to the step that releases
  the later work, at the end of 4.3.

### 4.3 What it selects

While the later work waits, the component reads the map at C a page at
a time, each page a step (`pages::load`, as a party's read pages it),
and keeps what it selects in memory, at most `FILES` files (2.3), the
stores' from their `Held` among them, and at most `FILES` run proofs'
tasks beside them:

1. **The run proofs' tasks,** `01 00 02 <task>`, by key alone: the
   tasks that may still write turns, as the open's cut reads them
   (`transcripts.md`, 4). Every task with a turns log or a transcript
   file still written to has one, from its claim to its end (temper's
   `draft/archive/checkpoints.md`, section 9, answered from S7's code).
2. **The transcripts,** each place row, `00 01 <task>`, with its
   committed `length` (`transcripts.md`, 1).
3. **The logs,** each extent row, `00 09 <log>`, with its extent
   (`logs.md`, 1).

Then the later work is released, and the rest runs while commits go on.
Each source is taken so:

| Source | Taken how |
|---|---|
| the store's snapshot and each segment `Held` names with no `end` | linked: skein-kv never writes a sealed segment or a named snapshot again (skein's `data/kv.md`, 3 and 5) |
| the store's segment `Held` names with an `end` | copied up to `end`: frames are only appended, so those bytes never change |
| the secrets' store's snapshot and segments | copied, each sealed one whole and the newest up to its `end`, each file `0o600` with one link, as an upgrade copies `secrets/` (`opening.md`, 2, step 2) |
| a turns log, `2 × task + 1`, whose task has no run proof at C | linked: nothing appends a turn to an ended task's log, and a task never claimed has none |
| every other turns log | copied up to its extent |
| every history log, `2 × task` | copied up to its extent: retention's walks and a forgetting append erase rows to an ended task's history log before its drop (`logs.md`, 3 and 9) |
| a transcript whose task has no run proof at C | linked: an ended task's file is not touched again (`transcripts.md`, 4) |
| a transcript whose task has one | copied up to its committed length, the whole prefix, older windows included |
| a transcript whose file is missing | left out, its record's entry `missing`: a drop a crash cut short leaves its places without the file, and reads answer it as dropped (`transcripts.md`, 5) |

- **Over a stopped deployment** every segment is sealed (5), so each is
  linked.
- **A selection past `FILES`** refuses `Files` at the page that passes
  it, its holds released.

### 4.4 Holds

From the barrier until its last link is made and its last copy synced
(4.5), nothing it selected is renamed over or removed:

- **skein-kv's holds,** on both stores, keep the snapshot `Held` names
  under its name, every segment it names, and every payload log's file
  (skein's `data/kv.md`, 2.3).
- **The component's,** on transcripts: a drop's removal of a transcript
  the checkpoint selected (`transcripts.md`, 5) is deferred, and the
  drop's commit goes on without it. The deferred tasks wait in a list,
  bounded by the transcripts selected; a drop of a transcript the
  checkpoint did not select removes its file at once, as ever.
- **Released** together once 4.5 is done, on success, refusal or
  failure alike: `Release` to both stores, then each deferred removal,
  in order, as a drop removes its file. A crash while a removal is
  deferred leaves that file, which no row names, on disk (10).

### 4.5 Linked and copied

On a file lane of its own, `checkpoints`, beside the store's, the
secrets' and the transcripts' (`Below`, `Io`; `host::Drivers`), one file
request in flight at a time, so its io queues behind no serving request
and no serving request behind more than one of its own.

1. **The temporary** made: `checkpoints/<name>.skein-<16 hex>/`, then
   `generations/1/`, `store/`, `store/logs/` and `transcripts/` in it,
   each `0o700`, and `secrets/` opened private (skein's
   `runtime/io/files.md`, 3); each directory durable as io makes it.
2. **Each name checked** in the active generation's `store/` and
   `secrets/`, a page at a time: `snapshot`, a segment's name, `logs`
   (the store's alone), and the temporaries of 2.2 are known; any other
   name refuses `Unrecognized`, naming it, rather than being skipped.
3. **Linked:** each file to link given a second name in the temporary
   by io's `HardLink`, each log's and each transcript's directory
   made where missing, by the task's or log's high bits as their own
   paths are. A link the file system refuses (`Permission`,
   `TooManyLinks` or `Other`) is that file copied instead, as an
   upgrade does (`opening.md`, 2, step 2), and counted as copied.
4. **Copied:** each file to copy opened for reading and created
   exclusively `0o600` in the temporary, then read and written in
   chunks of at most `CHUNK` (1 MiB) up to its length, synced and
   closed. A source shorter than its length is `Vanished`.
5. **Each directory synced** once its links and copies are made, deepest
   first, then 4.4's holds released.

- **A selected file not found** at its link or its open refuses
  `Vanished`, naming it, but for a transcript, which 4.3 leaves out:
  no part of a checkpoint is ever published with a file missing.
- **Its counts** are `linked`, `copied` and `bytes`, as made, a refused
  link counted as a copy.

### 4.6 In place

1. **The manifest** (2.4), `active` and the record (2.3), each by io's
   durable replace expecting no file, which syncs its directory; then
   `generations/` and the temporary's top synced.
2. **Renamed:** the temporary to `checkpoints/<name>`, then
   `checkpoints/` synced.
3. **Kept** (4.7).
4. **`held` measured:** the bytes of the checkpoint's copies, and of
   each linked file whose link count is then one (`Stat` of its entry):
   what removing this checkpoint alone would free. It grows as the live
   store replaces and removes the files it shares.
5. **Answered** `Checkpointed`, after that last sync.

- **Power loss before 2** leaves the temporary, which the next
  checkpoint or start removes (7), and never part of one under its
  name. After 2, the checkpoint is whole.

### 4.7 Kept

The newest `store.checkpoints.keep` (`Limits`'s `checkpoints.keep`,
`../limits.md`), by their manifest's `written`, then by name, the new
one always among them:

- **Read:** `checkpoints/` a page at a time, each name of 2.1's form
  but the new one's, its manifest loaded (a small file); at most
  `CHECKPOINTS`, 256, at once, the oldest beyond the bound left to the
  next checkpoint's retention. A manifest that can't be read leaves its
  checkpoint in place, counted in `skipped`.
- **Removed,** oldest first, each past the number kept: its directory
  opened and locked by io's `Lock`; `Busy` leaves it, counted in
  `skipped`, until a later checkpoint finds it free; locked, its tree is
  removed, deepest first, as a stray generation's is (`opening.md`,
  1.5), then its name, then the lock let go with its directory; then
  `checkpoints/` synced.
- **A name only:** a file another checkpoint or the live store links
  stays, under its other names.

### 4.8 When it fails

- **A refusal or a failed file request** after the barrier releases the
  holds (4.4), removes the temporary as 7 does (a failure there leaves
  it to the next sweep), and answers `CheckpointRefused`. The engine
  goes on serving: a checkpoint's failure is never the store's.
- **A failure of retention or of 4.6's measure** after the rename
  leaves the checkpoint in place, and answers `Failed`, naming the
  path.
- **A close** ends a checkpoint in progress: its asker is answered
  `Unavailable` once its file request in flight is done, skein-kv's
  holds end with the close (skein's `data/kv.md`, 2.3), and its
  temporary waits for the next sweep.
- **A fatal store failure** ends the holds (skein's `data/kv.md`, 2.3)
  and stops the engine, as ever; the checkpoint is answered
  `Unavailable`.

## 5. Over a stopped deployment

- **An open for a checkpoint:** `Request::Open` takes `purpose:
  Purpose`, `Serve` for every open there is today, or `Checkpoint`,
  which opens as `Serve` does through the stores' recovery and the
  checks of `opening.md`, 1 (the lock, `active`, the maps, the levels),
  but refuses, before it writes anything but recovery's:
  - a directory a first start would make (no `active`), `OpenRefusal::Unready
    { why: Unready::Empty }`;
  - a store the open must write before serving (`layouts.md`, 5.5:
    a patch pending, its registry to seed, a layout to register, a job
    pending), `Unready::Upgrade`.

  Their words, as `OpenRefusal::words` gives them:
  - `Empty`: "`<deployment>` holds no deployment to take a checkpoint
    of: no `active` names a generation, and nothing in it has changed."
  - `Upgrade`: "The store in `<deployment>` needs an upgrade before it
    serves, so this build takes no checkpoint of it, and nothing in it
    has changed. Take it with the build that served it, or start this
    build, which upgrades it, and take one while it serves."
- **Then the request,** as 4 says, with nothing else asking: the
  barrier's turn comes at once. skein-kv's recovery has started a new
  segment, so every segment `Held` names is sealed and linked. Then
  `Close`.
- **The deployment's lock** is the open's (`opening.md`, 1.1), held
  throughout, so it never runs beside a start or an upgrade.
- **The host's adoption,** `Adoption::Checkpoint { draw }` beside
  `Rehearse` and `Back` (`host.rs`): it opens for a checkpoint, asks for
  one, closes, and gives the shell `Checkpointed` or the refusal.

## 6. Readers, and removal by name

- **A reader holds its lock:** io's `Lock` on the checkpoint's
  directory, as a deployment's is held (`opening.md`, 1.1), for as long
  as it reads. `upgrade --check` over a checkpoint takes it as it takes
  its source's (`opening-rehearsal.md`, 1).
- **`Request::HoldCheckpoint { owner, root, name }`,** with the
  deployment's directory as `root`, needing no open: the checkpoint's
  directory opened and locked; answered `CheckpointHeld { owner }`, the
  lock kept until `Close { owner }`; `Busy` refuses `InUse`, and a name
  `checkpoints/` doesn't hold, `Missing`.
- **`Request::RemoveCheckpoint { owner, root, name }`,** needing no
  open either, while the engine serves or not: locked as above, its tree
  and name removed as retention removes one (4.7), `checkpoints/`
  synced; answered `CheckpointRemoved { owner }`, or refused `InUse` or
  `Missing`. A removal and retention never meet on one checkpoint: each
  holds its lock.
- **The host's adoptions,** `Adoption::HoldCheckpoint { name }` and
  `Adoption::RemoveCheckpoint { name }`, as `Back` is made.

## 7. At open

- **A first start** (`README.md`, section 2) whose directory holds a
  `checkpoints/` with any entry refuses `Generations::Checkpoints`, in
  `opening.md`, 1.8's words, and changes nothing: a deployment that lost
  its `active` beside its checkpoints, which a fresh store would number
  over and whose checkpoints its retention would remove.
- **The sweep,** with the strays (`opening.md`, 1.5), at every open that
  removes them, and before each checkpoint (4.1): each entry of
  `checkpoints/` of the temporary's form (2.1) removed, its tree deepest
  first, as a stray generation's, a checkpoint's depth allowed (its
  temporary, `generations/1/store/logs/<directory>/<log>`); then
  `checkpoints/` synced. A checkpoint's own name is never swept.

## 8. Bounds

- **Memory:** the selection, at most `FILES` files (2.3), each a part,
  a number, a length and a way, and as many run proofs' tasks; the
  deferred transcript removals, within it; the retention's `CHECKPOINTS` names (4.7); one page of the map,
  one of a directory and one copy chunk at a time.
- **Steps:** each reads one page of the map or a directory, or makes
  one file request.
- **io:** one request in flight on its own lane (4.5); a chunk of at
  most `CHUNK`.
- **Admission pauses** for 4.2 and 4.3 alone: both stores' holds and
  the map's pages at C. Nothing waits on a link or a copy.
- **Disk:** the copies, and what the live store lets go while a
  checkpoint links it: its snapshot and segments once replaced, its
  logs once dropped.

## 9. Tests

- **Step tests,** in `jig-protocol-store`: a name and a temporary's
  formatted and parsed, and every other form refused; the record's and
  the manifest's bytes, with fixed-rule values built from the schema's
  text; a selection over a map with
  live and ended tasks, transcripts and logs, each source taken as 4.3's
  table says; a second request refused `InProgress`; a memory store's
  `Unavailable`; the first start's refusal and the sweep.
- **The store's world,** `jig-store-world`, on its simulated disk,
  focused: commits and drops racing the cut (a later commit waits, then
  lands outside the checkpoint; a drop of a selected transcript or log
  deferred until release, then done); a snapshot finishing during it
  (held under its temporary, renamed after); a refused link copied; a
  vanished source refused, no checkpoint in place; power loss at each
  step of 4.6, leaving a temporary the next open sweeps or a whole
  checkpoint; retention keeping the newest and skipping a locked one;
  over a stopped deployment, every segment linked. The referee opens a
  copy of each checkpoint as a deployment and finds the state at C:
  both maps, each log to its extent, each transcript to its length.
- **Fuzzy:** commits, drops, snapshots and rollovers racing a
  checkpoint, crashes at its file requests, over seeds: each checkpoint
  in place recovers exactly the referee's state at its C, and the live
  store recovers its own.

## 10. Open questions

- **A removal deferred at a crash** leaves a transcript file no row
  names (4.4). Transcripts are written only by tasks from before S7,
  so it is rare and small; a sweep of such files is later work.
- **One sync for the copies:** each copy is synced, as an upgrade's is.
  If thousands of small history logs make that slow, one `SyncFilesystem`
  before the manifest would replace them.
- **History logs linked:** copying every history log follows from
  retention's erase rows; a log whose task's record retention has shrunk
  and whose drop is due could be linked if retention stopped appending.
