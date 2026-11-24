# The store

Provisional, 2026-10-12, revised 2026-10-28. What lies beneath the
engine's store vocabulary: skein's key-value store, embedded in the
engine's process, holds every record jig's core and the application
keep, transcripts are files beside it, and secrets are in a second
store. jig ships the component that does this, `jig-protocol-store`; the
application hands it its own records as bytes it encodes. The domain's
half (commits, loads, what is kept) is `../../domain/engine/decisions.md`, section 1,
and what the rest of the protocol layer relies on is `../README.md`,
section 9. Read it with skein's `data/kv.md`. What is still open is listed in
`README.md`, section 11.

- [`keys.md`](keys.md): key order, owner tags and ranges.
- [`commits.md`](commits.md): commit ordering, durability and failure.
- [`growth.md`](growth.md): retention, admission and the map census.
- [`testing.md`](testing.md): the store's tests and worlds.
- [`opening-owners.md`](opening-owners.md): owner sets at open.
- [`opening-rehearsal.md`](opening-rehearsal.md): rehearsing an upgrade and going back.
- [`settling.md`](settling.md): values put again at the binary's layouts while the store serves.

## 1. In one page

- **skein-kv, in the engine's process.** The engine's protocol layer owns
  `jig-protocol-store`, a component that owns two skein-kv stores (skein's
  `data/kv.md`): the store, with every record jig and the application keep,
  and the secrets' store. Keys are ordered bytes, values are bytes, both
  in memory, made durable by an append-only log and snapshots. There is
  no store process, no wire and no SQL: the engine is the store's only
  client.
- **One component, two owners of records.** jig's records (the core's
  and its children's) reach the component typed, in the core's
  vocabulary, and the component encodes them. The application's records
  (its connectors' and its own children's) reach it as key and value
  bytes the application encodes, under the application's tag. Both go
  into one commit (section 3).
- **Keys sort as the domain's keys do,** written with skein-kv's key
  writer, with a tag per owner, then per family. Every range the domain
  asks for maps to a short, fixed list of intervals (`keys.md`, section 1).
- **Records from schemas,** with skein's generator: a codec per family,
  versioned, with fixed-rule values built from schema text and fuzzing.
  A format row names the versions, each owner's level and the deployment;
  a floor row names the oldest
  store the build opens. A store of another version or deployment
  refuses to open (`records.md`).
- **Values name their layout.** A registered family's value carries its
  value type's layout id, an entry of the store's registry; the open
  checks that each layout held converts to the binary's, and each value
  is converted as it is read (`layouts.md`).
- **Generations.** The maps live in numbered generations under the
  deployment's directory, `active` naming the open one. A store the open
  must write before serving (a binary layout to register or a job
  pending) is upgraded at open in a new generation made
  by links, checked, and switched to by one durable rename; a crash
  anywhere leaves one whole generation active (`README.md`, section 2,
  and `opening.md`, section 2, and `opening-owners.md`, section 1, and `opening.md`, section 4). Jobs at open walk the map for their
  owners when a binary needs it in another shape (`jobs.md`).
- **Hot records apart from cold.** Live tasks and goals' projections
  keep what an act changes rarely in rows of their own, so a write puts
  what changed (`parts.md`).
- **One decision, one commit, the same number.** Each of the root's
  commits is one skein-kv commit with the same number. Group commit folds
  bursts into one sync. A failed or refused commit stops the engine
  (`commits.md`, section 1).
- **Loads are answered in the step that asks,** from memory, in bounded
  pages: the restart's live ranges, parties' reads with their joins, a
  child's pages, single records (`loads.md`). Only a turn's bytes, read
  from its transcript, take io.
- **Ended tasks are found through indexes,** the tasks child's: by their
  requester party, and by their project for those flagged listed, each
  by label and newest ended first. Live tasks are the tasks child's
  memory, never the store's to list (`loads.md`, 3, 4).
