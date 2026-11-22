# failure-worlds: execution

2026-11-20. How [failure-worlds.md](failure-worlds.md) gets built: one milestone, its goals run in parallel wherever their parts don't depend on one another. It designs nothing: each goal writes its design from the draft as it runs, and lands whole into the milestone's branch. Plans use the focused and fuzzy suites alone.

## 1. The milestone and its goals

One milestone, *failure worlds*, of ten goals. It lands into main whole, once its last goal has landed, so that what it frees in the budget and what it adds arrive together (`failure-worlds.md`, section 8).

| Goal | Draft parts | Trees | Needs | State |
|---|---|---|---|---|
| F0, the harness's interfaces and the budget plan: the inspection adapter's form in skein's world harness, and each component's adapter's shape; the fault ledger, with its causal identities, intervals and counters, and the assertion that each configured fault fell; the seed's independent streams; a sweep's named exclusions; and the budget plan, per tree and layer, from the suites' own reports, with the moves between layers it proposes | 3's kinds, 3.3's ledger, 5's streams, 6, 8 | skein, temper, jig | the reconciling start's first milestone landed | landed (`skein/docs/design/testing/inspection.md`; `skein/docs/design/testing/sweeps.md`; `../testing/checks.md`, 6); its budget plan's moves wait for the user's word |
| F1, jig's checks: jig's adapter over its tasks, waits and tables; every wait has a wake, and every resource its lifetime rule, in jig's core and engine-clients worlds; jig's open scenarios (P43, P117, S15, P77, P76, P93, P98, P82's residual) | 3.1, 3.2 for jig; 7's jig rows | jig | F0 | not started |
| F2, the engine's checks: the engine's adapter over its landings, change steps and tables; every wait has a wake, every resource its lifetime rule; finite goals end and others keep going, in the referee; the engine's open scenarios (P124's wait for room with a bound, S7, P88, P90, P67's residual, P37, P53, P85, P113, P120, P58's residual, P63's residual) | 3.1, 3.2, 3.4 for the engine; 7's engine rows | temper | F0 | not started |
| F3, the worker's checks and faults: the worker's adapter over its workspaces, scratch and disk; a workspace fetch timing out partway, the fake disk filling, an unremovable trash entry, a busy placement; P109's scenario | 3.1, 3.2 for the worker; 4.3's worker faults; 7's P109 row | temper, skein | F0 | landed (`../testing/worker.md`; `../testing/fakes.md`, 3); it changed no skein code, and skein's fake machine keeps no capacity (`../testing/worker.md`, 7) |
| F4, the forge's faults and gate blame: busy and slow, lagging views, past its bounds, refusing, and CI's lost runners, missing reports and shared runner, in skein's fake forge and recorded in the ledger; stale, reordered and rewritten state; blame and tries for gates, with the two controls; S5's and S2's scenarios | 4.1, 4.5; 3.3 for gates; 7's S5 and S2 rows | skein, temper | F0 | not started |
| F5, the providers' faults and run blame: the shared rate window without `retry-after`, spent credits, the synthetic billing envelope, unserved models, expired sign-ins, hanging requests and endpoint-bound windows; an untooled program and a check's clock in smith's worlds; blame and tries for runs; the providers' open scenarios (P26, P72, P127, S1, P126, P71) | 4.2; 4.3's smith faults; 3.3 for runs; 7's provider rows | skein, smith, temper | F0 | not started |
| F6, the settings and the store: the settings tests and their generated mutations; jig's isolated restore-and-check over a committed snapshot, and temper's connector checks on it, run after every commit or sampled with its coverage reported; P128's, P59's and P81's residual scenarios | 3.6, 3.7; 7's settings rows | jig, temper | F0 | not started |
| F7, room in the budget: retire the demonstrated duplicates, and move the real-loop and one-process stories' assertions down to the tiers that can show them, as F0's plan and the user's word on its moves say | 8 | all trees | F0, and the user's word on F0's moves | not started |
| F8, the restart faults: the engine restarted cold at any iteration; a settled stop, a stop in each phase of a start, a second signal; the forge down through a start; the tables full at a restart as fixtures; a worker drained, signalled or crashed mid-turn; the engine restarted while workers are mid-turn; an older layout from fixtures the previous release writes | 4.8 | temper, jig | F0 | not started |
| F9, the workloads and the sweeps: a random workload in each root's world; the composed workload; people's actions and runs that park, resume and move, at any step; a restart changes no outcome it shouldn't; one sweep per family and one mixing every family, each within the room F7 made | 4.4, 4.6, 4.7, 3.5, 5 | temper, jig | F1 to F8 | not started |

## 2. Order

- **After the reconciling start's first milestone,** S1, a start that reads and doesn't write: its failure is live (2026-11-20).
- **F0 first, alone:** it fixes the interfaces every later goal writes to (the adapters, the ledger, the streams, the exclusions), and the budget plan. Once it lands, the user decides the budget moves it proposes.
- **Then F1 to F8 side by side.** Each writes its own adapter, faults and scenarios in its own tree's worlds. Two pairs touch the same crates, and their shared files have one owner each:
  - F4 and F5 share skein's fault ledger, which F0 owns;
  - F2 and F6 share the engine's world kit, which F2 owns; F6 adds only the store check's call.
- **F9 last,** composing everything, within the budget F7 has freed.
- **Every goal's checks and sweeps pass on its branch,** with only the exclusions `failure-worlds.md`, section 6 allows. A finding outside them is repaired by the goal, or, if it is an open problem that section 7 there missed, gets its scenario and exclusion in the same goal, with its reason.
