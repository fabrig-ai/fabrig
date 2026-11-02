# configuration: status

2026-11-02. Where jig's code lags or departs from [configuration.md](configuration.md).

- **Not yet:** 3: retention's horizons in the deployment member: none is parsed. 3, 4, 7: the engine's own slots for agents in the engine, and TLS for the workers' listener and the worker's `engine` address: `jig/crates/jig-protocol-config/src/hosts.rs` and `address.rs` read plaintext and in-process addresses alone. 5: the reference service's first periods come from its engine's routing (`jig/testing/jig-test-service/src/engine.rs`), not a child of its root; `ops`'s calendar child (`../examples.md`, section 4). 6: the envelope diagnostics' words name `people-v13`, where the people family is v14 (`jig/crates/jig-protocol-config/src/problem.rs`, `domain.rs`).
