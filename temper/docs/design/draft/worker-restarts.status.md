# worker-restarts: status

2026-11-20. Where the design lags or departs from [worker-restarts.md](worker-restarts.md).

- **Design lags:**
  - 3: `smith/docs/design/domain/run.md` and `smith/docs/design/protocol/transcript.md` commit a turn only whole; `redeploys.md`, 6.1, waits for the whole turn.
  - 4: `redeploys.md`, 6.3, keeps no workspace across a restart.
  - 5: `jig/docs/design/domain/engine/executors.md` has no suspended end.
  - 6: `smith/docs/design/domain/run.md` resumes from the last whole turn, with no pending calls.
  - 7: `redeploys.md`, 12, leaves counting a returned worker's missing attempts for later.
  - 8: `../protocol/configuration/figures.md` applies a worker's figures at its start only.
- **Design departs:**
  - 9: test 6, a signal to an agent while its host lives, is in smith's agent-process world (`smith/tests/agent-process/`), where the agent's signals are tested, rather than `smith/tests/run/`; and test 8, with real processes, is a story of temper's process tier (`temper/bin/temper-box/tests/process.rs`), whose stories need real signals, so a plan writes and compiles it and it runs after landing (`AGENTS.md`, section 5). The designer, 2026-11-20.
  - 2: the ctl verbs are `worker pause`, `worker drain` and `worker resume` (`../client/ctl/workers.md`), and a pause is the engine's placement as well as the worker's, so a worker of an older build is kept from new work though it cannot drain (jig's `domain/engine/fleet.md`). The designer, 2026-11-20.
- **Open:** none.
