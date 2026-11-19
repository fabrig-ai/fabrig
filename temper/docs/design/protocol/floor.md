# The floor

Temper's side of the oldest store a temper build opens: dev.fabrig.ai's
store as F0 settled it, redeployed onto `3d1c717d37`. The mechanisms are
jig's (`jig/docs/design/protocol/store/records.md`, 3;
`jig/docs/design/protocol/store/opening.md`, 1.8 and 3) and skein's
(`skein/docs/design/data/codec/layouts.md`, 2). This document fixes what
temper hands them, its fixture and its policy.

## 1. In one page

- **Floor 1:** jig's `FLOOR`. A non-empty store with no floor row, or
  one below it, is refused before any registry plan, job or rehearsal
  decoding. `FLOOR_BRIDGE`, `3d1c717d379e`, names the build in its refusal.
- **Levels are explicit:** skein 0, smith 0, jig 6 and temper 14, each
  its `OwnerSet.level`, not a count inferred from an array. A store
  behind those levels is refused, not patched.
- **Registered families convert at read:** the registry holds their
  layouts and texts; defaults, former names and written jobs carry
  changes. No map patch, eager walk or seeding from code serves an
  older store.
- **Formats outside the registry are fixed at their versions** (2).
  The frames' patcher and the wires' patches keep their own rules.
- **One golden deployment at the floor** opens and serves with the
  production owner sets; a copy without its floor is refused (5).
- **Dev's build's image opens too** (5): the settled image as the build
  dev.fabrig.ai runs left it, opened and started by this build.
- **The deployed layouts are guarded** (6): a stored layout changed with
  no conversion and no version move fails a focused test against the
  texts of the builds whose layouts dev.fabrig.ai's store holds.
- **After dev runs a job, its code goes.** Dropping a whole older form
  moves the floor and names its bridge in the same landing (8).

## 2. What it fixes

The families declared by the production owner sets (3), and the
component's own formats:

| Family | Version | Owner | Where | Registry |
|---|---|---|---|---|
| `skein-oauth` | 3 | skein | `SavedToken` held in secrets' `Source` | outside |
| `smith-transcript` | 7 | smith | `Turn` held in transcript frames | outside |
| `jig-store-component` | 3 | jig | the component's own map rows | outside |
| `jig-store-openings` | 1 | jig | workers' openings | outside |
| `jig-store-registry` | 1 | jig | layouts, texts and done jobs | outside |
| `jig-store-core` | 6 | jig | core map rows; `TurnRecord` frame bodies | map value types registered; frames outside |
| `jig-store-tasks` | 13 | jig | tasks' map rows | registered |
| `jig-store-people` | 17 | jig | people's map rows | registered |
| `jig-store-notes` | 1 | jig | notes' map rows | registered |
| `jig-store-accounts` | 2 | jig | accounts' map rows | registered |
| `jig-store-settings` | 3 | jig | settings' map rows | registered |
| `jig-store-secrets` | 1 | jig | secrets' map rows | outside |
| `jig-accounts` | 1 | jig | `Moved` held in `Carried` | outside |
| `temper-store` | 25 | temper | application map rows | registered |
| `temper-secrets` | 1 | temper | application secrets' map rows | outside |

- **Registered means value types,** declared by each family's `types`
  and its crate's `schema/registered`. Values keep their layout ids;
  the read edge converts from the texts in the store's registry to
  the binary's layout (jig's `protocol/store/layouts.md`, 2 and 4).
  The core's frame body is not a registered map value: it holds
  smith's `Turn` and uses the frame patcher (jig's
  `protocol/store/transcripts.md`, 6).
- **The levels** stay jig 6 and temper 14. `STORE_VERSION` is 25,
  `temper-store`'s version in jig's format row; a registered schema
  change moves neither it nor an owner's level (`store.md`, 4).
- **The floor row** records a number and the build that wrote it.
  Jig's `FLOOR` is 1 and `FLOOR_BRIDGE` is `3d1c717d379e`. A fresh
  store's first commit writes the row, beside its format and
  installation; a reopen leaves it and its build unchanged. The
  upgrade's seal does not write it (jig's `protocol/store/records.md`, 3).
- **Beside the families:** skein-kv's frames, snapshots and payload
  logs, the transcripts' framing, the deployment's generations and
  small files keep their own formats (jig's `protocol/store/opening.md`;
  `README.md`, 7). This is a temper deployment's floor, not smith's
  local host store or a client's kept requests.

## 3. The owner sets

