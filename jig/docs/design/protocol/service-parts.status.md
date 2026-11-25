# service-parts: status

2026-11-25. Where the tree's code lags or departs from [service-parts.md](service-parts.md).

- **Not yet:** 1.4: a connection `close` ends with nothing left to read
  still waits for its peer's end. [in flight: change 18057]

- **Departs:** 1, 1.3: `Workers`' `Debug` counts, but its public `io`, formatted alone, prints a queued write's bytes, a grant's among them, because `Queue` and `io::Request` derive their `Debug`s in skein, which is no part. No service formats it.
