# slow-tests-down: status

2026-11-23. Where the design lags or departs from [slow-tests-down.md](slow-tests-down.md).

- **Design lags:**
  - 3 to 6: no tree's testing design yet places the behaviour this draft moves: each slow binary's line in `tools/tiers/layers.toml` and its tree's testing document (`../testing/README.md`, `skein/docs/design/testing/README.md`, `hearth/docs/design/testing/README.md`, `smith/docs/design/testing.md`, `jig/docs/design/domain/testing.md`) still describe today's slow tests, but for `smith::end_to_end`, which `smith/docs/design/testing.md`, 2.3 and 2.5 place, its line in `layers.toml` with its reason.
  - 7: no design sends a browser step's expiry through the starvation rule (`skein/docs/foundation/testing-strategy.md`, 2.10), which skein's browser harness fails at once today, and none bounds a red story's wall below the tier's hard limit.
- **Open:** none.
