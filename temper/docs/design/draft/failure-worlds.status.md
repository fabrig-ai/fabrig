# failure-worlds: status

2026-11-21. Where the design lags or departs from [failure-worlds.md](failure-worlds.md).

- **Design lags:**
  - 3: the inspection adapters' form, the two checks every component's waits and tables share, and the fault ledger are designed (`skein/docs/design/testing/inspection.md`; `skein/docs/design/testing/sweeps.md`, 3), with the engine's and the worker's adapters (`../testing/checks.md`, 6; the worker's waits and tables, `../testing/worker.md`, 2), jig's (`jig/docs/design/domain/testing.md`, 7) and smith's (`smith/docs/design/testing.md`, 6). No design yet holds 3.3's blame and tries for gates and runs with its failure table, 3.4's endings within their bounds, or 3.5's comparison of two runs.
  - 3.6: the stock profiles, policy and catalogues are checked figure by figure, not against one another.
  - 3.7: the start's link checks run only at a scenario's final cold restart.
  - 4: `skein/docs/design/testing/fakes/llm.md` has no shared rate window without `retry-after`; `skein/docs/design/testing/fakes/forge.md` has no lagging views; and nothing models a stop in each phase of a start. The faults marked [script], [composition] or [assertion] need no design change.
  - 5: `../testing/README.md`, 2.1 has no world running several goals at once.
  - 7: nothing.
- **Design departs:**
  - 3: an adapter is a kit of its component's tree, beside it, over `pub` reads the component documents as its adapter's, not code inside the component's crate (`skein/docs/design/testing/inspection.md`, 2): the workspace has no feature or `cfg` for test-only code, and a shipped binary links no testing crate. The adapter still changes with its component, reads it by a shared borrow and is linked only by worlds. The goal's designer, 2026-11-20.
  - 7: P124's `cancelled_landing_frees_its_write_entries` is fixed by its own goal, before the landing train: an active focused test of `temper-engine-forge-world`'s suite (`../domain/landing-forming.md`, 5), so F2 neither writes it nor excludes it from a sweep. P124's other scenario, a wait for room with a bound, stays F2's. The user, 2026-11-21.
- **Open:** 8: the user moved temper's CPU down three layers on 2026-11-22, the transfers temper's suites measured before and after F7 support (`../../development/failure-worlds-room.md`): default simulated to step 0.1 s, fuzzy real-loop to domain 20 s, fuzzy simulated to system 2 s. No default real-loop to domain move is supported, the other trees' moves in F0's plan are unmeasured, and the layers still over their ceilings stay so, for after landing.
