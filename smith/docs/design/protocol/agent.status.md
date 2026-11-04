# agent: status

2026-11-04. Where smith's code lags or departs from [agent.md](agent.md).

- **Not yet:**
  - 1, 3, 6, 8: the view. Commands, searches and checks run as plain
    children in process groups (`smith-protocol-machine/src/process.rs`),
    with no read-only git directory or machine and no writable scope;
    startup checks no machine for contained trees, and the agent says
    `containment: groups`.
- **Departs:**
  - 6: `skein/docs/foundation/programming-model.md`, 5.4 asks for one
    explicit state enum per lifecycle, each state holding exactly its
    data, no lifecycle in flags or `Option`s beside it;
    `smith-agent-service/src/service.rs`, `Stages`, holds the run's end
    across `failure`, `final_failure`, `final_refusal`, `outcome`,
    `delivery`, `cancel_pending` and `root_close_pending`; no reason is
    recorded (the code's side is
    `smith/docs/development/debt/D002-lifecycles-in-flags.md`).
