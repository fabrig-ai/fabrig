# checkpoints: a serving deployment's state, whole and consistent, without stopping it

Draft, 2026-11-15. The user asked for it after dev.fabrig.ai's upgrade of 11-15, naming a native checkpoint: a consistent backup taken when nothing writes. A checkpoint is the deployment's state at one commit, taken by the serving engine at a short barrier in its ordered work, mostly by hard links. It never changes once made, and it opens as a stopped deployment's directory does. It makes three things possible: an upgrade's full check while the old build still serves, a backup at every stop without copying the deployment whole, and a copy off the machine at any pace.

## 1. In one page

**Today**

- **A backup is a copy taken with the engine stopped** (`upgrades.md`, 3.3; jig's `protocol/store/README.md`, section 2). On dev.fabrig.ai it is 1.7 GB, made in about 5 s at every redeploy, and seven of them hold 7.5 GB beside 1.4 GB of live state.
- **The upgrade's check runs over that backup,** so it runs while dev.fabrig.ai is down (`../redeploy.md`, 4.2). On 11-15 the check took 6 min 41 s and the start 6 min 51 s, each doing the same patches and jobs, out of 15 minutes down.
- **A copy of a serving deployment is consistent only by luck.** `cp -a` of the state directory while the engine served passed the check once on 11-15 and failed on 11-14. Between two files a copy can:
  - miss a snapshot's rename and the segments it deletes;
  - catch a frame half written;
  - take transcripts ahead of the map.

  The online backup that jig's store README sketches (section 2) relies on skein-kv keeping each segment until the copy has read it, which nothing provides. `upgrades.md` leaves online backups open (section 15).

**The ideal**

- **A checkpoint is the state at a cut** of the component's ordered work, at C, the last main commit before the cut (2, 3.1).
- **What it costs:** a short pause in admitting commits, links for files that nothing writes again, and bounded copies of the prefixes of the few files that are appended in place.
- **Holds keep its sources:** from the cut until it is complete, nothing it selected is renamed over or removed (3.2).
- **Asking for one** (4): `temper ctl DIR deployment checkpoint` while the engine serves, or `temper-engine checkpoint` over a stopped deployment.
- **Using one** (5): the upgrade's full check runs over it while the old build serves; it can be copied off the machine at any pace; `temper-engine restore` brings it back, crash-safe (5.3). A reader holds a checkpoint's lock, so it isn't removed under it.
- **dev.fabrig.ai's redeploy** (6) becomes:
  1. a checkpoint and the full check, with its worker verdict, while the old build serves;
  2. the settled stop, a checkpoint as the backup, and a check of the workers' facts the verdict used;
  3. the swap and the start.

  On 11-15's figures the check's 6:41 leaves the outage and the backup copies no deployment whole. The outage is about half of today's: the start's 6:51 stays, plus the stop's checkpoint (6).

## 2. What a checkpoint is

- **Its name is unique:** `<deployment>/checkpoints/<C>-<16 hex>/`, C as 20 decimal digits and the suffix drawn at its making. Two checkpoints may record the same C: one taken online and another at a stop with no commit between, or later ones after a restore has rewound the map. No publication overwrites another, and retention orders by when each was made, not by C (3.4).
- **Laid out as a stopped deployment's state directory** (jig's `protocol/store/README.md`, section 2):

  | Path | Holds |
  |---|---|
  | `active` | `1` |
  | `generations/1/store/` | the snapshot and the segments recovery reads through C; from S7, `logs/` with the logs the map at C names |
  | `generations/1/secrets/` | the secrets' store at its own position at the cut |
  | `generations/1/manifest` | a generation manifest encoded afresh: no predecessor, the positions of both stores at the cut, the store's levels (jig's `protocol/store/README.md`, section 2) |
  | `transcripts/` | the files the map at C names |
  | `checkpoint` | the checkpoint's own record (below) |

