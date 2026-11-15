# store-floor: fabrig keeps no code for stores older than dev.fabrig.ai's

Draft, 2026-11-15, from the user's words: "We don't need to support any deployment other then dev right now, so in that case all of it can go. Expectation: fabrig does not keep any code relevant for upgrading or managing stores older than current dev." Then, on what dev's store still holds: "can we make a one-off migration to remove it? so the code-removal draft removes everything for good".

dev.fabrig.ai's store is first settled by a one-off migration, so that it holds every value at its build's layout and no form of anything older. The oldest store a build then opens is that settled store. Everything that exists only to upgrade, read, check or test an older store goes, the migration included, once dev has run it. An older store is refused by name, with the build to upgrade it through, before any upgrade is planned or written.

## 1. In one page

**Today,** at main `0b14894431`, against dev.fabrig.ai's store as built by `cb429613c6` (its census of 11-15 13:08Z):

- **Every map patch is still in the tree,** though dev's store has run them all: jig's 1 to 6 and temper's 1 to 14, at levels jig 6 and temper 14.
  - **The patch directories,** in `jig/crates/jig-protocol-store/patches/` and `temper/crates/temper-protocol-store/patches/`: patches with their expected bytes, generated modules, authored conversions and contexts. 33,695 files, 76,632 lines.
  - **The old texts,** in `schema/kept/`: 34 files.
  - **temper's old expected bytes,** `golden/v11` to `golden/v24`: 23,933 files.
- **The machinery only those patches need:**
  - the eager walk, S2's head-only rewrite among it;
  - the registry's seeding from texts in code, for a store from before S4;
  - the recognizer of the baseline store of `bc23255eb7`;
  - the shell's routes for patches' written sites.
- **The seven jobs dev's store has done.**
- **Tests and fixtures of older stores,** such as the baseline deployment that `temper/tests/one-process` upgrades at every run.
- **dev's store itself still holds older forms:**
  - **Values at an older layout,** converted at every read: 3,611 ended tasks at layout 13 (61 at 14), and 4,119 landing-history rows at 25 (250 at 26).
  - **Registry rows for layouts no value holds:** the registry has 114 layout rows, against 79 layouts the census sees held. Among them are tasks' milestones and temper's whole change records, both empty since S5 and S8, and the charter versions' old layout.
  - **The schema has to keep their old variants and value types,** because the registry's plans from those layouts derive through them. The tasks `Row` keeps its milestone and whole live forms, and the projection record its whole forms. Removing them would refuse dev's open.
- **An older store is refused without a way forward, or not at all:**
  - **seeded instead of refused:** a store with no registry gets one built from texts in code;
  - **no bridge named:** the refusals of a store older than the build name no build to upgrade through.

**The ideal**

- **F0 settles dev's store once** (2). It is a migration at open, in dev.fabrig.ai's redeploy with S7, before S7's job moves history into the task logs. It:
  - drops from the schema every variant and value type no current writer makes;
  - converts every value to its build's layout;
  - retires every other registry row;
  - writes the store's floor.
- **The floor** (3): a row in the store, written by F0 and by every fresh store's first commit. A store without it, or below it, is refused before any upgrade is planned or written, naming the build to upgrade through.
- **Then all of it goes** (4): the map patches' history and the walk, seeding from code, the baseline recognizer, all seven done jobs, S7's `task-logs`, F0's own migration, and the fixtures, rules and words that promise older stores.
- **What stays** (5) is what every future upgrade uses, not anything for an older store:
  - conversion at read, from texts the store's registry holds;
  - the jobs' runner;
  - generations and going back;
  - the check;
  - the formats outside the registry, at their current versions.

  The wires keep their own rule, outside this draft.
- **The floor moves** (6): each later redeploy of dev.fabrig.ai that runs a job is followed by a landing that drops that job's code.

## 2. F0: settling dev's store, once

