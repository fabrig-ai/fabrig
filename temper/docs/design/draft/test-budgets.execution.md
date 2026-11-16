# test-budgets: execution

2026-11-16. How [test-budgets.md](test-budgets.md) gets carried out: in small steps, each landing on its own before the next in its stream starts from main, so no branch grows large or old. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone. The gate holds nothing until its runner, outside this repository, reads what B3 and B4 write (section 2).

## 1. The milestones

Each is a goal, one change of one concern. A milestone that grows past a day's change splits, and this file says how.

| Milestone | Draft parts | Stream | Needs | Its design | State |
|---|---|---|---|---|---|
| B1, the layers: `tools/tiers/layers.toml`, a line for every test binary that isn't a library's own or generated, placed by its tree's testing document; a boundary check in `tests/boundary/` that every binary in `cargo metadata` has its layer and every line names one that exists | 4 | budgets | none | `tools/tiers/layers.md`; the strategy's 8.1, a binary in one layer | landed, goal 5270 |
| B2, the budgets by layer: `tools/tiers/budgets.toml`'s figures by tree, suite and layer at their start, with their ideals and the per-test references (`test-budgets.md`, 8, 9 and 6); `tools/tiers/one-process`'s summary by layer, red over a budget where the caller asks for the gate's verdict and a warning otherwise; the old per-tree figures replaced | 3, 4, 8, 9 | budgets | B1 | `tools/tiers/design.md`, 7.3 and 11; the strategy's 8 and 8.1 (budgets by layer, red at the gate only); `temper/docs/design/testing/checks.md`, 1 | not started |
| B3, a partial run judged whole: `tools/tiers/one-process` reads a ledger of main's figures and writes its own run's in the same form; a layer's figure as the measured plus the ledger's; the held budget, the lower of the ceiling and the lowest figure at main plus a tenth; the moves down and their check; a red naming its layer, binaries and tests | 5, 8 | budgets | B2 | `tools/tiers/design.md`, 11; the strategy's 8.1, the ratchet and the moves | not started |
| B4, each test's reference: new tests run alone after the suite, flagged ones alone at the head and the base, their CPU less an empty run's; the reasons' lines and their figures | 6 | budgets | B3 | `tools/tiers/design.md`, 11; the strategy's 8, the references | not started |
| B5, the wall: each profile's `global-timeout` at three times the gate's limit (810 s focused, 645 s fuzzy), a configuration with the gate's limits (270 s, 215 s) for the gate to pass, `default-wall-230.md` and its status deleted | 7 | wall | none | the strategy's 8.1, the gate's wall red unless starved | landed, goal 5271 (`tools/tiers/design.md`, 11.7); the gate's runner passes the walls and reads its stalls once it follows (section 2) |

## 2. Order

- **Each milestone is a goal,** started once the user approves it.
- **No priority mark:** budgets aren't one of v0.1.0's critical pieces. Its plans start when places are free and marked work isn't about to arrive.
- **One at a time within a stream,** each from main once the one before it has landed. B5 runs beside the budgets stream.
- **The gate's runner, outside this repository,** follows B3, B4 and B5: it keeps the ledger beside its reports, passes it and the gate's configuration to its runs, makes red cases of what they judge red, and reads its cgroup's stalls for the wall (`test-budgets.md`, 7). Until it does, B2 to B4's verdicts print and hold nothing.
- **The figures after landing:** each layer's figure, budget and ideal, from the gate's report, read by whoever runs the CI builder. No plan's.
