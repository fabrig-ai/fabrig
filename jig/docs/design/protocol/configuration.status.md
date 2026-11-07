# configuration: status

2026-11-07. Where jig's code lags or departs from [configuration.md](configuration.md).

- **Not yet:** 3: a provider's `keep` and `drop`: refused as unknown members.
- **Not yet:** 3: retention's horizons in the deployment member: none is parsed. 3, 4, 7: the engine's own slots for agents in the engine, and TLS for the workers' listener and the worker's `engine` address: `jig/crates/jig-protocol-config/src/hosts.rs` and `address.rs` read plaintext and in-process addresses alone. 5: `ops`'s calendar child (`../examples.md`, section 4). 6: the people envelope diagnostics' words name `people-v13`, where the people family is v14 (`jig/crates/jig-protocol-config/src/problem.rs`, `domain.rs`).
- **Departs:** 6: the public index's memory check uses 128 bytes per line
  and a `u32` byte ceiling (`jig/crates/jig-protocol-config/src/clients.rs`,
  `public_index_bytes`), rather than the process's fit check, because
  configuration has no per-process estimate.
- **Departs:** 5: the reference service's first periods come from its engine's routing, which hands the core each `Period` (`jig/testing/jig-test-service/src/engine.rs`), not from a child of its root, because jig's test root composes no period child.