- **Its record:** C and the secrets' position, when it was made and by which build, the deployment's identity, and every file with its length. Payload logs are listed with the extent the map at C names.
- **Left out on purpose:**
  - a snapshot still being written (`snapshot.tmp`, which recovery removes);
  - durable-replace temporaries;
  - transcript or log bytes no row names;
  - previous generations and other checkpoints.

  A file under `store/` that the checkpoint doesn't recognize fails it, naming the file, rather than being skipped.
- **Whole and fixed:** a checkpoint is in place only once complete (3.3). Its files never change: a file that is still appended to is copied up to its length at C, never linked (3.1).
- **On the deployment's file system,** beside `generations/`, and outside it, so that no start removes it as a stray (jig's `protocol/store/opening.md`, 1.5). A first start's checks read `checkpoints/` as part of the layout (jig's `protocol/store/README.md`, section 2).
- **Not in it:** the configuration, the cache, the forge, the programs' logins, and the workers' own state, as a backup today holds none of them.

## 3. Taking one

The engine's store component takes the checkpoint (jig's `protocol/store/`), through skein-kv for both of its stores, and takes the transcripts and logs itself.

### 3.1 The cut

- **A barrier in the component's ordered work:**
  1. Later main commits and secret mutations are held in order.
  2. Earlier accepted work is finished: transcript and log preparations, and both stores' commits up to the barrier, each durable.
  3. No destructive request against a file the checkpoint will select is still in flight.
- **At the barrier,** C is the last main commit. The component records:
  - the secrets' store's position;
  - each transcript's committed place at C, never a reservation made for a later commit (jig's `protocol/store/transcripts.md`, 2);
  - the logs' extents at C.
- **Before releasing the held work,** it selects the sources below and places the holds of 3.2. Admission pauses for this, and the loop keeps processing io. Each step is bounded: sources are enumerated a page at a time, under the holds.
- **The sources:**

  | The state's part | Taken how |
  |---|---|
  | each store's completed snapshot and the segments recovery needs through C | linked: skein-kv never writes a sealed segment or a snapshot once it is named, and replaces a snapshot by a rename (skein's `data/kv.md`, sections 3 and 5) |
  | each store's newest segment | copied up to the end of the frame of its position at the cut. Frames are only appended, so those bytes never change |
  | the secrets' store | copied whole, private, with a single link each (jig's `protocol/store/secrets.md`), as an upgrade copies it (jig's `protocol/store/opening.md`, 2, step 2) |
  | the transcripts of tasks ended by C | linked: an ended task's file is not touched again (jig's `protocol/store/transcripts.md`, 4) |
  | the transcripts of tasks live at C | copied up to their committed lengths: their committed places hold older turns too, so the whole prefix is copied, never only the current window |
  | from S7, the logs of tasks ended by C | linked, if nothing appends to a log once its task has ended (section 9) |
  | from S7, the logs of tasks live at C | copied up to their extents at C. Appends go on past the extent in the same file, so a link would not stay fixed (skein's `data/kv.md`, section 6) |

- **After the barrier** the held work resumes in order. The copies then run as bounded chunks that yield between them, with serving io first and explicit bounds on files, queued bytes and time. No commit waits on a copy, but the cut itself pauses admission briefly.
- **A link the file system refuses** is that file copied instead, as an upgrade copies one (jig's `protocol/store/opening.md`, 2, step 2).

### 3.2 Holds on its sources

From the barrier until the checkpoint has its last link durable and its last copy synced, nothing the checkpoint selected is renamed over or removed:

- **skein-kv, for both stores:**
  - doesn't rename a finished snapshot over the selected one until the link to it is made, since the replacement is fuzzy and may hold values past C (skein's `data/kv.md`, section 5);
  - doesn't delete a segment the selected snapshot needs, or any selected segment.
- **The component:**
  - doesn't remove a selected transcript at its drop's removal point, which comes before the drop's commit (jig's `protocol/store/transcripts.md`, 5);
  - doesn't remove a selected log at its drop.

Each held removal goes ahead in order once the holds are released, on success, refusal or failure alike. A selected file that has vanished anyway, removed from outside the engine, fails the checkpoint, and no part of one is ever published.

