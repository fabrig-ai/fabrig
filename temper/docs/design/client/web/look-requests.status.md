# look-requests: status

2026-11-26. Where temper's code lags or departs from [look-requests.md](look-requests.md).

- **Not yet:**
  - 2, 7: nothing reads `Look-hold`. Whoever starts requests' goals
    reads the trailers, keeps dependent requests held and stops at the
    first held request;
    `scripts/look-refresh.sh` refreshes and drops by `Look-request`
    and the print, not by holds.
