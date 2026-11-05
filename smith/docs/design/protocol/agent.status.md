# agent: status

2026-11-04. Where smith's code lags or departs from [agent.md](agent.md).

- **Not yet:**
  - 1, 3, 6, 8: the view. Commands, searches and checks run as plain
    children in process groups (`smith-protocol-machine/src/process.rs`),
    with no read-only git directory or machine and no writable scope;
    startup checks no machine for contained trees, and the agent says
    `containment: groups`.
- **Departs:** 6: a host may have the shell write each model request and what
  a program was given to a private directory (`smith-agent-shell/src/dump.rs`),
  with its lines also in `dump.log` there, because a development deployment
  must read what a run was sent; it comes out when that need ends. A write
  blocks the shell's pass, and never fails a run. Unasked, nothing is kept.