- **History in task logs:** a task's history and its turns are two of
  skein-kv's payload logs in the store's generation, each record synced
  before the commit that accepts it, compressed, read through a view of
  the map and the logs, and dropped whole (`logs.md`). Turns written
  before them stay in files of their own; the map keeps where each
  task's current window starts, so a resumed run reads only its window
  (`transcripts.md`).
- **Secrets in a second store,** in a directory only the engine's user
  may read, written by components alone: minted credentials' digests,
  and LLM accounts' records and keys, each kept before it is used
  (`secrets.md`).
- **Growth is bounded by retention and fill:** the domain forgets an
  ended task once read, or drops its turns, shrinks it to its summary
  and then erases it, each past a horizon, on its timer, by the task's
  retention class (`retention.md`), and refuses new work above a
  high-water mark of the map (`growth.md`, section 1).
- **The real store in every world:** the domain worlds run the component
  over skein-kv's in-memory mode, with an in-memory stand-in for
  transcripts' files, and the store's own world crashes it on skein's
  simulated disk (`testing.md`, section 1).
- **Two crates:** the component above its records' translation,
  `jig-protocol-store-records`, which it re-exports as `codec` (`crates.md`).

## 2. Where it sits

```
the application's root          commits (jig's writes typed, its own wrapped), loads, restart steps
        │ the root's store vocabulary, through the protocol layer's routing
        │   (the application's records encoded by its own codecs on the way down, decoded on the way up)
jig-protocol-store              keys and records to bytes; ranges to intervals; parties' reads and joins;
        │                       transcripts' files; secrets
        │ skein-kv's vocabulary (skein's data/kv.md, section 2)      │ io's file requests (skein's runtime/io/files.md)
skein-kv ×2                     the store; the secrets' store
        │ io's file requests
skein-io                        files: the ring, or the simulator's disk
```

- **A component of the engine's protocol layer,** step code under the
  subset (skein/docs/foundation/programming-model.md, section 10), with its limits, worst case,
  deadlines and most outputs per entry point. It depends on skein-lib,
  skein-kv, io's file vocabulary, skein-codec, its generated codecs and
  its records' translation, and the core's vocabulary (its keys, records, ranges and reads). It
  depends on no application's crate, and links no smith crate: turns are
  bytes to it.
- **On disk,** in the deployment's directory (`../configuration-engine.md`,
  section 1), which nothing else writes. The two maps live in numbered
  generations; `active` names the one a start opens, and an upgrade
  makes the next from the active one's files, linking the snapshot and
  the log segments skein-kv writes no more and copying the rest,
  upgrades it and switches to it (`opening.md`, 2). Two generations may
  share a file under two names, which neither writes. Transcripts
  sit outside the generations: an upgrade keeps their tree, while a
  restore replaces it (`restoring.md`):

```
<deployment>/
├── active          the active generation's number: its decimal digits and a newline
├── restore         an outstanding restore intent, resolved before active (restoring.md)
├── generations/
│   ├── 7/          the previous generation, kept until the next switch
│   │   ├── store/      skein-kv: snapshot, log segments, task logs (logs/)
│   │   ├── secrets/    the secrets' store: the directory 0o700, its files 0o600
│   │   └── manifest    what made this generation (below)
│   └── 8/          the active one, as 7
├── checkpoints/    the state at a commit, each laid out as this directory (checkpoints.md)
└── transcripts/    one file per task, in directories by the task's number
```

