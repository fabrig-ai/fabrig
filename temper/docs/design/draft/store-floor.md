# store-floor: fabrig keeps no code for stores older than dev.fabrig.ai's

Draft, 2026-11-15, from the user's words: "We don't need to support any deployment other then dev right now, so in that case all of it can go. Expectation: fabrig does not keep any code relevant for upgrading or managing stores older than current dev."

The oldest store a build opens is dev.fabrig.ai's, at its last redeploy. Everything that exists only to upgrade, read, check or test an older store goes, and an older store is refused by name, with the build to upgrade it through, before any upgrade is planned or written. The line, the floor, moves after each redeploy of dev.fabrig.ai that runs work no later build needs again.

## 1. In one page

**Today,** at main `0b14894431`, against dev.fabrig.ai's store as built by `cb429613c6`:

- **Every map patch is still in the tree,** though dev's store has run them all: jig's 1 to 6 and temper's 1 to 14, at levels jig 6 and temper 14.
  - **The patch directories,** in `jig/crates/jig-protocol-store/patches/` and `temper/crates/temper-protocol-store/patches/`: patches with their expected bytes, generated modules, authored conversions and contexts. 33,695 files, 76,632 lines.
  - **The old texts,** in the two crates' `schema/kept/`: 34 files, 11,383 lines.
  - **temper's old expected bytes,** `golden/v11` to `golden/v24`: 23,933 files.
- **The machinery only those patches need:**
  - the eager walk of a patch over the map, S2's head-only rewrite among it (`jig/crates/jig-protocol-store/src/patching/`);
  - the registry's seeding from texts in code, for a store from before S4;
  - the recognizer of the baseline store of `bc23255eb7` (`jig/crates/jig-protocol-store/src/baseline.rs`);
  - the shell's routes for patches' written sites.
- **Six of the seven jobs dev's store has done,** whose code serves only a store that hasn't run them: milestones, live-task-parts, projection-parts, retention-rows, retention-proposals and change-parts.
- **Tests and fixtures of older stores:**
  - the whole baseline deployment that `temper/tests/one-process` upgrades at every run;
  - an upgrade fixture holding all fourteen temper patches;
  - the patch suites over synthetic histories.
- **An older store is refused without a way forward, or not at all:**
  - **seeded instead of refused:** a store with no registry gets one built from texts in code;
  - **no bridge named:** the refusals of a store older than the build name no build to upgrade through.

**The ideal**

- **The floor** (2): each owner's set declares the oldest store it opens. Below it, the open refuses before any upgrade is planned or written, and names the build to upgrade through.
- **What goes** (3): the map patches' history and the walk; seeding from code; the baseline recognizer; the six done jobs' code; the fixtures, rules and words that promise older stores.
- **What stays** (4): what dev's store and its next upgrade use:
  - the registry and its conversion at read of dev's values at older layouts, from texts dev's own registry holds;
  - the schemas and value types those layouts need;
  - the jobs' runner;
  - generations and going back;
  - the check;
  - the formats outside the registry at their current versions.

  Outside this draft:
  - the wires, which keep their own rule;
  - readers of what dev's data may still hold, until a count shows none.
- **The floor moves** (5): once dev.fabrig.ai has run a build's jobs, the next landing that touches the store drops their code, and the floor names that build.
- **Later** (8): retiring layouts, value types and variants that no value holds, each by an exact proof; and checking that every layout a value holds is registered.

## 2. The floor

- **Declared by each owner's set,** jig's and temper's (`jig/crates/jig-protocol-store/src/patch_set.rs`, `temper/crates/temper-protocol-store/src/patch_set.rs`). A floor holds:
  - **the levels:** jig 6, temper 14 and the others 0, held explicitly and used by every writer and reader of levels: the first commit, the seal and the comparison at open. Today each infers an owner's level from the length of its patch array (`jig/crates/jig-protocol-store/src/patching/plan.rs`), so emptying the arrays would make dev's store look ahead of the build.
  - **the registry seeded;**
  - **the retired jobs:** the names of every job the build no longer runs, whose done rows the store must hold. That is six now, and from F4 also `task-logs`. The list is cumulative and never shares a name with a declared job, and a retired name is never given to another job (jig's `protocol/store/jobs.md`, 1).
  - **the bridge:** the exact build to upgrade through.
- **Checked by the open,** in `jig/crates/jig-protocol-store/src/open.rs`, after the format row is decoded and before:
  - any plan, patch or seeding;
  - the rehearsal's decoding of records (jig's `protocol/store/opening.md`, 5).

  The rehearsal shares the check, so a check over an older store refuses as its start would. The rehearsal never writes its source, only its scratch. The check refuses:
  - **a level below the floor;**
  - **a non-empty store with no registry,** which is refused, not seeded;
  - **a retired job with no valid done row** (`jig/crates/jig-protocol-store/src/jobs/kept.rs`).

  A store ahead of the build is refused as ahead first, as today (`format.rs`), and never told to go back through an older build. The refusals that come earlier, of an older format or an undecodable component row, name the bridge too.
