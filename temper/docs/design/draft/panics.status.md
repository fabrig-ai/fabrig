# panics: status

2026-11-13. Where the design lags or departs from [panics.md](panics.md). Nothing of it is designed or built yet. Each milestone of [the execution file](panics.execution.md) brings into line the entries for the tests it converts.

- **Design lags:**
  - 3.2: `skein/docs/foundation/programming-model.md`, 10.4, binds assertions to inputs a peer controls and says nothing of catching; its 10.1 row "Fail-stop" names neither the rule nor its checks. `skein/docs/foundation/testing-strategy.md`, 1 and 10, test every bound one past it, an asserted limit's included.
  - 3.1, 3.3: tests of assertions that the design asks for: `skein/docs/design/net/http/testing.md`, its step tests of caller bugs; `skein/docs/design/net/tls.md`, 5; `skein/docs/design/runtime/io/README.md`, 5, its stale tokens and over-grants; `skein/docs/design/data/json.md`, 6, the writer's; `skein/docs/design/ui/README.md`, 14, the builder's; `jig/docs/design/protocol/llm.md`, 8, failure classes that can't be reached.
  - 3.1: a link panics on the store's contents in `jig/docs/design/domain/engine/restart.md`'s start interface (`start_missing`), against its own rule; in `jig/docs/design/domain/accounts.md`, 3.6; `jig/docs/design/protocol/store/README.md`, 10.9; `jig/docs/design/protocol/store/secrets.md`'s sweep; and in `upgrades.md`, 6.4. `jig/docs/design/domain/engine/README.md`, 9's oracle reads a link panic, where it would read a refusal.
  - 4: `skein/docs/design/testing/README.md`, 6, runs a table's rows under `catch_unwind`, reports every failing row, and has `skein_rows::panics` hold what `#[should_panic]` holds; its section 5 ends a starved pass in the one-process mode with `resume_unwind`, and has the kit end a fixture's processes while a test unwinds; its section 9 has a table collect every row's failure. `tools/tiers/design.md`, 11.5 and 11.6, classify a starved pass by its unwinding. `skein/docs/design/examples.md`, 6, reports and cleans up as a test unwinds.
  - 5: `temper/docs/design/testing/live-tier.md` has `Lent`'s `Drop` settle the held record on every ending, and `run_walk` catch a panic to return it before its box stops.
  - 7: `tools/lints/design.md` has no count of catches and no ban.
- **Design departs:** none.
- **Open:** whether the implementers' charters cite section 3.3's exemption for an assertion (section 10).