- **A first start** is a directory with no `active`, and none of
  `store/`, `secrets/` and `transcripts/` at its top (the layout before
  generations, below). A `generations/` that holds only a `1/` whose `store/` and
  `secrets/` hold no file, beside at most its manifest or the manifest's temporary
  (`manifest.skein-<16 hex>`), is a first start a cut stopped before `active`
  named it (the stores' first open, which makes their files,
  comes after `active`): it is removed, with any temporary io's durable replace
  left beside `active` (`active.skein-<16 hex>`; skein's `runtime/io/files.md`, 2).
  A `generations/` that holds anything else, with no `active`, is
  refused, `Generations::Unnamed`, and left as it is: a generation
  whose `active` was lost, which the operator names again from a
  backup. The start then makes `generations/1/` with its empty
  `store/` and `secrets/` (each directory durable as io makes it), and
  generation 1's manifest; then writes `active`, `1`, by io's durable
  replace expecting no file there (`Store` with `Absent`); then makes
  `transcripts/`. The maps are empty: the root's first commit writes
  the format row (`records.md`, 3).
- **Strays.** `active` names a generation only once it is whole
  (`opening.md`, 4), and at most it and its previous (`active` less one) stand:
  every other entry of `generations/` is a stray, a copy a crash left
  before its switch or a generation a crash left in its drop, and every
  open removes them once the active generation has opened and nothing
  refuses (`opening.md`, 1). Nothing reads the previous generation; it is what an
  operator goes back to, until the active one's first commit of its
  own (temper's `draft/upgrades.md`, 3.2).
- **The manifest,** a file per generation, written by io's durable
  replace into the generation's directory before `active` names it,
  and never changed: a record of the component's own schema
  (`Manifest`, beside the format row in `component-v3.schema`, `records.md`, 3),
  encoded and versioned as every record. It names:
  - `generation`, the generation's number, and `from`, the one it was
    upgraded from, none for a first start's or a restored generation's;
  - `before` and `after`, each owner's levels (`records.md`, 3);
    a first start's before levels are all none, and its after levels
    the binary's;
  - `counts`: each kept family's count of records before the registry's
    commit and the jobs and after, one per family the binary's sets keep,
    in their order (a first start's each 0 and 0; none with no sets).
    These are reported, not judged by the check: a job may add or erase
    rows (`opening.md`, 2, step 5);
  - `store` and `secrets`, the last commit of each map when the upgrade
    ended: the store's, which the deployment's header names (`commits.md`, section 1),
    and the secrets' store's; for a first start, 0 and none respectively;
  - `written`, the wall-clock time it was written, `env.wall` in its own
    unit.

  An ordinary open reads no manifest; resolving a restore may read its
  staged generation's (`restoring.md`, 5), and going back reads its active
  one's (`opening-rehearsal.md`, 1). It is also the operator's record of what an
  upgrade did, printed by temper's `upgrade --check` and `versions`
  (temper's `draft/upgrades.md`, 3.1).
- **A directory in the layout before generations** (`store/`, `secrets/` or
  `transcripts/` at its top, and no `active`) is a store from an older
  build, made before generations. Generations never have the first
  two at the top, and a first start makes `transcripts/` last, after
  `active`, so no cut leaves it alone; a new store beside it would
  number its tasks again over its files. Open refuses it, `OlderBuild`
  with `Older::Directory`, before anything is written, and leaves it
  untouched (`records.md`, 3). The operator stops the service, moves `store/`, `secrets/` and
  `transcripts/` out of the deployment's directory together (its
  transcripts belong to its tasks), and starts again, which is a first
  start. The refusal's text says so, naming
  the directory:

  > `<deployment>` holds a store from an older build, in the layout
  > before generations (`store/`, `secrets/` or `transcripts/` at its
  > top), and this start has changed nothing in it. No build translates
  > a store from before generations, build `<bridge>` included, so every
  > start refuses this one until it is moved. To start afresh, stop the service, move
  > `store/`, `secrets/` and `transcripts/` out of `<deployment>`
  > together, and start again: nothing is deleted, and the next start
  > makes a new, empty store. The new store has no tasks, sign-ins,
  > accounts or keys, and every client signs in again.

- **`OpenRefusal::Format`** refuses another version of a family (`records.md`, 3),
  putting the family's lower-case name in place of <family>:

  > `<deployment>` holds a store whose <family> family is at another
  > version than this build's, and this start has changed nothing in it.
  > No build translates a store from before the baseline, build
  > `<bridge>` included, so every start refuses this one until it is
  > moved. To start afresh, stop the service,
  > move `active`, `generations/` and `transcripts/` out of `<deployment>`
  > together, and start again: nothing is deleted, and the next start
  > makes a new, empty store. The new store has no tasks, sign-ins,
  > accounts or keys, and every client signs in again.

- **`OpenRefusal::Deployment`** refuses a format row naming another
  deployment than its header (`records.md`, 3):

  > The store in `<deployment>` names another deployment than its header
  > does, so its files come from two deployments, and this start has
  > changed nothing in it. Restore `<deployment>` whole from one backup,
  > or, to start afresh, stop the service, move `active`, `generations/`
  > and `transcripts/` out of `<deployment>` together, and start again:
  > nothing is deleted, and the next start makes a new, empty store.

- **`OpenRefusal::Numbers`** refuses a recovered last commit other than
  the one the header records (`commits.md`, section 1):

  > The store in `<deployment>` does not end at the commit its header
  > records, so a file was lost, cut or restored apart from the others,
  > and this start has changed nothing in it. Restore `<deployment>` whole
  > from one backup, or, to start afresh, stop the service, move `active`,
  > `generations/` and `transcripts/` out of `<deployment>` together, and
  > start again: nothing is deleted, and the next start makes a new,
  > empty store.

- **`OpenRefusal::Failed`** stops an open when a file fails or a store
  cannot recover (`opening.md`):

  > A file in `<deployment>` could not be read or written, or its store
  > could not be recovered, while this start opened it, and the start
  > stopped there. Check the disk's space and the directory's permissions,
  > and start again. If the next start fails the same way, restore
  > `<deployment>` from a backup.

- **Each `Older` names its own layout's fix,** in a sentence its
  refusal's text carries whole:
  - `Directory`: "To start afresh, stop the service, move `store/`,
    `secrets/` and `transcripts/` out of `<deployment>` together, and
    start again: nothing is deleted, and the next start makes a new,
    empty store."
  - `Layout` and `Unleveled`: "To start afresh, stop the
    service, move `active`, `generations/` and `transcripts/` out of
    `<deployment>` together, and start again: nothing is deleted, and
    the next start makes a new, empty store."
  - `Floor` and `Unregistered`, the stores the bridge can upgrade:
    "Start build `<bridge>` on it and let it open, then this build."
    `Behind` also names that build, which brings the store to this
    build's levels (`opening.md`, 1.8; `<bridge>` is `records.md`, 3's
    `FLOOR_BRIDGE`).
- **A backup** copies, with the engine stopped, the deployment's
  directory whole, or is a checkpoint: the state at one commit, taken
  while the engine serves or over a stopped deployment, mostly by hard
  links (`checkpoints.md`). A first start refuses a `checkpoints/` that
  holds anything (`checkpoints.md`, 7). Restoring a checkpoint over a stopped
  deployment first takes a safety checkpoint, then stages and rolls forward
  its intent (`restoring.md`).
- **Opened before the root starts** (`opening.md`; `../service.md`,
  section 3).

## 3. The boundary

What the component takes and answers, as the engine's protocol layer
routes it. The core's side is `../../domain/engine/decisions.md`, section 1 and `../../domain/engine/restart.md`;
the root wraps it, and the protocol layer's routing unwraps it, encoding
the application's part with the application's codecs.

| Down, to the component | Up, from it |
|---|---|
| **commit** n: jig's writes, typed (the core's saves and erases of its records and its children's, a copy of the bytes under one of its keys to another, and a task's transcript or history log dropped whole); the application's writes, each a key's bytes and a value's bytes, or an erase | **committed** n, with the store's fill (`growth.md`, 1.2); **failed** n, with why (`commits.md`, section 1); or **FrameTooLarge** n, with task, full encoded bytes and page limit (`commits.md`, section 1) |
| **restore** a restart step's range: jig's (the core's live families, `loads.md`, 2), or the application's, as byte intervals under its tag | jig's records, typed, then done; the application's rows as bytes, then done |
| **load** a page of a child's range: jig's, or the application's intervals | the page's rows, typed or as bytes, and a cursor if more remain |
| **read** a party's read: its scope, position, direction, rows and bytes (`../../domain/engine/decisions.md`, 1.3) | the page, each row with its joins, and its cursor; or busy, or gone |
| **get** a record by key: jig's or the application's | the record, or none |
| **transcript**: a task's current window, from the turn the core names as its start; a turn by its task, attempt and number; a page of a task's turns from either end | the turns, with their fields and bytes, and a cursor; or unread |
| **secrets**: keep, replace, erase, get and load, by owner and number; carry a value under a drawn number; adopt a carried value into its source's row (`secrets.md`) | kept, stale, erased, the value or none, a page, or the carried number and digest |

- **Turns are records of the core's** (`../../domain/engine/decisions.md`, 1.4): the
  core saves each accepted turn, keyed by its task, attempt and number,
  with its spend, the last message it read, its time, whether it opens a
  window, and its bytes. The component keeps the bytes and those fields
  in the task's transcript file, and the transcript's places in the map
  (`transcripts.md`). The core never erases a single turn: a task's transcript
  goes whole, by retention (`growth.md`, 1.1).
- **Reads see what is durable.** A load, a read, a get or a transcript
  sees every commit answered before it was asked, and nothing that is
  not durable (skein's `data/kv.md`, section 2). Each load of the core's is
  held until every commit made before it is answered
  (`../../domain/engine/vocabulary.md`, 1.2, and `../../domain/engine/decisions.md`, 1.3), so it sees what the decisions
  before it wrote.
- **One terminal per request,** tagged with the asker's token
  (skein/docs/foundation/programming-model.md, 4.4).

## 4. Keys

Moved to [`keys.md`](keys.md), section 1.

## 5. Commits

Moved to [`commits.md`](commits.md), section 1.

## 6. Growth

Moved to [`growth.md`](growth.md), section 1.

## 7. The application's part

- **Its families,** under its tag (`keys.md`, 1.2), keyed by its own encoder by the
  same rule, encoded by its own codecs, with its store version (`records.md`, 2).
- **Its restart ranges** as byte intervals under its tag, each live
  interval of each connector, served in the restart's step for that
  connector (`../../domain/engine/restart.md`).
- **Its loads and gets** by byte intervals and keys, answered with bytes.
- **Its indexes,** as records of its own, by the same rule (`loads.md`, 4).
- **Its secrets,** under its tag in the secrets' store (`secrets.md`).
- **What it writes is in the same commits** as jig's: one decision, one
  commit, whoever's records it holds.

## 8. Limits and the worst case

- **The component's limits:** both stores' (skein-kv's, their maps'
  budgets among them), the bytes a read fetches from task logs
  (`Limits::fetch`, `logs.md`, 4), the fill's high-water mark, the open files,
  the frames' largest bytes (a turn's, from the domain's limits), the
  queue of commits behind transcripts' syncs, the reads in flight and the
  joins a page may make.
- **Three file drivers,** one for each store (skein's `data/kv.md`, section
  12) and one for transcripts, so a turn's sync never queues behind a
  store's write, nor a store's behind a turn's.
- **The worst case** is both stores' (skein's `data/kv.md`, section 7: each
  map's budget, its queued commits and its buffers, which follow its
  chunk target and the largest row it holds), the component's buffers
  and tables, and the three drivers' (skein's
  `runtime/io/files.md`). The upgrade's peak (`opening.md`, 2:
  one generation's maps open, the copy's chunk, the registry's planning
  bounds (`layouts.md`, 7), and a job's page and commit room
  (`jobs.md`, 3)) is counted apart from serving's: the root never serves
  while it upgrades. No map patch machine or closing-window walk is
  held. Fixed scenarios check measured heap peaks
  against named ceilings (skein/docs/foundation/programming-model.md, 6.3).
  Serving's holds no frames' machine: a frame converts in the read
  edge's buffer and workspace, the buffer grown to the largest frame
  converted, at most the page's bytes (`transcripts.md`, 6;
  `layouts.md`, 7), and settling holds one page (`settling.md`, 5).
