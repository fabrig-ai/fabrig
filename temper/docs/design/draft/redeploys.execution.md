# redeploys: execution

2026-11-12. How [redeploys.md](redeploys.md) gets carried out. It designs nothing: each milestone writes its own design from the draft as it runs. Goals use the focused and fuzzy suites alone; the drills on the deployments run after landing, as the draft's 7.5 says.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its drill after landing | State |
|---|---|---|---|---|---|
| R1, versions with layouts, each build's floor its current: every wire of 3.1 and the clients' move their version at the landing that changes their layout, with kept texts and the gate's check; ranges, fingerprints and terms at the opening; a mismatch refused by name and costing no try; `versions` printing them | 3.1, 3.2, 3.4, 8; 3.3's interim | temper/migration | the patch command and its check (goal 648) | a redeploy across a wire change, its old workers refused at the opening by name, no goal held | built by goal 1705 (`../protocol/wires.md`; jig's `protocol/channel/wires.md`); its drill not yet run |
| R2, the drain: agents pinned at the worker's start and going with it; each stopped at the end of its turn within the drain bound; jobs drained before their effect interrupted; the stop bound enforced over saves and teardown | 4.4, 6.1, 6.2 | temper/migration | — | a worker restarted mid-run: no try spent, the run resumed | not started |
| R3, preparations a few at a time, over warm stores | 6.3, 6.4 | temper/migration | goal 1492 | a worker restarted: no preparation timed out | not started |
| R4, bridges: reverses, floors and the breaking declaration in the command; translation in the channel's components; the bridges of 4.2, git identities, host tools' contracts and jobs' requests and results among them; placement by agreed versions; messages to running attempts whole | 3.3, 4 | temper/migration | R1 | the engine redeployed alone across a wire change, its workers kept | not started |
| R5, the engine alone: a settled stop that places nothing; the graces sized from the enforced stop bound; workers' openings kept; `upgrade --check` given its verdict on which workers may stay, at R1's floor and at R4's | 5 | temper/migration | R1, R2; goal 648's `upgrade --check` (main `182801bf82`), the store's rehearsal, whose verdict on the workers compares the stored wire numbers | the engine redeployed alone, its runs adopted | not started |
| R6, the redeploy script and its referee: the engine alone by default, workers in turn, every part together across a breaking change | 7 | temper/migration, as the deployments' operator | R2, R3, R5; R4 for the engine alone across a wire change | every redeploy from then on | not started |
| R7, the policy for infrastructure failures | 9 | temper/migration | — | none of its own | not started |

## 2. Order

- **R1 first:** until versions move with layouts, nothing tells two builds apart. Each build then speaks its current version alone, and a wire change makes its redeploy one of every part together.
- **R2, R3 and R7** are independent of R1 and of each other.
- **R5 after R1 and R2:** with R3 too, the redeploy (R6) restarts the engine alone whenever no wire moved, and drains each worker it restarts.
- **R4 after R1:** then the engine goes alone across a wire change too, and the check judges by floors.
- **Until R6,** a redeploy restarts every part together, drained once R2 lands (the draft's 7.3).
