# worlds: status

2026-11-11. Where the tree's code lags or departs from [worlds.md](worlds.md).

- **Not yet:** 1: the M2 store, build and system worlds on the complete fake
  machine. 3, 5: the build sandbox, sealed-layer and crash checks against
  that machine, including a cut between walk and record and
  `systemd-container`'s nested subvolume.  the cost checks remain `README.status.md`, 4.3.  these M2
  additions; the builder's domain tests use scripted layer operations.
- **Departs:** 4: a world's memory tests are modules of its suite, not a
  binary of their own, because a suite has one counting allocator
  (`skein/docs/design/testing/README.md`, 6).
