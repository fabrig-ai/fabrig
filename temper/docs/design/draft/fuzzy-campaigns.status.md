# fuzzy-campaigns: status

2026-11-26. Where the design lags or departs from [fuzzy-campaigns.md](fuzzy-campaigns.md).

- **Design lags:** 1, 5 and 8: `skein/docs/foundation/testing-strategy.md`, 1 asks the same work at every run, with no campaign beside the suites. Its 8 keeps a failing seed but asks no repair to widen its world. Its 8.1 gives every tier a CPU budget and a wall, and has no campaign pair.
- **Design lags:** 3: skein's channel suite runs two tests in both suites, `generic_protocol.rs` registered by `skein/tests/channel/tests/suite/main.rs` and again under `fuzzy/channel.rs`.
- **Design lags:** 5: `tools/tiers/design.md`, `tools/tiers/layers.md`, 9 and `tools/tiers/budgets.md`, 4.2 have no campaign pair, no `tools/tiers/campaign`, and layer rows for the real pairs alone. `.config/gate/README.md`, After landing names no campaign.
- **Design lags:** 4.4 and 6: `temper/docs/design/testing/workloads.md`, 8 gives each sweep one list of kinds and the fewest seeds that make them fall, with no campaign configuration. Its composed workload's draws aren't valid at every seed: a configured kind with no eligible goal panics, and a seed can hold a train member without releasing it. `skein/docs/design/testing/sweeps.md`, 4.2 gives a campaign as many cases as a sweep's gate seeds, with no way to ask for more.
- **Open:** 5.3 and 5.5: which builder runs the daily campaign, within what envelope and under what wall. These are the user's: hardware and figures. Once a day, on a builder apart from the gate, is the user's word (11-26).
