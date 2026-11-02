# hosts: status

2026-11-02. Where smith's code lags or departs from [hosts.md](hosts.md).

- **Not yet:**
  - 1, 3: the agent spawned in a contained tree with its view, its own
    trees inside it, and proved empty: `smith-host-protocol/src/process.rs`
    spawns a plain child with three pipes.
  - 1, 2: the withdrawn grant form's removal: the half still fills it
    (`smith-host-protocol/src/translate.rs`, `GrantValue::Withdrawn`).
