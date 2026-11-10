# delivery-checks: status

2026-11-10. Where the design lags or departs from [delivery-checks.md](delivery-checks.md).

- **Design lags:**
  - The draft's third ranked item: the coding charter's words still say
    to deliver the work it has before a question, not to `save` it. The
    charters' words are generated elsewhere, not in this plan.
  - The draft's third ranked item: an early start may merge a dependency's save, unchecked
    (`../domain/forge.status.md`, 8.11).
- **Design departs:** none.
- **Decided:** 3, by the owner, 2026-11-09: work in progress before a
  question is saved without a delivery, by a new tool, `save`, which pushes
  the tree to the change's own branch with no checks, marked unchecked in
  its commit and its receipt; `deliver` and a `finish` that delivers run the
  full check, and only a producer's checked finish is merged into the goal's
  branch. smith's charter (version 4) and channel (version 6) change in
  place, before v0.1.0; temper's store and the worker wires do not
  (smith's `domain/run.md`, 8.5; jig's `domain/hosts.md`, sections 7 to 9;
  `../domain/agent.md`, 4.3 and section 6; `../domain/worker.md`,
  section 5; `../protocol/git.md`, 4.7).
