# client: status

2026-11-04. Where jig's code lags or departs from [client.md](client.md).

- **Not yet:** 6: a turn read answered gone past its transcript's horizon: no retention drops a transcript (`store.md`, 11.1). 14: the protocol world's stories over an in-process transport: `jig/tests/client-protocol` runs them through native HTTP bytes alone.
- **Departs:** 17.4: the reference server, by its `PeerEnd::Call` (`jig/testing/jig-test-service/src/limits.rs`), leaves a peer's end during a mounted call to skein's server and does not end the connection itself (`jig/crates/jig-protocol-engine-clients-http/src/server.rs`): a request all read goes on to its response, the engine end hears nothing of that end, and a stream's end is seen at a failed write or the stall.
