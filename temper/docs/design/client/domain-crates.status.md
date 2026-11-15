# domain-crates: status

2026-11-15. Where the code lags or departs from [domain-crates.md](domain-crates.md).

- **Not yet:** 3.1: the forge's side names the pages' `delegation`, holds `stock_ways`, reads the inbox's cards in `forge::ways::proposals`, and names `authorize::QUERY_BYTES`; `Fact` is the boundary's [in flight: change A].
- **Not yet:** 3.2: there is no `pages::Root` or `pages::Routing`: the pages take `Domain` and call the root's routes, and `forge::listed` and `forge::trials` take the root [in flight: change B].
- **Not yet:** 4.3: the tests aren't staged by piece [in flight: change C].
- **Not yet:** 2, 5: there is no `temper-client-domain-pages` or `temper-client-domain-forge`: `temper-client-domain` holds every file [in flight: change D].
- **Not yet:** 6: `temper-client-tools`, the engine's service crates' tests, `temper-client-browser-world`'s tests and `temper-client-shell-cli` depend on `temper-client-domain`, and `temper-service` names it unused [in flight: change E].
