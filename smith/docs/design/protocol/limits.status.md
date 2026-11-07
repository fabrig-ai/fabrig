# limits: status

2026-11-07. Where smith's code lags or departs from [limits.md](limits.md).

- **Not yet:** 3.10: `run_bytes` and `outcome_bytes` are fixed at 65,536 and 4,096 (`smith-domain-run/src/limits.rs`). 2.4: a host's input and answer stop at 65,536 (`smith-domain-run/src/host.rs`).
- **Departs:** 9: a derived limit is named by its layer and its field, but
  `ComponentLimits`' `translation` reports `adapter.tool_bytes` and
  `adapter.history_bytes`
  (`smith-agent-service/src/profile/effective/llm.rs`), because the names
  are output and the user's request of 2026-11-04 changed no behaviour
  (13).
