# worker-restarts: status

2026-11-20. Where the design lags or departs from [worker-restarts.md](worker-restarts.md).

- **Design lags:**
  - 2: `../help/server.md`, 5, gives the worker's unit `KillMode=control-group`; `../domain/worker/README.md`, 2, and `jig/docs/design/domain/hosts/hub.md`, 7, start a drain only on a signal; `smith/docs/design/domain/host.md` has an agent cancel on its own signal.
  - 3: `smith/docs/design/domain/run.md` and `smith/docs/design/protocol/transcript.md` commit a turn only whole; `redeploys.md`, 6.1, waits for the whole turn.
  - 4: `redeploys.md`, 6.3, keeps no workspace across a restart.
  - 5: `jig/docs/design/domain/engine/executors.md` has no suspended end.
  - 6: `smith/docs/design/domain/run.md` resumes from the last whole turn, with no pending calls.
  - 7: `redeploys.md`, 12, leaves counting a returned worker's missing attempts for later.
  - 8: `../protocol/configuration/figures.md` applies a worker's figures at its start only.
- **Design departs:** none.
- **Open:** none.
