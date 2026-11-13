# openid: status

2026-11-10. Where the tree's code lags or departs from [openid.md](openid.md).

- **Departs:** the whole document: `authorize_reply` in
  `jig/crates/jig-protocol-engine-clients/src/provider.rs` maps people's
  `Again`, including `NotReady` during restoration, to `NotReady`;
  the design gives that refusal while stopping alone. The
  `ui-authorize-busy` repair fixes client transport admission, not this
  readiness path. No bounded provider wait is implemented; `README.md`, section 12
  keeps the readiness policy for separate work.

`wire.md`, section 1's invalid run ends carry their immutable contract kind or
unreadable-result cause through wire 19 and the client link. The protocol
world drives contract-invalid and adapter-invalid endings; no gap is known
in this diagnostic.

`wire.md`, section 1's v20 run-wait records carry hello figures, bounds refusals
and the turns, time and waiting shortfalls' charters in both directions.
The server-bytes-to-client-task table covers figures present and absent,
and a bounds refusal with no shared shortfall. No known gap remains in
these records or their translations.

`sign-in.md`'s used, expired and unknown link answers are present, including
the eight-digest history, constant-time digest comparison and preservation
of a live or reserved link. The engine table covers empty and unknown
secrets and the eighth and ninth past links. Longer-lived history is
deferred in `README.md`, section 12.

`wire.md`, section 2.2 and `sign-in.md`'s local link has no known gap: `LocalLink` said after
the printed link at start, each link reserved, returned and spent apart,
a spent local link drawn again at once outside close and abort, an
expired one by the next sign-in that finds none live, and a remembered
local link's secret drawing no printed link. The engine-clients world
holds both links signing in once in either order and crossing a
reservation, a dead local link never printing, a restart forgetting
both, and no printed link ever a local one's; the closing unit table
holds close and abort drawing none. `sign-in.md`'s first local link given,
`local_secret`, and `LinkSignedIn` (`wire.md`, 2.2) have no known gap either: the
engine-clients world's sign-in suite holds a given local secret signing
in as the first local link with no `LocalLink` said, and `LinkSignedIn`
said once for a printed link's sign-in made or kept, never for a used
link's refusal, a `Mint` or the local link.
