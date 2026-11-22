# failure-worlds: status

2026-11-20. Where the design lags or departs from [failure-worlds.md](failure-worlds.md).

- **Design lags:**
  - 3: the inspection adapters' form, the two checks every component's waits and tables share, and the fault ledger are designed (`skein/docs/design/testing/inspection.md`; `skein/docs/design/testing/sweeps.md`, 3), with the engine's and the worker's adapters (`../testing/checks.md`, 6), jig's (`jig/docs/design/domain/testing.md`, 7) and smith's (`smith/docs/design/testing.md`, 6). No design yet holds 3.3's blame and tries for gates and runs with its failure table, 3.4's endings within their bounds, or 3.5's comparison of two runs.
  - 3.6: the stock profiles, policy and catalogues are checked figure by figure, not against one another.
  - 3.7: the start's link checks run only at a scenario's final cold restart.
  - 4: `skein/docs/design/testing/fakes/llm.md` has no shared rate window without `retry-after`; `skein/docs/design/testing/fakes/forge.md` has no lagging views; and the fake machine has no partial fetch or full disk. 4.8's faults are designed (`../testing/restarts.md`). The faults marked [script], [composition] or [assertion] need no design change.
  - 5: `../testing/README.md`, 2.1 has no world running several goals at once.
  - 7: nothing.
- **Design departs:**
  - 3: an adapter is a kit of its component's tree, beside it, over `pub` reads the component documents as its adapter's, not code inside the component's crate (`skein/docs/design/testing/inspection.md`, 2): the workspace has no feature or `cfg` for test-only code, and a shipped binary links no testing crate. The adapter still changes with its component, reads it by a shared borrow and is linked only by worlds. The goal's designer, 2026-11-20.
  - 4.8: an older layout falls at a run's start, drawn for each seed, not at any iteration: a world runs one build, so the only store the previous release writes is the image that release's build wrote in the one-process world (`../protocol/floor.md`, 5), and the upgrade is the first start over it. The restarts within that run still fall at any iteration (`../testing/restarts.md`, 8). The goal's designer, 2026-11-21.
  - 4.8: a stop in each phase of a start has no stop between the mark and the opening in any world. Since 3f1a0ea3ff (the engine opens before its forge has caught up), the start adopts its runs in the same root input that takes the mark's answer and opens, so nothing a deployment does stops it there, and no world injects a state nothing reaches. The core's step test covers a stop at `AdoptRuns` (`temper/crates/temper-engine-domain/src/tests/start_stop.rs`), and a stop after the opening covers the forge's catch-up (`../testing/restarts.md`, 4). Decided by the drafter, 2026-11-22. Open, the restart drafts' owner's to design: what a stop does with the mark's commit stored and not yet answered.
- **Open:** 8: the moves between layers that F0's plan proposes are the user's decision.
