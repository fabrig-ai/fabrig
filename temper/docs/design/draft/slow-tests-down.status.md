# slow-tests-down: status

2026-11-23. Where the design lags or departs from [slow-tests-down.md](slow-tests-down.md).

- **Design lags:**
  - 3 to 6: no tree's testing design yet places the behaviour this draft moves: each slow binary's line in `tools/tiers/layers.toml` and its tree's testing document (`../testing/README.md`, `skein/docs/design/testing/README.md`, `hearth/docs/design/testing/README.md`, `smith/docs/design/testing.md`, `jig/docs/design/domain/testing.md`) still describe today's slow tests, but for GitHub's conformance (5.2), which `skein/docs/design/testing/fakes/github.md`, 2 places.
  - 7: no design sends a browser step's expiry through the starvation rule (`skein/docs/foundation/testing-strategy.md`, 2.10), which skein's browser harness fails at once today, and none bounds a red story's wall below the tier's hard limit.
- **Design departs:** 5.2: `github-conformance::suite`'s runner tests run in a simulated world, not a protocol one, since they host the fake GitHub as a process of skein's simulator, which the strategy's table places there (`skein/docs/foundation/testing-strategy.md`, 2); its checks that drive no runner are step tests (`skein/docs/design/testing/fakes/github.md`, 2). The goal's designer decided it, 11-23, within the goal's word, "protocol or simulated".
- **Open:** none.