Temper hands jig's store `temper_engine_service::store::OWNER_SETS`,
declared in `temper-engine-service-vocabulary`, upstream first:

| Set | Declared by | Level | Families |
|---|---|---|---|
| `SKEIN_SET` | `jig-protocol-store` | 0 | `skein-oauth` |
| `SMITH_SET` | `jig-protocol-store` | 0 | `smith-transcript` |
| `JIG_SET` | `jig-protocol-accounts` | 6 | `JIG_STORE_SET`'s ten families, then `jig-accounts` |
| `OWNER_SET` | `temper-protocol-store` | 14 | `temper-store`, `temper-secrets` |

- **`OwnerSet`, `OwnerSets` and `KeptFamily`** are jig's data contract
  (`protocol/store/opening.md`, 3): current families and key ranges,
  registered value types, retired ranges, jobs and levels. A frames'
  family also carries its frame root, texts and patches; no set
  carries a map patch history. `JIG_STORE_SET` owns the store's metadata;
  accounts appends its held family without the store naming that
  consumer.
- **Keys** are whole map keys for upstream owners, relative to jig's
  application tag for temper's map rows, and relative to the secrets'
  prefix for temper's secrets. Held families have no key range:
  skein's token is held in `Source`, smith's turn in `TurnRecord` and
  jig-accounts' `Moved` in `Carried`. The core's frames name
  `TurnRecord`; smith's name `Turn` (jig's `protocol/store/opening.md`, 3).
- **Every engine shape hands the same sets** through
  `Store::owner_sets` before opening, beside its wires and build name.
  `Service::owner_sets`, called before `adopt_store`, is the test seam
  in `temper-service` and `temper-engine-service-component`: it replaces
  the sets for the next open. Production keeps the binary's sets.
- **`versions` prints the same levels:**
  `jig_protocol_store::levels(&OWNER_SETS)` supplies `level skein 0`,
  `level smith 0`, `level jig 6` and `level temper 14`
  (`binaries.md`, 2).
- **Admission checks the sets at open** (jig's
  `protocol/store/opening.md`, 3). The one-process test
  `every_golden_key_has_exactly_one_patch_family_or_a_reserved_component_row`
  additionally reads both maps of the golden deployment (5): each
  non-reserved key lies in exactly one family's range. It checks the
  reserved format, layouts, wires, floor, installation and log rows
  separately, outside every range; the registry's rows lie in
  `jig-store-registry`'s ranges.

## 4. A store below the floor

Jig's `protocol/store/opening.md`, 1.8 gives each refusal's words and
remedy; jig's `protocol/store/records.md`, 3 gives their order. An ahead
level is refused first. A missing level is `Older::Unleveled`; the
floor check follows, before the behind-level check and before reading
the registry.

- **`Older::Floor`**: no floor row or a number below `FLOOR`.
- **`Older::Behind`**: a level behind the binary's.
- **`Older::Unregistered`**: a store holding values but no registry.

These name `FLOOR_BRIDGE`: start that build on the store and let it
open, then this build. There is no upgrade, new generation or seeding
on the refused open; skein-kv's recovery has already started its empty
log segments, as on every open.

`Older::Directory` (before generations), `Older::Unleveled`, an
undecodable component row (`Older::Layout`) and another format's
version (`OpenRefusal::Format`) name the bridge too, but do not
promise a way through it: the bridge refuses those forms as well. The
operator starts afresh or restores a usable backup, following the
named refusal. Starting afresh moves `active`, `generations/` and
`transcripts/` aside together; nothing is deleted, but the new store
has no tasks, sign-ins, accounts or keys and every client signs in
again.

This applies to any other store, a laptop's box or a test deployment
left behind: upgrade through the named builds where they serve it, or
start afresh. Staging is made again at dev's build for each drill
(`../draft/store-floor.md`, 6).

## 5. The golden store at the floor

- **What:** `temper/tests/one-process/golden/floor/`, the whole
  deployment left by
  `golden::floor_making_story_stops_a_whole_deployment` with production
  crates. Its `README.md` names the making build and story. The person
  signs in, adds an account, adopts the fake repository and lands a
  change; a chat waits and a coding task is stopped mid-run by a
  drained signal stop. All identities and credentials are the fakes'.
- **Kept once:** the machine's `state/`, `worker/`, `client/` and
  `setup/` roots, `configuration.json`, and `modes.txt` for private modes
  and empty directories. The store holds floor 1, its registry and
  inert done-job rows. Nothing rewrites the directory; moving the
  floor replaces it with a fixture at the new floor (8).
