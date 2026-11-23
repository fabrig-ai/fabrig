# slow-tests-down: status

2026-11-23. Where the design lags or departs from [slow-tests-down.md](slow-tests-down.md).

- **Design lags:**
  - 3 to 6: no testing design of skein, hearth, smith or temper yet places the behaviour this draft moves: each slow binary's line in `tools/tiers/layers.toml` and its tree's testing document (`../testing/README.md`, `skein/docs/design/testing/README.md`, `hearth/docs/design/testing/README.md`, `smith/docs/design/testing.md`) still describe today's slow tests.
  - 7: no design sends a browser step's expiry through the starvation rule (`skein/docs/foundation/testing-strategy.md`, 2.10), which skein's browser harness fails at once today, and none bounds a red story's wall below the tier's hard limit.
- **Open:** none.