- **What a refusal writes:** no upgrade, no new generation and no seeding. Opening the maps has already run skein-kv's recovery, which starts a new log segment as every open does (jig's `protocol/store/opening.md`, 1.4); nothing else is written.
- **Its words,** through the refusals' existing lines (jig's `protocol/store/README.md`, section 2): "This store is older than this build opens: its jig level is 5, below 6. Start build `cb429613c6` on it and let it open, then this build. This build upgraded nothing."
- **What it can tell apart:** the states these predicates name. A store from an earlier build in the same state as the floor is the floor's, and opens.
- **A fresh store's first commit** writes the current layouts and levels, and valid done rows for every declared job and every retired one (`jig/crates/jig-protocol-store/src/jobs/kept.rs`, `commits.rs`). Otherwise a store made after F1 would fail the floor at its next open.
- **A store from before generations** is still refused by name (`Older::Directory`), its words naming the bridge.
- **What a floor doesn't require:** the newest layout everywhere. dev's ended tasks hold layouts 13 and 14, and its landing history 25 and 26. Values at an older layout are dev's own, and convert at read from texts its registry holds (jig's `protocol/store/layouts.md`, 4).

## 3. What goes

### 3.1 The map patches' history

- **Each crate's `patches/`:** the `.patch` files, `golden/`, the generated `mod.rs`, `context.rs`, and the authored conversions (jig's `p0001.rs`; temper's `p0001`, `p0005`, `p0007`, `p0014`).
- **The old registered texts** in `schema/kept/`. The old sides are read from the store's registry (`jig/crates/jig-protocol-store/src/registry/check.rs`), never from code. A kept text that is still a format's current text, such as the component's, the registry's or the secrets', moves to the current schema's sources first.
- **temper's `golden/v11` to `golden/v24`.** Its `golden/v25` stays.
- **The `temper-protocol-store-patches` package.**
- **The patch-history fields of the owner hand-over** (`temper/crates/temper-engine-service-vocabulary/src/store.rs`), through a new API that keeps every current family, value type, held-family link, frame root, job and floor.
- **The shell's routes for patches' written sites,** among them patch 5's fetch of adoption settings (`temper/crates/temper-engine-service-vocabulary/src/store/route.rs`). The job routing in the same file stays.

### 3.2 What only patches and older stores need

- **The eager walk** in `patching/machines.rs`, `walk.rs` and `phase/walking.rs`, with S2's head-only rewrite called from it. The transcripts' frame patcher uses `walk::held_root`, `machines::capacity` and `machines::kept_patcher`, so those move out first (`jig/crates/jig-protocol-store/src/transcripts/patches.rs`).
- **Patch scheduling and levels from arrays,** in `patching/plan.rs`. Family admission, bounds and owner lookup stay, moved where they belong.
- **Seeding from code:**
  - `registry.rs`'s seed, `registry/rows.rs`'s seed rows, and `registry/texts.rs`'s and `patching/types.rs`'s use of the last kept text, replaced by the current text for registered and unregistered families alike;
  - `registry/check.rs`'s checks of pending patches and of the pre-seed census.
- **The baseline recognizer:** `jig/crates/jig-protocol-store/src/baseline.rs` and `format.rs`'s recognition of unlevelled and historical levels, with `open.rs`'s load of the old layouts row for that case. `patching/baseline.rs` is another file: it declares current families, OAuth's and the transcripts', and stays under a better name.
- **The rehearsal's decoding of a store at patch levels.** Its decoding of the registry's layouts, and of the unregistered rows at their current versions, stays (`rehearsal/decode.rs`).

