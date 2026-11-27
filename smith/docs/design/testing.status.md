# testing: status

2026-11-27. Where smith's code lags or departs from [testing.md](testing.md).

- **Not yet:**
  - 6: smith's inspection adapter, `smith-inspect`, and its worlds'
    calls of the shared checks.
  - 6: job ends paired across the event stream (they are paired on the
    domain's facts only), state-digest replay (skein has none), and,
    with contained trees, containment (`testing-tiers.md`, 2).
  - 7: the chat store's layout cases run over a real directory in
    `smith-local-protocol::suite`, not on the fake disk.
    [in flight: change 13947]
  - 6.3: the session world's loop of 1000 seeds takes no case from
    skein's seam, its checks over the loop run in every mode, its
    referee refuses a write that creates a fixture file its session
    removed, and `campaign-smith` doesn't list
    `smith-session-world::suite` (`tools/tiers/layers.md`, 9.3).
    [in flight: change 20858]
