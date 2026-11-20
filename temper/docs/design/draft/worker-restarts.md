# A worker restarts without losing its runs' work

Draft, 2026-11-20, on the user's word. Their aim: a stopping worker lets the LLM turns and tool calls in flight settle, then stops; restarted, its agents carry on where they left off, and the engine agrees, as if nothing happened. They don't want a full drain, which takes long and leaves the builder idle.

Between 11-15 and 11-19, every stop of a dev.fabrig.ai worker cut every run on it: 32 shutdown endings across 23 tasks in one rolling restart, 12 in another, and all 11 runs, 7 of them in delivery checks, at the 11-19 upgrade (P123). The drain of `redeploys.md`, 6.1, was built and deployed, and never ran: the worker's unit stops by signalling every process in it, and an agent cancels on its own copy of the signal before the worker's drain can reach it.

## 1. In one page

- **The stop is the product's** (section 2): the engine asks a worker to drain over its link, which also gives a switch for no new work. A shipped unit signals the worker alone, and an agent whose host is alive leaves its stop to its host.
- **A run settles at its LLM answer, not at its whole turn** (section 3): the answer in flight finishes within a bound and is committed as soon as it arrives; the calls it made are cut after a short grace.
- **The run's place is kept** (section 4): its workspace, build outputs and check results stay on its worker for that run alone, and the engine sends it back there within a window.
- **A cut run is suspended, not failed** (section 5): no try, no clock and no blame.
- **On resume, the agent runs again the calls left pending** (section 6): code reruns checks and calls whose effect the engine records; any other call goes back to the model as interrupted, for it to judge.
- **A crash takes the same path** (section 7), without the settle, so every redeploy exercises crash recovery.
- **Settings apply live where that's safe** (section 8), so a settings repair stops costing a restart.
- **Tested in memory** (section 9): a worker stopped or crashed at every point of a turn, with the same outcomes as the same seed with no stop.

## 2. The stop is the product's

- **Today:**
  - temper's help gives a worker's unit `KillMode=control-group` (`temper/docs/help/server.md`, section 5). On a stop, systemd signals every process in the unit's cgroup.
  - The worker spawns its agents, their programs, shell calls and check programs in process groups of their own, inside its cgroup (`skein/crates/skein-shell/src/linux/process.rs`).
  - The worker drains on its first SIGTERM (`temper/crates/temper-worker-service/src/worker_alone.rs`). A live agent cancels on its own (`smith/crates/smith-agent-service/src/service/teardown.rs`, `shutdown`, and its routing), and a test pins that. Check programs die at once.
  - So a stop ends every run within seconds. The 3-minute drain never binds, and neither does the unit's 360 s.
  - Nothing tells a worker to take no new work short of stopping it.
- **Wanted:**
  - **The engine asks a worker to drain, over its link.** A ctl verb names the worker. The worker takes no new run or job and drains as `redeploys.md`, 6.1, has it, then says it's drained. The operator, or the deployment's redeploy tool, stops the process only then.
  - **A worker can be told to take no new work** and to take it again, without a stop. This is the same request, without the drain.
  - **A shipped unit signals the worker alone:** `KillMode=mixed`, with `TimeoutStopSec` taken from the worker's printed stop bound. A stray signal then reaches only the worker, which drains.
  - **An agent with a live host leaves its stop to the host.** A termination signal while its channel is open is a request to the host, not a cancel. It cancels on its own only once the channel is gone.
  - **One test with real processes** stops a worker as its unit does and checks that the turn in flight settles.
  - Revise `temper/docs/design/domain/worker/README.md`, 2, `jig/docs/design/domain/hosts/hub.md`, 7, `smith/docs/design/domain/host.md` (a signal with a live host), `temper/docs/help/server.md` and the ctl verbs' design, with their status files.

## 3. A run settles at its LLM answer, not at its whole turn

- **Today:**
  - A drain waits for a turn to end, and a turn ends once its LLM answer and every call it made have answered, a sub-agent's whole conversation and a check included (`smith/docs/design/domain/run.md`, the turn's end). Sub-agents' turns never reach the host's transcript.
  - So a 3-minute bound saves the cheap part of a turn and cuts the expensive part: a 30-minute check outlives a stop only when less than 3 minutes of it are left.
