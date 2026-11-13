# panics: status

2026-11-13. Where the design lags or departs from [panics.md](panics.md). Nothing of it is designed or built yet.

- **Design lags:**
  - 3.2: `skein/docs/foundation/programming-model.md`, 10.4, binds assertions to inputs a peer controls and says nothing of catching; its 10.1 row "Fail-stop" names neither the rule nor its checks.
  - 4: `skein/docs/design/testing/README.md`, 6, runs a table's rows under `catch_unwind` and has `skein_rows::panics` hold what `#[should_panic]` holds; its section 5 ends a starved pass in the one-process mode with `resume_unwind`.
  - 5: `temper/docs/design/testing/live-tier.md` has `Lent`'s `Drop` settle the held record on every ending, and `run_walk` catch a panic to keep its box up.
  - 7: `tools/lints/design.md` has no count of catches and no ban.
- **Design departs:** none.
- **Open:** whether the implementers' charters cite section 3.3's exemption for an assertion (section 10).
