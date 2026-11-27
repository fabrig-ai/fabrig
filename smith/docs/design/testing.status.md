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
  - 6.3: the tools world's fuzzy loops take no case from skein's seam,
    and there is no `campaign-smith` pair.
    [in flight: change 19841]
  - 6.3: the run world's fuzzy loops take no case from skein's seam,
    and `campaign-smith` doesn't list `smith-run-world::suite`.
    [in flight: change 19843]
  - 6.3: the agent world's fuzzy loops take no case from skein's seam,
    and `campaign-smith` doesn't list `smith-agent-world::suite`.
    [in flight: change 19844]
