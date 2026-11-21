# failure-worlds: status

2026-11-21. Where the design lags or departs from [failure-worlds.md](failure-worlds.md).

- **Design lags:**
  - 3: the inspection adapters' form, the two checks every component's waits and tables share, and the fault ledger are designed (`skein/docs/design/testing/inspection.md`; `skein/docs/design/testing/sweeps.md`, 3), with the engine's and the worker's adapters (`../testing/checks.md`, 6), jig's, with its worlds' calls and its open problems (`jig/docs/design/domain/testing.md`, 7.1 to 7.3), and smith's (`smith/docs/design/testing.md`, 6). No design yet holds 3.3's blame and tries for gates and runs with its failure table, 3.4's endings within their bounds, or 3.5's comparison of two runs.
  - 3.6: the stock profiles, policy and catalogues are checked figure by figure, not against one another.
  - 3.7: the start's link checks run only at a scenario's final cold restart.
  - 4: `skein/docs/design/testing/fakes/llm.md` has no shared rate window without `retry-after`; `skein/docs/design/testing/fakes/forge.md` has no lagging views; the fake machine has no partial fetch or full disk; and nothing models a stop in each phase of a start. The faults marked [script], [composition] or [assertion] need no design change.
  - 5: `../testing/README.md`, 2.1 has no world running several goals at once.
  - 7: nothing.
- **Design departs:**
  - 3.1: jig's bounded waits are checked at each quiet point against the component's own armed deadline, not by a deadline the referee arms: jig's worlds settle within each drain or fail, so a wait starved past its bound is found at the first quiet point after it (`jig/docs/design/domain/testing.md`, 7.2). The goal's designer, 2026-11-21.
  - 7: P76's scenario runs in jig's core world, not its people world, which answers account changes with a fixed outcome and has no accounts child, so only the core world shows a service paid by an account (`jig/docs/design/domain/testing.md`, 7.3). The goal's designer, 2026-11-21.
  - 3: an adapter is a kit of its component's tree, beside it, over `pub` reads the component documents as its adapter's, not code inside the component's crate (`skein/docs/design/testing/inspection.md`, 2): the workspace has no feature or `cfg` for test-only code, and a shipped binary links no testing crate. The adapter still changes with its component, reads it by a shared borrow and is linked only by worlds. The goal's designer, 2026-11-20.
- **Open:** 8: the moves between layers that F0's plan proposes are the user's decision.
