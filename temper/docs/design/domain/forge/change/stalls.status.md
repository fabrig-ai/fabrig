# stalls: status

2026-11-16. Where the tree's code lags or departs from [stalls.md](stalls.md).

- **Not yet:** 1 and 3: a `Stalled` hold names neither the step nor what it waits on, and a goal's landing can sit at updating with no hold (`temper-engine-domain/src/route/forge_route.rs`, `hold_change_decision`; `temper-engine-domain-forge-change/src/domain.rs`, `updating`).
- **Not yet:** 2.2 and 2.3: the change policy counts the stall at checking from `since`, waiting and running alike; it has no clock, no reading, and neither the unknown-state grace nor the runner wait, and the change's record keeps no clock (`temper-engine-domain-forge-change/src/domain.rs`, `checking`; `temper-protocol-store`'s `record Change`) [in flight: change 4873]
- **Not yet:** 2.1, 2.4 and 3: the connector reads no run while CI is pending at checking and steps such a change only at its stall's deadline, and no hold says it waits for a runner (`temper-engine-domain-forge/src/domain.rs`, `step_read`, `change_facts`; `temper-engine-domain/src/route/forge_route/holds/change.rs`) [in flight: change 4875]
