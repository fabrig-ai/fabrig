# panics: status

2026-11-14. Where the design lags or departs from [panics.md](panics.md). Each milestone of [the execution file](panics.execution.md) brings into line the entries for the tests it converts.

- **Design lags:**
  - 3.1, 3.3: tests of assertions that the design asks for: `skein/docs/design/net/http/testing.md`, its step tests of caller bugs; `skein/docs/design/net/tls.md`, 5; `skein/docs/design/runtime/io/README.md`, 5, its stale tokens and over-grants; `skein/docs/design/data/json.md`, 6, the writer's; `skein/docs/design/ui/README.md`, 14, the builder's; `jig/docs/design/protocol/llm.md`, 8, failure classes that can't be reached.
  - 3.1: a link panics on the store's contents in `jig/docs/design/domain/engine/restart.md`'s start interface (`start_missing`), against its own rule; in `jig/docs/design/domain/accounts.md`, 3.6; `jig/docs/design/protocol/store/README.md`, 10.9; `jig/docs/design/protocol/store/secrets.md`'s sweep; and in `upgrades.md`, 6.4. `jig/docs/design/domain/engine/README.md`, 9's oracle reads a link panic, where it would read a refusal.
  - 4: `skein/docs/design/testing/README.md`, 5, has the kit end a fixture's processes while a test unwinds. `tools/tiers/design.md`, 11.5 and 11.6, classify a starved pass by its unwinding. `skein/docs/design/examples.md`, 6, reports and cleans up as a test unwinds.
  - 5: `temper/docs/design/testing/live-tier.md` has `Lent`'s `Drop` settle the held record on every ending, and `run_walk` catch a panic to return it before its box stops.
  - 7: `tools/lints/design.md`, 4, bans neither `catch_unwind` nor `resume_unwind`, which `skein/docs/foundation/programming-model.md`, 10.1, names among the checks of fail-stop.
- **Design departs:** none.
- **Open:** whether the implementers' charters cite section 3.3's exemption for an assertion (section 10).
