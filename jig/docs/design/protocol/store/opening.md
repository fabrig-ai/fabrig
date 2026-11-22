# Opening and restart

1. **The shell opens the deployment's directory,** and the component
   finds the active generation (1) and opens both its stores;
   skein-kv recovers each: the snapshot, then the log's longest valid
   prefix (skein's `data/kv.md`, section 3). A store the open must write
   before serving (`layouts.md`, 5.5: a binary layout to register, a job
   pending) is upgraded in a
   new generation, and the open goes on over it, pending, which becomes
   the active generation once the root has restored from it (2).
2. **The component reads its own rows,** and the core's that it checks:
   the format row, refusing another version, another level or another
   deployment (`records.md`, 3); the
   deployment's header, refusing a last commit other than skein-kv's
   recovered one (`README.md`, section 5); the run proofs' tasks' committed lengths,
   cutting each file to its length (`transcripts.md`, 4); and the installation's
   identifier, which replaces the one the shell drew (`README.md`, 4.4). An empty
   store gets its format row and the installation's identifier in the
   root's first commit, with the deployment's header.
3. **The root starts** (`../service.md`, section 3): the core's restart
   script asks for one step at a time, and the store serves each step's
   ranges (`loads.md`, 2). The accounts child's want saves its never-reused
   counter when it advances, and the ordinary account/source rows it
   makes for providers whose source is none. A first start with no
   accounts needs no such commit. Restarts retain those same numbers.
4. **Secrets are served** to the protocol layer's other components once
   the start's wires are durable, at its switch step (`records.md`, 3;
   `../../domain/engine/restart.md`), at every start and never before:
   each signed-in source's record to make
   skein's driver with, and each key to the accounts' table, when the
   accounts child names the restored sources. After all are named, the
   component sweeps unnamed source secrets (`../llm.md`, 5.1).

## 1. Generations at open

