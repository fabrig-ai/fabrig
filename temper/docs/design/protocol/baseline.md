# The baseline

Provisional, 2026-11-11. The store every later temper upgrades from:
what it fixes, the kept texts and patch sets that make a build patch a
store at it, the golden store that proves it, and the gate that holds
every later change of what temper stores to its patch. It carries
`../draft/upgrades.md`, sections 4.1, 4.2, 12 and 13 for a deployment's
store, moved by the user's request of 2026-11-11: the baseline is the
store `main` wrote at `bc23255eb7`, the start of the goal that built
it, not the first release. The mechanisms are skein's
(`data/codec/README.md`, sections 5 to 9; `data/codec/command.md`) and
jig's (`protocol/store/records.md`, 3; `protocol/store/opening.md`);
this document says what temper's deployment fixes with them.

## 1. In one page

- **The baseline is `bc23255eb7`'s store.** Its thirteen stored
  families' layouts (2) are the first kept versions. A deployment that
  ran that build, or any build of `main` after it until this design
  landed, upgrades to every later `main` without a reset.
- **Every owner at level 0:** each owner crate's current texts kept by
  the patch command's `--baseline` run, no patch (3).
- **The real sets handed:** temper-service hands jig's store the four
  owners' sets in place of `PatchSets::NONE`; a first start writes
  level 0 for each, and `temper versions` prints them (4).
- **A store at the baseline reads as level 0,** though its row records
  none: jig compares it with a constant of the baseline's versions and
  layouts (5).
- **The golden store:** that build's deployment, made from a story in
  the one-process world and kept for good; every later build starts on
  it, patching through every landed patch, and asserts what it promised
  (6).
- **The policy:** from the baseline, a change of a stored layout lands
  with its patch, and moves its family's version in that landing; a
  focused test holds it (7).
- **The catch-up:** what `main` changed in place after `bc23255eb7`
  comes in as jig's and temper's first patches, from the baseline (9).

## 2. What it fixes

| Family, version | Crate | Owner | Where its records are |
|---|---|---|---|
| `skein-oauth` 3 | `skein/crates/skein-oauth` | skein | the secrets' map, its sources' records |
| `smith-transcript` 7 | `smith/crates/smith-transcript` | smith | transcripts' frames, in the core's `TurnRecord` |
| `jig-store-component` 3, `jig-store-core` 6, `jig-store-tasks` 9, `jig-store-people` 14, `jig-store-notes` 1, `jig-store-accounts` 2, `jig-store-settings` 1 | `jig/crates/jig-protocol-store` | jig | the store's map; the core's frames |
| `jig-store-secrets` 1 | `jig/crates/jig-protocol-store` | jig | the secrets' map |
| `jig-accounts` 1 | `jig/crates/jig-protocol-accounts` | jig | the secrets' map (jig's `protocol/store/secrets.md`) |
| `temper-store` 11 | `temper/crates/temper-protocol-store` | temper | the store's map, under temper's tag |
| `temper-secrets` 1 | `temper/crates/temper-protocol-records` | temper | the secrets' map, temper's numbers |

- **Beside the records:** the disk formats (skein-kv's frames and
  snapshot, the transcripts' framing, the deployment's directory with
  its generations) are read as that build wrote them
  (`../draft/upgrades.md`, 6.5); the small files and the configuration
  are as `README.md`, section 7 says.
- **Not this baseline's:** `smith-local-store`, smith's local host's own
  store, which no temper deployment holds, keeps its baseline at the
  first release (smith's `protocol/transcript.md`, section 12), and so
  do the client's kept requests (`../draft/upgrades.md`, 7.4).
- **The application's version** in jig's format row is `STORE_VERSION`,
  11 at the baseline; temper's patches move it (jig's
  `protocol/store/records.md`, 2).

## 3. The kept texts

- **One `--baseline` run per owner crate,** the six of 2's table:
  `skein-codegen patch <crate> --baseline` (skein's
  `data/codec/command.md`, 1), keeping each family's current text,
  its greatest version, as `schema/kept/<family>-v<N>.schema`, and
  generating `patches/mod.rs` and `patches/context.rs` with no patch.
  No version moves and no generated codec changes.