- **Whole files:** the manifest (section 2) is read and written whole,
  and at its bounds, 64 kept families each named in 64 bytes, it is 5464
  bytes (88, and 84 for each family). The store's file driver's
  whole-file bound is at least that, checked at startup with the worst
  case, or the start is refused naming it; a test world's drivers are
  made with such a bound (skein-io's `file_layer::Limits`, its `whole`).
- **A page read whole:** a transcript frame of up to the page is read
  back in one read, so each file driver reads at least the page, checked
  at startup, or the start is refused naming it; a test world that
  raises its page raises its drivers' read with it.
- **Checked whole at startup** with the root's and every component's
  (`../limits.md`).
- **Checked once:** `check` gives the check's result, `Checked`: the
  limits with the codec view derived from them; `runtime_limits` gives
  the runtime one, its turn's transcript bound narrowed. The component
  is made over one (`Store::over`, `Store::on_directory`,
  `host::Hosted::new`) and derives no bound again, and an install takes
  the runtime check's view as it is. A reopen over the same maps at the
  same limits takes the `Checked` the first component was made over; one
  at other limits is checked afresh. `Store::checked` gives the one in
  force.

`jig_protocol_store::host` hosts the three file drivers around the store for any engine: the drivers, their kernel hand-offs and their adoption and close. An application's service routes its own keys and records to it and writes no driver of its own.

## 9. Testing

Moved to [`testing.md`](testing.md), section 1.

## 10. What it owes and is owed

The domain owes 10.1 to 10.9; skein owes 10.10, and the application
15.11.

### 10.1 Live before history

In each child's keys: the tasks child's history and ended tasks after
its live families (stubs among these), decided proposals apart from
pending ones, and past funding periods apart from live ones (`keys.md`, 1.3). A
pool's, a recurring task's or a period's ledger closes into the past
periods once nothing is reserved on it and its project has a newer
period, so a live task whose budget is 0, which reserves nothing, may
still name a closed one as its funder. The start reads no past period
for it: it links such a task by its budget of 0 and its funder's period
being older than its project's newest live one, and refuses a task
whose ledger is not live on any other ground, as before. The task's end
posts nothing to the closed ledger, and what it spent, if anything,
goes to its project's current period (`../../domain/authority-budgets.md`,
section 1). No ledger is kept open for such a task, and no task is
moved off its funder. An escalation's decider is stored with its kind,
a task's or a person's, and linked by it (`../../domain/tasks/proposals.md`,
*An escalation's decider has its kind*; `jobs.md`, 1, for the stores
that kept it as a bare number): each party a live task names is linked by its kind, and a
start refuses what it cannot link;

