# client: status

2026-11-02. Where jig's code lags or departs from [client.md](client.md).

- **Not yet:** 6: a turn read answered gone past its transcript's horizon: no retention drops a transcript (`store.md`, 11.1). 14: the protocol world's stories over an in-process transport: `jig/tests/client-protocol` runs them through native HTTP bytes alone. 17.4: the reference server hands a peer's end during a mounted call to the engine end as the call's `Ended`, which ends the call, and does not end the connection itself (`jig/testing/jig-test-service/src/server.rs`).
- **Departs:** 3: the wire still carries `TaskWaitRefused::program` (`TaskWaitRefusedProgram`, `TaskWaitProgram` and `TaskWaitProgramSource` in `client-v19.schema`), which nothing produces and the engine translates (`jig/crates/jig-protocol-engine-clients/src/translate/task_wait.rs`), because the design dropped the arm (the user, 2026-11-02) and a code task removes it with its phrases.
