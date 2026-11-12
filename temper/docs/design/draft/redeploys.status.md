# redeploys: status

2026-11-12. Where the design lags or departs from [redeploys.md](redeploys.md).

Sections 6.3 and 6.4 are designed, in `../domain/worker/checkouts.md`: stores kept across a restart and made once, blob-less, workspaces not kept, and preparations a few at a time, `checkout.preparations` of them, 1 by default. Nothing else of this draft is designed or built yet.

- **Design lags:**
  - 3.1, 8: jig's `protocol/README.md`, section 7, jig's `protocol/channel/README.md`, 4.1, smith's `protocol/README.md`, section 8, skein's `data/codec/README.md`, section 5, and `../protocol/README.md`, section 7 still change the wires in place before v0.1.0, keep no older version and translate nothing. `../protocol/channel.md`, section 3 versions temper's payloads by one number; `../domain/worker/jobs.md` gives jobs' requests and results no version; the host tools have no versioned contract.
  - 3.2: jig's `protocol/channel/README.md`, 4.1 states terms from the current codec, and its kinds' table is outside any schema.
  - 3.2, 5.1: `../protocol/binaries.md` has `versions` print the wires' numbers alone, and `upgrade --check` compare them.
  - 3.4: jig's `domain/tasks/lifecycle.md`, 2 makes an assignment its host found invalid a permanent failure, whatever the cause.
  - 4.1: skein's `data/codec/command.md`, section 3 writes forward patches alone, and no floor.
  - 4.2: `../protocol/channel.md`, section 6 encodes a git identity's value once, for every worker.
  - 4.4: `../domain/worker/README.md`, section 5 and `../protocol/binaries.md`, section 5 start the agent the configuration names, read again at each spawn.
  - 6.1: jig's `domain/hosts/hub.md`, 7 cancels every run at once, and a job cancelled by its worker's shutdown ends lost.
  - 9: jig's `domain/tasks/lifecycle.md`, 2 counts a preparation's transient failure against the task's tries.
- **Design departs:** none yet.
- **Open:** none.