### 3.3 The done jobs' code

- **Six converters go:** `jobs/milestones.rs`, `live_task_parts.rs`, `projection_parts.rs` and `retention.rs` in jig's store, and temper's `change-parts` in `temper/crates/temper-protocol-store/src/jobs.rs`, each with its dispatch arm, declaration and tests. Code that only they call goes too, such as the split of a whole projection record.
- **What stays:**
  - `Job`, `Converts`, the pages and their room, admission, `jobs/kept.rs`, the runner and the routing;
  - retention's timers, which are not the backfill jobs;
  - the hot and cold parts and their joins;
  - the before-talk reader that TaskRecord's current reader still calls (`codec/tasks/record.rs`).
- **charter-versions stays whole.** Its declaration supplies the conversion that lets the registry plan the old CharterVersion layout (`registry/check.rs`). The census can't prove that layout unused, because its value type is a subrange of the settings' census bucket. It goes once that layout is retired (8).
- **The schemas and value types stay,** tasks' milestones and temper's whole change records among them:
  - **Variants in dev's layouts:** the broad tasks `Row` and the nested projection record still carry the old variants, and a plan for dev's layouts 13, 14 and 7 derives from them.
  - **Value-type declarations:** dropping one leaves its old registry rows undeclared, refused at the next open.
  - **Positional tags:** enum tags are positional, so deleting a variant renumbers those after it (skein's `data/codec/README.md`).

  Each goes only by section 8's retirement.

### 3.4 Tests, fixtures and rules

- **With F1:** the one-process baseline deployment's success story (`temper/tests/one-process/golden/baseline/`, its reader, its expected patch table, its entry in `.config/fixture-readers.toml`) is replaced by a fixture at the floor that opens and serves, and a test that the baseline is refused by name. Every level writer moves to the explicit levels, and the done jobs' phase tests go.
- **With F2:**
  - **Rewritten:** the upgrade fixture holding the fourteen patches (`temper/tests/one-process/src/upgrade.rs`), as a registry change and a job.
  - **Removed:**
    - the generated patch tests (`temper/crates/temper-protocol-store/tests/patches.rs`);
    - the two crates' catch-up, unseeded and old-level tests;
    - the patch suites of `jig/tests/store`, whose world stays;
    - the workspace's entries for what goes.
  - **Moved out of `jig/testing/jig-test-patches*`:** the fixtures that current registry and frame tests import (`src/tests/phase/registry.rs`, `src/tests/frame_patches.rs`).
  - **The rules, with the code they describe:**
    - `tests/boundary/tests/patches.rs` narrows to the wires and the formats outside the registry.
    - The root `AGENTS.md` rule on stored families gains the floor: no code for a store older than dev.fabrig.ai's. A format outside the registry is fixed (4).
    - skein's `data/codec/command.md` drops the frozen-text rule for a store's history, and keeps it for wires.
- **With F3, the words:**
  - `temper/docs/design/protocol/baseline.md`;
  - jig's `protocol/store/layouts.md` (the seed), `opening.md` (the eager walk), `jobs.md` and `parts.md` (the six);
  - the drafts `upgrades.md`, `staged-upgrades.md`, `redeploys.md` and `store-shape.md` and their execution and status files, where they promise older stores.

  Each is rewritten to the floor, its citations repaired.

## 4. What stays

- **What dev's store and its next upgrade use:**
  - the current schemas, codecs and value types, those that dev's older layouts derive from included;
  - the registry: loading it, current registration, fingerprints, held texts, derived plans at read, retirement;
  - written jobs and their runner;
  - generations, the switch, the seal and the manifest;
  - going back;
  - the upgrade's check without its old decoders;
  - the census;
  - retention and its timers.
- **The owner hand-over's current metadata:** families, value types, held-family links, frame roots, jobs, the floor.
- **The formats outside the registry, at their current versions:**
  - the component's own rows;
  - the secrets' store;
  - OAuth's saved token 3, jig-accounts 1 and temper-secrets 1;
  - the transcripts' frames: the outer `TurnRecord` at the core's version 6, holding smith's `Turn` at version 7.

  None has a landed patch, and they are fixed by this cleanup. A later change to one designs and brings, in the same landing, how dev's store gets there: an executor, a conversion at read, or the format registered. Generating a patch alone no longer upgrades anything. The frame patcher, which patches a frame behind the binary's versions as it is read (jig's `protocol/store/transcripts.md`, 6), stays for frames.
- **Readers of what dev's data may still hold,** until a count of exactly that shows none. A layout id says nothing of which variant a value holds, or which version a turn inside a frame was written in.
  - **v6 turns:** smith-transcript's reader of turns written before v7. temper-twin and the client's transcript decoding read through it too.
  - **The legacy transcript files:** their read, write, cut and retention paths. S7 keeps writing a task's existing file until the task ends (`store-shape.md`, section 10).
  - **Pre-talk ended tasks:** their variants.
  - **The landing views' branches for an empty round branch** (`temper/crates/temper-engine-domain-forge/src/views.rs`).

  Each goes at a later move of the floor (5), after a count of turn versions and nested variants, S7's logs included, shows none.
- **The wires:** jig's and temper's worker wires, smith's charter, channel and pieces, and the clients' wires. They keep their own rule, each build speaking at least the previous version and stored wire history kept (`../protocol/wires.md`). They are not stores, and this draft changes none of them.
- **The codec's generic patch machinery:** skein-codec's patcher, its authoring command, its test story, and the layout derivation built on it. The wires use it.
- **The store world and every test of a mechanism above.**

## 5. The floor moves

- **Jobs alone, from now on.** No patch file has landed for a registered family since S4, and the formats outside it are fixed (4). A redeploy of dev.fabrig.ai that runs a build's jobs makes their code dead.
- **A small landing after such a redeploy** drops those jobs' code, adds their names to the retired list, and moves the bridge to that build. A job that supplies a conversion for a layout, as charter-versions does, waits for that layout's retirement (8).
- **Readers kept for dev's data** (4) go when a count on dev's store of exactly what they read shows none left. For the v6 turns, that is once their transcripts have passed their horizon.
- **Any other store,** such as a laptop's `temper-box` or a test deployment left behind, is refused by name. It is upgraded through the named build or started afresh. staging.fabrig.ai is made again at dev.fabrig.ai's build for each drill (`staged-upgrades.md`, 6), so it never falls below the floor.

## 6. Risks

- **Levels:** read from the floor everywhere, never from the arrays' lengths. Otherwise dev's store looks ahead of the build and is refused.
- **Layouts dev holds:** a schema variant or value type removed while one of dev's registry layouts names it makes the registry's check refuse dev's open (jig's `protocol/store/layouts.md`, 5.3). Layout ids are per value type, so a count of a head across the census proves nothing about one type. F1 tests dev's layouts 13, 14 and 7 deriving, not only current values round-tripping.
- **A fresh store** must write the retired jobs' done rows, or it fails the floor at its second open. F1 and F4 each test a first commit, a close and a reopen.
- **Each landing whole:** every milestone takes its own tests, fixtures, workspace entries and gate rules with it, so that each lands green.
- **Budgets:** the suites lose the old patch tests and fixtures. A replacement test stays within today's budgets.
- **Citations:** sections cited elsewhere are repaired, not left broken (`tools/citations/`).

## 7. Open questions

- **The floor's form:** a declaration in each set, as section 2 has it, or a row the bridge build writes once. The declaration needs nothing new on disk. A row would say "at or above the floor" in one read, and would tell a store of an earlier build from the floor's. F1's design chooses.

## 8. Later

- **Retiring what no value holds:**
  - **What:** the old CharterVersion layout, with charter-versions then; tasks' milestones (value type 10); temper's whole change records (value type 12); and the variants of the tasks `Row` and the projection record that no layout of dev's still needs.
  - **How:** each needs an exact proof that its own key range holds no value, not a census bucket's count. Its registry rows are erased in an upgrade's generation, and a design says how enum tags stay stable.
- **Every layout a value holds is registered:** today a value whose head the registry lacks is read raw (`jig/crates/jig-protocol-store/src/registry.rs`). A check at open would:
  - work per family, value type and layout id;
  - leave aside the heads of the formats outside the registry;
  - make an exact scan, or refuse as indeterminate, where a census bucket spans several value types or has overflowed;
  - leave S7's logs, which the map's census can't see, to S7's own rule that a log's layouts never retire.
