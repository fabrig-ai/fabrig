# connectors: status

2026-11-28. Where jig's code lags or departs from [connectors.md](connectors.md).

- **Not yet:** 2, 14: the engine asking a connector to restore its records, and taking what it says meanwhile [in flight: change 22485]
- **Not yet:** 14: `ops`'s infrastructure and observability restoring their own records when asked, and infrastructure's closes sent again as it catches up [in flight: change 22494]
- **Not yet:** 14: the test connector restoring its own records when asked [in flight: change 22499]
- **Not yet:** 14: temper's forge restoring its own records when asked [in flight: change 22501]

Section 11's procedure beside its requester's own writing run is present
in temper's forge connector. Its held-branch step tests and engine domain
stories cover that exception; another run, a lost writer and an effect
entry still stop the procedure. The exception grants no second writer.

Section 17 has no known gap in naming, at each start, the defaults a
connector stored that the application no longer gives:
`jig-test-connector` (`src/settings.rs`, `defaults`) and temper's forge
(`temper-engine-domain-forge/src/kinds.rs`, `dropped_settings_defaults`)
name them in `settings::Up::Defaults`' `dropped`;
the test connector no longer reports one as a default of size 0
(`engine/README.status.md`, its 17.5).

