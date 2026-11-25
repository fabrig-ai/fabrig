# look-requests: status

2026-11-25. Where temper's code lags or departs from [look-requests.md](look-requests.md).

- **Not yet:**
  - 2, 7: nothing reads `Look-hold`. Whoever starts requests' goals
    reads the trailers, keeps dependent requests held and stops at the
    first held request;
    `scripts/look-refresh.sh` refreshes and drops by `Look-request`
    and the print, not by holds.
  - 4: no `look-render` command renders a landed request beside the
    print. [in flight: change 17984]
  - 4: the writer's run checks no landed request and writes no
    `looks.html`, so no verdict reaches whoever starts requests' goals.
    [in flight: change 17985]
