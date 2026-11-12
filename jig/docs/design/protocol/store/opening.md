# Opening and restart

1. **The shell opens the deployment's directory,** and the component
   finds the active generation (1) and opens both its stores;
   skein-kv recovers each: the snapshot, then the log's longest valid
   prefix (skein's `data/kv.md`, section 3). A store behind the binary's
   patch levels is patched in a copy, and the open goes on over the
   copy, pending, which becomes the active generation once the root has
   restored from it (2).
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
(`README.md`, section 2). Nothing is removed or made until the active generation has
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
switch beside each other.

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

The stores open (step 1), and the format row, the layouts and the
deployment's header are read and checked (step 2, `records.md`, 3). Every refusal of
the levels, versions, layouts and deployment comes here, and nothing has
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
active generation or its previous.

### 1.6 `transcripts/`

`transcripts/` is made where it is missing.

### 1.7 At the binary's levels

At the binary's levels, the open goes on with the rest of step 2 (the
run proofs' cuts, the installation's identifier). Behind them, it
patches a copy (2).

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

- `OlderBuild { why: Unleveled }`: "`<deployment>` holds a store from before
  the baseline, the first build whose stores later builds upgrade, and this
  start has changed nothing in it. A store from before the baseline is not
  translated, so every start refuses this one until it is moved. To start
  afresh, stop the service, move `active`, `generations/` and `transcripts/`
  out of `<deployment>` together, and start again: nothing is deleted, and
  the next start makes a new, empty store. The new store has no tasks,
  sign-ins, accounts or keys, and every client signs in again."
- `Ahead`: "`<deployment>` holds a store from a later build, at patch levels
  past this build's, and this start has changed nothing in it. No patch runs
  backwards: start the later build again, or restore `<deployment>` from a
  backup this build made."
- `Patch`: "A patch refused a record while this start patched a copy of the
  store in `<deployment>`, and the generation it patched from stays active,
  unchanged. Every start of this build refuses the same record: start the
  build before it again, which opens that generation as it was."
- `Set`: "This build's patches do not plan, and this start has changed
  nothing in `<deployment>`. The build is wrong: start the build before it
  again."
- `InFlight`: "`<deployment>` holds effects its last run sent and did not
  settle, in records this build's patches change, and this start has
  changed nothing in it. Start the build before it again, let it settle
  them, and stop it: a stop settles what it sent first. Then start this
  build." The shell names each effect after it, from its key (3).
- `Check { why: Large }`: "The check before the switch found a patched record
  too large for a commit alone in the copy of the store this start patched in
  `<deployment>`, and the generation it patched from stays active, unchanged.
  Every start of this build checks a fresh copy the same way: start the build
  before it again, which opens that generation as it was."
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

The refusals at open, beside `README.md`, section 3's others:

```rust
pub enum OpenRefusal {
    Format { family: Family, found: u32, wanted: u32 },
    Deployment,
    Numbers { store: u64, header: u64 },
    Failed,
    OlderBuild { why: Older },                                 // nothing translated before the baseline
    Ahead { owner: Owner, found: Option<u32>, wanted: Option<u32> }, // no patch runs backwards
    Generations { why: Generations },
    Patch { owner: Owner, patch: u32, key: Box<[u8]>, halt: Halt }, // a record the patch refused (10.2)
    Set { owner: Owner, patch: u32 },                          // a patch set that does not plan (10.3)
    Check { why: Check },                                      // the check before the switch (10.2, step 5)
    InFlight { owner: Owner, family: &'static str, count: u32, keys: Box<[Box<[u8]>]> }, // effects a pending patch reshapes (10.3)
}
pub enum Older { Unrecorded, Layout(Family), Directory, Unleveled }
pub enum Generations { Busy, Unreadable, Unnamed, Missing { generation: u64 } }
pub enum Check { Large { key }, Load { key }, Private { name } }
```

`Older`'s first two compare the layouts' row (`records.md`, 3); `Check` names what
failed: a record too large for a commit alone, or a row that does not
load, by its key; a secrets' file
that is not private, by its name.

## 2. Patching a copy

