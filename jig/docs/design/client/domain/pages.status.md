# pages: status

2026-11-03. Where jig's code lags or departs from [pages.md](pages.md).

- **Not yet:** 3: an entry carries at most 32 goals, the client wire's `InboxSelection.goals` (`jig/crates/jig-client-protocol-wire/schema/client-v19.schema`), and a client receives its `rows` of them (`jig/crates/jig-client-protocol-link/src/limits.rs`, `inbox_selection_goals`). 6.5: the policy section shows no template overrides. 7: connectors' request budgets. 12.1: `pages::owed` is no function of its own: `after_page` builds the owed cards inline (`jig/crates/jig-client-view/src/tree/pages.rs`), so an application cannot place them apart.
