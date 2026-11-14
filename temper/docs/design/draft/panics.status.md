# panics: status

2026-11-14. Where the design lags or departs from [panics.md](panics.md). Section 5 is designed, by milestone P1. Each milestone of [the execution file](panics.execution.md) brings into line the entries for the tests it converts.

- **Design lags:**
  - 3.1, 3.3: tests of assertions that the design asks for: `skein/docs/design/net/http/testing.md`, its step tests of caller bugs; `skein/docs/design/net/tls.md`, 5; `skein/docs/design/runtime/io/README.md`, 5, its stale tokens and over-grants; `skein/docs/design/data/json.md`, 6, the writer's; `skein/docs/design/ui/README.md`, 14, the builder's; `jig/docs/design/protocol/llm.md`, 8, failure classes that can't be reached.
  - 3.1: a link panics on the store's contents in `jig/docs/design/domain/engine/restart.md`'s start interface (`start_missing`), against its own rule; in `jig/docs/design/domain/accounts.md`, 3.6; `jig/docs/design/protocol/store/README.md`, 10.9; `jig/docs/design/protocol/store/secrets.md`'s sweep; and in `upgrades.md`, 6.4. `jig/docs/design/domain/engine/README.md`, 9's oracle reads a link panic, where it would read a refusal.
  - 4: `skein/docs/design/testing/README.md`, 5, has the kit end a fixture's processes while a test unwinds. `skein/docs/design/examples.md`, 6, reports and cleans up as a test unwinds.
  - 7: `tools/lints/design.md`, 4, bans neither `catch_unwind` nor `resume_unwind`, which `skein/docs/foundation/programming-model.md`, 10.1, names among the checks of fail-stop.
- **Design departs:**
  - 5: the kits' processes start dying with their parent through skein-shell's parent-side `spawn_dying_with_parent` (`skein/docs/design/runtime/shell.md`, 3), the same signal and parent re-check as `exec_dying_with_parent`, because a test kit has no binary of its own to exec through. The designer's, goal 2654, 2026-11-14.
  - 5: smith's live Claude keeps the sessions it writes in the signed-in configuration directory it uses in place (`smith/docs/design/protocol/programs.md`, 8), not in the test's scratch, because a login is never copied (`smith/docs/design/testing.md`, 2.3). Its home and working directory are the test's scratch. The designer's, goal 2654, 2026-11-14.
- **Open:** whether the implementers' charters cite section 3.3's exemption for an assertion (section 10).