- **Its focused open:** loads a fresh machine, starts with production
  sets, and compares `Event::Ran`'s job counts with `upgraded.txt`,
  empty at the floor. New layouts register in a copied generation;
  without a registration or job pending, no new generation is made.
  Once the start's wires are durable it checks
  literal ids, the kept person and account, repository and landing,
  chat turns, mid-run task and next number, then serves the person
  through the kept client's token. A landing adding a job reviews its
  counts' line and after-state assertions.
- **Its refusal:** a copy with the floor row erased refuses
  `OlderBuild { why: Floor }`, naming `FLOOR_BRIDGE`, before decoding.
  `active` and all existing paths and bytes stay unchanged; only
  recovery's empty log segments are added (skein's `data/kv.md`, 3).
- **Its rehearsal:** `upgrade --check` decodes every record and
  restores with no job pending; a broken record is named
  `Undecodable`; the floorless copy refuses before decoding. Each
  source stays unchanged and its scratch is removed.
- **Its configuration:** the world's kept test limits are raised in
  the loaded copy only if a build's limits require it; the kept file
  stays as made. Its retired `limits.profile` `small` is dropped from
  the loaded copy, which takes small's figures from test fixtures with
  the kept overrides over them (`../draft/config.md`, 5). No other
  configuration member is changed to evade a refusal. Its paths are listed in `.config/fixture-readers.toml`.
- **Dev's build's image:** `temper/tests/one-process/golden/dev/`, the
  settled image (7) as the build dev.fabrig.ai runs, `13efa77461`, made
  and kept it in its own tree, copied from that commit byte for byte and
  never written again. Its `README.md` names the build and the copy.
  It is the store dev's own codec wrote, at that build's layouts, with
  ended tasks whose history is in their task logs. Its focused open
  loads a fresh machine from its roots, as the floor's does, and starts
  this build with production sets: the open registers this build's
  layouts in a new generation, the start's restore pages and its census
  pass, and each ended task's history reads back through the kept
  client's token. The source generation's files stay unchanged, and a
  second start writes no generation and reads the same. Its
  configuration is loaded as the floor's is. When dev.fabrig.ai moves to
  a newer build, the image is replaced by that build's, in the landing
  that moves the guard's builds (6).

## 6. The policy and the gate

- **A registered family's change brings no patch and moves no
  version.** An added field takes a default (`= value`), a renamed part
  its former name (`was`); a change no plan derives is its owner's
  written job (jig's `protocol/store/layouts.md`;
  `jig/docs/design/protocol/store/jobs.md`).
  `temper-protocol-store/schema/store-v25.schema` holds temper's current
  text, not older map versions. Records drift and refusal of unpinned
  constructs check it; tests build their values, with the bytes pinned
  by skein's `every` (`skein/docs/design/data/codec/README.md`, 6).