- **smith-transcript's hand-written module goes:** its `src/patches.rs`,
  empty in the generated shape, is replaced by the generated
  `patches/mod.rs` under the same module name (smith's
  `protocol/transcript.md`, section 12). Its archived `transcript-v6.schema`
  stays a current reader's text and is never kept.
- **Each crate includes its module** (`#[path = "../patches/mod.rs"] pub
  mod patches;`) and takes skein-codegen in its tests for the module's
  generated `current` test.
- **Nothing reads them until the sets** (4): the run changes no byte a
  build writes, so it may land before or after the golden store (6).

## 4. The sets

Each owner's `PatchSet` (jig's `protocol/store/opening.md`, 3) is a
constant built from its crate's generated module, naming no other owner:

```rust
// jig_protocol_store: skein's and smith's records live in jig's maps, so
// jig builds their sets from their crates' constants
pub const SKEIN_PATCH_SET: PatchSet; // skein-oauth's `SavedToken`, map Secrets, no keys: held in jig's `Source`
pub const SMITH_PATCH_SET: PatchSet; // smith-transcript's `Turn`, frames "Turn", no keys, fresh FRESH_WINDOWS
// jig_protocol_accounts: the lowest jig crate that sees every jig family,
// since jig-protocol-accounts depends on jig-protocol-store
pub const JIG_PATCH_SET: PatchSet;   // jig's eight store families and jig-accounts; patches from jig_protocol_store::patches
// temper_protocol_store
pub const PATCH_SET: PatchSet;       // temper-store and temper-secrets
// temper_service::store
pub const PATCH_SETS: PatchSets;     // the four, upstream first
```

