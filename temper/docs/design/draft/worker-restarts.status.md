# worker-restarts: status

2026-11-20. Where the design lags or departs from [worker-restarts.md](worker-restarts.md).

- **Design lags:**
  - 2: `../help/server.md`, 5, gives the worker's unit `KillMode=control-group`; `../domain/worker/README.md`, 2, and `jig/docs/design/domain/hosts/hub.md`, 7, start a drain only on a signal; `smith/docs/design/domain/host.md` has an agent cancel on its own signal.
  - 3: `smith/docs/design/domain/run.md` and `smith/docs/design/protocol/transcript.md` commit a turn only whole; `redeploys.md`, 6.1, waits for the whole turn.
  - 4: `redeploys.md`, 6.3, keeps no workspace across a restart.
  - 5: `jig/docs/design/domain/engine/executors.md` has no suspended end.
  - 6: `smith/docs/design/domain/run.md` resumes from the last whole turn, with no pending calls.
  - 7: `redeploys.md`, 12, leaves counting a returned worker's missing attempts for later.
- **Design departs:**
  - 8: `link.ping_ms` and `link.silence_ms` wait for a worker's next start, though they are timeouts, and so does the engine's own copy of every time bound but the link's (`../protocol/configuration/limit-settings.md`, 5.2 and 11). Ping and silence make the stop bound a worker's hello states, which the engine checks against its grace and holds runs by (`redeploys.md`, 5.3), so a running link can't take them without a new opening; the engine's children copy their own time bounds at construction. The draft's aim, a restart only where one is needed, holds: every time bound a worker reads but those two is live. Decided by the goal's design, 2026-11-20.
  - 9, test 7: in `temper-worker-service`'s own tests, not `temper/tests/worker/domain/`, since a worker's figures reach its service, which the domain world below it doesn't hold (`../protocol/configuration/figures.md`, 9). Decided by the goal's design, 2026-11-20.
- **Open:** none.
