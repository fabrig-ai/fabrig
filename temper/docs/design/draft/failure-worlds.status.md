# failure-worlds: status

2026-11-21. Where the design lags or departs from [failure-worlds.md](failure-worlds.md).

- **Design lags:**
  - 3: the inspection adapters' form, the two checks every component's waits and tables share, and the fault ledger are designed (`skein/docs/design/testing/inspection.md`; `skein/docs/design/testing/sweeps.md`, 3), with the engine's and the worker's adapters (`../testing/checks.md`, 6), jig's (`jig/docs/design/domain/testing.md`, 7) and smith's (`smith/docs/design/testing.md`, 6). 3.3's blame and tries for runs are designed, smith's half and the system world's (`smith/docs/design/faults.md`, 4; `../testing/checks.md`, 6.3). No design yet holds 3.3's blame and tries for gates with its failure table, 3.4's endings within their bounds outside the system world's account case, or 3.5's comparison of two runs.
  - 3.6: the stock profiles, policy and catalogues are checked figure by figure, not against one another.
  - 3.7: the start's link checks run only at a scenario's final cold restart.
  - 4: `skein/docs/design/testing/fakes/forge.md` has no lagging views; the fake machine has no partial fetch or full disk; and nothing models a stop in each phase of a start. The faults marked [script], [composition] or [assertion] need no design change.
  - 5: `../testing/README.md`, 2.1 has no world running several goals at once.
  - 7: nothing.
- **Design departs:**
  - 3: an adapter is a kit of its component's tree, beside it, over `pub` reads the component documents as its adapter's, not code inside the component's crate (`skein/docs/design/testing/inspection.md`, 2): the workspace has no feature or `cfg` for test-only code, and a shipped binary links no testing crate. The adapter still changes with its component, reads it by a shared borrow and is linked only by worlds. The goal's designer, 2026-11-20.
  - 6: a fault every draw of which is its open scenario's combination (today the cut streams, the billing refusal and the unserved model) is not configured in its sweep until its fix, rather than configured and excluded: `skein/docs/design/testing/sweeps.md`, 5 refuses an exclusion with no condition, and a configured fault that never falls fails its sweep. Its ignored scenario holds it meanwhile, and its step tests show it falls (`smith/docs/design/faults.md`, 5). The goal's designer, 2026-11-21.
  - 7: P72's scenario is in smith's agent-process world, not its session world, which runs no program; S1's and P126's are in temper's system world, since placement and an account's models are jig's (`smith/docs/design/faults.md`, 6; `../testing/checks.md`, 6.3). The goal's designer, 2026-11-21. P71's scenario passes as written, so it is held enabled among the active regressions rather than ignored, and its row left the table (`smith/docs/design/faults.md`, 6). The goal's coordinator, 2026-11-22.
- **Open:** 8: the moves between layers that F0's plan proposes are the user's decision.
