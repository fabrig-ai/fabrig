# redeploys: status

2026-11-12. Where the design lags or departs from [redeploys.md](redeploys.md).

Sections 3.1, 3.2, 3.3's interim, 3.4 and 8 are designed (`../protocol/wires.md`; jig's `protocol/channel/wires.md`; skein's `data/codec/command.md`, 2, the wire rule).

- **Design lags:**
  - 3.2, 4.1: the engine states nothing back to the worker: with each build's floor its current, an accepted channel means every wire at both sides' one version (`../protocol/wires.md`, section 11; jig's `protocol/channel/wires.md`, section 8).
  - 3.3, 4.1, 4.5: skein's `data/codec/command.md` writes forward patches alone, with no floor kept in the repository and no breaking declaration; each build's floor is its current version (`../protocol/wires.md`, 3.2).
  - 4.2: `../protocol/channel.md`, section 6 encodes a git identity's value once, for every worker; a run's charter names no tools' version, and the engine answers only its current host tools' contract (`../protocol/wires.md`, section 6).
  - 4.3: the fleet places by no agreed version: every agreed version is the current.
  - 4.4: the pieces of smith's channel the engine keeps are versioned as `smith-channel` itself, not as wires of their own (`../protocol/wires.md`, section 2); `../domain/worker/README.md`, section 5 and `../protocol/binaries.md`, section 5 start the agent the configuration names, read again at each spawn.
  - 5.1: `../protocol/binaries.md`'s `upgrade --check` compares the stored wires' names and versions, not each worker's ranges, fingerprints and terms.
  - 6.1: jig's `domain/hosts/hub.md`, 7 cancels every run at once, and a job cancelled by its worker's shutdown ends lost.
  - 6.4: `../domain/worker/checkouts.md` admits every preparation at once.
  - 9: jig's `domain/tasks/lifecycle.md`, 6 counts a preparation's transient failure against the task's tries.
- **Design departs:**
  - 3.1, 4.2: a job kind's request and result are no wire between temper's builds: its spec is `temper-worker-wire`'s `JobSpec`, and its request and result files are the project's tooling contract, which the worker carries unread; a kind's meaning to a worker is fixed by its number (`../protocol/wires.md`, section 7). Because `../domain/worker/jobs.md` has the worker never read them, so no build of temper's reads another's; the goal of 2026-11-12 decided it.
  - 3.1: the host tools' contract's layout is each declaration's name, effect and input schema; what a tool means, and its description, move the version by its author's hand (`../protocol/wires.md`, section 6). Because a description's wording is no layout a gate can judge; the goal of 2026-11-12 decided it.
- **Open:** none.
