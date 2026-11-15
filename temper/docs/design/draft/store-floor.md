# store-floor: fabrig keeps no code for stores older than dev.fabrig.ai's

Draft, 2026-11-15, from the user's words: "We don't need to support any deployment other then dev right now, so in that case all of it can go. Expectation: fabrig does not keep any code relevant for upgrading or managing stores older than current dev."

The oldest store a build opens is dev.fabrig.ai's, at its last redeploy. Everything that exists only to upgrade, read, check or test an older store goes, and an older store is refused by name before anything is written, with the build to upgrade it through. The line, the floor, moves after each redeploy of dev.fabrig.ai that runs work no later build needs again.

## 1. In one page

**Today,** at main `0b14894431`, against dev.fabrig.ai's store as built by `cb429613c6`:

- **Every map patch is still in the tree, though dev's store has run them all:** jig's 1 to 6 and temper's 1 to 14, at levels jig 6 and temper 14.
  - **Patch directories:** in `jig/crates/jig-protocol-store/patches/` and `temper/crates/temper-protocol-store/patches/`, with their expected bytes, generated modules, authored conversions and contexts: 33,695 files, 76,632 lines.
  - **Old texts:** 34 files and 11,383 lines in the two crates' `schema/kept/`.
  - **temper's old expected bytes,** `golden/v11` to `golden/v24`: 23,933 files.
- **The machinery only those patches need:**
  - the eager walk of a patch over the map, S2's head-only rewrite among it (`jig/crates/jig-protocol-store/src/patching/`);
  - the registry's seeding from texts in code, for a store from before S4 (`registry.rs`, `registry/rows.rs`, `registry/texts.rs`);
  - the recognizer of the baseline store of `bc23255eb7` (`baseline.rs`), and the shell's routes for patches' written sites.
- **The seven jobs dev's store has done** (charter-versions, milestones, live-task-parts, projection-parts, retention-rows, retention-proposals, change-parts): 662 lines. Also the stored forms only they read: tasks' milestones, whole live tasks and projections, whole temper change records.
- **Tests and fixtures of older stores:**
  - the whole baseline deployment that `temper/tests/one-process` upgrades at every run (69 files);
  - an upgrade fixture holding all fourteen temper patches;
  - `jig/testing/jig-test-patches*`;
  - the patch suites of jig's store world.
- **An older store isn't refused cleanly:**
  - a store with no registry is seeded from code;
  - a value whose layout the registry doesn't hold is read raw;
  - the refusals of a store older than the build name no build to upgrade through.

**The ideal**

- **The floor** (2): each owner's set declares the oldest store it opens. Below it the open refuses, naming the build to upgrade through, before planning or writing anything.
- **What goes** (3): the map patches' history and the walk, seeding from code, the baseline recognizer, the done jobs and what only they read, and the fixtures, rules and words that promise older stores.
- **What stays** (4): what dev's store and its next upgrade use, which includes:
  - the registry and its conversion at read of dev's values at older layouts, from texts dev's own registry holds;
  - the jobs' runner;
  - generations and going back;
  - the check.

  Outside this draft: the wires, which keep their own rule, and readers of what dev's data may still hold until a count shows none.
- **The floor moves** (5): once dev.fabrig.ai has run a build's jobs, the next landing that touches the store drops them, and the floor names that build.

## 2. The floor

- **Declared by each owner's set,** jig's and temper's (`jig/crates/jig-protocol-store/src/patch_set.rs`, `temper/crates/temper-protocol-store/src/patch_set.rs`). A floor holds:
  - **the levels:** jig 6, temper 14 and the others 0, each held explicitly. Today's plan infers an owner's level from the length of its patch array (`jig/crates/jig-protocol-store/src/patching/plan.rs`), so emptying the arrays would make dev's store look ahead of the build;
  - **the registry seeded;**
  - **the done rows** of every job the build no longer declares: the seven now, and from F4 `task-logs`;
  - **the bridge,** the build to upgrade through.
- **Checked by the open,** in `jig/crates/jig-protocol-store/src/open.rs`, after the format row is decoded and before any plan, patch or seeding. The rehearsal shares the check (jig's `protocol/store/opening.md`, 5), so a check over an older store refuses as its start would. It refuses:
  - **a level below the floor;**
  - **a non-empty store with no registry,** which is refused, not seeded;
  - **a missing done row;**
  - **a layout id in the census that the registry doesn't hold** (jig's `protocol/store/README.md`, 6.3), which is refused, not read raw.
