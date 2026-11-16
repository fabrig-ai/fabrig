# test-budgets: status

2026-11-16. Where the design lags or departs from [test-budgets.md](test-budgets.md).

- **Design lags:** 3: `skein/docs/foundation/testing-strategy.md`, 8.1 holds a budget wherever the suites run, and over budget is red in any run; `tools/tiers/design.md`, 7.3 and 11 judge a tree's suite as one figure; the gate's runner reads no budget. 4: no document places each test binary in a layer, and `tools/tiers/layers.toml` doesn't exist. 5 and 8: `tools/tiers/design.md`, 11 has no ledger, held budget or move. 6: the strategy's section 8 gives per-test guides, not references a test is held to. 7: the strategy's 8.1 makes every wall limit's expiry starved; each profile's `global-timeout` is 300 s focused (`default-wall-230.md`) and 600 s fuzzy. 9: `temper/docs/design/testing/checks.md`, 1 sets temper's focused budget at 1,250 CPU s, holds the worlds below the domain other than the real loop to 8 s together and the real loop to 15 s, and has no ideal by layer; the strategy's section 8 still says the focused suite runs by `cargo nextest run --workspace`.
- **Design departs:** none.
- **Open:** none.
