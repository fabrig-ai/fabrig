# slow-tests-down: status

2026-11-23. Where the design lags or departs from [slow-tests-down.md](slow-tests-down.md).

- **Design lags:**
  - 3 to 6: each slow binary's line in `tools/tiers/layers.toml` and its tree's testing document (`../testing/README.md`, `skein/docs/design/testing/README.md`, `hearth/docs/design/testing/README.md`, `smith/docs/design/testing.md`, `jig/docs/design/domain/testing.md`) still describe today's slow tests, with two exceptions: `../testing/process-tier.md` places temper's real-Forgejo stories and repeated command paths (5.1), with each moved story beside its holders in `../../development/slow-tests-moves-2026-11-23-temper-forgejo.md`; `skein/docs/design/data/codec/command.md`, 9 places the `skein-codegen::suite` tables and the patch command's story (5.2). `temper-box::process`'s line in `tools/tiers/layers.toml` gives no reason yet, since later goals of this draft still move its stories (5.4 and 6).
  - 7: no design sends a browser step's expiry through the starvation rule (`skein/docs/foundation/testing-strategy.md`, 2.10), which skein's browser harness fails at once today, and none bounds a red story's wall below the tier's hard limit.
- **Open:** none.