- **Its words,** through the refusals' existing lines (jig's `protocol/store/README.md`, section 2): "This store is older than this build opens: its jig level is 5, below 6. Start build `cb429613c6` on it and let it open, then this build. This build wrote nothing." The bridge is the floor's build, and is named exactly.
- **Unchanged:**
  - an empty store, whose first commit writes the current rows and every declared job's done row;
  - a store from before generations, refused by name as today (`Older::Directory`);
  - a store ahead of the build, refused as ahead and never told to go back through an older build.
- **What a floor doesn't require:** the newest layout everywhere. dev's ended tasks hold layouts 13 and 14, and its landing history 25 and 26. Values at an older layout are dev's own and convert at read, from texts its registry holds (jig's `protocol/store/layouts.md`, 4).
- **Retired job names** stay listed in the set, never reused for another job (jig's `protocol/store/jobs.md`, 1). Their done rows stay in the store, inert, since the runner looks up declared jobs alone.

## 3. What goes

### 3.1 The map patches' history

- **The patch files:** each crate's `patches/` (`.patch` files, `golden/`, generated `mod.rs`, `context.rs`, and the authored conversions `p0001.rs` and temper's `p0001`, `p0005`, `p0007`, `p0014`).
- **The texts:** the old sides in `schema/kept/`. A kept text that is a format's current text moves out to the current schema's sources first.
- **The rest:** temper's `golden/v11` to `golden/v24` (its `golden/v25` stays), the `temper-protocol-store-patches` package, and the four owner sets' hand-over in the engine's vocabulary (`temper-engine-service-vocabulary/src/store.rs`).
- **The shell's written-site contexts and routes,** among them patch 5's fetch of adoption settings (`temper-engine-service-vocabulary/src/store/route.rs`). The job routing in the same file stays.

### 3.2 What only patches and older stores need

- **The eager walk:** `patching/machines.rs`, `walk.rs` and `phase/walking.rs`, and S2's head-only rewrite called from them. The transcripts' frame patcher uses `walk::held_root`; it moves out first.
- **Patch scheduling and levels from patch arrays** in `patching/plan.rs`. The family admission, bounds and owner lookup in the same file stay, moved where they belong.
- **Seeding from code:** `registry.rs`'s seed, `registry/rows.rs`'s seed rows, and `registry/texts.rs`'s use of the last kept text. Also `registry/check.rs`'s checks of pending patches against registered families and of the pre-seed census.
- **The baseline recognizer:** `baseline.rs`, and `format.rs`'s recognition of unlevelled baselines and historical levels. `open.rs`'s load of the old layouts row for that case goes with it.
- **The rehearsal's decoding of a store at patch levels** (`rehearsal/decode.rs`). Its decoding of a store's registry layouts stays.

### 3.3 The done jobs, and what only they read

- **The seven jobs:**
  - **The converters:** `jobs/charter_versions.rs`, `milestones.rs`, `live_task_parts.rs`, `projection_parts.rs` and `retention.rs` in jig's store, and temper's `jobs.rs` `change-parts`.
  - **Their wiring:** their dispatch arms, declarations, tests, and charter-versions' conversion exemption.
  - **What stays:** `Job`, `Converts`, the pages and room, admission, `jobs/kept.rs`, the runner and the routing.
- **Stored forms whose keyspaces dev holds empty** (its census at 13:08Z, 11-15), removed with their schema variants, codecs, tests and every exhaustive match over them. No surviving variant is renumbered:
  - tasks' milestones (`01 01 0a`, value type 10);
  - temper's whole change records (`02 0c`, value type 12, `Stored::Change`).
- **The jobs' own readers:** of whole live tasks and whole projection records. The hot and cold parts and their joins stay.

### 3.4 Tests, fixtures and rules

- **The one-process baseline deployment** (`temper/tests/one-process/golden/baseline/`), its reader and its expected patch table. It is replaced by a fixture at the floor and a test that an older store is refused by name.
- **The upgrade fixture holding the fourteen patches** (`temper/tests/one-process/src/upgrade.rs`), rewritten as a registry change and a job.
- **The test patch crates and suites:**
  - `jig/testing/jig-test-patches` and `jig-test-patches-jig`;
  - the patch suites of `jig/tests/store`, whose world stays;
  - the catch-up, unseeded and old-level tests of the two store crates;
  - the old jobs' phase tests.
