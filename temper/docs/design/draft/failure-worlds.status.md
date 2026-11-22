# failure-worlds: status

2026-11-21. Where the design lags or departs from [failure-worlds.md](failure-worlds.md).

- **Design lags:**
  - 3: the inspection adapters' form, the two checks every component's waits and tables share, and the fault ledger are designed (`skein/docs/design/testing/inspection.md`; `skein/docs/design/testing/sweeps.md`, 3), with the engine's and the worker's adapters (`../testing/checks.md`, 6; the worker's waits and tables, `../testing/worker.md`, 2), jig's (`jig/docs/design/domain/testing.md`, 7) and smith's (`smith/docs/design/testing.md`, 6). No design yet holds 3.3's blame and tries for gates and runs with its failure table, 3.4's endings within their bounds, or 3.5's comparison of two runs.
  - 4: `skein/docs/design/testing/fakes/llm.md` has no shared rate window without `retry-after`; `skein/docs/design/testing/fakes/forge.md` has no lagging views; and nothing models a stop in each phase of a start. The faults marked [script], [composition] or [assertion] need no design change.
  - 5: `../testing/README.md`, 2.1 has no world running several goals at once.
  - 7: no design holds its rows but the settings rows, P128, P59 and P81's residual (`../testing/settings-tests.md`, 7).
- **Design departs:**
  - 3.7: the engine's forge world, which runs the forge without jig's core or store, checks the forge's half of a start over its stored records, by the function the root's link calls (`../testing/store-check.md`, 2); jig's half is checked in the worlds that run the core. The goal's designer, 2026-11-21.
  - 3.7: a sampled world's commits are drawn from a part of its own of the seed's scheduling stream, `store check`, so that the sample moves none of the world's draws (`jig/docs/design/domain/testing.md`, 7). The goal's designer, 2026-11-21.
  - 7, P59: the row is fixed. Settings apply's whole check already refuses removing a producer a stock template's work names, `Fault::Reference`, committing nothing; its scenario, `a_retired_producer_is_refused_while_named`, passed when written and runs enabled in the engine's domain world, not ignored for the provider failures' work (`../testing/settings-tests.md`, 3 and 7). The goal's coordinator, 2026-11-22, from the scenario's change.
  - 3: an adapter is a kit of its component's tree, beside it, over `pub` reads the component documents as its adapter's, not code inside the component's crate (`skein/docs/design/testing/inspection.md`, 2): the workspace has no feature or `cfg` for test-only code, and a shipped binary links no testing crate. The adapter still changes with its component, reads it by a shared borrow and is linked only by worlds. The goal's designer, 2026-11-20.
- **Open:**
  - 8: the moves between layers that F0's plan proposes are the user's decision.
  - 7: the settings walk found a problem the catalog has no row for: a member's batch addressed to a role whose authority spend can't fund it (`../testing/settings-tests.md`, 5 and 7). Its scenario is ignored; no work fixes it yet, and one of its fixes moves a stock figure, the user's.
