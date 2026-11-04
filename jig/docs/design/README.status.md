# README: status

2026-11-02. Where jig's code lags or departs from [README.md](README.md).

- **Not yet:** 6.7, 9: agents in the engine: no root composes `smith-inline-agent`, and `ops`'s root holds the hub with no agent child (`jig/examples/ops/domain/src/domain.rs`). 10: a service's client root and the one-shot mode: no host or world runs either, though the client takes `With::Service` (`jig/crates/jig-client/src/client.rs`).
- **Departs:** 15: `jig-client-views-world` (`jig/tests/views`), the views child's world, runs in the engine's tests under a client crate's name; no reason is written.