The component's own phase, before the root starts, when the format row
names an owner's level behind the binary's (`records.md`, 3). From the active
generation `A` to `A + 1`:

1. **Plan,** over `A` as 1 opened it: the levels, the patches
   between them and their order across owners (3), a set that does
   not plan refused; the header decoded by the kept text of its own
   level and checked, as is the deployment; one machine per kept family
   made (3); whether a pending patch is marked fresh (3); the room
   (below) against the limits. Every refusal of
   `records.md`, 3, and an effect in flight that a pending patch changes (temper's
   `draft/upgrades.md`, section 8), comes here. Then `A`'s stores
   close: no transcript has been cut, which the open over the copy
   does (step 7). Then `A - 1`, where one stands, is dropped, since once `A + 1`
   exists going back is to `A`, which nothing here changes: renamed out
   of the numbers (`generations/dropped`), `generations/` synced, then
   removed as 1 removes a stray, so that a cut never leaves part of
   a generation where the previous stands.
2. **Copy** `A`'s `store/` and `secrets/` into `generations/A + 1/`,
   file by file, each read and written in bounded chunks through the
   file driver (io has no copy), each file synced and each directory
   made durable as io makes it, then `generations/` synced. The copy's
   `secrets/` is opened private (`OpenPrivate`: 0o700) and its files
   made 0o600, as `secrets.md`'s are. `A` is only read.
3. **Open the copy:** both its stores recovered, and its format row,
   layouts and header read and checked at the copy's own levels, as
   1 (step 4) reads `A`'s; no transcript is cut.
4. **Patch,** one walk per patch, in the plan's order (3): its
   families' keys (`KeptFamily::keys`) walked in key order by pages;
   each record, at its block's `from`, decoded by that version's kept
   text, run through the patch, encoded by its `to`'s and put back at
   its key; a record at another version refuses, `Patch` with its key.
   Each page is one commit through the component's own path, within
   skein-kv's bounds (`records.md`, 4); a record a page cannot hold alone refuses,
   `Check { why: Large }` with its key. The secrets' store is patched the
   same way, in its own commits, within its own limits (`secrets.md`).
   **Windows closed:** where a pending patch is marked fresh, a walk of
   its own after the plan's last patch, over the core's run proofs (`README.md`, 4.4),
   one for each live task ever claimed, by pages, each page a commit as
   a patch's are: each proof is decoded by the binary's codec, since
   every core record is at the binary's version by then, and each is
   put back at its key marked crossed (core-v6's `RunProof.crossed`),
   with its window closed where it has one not yet closed
   (`Window.closed`), every other field as it was, so that the restart
   settles its claim (`../../domain/engine/executors.md`, 2). A proof already
   crossed, with no window open, is not written, and a page with none to
   write makes no commit; a proof that does not decode, or
   decodes as another key's record, refuses `Check { why: Load }` with
   its key, as step 5 would. Nothing else is written, and no transcript:
   the frames are read patched (`transcripts.md`, 6), and the closed window is the
   core's to act on (`../../domain/engine/executors.md`, 2). The walk answers no
   `Patched`, since it is no patch, and adds or removes no record.
   The store's
   last commit, one of its own after the last page's, writes the format
   row's levels and versions and the layouts, the binary's (`records.md`, 3), and
   the deployment's header as patched, naming that commit's number
   (`README.md`, section 5). The commits before it leave the header naming `A`'s
   last, which nothing reads: a cut removes `A + 1` (4), and the
   check runs after the last. The numbering carries on from `A`'s, and
   the journal's next commit follows patching's last. As each patch
   ends, the component answers `Patched` with its owner, number and
   counts, for the shell's line (temper's `draft/upgrades.md`, 3.1).