### 3.3 In place

1. **Made under a temporary name:** `checkpoints/<name>.skein-<16 hex>`.
2. **Synced from the bottom up:** each copy and linked directory, then the manifest and the record and their directories.
3. **Renamed** to `checkpoints/<name>`, then `checkpoints/` synced.
4. **Acknowledged** only after that last sync.

- **Power loss before the acknowledgement** leaves either the temporary or a complete checkpoint, never a partial one under the final name. The next checkpoint, or a start, removes a temporary of this pattern alone. This sweep is new; the open's stray removal doesn't do it today (jig's `protocol/store/opening.md`, 1.5).
- **One at a time:** a request for another while one is being taken is refused, *checkpoint in progress*.
- **Against the open:** a stopped deployment's checkpoint takes the deployment's lock, so it never runs beside an open's generation clean-up.

### 3.4 Kept

- **The newest `store.checkpoints.keep`,** a configuration setting, default 2, ordered by when each was made.
- **When a new one is in place,** the oldest past that number is removed, as a dropped generation is (jig's `protocol/store/opening.md`, 2, step 1). One whose lock is held (5) is skipped until it's free.
- **Removing a name never removes a file** that the live store or another checkpoint still names.
- **The operator may also remove one by name,** which is refused while it is in use.

## 4. Asking for one

- **While serving:** `temper ctl DIR deployment checkpoint` (`../client/ctl/tools.md`, 3). It is an act of a person holding an owner's role over the deployment, the people the deployment's settings are proposed to.
- **Over a stopped deployment:** `temper-engine checkpoint --data DIR`. It takes the deployment's lock, as a rehearsal does (jig's `protocol/store/opening.md`, 5). Every segment can then be linked, since the next recovery starts a new one; the secrets and the live tasks' transcripts and logs are still copied.
- **The answer,** once the checkpoint is in place, gives:
  - its name, C and the secrets' position;
  - the files linked, the bytes copied and the time taken, as measured;
  - the bytes it holds that the live store doesn't name, as of then. That figure changes as the live store replaces its files.

## 5. Using one

### 5.1 Readers hold its lock

Whatever reads a checkpoint holds its lock (io's `Lock`, as a deployment's is held) while it reads: a rehearsal, a restore and its source, or an operator's copy through `temper-engine checkpoint --hold NAME -- COMMAND`. Retention skips a checkpoint whose lock is held, and an explicit removal refuses it.

### 5.2 The upgrade's check, while the old build serves

`temper-engine upgrade --check` runs over a checkpoint, holding its lock. The rehearsal copies its input into its scratch file by file and never writes it (jig's `protocol/store/opening.md`, 5), so neither the checkpoint nor the live store changes.

- **It proves C:** every record decoded, the patches, the jobs and the restores, and the worker verdict (`../redeploy.md`, 4.2).
- **What the start then does:** it repeats the real path over the stopped state: patches, jobs, the open and the restore. It refuses before its switch for any failure that path detects, and leaves the active generation's state as it was (jig's `protocol/store/opening.md`, 2).
- **What nothing repeats:** the rehearsal's walk of every cold record, for what was committed after C. Those records come from the same build, in the same types and layouts, so a bad one is unlikely, not impossible. Such a record fails later, when read. The workers' facts the verdict used are checked again at the stop (6).

### 5.3 Restore

`temper-engine restore NAME`, with the engine stopped. It holds the deployment's lock from start to end, and the source checkpoint's lock too.

1. **A safety checkpoint** of the stopped deployment (4), under the same lock, kept and held until the restore is complete. Undoing a restore is restoring it.
2. **Staged:** the next generation, made from the checkpoint's as an upgrade makes `A + 1` from `A` (jig's `protocol/store/opening.md`, 2, step 2).
   - **Linked:** the snapshot, every segment, and each log whose file ends at its extent.
   - **Copied:** a log that a later state extended past C, and the secrets' store. A linked payload log is never cut to make it end at its extent.
   - **Its manifest:** written afresh, with no predecessor, so that `temper upgrade --back` refuses after a restore. The transcripts have changed, which going back assumes they haven't (jig's `protocol/store/opening.md`, 5).
