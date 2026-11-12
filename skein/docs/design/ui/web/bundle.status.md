# bundle: status

2026-11-12. Where the tree's code lags or departs from [bundle.md](bundle.md).

- **Departs:** 2: `Cargo.toml` builds `flate2`, `miniz_oxide`, `crc32fast` and
  `adler2` at `opt-level` 3 in development, as it does the bundler, because the tiers
  and the build scripts run the dev bundler. `../../../foundation/testing-strategy.md`, 8 lets
  a tool build optimized, and has a dependency's override show a profile and the user's
  sign-off, with a comment citing both. Neither is recorded, and `flate2` also reaches
  `skein-ui-web-tests` as a dev dependency.
- **Not yet:** 3: no run on record shows skein's browser tier with the loader, which
  its stories on the example load, within `tools/tiers/budgets.toml`'s `browser-skein`, nor
  `browser_stories`'s `a_placeholder_gives_way_to_the_client` and
  `a_module_not_found_says_so_and_offers_to_try_again` passing there: the plan that wrote
  them ran focused and fuzzy alone, the tier waiting for its run after landing
  (the `bundle-loading` plan's figures on the browser tier, at `668f7cd3e5`).
- **Not yet:** 1: no `rust-toolchain.toml` pins the toolchain or names the target; the
  module builds on whatever toolchain the machine has, stable 1.95 on the builders
  [in flight: change 1919].
