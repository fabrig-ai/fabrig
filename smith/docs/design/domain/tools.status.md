# tools: status

2026-11-02. Where smith's code lags or departs from [tools.md](tools.md).

- **Not yet:** 2, 5: commands in the workspace's view, through skein's
  contained trees. A command runs with `sh` in its own process group
  (`smith-protocol-machine/src/process.rs`), with the agent user's
  permissions, so host absolute paths stay aliases (2).
