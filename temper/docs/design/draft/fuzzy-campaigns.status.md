# fuzzy-campaigns: status

2026-11-26. Where the design lags or departs from [fuzzy-campaigns.md](fuzzy-campaigns.md).

- **Design lags:** 4: `skein/docs/design/testing/sweeps.md`, 4 has `each_seed` alone, a range judged seed by seed, with no test identity, no mode, no case lines and no `gate()`. `Sweep` keeps the set of kinds that fell, not their counts. skein has no test table and no generated test-env document (`skein/docs/foundation/cli.md`, 8).
- **Design lags:** 4.2: `tools/tiers/design.md`, 11.5 gives each test process the caller's environment less the `NEXTEST` variables, so a campaign's variable in the caller's shell would reach the gate's run.
- **Design lags:** 5: `tools/tiers/design.md`, `tools/tiers/layers.md`, 9 and `tools/tiers/budgets.md`, 4.2 have no campaign pair, no `tools/tiers/campaign`, and layer rows for the real pairs alone. `.config/gate/README.md`, After landing names no campaign.
- **Design lags:** 4.4 and 6: `temper/docs/design/testing/workloads.md`, 8 gives each sweep one list of kinds and the fewest seeds that make them fall, with no campaign configuration. Its composed workload's draws aren't valid at every seed: a configured kind with no eligible goal panics, and a seed can hold a train member without releasing it.
- **Open:** 5.3 and 5.5: which builder runs the daily campaign, within what envelope and under what wall. These are the user's: hardware and figures. Once a day, on a builder apart from the gate, is the user's word (11-26).
