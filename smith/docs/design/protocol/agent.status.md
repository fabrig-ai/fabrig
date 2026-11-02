# agent: status

2026-11-02. Where smith's code lags or departs from [agent.md](agent.md).

- **Not yet:**
  - 1, 3, 6, 8: the view. Commands, searches and checks run as plain
    children in process groups (`smith-protocol-machine/src/process.rs`),
    with no read-only git directory or machine and no writable scope;
    startup checks no machine for contained trees, and the agent says
    `containment: groups`.
  - 4: the withdrawn forms' removal. The `codex` dialect (number 1), an
    endpoint's `account` and its identity profile are still accepted
    (`smith-agent-shell/src/config/document.rs`). jig's configuration
    takes any dialect number (`jig-protocol-config/src/llm.rs`) and its
    worlds write 1.
