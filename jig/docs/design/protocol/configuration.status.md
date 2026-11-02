# configuration: status

2026-11-02. Where jig's code lags or departs from [configuration.md](configuration.md).

- **Not yet:** 3: retention's horizons in the deployment member: none is parsed. 3, 3.1, 10: a charter's default wake policy, its `wake` member: `jig-protocol-config` reads none. 3, 4, 7: the engine's own slots for agents in the engine, and TLS for the workers' listener and the worker's `engine` address: `jig/crates/jig-protocol-config/src/hosts.rs` and `address.rs` read plaintext and in-process addresses alone. 5: `ops`'s calendar child (`../examples.md`, section 4). 6: the envelope diagnostics' words name `people-v13`, where the people family is v14 (`jig/crates/jig-protocol-config/src/problem.rs`, `domain.rs`).
- **Departs:** 5: the reference service's first periods come from its engine's routing, which hands the core each `Period` (`jig/testing/jig-test-service/src/engine.rs`), not from a child of its root, because jig's test root composes no period child.
