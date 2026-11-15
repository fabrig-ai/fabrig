# store-floor: execution

2026-11-15. How [store-floor.md](store-floor.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Each is a goal the user approves. Plans use the focused and fuzzy suites alone; walks come after landing.

## 1. The milestones

| Milestone | Draft parts | Needs | Its walk | State |
|---|---|---|---|---|
| F1, the floor and the done jobs: each owner's floor (explicit levels, the registry seeded, the done rows, the bridge build), the open's check before any plan and the rehearsal's with it, its refusal words; a non-empty store with no registry refused, a census layout id the registry lacks refused; the seven jobs and the stored forms only they read removed (tasks' milestones, whole temper change records, the jobs' readers of whole live tasks and projections), their names retired | 2, 3.3, 6 | store-shape's S7 on main, since F1 changes the same job sets and crates | after landing, on a staging deployment made at dev.fabrig.ai's build: it opens and serves, and the old baseline store is refused with the bridge build's name | not started |
| F2, the map patches' history and the walk: the patch files, old texts and expected bytes, temper's `golden/v11` to `golden/v24`, the conversions and contexts, the patches package, the shell's site routes, the eager walk and S2's rewrite, seeding from code, the baseline recognizer, the old levels' format logic and the rehearsal's decoding at patch levels; the upgrade's transaction (runner, seal, switch, check) kept and moved out of `patching/` | 3.1, 3.2 | F1 | after landing, on a staging deployment made at dev.fabrig.ai's build: an upgrade bringing a registry change and a job opens, and `temper-engine upgrade --check` over it passes | not started |
| F3, fixtures, rules and words: the one-process baseline replaced by a fixture at the floor and a refusal test, the upgrade fixture rewritten as a registry change and a job, the test patch crates and suites, `AGENTS.md`'s rule, skein's `data/codec/command.md`, `tests/boundary/tests/patches.rs`, `.config/fixture-readers.toml`, and the designs and drafts that promise older stores | 3.4 | F2 | none: its suites are its check | not started |
| F4, the floor after S7: once dev.fabrig.ai has run S7's `task-logs`, that job and the map's history readers go, and the floor's done rows and bridge move to S7's build | 5 | F3; dev.fabrig.ai redeployed with S7 | after landing, on a staging deployment made at dev.fabrig.ai's build: it opens, and a store from before S7 is refused with S7's build named | not started |

## 2. Order

F1 to F3 in that order, from S7's landing on: each changes the store crates S7 changes. F2 and F3 may land as one if the fixtures can't be parted from the history. checkpoints' C1 (`checkpoints.execution.md`) follows F2, so that it is built on the smaller store crate. After F4, each redeploy of dev.fabrig.ai that runs jobs is followed by a landing like F4 (`store-floor.md`, section 5).
