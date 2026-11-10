# testing: status

2026-11-10. Where the tree's code lags or departs from [testing.md](testing.md).

- **Not yet:**  a server's written head: `server.rs`'s `Responding::Head`
  derives `Debug` over the head it holds until room comes to send it, so a
  `Server` prints the `Set-Cookie` value jig's sign-in writes
  (`jig-protocol-engine-clients-http/src/call.rs`, `respond_with_cookie`).
  The request of 2026-11-05 named a server's `Call` and a client's written
  head, not this one, and no test formats a server holding it. Each machine's
  `Intake` also holds a head's bytes until their line is whole (../../runtime/lib.md, 6).
  the whole document: skein-json's selective collector on the protocol worlds' event data,
  their memory under scenario ceilings, and the fuzz targets.
