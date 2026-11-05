# limits: status

2026-11-05. Where smith's code lags or departs from [limits.md](limits.md).

- **Departs:** 9: a derived limit is named by its layer and its field, but
  `ComponentLimits`' `translation` reports `adapter.tool_bytes` and
  `adapter.history_bytes`
  (`smith-agent-service/src/profile/effective/llm.rs`), because the names
  are output and the user's request of 2026-11-04 changed no behaviour
  (13).
