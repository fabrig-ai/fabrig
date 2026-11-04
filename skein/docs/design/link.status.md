# link: status

2026-11-04. Where skein's code lags or departs from [link.md](link.md).

- **Not yet:** 3.1: `Link::socket` is still public, for one guard of the forge connection's and
  the link world. 4: no
  test drives `Up::Failed` or `Up::End` during a close, or a close after TLS
  failed; the forge connection's stalled close has no test.
- **Departs:** 1: `skein-oauth-accounts/src/exchange.rs` tells whether its link
  is encrypted and pumps at once when it isn't, because removing that retimes
  requests, which is the user's to decide. 3.3: `Link` holds how it ends in
  `Ending`, its deadline in `Flushing`, but a close asked before `connecting`
  waits in `held`, an `Option` beside the state that means something only in
  Graceful and Aborted, against
  `skein/docs/foundation/programming-model.md`, 5.4; no reason is recorded.
