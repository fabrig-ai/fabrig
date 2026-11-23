# slow-tests-down: status

2026-11-23. Where the design lags or departs from [slow-tests-down.md](slow-tests-down.md).

- **Design lags:**
  - 3 to 6: no tree's testing design yet places the behaviour this draft moves: each slow binary's line in `tools/tiers/layers.toml` and its tree's testing document (`../testing/README.md`, `skein/docs/design/testing/README.md`, `hearth/docs/design/testing/README.md` but for hearth's groups of 5.1 and 5.2, which its 2.5, 2.7 and 3.2 place, `smith/docs/design/testing.md`, `jig/docs/design/domain/testing.md`) still describe today's slow tests.
  - 7: no design sends a browser step's expiry through the starvation rule (`skein/docs/foundation/testing-strategy.md`, 2.10), which skein's browser harness fails at once today, and none bounds a red story's wall below the tier's hard limit.
- **Design departs:**
  - 5.2: hearth's builder matrix goes to the builder's protocol world alone, and none of it to the render domain (`hearth/docs/design/testing/README.md`, 2.5), because each of its cases checks what the client prints or its exit, which the render domain doesn't make; the goal's designer decided, 2026-11-23.
  - 5.1: hearth's index and progress checks go to the builder's protocol world, resolving over mmdebstrap's recorded failures (`hearth/docs/design/testing/README.md`, 2.5), rather than to `recorded_resolve`, which runs the fake without the builder that caches the indices and prints the progress; the goal's designer decided, 2026-11-23.
- **Open:** none.