- **The rules:**
  - The root `AGENTS.md` rule on stored families keeps the registered families' rule and the other formats' patch rule, and gains the floor's: no code for a store older than dev.fabrig.ai's.
  - skein's `data/codec/command.md` keeps the frozen-text rule for wires and the unregistered formats, and drops it for a store's history.
  - `tests/boundary/tests/patches.rs` narrows to them.
  - `.config/fixture-readers.toml` loses the baseline's entry.
- **The words:**
  - `temper/docs/design/protocol/baseline.md`;
  - jig's `protocol/store/layouts.md` (the seed), `opening.md` (the eager walk), `jobs.md` and `parts.md` (the seven);
  - the drafts `upgrades.md`, `staged-upgrades.md`, `redeploys.md` and `store-shape.md` and their execution and status files, where they promise older stores.

  Each is rewritten to the floor, its citations repaired.

## 4. What stays

- **What dev's store and its next upgrade use:**
  - the current schemas and codecs;
  - the registry: loading it, current registration, fingerprints, held texts, derived plans at read, retirement;
  - written jobs and their runner;
  - generations, the switch, the seal and the manifest;
  - going back;
  - the upgrade's check without its old decoders;
  - the census;
  - retention and its timers.
- **The formats outside the registry, at their current versions:** the component's own rows, the secrets' store, OAuth's saved token 3, jig-accounts 1, temper-secrets 1 and transcript frames 7. None has a landed patch. Their empty patch scaffolding may be simplified, but their codecs stay, and a change to one still brings its patch under the codec's rule.
- **Readers of what dev's data may still hold,** until a count of dev's store shows none. A layout id doesn't say which variant a value holds, nor which version a transcript's turn was written in:
  - smith-transcript's v6 reader, for turns written before v7 in transcripts that stay until their 14-day horizon;
  - ended tasks' variants from before talk;
  - the landing views' branches for an empty round branch (`temper-engine-domain-forge/src/views.rs`).

  Each goes at a later move of the floor (5).
- **The wires:** jig's and temper's worker wires, smith's charter, channel and pieces, and the clients' wires keep their own rule, each build speaking at least the previous version (`../protocol/wires.md`). They are not stores, and this draft changes none of them.
- **The codec's generic patch machinery** (skein-codec's patcher, its authoring command and its test story), which the wires use, and the layout derivation built on it.
- **The store world and every test of a mechanism above.**

## 5. The floor moves

- **Jobs alone, from now on:** no patch file has landed for a registered family since S4, and none can (`AGENTS.md`). A redeploy of dev.fabrig.ai that runs a build's jobs makes them dead code.
- **A small landing after such a redeploy** drops those jobs and what only they read, lists their names as retired, and moves the floor's done rows and bridge to that build. A patch to an unregistered format goes the same way once dev's store has run it.
- **Readers kept for dev's data** (4) go when a count on dev's store shows none left: the v6 turns once their transcripts pass their horizon, and the others once counted.
- **Any other store,** a laptop's `temper-box` or a test deployment left behind, is refused by name and started afresh or upgraded through the named build. staging.fabrig.ai is made again at dev.fabrig.ai's build for each drill (`staged-upgrades.md`, 6), so it never falls below the floor.

## 6. Risks

- **Levels:** read explicitly from the floor, never from the arrays' lengths. Otherwise dev's store looks ahead of the build and is refused.
- **Variants:** a variant dropped while dev's store holds it makes the registry's check refuse dev's open (jig's `protocol/store/layouts.md`, 5.3). Only what dev holds none of is dropped, and every other variant keeps its number.
- **Budgets:** the suites lose the old patch tests and fixtures. A replacement test stays within today's budgets.
- **Citations:** sections cited elsewhere are repaired, not left broken (`tools/citations/`).

## 7. Open questions

- **The floor's form:** a declaration in each set, as section 2 has it, or a row the bridge build writes once. The declaration needs nothing new on disk. A row would say "at or above the floor" in one read. F1's design chooses.
