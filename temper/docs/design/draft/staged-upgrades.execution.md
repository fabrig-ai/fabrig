# staged-upgrades: execution

2026-11-11. How [staged-upgrades.md](staged-upgrades.md) gets carried out. It designs nothing: each milestone writes its own design from the draft as it runs. Goals use the focused and fuzzy suites alone; the drills run after landing, as the draft's sections 6 and 7 say.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | State |
|---|---|---|---|---|
| U1, the catch-up: a patch from the baseline for each stored layout `main` changed in place, each with its line over the golden store; and `upgrade --check` decoding every record of every family | 3, 5 | goal 648, or the goal after it (temper/migration) | goal 648's sets, gate and `upgrade --check` | asked of goal 648 on 2026-11-11 |
| U2, staging.fabrig.ai made again at dev.fabrig.ai's build, with its workload | 6 | the deployments' operator; the workload temper/migration | a worker and an account (agreed 2026-11-11) | not started |
| U3, the rehearsal on a backup of dev.fabrig.ai | 5 | the deployments' operator for the backup; temper/migration | U1 landed | not started |
| U4, the drill script, run on staging | 6 | temper/migration | U2, U3 | not started |
| U5, dev.fabrig.ai upgraded | 7 | temper/migration | U4 passed | not started |

## 2. Order

- **U1 first:** nothing upgrades before the catch-up lands.
- **U2 may run before U1 lands:** staging at dev.fabrig.ai's build with its workload, ready for the drill.
- **U3 and U4 on the same build,** the one U5 deploys.
- **Section 8 of the draft** is the routine after U5, no milestone's.
