# panics: status

2026-11-14. Where the design lags or departs from [panics.md](panics.md). Section 5 is designed, by milestone P1. Each milestone of [the execution file](panics.execution.md) brings into line the entries for the tests it converts.

- **Design lags:**
  - 4: `tools/tiers/design.md`, 11.5 and 11.6, classify a starved pass by its unwinding.
  - 7: `tools/lints/design.md`, 4, bans neither `catch_unwind` nor `resume_unwind`, which `skein/docs/foundation/programming-model.md`, 10.1, names among the checks of fail-stop.
- **Design departs:**
  - 4, its row "An assertion": a connector's kinds and a charter's models, which the row names among the causes outside the process, stay assertions, and their tests go. jig's tasks child takes both from code: temper's root sends its forge connector's one kind from its constants, and jig's core builds a charter's model indexes and the models' efforts from one settings catalogue, so a mismatch is our own code's bug. A model the configuration no longer lists is skipped there, and a retained choice it no longer admits refuses the start as `LlmChoice` (`jig/docs/design/domain/engine/restart.md`). Goal 2700, 2026-11-14, reading the tests again as section 4 asks.
  - 5: the kits' processes start dying with their parent through skein-shell's parent-side `spawn_dying_with_parent` (`skein/docs/design/runtime/shell.md`, 3), the same signal and parent re-check as `exec_dying_with_parent`, because a test kit has no binary of its own to exec through. The designer's, goal 2654, 2026-11-14.
  - 5: smith's live Claude keeps the sessions it writes in the signed-in configuration directory it uses in place (`smith/docs/design/protocol/programs.md`, 8), not in the test's scratch, because a login is never copied (`smith/docs/design/testing.md`, 2.3). Its home and working directory are the test's scratch. The designer's, goal 2654, 2026-11-14.
- **Open:** whether the implementers' charters cite section 3.3's exemption for an assertion (section 10).
