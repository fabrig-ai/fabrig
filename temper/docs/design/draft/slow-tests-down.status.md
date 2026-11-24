# slow-tests-down: status

2026-11-23. Where the design lags or departs from [slow-tests-down.md](slow-tests-down.md).

- **Design lags:**
  - 3 to 6: the testing designs of skein, hearth, smith and temper do not yet place the behaviour this draft moves, except for 5.2's `skein-codegen::suite` tables and the patch command's story, which `skein/docs/design/data/codec/command.md`, 9 places, and `smith::end_to_end`, which `smith/docs/design/testing.md`, 2.3 and 2.5 place, its line in `layers.toml` with its reason. The remaining slow binaries' lines in `tools/tiers/layers.toml` and their trees' testing documents (`../testing/README.md`, `skein/docs/design/testing/README.md`, `hearth/docs/design/testing/README.md`, `smith/docs/design/testing.md`) still describe today's slow tests.
  - 7: no design sends a browser step's expiry through the starvation rule (`skein/docs/foundation/testing-strategy.md`, 2.10), which skein's browser harness fails at once today, and none bounds a red story's wall below the tier's hard limit.
- **Open:** none.
