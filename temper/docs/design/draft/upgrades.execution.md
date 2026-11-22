# upgrades: execution

2026-11-11. How [upgrades.md](upgrades.md)'s sections 3 to 13 get
built, with dev.fabrig.ai's settled store as the floor (4.1;
`store-floor.md`, section 3). It designs nothing: each milestone's
design is the documents it names, written before its changes.

## 1. In one page

- **Twelve milestones,** M1 to M12 (section 3), landed. Where
  store-floor removed their store-history work, the state says so;
  it is not work to build again.
- **M1 to M3:** skein's run-time schemas and patch tooling, jig's
  generations and upgrade at open, temper's settled stop. The generic
  codec, frames' patcher, generations and settled stop stay; the map
  patch walk and its admission do not.
- **M4 to M12:** the level rule, fixture, owner sets and gate;
  rehearsing and going back; temper's commands; the upgrade story
  and power loss; the catch-up; every record decoded. Their current
  forms use the floor, registry and jobs, not the baseline's patches
  (`store-floor.md`, sections 4 and 5).
- **No walk:** the user's request of 2026-11-11 set none; each
  milestone is held by focused and fuzzy tests alone.
- **Out:** the process tier's drill on a real Forgejo (`upgrades.md`,
  13), which follows; the engine alone (`upgrades.md`, section 9) past
  what the story needs. Golden stores of later releases promised to
  upgrade from below the floor are dropped by store-floor.
- **Settled by rule, nothing to build now:** the configuration's
  previous release (10), until a release removes a member; the small
  files and the forge's markers (10, 11), each read in every version it
  has, one today (`../protocol/README.md`, 7).
- The draft's status keeps only the gaps that stand. This file goes
  with the draft (`DESIGN.md`, section 3).

## 2. How a milestone runs

- **Its changes** build into the goal's branch, each with its focused
  tests, and the goal lands on `main` whole.
- **The live store opens at every landing:** the floor and explicit
  levels are checked before registry planning or jobs. A store below
  the floor is refused by name, with bridge build `3d1c717d379e`
  (`store-floor.md`, section 3, with its status's departures).
- **The fixture is at the floor:** it opens and serves; the older
  store's test holds the named refusal, not an upgrade path (section 3,
  M5). Neither keeps a table of map patches.

## 3. The milestones

| | Milestone | Executes (`upgrades.md`) | Needs | State |
|---|---|---|---|---|
| M1 | skein-codec's run-time schemas, fingerprints, the diff, patches and the command | 4.3, 4.5, 5 | | landed before 2026-11-11; generic machinery and wire tooling stay |
| M2 | jig's levels and upgrade at open: generations, plan, copy, switch, power loss, transcripts patched as read and the settled start | 3.1, 4.2, 4.4, 6, 7.2, 8 | M1 | landed before 2026-11-11; its map patch walk, fresh-window marks and effect-family admission removed by store-floor's F2 |
| M3 | the settled stop, temper's root and connector | 8 | | landed before 2026-11-11 |
| M4 | the baseline's level-0 recognition | 4.1, 4.2 | M2 | landed; its recognizer removed by store-floor's F2, the floor check supplied by F1 |
| M5 | the baseline's golden store and success test | 13 | | landed; its success story removed by store-floor's F0, with the floor's fixture and named refusal supplied by F1 |
| M6 | kept texts, real owner sets and `temper versions` | 4.2, 4.3, 5 | M4, M5 | landed; its old registered texts removed and owner hand-over made current-only by store-floor's F2 |
| M7 | the policy and gate | 12 | M6 | landed; its registered-family patch policy and mutations removed by store-floor's F2, the wire and fixed-format rule stays |
| M8 | rehearsing and going back in jig's store | 3.1, 3.2 | M2 | landed |
| M9 | `temper upgrade --check` and `--back`, the rehearsing root and start's lines | 3.1, 3.2, 9 | M6, M8 | landed; its map patch lines removed by store-floor's F2, job lines stay |
| M10 | a story through an upgrade, and power loss in it | 13 | M6 | landed; its patch fixture rewritten as a registry change and job by store-floor's F2 |
| M11 | the catch-up as jig's and temper's patch 1 | 4.1, 12, 13 | M6, M7 | landed; its patches removed by store-floor's F2 |
| M12 | every record decoded in a rehearsal, the first that does not named | 3.1 | M8, M9, M11 | landed; its decoding at patch levels removed by store-floor's F2, registry and current-format decoding stay |

### M4. The level rule and floor

- **Design:** jig's `protocol/store/records.md`, 3; `store-floor.md`,
  section 3, with its status's departures.
- **Current scope:** explicit levels jig 6 and temper 14, the floor
  check before any planning, the named older-store refusals. The
  baseline recognizer and level-0 translation are removed, not pending.
- **Tests:** comparisons at open, a missing or lower floor refused
  with bridge build `3d1c717d379e`, and a fresh store reopening.

### M5. The floor's fixture

- **Design:** `store-floor.md`, 4.4; jig's `protocol/store/opening.md`,
  1.8.
- **Current scope:** the one-process world's fixture at the floor
  opens and serves; the older store is refused by name, naming the
  bridge. No map patch table or older-store success story stays.

### M6. Current texts and owner sets

- **Design:** jig's `protocol/store/opening.md`, 3, and
  `protocol/store/layouts.md`, 1 to 5; `store-floor.md`, 4.1 and 4.2.
- **Current scope:** current families and value types, held-family
  links, frame roots and jobs handed by the shell; levels held
  explicitly and printed by `temper versions`. Registered texts of
  layouts held by values live in the store, not a kept directory.
- **Tests:** first commit and reopen at the floor; family ranges;
  a registry change and job without a level move. Frames' fixture
  owners keep their texts and patches for the read-time patcher.

### M7. The policy and the gate

- **Design:** `AGENTS.md`, 2; `store-floor.md`, 4.4.
- **Current scope:** registered defaults, former names and jobs; the
  boundary's patches check for wires and formats outside the registry.
  No deleted map patch or edited old registered text is a mutation
  the gate needs to hold.

### M8. Rehearsing and going back

- **Design:** jig's `protocol/store/opening.md`, 5.
- **Builds:** `Request::Rehearse`, `Request::Back`, `Request::Close`
  and their events in jig's store, with its store world's stories.

### M9. temper's commands

- **Design:** `../protocol/binaries.md` (`upgrade`, job lines);
  `../domain/root-restart.md`, 1.1.
- **Builds:** both binaries' `upgrade --check` and `--back`, the
  rehearsing root, the job lines; the one-process world's rehearsal
  and going-back stories.

### M10. A story through an upgrade

- **Design:** `../testing/tiers.md`, 1.5; `store-floor.md`, 4.4.
- **Current scope:** the story on a registry change and job, its
  referee, the unsettled stop, and a fuzzy sweep of power lost during
  the upgrade's start. Levels and versions stay.

### M11. The catch-up

Landed; its patches, old texts, goldens and patch-count table are
removed by store-floor's F2 (`store-floor.md`, 4.1 and 4.4). No
catch-up work remains. Changes use the registry and jobs (M6).

### M12. Every record decoded

- **Design:** jig's `protocol/store/opening.md`, 5 ("Every record
  decoded"); `../protocol/binaries.md` (`upgrade --check`).
- **Current scope:** the rehearsal decodes every registered value
  from its layout's text in the store's registry, other records from
  their current formats and frames from the frames' texts. The floor
  check precedes decoding. It names the first record that does not
  decode, with its key or transcript and offset.
- **Tests:** a broken record found in jig's store world and the
  one-process world's rehearsal, not decoding an older store at
  pending patch levels.
