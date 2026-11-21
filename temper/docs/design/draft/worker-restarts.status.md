# worker-restarts: status

2026-11-20. Where the design lags or departs from [worker-restarts.md](worker-restarts.md).

- **Design lags:**
  - 3: `smith/docs/design/domain/run.md` and `smith/docs/design/protocol/transcript.md` commit a turn only whole; `redeploys.md`, 6.1, waits for the whole turn.
  - 4: `redeploys.md`, 6.3, keeps no workspace across a restart.
  - 6: `smith/docs/design/domain/run.md` resumes from the last whole turn, with no pending calls.
  - 7: `redeploys.md`, 6.1, has a kill lose the workspace's changes since its last save, and no design saves a kept workspace at the worker's next start.
- **Design departs:**
  - 4: the window a resumed attempt waits for its worker is the restart wait itself, `fleet.restart_wait` (`jig/docs/design/domain/engine/executors.md`, 6.4), with no setting of its own defaulting to it: one figure serves both, as `redeploys.md`, 12, has the restart wait serve every worker. The goal's designer, 2026-11-20.
  - 7: a task whose previous attempt was suspended by its worker coming back as a new process has its next such end counted lost, spending a try (`jig/docs/design/domain/engine/executors.md`, 6.1), so that a run that brings its worker down is not tried for ever. The goal's designer, 2026-11-20.
  - 8: `link.ping_ms` and `link.silence_ms` wait for a worker's next start, though they are timeouts, and so does the engine's own copy of every time bound but the link's (`../protocol/configuration/limit-settings.md`, 5.2 and 11). Ping and silence make the stop bound a worker's hello states, which the engine checks against its grace and holds runs by (`redeploys.md`, 5.3), so a running link can't take them without a new opening; the engine's children copy their own time bounds at construction. The draft's aim, a restart only where one is needed, holds: every time bound a worker reads but those two is live. Decided by the goal's design, 2026-11-20.
  - 9, test 7: in `temper-worker-service`'s own tests, not `temper/tests/worker/domain/`, since a worker's figures reach its service, which the domain world below it doesn't hold (`../protocol/configuration/figures.md`, 9). Decided by the goal's design, 2026-11-20.
  - 9: test 6, a signal to an agent while its host lives, is in smith's agent-process world (`smith/tests/agent-process/`), where the agent's signals are tested, rather than `smith/tests/run/`; and test 8, with real processes, is a story of temper's process tier (`temper/bin/temper-box/tests/process.rs`), whose stories need real signals, so a plan writes and compiles it and it runs after landing (`AGENTS.md`, section 5). The designer, 2026-11-20.
  - 2: the ctl verbs are `worker pause`, `worker drain` and `worker resume` (`../client/ctl/workers.md`), and a pause is the engine's placement as well as the worker's, so a worker of an older build is kept from new work though it cannot drain (jig's `domain/engine/fleet.md`). The designer, 2026-11-20.
- **Open:** none.
