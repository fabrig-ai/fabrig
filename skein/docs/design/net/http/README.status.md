# http: status

2026-11-11. Where skein's code lags or departs from [http.md](http.md).

- **Not yet:** 2: a server's written head: `server.rs`'s `Responding::Head`
  derives `Debug` over the head it holds until room comes to send it, so a
  `Server` prints the `Set-Cookie` value jig's sign-in writes
  (`jig-protocol-engine-clients-http/src/call.rs`, `respond_with_cookie`).
  The request of 2026-11-05 named a server's `Call` and a client's written
  head, not this one, and no test formats a server holding it. Each machine's
  `Intake` also holds a head's bytes until their line is whole (../../runtime/lib.md, 6).
   the collector, memory and fuzz checks remain `testing.status.md` (Not yet, the whole document).
