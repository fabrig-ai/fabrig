# host: status

2026-11-02. Where smith's code lags or departs from [host.md](host.md).

- **Not yet:**
  - 2, 9.1: live content in one process. The inline agent's LLM deltas
    never reach its owner: `smith-protocol-llm` hands deltas over only
    for a channel's ask, and `smith-local-service`'s `pop_live()` reads
    a spawned agent's alone.
  - 4: an agent spawned in a contained tree, in its view, and gone once
    the tree is proved empty: `smith-host-protocol/src/process.rs`
    spawns a plain child in a process group.