- **Formats outside the registry are fixed at their versions.** A
  landing changing one designs how dev.fabrig.ai's store gets there:
  an executor, conversion at read, or registration (`AGENTS.md`, 2).
  The frame patcher stays (jig's `protocol/store/transcripts.md`, 6).
- **`tests/boundary/tests/patches.rs` holds the wires and the formats
  outside the registry.** Its owner inventory covers the crates with
  `schema/kept/`; `patch_modules_are_current` checks their generated
  modules, and mutations pin an added field without a patch and a
  landed kept text edited. The stored-change comparison reads landed
  files at the merge base with `origin/main` (or `HEAD` without it;
  during a landing merge, `MERGE_HEAD`). It is not a registered-family
  patch gate. Each format's byte tests, the schemas' records drift and
  pinned constructs, built-value round trips, registry tests and the
  golden deployment's outcomes stand beside it.
- **The deployed layouts are guarded.** A focused test holds every
  family the production owner sets declare (2), and the client's kept
  records (jig's `client/domain/view-state.md`), against the texts of
  each build whose layouts dev.fabrig.ai's store holds:
  - **The builds:** `3d1c717d379e`, the floor's bridge, at whose layouts
    F0 settled every value (`../draft/store-floor.md`, 2), and
    `13efa77461`, the build dev.fabrig.ai runs. Each build's texts are
    kept verbatim, as `git show` gives them at that commit, under
    `temper/tests/codecs/deployed/<build>/`, one file per family at its
    path's file name, with a `README.md` naming each build in full and
    why it is held. Nothing rewrites a kept text.
  - **A registered family:** each value type its family declares
    (jig's `protocol/store/layouts.md`, 2), at a build whose text holds
    its root and variant, has its layout planned to the binary's from
    that build's texts and their held texts, as the open's check plans
    a registry row (jig's `protocol/store/layouts.md`, 5.3) with every
    layout taken as held. It passes where a plan derives, or, for a
    value type no task log holds (jig's `protocol/store/logs.md`, 8),
    where a written job of the binary converts that layout (jig's
    `protocol/store/layouts.md`, 6).
  - **A family outside the registry,** and the client's records: at the
    same version, every declaration both texts hold keeps its layout's
    fingerprint (skein's `data/codec/README.md`, 9.4), so names and
    comments may change and a declaration no row is read by may go; a
    family whose version moved passes where its patch from the kept
    version is in the tree (skein's `data/codec/command.md`, 2).
  - **Inventory:** each family the sets declare is kept at each build,
    or is new since it; a family that was stored at a build and is
    neither in its directory nor new fails, so a new stored family joins
    the guard.
  - **Its failure** names the build, the family, the value type or
    declaration, and the rule: a default (`= value`), a former name
    (`was`), a written job, or a version move with its patch.
  - **The builds move** by a landing after dev.fabrig.ai is redeployed:
    the new build's texts are added beside the others, its image
    replaces dev's (5), and a build's texts go only once dev's registry
    holds no layout they give (jig's `protocol/store/layouts.md`, 5.4)
    or a floor move settled them (8).
- **The wires' rule is unchanged** (`wires.md`, 3): a layout change
  moves its wire version, keeps the old text and brings its patch and
  reverse or breaking declaration. A store's floor promises no wire
  compatibility of its own.

## 7. Stories through an upgrade

- **The fixture is a registry change and a job, not a patch.** In
  `temper/tests/one-process/src/upgrade.rs`, `CURRENT_SETS` is the
  production `OWNER_SETS`. `first_sets()` keeps those levels and
  families, omits `deleted-repository-forge-rows` and removes the unused
  `ReceiptRow.report` field from `temper-store`'s text. The stories
  write no receipts. A restart with `CURRENT_SETS` registers the
  binary's layout, whose field defaults to `unset`, and runs the job
  once in a copy before restoring, switching and committing its wires.
  It closes no window by a mark (`../testing/README.md`, 2.5).
- **The settled image** in `tests/one-process/golden/settled/` is read by
  the one-process world. It holds the four machine roots, modes and
  configuration, the fake forge's image, issued fake tokens and users,
  the stop wall, mid-run task and attempt, terminal answer and ledger.
  The loader makes a cold referee store and rebuilds its committed
  prefix from the image. Unlike the floor fixture it is the current
  story's image: `the_settled_image_matches_the_story` compares every
  file and rewrites it only with `TEMPER_TEST_UPGRADE_IMAGE=1`.
- **Focused stories** cover the settled upgrade, a drawn power loss,
  an unsettled stop, rehearsal and going back, and a worker's opening
  kept through the stop. Fuzzy sweeps moments and named cuts at the
  copy's first write, the seal commit, switch rename and wires'
  commit. The referee compares the kept history across the upgrade,
  permitting the job's declared changes, restores and wires; no forge
  write is made twice. Jig's store world checks the store's own cuts
  (jig's `domain/testing.md`, 9).
- **Reports come from jobs:** the shell renders `Event::Ran` as
  `ran OWNER JOB: ROWS rows in COMMITS commits` with declared counts.
  The check's job line ends `time unavailable`; it separately prints
  the whole rehearsal/tool time (`binaries.md`, 1 and 2).

## 8. The floor moves

After dev.fabrig.ai runs a build's jobs, a small landing drops their
code. A retired job's name is never reused; its done row stays inert
(jig's `protocol/store/jobs.md`, 1). No code serves a store older than
dev's (`AGENTS.md`, 2).

The floor's number moves only when a landing drops a whole older
form. That landing also names the bridge build and designs how its
upgrade writes the new floor row, replaces the golden floor fixture,
and keeps the named refusals true. Another store left behind uses
those builds or starts afresh (`../draft/store-floor.md`, 6).

## 9. Open questions

- **Builds between the floor's bridge and dev's:** a build dev.fabrig.ai
  ran between `3d1c717d379e` and `13efa77461` may have left values at
  layouts of its own, which the guard (6) does not keep. The redeploy's
  check over a checkpoint of dev's store refuses such a layout by name
  (jig's `protocol/store/layouts.md`, 5.3); its registry's texts could
  be added to the guard's builds from that store.
