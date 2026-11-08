# client: status

2026-11-08. Where jig's code lags or departs from [client.md](client.md).
Unlisted sections have no known gap.

Sections 13 and 14's transport accounting is present: emitted exchanges
keep credit through `Reply` or `Lost`, including cancelled watch calls.
The stream table holds authorization behind a full fetch table, then
releases it after successful or failed HTTP terminals, with no duplicate
watch ending.

- **Departs:** 18: `authorize_reply` in
  `jig/crates/jig-protocol-engine-clients/src/provider.rs` maps people's
  `Again`, including `NotReady` during restoration, to `NotReady`;
  the design gives that refusal while stopping alone. The
  `ui-authorize-busy` repair fixes client transport admission, not this
  readiness path. No bounded provider wait is implemented; section 15
  keeps the readiness policy for separate work.

Section 3's invalid run ends carry their immutable contract kind or
unreadable-result cause through wire 19 and the client link. The protocol
world drives contract-invalid and adapter-invalid endings; no gap is known
in this diagnostic.

Section 3's v19 run-wait records carry hello figures, bounds refusals
and the turns, time and waiting shortfalls' charters in both directions.
The server-bytes-to-client-task table covers figures present and absent,
and a bounds refusal with no shared shortfall. No known gap remains in
these records or their translations.

Section 8's used, expired and unknown link answers are present, including
the eight-digest history, constant-time digest comparison and preservation
of a live or reserved link. The engine table covers empty and unknown
secrets and the eighth and ninth past links. Longer-lived history is
deferred in section 15.

Sections 4.2 and 8's local link has no known gap: `LocalLink` said after
the printed link at start, each link reserved, returned and spent apart,
a spent local link drawn again at once outside close and abort, an
expired one by the next sign-in that finds none live, and a remembered
local link's secret drawing no printed link. The engine-clients world
holds both links signing in once in either order and crossing a
reservation, a dead local link never printing, a restart forgetting
both, and no printed link ever a local one's; the closing unit table
holds close and abort drawing none. Section 8's first local link given,
`local_secret`, and `LinkSignedIn` (4.2) have no known gap either: the
engine-clients world's sign-in suite holds a given local secret signing
in as the first local link with no `LocalLink` said, and `LinkSignedIn`
said once for a printed link's sign-in made or kept, never for a used
link's refusal, a `Mint` or the local link.

- **Design departs:** 8, from the request `ui-small-1` of 2026-11-06,
  item 1's unconditional used-link explanation: after a restart or eviction
  from the last 8 remembered links, a used link answers `link_unknown`.
  The plan chose bounded memory rather than a stored history; its direction
  to use the current terminal's link still holds.

- **Not yet:** 6: a turn read answered gone past its transcript's horizon: no retention drops a transcript (`store.md`, 11.1). 14: the protocol world's stories over an in-process transport: `jig/tests/client-protocol` runs them through native HTTP bytes alone.
- **Departs:** 6: a tree's `next` is a cursor no read takes: `jig/crates/jig-protocol-engine-clients/src/translate/watches.rs` writes it with `cursor::encode_tree`, in the form `Tree`; `cursor::decode` refuses that form, `cursor::decode_window`, which a family's window reads, takes a bare `DocumentCursor` (`jig/crates/jig-protocol-engine-clients/src/tests/reads.rs`), and a watch opens with no cursor at all, so a client that gives it back is refused. It stays because the request of 2026-11-05 held behaviour still while the cursor became a schema's record, and no debt item holds it. 17.4: the reference server, by its `PeerEnd::Call` (`jig/testing/jig-test-service/src/limits.rs`), leaves a peer's end during a mounted call to skein's server and does not end the connection itself (`jig/crates/jig-protocol-engine-clients-http/src/server.rs`): a request all read goes on to its response, the engine end hears nothing of that end, a stream's end is seen at a failed write or the stall, and an exchange left unanswered says `ExchangeGone` (4.2) only at the stall. The engine-clients world plays the section's end in `jig/tests/engine-clients/tests/suite/gone.rs` alone, whose server it builds with `PeerEnd::Connection`, as temper's (`temper/crates/temper-service/src/limits.rs`).

Sections 4.1, 4.2, 8, 17.1 and 17.3 have the approval's ask, read,
decision and take, its four HTTP routes and cookie-only browser mark.
The component holds four live approvals for ten minutes and remembers
eight ended approvals. Its tables and joint-client stories cover the
ends, first decision, one credential, the approving party, a bearer
refused and an approval never taken making no sign-in. Section 15 keeps
approval between hosts and the person's list of sign-ins deferred.

- **Departs:** 8, *Never taken, nothing made*: a take admitted before
  expiry remains reserved until the core answers, even past the approval's
  expiry. `jig-protocol-engine-clients/src/approval.rs`, `State::sweep`
  and `next_deadline`, exclude `Decision::Taking`; `approval_whose` and
  `approval_reply` do not check the expiry again. The implementation
  explicitly preserves the reservation as a printed link's does, rather
  than enforcing the section's unconditional expiry. Seen by reading;
  no runtime result establishes this boundary.
