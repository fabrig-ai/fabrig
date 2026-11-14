# landing-history: status

2026-11-13. Where the design lags or departs from [landing-history.md](landing-history.md).

Milestone H1, goal 2474, is designed in `../domain/landing-history.md`: the events, their families, the writer at each step under `queue` and `turn`, the sweep, `landing history` and `change show` of a change whose row went. Milestone H2, goal 2649, is designed in `../domain/landing-times.md`: the split, each landing's rounds, `landing times` and its wire.

- **Design lags:** none.
- **Design departs:**
  - 3, `gate`: written when the run is first known by its id, its dispatch found or its rerun made, not when the dispatch is sent, since a run's identity is its id and attempt and a dispatch has none until found; its `source_at` is the run's creation (`../domain/landing-history.md`, 2.2). Decided by goal 2474, 2026-11-13.
  - 7, waiting: ready and neither selected nor in a formed round, not "ready and joined", since under `turn` a change is joined only with its landing's first formed round (`../domain/landing-history.md`, 2.4), so a `turn` change awaiting its landing would wait in other (`../domain/landing-times.md`, 2.2). Decided by goal 2649, 2026-11-13.
  - 7, proving: a `queue` change's checks at its own head are no interval, and their time falls to what else holds, most often other, since H2 reads only what H1 writes and H1 writes `gate` for a round's top or a probed prefix alone (`../domain/landing-times.md`, 2.2). Decided by goal 2649, 2026-11-13.
  - 7, delivering: a head temper pushed itself (an update's or a job's) or one a delivery already brought opens no delivering, and an observed push no delivery accepts ends at the next `ready` or push, so an update's head is not counted as awaiting acceptance until the change's end (`../domain/landing-times.md`, 2.2). Decided by goal 2649, 2026-11-13.
  - 7, partial: a change is partial where its history holds, before its first `ready`, an event only a ready change writes, not wherever its first event isn't `ready`, since a whole history begins with the change's first delivery or push before it is ready (`../domain/landing-times.md`, 2.4). Decided by goal 2649, 2026-11-13.
- **Open:** none.
