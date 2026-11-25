# sockets: status

2026-11-25. Where the tree's code lags or departs from [sockets.md](sockets.md).

- **Not yet:** 3: no close without the wait; every `Close` drains to
  the peer's end or the close deadline. [in flight: change 18057]
