# dashboards: status

2026-11-08. Where the design lags or departs from [dashboards.md](dashboards.md).

- **Design lags:**
  - 2, 4: jig's `domain/engine/settings.md`, 5 has no metric kind, and temper's kinds (`../protocol/configuration.md`, 13.2) have no dashboard kind and no forge's metrics kind; no child of temper's root keeps dashboards.
  - 2, 4.3: jig's `client/domain/contract/README.md`, 5.4 gives "a metric's latest points" as an example of an application's stream, where this draft makes a metric's values a subject of committed documents (its 7.2), never a stream.
  - 5: skein's `ui/README.md` has no palette of panels, and `temper-client-view` no dashboard page (`../client/README.md`).
  - 6: Ask temper, which configures (`../domain/agent.md`, 8.1), has no reads of facts, metrics or dashboards (`../domain/agent.md`, 8.4.1).
  - 8: no limits for dashboards, panels or metrics (`../protocol/configuration.md`).
  - 9: no design document holds the stories or the golden (`../domain/testing.md`), and the example project's description the golden reads is not written.
- **Design departs:**
  - 1, 6.1: `../domain/agent.md`, 8.1 has Ask temper configure, where this draft has the configuring agent and the configuring chat ask and propose: `ask-temper.md`, section 9, the user, 2026-11-07.
- **Open:**
  - 11: where the question of public dashboards stands: the draft names a branch, `draft-public-reading`, that does not exist, and `../protocol/sign-in.md`, which designs public reading, has no dashboards.
