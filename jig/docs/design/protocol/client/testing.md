# The client's protocol: testing

## 1. Testing

Each within jig's suites' budgets.

- **The client link's admission:** a watch ended by its stream while its
  opening and cleanup HTTP calls remain in the browser's fetch table.
  Authorization waits for credit, then receives its code; both replies
  and failures release credit, with one watch terminal throughout
  (`jig-client-protocol-link`'s stream table).
- **Step tests** in `jig-client-protocol-wire` and each application's
  wire are written by hand, with values built from schema text or public
  fields, as skein's `data/codec/golden.md`, section 1 says; the fixed
  envelope read across versions among them. No tests are generated.
- **The client's wire world,** `jig-client-wire-world`: the client
  side's generated codecs (jig's wire, the reference UI's records,
  `ops`'s wire and jig's test family's) checked for drift against their
  schemas, and jig's wire's decoders run over arbitrary and mutated
  bytes in the fuzzy suite (skein's `data/codec/golden.md`, section 1).
- **The template preview** in the client-protocol carriage tests and
  wire tests built from schema text or public fields: both configured
  intent and `resolved_default`, including
  `None`, cross whole. The shapes tests keep omission, and the
  reference view tests consume the supplied preview;
  core charters/templates stories own resolution order and invalidation
  (`../../domain/testing.md`, section 4).
- **The component's world,** `jig-engine-clients-world`: the component
  against a scripted transport of messages and a scripted root, with no
  HTTP: answers of every kind and none, deliveries paced by a slow
  `StreamTaken`, a stream gone mid-delivery, two streams of one sign-in
  each with its own name, a stream sent gone by the client that followed
  it, signed out, records that do not decode, a version not served, a
  `Mint` answered and refused, the printed link spent and drawn again, a
  link's sign-in carrying a live credential kept (no credential drawn,
  no digest kept), and one carrying a credential bad, expired or ended
  while its check is pending signed in afresh and refused nothing; the
  link's sign-in answered not ready, busy and full, its secret the
  link's again with its expiry as drawn, and the same link signing in
  once there is room, or answered `link_expired` past that expiry; a spent
  secret answered `link_used`, another's `link_unknown`, an empty one's too; the
  same secret sent again while reserved answered busy, another with no fresh link
  drawn; the reservation dropped by the core's refusal, a close and an
  abort; the exchanges' admission full answered busy; full carrying the
  link's expiry as drawn; a `Mint` and a service's sign-in answered busy
  for full; a
  service signing in, a key added, sent again under its key with the
  same value and with another; an account moved out, its record in the
  first answer alone and empty in the kept answer sent again, and a
  moved file moved in under its key; a family's request admitted and
  its caller gone before the stop, between the stop and the close, and
  after the close, the component closed with the request's answer still
  out, and an answer that comes once its caller has gone settling with
  no reply. Its referee: one reply per exchange; one
  streamed message in flight per stream; no credential reaches a
  domain; no key and no moved record is in the small store, a
  committed request or an answer kept.
- **A batch, at the component:** its members' parts in the order
  released, a read held until durable holding back no other member's;
  one exchange of the sign-in's taken whatever its members, and each
  read and opening reserved as alone, past `reads_per_client` or
  `openings_per_client` that member alone busy; a batch finding no
  exchange's room answered as its messages apart, reads and openings
  busy and closings taken; a member malformed or conflicting a part with
  no reply; a batch past its bytes or its 64 members, or no record,
  answered not a record with nothing done; its caller gone mid-batch,
  its members settling with no part. A stream's opening naming one of
  its sign-in's ending it first, so that it is admitted at the
  streams' bound, and one naming another sign-in's stream or none open
  changing nothing.
- **A batch, at the mapping's ends:** the engine end's frames written
  as the parts come, one slow member holding none back, at most 64
  unwritten, the body ended at `Done`, one call held until then, `400`
  and `413` whole; the client end's frames cut out as their last byte
  comes, from pieces cut at random, each a `Part` at once, `Done` at a
  boundary, `Lost` after the whole parts for a body cut inside a frame,
  a frame past the record's bound, or a status other than `200`; the
  stream's `replaces` query sent and read, and any other query `404`.
- **A batch, at the link:** gathered messages left at the settling
  alone, in order, one batch while more than one may leave, split past
  64 members or `BATCH_BYTES`, and what finds no exchange waiting for a
  later settling; each part to its member as alone, a busy opening
  opened again after its backoff and a busy read ended busy; a batch
  cut short, its answered members kept, its unanswered reads ended
  unreachable and an unanswered opening or closing dropping the stream;
  a part naming no unanswered member taken as the batch lost; each
  stream's opening naming the last stream known, with no stream gone
  sent.
- **The default mapping's ends,** each against a scripted peer of bytes:
  every route, the stream's events cut at random, the credential as a
  cookie and as a bearer, the printed link's route (a browser's link
  sign-in with its live cookie answered with no `Set-Cookie`, and one
  answered busy or full with none), a call ended before its reply said
  gone once, as `ExchangeGone`, and a call answered or never handed
  over said gone never, and the
  example guards' stories (`../browser/testing.md`).
- **The protocol world,** `jig-client-protocol-world`: the client's
  world's stories (`../../client/domain/testing.md`, section 4) run through
  bytes: `jig-client` over the native shell's machines and the default
  mapping's client end, against the engine's root and core over
  `jig-protocol-engine-clients` and the mapping's engine end on skein's
  HTTP server, joined by in-memory streams cut at random; and the same
  stories over an in-process transport with no HTTP. One story puts
  skein's text shell's terminal in front of the native client, on
  skein's simulated machine: lines in on a pipe, pages out on another
  (`../../client/domain/view.md`, 15.6). Its referee is the joint world's,
  and adds: every record crossed decodes to what was encoded; no secret,
  nor any value a request carried, reaches a domain; a carried value is
  sent once, never resent nor in the small store (`README.md`, section 3); a slow
  stream never holds the engine up.
- **The browser tier:** the reference UI's browser shell in Chromium,
  against jig's test application, `jig-test` (`../browser/testing.md`
  ).
