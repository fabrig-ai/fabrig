# tools: status

2026-11-05. Where smith's code lags or departs from [tools.md](tools.md).

- **Not yet:** 2, 5: commands in the workspace's view, through skein's
  contained trees. A command runs with `sh` in its own process group
  (`smith-protocol-machine/src/process.rs`), with the agent user's
  permissions, so host absolute paths stay aliases (2).
- **Departs:** 5: `smith-protocol-machine/src/search.rs`, `parse`, drives
  a tokenizer over one whole `rg --json` line by hand, with its own step
  bound, where `skein/docs/design/data/json.md`, 5.3 gives one whole buffer to
  `parse` or a `Feed`, and the user's request of 11-04 asked that no step
  crate outside skein-json do so. No reason is recorded, and no debt item
  holds it.
