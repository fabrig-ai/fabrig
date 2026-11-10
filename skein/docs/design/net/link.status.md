# link: status

2026-11-06. Where skein's code lags or departs from [link.md](link.md).

- **Departs:** 1: `skein-oauth-accounts/src/exchange.rs` reads `encrypted` for a
  move, not a size: it pumps at once when its link isn't, because removing that
  retimes requests. The user left it so, 2026-11-06.
