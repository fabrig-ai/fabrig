# reconciling-start: status

2026-11-20. Where the design lags or departs from [reconciling-start.md](reconciling-start.md).

- **Design lags:**
  - 2: `../domain/forge/branches.md`, 1, has a restart remake a missing goal branch in read-afresh; `skein/docs/design/forge/client.md`, 8, has the restart's reads before the settle with no rule against a read waiting on a write.
  - 3: `jig/docs/design/domain/model.md`, 4, has a restart read every connector afresh before deciding anything new; `jig/docs/design/domain/engine/restart.md` opens after every connector's settle; `jig/docs/design/domain/connectors.md`, 5.1, has no reconciliation after opening; `jig/docs/design/protocol/service.md`, 3, admits clients only once decisions open.
  - 4: `../domain/root.md`, 8.2, hears a stop only once the root is ready; `../../help/server.md` sets no unit timeout from the engine's bounds.
  - 5: no design names a start's or a reconciliation's waiters.
  - 6: `../protocol/binaries.md`, 2, ends `upgrade --check` at the switch; `../redeploy.md`, 4.1, passes the check there.
- **Design departs:** none.
- **Open:** none.