3. **A staged `transcripts/`** beside the live one, as the checkpoint's: ended tasks' files linked, live tasks' copied. Staged and synced.
4. **A durable restore intent** at the deployment's top, written only once both are staged. It names:
   - the staged generation and transcript tree;
   - the safety checkpoint;
   - `active`'s digest as read.
5. **Rolled forward,** each step idempotent:
   1. the live transcript tree renamed aside;
   2. the staged one renamed into place;
   3. `active` replaced by io's durable replace, expecting that digest;
   4. the intent removed;
   5. the tree set aside removed. The safety checkpoint holds its files.

- **Every open resolves an outstanding intent before its recovery and its stray removal.** It rolls forward from whichever rename the intent's steps show was last made durable, so it never opens the restored maps with the old transcripts, or the old maps with the restored ones. A restore that fails before its intent changes nothing but its staged files, which the next open removes.
- **Its output** names the checkpoint restored and the safety checkpoint.

## 6. dev.fabrig.ai's redeploy with checkpoints

This replaces the backup and moves the check in `../redeploy.md`, 4.2 and 4.3, and in `staged-upgrades.md`, 5 and 7:

1. **While the old build serves:** a checkpoint, then the new build's full `upgrade --check` over it, keeping its worker verdict. A check that refuses ends the redeploy before anything stops.
2. **For the verdict:**
   - **yes:** only the engine stops;
   - **no or unknown, or a patch marked fresh:** the workers are drained while the old engine still serves, then it stops, as `../redeploy.md`, 4.5 takes every part together.
3. **The settled stop,** then `temper-engine checkpoint` over the stopped deployment as the backup. Only links and the small copies of 3.1 are made, with no deployment copied whole.
4. **The verdict's inputs checked again** over the stop's checkpoint: the engine's wires, whether a pending patch is fresh, the configured workers and the workers' openings. This reads those rows alone, with no second rehearsal. Any changed input is judged again by the same compatibility rules, or sends the redeploy down the together path, restarting the old engine first where draining needs it, as `../redeploy.md`, 4.2 does.
5. **The swap and the start,** with `../redeploy.md`'s opening waits, grace and failure recovery as they stand. The outage is:
   - the stop and its checkpoint;
   - the check of step 4;
   - the start's own patches and jobs;
   - for the together path, the workers' return.
6. **Going back:**
   - **before the new generation's first commit:** `temper upgrade --back`, as today;
   - **after it:** `temper-engine restore` of the stop's checkpoint, with the previous build.

Patching ahead of the stop, and a switch that adopts the check's rehearsed generation, stay later work (8). No patch file has landed since S4 (`store-shape.md`, section 3), so an upgrade's start work is now the open and its jobs.

## 7. What it is not

- **Not off-machine:** a checkpoint shares the deployment's disk. An off-machine backup is an operator's copy of one, made under its lock (5.1).
- **Not a log of every commit:** it holds the state at C alone, so a point between two checkpoints can't be restored.
- **Not the forge, the logins or the configuration.** As with a backup today, those are restored from where they live.

## 8. Later

- **A schedule** (a checkpoint every N hours) and a size alarm when checkpoints hold more than a share of the disk.
- **The switch adopting the check's rehearsed generation,** for an upgrade with a long patch or job, so that the start does none of it again. The stopped deployment's state is exactly the check's input when the check runs over the stop's checkpoint.
- **Patching ahead** on a checkpoint while the old build serves, with the commits since C caught up at the stop.

## 9. Open questions

- **An ended task's logs under S7:** whether anything appends to them after the task ends. C1 checks this against S7's code. If something does, they are copied up to their extents, as a live task's are.
- **Disk:** once the live store has replaced its files, what a checkpoint holds is measured, not assumed. C1 reports the figure from dev.fabrig.ai's store, against the snapshot, the segments and the live tasks' copies.
