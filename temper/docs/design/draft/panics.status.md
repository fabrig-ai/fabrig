# panics: status

2026-11-14. Where the design lags or departs from [panics.md](panics.md). Each milestone of [the execution file](panics.execution.md) brings into line the entries for the tests it converts.

- **Design lags:**
  - 4: `skein/docs/design/testing/README.md`, 5, has the kit end a fixture's processes while a test unwinds. `tools/tiers/design.md`, 11.5 and 11.6, classify a starved pass by its unwinding. `skein/docs/design/examples.md`, 6, reports and cleans up as a test unwinds.
  - 5: `temper/docs/design/testing/live-tier.md` has `Lent`'s `Drop` settle the held record on every ending, and `run_walk` catch a panic to return it before its box stops.
  - 7: `tools/lints/design.md`, 4, bans neither `catch_unwind` nor `resume_unwind`, which `skein/docs/foundation/programming-model.md`, 10.1, names among the checks of fail-stop.
- **Design departs:**
  - 4, its row "An assertion": a connector's kinds and a charter's models, which the row names among the causes outside the process, stay assertions, and their tests go. jig's tasks child takes both from code: temper's root sends its forge connector's one kind from its constants, and jig's core builds a charter's model indexes and the models' efforts from one settings catalogue, so a mismatch is our own code's bug. A model the configuration no longer lists is skipped there, and a retained choice it no longer admits refuses the start as `LlmChoice` (`jig/docs/design/domain/engine/restart.md`). Goal 2700, 2026-11-14, reading the tests again as section 4 asks.
- **Open:** whether the implementers' charters cite section 3.3's exemption for an assertion (section 10).
