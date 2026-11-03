# client: status

2026-11-03. Where jig's code lags or departs from [client.md](client.md).

- **Not yet:** 6: a turn read answered gone past its transcript's horizon: no retention drops a transcript (`store.md`, 11.1). 13: the public reads' limits. 14: the protocol world's stories over an in-process transport: `jig/tests/client-protocol` runs them through native HTTP bytes alone. 17.1, 17.5: the public reads' `GET`, its path's parser, its answers and their fixed headers, the shortened `max-age`, its room in the engine end, and the client end's `GET`. 19: `PublicRead`, its admission and reservations in the component, public identifiers and their key, declared kinds, `PublicApplication`, its tests and fuzzy tests.
- **Departs:** 17.4: the reference server hands a peer's end during a mounted call to the engine end as the call's `Ended` and does not end the connection itself (`jig/testing/jig-test-service/src/server.rs`), because the engine end takes `Ended` as the call's end, a stream's as its connection's (`jig/crates/jig-protocol-engine-clients-http/src/call.rs`).
