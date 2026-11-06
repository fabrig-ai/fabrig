# client: status

2026-11-06. Where jig's code lags or departs from [client.md](client.md).
Unlisted sections have no known gap.

- **Not yet:** 6: a turn read answered gone past its transcript's horizon: no retention drops a transcript (`store.md`, 11.1). 14: the protocol world's stories over an in-process transport: `jig/tests/client-protocol` runs them through native HTTP bytes alone.
- **Departs:** 6: a tree's `next` is a cursor no read takes: `jig/crates/jig-protocol-engine-clients/src/translate/watches.rs` writes it with `cursor::encode_tree`, in the form `Tree`; `cursor::decode` refuses that form, `cursor::decode_window`, which a family's window reads, takes a bare `DocumentCursor` (`jig/crates/jig-protocol-engine-clients/src/tests/reads.rs`), and a watch opens with no cursor at all, so a client that gives it back is refused. It stays because the request of 2026-11-05 held behaviour still while the cursor became a schema's record, and no debt item holds it. 17.4: the reference server, by its `PeerEnd::Call` (`jig/testing/jig-test-service/src/limits.rs`), leaves a peer's end during a mounted call to skein's server and does not end the connection itself (`jig/crates/jig-protocol-engine-clients-http/src/server.rs`): a request all read goes on to its response, the engine end hears nothing of that end, and a stream's end is seen at a failed write or the stall.
