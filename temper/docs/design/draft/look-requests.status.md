# look-requests: status

2026-11-11. Where the design lags or departs from [look-requests.md](look-requests.md).

`../client/web/look-requests.md`, sections 1 to 7, holds the round trip;
its status records the implementation gaps. `../client/web/journeys.md`,
7, keeps the journeys' side and cites it for the procedure. The draft's
section 8 is `../client/web/look-requests.md`, 8.

- **Design lags:** none.
- **Design departs:**
  - 1, 2: the TUI's requests stack on a branch of their own, `tui-requests`, named `tui-…`, apart from the browser's on `look-requests`, where this draft keeps every request on one branch; the same `look-refresh.sh` keeps each on `main` (`../client/tui/journeys.md`, 10; `../client/web/look-requests.md`, 1 and 2), so that the TUI's look and the web's are asked for in parallel (the user, 2026-11-08).
- **Open:** none.
