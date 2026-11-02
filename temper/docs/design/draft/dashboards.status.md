# dashboards: status

2026-11-02. Where the design lags or departs from [dashboards.md](dashboards.md).

- **Design lags:**
  - 2, 4: jig's `domain/engine.md`, 17.5 has no metric kind, and temper's kinds (`../protocol/configuration.md`, 13.2) have no dashboard kind and no forge's metrics kind; no child of temper's root keeps dashboards.
  - 2, 4.3: jig's `client/domain/contract.md`, 7.4 gives "a metric's latest points" as an example of an application's stream, where this draft makes a metric's values a subject of committed documents (its 7.2), never a stream.
  - 5: skein's `ui.md` has no palette of panels, and `temper-client-view` no dashboard page (`../client/README.md`).
  - 6: the configuring agent (`../domain/agent.md`, 8.1) has no reads of facts, metrics or dashboards.
  - 8: no limits for dashboards, panels or metrics (`../protocol/configuration.md`).