Before a map opens, the component reads the deployment's directory
(`README.md`, section 2). Except for resolving an outstanding restore intent
(1.1), nothing is removed or made until the active generation has
opened and nothing refuses. The shell seeds each of the store's file
drivers before the open, through `seed`, whose `Seeded` witness
`on_directory` takes: io names a `Store`'s temporaries from a driver's
randomness (skein's `runtime/io/files.md`), and an unseeded driver refuses every
`Store`, a first start's write of `active` among them, so a shell that
seeds none does not build. The witness proves that three drivers were
seeded, not that they are the three the shell serves the store with.

### 1.1 The lock

io's `Lock` on the deployment's root, held as long as the process
(skein's `runtime/io/files.md`). `Busy` refuses, `Generations::Busy`: another
process has the deployment open, and two starts never copy, remove or
switch beside each other. Once locked, an outstanding `restore` intent is
rolled forward before `active` is read, or refused `Generations::Unmatched`
(`restoring.md`, 5).

### 1.2 `active`

A whole-file load of at most 21 bytes: a generation's number in decimal,
with no leading zero, and a newline; anything else refuses,
`Generations::Unreadable`. With none, the directory is a first start's,
or refused as the layout before generations (`Older::Directory`) or as
generations no `active` names (`Generations::Unnamed`), as `README.md`, section 2
says.

### 1.3 The active generation's maps

Its `store/` and `secrets/` must stand, or the start refuses,
`Generations::Missing { generation }`.

### 1.4 The stores open

The stores open (step 1), and the format row, the floor, the layouts and the
deployment's header are read and checked (step 2, `records.md`, 3). Every refusal of
the levels, the floor, versions, layouts and deployment comes here, and nothing has
been written but skein-kv's recovery of the active generation's maps
(its next segment, a leftover snapshot's removal, and the snapshot a torn
tail or a replay past its bound starts; skein's `data/kv.md`, sections 3 and
5).

### 1.5 The strays removed

Every entry of `generations/` but the active generation and its
previous, and any temporary io's durable replace left beside `active`.
io has no recursive removal: each directory is listed from its first
page, each file in it removed, and listed again until it is empty (a
removal moves what a later page would show), then removed itself,
deepest first; then `generations/` is synced. A start never removes the
active generation or its previous. Then each checkpoint temporary in
`checkpoints/`, the same way, and `checkpoints/` synced: never a
checkpoint (`checkpoints.md`, 7). With no intent standing, the restore's
`transcripts.staged/`, `transcripts.aside/` and `restore.skein-<16 hex>`
are removed too, each tree deepest first, then the top synced
(`restoring.md`, 5).

### 1.6 `transcripts/`

`transcripts/` is made where it is missing.

### 1.7 At the binary's levels

At the binary's levels, the open reads the registry and plans its
layouts (`layouts.md`, 5.3), refusing a layout it cannot read before
anything is written. It reads it only once the format row's versions
are those its levels want (`records.md`, 3): a store at another version
is refused first. With nothing to write (`layouts.md`, 5.5), it goes
on with the rest of step 2 (the run proofs' cuts, the installation's
identifier), its read edge converting each value (`layouts.md`, 4).
With something to write, it upgrades in a new generation (2). A store
behind them, or holding values and no registry, is refused (1.8): the
refusals name `FLOOR_BRIDGE`, the build that brings it to them
(`records.md`, 3).

### 1.8 Its refusals

Each refusal names the deployment's directory and what to do: another
process holding the lock is stopped first; an unreadable or missing
generation is restored from a backup, or `active` written again to name
one that stands; an older build's is moved aside (`README.md`, section 2). Each
`Generations` refusal's text, whole, as `OpenRefusal::words` gives it
(the deployment's directory where `<deployment>` stands):

- `Busy`: "Another process has `<deployment>` open, and this start has
  changed nothing in it. Stop that process first, then start again."
- `Unreadable`: "The `active` file in `<deployment>` does not name a
  generation, and this start has changed nothing in it. Restore
  `<deployment>` from a backup, or write `active` again to name a
  generation under `generations/` that stands, and start again."
- `Missing`: "The generation `active` names in `<deployment>` has lost
  its `store/` or `secrets/`, and this start has changed nothing in it.
  Restore `<deployment>` from a backup, or write `active` again to name
  a generation under `generations/` that stands, and start again."
- `Unnamed`: "`<deployment>` holds generations that no `active` names,
  and this start has changed nothing in it. Name one again from a
  backup of `<deployment>`, or write `active` to name the generation
  under `generations/` to open, and start again."
- `Checkpoints`: "`<deployment>` holds checkpoints but no `active` names a
  generation, and this start has changed nothing in it. Restore
  `<deployment>` from a backup or from one of its checkpoints, or, to
  start afresh, move `checkpoints/` out of `<deployment>`, and start
  again." A first start beside checkpoints (`checkpoints.md`, 7).

- `Unmatched`: "`<deployment>` holds an unfinished restore whose files no
  longer match its intent, `restore`, and this start has changed nothing
  in it. Something outside the engine changed them: restore
  `<deployment>` from a backup, or copy one of its checkpoints into
  place, and start again."
- `Unfinished`: "`<deployment>` holds a restore that has not finished,
  and this rehearsal has changed nothing in it. Start the deployment,
  or take a checkpoint of it, which finishes the restore, then rehearse
  again."

- `OlderBuild { why: Unleveled }`: "`<deployment>` holds a store from before
  the baseline, the first build whose stores later builds upgrade, and this
  start has changed nothing in it. No build translates a store from before
  the baseline, build `<bridge>` included, so every start refuses this one
  until it is moved. To start
  afresh, stop the service, move `active`, `generations/` and `transcripts/`
  out of `<deployment>` together, and start again: nothing is deleted, and
  the next start makes a new, empty store. The new store has no tasks,
  sign-ins, accounts or keys, and every client signs in again."
- `OlderBuild { why: Behind }`: "`<deployment>` holds a store from an
  older build, at patch levels behind this build's, and this start has
  changed nothing in it. This build keeps no patch: start build
  `<bridge>` on it and let it open, which brings it to this
  build's levels, then start this build."
- `OlderBuild { why: Floor }` (`records.md`, 3): "`<deployment>` holds a
  store older than this build opens: it has no floor, or one below this
  build's, and this start has changed nothing in it. Start build
  `<bridge>` on it and let it open, then this build."
- `OlderBuild { why: Unregistered }` (`layouts.md`, 5.1): "`<deployment>`
  holds a store older than this build opens: it has no registry of its
  layouts, and this start has changed nothing in it. Start build
  `<bridge>` on it and let it open, then this build."

`<bridge>` is `FLOOR_BRIDGE` (`records.md`, 3) in every refusal of a store
older than the build, and only there.

- `Ahead`: "`<deployment>` holds a store from a later build, at patch levels
  past this build's, and this start has changed nothing in it. No patch runs
  backwards: start the later build again, or restore `<deployment>` from a
  backup this build made."
- `Set`: "This build's owner sets do not check, and this start has
  changed nothing in `<deployment>`. The build is wrong: start the build
  before it again."
- `Check { why: Load }`: "The check before the switch found a record the
  restart could not load within the deployment's limits in the copy of the
  store this start patched in `<deployment>`, and the generation it patched
  from stays active, unchanged. Every start of this build checks a fresh copy
  the same way: start the build before it again, which opens that generation
  as it was."
- `Check { why: Private }`: "The check before the switch found a file in the
  secrets' store of the copy this start patched in `<deployment>` that is not
  private to the service's user, and the generation it patched from stays
  active, unchanged. A start makes its copy's files private, so another
  process or user changed one: make sure nothing else writes in
  `<deployment>`, and start again."
- `Layout`: "`<deployment>` holds values at a layout this build cannot
  read, and this start has changed nothing in it. Start the build before
  it again, or a later build whose schema converts that layout, by its
  defaults and former names or by a written job." The shell names the
  family, the value type, the layout's id, why and how many values hold
  it after it (`layouts.md`, 5.3). For a retired value type's value
  (`Retired`), why is "the value type is retired, and its keys hold a
  value".
- `Job`: "A job this build runs at open refused a record while this
  start upgraded the store in `<deployment>`, and the generation it
  upgraded from stays active, unchanged. Every start of this build
  refuses the same record: start the build before it again, which opens
  that generation as it was." The shell names the owner, the job, the
  key and why after it (`jobs.md`, 3).

The refusals at open, beside `README.md`, section 3's others:

```rust
pub enum OpenRefusal {
    Format { family: Family, found: u32, wanted: u32 },
    Deployment,
    Numbers { store: u64, header: u64 },
    Failed,
    OlderBuild { why: Older },                                 // a store this build does not translate
    Ahead { owner: Owner, found: Option<u32>, wanted: Option<u32> }, // no level runs backwards
    Generations { why: Generations },
    Set { owner: Owner },                                      // an owner set that does not check (3)
    Check { why: Check },                                      // the check before the switch (2, step 5)
    Layout { family: Box<str>, value_type: u16, id: u16, values: Option<u64>, why: Unplanned }, // layouts.md, 5.3; the family as its row's key names it, which may be one the binary lacks
    Job { owner: Owner, job: Box<str>, key: Option<Box<[u8]>>, why: Box<str> },   // jobs.md, 3
}
pub enum Unplanned { Underived(skein_codec::layout::Underived), Text, Undeclared, Retired } // no plan; a text missing or unparsed; a value type the binary lacks, in a family it declares or one it does not; a retired value type's keys holding a value (layouts.md, 2)
pub enum Older { Layout(Family), Directory, Unleveled, Floor, Behind, Unregistered }
pub enum Generations { Busy, Unreadable, Unnamed, Checkpoints, Unmatched, Unfinished, Missing { generation: u64 } }
pub enum Check { Load { key }, Private { name } }
```

`Older::Layout` is the component's own row at the binary's version that
does not decode (`records.md`, 3), the only family it names now;
`Unleveled` a level none where the binary's is set, the baseline's
among them; `Floor` a missing floor row or one below `FLOOR`, before a plan or
rehearsal decoding; `Behind` a level behind the binary's; `Unregistered`
a store holding values and no registry (`layouts.md`, 5.1). These three
name `FLOOR_BRIDGE`, the build that brings such a store to this one
(temper's `draft/store-floor.md`, section 3); `Check` names what
failed: a row that does not load, by its key; a secrets' file
that is not private, by its name. A job's write too large for a commit
alone is the job's refusal (`jobs.md`, 3). There is no eager map patch
whose output needs a separate size check.

## 2. Upgrading a copy

The component's own phase, before the root starts, when the open must
write before serving (`layouts.md`, 5.5): a binary layout is not
registered, or a job is pending (`jobs.md`, 2). The store is at the
binary's levels, or 1 has refused it (`records.md`, 3). No record is
rewritten but by a job or the seal: the copy is
upgraded by the registry's commit and the jobs alone (temper's
`draft/store-floor.md`, 4.2). From the active generation `A` to
`A + 1`:

1. **Plan,** over `A` as 1 opened it: the layouts planned, each held
   layout's, refused before anything is written where one has no plan
   (`layouts.md`, 5.3); the owner sets checked (3); the header decoded
   through its registry plan, then by the binary's codec and checked,
   as is the deployment; the room
   (below) against the limits. Every refusal of
   `records.md`, 3 comes here. Then `A`'s stores
   close: no transcript has been cut, which the open over the copy
   does (step 7). Then `A - 1`, where one stands, is dropped, since once `A + 1`
   exists going back is to `A`, which nothing here changes: renamed out
   of the numbers (`generations/dropped`), `generations/` synced, then
   removed as 1 removes a stray, so that a cut never leaves part of
   a generation where the previous stands. Removing a name removes no
   file another generation names: a file `A - 1` shares with `A` stays
   `A`'s.
2. **Link and copy** `A`'s `store/` and `secrets/` into
   `generations/A + 1/` (temper's `draft/store-shape.md`, section 5),
   its `store/` and `secrets/` made 0o700, each directory made durable
   as io makes it. `A` is only read, and the two generations may share
   a file because neither writes one again once a later open has
   started: skein-kv replaces its `snapshot` by a rename and starts a
   new segment at every open, never appending to an older one, cutting
   it or writing it in place (skein's `data/kv.md`, sections 3 and 5), so a shared
   file stays as `A` left it, whatever `A + 1` commits, snapshots or
   removes. A task log is the one shared file `A + 1` writes again, and
   only past `A`'s extent of it, which `A` reads as garbage (skein's
   `data/kv.md`, 6.4; `logs.md`, 7).
   - **Linked:** `A`'s `snapshot` and each of its log segments but the
     newest (`log-<20 digits>`, the newest the one whose number is
     largest), each given a second name in `A + 1`'s `store/` by io's
     `HardLink` (skein's `runtime/io/files.md`; `runtime/kernel.md`,
     6.4). The newest is found over every page of `A`'s `store/`
     before the first link is made. So is each task log in `A`'s
     `store/logs/`: `logs/` and each directory in it made in `A + 1`'s
     `store/`, each log file linked there, and each such directory
     synced once its links are made (`logs.md`, 7).
   - **Copied,** file by file, each read and written in bounded chunks
     through the file driver (io has no copy), each file created
     exclusively 0o600 and synced: the newest segment, which the draft
     keeps unshared (its section 5) though skein-kv writes it no more
     than the others; any other file of `A`'s `store/`; and the whole
     of `secrets/`, small, whose files step 5 requires private with
     one link. The copy's `secrets/` is opened private (`OpenPrivate`:
     0o700) and its files made 0o600, as `secrets.md`'s are.
   - **A link the file system refuses,** answered `Permission` (a file
     system with no hard links, or Linux's `protected_hardlinks`),
     `TooManyLinks` or `Other` (`EXDEV` among them), is that file
     copied instead, and the start goes on: a deployment on a file
     system without hard links upgrades as before, its maps twice on
     disk. Any other failure is a file's fault, as a copy's is.
   - Then `A + 1`'s `store/` and `secrets/` are synced, which makes
     their links and new names durable (skein's `runtime/kernel.md`,
     6.4), then `generations/`. This document calls `A + 1` the copy,
     however its files were made.
3. **Open the copy:** both its stores recovered, and its format row,
   layouts and header read and checked at the copy's own levels, as
   1 (step 4) reads `A`'s; no transcript is cut.
4. **The registry and the jobs:** the registry's commit (`layouts.md`,
   5.5: the binary's new layouts, the retirements), from which the
   edges map heads (`layouts.md`, 4); then each pending job, in order,
   its pages committed through the component's own path, within
   skein-kv's bounds (`records.md`, 4; `jobs.md`, 3). Nothing else is
   written, and no transcript.
   The store's
   last commit, one of its own after the last page's and the last job's, writes the format
   row's levels and versions and the layouts, the binary's (`records.md`, 3), and
   the deployment's header as upgraded, naming that commit's number
   (`README.md`, section 5). The commits before it leave the header naming `A`'s
   last, which nothing reads: a cut removes `A + 1` (4), and the
   check runs after the last. The numbering carries on from `A`'s, and
   the journal's next commit follows the upgrade's last. Each job
   answers its counts as `jobs.md`, 3 says, for the shell's line.
5. **Open,** before anything switches (temper's `draft/upgrades.md`,
   6.4), over the copy as upgraded and still open (skein-kv has no
   read-only open), writing nothing; the first failure refuses, `Check`:
   every row of the core's live ranges and of the census's ended read
   positions (`loads.md`, 2), which the restart reads whole, decodes by the
   binary's codecs within the deployment's limits, its key its
   record's; and each file of the copy's secrets' store is a regular
   file, private to the service's user, with one link. Nothing is
   judged: each kept family's counts before and after go into the
   manifest (step 6), which the tests compare, and the records' links,
   an ended read position's task among them, are the domain's, made by
   the restart over the copy (step 7; `../../domain/engine/restart.md`).
6. **The manifest,** written into `A + 1` (`README.md`, section 2) by io's durable
   replace expecting no file there, once the root asks for the switch
   (step 7): each kept family's counts before and after the registry's
   commit and the jobs, taken as the copy opened and as step 5 read it,
   and the commit position then,
   which commits of the restore's may have moved past the upgrade's last.
7. **Switch, once the root has restored from the copy.** The open
   goes on over the copy, still open, as 1's step 7 goes on over an
   active generation (the run proofs' cuts, which cut the shared
   transcripts to the copy's committed lengths, `A`'s, since the upgrade
   writes no frame; the installation's identifier). The model catalogue
   overlay (`records.md`) is read by the binary's serving codec here,
   after the registry's commit and the jobs. The open answers
   `Opened` with `pending`: an upgraded generation awaits its switch. The
   root's restart restores from it as from any store, and asks for the
   switch (`Request::Switch`) once its restores are whole
   (`../../domain/engine/restart.md`). Until then what it commits lands
   in `A + 1`'s maps alone, and nothing leaves the process: no secret
   is served (`opening.md`, step 4) and no source swept, no run adopted
   or placed, no connector reads or writes its system, and no
   transcript is written or dropped. Once every commit before the ask
   is durable (`Committed`), the manifest (step 6), then
   `active` replaced by io's durable replace, expecting the digest of
   the `active` 1 read, so that only the start that planned from
   `A` switches from it; the component answers `Switched` and takes
   the generation it switched to as its active one. One rename, never
   two standing for one. A restart that refuses before it ends the
   start with `A` active and unchanged, and `A + 1` a stray; a switch
   that fails ends it as a failed commit does, and the next start opens
   whichever generation `active` names.
8. **The restart goes on** over `A + 1`, now active, as any restart:
   nothing reopens, since the copy's open made every check a later
   start makes. `A + 1`'s first commit of its own, which ends going
   back (temper's `draft/upgrades.md`, 3.2; 5), is the start's wires
   (`records.md`, 3), at once after the switch and before anything leaves the
   process.

```rust
Event::Opened { owner: Token, last: u64, installation: [u8; 16], pending: bool }
Request::Switch { owner: Token }   // once, and only when `pending`
Event::Switched { owner: Token }   // or `OpenRefused`, or `Failed` with the rename's fault
```

- **A refusal or a failure** at any step (a job's refusal, a check
  failed, a file's fault) stops the start, answering `OpenRefused`
  with why: the job and its key, or the check and what
  failed; so does a restore the root refuses before it asks for the
  switch, which the root names as any restart's refusal. Before the
  switch's rename, `A` stays active and unchanged,
  and the next start removes `A + 1` and begins again from a fresh
  copy. A switch answered `Failed` with its rename made (`committed`:
  skein's `runtime/io/files.md`, 2) may leave `A + 1` active, whole and checked,
  which the next start opens.
- **Room.** On disk, what the two generations do not share, never the
  transcripts (temper's `draft/store-shape.md`, section 5): at the
  switch, `A + 1`'s newest segment and secrets' store, copied, and the
  log of the registry's and the jobs' commits, each record a job puts
  once. As
  `A + 1` moves on, what it writes is its own: once its snapshot has
  replaced the linked one and dropped the linked segments, the maps
  are on disk twice, `A`'s as they were and `A + 1`'s, until the next
  upgrade drops `A` (temper's `draft/upgrades.md`, 6.1). A file system that refuses
  links copies, and needs the maps twice from the start. A disk that
  cannot hold them fails the copy with a file's fault, and every start
  fails again until there is room: checking the free space before the
  copy is open (`README.md`, section 11). In memory: one map at a time (`A`'s closed before the
  copy's opens) and a job's page and room (`jobs.md`, 3). `README.md`, section 8's worst case
  counts the larger of the upgrade's peak and serving's, never both. A
  cold restart's page workspace (`loads.md`, 1) is held the same way: only before
  the engine serves, so the restart's peak counts it, never a message's
  or a retained answer's bound, since no message carries it.
- **The domain sees** the copy as any restart's store, pending until it
  asks for the switch (step 7).

## 3. Owner sets and frames' sites

- **Owner sets are data,** handed to the component's open by its shell
  beside the limits and its build's name, a `&'static str` the floor row
  records (`records.md`, 3): for each owner whose records it keeps, its
  families, their value types, its jobs and its level, and for a
  frames' family its frames' texts and patches, the generated modules'
  constants (skein's `data/codec/command.md`, 3), no function. A set
  carries no map patch, no older text of a map family, no fresh mark
  and no effects in flight (temper's `draft/store-floor.md`, 4.1 and
  4.2): no map record is rewritten but by a job or the seal, and a
  format outside the registry is fixed (temper's `draft/store-floor.md`, section 5).

```rust
// jig_protocol_store
pub enum Owner { Skein, Smith, Jig, Application }   // upstream first
pub struct OwnerSet {
    pub owner: Owner,
    pub name: &'static str,               // the owner, as its frames' patches' headers spell it
    pub families: &'static [KeptFamily],
    pub jobs: &'static [Job],             // its jobs at open, in the order they run (`jobs.md`, 1)
    pub level: Option<u32>,               // its level (`records.md`, 3): none for an owner the row records no level of
}
pub enum KeptMap { Store, Secrets }       // the map its records live in (`secrets.md`)
pub struct KeptFamily {
    pub name: &'static str,               // the family, as its texts spell it
    pub map: KeptMap,
    pub keys: &'static [Keys],            // its records' keys, and no other's
    pub frames: Option<Frames>,           // the frames' family's (`transcripts.md`, 6); none for every other
    pub current: &'static str,            // the binary's text of the family (`layouts.md`, 2)
    pub types: &'static [ValueType],      // a registered family's value types; empty otherwise (`layouts.md`, 2)
    pub retired: &'static [Retired],      // its retired value types (`layouts.md`, 2)
}
pub struct Frames {
    pub root: &'static str,               // the declaration it is in transcripts' frames, where their machine starts
    pub texts: &'static [&'static str],   // its frames' kept texts, in version order, the last at `current`'s version
    pub patches: &'static [&'static str], // the patches the frames' machine plans, its owner's, in number order
}
pub struct Keys {
    pub from: &'static [u8],              // inclusive
    pub to: &'static [u8],                // exclusive
    pub root: &'static str,               // the declaration its records are
}
pub struct OwnerSets {
    pub sets: &'static [OwnerSet],        // at most one per owner
}
impl OwnerSets {
    pub const NONE: OwnerSets = OwnerSets { sets: &[] };
}
```

- **Handed in by the shell:** `Store::owner_sets` takes `OwnerSets`.
  This crate's `owners` module declares `SKEIN_SET` and `SMITH_SET`;
  `JIG_STORE_SET` is its store-owned part of jig's set.
  `jig-protocol-accounts` appends its family to make `JIG_SET`.
  temper's `temper_protocol_store::OWNER_SET` and the shell's
  `OWNER_SETS` (`temper-engine-service-vocabulary`) complete the
  hand-over. `jig-test-store-files` takes fixture sets through
  `Store::with_owner_sets`. The `upgrade` module runs the copy (2).
- **Keys.** A family's records are its `map`'s, committed there
  (`secrets.md`). A family's ranges hold records of different
  declarations (the core's tasks beside their turns), so each range
  names its records' own root. Keys' bytes: the application's are its
  own keys, relative to its tag, as its rows are (`README.md`, 4.2: the
  component adds the tag), and in the secrets' map relative to its
  secrets' prefix `[2, 0]` (`secrets.md`); every other owner's are the
  map's whole keys. A family's `keys` hold its own records and leave
  out what lies among them but is another's (the installation's
  identifier at `01 04 03` among the accounts', the settings' records
  among the core's): jig's from the component's own table (`README.md`,
  4.4), the application's from its keys' encoders (`README.md`,
  section 7).
- **The frames' families** (`transcripts.md`, 6) are the two whose
  `frames` is set: in jig's set, the core's, whose `TurnRecord` is a
  frame's body, its `texts` the core's `current` alone and its
  `patches` none; in smith's, smith-transcript, whose `Turn` the body's
  `transcript` holds, with no `keys`, since none of its records is a
  map's, its `texts` and `patches` smith-transcript's generated
  module's (smith's `protocol/transcript.md`, section 12). Every other
  family's is `None`. A set with a frames' family in another slot, or
  two in one, is refused, `Set`; so is a body whose held turn names a
  family other than smith's frames' one, judged from the frames'
  texts, at every open before its machines are made (`transcripts.md`,
  6). A frames' family's `texts` must keep one version after another,
  each spelling the family's name, its last at `current`'s version.
- **Levels.** A set's level is its `level`, held explicitly: jig's 6,
  temper's 14, skein's and smith's 0, which no binary moves
  (`records.md`, 3). Every writer and reader of a level (the first
  commit, the seal, the comparison at open, the manifest, the rehearsal
  and the shell's versions line) reads `level`. temper's store shell
  hands the four real sets, `SKEIN_SET` and `SMITH_SET` of this crate,
  `JIG_SET` of `jig-protocol-accounts` and temper's `OWNER_SET`
  (temper's `protocol/floor.md`, 3). This crate owns
  `STORE_FAMILIES` and `JIG_STORE_SET`: its families' metadata,
  registered value types and written jobs. Accounts builds `JIG_SET`
  from that set, appending its own `jig-accounts` family; the store
  names no accounts consumer. The store's real-family layout and job
  tests use its own set. The component's other tests hand it fixture
  owners' sets: `jig-test-layouts`' fixture families for the registry's
  tests, and `jig-test-patches-jig` in jig's slot and
  `jig-test-patches-smith` in smith's, fixture frames' families whose
  frames are patched as read. No test owner names a production family.
  A fixture body family's last text is the core's `TurnRecord` at the
  binary's version under its own name, its `transcript` holding the
  fixture turn, since a patched body is decoded by the binary's codec;
  the fixture turn's last is its own, since the component hands a
  turn's bytes on undecoded. The names, texts and keys are `&'static`,
  the binary's own constants, never state: the only references in
  fields the component holds (skein/docs/foundation/programming-model.md,
  10.3), beside optional immutable parsed binary texts (below), read by each
  open's check and by each serving open, which makes
  the frames' machines from them (`transcripts.md`, 6).
- **A set is checked where it is admitted,** at every open, before the
  levels are compared, so a start at equal levels proves its sets too:
  one set per owner; the manifest's bounds, at most 64 families in all
  and each name at most 64 bytes, since the manifest counts every kept
  family by its name (`README.md`, section 2); each family's key ranges
  ordered and non-empty, disjoint from every other family's in the same
  map (and from the installation's identifier and the other rows the
  component keeps), within its owner's tag (the application's relative
  to it), and each range's root a declaration of the family's
  `current`; its value types (`layouts.md`, 2) and its jobs (`jobs.md`,
  1); the frames' families (above); and the holds: the plan refuses
  sets whose texts' held families (each family's `current` and each
  frames' family's `texts`) close a cycle. A set that fails is refused,
  `Set` with its owner, before anything is written. That each record of
  the map lies in one family's range takes a walk, which a rehearsal
  makes (5), not every open.
- **One check per open.** An open checks its sets once. By default each
  text it reads is parsed for it alone. A shell may supply immutable parsed
  binary schemas through `Store::prepared_schemas` before open. Admission
  borrows a supplied schema only when its entire text equals the text being
  checked; a missing or unmatched text is parsed by that open. These schemas
  contain no stored data, admission verdict or layout verdict: every admission
  and layout check still runs. jig's test store keeps only schemas derived
  from its constant Skein, Smith and jig owner-set texts across tests, never a store or an
  open's state; first starts and cold restarts remain real opens. The
  component owns no cache that outlives an open. Whichever step needs the check first
  runs it: a first start before anything is written (1.2), the
  rehearsal (5), the registry's (`layouts.md`, 5) or readiness before
  the open serves. Its verdict, passed or the `Set` it refuses with its
  owner, is that open's: kept through the copy, its check and the
  reopen (2), and read by each later step in place of a fresh check, so
  each step refuses where it did, with the same `Set`. The verdict is a
  few bytes beside the open's state, not a parsed text: dropped when the
  open settles, served (`Opened`) or refused, and whenever the shell
  hands new sets (`Store::owner_sets`) or prepared schemas, so an open's start finds none
  and no verdict outlives the open it was made in. The jobs' check the
  open makes alone at its first step (`jobs.md`, 1), before reading the
  format row, is cheap and stays where it is.
- **A frame's written site goes up.** Step code holds no function
  (skein's `data/codec/patches.md`, 4; skein/docs/foundation/programming-model.md,
  10.3), so where a frame read while serving meets `Step::Written`
  (`transcripts.md`, 6), the component answers `Site` to the read's
  token with the owner, the frame's key, the site, the node and the
  frames' `Patcher` itself, moved, never shared. The shell sets the key
  in a `Default` `Context` of that owner's for each frame
  (skein's `data/codec/patches.md`, 5), runs the owner's generated
  `written` over `parts()`, and hands the `Patcher` back in a `Site`
  request with the result; the component then calls `next()`. One
  frame waits on at most one site at a time. No map record goes up: the
  open runs jobs and seals, with no eager map patching walk.

```rust
// up                                      // down
Event::Site {                              Request::Site {
    owner: Token,                              owner: Token,
    of: Owner,                                 patcher: Box<Patcher>,
    key: Box<[u8]>,                            result: Result<(), Halt>,
    site: Site,                            }
    at: Node,
    patcher: Box<Patcher>,
}
```

- **A job's other stored inputs:** while a job's page is outstanding
  (`jobs.md`, 3), `Store::patch_page` reads a bounded page of public
  application rows from the copy's map, with relative keys and an
  exclusive cursor, for the shell answering the page. No secrets are
  read, no serving codec is selected by the component, and no input is
  retained for a later page. Outside a job's page the method refuses. It
  reads at most the store's admitted operations per page and reports the
  same data failure as the job's page.
- **The shell that answers** is the store's shell, the one that names
  the owners' crates and drives the store's open: the application's
  (temper's `temper-service`, its `Store`); jig has no production shell
  of its own. In jig's tests the store world's (`tests/store`) answers
  the fixture owners' frames' sites and jobs. `jig-test-service`'s
  hands no sets. `jig-test-store-files`' hands none either, unless a
  core world gives it fixture sets (`Store::with_owner_sets`, and
  `owner_sets` for the next open): then it opens over a fake machine's
  directory, so that the copy, the jobs and the switch run. A site its
  shell has no function for is answered `Halt::NoSite` (skein's
  `data/codec/patches.md`, 4), as their routes already do for any token.
- **A shell with no sets.** The component holds `OwnerSets::NONE`
  until a shell hands it sets, by a setter beside the constructors
  (`Store::owner_sets`); with none it never answers `Site`, never wants
  `Request::Site`, and `Opened`'s `pending` is false. A root that sees
  `pending` true asks for `Request::Switch` before its runs are adopted
  (2, step 7).

## 4. Power loss

Every write, sync and rename above is a cut point. After each, the
next start opens one whole generation: the old, untouched, with the
partial one removed, or the new.

| Cut during | What the next start finds | What it does |
|---|---|---|
| a first start, before `active`'s rename | no `active`; at most `generations/1/` and a temporary | removes both; a first start |
| a first start, after `active`'s rename | `active` lost with its directory's sync, or `1` | as above, or opens 1 and makes `transcripts/` |
| removing strays | part of a stray | removes the rest |
| the drop of `A - 1`, before its rename | `active` is `A`; `A - 1` whole | upgrades again, its plan dropping `A - 1` |
| the drop, after its rename | `active` is `A`; part of `generations/dropped` | removes the rest as a stray; upgrades again |
| a link or a file copied, the registry's commit, a job's commit, the last commit, the check, the manifest, the restore before the switch | `active` is `A`; `A + 1` partial or whole | removes `A + 1`, whose names alone go, every file it shares staying `A`'s as `A` left it; upgrades again from a fresh copy |
| the switch, before its rename | `active` is `A` | as above |
| the switch, after its rename | `active` is `A`, or `A + 1` whole | as above, or opens `A + 1` |

- **A cut per write, sync, link and rename,** each run from its own start in
  the store's world (`README.md`, section 9), not their product. A link
  is kept past a cut once its directory is synced, as a new file's name
  is (skein's `runtime/kernel.md`, 6.4).
- **No `active` names a generation before it is whole:** its maps,
  their directories and its manifest are synced before the rename that
  names it, and nothing writes a generation once a later one is
  active.
- **A read writes nothing,** patched or not (`transcripts.md`, 6): a cut anywhere in a
  read leaves every file as it was, and the next read patches the same
  frame the same way.

## 5. Rehearsing and going back

Two operations the application's tools run with the engine stopped
(temper's `draft/upgrades.md`, 3.1 and 3.2), each taking the
deployment's lock as a start does (1, step 1).

- **A rehearsal** is a start that runs on to its opening with no
  network (*On to the opening*, below), never past it. It
  refuses a directory holding `restore`, `Generations::Unfinished`, before
  copying anything (`restoring.md`, 5). It never writes the directory it is given, a copy of a stopped
  deployment's such as a backup: it reads `active` and that
  generation's maps, copies them into a scratch deployment as that
  generation, file by file and never by links (the directory it is
  given may be a backup on another file system, and a link would
  change its files' links; `Openings` links what it can, below), its
  task logs among them (`logs.md`, 7),
  and opens the
  scratch as any start does (1 and 2), with its owner sets, the
  transcripts read where they are and never cut, and no generation
  dropped. The plan, the copy, the registry's commit and
  the jobs (`layouts.md`, 5.5; `jobs.md`, 2), and the restores (loaded and
  linked, `../../domain/engine/restart.md`)
  run (over `A + 1` where the open writes, over the scratch's `A`
  otherwise). It answers what it ran:
  the jobs with their counts (`jobs.md`, 3), the
  refusal a start would meet, or the restores whole; the wires `A`'s
  last start spoke (`records.md`, 3), and the workers' openings `A` keeps since that start
  (`records.md`, 3, *The workers' openings*), read before its own wires
  commit, from which the application
  says whether each of its workers may
  stay; how many files it linked into the scratch and how many it
  copied, a link the file system refused counted as a copy, with
  `Rehearsing`; and the opening (below). The scratch is
  emptied as it ends.
- **On to the opening.** After the restores the rehearsing root takes
  the switch step inside the scratch: the store marks the scratch's
  generation active there, the directory it was given untouched, and
  commits the start's wires into the copy (`records.md`, 3). The runs
  are adopted with no worker listening: claims in the engine and on
  in-process workers settle at once as at any start, and claims on
  other workers wait in the fleet, unawaited. Decisions open, and each
  connector begins catching up (`../../domain/engine/restart.md`, steps 4
  and 5) with every call to its system refused at once, so nothing
  leaves the process: no effect is made and no secret served beyond
  it. The rehearsal ends there, at the opening, and answers how long
  the start took to open and what each connector would still have to
  catch up, as the application reports it. A transcript is never
  written or dropped in a rehearsal, since its transcripts are the
  given directory's (above).
- **Its reach,** `Reach`, which the request names. `Whole` is all of the
  above. `Openings` reads only the rows from which the application
  judges its workers, and ends there, for a redeploy that judges them
  again at its stop (temper's `protocol/binaries.md`, section 2):
  - it locks the directory, refuses an outstanding intent and reads
    `active` as `Whole` does;
  - it makes that generation in the scratch as an upgrade makes `A + 1`
    (2, step 2), but for `store/logs/` and every task log in it, which
    no row it reads names, so that a redeploy's stop, where it runs
    over the stop's checkpoint, makes links and small copies alone
    (temper's `draft/checkpoints.md`, 6, step 3):
    - **linked:** `store/`'s `snapshot` and each of its segments but
      the newest, the newest found over every page first, by io's
      `HardLink`;
    - **copied,** file by file as `Whole` copies: the newest segment,
      any other file of `store/`, and the whole of `secrets/`;
    - **a link the file system refuses** (the scratch on another file
      system than the directory, as a backup's may be) is that file
      copied instead.

    Nothing writes a linked file again (2, step 2), so the directory's
    files keep their bytes; only their link counts move, while the
    scratch holds them;
  - it opens the scratch through the stores' recovery, and answers
    `Rehearsing` with the wires and the openings, as `Whole` does,
    before any check of the levels, the floor or the format;
  - then it ends as `Close` ends a rehearsal: the scratch emptied, and
    `Closed`. It answers no `Opened` or `OpenRefused`, decodes no record
    and runs no plan, job or restore, and the application asks no
    `Close` of it.

  A refusal before the rows are read (the lock `Busy`, `Unfinished`,
  `Unreadable`, a copy or a recovery failing) is answered as `Whole`'s
  is, `OpenRefused` after `Rehearsing`, and the application closes it
  as it closes any rehearsal.
- **Every record decoded** in a rehearsal, after the copy and before
  the plan, whether or not a job changes its family (the
  user's words of 2026-11-11): each family of each set is walked over its
  ranges of `A`'s maps (3, `Keys`), each record of a registered family
  decoded as a generic value of its layout's text from its value type's
  root (`layouts.md`, 4), and each other record as a generic value of
  its family's `current` from its range's root; a family of
  frames, each frame of each transcript from its frames' root. A start
  never does it: its codecs decode each record as read. The first
  record that does not decode ends the
  rehearsal there, with `Event::Undecodable` naming its family, version
  and place, and the plan does not run; all decoding, `Event::Decoded`
  with the count, and the plan runs. A store 1 refuses (ahead, behind,
  unleveled, below the floor, unregistered: `records.md`, 3; `layouts.md`, 5.1) is not
  walked: the refusal, the same as a start's, says why.
- **Going back** (`Store::back`, over the deployment's directory): once
  an upgraded generation is active, its previous one, `A`, stays on disk
  (`README.md`, section 2), and `active` may name it again until `A + 1`'s first
  commit of its own, the first past the position its manifest names
  (2, steps 6 and 8). It takes the lock, rolls an outstanding restore
  intent forward first (`restoring.md`, 5), then reads `active` and `A + 1`'s
  manifest, opens `A + 1`'s store as 1 opens one (skein-kv's recovery
  alone writes), and compares its last commit with the manifest's
  position. Equal, it replaces `active` by io's durable replace,
  expecting the digest it read, to name `A`, and answers `Back`; `A + 1`
  is then a stray the next start removes, and that start opens `A` as it
  was, cutting the transcripts to `A`'s lengths (`transcripts.md`, 4):
  `A`'s files are as `A` left them, those it shares with `A + 1`
  included, since nothing `A + 1` did (upgrading, snapshotting, this
  recovery, the stray's removal) writes a shared file (2, step 2). Past it, or with
  no previous generation or no manifest, it refuses, `Back { why }`,
  changing nothing: the upgraded generation has made commits of its own,
  which may have acted beyond the store, and going back is restoring a
  backup. A restored generation names no predecessor in its manifest, so
  going back refuses it `NoPrevious` (`restoring.md`, 4.4).

```rust
Request::Rehearse { owner: Token, directory: Root, scratch: Root, installation: [u8; 16], reach: Reach } // `directory` never written; `scratch` emptied, never removed
pub enum Reach { Whole, Openings }                     // `Openings`: `Rehearsing`, then `Closed` unasked
Adoption::Rehearse { scratch: Fd, reach: Reach }      // the host's adoption (`host.rs`), beside `Back` and `Checkpoint`
Event::Rehearsing { owner: Token, wires: Option<Box<[KeptWire]>>, openings: Box<[WorkerOpening]>, linked: u32, copied: u32 } // before `Opened` or `OpenRefused`; files linked into the scratch and copied (`Whole` links none)
pub struct KeptWire { pub name: Box<str>, pub version: u32 }                               // `None`: A's last start kept none
pub struct WorkerOpening { pub worker: Box<str>, pub wires: Box<[OpenedWire]>, pub output: u32, pub terms: Box<[OpenedTerm]>, pub at: Wall } // in the workers' names' order
pub struct OpenedWire { pub name: Box<str>, pub lowest: u16, pub layouts: Box<[[u8; 32]]> }
pub struct OpenedTerm { pub kind: u16, pub largest: u32 }
Write::Opening(WorkerOpening)                         // replaces that worker's row; `Write::Wires` clears them all
Event::Decoded { owner: Token, records: u64 }                                              // every record of A decoded; the plan runs
Event::Undecodable { owner: Token, family: Box<str>, version: u32, at: Undecoded }          // the rehearsal's last event but `Closed`
pub enum Undecoded { Record { map: KeptMap, key: Box<[u8]> }, Frame { transcript: Box<str>, offset: u64 } }
Request::Back { owner: Token, root: Root }
pub enum BackRefusal { Busy, NoPrevious, Committed { manifest: u64, last: u64 }, Unreadable }
Event::Back { owner: Token, to: u64 }                 // `active` names generation `to`
Event::BackRefused { owner: Token, why: BackRefusal }
pub fn levels(sets: &OwnerSets) -> Levels             // the binary's levels, for its versions' lines
Request::Close { owner: Token }                       // a rehearsal's end, in any state; refused for any other store
Event::Closed { owner: Token, removed: bool }         // `scratch` emptied; false on a file's fault: the next rehearsal removes the rest
```

The scratch is a `Root` like the directory: a directory the
application makes under its cache directory and adopts before it
asks, as it adopts the directory. The store never removes that directory, only what is under it, deepest
first as 1, step 5 removes a stray; making and removing the directory
itself are the application's. A rehearsal's store refuses
`Request::Switch`, though its root never asks, and its close
(`Request::Close`; an engine's store ends with its process and has
none) empties the scratch, `removed` saying it is empty; a rehearsal
that crashed leaves entries there, which the next rehearsal given that
scratch removes before it copies. An application that makes a new
scratch for each rehearsal removes those left behind itself, as
temper's does (temper's `protocol/binaries.md`, section 2). Its jobs
answer their counts as a start's do.
