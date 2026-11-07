# limits: status

2026-11-07. Where smith's code lags or departs from [limits.md](limits.md).

Sections 2.1, 2.2, 3.9 and 4 have no known gap in endpoint limits,
`charter_models`, saved-window fallback quantities or entrance checks
against the endpoint rather than a model declaration.

- **Departs:** 9: a derived limit is named by its layer and its field, but
  `ComponentLimits`' `translation` reports `adapter.tool_bytes` and
  `adapter.history_bytes`
  (`smith-agent-service/src/profile/effective/llm.rs`), because the names
  are output and the user's request of 2026-11-04 changed no behaviour
  (13).