- **jig's set:** each family's ranges from the component's key table
  (jig's `protocol/store/README.md`, 4.4), leaving out what lies among
  them but is another's; each range's root its records' declaration;
  the core's frames' root `TurnRecord` (jig's `protocol/store/opening.md`, 3).
- **Held families have no keys:** a family whose every record is held
  inside another family's record has empty `keys`, like smith's, and is
  patched at `Step::Held` (jig's `protocol/store/opening.md`, 5).
  skein-oauth's `SavedToken` is held in jig-store-secrets' `Source`, which
  owns the sources' range `[3, 0]` to `[3, 1]`. jig-accounts' `Moved` is
  held in `Carried`.
- **temper's set:** `temper-store`'s ranges, relative to temper's tag,
  one per tag of temper's keys (`store.md`, section 3) with its root, and its
  `flights` the `Flight` tag's range, `[21]` to `[22]`; `temper-secrets`
  in the secrets' map below 2^63, relative to its prefix (jig's
  `protocol/store/secrets.md`). `fresh` none.
- **Handed by temper's store shell:** temper-service's `Store` calls
  jig's `Store::patch_sets` with `PATCH_SETS` before its open, beside
  `wires`. Its test seam: `Service::patch_sets`, called before
  `adopt_store`, replaces them for the next open; production never calls
  it. Every shape with a store takes the same sets: `temper serve`, the
  engine alone, the box.
- **`versions` reads the same constant:** `temper-engine-shell`'s
  `versions` prints `jig_protocol_store::levels(&PATCH_SETS)`, so a build
  at the baseline prints `level skein 0`, `level smith 0`, `level jig 0`
  and `level temper 0` (`binaries.md`, 2).
- **Checked where admitted:** every open checks the sets (jig's
  `protocol/store/opening.md`, 3), so a first start under them is the
  plan's proof. That every record of a map lies in exactly one family's
  range takes a walk: a focused test walks the golden store's two maps
  (6) and finds each key in one range of `PATCH_SETS`. The component's
  own rows (the format row, the layouts, the wires, the installation's
  identifier) are in no range (jig's `protocol/store/opening.md`, 3).
  The walk lists them on their own and finds each one exactly where the
  component writes it.

## 5. A store at the baseline

The baseline's build handed no set, so its stores' format rows record
every level none. jig reads such a row as level 0 for each owner whose
level the binary sets, when its versions and layouts are the baseline's
(jig's `protocol/store/records.md`, 3, `BASELINE`); then it compares as
for any store at level 0: equal, it opens; behind, it patches a copy. A
row of levels none at any other layout is from before the baseline, and
refused as an older build's, as it was. The rule holds for good: such a
store keeps its row until the first patch it meets writes the levels.

## 6. The golden store

- **What:** the deployment's whole data as the baseline's build left it,
  the files of every root of the one-process world's machine
  (`../testing/README.md`, 2.5): the deployment's directory (`active`,
  the generation's `store/` and `secrets/`, `transcripts/`), the
  worker's and the client's roots, the setup's, and the configuration
  the world started the service with, kept as a file beside them. Kept
  in `tests/one-process/golden/baseline/`, one directory per root, with
  a `README.md` naming the commit and the story that made it.
- **Made once, by the baseline's code:** a story of the one-process
  world, run with the production crates as at `bc23255eb7`, before any
  change of this design touched what a start writes: a person signs in
  by the printed link and adds an account; a repository adopted on the
  fake forge, its records and secrets kept; a change landed through it,
  as story A lands one; a chat that replied and waits; a coding task
  mid-run; then a signal, the settled stop. The story stays a test of
  the world; it writes its files only into the test's own temporary
  directory, and they were copied into `golden/` once, by hand. Nothing
  writes `golden/baseline/` again.
- **Its test, focused:** every build loads the files into a fresh
  machine and starts the service on them with its production sets. The
  open patches through every landed patch (none at the baseline), the
  root restores, and once the start's wires are durable the test
  asserts, from literals written once from the story, never from
  today's encoders: the deployment's and the installation's ids; the
  person still signed in by the client's kept token; the account and
  its source; the project and its repository; the landed change's task
  ended and its landing recorded; the paused chat's turns read in the
  current version; the mid-run task's attempt crossed and its next one
  started; and the next task number above every kept one.
- **Its configuration, as a start reads it:** the kept
  `configuration.json` is the world's test configuration, whose small
  profile sets limits below a production deployment's. Where a later
  build raises a limit's floor (main's added brief section raised
  `brief.sections`' floor by one, to 8 when `tasks.charter_documents`
  is 0), or configures more of a thing than the override bounds (its
  default charters, against `tasks.charters` 9), the test raises that
  override to this build's floor or count in the copy it starts the
  service with, and the file stays as kept. Nothing
  else in the configuration is changed: a member the build refuses by
  name stays a red for the change that refused it
  (`../draft/upgrades.md`, 10).
- **A step per patch over it:** `golden/baseline/patched.txt` holds one
  line per patch the open applies, `<owner> <patch> <records> <commits>`
  in plan order, from the start's `Patched` events; the test fails
  unless the start's lines equal it. Empty at the baseline. A landing
  that adds a patch adds its line, its counts reviewed: the patch's
  step over the golden store, beside its own goldens (7).
- **Under 1 MB** in all; its paths a glob in `.config/fixture-readers.toml`.

## 7. The policy and the gate

- **A stored change brings its patch** in the landing that makes it
  (`../draft/upgrades.md`, 12): its author edits the schema, moves the
  family's version where a patch is needed, and runs `cargo run -p
  skein-codegen -- patch <crate>` (skein's `data/codec/command.md`, 2),
  gives the values it asks, reviews the patch's goldens, and adds the
  golden store's line (6). A change every old value survives byte for
  byte needs none while its kept text is not frozen (skein's
  `data/codec/command.md`, 2, step 3).
- **The gate's check is a focused test,** `fabrig-boundary`'s
  `tests/patches.rs`, which the gate runs for every change
  (`.config/gate/always.txt`): for each owner crate of 2's table, the
  patch command's `check` (skein's `data/codec/command.md`, 7) finds
  nothing and `current` passes. What is landed is read with git as the
  command's binary reads it, at `git merge-base HEAD origin/main`, or at
  `HEAD` where `origin/main` does not resolve, and handed to the library
  as `Landed::new`. The test also holds its list of owner crates equal
  to the crates under a tree's `crates/` that hold `schema/kept/`.
- **Its mutations,** over a scratch copy of `temper-protocol-records`:
  a field added to its current schema with no patch fails it
  (`Unpatched`), and its kept text edited after it landed fails it
  (`Changed`).
- **Beside it:** each owner crate's generated golden tests, each
  patch's old goldens through it (skein's `data/codec/command.md`, 3);
  the golden store's test (6); and the formats on disk, held by their
  goldens (jig's `tests/store/golden/`).
- **Named in `AGENTS.md`,** section 2, so every agent changing a schema
  knows it.

## 8. Stories through an upgrade

- **The fixture patch:** the real sets with temper's extended by one
  patch that changes no family, the one after temper's last (`patch 1`,
  `owner temper 1 after jig 0` before the catch-up; `patch 3`, `owner
  temper 3 after jig 2` after the devices' patch 2 and the overlays'
  jig patch 2), `mark fresh-windows`, its set's `fresh` that word (skein's
  `data/codec/patches.md`, 1: a patch may carry a mark and no block).
  A store made under the real sets is behind it, so a start under it
  plans, copies, patches (no record changed), closes every live window,
  switches and commits its wires. The one-process world keeps it,
  `temper_one_process_world::upgrade::FIXTURE_SETS`, for its upgrade
  stories (`../testing/README.md`, 2.5); no fixture names a family it
  changes.

## 9. The catch-up

The user's words of 2026-11-11: every stored layout `main` changed in place after
`bc23255eb7` comes into this design's landing as a patch from the
baseline, so a store a build at the baseline's layouts wrote, such as
dev.fabrig.ai's, upgrades to it.

- **Found by the merge:** once `main` is merged into the goal's branch,
  each family whose current text differs from its kept text, the
  baseline's, is caught up; the kept text is never refreshed to `main`'s,
  since `BASELINE` (5) and the golden store (6) are the baseline's.
- **On 2026-11-11, goals 63 and 187:**

| Family | Change on `main` | Every old record takes | Here |
|---|---|---|---|
| `jig-store-tasks` 9 | `TaskRecord.activation_open`, `TaskRecord.session`; activity lines `stood`, `session_ended` appended | `activation_open` false, `session` none | moves to 10 in jig's patch 1 |
| `jig-store-settings` 1 | `Model.cache_life_ms` inserted; `EndedBy`'s `standing` appended | `cache_life_ms` 0, the configuration's "none declared" | moves to 2 in jig's patch 1 |
| `jig-store-people` 14 | `EndedBy`'s `standing` appended; configured delegation executors narrowed | retained policy ceiling and each role retain only executors also granted by the configured deployment ceiling; all other fields kept | moves to 15 in jig's written patch 1 |
| `temper-store` 11 | `LandingRule.sets` appended | `sets` empty: a rule naming no set covers every change | moves to 12 in temper's patch 1; `STORE_VERSION` 12 |

- **The patches:** jig's patch 1, `owner jig 1 after smith 0 skein 0`,
  and temper's patch 1, `owner temper 1 after jig 1`, each `after`
  listing its upstreams' levels at this landing (skein's
  `data/codec/patches.md`, 1), each written by
  `skein-codegen patch <crate>` with the values above, their goldens
  reviewed. Another family `main` changed by the merge comes the same way:
  a value needed moves its version in its owner's patch; none needed
  refreshes its kept text.
- **Their step over the golden store:** a line each in `patched.txt`
  (6), `jig 1 …` then `temper 1 …`, the counts from the start and
  reviewed; the golden test's assertions hold under them.
- **What moves with them:** the binary's levels, jig 1 and temper 1, so
  `versions` prints `level jig 1` and `level temper 1`; the fixture patch
  (8) follows the devices' patch 2 at level 3, `owner temper 3 after
  jig 2`, following the overlays' jig patch 2.
- **Unchanged:** `BASELINE` (5), the baseline's versions and layouts, by
  which a store the baseline's build wrote reads as level 0 and is
  patched through both. A store a build of `main` after `bc23255eb7`
  wrote, at no level and other layouts, is refused as an older build's;
  such a deployment is made again.
- **A stored change landing on `main` later,** before this design lands,
  joins the catch-up at the closer's merge; after it lands, it brings
  its own patch, the gate (7) refusing it otherwise.

## 10. Open questions

- **The golden stores of later releases** (`../draft/upgrades.md`, 13),
  each made by its release's binary, join this one at each release.
- **A patch that walks the golden store's records alone,** decoded and
  compared record by record, in place of its counts' line. Default: the
  counts and the after-state's assertions.
