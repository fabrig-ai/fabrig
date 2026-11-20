# worker-restarts: status

2026-11-20. Where the design lags or departs from [worker-restarts.md](worker-restarts.md).

- **Design lags:**
  - 2: `../help/server.md`, 5, gives the worker's unit `KillMode=control-group`; `../domain/worker/README.md`, 2, and `jig/docs/design/domain/hosts/hub.md`, 7, start a drain only on a signal; `smith/docs/design/domain/host.md` has an agent cancel on its own signal.
  - 3: `smith/docs/design/domain/run.md` and `smith/docs/design/protocol/transcript.md` commit a turn only whole; `redeploys.md`, 6.1, waits for the whole turn.
  - 4: `redeploys.md`, 6.3, keeps no workspace across a restart.
  - 6: `smith/docs/design/domain/run.md` resumes from the last whole turn, with no pending calls.
  - 7: `redeploys.md`, 6.1, has a kill lose the workspace's changes since its last save, and no design saves a kept workspace at the worker's next start.
  - 8: `../protocol/configuration/figures.md` applies a worker's figures at its start only.
- **Design departs:**
  - 4: the window a resumed attempt waits for its worker is the restart wait itself, `fleet.restart_wait` (`jig/docs/design/domain/engine/executors.md`, 6.4), with no setting of its own defaulting to it: one figure serves both, as `redeploys.md`, 12, has the restart wait serve every worker. The goal's designer, 2026-11-20.
  - 7: a task whose previous attempt was suspended by its worker coming back as a new process has its next such end counted lost, spending a try (`jig/docs/design/domain/engine/executors.md`, 6.1), so that a run that brings its worker down is not tried for ever. The goal's designer, 2026-11-20.
- **Open:** none.