### 10.2 The restart reads live work only

The inbox's census reads unread results from the results' index and
ended conversations from the family of ended read positions, not every
ended task, joining each to its record by a point read; the census hands
each ended read position up with its record, and the people child keeps
none (`loads.md`, 2; `../../domain/people/README.md`, section 5);

### 10.3 Indexes for parties' reads

Each party's inbox by time; ended tasks by requester party, and by
project for those flagged listed, each keyed by label and ended at
reversed, the flag read from the task, never from a template; each
task's ended delegates; a subtree's delegates for its history (`loads.md`, 3, 4
);

### 10.4 Turns as the store keeps them

Saved with whether each opens a window, never erased one by one, a
task's transcript dropped whole; a resume reads the current window, from
the turn the core keeps as its start (`transcripts.md`, 3; `../../domain/engine/decisions.md`, 1.4, and `../../domain/engine/executors-transcripts.md`, 1
);

### 10.5 Fill

With each commit's answer, and new work refused above the high-water
mark (`growth.md`, 1.2);

### 10.6 Retention

By each task's retention class, summaries of ended tasks kept,
transcripts dropped, answered keys expired, and the remains of tasks
forgotten once read collected, each on the domain's timer (`growth.md`, 1.1;
`retention.md`); a task forgotten once read erased in one decision (`growth.md`, 1.1; `../../domain/people/README.md`,
section 5);

