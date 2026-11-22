# layers: status

2026-11-21. Where the code lags or departs from [layers.md](layers.md).

- **Not yet:** 9 and 9.1: `tools/tiers/one-process` runs every layer the focused and fuzzy profiles select, the real loop and end to end included, takes no `--layers` and writes no `selected_layers`, so the gate and the delivery check still run the real-loop and end-to-end binaries; the macOS suites parts pass no `--layers all`. [in flight: change 11541]
- **Not yet:** 7 and 9.2: `.config/nextest.toml` and `tools/tiers/packages.toml` have no `real-TREE` pair, and the boundary check reads no profile. [in flight: change 11543]
