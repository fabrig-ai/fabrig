# default-http: status

2026-11-11. Where the tree's code lags or departs from [default-http.md](default-http.md).

- **Departs:**  the tree-cursor refusal remains `README.status.md`, 4. 4: the reference server, by its `PeerEnd::Call` (`jig/testing/jig-test-service/src/limits.rs`), leaves a peer's end during a mounted call to skein's server and does not end the connection itself (`jig/crates/jig-protocol-engine-clients-http/src/server.rs`): a request all read goes on to its response, the engine end hears nothing of that end, a stream's end is seen at a failed write or the stall, and an exchange left unanswered says `ExchangeGone` (`wire.md`, 2.2) only at the stall. The engine-clients world plays the section's end in `jig/tests/engine-clients/tests/suite/gone.rs` alone, whose server it builds with `PeerEnd::Connection`, as temper's (`temper/crates/temper-service/src/limits.rs`).

`wire.md`, sections 2.1 and 2.2 and `sign-in.md` and `default-http.md`, sections 1 and 3 have the approval's ask, read,
decision and take, its four HTTP routes and cookie-only browser mark.
The component holds four live approvals for ten minutes and remembers
eight ended approvals. Its tables and joint-client stories cover the
ends, first decision, one credential, the approving party, a bearer
refused and an approval never taken making no sign-in. `README.md`, section 12 keeps
approval between hosts and the person's list of sign-ins deferred.
