# link: status

2026-11-04. Where skein's code lags or departs from [link.md](link.md).

- **Departs:** 1: `skein-oauth-accounts/src/exchange.rs` tells whether its link
  is encrypted and pumps at once when it isn't, because removing that retimes
  requests, which is the user's to decide.
