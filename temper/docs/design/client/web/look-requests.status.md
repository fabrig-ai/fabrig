# look-requests: status

2026-11-11. Where temper's code lags or departs from [look-requests.md](look-requests.md).

- **Not yet:**
  - 2, 3: nothing reads `Look-hold`. Whoever lands reads the trailers,
    keeps dependent requests held and stops at the first held request;
    `scripts/look-refresh.sh` refreshes and drops by `Look-request`
    alone, not by holds.
  - 4: no built render comparison checks the request against the landed
    pages. The after-landing owner carries out the comparison; the
    journeys' test checks only the reprint against the committed pages.
    There is no render-check command; section 8 defers one.
- **Departs:** none.
