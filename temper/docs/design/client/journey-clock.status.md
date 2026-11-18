# journey-clock: status

2026-11-18. Where temper's code lags or departs from [journey-clock.md](journey-clock.md).

- **Not yet:**
  - 4.1: no fake box takes `--fixed-start` [in flight: change 7720].
  - 4.2 and 4.3: neither client takes a fixed start, and no recording holds `fixed` [in flight: change 7721].
  - 4.4: `again` gives no part a fixed start [in flight: change 7722].
  - 4.4: the committed journeys hold the host's clocks and seeds until `again` walks each of them again, once, after landing (temper/tiers'); until then each still prints as it did, and its next walk rewrites it once.
