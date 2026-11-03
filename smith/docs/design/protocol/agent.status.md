# agent: status

2026-11-03. Where smith's code lags or departs from [agent.md](agent.md).

- **Not yet:**
  - 1, 3, 6, 8: the view. Commands, searches and checks run as plain
    children in process groups (`smith-protocol-machine/src/process.rs`),
    with no read-only git directory or machine and no writable scope;
    startup checks no machine for contained trees, and the agent says
    `containment: groups`.
