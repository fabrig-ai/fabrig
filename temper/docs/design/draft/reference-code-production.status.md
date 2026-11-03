# reference-code-production: status

2026-11-03. Where the design lags or departs from [reference-code-production.md](reference-code-production.md).

- **Design departs:**
  - **10, 13: a red of g's tier waking G.** `../domain/after-landing.md`,
    5.4 and 7 wake the goal on a red with a test the branch's previous
    run did not fail; a red whose tests all failed that run too is news
    kept for the goal's next run, not waking it, so that a red a live
    repair already owns does not wake G at xhigh at each span (N11).
    Decided by reference-m9's cut, 2026-11-03.
