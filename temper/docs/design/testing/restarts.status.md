# restarts: status

2026-11-21. Where temper's code lags or departs from [restarts.md](restarts.md).

- **Not yet:** 2: `temper-faults` and its `restarts` module don't exist. [in flight: change 11570]
- **Not yet:** 4, and 9's engine domain row: the engine's domain world restarts only once nothing is in flight, gives no stop at a drawn moment or in a drawn class of a start, never aborts a stop's settle, and keeps no fault ledger; `temper_faults::restarts::ENGINE` still holds `STOP_BEFORE_OPENING`. [in flight: change 11574]
- **Design departs:** 4, from the failure-worlds draft's 4.8: no world stops a start between the mark and the opening. Since 3f1a0ea3ff the start adopts its runs in the root input that takes the mark's answer and opens, so no deployment is stopped there; `STOP_BEFORE_OPENING` keeps its name in the kit and is in no sweep, the core's step test covers a stop at `AdoptRuns`, and `STOP_AFTER_OPENING` covers the forge's catch-up. No test hook enters product code. Decided by the drafter, 2026-11-22. Open, the restart drafts' owner's to design: what a stop does with the mark's commit stored and not yet answered.
- **Not yet:** 5, and 9's engine forge row: the engine's forge world draws no restart, outage, synthetic table or per-call script from the seed, and keeps no fault ledger. [in flight: change 11579]
- **Not yet:** 6, and 9's worker domain row: the worker's domain world stops a worker only before a turn is said or during a held push, never crashes one or brings it back as a new process, and keeps no fault ledger. [in flight: change 11582]
- **Not yet:** 7, and 9's system row: the system world restarts the engine mid-turn only at a scenario's chosen query, and keeps no fault ledger. [in flight: change 11591]
- **Not yet:** 8, and 9's one-process row: the one-process world never signals every process of a unit, starts a drawn run from dev's image, or keeps a fault ledger. [in flight: change 11594]
