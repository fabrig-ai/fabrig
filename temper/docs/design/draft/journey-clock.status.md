# journey-clock: status

2026-11-18. Where the design lags or departs from [journey-clock.md](journey-clock.md).

`../client/journey-clock.md` designs it whole; `../client/journey-placeholders.md`, 2.4 and 8, `../client/web/journeys.md`, 3.2 and 8.1, and `../client/tui/journeys.md`, 8.2, are revised to it. `../client/web/journeys.md`, 10 held nothing of a journey's clock, so it is unchanged.

- **Design departs:**
  - 2: the fake box's own loop, its set-up and its fetch keep the host's clock and randomness, where the draft starts the fake box from the fixed instant: its fetch of Forgejo checks real hosts' certificates against its wall, and a wall in the past would refuse them. Its engine and worker, and its fake peers, the fake model among them, take the fixed start; the agents and the program they run, `fake-claude`, keep the host's, since none of their times or draws reaches a client (`../client/journey-clock.md`, 4.1). Decided by the goal's designer on 2026-11-18: only the draft's letter is left, and what reaches a recording starts from the fixed start.
  - 2: each part's clock moves to its own deadlines, once the host's time for each has come, where the draft's clocks advance as `again` decides: no part's clock is `again`'s to reach while it runs, and moving only to its own deadlines makes the same steps the same turns at the same times, which the draft asks it for (`../client/journey-clock.md`, 3). `again` decides the rule, and the start each part takes. Decided by the goal's designer on 2026-11-18.
