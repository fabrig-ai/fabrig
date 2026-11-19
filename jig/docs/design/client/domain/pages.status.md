# pages: status

2026-11-19. Where jig's code lags or departs from [pages.md](pages.md).

- **Not yet:** 5: a run's activity saying why it began fresh. [in flight: change 9225]

- **Not yet:** 3, *Live and bounded*: the needs past the engine's cached prefix are read from the store, which holds none [in flight: change 7577].

- **Not yet:** 6.5: the policy section shows no template overrides. 7: connectors' request budgets. 12.1: `pages::owed` is no function of its own: `after_page` builds the owed cards inline (`jig/crates/jig-client-view/src/tree/pages.rs`), so an application cannot place them apart.
