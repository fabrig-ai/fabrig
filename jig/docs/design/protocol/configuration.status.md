# configuration: status

2026-11-21. Where jig's code lags or departs from [configuration.md](configuration.md).


- **Not yet:** 4: the `programs` row's credential source on a provider's `direct` endpoint (`../domain/accounts-sources.md`, 1) [in flight: change 928].

- **Departs:** 5: the reference service's first periods come from its engine's routing, which hands the core each `Period` (`jig/testing/jig-test-service/src/engine.rs`), not from a child of its root, because jig's test root composes no period child.

- **Not yet:** 4, 6: the engine's own slots for agents in the engine, and TLS for the workers' listener and the worker's `engine` address: `jig/crates/jig-protocol-config/src/hosts.rs` and `address.rs` read plaintext and in-process addresses alone. 5: `ops`'s calendar child (`../examples.md`, section 4).  the people-envelope diagnostic gap remains `limits.status.md` (Not yet, the whole document).

