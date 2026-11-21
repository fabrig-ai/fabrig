# look-requests: status

2026-11-21. Where temper's code lags or departs from [look-requests.md](look-requests.md).

- **Not yet:**
  - 2, 3: nothing reads `Look-hold`. Whoever lands reads the trailers,
    keeps dependent requests held and stops at the first held request;
    `scripts/look-refresh.sh` refreshes and drops by `Look-request`
    and the print, not by holds.
  - 4: no built render comparison checks the request against main's
    pages as the writer printed them. The after-landing owner carries
    out the comparison. There is no render-check command; section 8
    defers one.