- **Wanted:**
  - **The LLM's answer is committed when it arrives,** before its calls run. This is a new record on the turns' wire, an added kind its bridge carries (the user's rule of 2026-11-17), not a breaking change.
  - **Each call's answer is committed when it ends.** A turn's end stays what it is today: the last of its calls answered.
  - **A drain waits for the answer in flight,** within its bound, by default 3 minutes. The calls in flight are cut after a short grace, by default 10 seconds, so that one about to end can.
  - Revise `smith/docs/design/domain/run.md` (a turn's records), `smith/docs/design/protocol/transcript.md` and `temper/docs/design/draft/redeploys.md`, 6.1 and 6.2, with their status files.

## 4. The run's place is kept

- **Today:**
  - A workspace isn't kept across a restart (`redeploys.md`, 6.3), since an agent can write a workspace's `.git` and so reach later workstreams.
  - The next attempt starts from the saved-work branch on any worker. Ignored files and build outputs are lost, and a check starts again from its first check (`smith/crates/smith-domain-run/src/land.rs`).
- **Wanted:**
  - **A cut run's workspace is kept on its worker for that run alone:** its tree, its build outputs and its checks' results directory. It is never another workstream's, so `redeploys.md`, 6.3's concern doesn't apply. It goes when the run ends, or when the worker needs the room, oldest first.
  - **The engine sends the resumed attempt to the same worker** while that worker is back within a window, by default the announced restart wait (`redeploys.md`, 5.3). Otherwise it goes to any worker, from the saved-work branch, as today.
  - **Check results are the repository's check program's,** keyed by the tree: smith gives it a results directory that survives the restart, and temper and smith stay generic.
  - Revise `temper/docs/design/domain/worker/checkouts.md` (a kept workspace), `jig/docs/design/domain/engine/executors.md` (placing a resumed attempt) and `redeploys.md`, 6.3, with their status files.

## 5. A cut run is suspended, not failed

- **Today:** a shutdown spends no try (`jig/crates/jig-domain-tasks/src/run.rs`), but its activity records the end as refused and reads "failed Cancelled(Shutdown)". Deadlines and no-progress timers count the gap, and nothing prefers the worker that kept the workspace.
- **Wanted:**
  - **An attempt cut by a drain or a restart is suspended:** recorded and shown as suspended, with no try spent.
  - **No clock counts the gap:** not the task's deadline, a stall's or no-progress timer, nor a check's time budget.
  - **No repair is asked** for work a restart cut. That is the same rule as for a CI runner's stop, which the shared failure worlds check, by the goal-execution draft's rule that blame matches the injected cause.
  - Revise `jig/docs/design/domain/engine/executors.md` and jig's tasks design for the suspended end, with their status files.

## 6. On resume, the agent runs again the calls left pending

- **Today:** the next attempt resumes the conversation from its last committed turn, with the calls committed since given as a note (`jig/docs/design/domain/engine/executors.md`, 1). It doesn't know it was cut, plans the cut turn again, and may choose differently.
- **Wanted:**
  - **The agent, not the worker, runs the pending calls again,** since the worker interprets no tool names (`temper/docs/design/domain/worker/README.md`, 1).
  - **Code reruns what is known to be repeatable:** a check, and a host call whose effect the engine records. Its answer reaches the model as if the call had run once.
  - **Any other call goes back to the model** as "interrupted by a worker restart after N s", with its output so far, for the model to judge whether to run it again: LLM for judgement.
  - **A sub-agent's conversation resumes** as its parent's does, from its own committed answers.
  - Revise `smith/docs/design/domain/run.md` (a resumed turn) and `smith/docs/design/domain/host.md`, with their status files.

## 7. A crash takes the same path

- **Today:** a kill is a crash. The workspace's changes since its last save are lost, and so is the attempt (`redeploys.md`, 6.1).
- **Wanted:**
  - **A crash is a stop without its settle:** sections 4 to 6 hold, and the workspace kept on disk is saved at the worker's next start.
  - **A worker back as a new process** has the engine count the attempts it no longer lists as suspended at once, not after its grace (`redeploys.md`, 12, which this draft brings forward).
  - So every planned restart exercises the crash path, and a crash loses only what wasn't committed.

## 8. Settings apply live where that's safe

- **Today:** a worker's figures apply only at its start (`temper/docs/design/protocol/configuration/figures.md`). Every repair of one is a restart, and with section 2 unbuilt a restart cuts every run: the Git timeouts were restored twice that way in three days.
- **Wanted:**
  - **Timeouts, deadlines and bounds apply live:** at the next operation that reads them.
  - **A restart is needed only for what resizes a table or re-pins the agent** (`redeploys.md`, 4.4), and the setting says which it is.
  - Revise `figures.md` and `temper/docs/design/protocol/configuration/limit-settings.md`, with their status files.

## 9. Tests

Focused and fuzzy suites only, within their budgets. Every case is a world below the real loop, but the one test of section 2 with real processes.

- **Focused,** in `temper/tests/worker/domain/`, `jig/tests/channel/` and `smith/tests/run/`:
  1. A drain asked over the link while a run waits for its LLM answer: the answer is committed, the run suspends, no try is spent, and it resumes on the same worker with its conversation and workspace.
  2. The same while a delivery check runs: the check is cut, and on resume it runs again, its passed tests skipped from its results directory.
  3. The same while a shell call runs: on resume the model reads "interrupted by a worker restart" with the output so far.
  4. A crash at each of those points: the same outcomes, and the workspace is saved at the next start.
  5. A worker back as a new process: its attempts are suspended at once.
  6. A signal to an agent while its host lives: it waits for its host's word.
  7. A timeout setting changed while runs work: the next operation uses it, and no run is cut.
  8. With real processes: a worker stopped as its unit does settles its turn in flight.
- **Fuzzy:** in the shared failure worlds of the goal-execution draft, once they land, a worker stopped or crashed at any iteration of a random workload, with their checks: no try spent for a stop, every goal goes on within a bound, and the same seed with and without the stops reaches the same outcomes.

## 10. Out

- **When to stop a worker, and in what order:** the deployment's redeploy tool, outside this repository. It drains one worker at a time, and restarts the engine alone by default (`redeploys.md`, 7.1 and 7.2).
- **Agents outliving their worker,** adopted by its next start: still later (`redeploys.md`, 4.4 and 12), by the user's word of 2026-11-12.

## 11. Later

- A run moved to another worker with its workspace, not just its branch.
