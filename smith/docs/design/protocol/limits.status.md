# limits: status

2026-11-07. Where smith's code lags or departs from [limits.md](limits.md).

- **Not yet:** 2.4: a host's answer stops at 65,536 rather than 262,144
  bytes (`smith-domain-run/src/host.rs`, `HostAnswer::CAPACITY`).
- **Departs:** 2.4: a host's input admits up to 1,048,576 rather than
  262,144 bytes (`smith-domain-run/src/host.rs`, `HostInput::CAPACITY`),
  matching the channel's `host_ask_input` and transcript's `call_input`
  ceilings; the deployment's receiving limit still applies.
- **Departs:** 9: a derived limit is named by its layer and its field, but
  `ComponentLimits`' `translation` reports `adapter.tool_bytes` and
  `adapter.history_bytes`
  (`smith-agent-service/src/profile/effective/llm.rs`), because the names
  are output and the user's request of 2026-11-04 changed no behaviour
  (13).
