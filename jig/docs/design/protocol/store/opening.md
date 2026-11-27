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
   recovered one (`commits.md`, section 1); the run proofs' tasks' committed lengths,
   cutting each file to its length (`transcripts.md`, 4); and the installation's
   identifier, which replaces the one the shell drew (`keys.md`, 1.4). An empty
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
- `Layout { why: Unkept }` (`layouts.md`, 5.6): "`<deployment>` holds
  values a later build wrote that this build cannot keep, and this
  start has changed nothing in it. Start the later build again, or,
  with it, restore a checkpoint of `<deployment>` taken before it
  started, then start this build." The shell names the family, the
  value type, the layout's id, how many of its values this build
  cannot keep, and the first one's declaration, part and why, after it.
- `Layout { why: Unread }` (`layouts.md`, 5.6): "`<deployment>` holds
  values at an older layout that this build cannot read, and this
  start has changed nothing in it. Start the build before it again, or
  a later build whose written job converts that layout." The shell
  names the family, the value type, the layout's id, how many of its
  values this build cannot read, and the first one's declaration, part
  and why, after it.
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
pub enum Unplanned { Underived(skein_codec::layout::Underived), Text, Undeclared, Retired, Unkept(Refusals), Unread(Refusals) } // no plan; a text missing or unparsed; a value type the binary lacks, in a family it declares or one it does not; a retired value type's keys holding a value (layouts.md, 2); a later layout's values its Keep plan refuses, an older layout's values its Read plan refuses (layouts.md, 5.6)
pub struct Refusals { pub refused: u64, pub why: skein_codec::layout::Refused, pub declaration: Box<str>, pub part: Box<str> } // how many it refused; the first refusal, by the stored text's names
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
(temper's `draft/archive/store-floor.md`, section 3); `Check` names what
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
`draft/archive/store-floor.md`, 4.2). From the active generation `A` to
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
     `HardLink` (skein's `runtime/io/files.md`; `skein/docs/design/runtime/kernel-files.md`,
     2). The newest is found over every page of `A`'s `store/`
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
     their links and new names durable (skein's `runtime/kernel-files.md`,
     2), then `generations/`. This document calls `A + 1` the copy,
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
   (`commits.md`, section 1). The commits before it leave the header naming `A`'s
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
  skein's `runtime/io/files.md`, 10) may leave `A + 1` active, whole and checked,
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

Moved to [`opening-owners.md`](opening-owners.md), section 1.

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
  the store's world (`testing.md`, section 1), not their product. A link
  is kept past a cut once its directory is synced, as a new file's name
  is (skein's `runtime/kernel-files.md`, 2).
- **No `active` names a generation before it is whole:** its maps,
  their directories and its manifest are synced before the rename that
  names it, and nothing writes a generation once a later one is
  active.
- **A read writes nothing,** converted or not (`transcripts.md`, 6): a cut anywhere in a
  read leaves every file as it was, and the next read converts the same
  frame the same way.
## 5. Rehearsing and going back

Moved to [`opening-rehearsal.md`](opening-rehearsal.md), section 1.