A migration at open (jig's `protocol/store/jobs.md`), carried by the build that dev.fabrig.ai is redeployed onto with S7, and rehearsed first by that redeploy's check.

- **The schema drops** every variant and value type that no current writer makes:
  - **tasks' milestones** (value type 10) and **temper's whole change records** (value type 12), their keyspaces empty since S5 and S8;
  - **in the tasks `Row`,** the milestone variant and the whole live forms;
  - **in the projection record,** its whole goal and task forms;
  - **the charter versions' old layout,** with the conversion charter-versions supplies for it.

  The milestone's design says how the remaining variants' tags move: they are positional (skein's `data/codec/README.md`), so the conversion maps old tags to new.
- **Written jobs convert what a plan can't** (jig's `protocol/store/layouts.md`, 6). For each layout dev's registry holds that has lost a variant, a job:
  - reads the values at that layout as the store holds them, by that layout's text;
  - puts each one again at the build's layout;
  - refuses a value that holds a dropped variant. The open then refuses before its switch and leaves the store as it was (jig's `protocol/store/opening.md`, 2), so nothing is lost, and the milestone's design goes back to that form.
- **What a plan reaches is put back at the build's layout too:** temper's landing history at 25, for one. After F0, every value of every registered value type is at the build's layout.
- **Retirement by proof:**
  - Each job walks its value type's ranges whole, so its end commit erases that type's other registry rows and the texts no other row names (jig's `protocol/store/layouts.md`, 5.4).
  - The two dropped value types are declared retired, with their ranges. The open reads those ranges, refuses unless they are empty, and erases their registry rows.
  - This is exact where a census bucket would not be, as with the charter versions' subrange of the settings' bucket.
- **Before S7's `task-logs`,** in the same open: the jobs run in their sets' order (jig's `protocol/store/jobs.md`, 1), so F0's jig jobs are declared before it. History then enters the task logs already at the build's layouts. S7's design never retires a log-held layout, so a store whose history reached its logs first would keep its old forms there.
- **Counted, and converted where a current form exists:** what a layout id can't show.
  - **Transcript turns by their inner version.** smith's `Turn` 7 was in the tree from 11-08, before dev's oldest transcript of 11-10, so none at 6 is expected.
  - **Ended tasks by variant.**
  - **Landing records with an empty round branch** (`temper/crates/temper-engine-domain-forge/src/views.rs`).

  A form F0 can't convert is reported with its count. Its reader stays until retention drops what holds it: turns 14 days after their task ends, ended tasks' full records 30 days after (`store-shape.md`, section 6). It goes with the next floor move after that (6).
- **The floor row** is written in F0's last commit (3).
- **Cost at open:** a read of the map's 220,890 rows and a few thousand puts, about 15 s at the rates of 11-15's upgrade. The redeploy's check measures it first.

## 3. The floor

- **A row in the store's component family** (jig's `protocol/store/records.md`, 3): the floor's number, and the build that wrote it.
  - **Written by F0's last commit** on dev's store.
  - **Written by every fresh store's first commit,** at the build's floor. A store made after F1 therefore opens again.
- **Checked by the open,** in `jig/crates/jig-protocol-store/src/open.rs`, after the format row is decoded and before:
  - any plan or seeding;
  - the rehearsal's decoding of records (jig's `protocol/store/opening.md`, 5).

  The rehearsal shares the check, so a check over an older store refuses as its start would. It never writes its source, only its scratch.
- **What it refuses:**
  - a store with no floor row, or one below the build's;
  - a non-empty store with no registry, which is refused, not seeded.

  A store ahead of the build is refused as ahead first, as today (`format.rs`). The refusals that come earlier, of an older format or an undecodable component row, name the bridge too.
- **Levels:** jig 6 and temper 14 stay the stores' levels, held explicitly by the first commit, the seal and the comparison at open. Today each infers an owner's level from the length of its patch array (`jig/crates/jig-protocol-store/src/patching/plan.rs`), so emptying the arrays would make dev's store look ahead of the build.
- **What a refusal writes:** no upgrade, no new generation and no seeding. Opening the maps has already run skein-kv's recovery, which starts a new log segment as every open does (jig's `protocol/store/opening.md`, 1.4).
- **Its words,** through the refusals' existing lines (jig's `protocol/store/README.md`, section 2): "This store is older than this build opens: it has no floor. Start build `<F0's build>` on it and let it open, then this build. This build upgraded nothing." The bridge is named exactly, by its commit.
- **A store from before generations** is still refused by name (`Older::Directory`), its words naming the bridge.
- **The jobs' done rows** stay in the store, inert, since the runner looks up declared jobs alone. A retired job's name is never given to another job (jig's `protocol/store/jobs.md`, 1).

## 4. What goes

Once dev.fabrig.ai has run F0 and S7's `task-logs`, every item below goes, and nothing in the tree reads or upgrades a store from before the floor.

### 4.1 The map patches' history

- **Each crate's `patches/`:** the `.patch` files, `golden/`, the generated `mod.rs`, `context.rs`, and the authored conversions (jig's `p0001.rs`; temper's `p0001`, `p0005`, `p0007`, `p0014`).
- **The old registered texts** in `schema/kept/`. After F0 the registry holds the build's layouts alone, and its texts are in the store. A kept text that is still a format's current text, such as the component's, the registry's or the secrets', moves to the current schema's sources first.
- **temper's `golden/v11` to `golden/v24`.** Its `golden/v25`, or whatever is current by then, stays.
- **The `temper-protocol-store-patches` package.**
- **The patch-history fields of the owner hand-over** (`temper/crates/temper-engine-service-vocabulary/src/store.rs`), through a new API that keeps every current family, value type, held-family link, frame root and job.
- **The shell's routes for patches' written sites** (`temper/crates/temper-engine-service-vocabulary/src/store/route.rs`). The job routing stays.

### 4.2 What only patches and older stores need

- **The eager walk:** `patching/machines.rs`, `walk.rs` and `phase/walking.rs`, with S2's head-only rewrite. The frame patcher's helpers (`walk::held_root`, `machines::capacity`, `machines::kept_patcher`; `jig/crates/jig-protocol-store/src/transcripts/patches.rs`) move out first.
- **Patch scheduling and levels from arrays,** in `patching/plan.rs`. Family admission, bounds and owner lookup stay, moved where they belong.
- **Seeding from code:**
  - `registry.rs`'s seed and `registry/rows.rs`'s seed rows;
  - `registry/texts.rs`'s and `patching/types.rs`'s use of the last kept text, replaced by the current text;
  - `registry/check.rs`'s checks of pending patches and of the pre-seed census.
- **The baseline recognizer** (`jig/crates/jig-protocol-store/src/baseline.rs`), with `format.rs`'s recognition of unlevelled and historical levels and `open.rs`'s load of the old layouts row. `patching/baseline.rs` declares current families and stays under a better name.
- **The rehearsal's decoding of a store at patch levels.** Its decoding of registry layouts and of current unregistered rows stays (`rehearsal/decode.rs`).

### 4.3 Every done job, and F0

- **Gone, each with its dispatch, declaration and tests:**
  - jig's `charter-versions`, `milestones`, `live-task-parts`, `projection-parts`, `retention-rows` and `retention-proposals`;
  - temper's `change-parts`;
  - S7's `task-logs` and the map's history readers that only it used;
  - F0's written jobs, its put-back and its retired value types' declarations.

  Code that only they call goes too, such as the split of a whole projection record. The schema variants and value types F0 dropped are gone already.
- **Staying:**
  - `Job`, `Converts`, the pages and their room, admission, `jobs/kept.rs`, the runner and the routing, for the next job;
  - retention's timers, which are not the backfill jobs;
  - the hot and cold parts and their joins;
  - S7's log decoders and the registry's log-held layout checks;
  - TaskRecord's current reader and what it calls.

### 4.4 Tests, fixtures and rules

- **Fixtures of older stores:**
  - **The one-process baseline deployment and its success story** (`temper/tests/one-process/golden/baseline/`, its reader and expected patch table, `.config/fixture-readers.toml`'s entry) are replaced by a fixture at the floor that opens and serves, and a test that a store without a floor is refused by name.
  - **The upgrade fixture** holding the fourteen patches (`temper/tests/one-process/src/upgrade.rs`) is rewritten as a registry change and a job.
- **Patch tests:**
  - **Removed:** the generated patch tests, the catch-up, unseeded and old-level tests, the patch suites of `jig/tests/store` (whose world stays), and the workspace's entries for what goes.
  - **Moved out of `jig/testing/jig-test-patches*`:** the fixtures that current registry and frame tests import.
- **The rules:**
  - `tests/boundary/tests/patches.rs` narrows to the wires and the formats outside the registry.
  - The root `AGENTS.md` rule on stored families gains the floor: no code for a store older than dev.fabrig.ai's, and a format outside the registry is fixed (5).
  - skein's `data/codec/command.md` drops the frozen-text rule for a store's history, and keeps it for wires.
- **The words** that promise older stores are rewritten to the floor, their citations repaired:
  - `temper/docs/design/protocol/baseline.md`;
  - jig's `protocol/store/layouts.md` (the seed), `opening.md` (the eager walk), `jobs.md` and `parts.md` (the done jobs);
  - the drafts `upgrades.md`, `staged-upgrades.md`, `redeploys.md` and `store-shape.md`, and their execution and status files.

## 5. What stays, and why it isn't old

These serve dev's store as it will be and every future upgrade of it, not an older store:

- **Conversion at read:** derived plans from the texts in the store's registry (jig's `protocol/store/layouts.md`, 4). It is how the next schema change reaches dev with no patch and no walk: a value written before the change keeps its layout, and is read through a plan built from data the store holds. It is one generic mechanism with no code per change.
- **The registry:** loading it, current registration, fingerprints, held texts, retirement.
- **The jobs' runner:** how a future change of keys or of meaning runs once at open.
- **Generations, the switch, the seal, the manifest and going back:** what every upgrade writes in and switches through.
- **The upgrade's check, without its old decoders,** and **the census** and **retention**.
- **The formats outside the registry, at their current versions:**
  - the component's own rows;
  - the secrets' store;
  - OAuth's saved token 3, jig-accounts 1 and temper-secrets 1;
  - the transcripts' frames: the outer `TurnRecord` at the core's version 6, holding smith's `Turn` at 7.

  None has a landed patch, and this cleanup fixes them. A later change to one designs, and brings in the same landing, how dev's store gets there: an executor, a conversion at read, or the format registered. The frame patcher, which patches a frame behind the binary's versions as it is read (jig's `protocol/store/transcripts.md`, 6), stays.
- **The wires:** jig's and temper's worker wires, smith's charter, channel and pieces, and the clients' wires. They keep their own rule, each build speaking at least the previous version and stored wire history kept (`../protocol/wires.md`). They are not stores, and this draft changes none of them.
- **The codec's generic patch machinery:** skein-codec's patcher, its authoring command, its test story and the layout derivation built on it. The wires use it.

## 6. The floor moves

- **Jobs alone, from now on.** No patch file has landed for a registered family since S4, and the formats outside the registry are fixed (5). A redeploy of dev.fabrig.ai that runs a build's jobs makes their code dead.
- **A small landing after such a redeploy** drops those jobs' code. A job that dropped a variant or value type has already settled what it converted, so nothing older stays behind it.
- **The floor's number moves** only when a landing drops a whole older form, as F0 does. The bridge named in the refusal is then that build.
- **Any other store,** such as a laptop's `temper-box` or a test deployment left behind, is refused by name. It is upgraded through the named builds or started afresh. staging.fabrig.ai is made again at dev.fabrig.ai's build for each drill (`staged-upgrades.md`, 6), so it never falls below the floor.

## 7. Risks

- **dev's redeploy with S7 waits for F0.** If S7 reaches dev first, its history enters the logs at the old layouts, which never retire there. The variants that F0 drops would then stay in the schema until a later design rewrites those logs.
- **A dropped variant a value still holds** makes F0's job refuse dev's open, before its switch, with nothing lost. The milestone then keeps that variant, or converts it. The redeploy's check finds this first, over a copy.
- **Levels:** read from the floor everywhere, never from the arrays' lengths.
- **Each landing whole:** every milestone takes its own tests, fixtures, workspace entries and gate rules with it, so that each lands green.
- **Budgets:** the suites lose the old patch tests and fixtures. A replacement test stays within today's budgets.
- **Citations:** sections cited elsewhere are repaired, not left broken (`tools/citations/`).

## 8. Later

- **Every layout a value holds is registered:** today a value whose head the registry lacks is read raw (`jig/crates/jig-protocol-store/src/registry.rs`). A check at open would:
  - work per family, value type and layout id;
  - leave aside the heads of the formats outside the registry;
  - make an exact scan, or refuse as indeterminate, where a census bucket spans several value types or has overflowed;
  - leave S7's logs, which the map's census can't see, to S7's own rule.
