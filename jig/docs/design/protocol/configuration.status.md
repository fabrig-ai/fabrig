# configuration: status

2026-11-07. Where jig's code lags or departs from [configuration.md](configuration.md).

Section 3 has no known gap in passed model defaults, refusing document
model tables under those defaults, provider endpoints or charter `rest`.
Section 4's agent configuration declares endpoint limits, without models.
Section 3's provider-list keep and drop patterns and their bounds have no
known implementation gap.

Section 3's list configuration is optional: no keep means no read, and
keep requires an endpoint. Invalid patterns are refused at their path.

Section 6 has no known gap in standard's `authority.batch` of 32,
`hub.charter_bytes` of 262,144 and `hub.outcome_bytes` of 131,072.
Small keeps its charter and outcome figures. The larger batch sizes
aggregate bounds without reserving their capacity ahead.

- **Not yet:** 3: retention's horizons in the deployment member: none is parsed. 3, 4, 7: the engine's own slots for agents in the engine, and TLS for the workers' listener and the worker's `engine` address: `jig/crates/jig-protocol-config/src/hosts.rs` and `address.rs` read plaintext and in-process addresses alone. 5: `ops`'s calendar child (`../examples.md`, section 4). 6: the people envelope diagnostics' words name `people-v13`, where the people family is v14 (`jig/crates/jig-protocol-config/src/problem.rs`, `domain.rs`).
- **Not yet:** 3: the clients' `approval` member.
- **Departs:** 6: the public index's memory check uses 128 bytes per line
  and a `u32` byte ceiling (`jig/crates/jig-protocol-config/src/clients.rs`,
  `public_index_bytes`), rather than the process's fit check, because
  configuration has no per-process estimate.
- **Departs:** 5: the reference service's first periods come from its engine's routing, which hands the core each `Period` (`jig/testing/jig-test-service/src/engine.rs`), not from a child of its root, because jig's test root composes no period child.
