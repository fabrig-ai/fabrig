# layers: status

2026-11-16. Where the code lags or departs from [layers.md](layers.md).

- **Not yet:** 1, 4, 5: `tools/tiers/one_process.py` and `tests/boundary/tests/layers.rs` still place a test target named `generated` by rule, and `layers.toml`'s header names it [in flight: change 7639].

No other gap: `tools/tiers/layers.toml` places every test binary of the workspace, and `tests/boundary/tests/layers.rs` keeps it whole.
