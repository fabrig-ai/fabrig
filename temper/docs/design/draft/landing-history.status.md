# landing-history: status

2026-11-13. Where the design lags or departs from [landing-history.md](landing-history.md).

Milestone H1, goal 2474, is designed in `../domain/landing-history.md`: the events, their families, the writer at each step under `queue` and `turn`, the sweep, `landing history` and `change show` of a change whose row went.

- **Design lags:**
  - 7: `landing times`, where the time went, is no design's yet: `../domain/landing-history.md`, 9 leaves it to milestone H2.
- **Design departs:**
  - 3, `gate`: written when the run is first known by its id, its dispatch found or its rerun made, not when the dispatch is sent, since a run's identity is its id and attempt and a dispatch has none until found; its `source_at` is the run's creation (`../domain/landing-history.md`, 2.2). Decided by goal 2474, 2026-11-13.
- **Open:** none.
