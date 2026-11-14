# pages: status

2026-11-13. Where jig's code lags or departs from [pages.md](pages.md).


- **Not yet:** 6.5: the policy section shows no template overrides. 7: connectors' request budgets. 12.1: `pages::owed` is no function of its own: `after_page` builds the owed cards inline (`jig/crates/jig-client-view/src/tree/pages.rs`), so an application cannot place them apart.
