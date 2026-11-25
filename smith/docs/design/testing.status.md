# testing: status

2026-11-24. Where smith's code lags or departs from [testing.md](testing.md).

- **Not yet:**
  - 4: the fake peers' held-listener table is reached only where a
    listener is held; a simulated world's peers still lock it.
    [in flight: change 17909]
  - 6: smith's inspection adapter, `smith-inspect`, and its worlds'
    calls of the shared checks.
  - 6: job ends paired across the event stream (they are paired on the
    domain's facts only), state-digest replay (skein has none), and,
    with contained trees, containment (`testing-tiers.md`, 2).
  - 7: the chat store's layout cases run over a real directory in
    `smith-local-protocol::suite`, not on the fake disk.
    [in flight: change 13947]
