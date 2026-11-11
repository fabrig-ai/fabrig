# upgrades: execution

2026-11-11. How [upgrades.md](upgrades.md)'s sections 3 to 13 get
built, its baseline moved to the store `main` wrote at `bc23255eb7`
(4.1; `../protocol/baseline.md`). It designs nothing: each milestone's
design is the documents it names, written before its changes.

## 1. In one page

- **Ten milestones,** M1 to M10 (section 3), each landing on its own,
  leaving `main` green and reading the live store, the smallest useful
  first.
- **M1 to M3 landed before this file:** skein's run-time schemas and
  patch tooling, jig's levels and patching at open, temper's settled
  stop.
- **M4 to M10 are the goal of 2026-11-11:** the baseline's level rule,
  its golden store, its kept texts and real sets, the gate, jig's
  rehearsal and going back, temper's commands, and a story through an
  upgrade with power loss.
- **No walk:** the user's request of 2026-11-11 set none; each
  milestone is held by focused and fuzzy tests alone.
- **Out:** the process tier's drill on a real Forgejo (`upgrades.md`,
  13), which follows; the engine alone (`upgrades.md`, section 9) past
  what the story needs; the
  golden stores of later releases.
- **Settled by rule, nothing to build now:** the configuration's
  previous release (10), until a release removes a member; the small
  files and the forge's markers (10, 11), each read in every version it
  has, one today (`../protocol/README.md`, 7).
- When M10 lands, the draft's status keeps only its departures, and
  this file goes with the draft (`DESIGN.md`, section 3).

## 2. How a milestone runs

- **Its changes** build into the goal's branch, each with its focused
  tests, and the goal lands on `main` whole.
- **The live store opens at every landing:** M4's rule lands before M6
  hands real sets, so a store the baseline's build wrote opens under
  every milestone's build, at level none before M6 and at level 0 after.
- **The golden store is the baseline's code's:** M5 makes it before
  any change touches what a start writes, and M6 waits for it.

## 3. The milestones

| | Milestone | Executes (`upgrades.md`) | Needs | State |
|---|---|---|---|---|
| M1 | skein-codec's run-time schemas, fingerprints, the diff, patches and the command | 4.3, 4.5, 5 | | landed before 2026-11-11 |
| M2 | jig's levels in the format row and patching at open: generations, the plan, the copy, the switch, power loss, transcripts patched as read, fresh windows, effects in flight | 3.1, 4.2, 4.4, 6, 7.2, 8 | M1 | landed before 2026-11-11 |
| M3 | the settled stop, temper's root and connector | 8 | | landed before 2026-11-11 |
| M4 | a store at the baseline opens at level 0 | 4.1, 4.2 | M2 | in flight: change 691 |
| M5 | the golden store at the baseline and its test | 13 | | in flight: change 693 |
| M6 | the kept texts and the real sets: temper's families and every owner's at level 0, `temper versions` | 4.2, 4.3, 5 | M4, M5 | in flight: changes 692 and 706 |
| M7 | the policy and the gate | 12 | M6's kept texts | in flight: change 707 |
| M8 | rehearsing and going back in jig's store | 3.1, 3.2 | M2 | in flight: change 670 |
| M9 | `temper upgrade --check` and `--back`, the rehearsing root, a patching start's lines | 3.1, 3.2, 9's tool | M6, M8 | in flight: change 709 |
| M10 | a story through an upgrade, and power loss in it | 13 | M6 | in flight: change 708 |

### M4. A store at the baseline

- **Design:** jig's `protocol/store/records.md`, 3 (`BASELINE`, the
  comparison); `../protocol/baseline.md`, 5.
- **Builds:** the baseline's constant; a levels-none row at it read as
  level 0 where the binary sets a level; layouts compared only where
  both levels are none; the refusals' words.
- **Tests:** the comparison's table over rows built from `BASELINE`;
  one store-world story patched from level 0.

### M5. The golden store

- **Design:** `../protocol/baseline.md`, 6.
- **Builds:** the one-process world's golden store, made once by the
  baseline's code; its test, every build starting on it; its table of
  patched lines, empty.

### M6. Kept texts and real sets

- **Design:** `../protocol/baseline.md`, 3, 4 and 8.
- **Builds:** six owner crates' kept texts (change 692); the four
  owners' sets handed by `temper-service`, its test seam, `versions`
  from them, the fixture sets (change 706).
- **Tests:** a first start at level 0; the golden store at level 0;
  every key of the golden store in one family's range; the fixture
  patch over a made store.

### M7. The policy and the gate

- **Design:** `../protocol/baseline.md`, 7.
- **Builds:** `fabrig-boundary`'s `patches` test, its mutations, the
  gate's item and `AGENTS.md`'s line.

### M8. Rehearsing and going back

- **Design:** jig's `protocol/store/opening.md`, 5.
- **Builds:** `Request::Rehearse`, `Request::Back`, `Request::Close`
  and their events in jig's store, with its store world's stories.

### M9. temper's commands

- **Design:** `../protocol/binaries.md` (`upgrade`, a patching start's
  lines); `../domain/root.md`, 8.1.
- **Builds:** both binaries' `upgrade --check` and `--back`, the
  rehearsing root, the lines; the one-process world's rehearsal and
  going-back stories.

### M10. A story through an upgrade

- **Design:** `../testing/README.md`, 2.5.
- **Builds:** the story on the fixture sets, its referee, the unsettled
  stop, and a fuzzy sweep of power lost during the upgrade's start.