### 10.7 Keys for single reads

A question's line and its answer's, an escalation's hold, and a task's
proposal's row in its proposer's history, each named by its position in
its task's record, as a reply's is, an answer's with the task whose
activity holds it, so a read finds it by key, never by a scan (`loads.md`, 3); and
each open question its task asked, by its number, answering party,
logical addressed party and question activity position
(`QuestionCredit`, `records.md`, 1), so a person's answer is credited to its asker,
a move carries the question to the asker's new requester under the same
number, and an answering task's end lifts its asker's keep with a notice
(`../../domain/tasks/lifecycle-endings.md`, 1; `../../domain/people/README.md`, section 5);

### 10.8 Read positions over conversations

A live conversation's in the people child's family, loaded at start;
moved, by the decision that ends its task, to the family of ended read
positions, read on demand; erased with the task (`loads.md`, 4, `growth.md`, 1.1).

### 10.9 The accounts' family

The next number; each account with its holder, LLM provider, subject,
display name and uses; each source with its place in its holder's order,
kind, name for a program's, and its hold, limited with its reset,
refused or moving; each account's facts, at most 24, a record apart that goes
with its account (`../../domain/accounts-lists.md`, 1); restored at start by fit and
link, a set past its limit refusing the start and a link the start cannot make
refusing it too (`../../domain/accounts.md`, 3.6), and facts whose account is gone dropped
as they are read; a source's record or key named by the
source's number, and erased when the source goes (`keys.md`, 1.4, `secrets.md`;
`../../domain/accounts-sources.md`, 2). The installation's identifier is the
component's (`keys.md`, 1.4, `records.md`, 3).

