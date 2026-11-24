# landing-waits: status

2026-11-24. Where temper's code lags or departs from [landing-waits.md](landing-waits.md).

- **Not yet:** 3 (*A wait for a fresh read is bounded*): a run of `read`
  waits has no clock and no bound, and no `read` hold
  (`temper-engine-domain-forge/src/landing.rs`, `wait`) [in flight:
  change 15531].
- **Not yet:** 4: the engine's three lines for a run of read waits, and
  `landing show`'s `read` hold [in flight: change 15531].
