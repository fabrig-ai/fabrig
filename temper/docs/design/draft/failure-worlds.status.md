# failure-worlds: status

2026-11-21. Where the design lags or departs from [failure-worlds.md](failure-worlds.md).

- **Design lags:**
  - 3: the inspection adapters' form, the two checks every component's waits and tables share, and the fault ledger are designed (`skein/docs/design/testing/inspection.md`; `skein/docs/design/testing/sweeps.md`, 3), with the engine's and the worker's adapters (`../testing/checks.md`, 6), jig's (`jig/docs/design/domain/testing.md`, 7) and smith's (`smith/docs/design/testing.md`, 6); the engine's in full, its world kit and 3.4's endings within their bounds in `../testing/engine-checks.md`. No design yet holds 3.3's blame and tries for gates and runs with its failure table, or 3.5's comparison of two runs.
  - 3.6: the stock profiles, policy and catalogues are checked figure by figure, not against one another.
  - 3.7: the start's link checks run only at a scenario's final cold restart.
  - 4: `skein/docs/design/testing/fakes/llm.md` has no shared rate window without `retry-after`; `skein/docs/design/testing/fakes/forge.md` has no lagging views; the fake machine has no partial fetch or full disk; and nothing models a stop in each phase of a start. The faults marked [script], [composition] or [assertion] need no design change.
  - 5: `../testing/README.md`, 2.1 has no world running several goals at once.
  - 7: nothing.
- **Design departs:**
  - 3: an adapter is a kit of its component's tree, beside it, over `pub` reads the component documents as its adapter's, not code inside the component's crate (`skein/docs/design/testing/inspection.md`, 2): the workspace has no feature or `cfg` for test-only code, and a shipped binary links no testing crate. The adapter still changes with its component, reads it by a shared borrow and is linked only by worlds. The goal's designer, 2026-11-20.
  - 3.2: build places are the worker's (`../domain/worker/README.md`), and the engine keeps none, so the engine's adapter lists none, though the engine's goal named them among its tables (`../testing/engine-checks.md`, 2.3). The engine's goal's designer, 2026-11-21.
  - 3.4: the engine emits no fact at its opening, so its referee observes the opening as the shell reads it, the driver's reading of the root's readiness before its first period, never the root's state otherwise (`../testing/engine-checks.md`, 4.1). The engine's goal's designer, 2026-11-21.
  - 7: P124's first scenario is the active test of its fix's goal, *A cancelled landing gives back its forge write slots*, not an ignored scenario of the engine's, and takes no exclusion; a sweep drawing its combination before that fix lands leaves it out by name (`../testing/engine-checks.md`, 5.1 and 5.2). The milestone, 2026-11-21.
  - 7: P124's second scenario bounds a wait for room by the change's stall, since no design gives that wait a bound; P90's refuses a change's close by permission, since the fake forge refuses no close by branch policy until the forge faults' goal builds 4.1's *Refusing* (`../testing/engine-checks.md`, 5.1). The engine's goal's designer, 2026-11-21.
- **Open:** 8: the moves between layers that F0's plan proposes are the user's decision.