### 10.10 What skein owes

io's `OpenWrite` and `Cut` (skein's `runtime/io/files.md`), for transcripts.md, 2 and 4. skein-kv serves as it is: its in-memory mode, its key writer, its
fill. Its accounts driver takes the owner's store as keeper, made with
the records it holds (skein's `oauth/accounts.md`, 4).

### 10.11 What the application owes

Its keys' encoders, its records' codecs, its store version, its live
intervals, and its indexes, by the rules of section 7.

## 11. Open questions

- **Upgrades** register layouts and run jobs in a copy, then switch to it
  (`opening.md`, sections 1 to 2, and `opening-owners.md`, section 1, and `opening.md`, section 4). Still open:
  - the free space checked before the copy (`opening.md`, 2), which
    links its maps' sealed files and so needs room for its newest
    segment, its secrets and its registry and job commits;
  - what the copy's working buffers pay (`opening.md`, 2): it reads and
    writes each file it copies in pieces of skein-kv's `Limits::chunk`,
    the row ceiling, not its chunk target (skein's `data/kv.md`, 5 and 7),
    and the switch's digest check reads `active` into a buffer of the
    drivers' whole read, which that ceiling sets (`host::max_read`). So
    an upgrade pays about 60 MB on standard however small its rows; a
    deployment's budgets meet this (skein's `data/kv.md`, section 12).
- **Backups:** copying the deployment's directory whole while the engine
  is stopped works; a copy that keeps hard links keeps the files two
  generations share as one, and one that does not stores them twice and
  restores a deployment that opens the same. A copy while it serves is
  a checkpoint's (`checkpoints.md`), restored by `restoring.md`.
- **Finding a turn faster:** a row per turn in the map, if walking an
  attempt's frames proves slow for parties' reads.
- **Values on disk,** with only keys in memory, if a deployment outgrows
  its map's budget after retention (skein's `data/kv.md`, section 12).
- **Search** over tasks, notes and transcripts: an index of the domain's,
  or a file-based one beside the store.
- **Several engines:** the store is one process's; the log of commits is
  what would be replicated.
- **One remap of a mask:** `jig-domain-authority`, `jig-domain-tasks` and
  `jig-domain-people` each own a `Models` with the same `remap` (`records.md`, 1), since
  step code in jig has no generics. Sharing them takes one type the three
  name.
