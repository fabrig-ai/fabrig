# client: status

2026-11-11. Where jig's code lags or departs from [README.md](README.md).
Unlisted sections have no known gap.

Sections 10 and 11's transport accounting is present: emitted exchanges
keep credit through `Reply` or `Lost`, including cancelled watch calls.
The stream table holds authorization behind a full fetch table, then
releases it after successful or failed HTTP terminals, with no duplicate
watch ending.

- **Design departs:** 8, from the request `ui-small-1` of 2026-11-06,
  item 1's unconditional used-link explanation: after a restart or eviction
  from the last 8 remembered links, a used link answers `link_unknown`.
  The plan chose bounded memory rather than a stored history; its direction
  to use the current terminal's link still holds.

- **Not yet:** 10, *A watch's opening and closing wait for credit*: `jig-client-protocol-link` ends an opening that finds every exchange in flight busy, and drops a closing. [in flight: change 997]
- **Not yet:** 4: a turn read answered gone past its transcript's horizon: no retention drops a transcript (`../store/README.md`, 6.1). 11: the protocol world's stories over an in-process transport: `jig/tests/client-protocol` runs them through native HTTP bytes alone.
- **Departs:** 8, *Never taken, nothing made*: a take admitted before
  expiry remains reserved until the core answers, even past the approval's
  expiry. `jig-protocol-engine-clients/src/approval.rs`, `State::sweep`
  and `next_deadline`, exclude `Decision::Taking`; `approval_whose` and
  `approval_reply` do not check the expiry again. The implementation
  explicitly preserves the reservation as a printed link's does, rather
  than enforcing the section's unconditional expiry. Seen by reading;
  no runtime result establishes this boundary.
- **Departs:** 4: a tree's `next` is a cursor no read takes: `jig/crates/jig-protocol-engine-clients/src/translate/watches.rs` writes it with `cursor::encode_tree`, in the form `Tree`; `cursor::decode` refuses that form, `cursor::decode_window`, which a family's window reads, takes a bare `DocumentCursor` (`jig/crates/jig-protocol-engine-clients/src/tests/reads.rs`), and a watch opens with no cursor at all, so a client that gives it back is refused. It stays because the request of 2026-11-05 held behaviour still while the cursor became a schema's record, and no debt item holds it.  the reference server, by its `PeerEnd::Call` (`jig/testing/jig-test-service/src/limits.rs`), leaves a peer's end during a mounted call to skein's server and does not end the connection itself (`jig/crates/jig-protocol-engine-clients-http/src/server.rs`): a request all read goes on to its response, the engine end hears nothing of that end, a stream's end is seen at a failed write or the stall, and an exchange left unanswered says `ExchangeGone` (`wire.md`, 2.2) only at the stall. The engine-clients world plays the section's end in `jig/tests/engine-clients/tests/suite/gone.rs` alone, whose server it builds with `PeerEnd::Connection`, as temper's (`temper/crates/temper-service/src/limits.rs`).

`wire.md`, sections 2.1 and 2.2 and `sign-in.md` and `default-http.md`, sections 1 and 3 have the approval's ask, read,
decision and take, its four HTTP routes and cookie-only browser mark.
The component holds four live approvals for ten minutes and remembers
eight ended approvals. Its tables and joint-client stories cover the
ends, first decision, one credential, the approving party, a bearer
refused and an approval never taken making no sign-in. `README.md`, section 12 keeps
approval between hosts and the person's list of sign-ins deferred.
