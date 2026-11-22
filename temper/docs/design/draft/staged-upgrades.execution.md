# staged-upgrades: execution

2026-11-22. How [staged-upgrades.md](staged-upgrades.md) gets carried out. It designs nothing: each milestone writes its own design from the draft as it runs. Goals use the focused and fuzzy suites alone; the drills run after landing, as the draft's sections 6 and 7 say.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | State |
|---|---|---|---|---|
| U1, the catch-up and full decode | 3, 5 | jig and temper | the owner sets and `upgrade --check` | landed, goal 648; its catch-up patches removed by store-floor's F2; full decode stays, using registry texts and current formats |
| U2, staging.fabrig.ai made again at dev.fabrig.ai's build for each drill, with its workload | 6 | the deployments' operator; the workload temper/migration | a worker and an account (agreed 2026-11-11) | not started |
| U3, the rehearsal on a backup of dev.fabrig.ai | 5 | the deployments' operator for the backup; temper/migration | U1 | rehearsal tooling landed, goal 648: `upgrade --check` over a checkpoint taken while the old build serves (`../protocol/binaries.md`, section 2); running it is the deployments' operator's, outside this repository (the user, 2026-11-20); a rehearsal on the chosen build is required for each drill |
| U4, the drill script, run on staging | 6 | the deployments' operator, outside this repository | U2, U3 | the operator's (the user, 2026-11-20): goal 2487's redeploy script and its worker/task referee left fabrig (`staged-upgrades.status.md`); the full workload-and-restore drill is not started |
| U5, dev.fabrig.ai upgraded | 7 | temper/migration | U4 passed for the staged path | landed; dev.fabrig.ai redeployed with S7 and F0 onto `3d1c717d379e` (11-16; `store-floor.execution.md`, 1); U4's staging drill remains outstanding |

## 2. Order

- **U1 is landed:** there is no catch-up left to wait for or build. Registered changes use defaults, former names or jobs (`staged-upgrades.md`, 3).
- **U2 is made again for each drill:** staging starts at dev.fabrig.ai's build, with its own sign-ins and workload, never a copy of dev's store and never a store below the floor (`store-floor.md`, 6).
- **U3 and U4 on the same build,** the one U5 deploys. The staging drill's outstanding workload and restore are not proved by a redeploy of dev alone.
- **Section 8 of the draft** is the routine after U5, no milestone's: rehearse, make staging again and drill, then dev follows. A landing after dev runs jobs removes their code; dropping a whole older form moves the floor and its bridge together.