5. **Open,** before anything switches (temper's `draft/upgrades.md`,
   6.4), over the copy as patched and still open (skein-kv has no
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
   (step 7): each kept family's counts before and after the patches,
   taken as the copy opened and as step 5 read it, and the commit
   position then,
   which commits of the restore's may have moved past patching's last.
7. **Switch, once the root has restored from the copy.** The open
   goes on over the copy, still open, as 1's step 7 goes on over an
   active generation (the run proofs' cuts, which cut the shared
   transcripts to the copy's committed lengths, `A`'s, since patching
   writes no frame; the installation's identifier), and answers
   `Opened` with `pending`: a patched generation awaits its switch. The
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

- **A refusal or a failure** at any step (a patch's `Halt`, a check
  failed, a file's fault) stops the start, answering `OpenRefused`
  with why: the record's key and the patch, or the check and what
  failed; so does a restore the root refuses before it asks for the
  switch, which the root names as any restart's refusal. Before the
  switch's rename, `A` stays active and unchanged,
  and the next start removes `A + 1` and begins again from a fresh
  copy. A switch answered `Failed` with its rename made (`committed`:
  skein's `runtime/io/files.md`, 2) may leave `A + 1` active, whole and checked,
  which the next start opens.
- **Room.** On disk, the maps twice (temper's `draft/upgrades.md`,
  6.1): `A` and `A + 1`, never the transcripts; `A + 1` is `A`'s maps and,
  beside them until skein-kv's next snapshot, the log of the patch
  commits, each record a walk rewrites once. A disk that cannot hold
  them fails the copy with a file's fault, and every start fails again
  until there is room: checking the free space before the copy is open
  (`README.md`, section 11). In memory: one map at a time (`A`'s closed before the
  copy's opens) and one machine per kept family (3), each
  `Patcher::workspace_bound` of its capacity. `README.md`, section 8's worst case
  counts the larger of patching's peak and serving's, never both. A
  cold restart's page workspace (`loads.md`, 1) is held the same way: only before
  the engine serves, so the restart's peak counts it, never a message's
  or a retained answer's bound, since no message carries it.
- **The domain sees** the copy as any restart's store, pending until it
  asks for the switch (step 7), with the windows a fresh mark closed and
  the claims it crossed (step 4).

## 3. Patch sets, the plan and written sites

- **Patch sets are data,** handed to the component's open by its shell
  beside the limits: for each owner whose records it keeps, the
  generated module's constants (skein's `data/codec/command.md`, 3), no function:

```rust
// jig_protocol_store
pub enum Owner { Skein, Smith, Jig, Application }   // upstream first
pub struct PatchSet {
    pub owner: Owner,
    pub name: &'static str,               // the owner as its patches' headers spell it
    pub families: &'static [KeptFamily],
    pub patches: &'static [&'static str], // `patches::PATCHES`, in number order
    pub fresh: Option<&'static str>,      // the mark across which windows start fresh, as its owner spells it
}
pub enum KeptMap { Store, Secrets }       // the map its records live in (section 9)
pub struct KeptFamily {
    pub name: &'static str,               // the family, as its texts spell it
    pub texts: &'static [&'static str],   // `patches::kept::<family>::TEXTS`, in version order
    pub map: KeptMap,
    pub keys: &'static [Keys],            // its records' keys, and no other's
    pub frames: Option<&'static str>,     // the declaration it is in transcripts' frames, where their machine starts
    pub flights: &'static [Keys],         // ranges whose every record is an effect in flight, its key naming it
}
pub struct Keys {
    pub from: &'static [u8],              // inclusive
    pub to: &'static [u8],                // exclusive
    pub root: &'static str,               // the declaration its records are, where its machine starts
}
pub struct PatchSets {
    pub sets: &'static [PatchSet],        // at most one per owner
}
impl PatchSets {
    pub const NONE: PatchSets = PatchSets { sets: &[] };
}
```

  A family's records are its `map`'s, walked and committed there
  (`secrets.md`). A family's ranges hold records of different declarations (the
  core's tasks beside their turns), and a machine starts each record at a
  declaration (skein's `data/codec/patches.md`, 4: `start(plan, root, record)`), so
  each range names its records' own. Keys' bytes: the application's are
  its own keys, relative to its tag, as its rows are (`README.md`, 4.2: the component
  adds the tag), and in the secrets' map relative to its secrets' prefix
  `[2, 0]` (`secrets.md`); every other owner's are the map's whole keys.

  **The frames' families** (`transcripts.md`, 6) are the two whose `frames` names a
  root: in jig's set, the core's, whose `TurnRecord` is a frame's body;
  in smith's, smith-transcript, whose `Turn` the body's `transcript`
  holds, with no `keys`, since none of its records is a map's. Every
  other family's is `None`. A set with a frames' family in another slot,
  or two in one, is refused, `Set` with patch 0; so is a body whose held
  turn names a family other than smith's frames' one, judged from the
  kept texts, at the plan and at every open that serves, before its
  machines are made (`transcripts.md`, 6). Smith's set is smith-transcript's constants
  in the `Smith` slot (smith's `protocol/transcript.md`, section 12),
  its `map` `Store` and unused, its `keys` empty.

  A family's `keys` hold its own records and leave out what lies among
  them but is another's (the installation's identifier at `01 04 03`
  among the accounts', the settings' records among the core's): jig's
  from the component's own table (`README.md`, 4.4), the application's from its
  keys' encoders (`README.md`, section 7). A set with no kept text is an owner
  before its baseline, level none; otherwise its level is its patches'
  count. From the baseline, temper's store shell hands the four real
  sets, `SKEIN_PATCH_SET` and `SMITH_PATCH_SET` of this crate,
  `JIG_PATCH_SET` of `jig-protocol-accounts` (the lowest jig crate that
  sees every jig family, since it depends on this one) and temper's own
  (temper's `protocol/baseline.md`, 4), each kept
  at level 0 with no patch until a landing brings one; the component's tests hand it fixture owners' sets:
  `jig-test-patches` in the application's slot and `jig-test-patches-jig`
  in jig's, so that the order across owners is shown, and
  `jig-test-patches-smith` in smith's, a fixture turn family whose frames
  are patched as read, with a patch marked fresh. No test owner names a
  production family. A fixture body family's last text is the core's
  `TurnRecord` at the binary's version under its own name, its
  `transcript` holding the fixture turn, since a patched body is decoded
  by the binary's codec; the fixture turn's last is its own, since the
  component hands a turn's bytes on undecoded. The names, texts and keys are `&'static`, the binary's own constants,
  never state: the only references in fields the component holds
  (skein/docs/foundation/programming-model.md, 10.3), read by its plan and by each serving
  open, which makes the frames' machines from them (`transcripts.md`, 6), since `keep`
  and `plan` copy what they need into each machine's tables.
- **A set is checked where it is admitted,** at every open, before the
  levels are compared, so a start at equal levels proves its sets too:
  each family's key ranges ordered and non-empty, disjoint from every
  other family's in the same map (and from the installation's identifier
  and the other rows the component keeps), within its owner's tag (the
  application's relative to it), and each range's root a declaration of
  the family's last kept text; a set that fails is refused, `Set` with
  patch 0. That each record of the map lies in one family's range takes
  a walk, which the baseline's rehearsal makes (temper's
  `protocol/README.md`, section 7), not every open.
- **One sequence per owner,** numbered in one crate, its store crate
  (jig's `jig-protocol-store`, temper's `temper-protocol-store`): an
  owner whose families another crate of its keeps (jig's `jig-accounts`
  in `jig-protocol-accounts`, temper's `temper-secrets` in
  `temper-protocol-records`) names them in the same set, their kept
  texts in the crate that keeps their current schema, and every patch
  of any of them in the store crate's sequence, so the owner has one
  level. The other crates' `patches/` stay empty.
- **The plan** reads each patch's header (skein's `data/codec/patches.md`, 6:
  `header`, `level`, `after`): a patch's owner must be its set's `name`
  and its number its place there, or the set is refused, naming it.
  The manifest counts every kept family by its name (`README.md`, section 2), so the
  sets fit its bounds, at most 64 families in all and each name at most
  64 bytes, or the set where they pass them is refused, `Set` with
  patch 0, as two sets for one owner are: judged before the plan, and
  at a first start before anything is written.
  The order across owners follows the pins (temper's
  `draft/upgrades.md`, 4.4), downstream first. From the most
  downstream owner with patches pending, its patches run in number
  order; before a patch that names `after U L`, `U`'s pending patches
  up to level `L` run, each planned the same way, and none past it;
  after an owner's last, its nearest upstream's remaining ones, then
  that one's upstream's. A patch whose `after` names a level its
  upstream has already passed in the store, or one past the binary's,
  refuses the set: under the pins that is a binary built wrongly
  (`README.md`, section 11). The plan is a list of (owner, patch), its length the
  patches' count, made by a loop bounded by it.
- **The fresh mark** is a set's word (`PatchSet::fresh`), its owner's to
  spell and to mean: smith's `fresh-windows` (skein's `data/codec/patches.md`, 1;
  smith's `protocol/transcript.md`, section 12), handed in by the shell
  as data, so the component names no owner's constant. The plan reads
  it with each pending patch's header (`Header::marked`): a set's
  patches past its level in the store and up to the binary's, one marked
  by its set's word enough. Then the copy's windows are closed (2,
  step 4); a set whose `fresh` is `None` marks nothing, and a patch's
  other marks are its owner's alone. A patch may carry a mark and change
  no family.
- **Effects in flight** (temper's `draft/upgrades.md`, section 8) are
  the application's to show and the plan's to refuse. A connector that
  keeps an effect's sent attempt keeps, beside its outbox entry, a
  record of its own in a range that holds nothing else, its key naming
  the effect and its task: written in the commit that records the
  attempt, kept while the effect is unresolved (uncertain, retried,
  settled at a restart), and erased in the one that settles it for good
  or holds it for a person (`../../domain/connectors.md`, 4.2 and 4.3);
  its set names those ranges (`KeptFamily::flights`). The plan reads
  them in `A`, for each family a pending patch changes (its blocks name
  it), in the plan's order, before anything is written: the first
  family with a record there refuses the start, `InFlight` with the
  family, how many it holds and their first keys, at most 64, since the
  patch would reshape what the next start must recover in its old
  shape. The shell names each key it was given (temper's: the entry and
  its task) and how many more there are. A family is the patch's unit,
  so an application whose records are one family refuses any patch of
  its own while one effect is in flight.
  A family no pending patch changes keeps its effects, which the new
  version recovers as after any stop (`../../domain/engine/restart.md`,
  step 5). Every stop settles first, so the ranges are empty after one
  that settled, and the refusal meets a crash or a stop past its bound.
  A set with no such range never refuses.
- **One machine per kept family,** a `Patcher` (skein's `data/codec/patches.md`,
  4 and 6), made at the plan, on the patch path alone, with its
  capacity from the family's limits and texts, each kept text kept and
  each of its owner's patches planned. In a patch's walk a record is
  one block behind; a held value `k` versions behind runs `k` plans
  to the version its family has after the patch, `next_pass` after the
  first. In a patch's walk, its families' records run its plan,
  and a family the patch leaves alone whose records hold one it moves runs
  its identity plan (skein/docs/design/data/codec/patches.md 3), which reaches the held fields alone.
  The frames' machines are others, made at each open that serves
  (`transcripts.md`, 6).
- **A written site goes up.** Step code holds no function (skein/docs/design/data/codec/patches.md,
  4; skein/docs/foundation/programming-model.md, 10.3), so at `Step::Written` the
  component answers `Site` with the owner, the record's key, the site,
  the node and the family's `Patcher` itself, moved, never shared. The
  shell sets the key in that owner's `Context` (skein/docs/design/data/codec/patches.md, 5), which
  it keeps for the whole patching (`Default` at its start), runs the
  owner's generated `written` over `parts()`, and hands the `Patcher`
  back in a `Site` request with the result; the component then calls
  `next()`. One record's walk waits on at most one site at a time. A
  frame read while serving sends its sites up the same way, to the
  read's token, with a `Default` `Context` for each frame (`transcripts.md`, 6).

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
Event::Patched { owner: Token, of: Owner, patch: u32, counts: Counts }
pub struct Counts { pub records: u64, pub commits: u64 }
```

- **The shell that answers** is the store's shell, the one that names
  the owners' crates and drives the store's open: the application's
  (temper's `temper-service`, its `Store`); jig has no production
  shell of its own. In jig's tests the store world's (`tests/store`) answers
  the fixture owners', while patching and while serving. `jig-test-service`'s
  hands no sets. `jig-test-store-files`' hands none either, unless a core
  world gives it fixture sets (`Store::with_patch_sets`, and `patch_sets`
  for the next open): then it opens over a fake machine's directory,
  so that the copy, the walk and the switch run, and it reads the
  patched proofs back through the component. Its fixture sets write no
  site, and both shells would answer one `Halt::NoSite` (skein's
  `data/codec/patches.md`, 4: a site its caller has no function for), as their
  routes already do for any token.
- **A shell with no sets.** The component holds `PatchSets::NONE`
  until a shell hands it sets, by a setter beside the constructors
  (`Store::patch_sets`); with none it never answers `Site` or
  `Patched`, never wants `Request::Site`, and `Opened`'s `pending` is
  false. A root that sees `pending` true asks for `Request::Switch`
  before its runs are adopted (2, step 7).
- **Held fields** are the component's: at `Step::Held`, the held
  family's own machine patches the held bytes, its own sites going up
  as above, and its bytes are put into the holder's (`put_content`),
  or carried where that family does not move; the holder's machine
  waits meanwhile. No family holds itself through another, so one machine
  per family suffices: the plan refuses sets whose kept texts' holds
  close a cycle, `Set` with patch 0, before anything is written, and a
  walk that meets one still halts `Order`, refused as a patch's halt
  (2).
- **Keys do not move** by patch: a key is its owner's code (`README.md`, 4.1), and a
  patch's text has no line that moves one (skein's `data/codec/patches.md`, 1). A
  moved key would need its function's answer up and back, and a check
  that it lands on no record its walk has yet to meet. A family whose
  keys change waits for that (`README.md`, section 11); before the baseline none
  does.

## 4. Power loss

Every write, sync and rename above is a cut point. After each, the
next start opens one whole generation: the old, untouched, with the
partial one removed, or the new.

| Cut during | What the next start finds | What it does |
|---|---|---|
| a first start, before `active`'s rename | no `active`; at most `generations/1/` and a temporary | removes both; a first start |
| a first start, after `active`'s rename | `active` lost with its directory's sync, or `1` | as above, or opens 1 and makes `transcripts/` |
| removing strays | part of a stray | removes the rest |
| the drop of `A - 1`, before its rename | `active` is `A`; `A - 1` whole | patches again, its plan dropping `A - 1` |
| the drop, after its rename | `active` is `A`; part of `generations/dropped` | removes the rest as a stray; patches again |
| the copy, a patch commit, a closing commit, the check, the manifest, the restore before the switch | `active` is `A`; `A + 1` partial or whole | removes `A + 1`; patches again from a fresh copy |
| the switch, before its rename | `active` is `A` | as above |
| the switch, after its rename | `active` is `A`, or `A + 1` whole | as above, or opens `A + 1` |

- **A cut per write, sync and rename,** each run from its own start in
  the store's world (`README.md`, section 9), not their product.
- **No `active` names a generation before it is whole:** its maps,
  their directories and its manifest are synced before the rename that
  names it, and nothing writes a generation once a later one is
  active.
- **Windows closed with the copy:** the closing commits are `A + 1`'s,
  cut as a patch commit is, so the generation that the switch makes
  active holds every live window closed, and a crash after it keeps
  them so; `A`, which going back opens (temper's `draft/upgrades.md`,
  3.2), holds them as they were.
- **A read writes nothing,** patched or not (`transcripts.md`, 6): a cut anywhere in a
  read leaves every file as it was, and the next read patches the same
  frame the same way.

## 5. Rehearsing and going back

Two operations the application's tools run with the engine stopped
(temper's `draft/upgrades.md`, 3.1 and 3.2), each taking the
deployment's lock as a start does (1, step 1).

- **A rehearsal** is a start that stops before its switch step. It
  never writes the directory it is given, a copy of a stopped
  deployment's such as a backup: it reads `active` and that
  generation's maps, copies them into a scratch deployment as that
  generation, and opens the
  scratch as any start does (1 and 2), with its patch sets, the
  transcripts read where they are and never cut, and no generation
  dropped. The plan, the copy, the patches and the restores (loaded and
  linked, `../../domain/engine/restart.md`)
  run (over `A + 1` where patches are pending, over the scratch's `A`
  otherwise); then the rehearsing root ends where it would take the
  switch step, at which every start's first act beyond the process
  waits (`../../domain/engine/restart.md`), so a rehearsal makes no
  effect, serves no secret and adopts no run. It answers what it ran:
  the patches with their counts and times (`Event::Patched`), the
  refusal a start would meet, or the restores whole; the wires `A`'s
  last start spoke (`records.md`, 3), with whether a pending patch is marked fresh
  (3), from which the application says whether its workers may
  stay; and that what follows the switch (adopting runs, reading the
  systems, settling the outboxes) is not rehearsed. The scratch is
  emptied as it ends.
- **Every record decoded** in a rehearsal, after the copy and before
  the plan, whether or not a pending patch changes its family (the
  user's words of 2026-11-11): each family of each set, at the version `A`
  holds it (its owner's level in `A`'s row, 0 by `BASELINE` (`records.md`,
  3), and the family's kept version at that level, from its first kept
  version through the set's patches' blocks), is walked over its
  ranges of `A`'s maps (3, `Keys`), each record decoded as a generic
  value of that version's kept text from its range's root; a family of
  frames, each frame of each transcript from its frames' root. A start
  never does it: its patches decode what they change, and its codecs
  the rest as read. The first record that does not decode ends the
  rehearsal there, with `Event::Undecodable` naming its family, version
  and place, and the plan does not run; all decoding, `Event::Decoded`
  with the count, and the plan runs. An owner whose level the binary
  does not know (ahead, unleveled) is not walked: the plan's refusal
  says why.
- **Going back** (`Store::back`, over the deployment's directory): once
  a patched generation is active, its previous one, `A`, stays on disk
  (`README.md`, section 2), and `active` may name it again until `A + 1`'s first
  commit of its own, the first past the position its manifest names
  (2, steps 6 and 8). It takes the lock, reads `active` and `A + 1`'s
  manifest, opens `A + 1`'s store as 1 opens one (skein-kv's recovery
  alone writes), and compares its last commit with the manifest's
  position. Equal, it replaces `active` by io's durable replace,
  expecting the digest it read, to name `A`, and answers `Back`; `A + 1`
  is then a stray the next start removes, and that start opens `A` as it
  was, cutting the transcripts to `A`'s lengths (`transcripts.md`, 4). Past it, or with
  no previous generation or no manifest, it refuses, `Back { why }`,
  changing nothing: the patched generation has made commits of its own,
  which may have acted beyond the store, and going back is restoring a
  backup.

```rust
Request::Rehearse { owner: Token, directory: Root, scratch: Root, installation: [u8; 16] } // `directory` never written; `scratch` emptied, never removed
Event::Rehearsing { owner: Token, wires: Option<Box<[KeptWire]>>, fresh: bool }           // before `Opened` or `OpenRefused`
pub struct KeptWire { pub name: Box<str>, pub version: u32 }                               // `None`: A's last start kept none
Event::Decoded { owner: Token, records: u64 }                                              // every record of A decoded; the plan runs
Event::Undecodable { owner: Token, family: Box<str>, version: u32, at: Undecoded }          // the rehearsal's last event but `Closed`
pub enum Undecoded { Record { map: KeptMap, key: Box<[u8]> }, Frame { transcript: Box<str>, offset: u64 } }
Request::Back { owner: Token, root: Root }
pub enum BackRefusal { Busy, NoPrevious, Committed { manifest: u64, last: u64 }, Unreadable }
Event::Back { owner: Token, to: u64 }                 // `active` names generation `to`
Event::BackRefused { owner: Token, why: BackRefusal }
pub fn levels(sets: &PatchSets) -> Levels             // the binary's levels, for its versions' lines
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
that crashed leaves entries there, which the next rehearsal removes
before it copies. Its patches answer `Patched` as a start's do.
