# The client's protocol

Provisional, 2026-10-10, revised 2026-10-24. What carries jig's API
(`../../client/domain/contract/README.md`) as messages and bytes, whatever the
transport: jig's messages; the clients' component's messages toward the
application's transport and server, and the client's link toward its
transport; the wire's records and their versions; exchanges and the one
stream a client keeps; how a keyed request's answer becomes the request
table's envelope; reads, watches and a run's text; signing in, by jig's
built-ins or by the application's sign-in and a `Mint`; the small
store's encoding; an application's documents and requests on the same
messages; the shells; and jig's default HTTP mapping, an option of
jig's (`default-http.md`). Its two sides are a component of the engine's
protocol layer and a part of `jig-client` (`../components.md`, section 1). What
is still open is listed in `README.md`, section 12.

- [`wire.md`](wire.md): jig's messages and their framing.
- [`versions.md`](versions.md): versions of the client's wire and its records.
- [`small-store.md`](small-store.md): the small store's keys and encoding.
- [`sign-in.md`](sign-in.md): sign-in flows and credentials.
- [`openid.md`](openid.md): sign-in with OpenID.
- [`public-reads.md`](public-reads.md): reads that do not require a sign-in.
- [`default-http.md`](default-http.md): the default HTTP mapping.
- [`shells.md`](shells.md): the native and browser shells.
- [`testing.md`](testing.md): the client's protocol tests and worlds.

## 1. In one page

- **jig's messages, transport-agnostic.** A keyed request, a read,
  opening and closing a watch, a stream gone, signing in and signing
  out: each an exchange, one record each way, answered once; or reads,
  openings and closings many in one exchange, a batch, each member
  answered as alone, as it is ready (`wire.md`, section 2). And one
  stream per client, for every delivery, tagged with its watch. Every
  message names its owner and its version, and comes with its credential
  as opaque bytes, or none (`wire.md`, section 2).
- **The clients' component knows no HTTP.** `jig-protocol-engine-clients`
  takes jig's messages from the application's transport, and a `Mint`
  from the application's server, and answers with jig's messages, the
  stream's, and credentials. Paths, cookies, hosts, cross-origin rules
  and server-sent events are the transport's (`wire.md`, 2.2).
- **Nor does the client's link.** `jig-client-protocol-link`, inside
  `jig-client`, encodes and decodes jig's messages, makes the request
  table's envelopes, judges the stream's hello, and encodes the small
  store's records; `jig-client` hands its transport each message as
  bytes (`wire.md`, 2.3).
- **The transport is the application's,** with jig's default HTTP
  mapping to use, wrap or replace: its client end,
  `jig-client-protocol-http`, and its engine end,
  `jig-protocol-engine-clients-http`, with example guards; its routes
  under the prefix the application mounts it at, its stream as
  server-sent events (`default-http.md`).
- **Records from schemas, per owner, in general form.**
  `jig-client-protocol-wire` holds jig's records for both ends: the
  plumbing operations and the compound request, their answers, reads,
  task queries, subjects, `me`, the task line and the other documents.
  No record names a shape of work. Each application's wire crate holds
  its own (`wire.md`, section 1).
- **One attempt per exchange.** The link makes one attempt per request,
  read or opening, and says how it ended. For a keyed request it makes
  the request table's envelope: *final* with the engine's answer decoded
  into its owner's vocabulary, *again*, *lost*, or *signed out*
  (`../../client/domain/client/requests.md`). Retrying is the table's.
- **The stream paces the watches.** The component hands the transport
  one message per stream at a time, the next once the transport has
  taken it; a client that reads slowly holds its stream back, and the
  engine coalesces its watches' changes meanwhile. Nothing is
  acknowledged by hand, and the engine keeps no backlog (section 5).
- **The stream keeps the link.** It opens with a hello (the engine's
  versions, its heartbeat, then `me`), carries a heartbeat while nothing
  else is sent, and its loss drops every watch on it, which the client
  reopens. Every stream watches `me` from its opening, so the client
  opens no watch of it (`wire.md`, section 2).
- **jig owns credentials; sign-in flows are the application's.** jig's
  built-ins are the printed link, on loopback, and services'
  credentials; every other way is the application's own flow on its own
  routes, then a `Mint`. A credential is a sign-in's number beside
  random bytes; the engine keeps its digest, and hands the domain only
  the number (`sign-in.md`).
- **Public reads need no credential.** A public read, one exchange of
  its own, is served as the public party (`../../domain/people/public-reading.md`
  ), within its own room and rate, behind every signed-in
  exchange; over HTTP it is a `GET` a caching proxy may keep (`public-reads.md`
  ; `default-http.md`, 5).
- **Versions per owner:** the ranges served in the hello, and a version
  not served answered so, typed, naming the owner and the ranges, in an
  envelope every version reads (`versions.md`, section 1).
- **The small store is the client's records,** the request table's and
  its own, per owner, versioned, each kept request's stored before it
  leaves. A top-level `Seen` stays in memory, and a request carrying a
  value is sent once and never stored (section 3). The application keeps
  its own (`small-store.md`, section 1).
- **Shells are the application's.** jig ships a native shell,
  `jig-client-shell-native`, and its browser parts,
  `jig-client-shell-web` (`shells.md`, section 1).
- **On one person's machine** (`../README.md`, section 4) the client reaches
  the engine on a loopback address, in plaintext, and the person signs
  in by a link the application's startup prints at start (`sign-in.md`).

## 2. Where it sits

```
client                                                     engine
the application: its presentation, jig's helpers           the engine's root (core, connectors)
  ▲ jig_client::Client's surface (../client/domain/)         ▲ the core's vocabulary, and connectors'
jig-client: jig-client-domain, jig-client-protocol-link    jig-protocol-engine-clients, the clients' component
  ▲ jig's messages, as bytes                                 ▲ jig's messages, as bytes; Mint
the application's transport ◄── exchanges, one stream ──► the application's transport and server
  jig-client-protocol-http, the default mapping's            jig-protocol-engine-clients-http, the default mapping's
the shell: fetch, skein's HTTP client, a socket            skein's HTTP server, TLS or plaintext on loopback; io
```

- **`jig-protocol-engine-clients`,** the clients' component, is a
  component of the engine's protocol layer: jig's messages decoded and
  answered; each credential checked against its digest, and the domain
  handed the sign-in's number; the clients' streams and their pacing;
  the printed link and services' credentials; `Mint`; the translation
  between the wire's records and the core's vocabulary. An application's
  requests, reads and subjects are decoded by the application's wire,
  through the protocol layer's routing, and reach the engine's root
  typed. The application's server mounts it wherever it chooses; jig
  reserves no path.
- **`jig-client-protocol-link`,** inside `jig-client`, is the client's
  half toward the transport: each attempt's message, the stream's
  messages and their watches, the envelopes, the stream's hello judged
  against the client's versions, the small store's records encoded and
  decoded, turns decoded by `jig-protocol-transcript`, and the
  translation between the wire's records and `jig-client`'s vocabulary.
  It is step code, built for the host and for wasm32
  (`../../client-stack.md`, section 1).
- **`jig-client-protocol-wire`** holds jig's records, for both ends
  (`wire.md`, section 1).
- **The transport** is the application's: what carries a message's bytes
  and its credential to the engine and back, and the stream's messages
  as they come. HTTP by jig's default mapping (`default-http.md`), a local
  socket for a command line, or an in-process channel for one process
  and the worlds.
- **The shell** is the platform's: its HTTP stack or socket, its
  storage, its clock and its entropy, around the application's loop
  (`shells.md`, section 1).

## 3. Keyed requests

An owner proposal outcome carries its proposal number and the role
first admitted (`OutcomeProposed.to`), in the project of the request.
Replays retain that role; the proposal document gives its current
recipient.

- **One exchange per attempt.** The request table's send (its attempt,
  its key, its request) becomes one keyed request: the key, the
  request's owner, version and record, with the credential. Its reply,
  or `Lost`, is the attempt's one terminal.
- **A compound request** is one keyed request: one record, `Compound`,
  holding its operations in order, answered by one answer
  (`../../client/domain/contract/requests.md`, 4).
- **The engine's side** checks the credential (`sign-in.md`); hands the core
  the sign-in's number, the key and the request, decoded into the core's
  vocabulary; and, for an application's request, the digest of the
  record's bytes as they came (SHA-256), which tells a key sent again
  with another request apart (`../../client/domain/contract/requests.md`, 3). The
  record is canonical (skein's `data/codec/README.md`, section 1), so the same
  request always has the same digest. A request carrying a value
  (below) has a digest of its own.
- **A request carrying a value,** an LLM account's key added or
  replaced, a sign-in's pasted address, or a moved account's file moved in
  (`../../client/domain/contract/requests.md`, 2), or an application's
  request that carries one, is sent once and never kept: the client
  never stores it in the small store, and never sends it again by
  itself (`../../client/domain/client/requests.md`). Its key is its
  caller's name, never made from the value
  (`../../client/domain/contract/requests.md`, 1).
  - *The engine's side* takes the value out of the record as it decodes
    it, keeps it in the secrets' store before the core sees the request
    (`../store/secrets.md`), and hands the core the request with the
    value named by a number jig draws for it, never one the request
    names, and the value's digest. An LLM key's value is then adopted
    into its source's secret row in one commit, which erases the
    carried row (`../store/secrets.md`). A sign-in's pasted address is
    held, not kept: in the secrets' store's memory alone, never written,
    and erased once its request is answered, the core leaving every
    finish unused (`../store/secrets.md`; `../llm.md`, 5.4). The request's own digest covers
    its record, the value left out, and its value, each after its
    length, in place of the record's bytes as they came, so that a key
    sent again with another value is told apart. Those digests are the
    only trace of the bytes the core keeps, and no value can be read
    from them.
  - *The client's side* sends it once: an answer of busy or not ready, a
    link down, or a party signed out or not yet confirmed ends it as
    *unsent*, the engine having made nothing; an exchange that ends with
    no whole answer ends it as *lost*, the engine possibly having made
    it. Nothing is sent again by the client. Its caller asks again under
    the same key, with the value; the engine answers a key it has
    answered with its first answer, and erases unused a value kept for
    it again.
  - *No value is in a record kept:* not in the small store, the core's
    commits, an answer kept for its key, a log or a progress report.
  - *Its bound* is the wire's: a keyed request's value is at most
    128 KiB, which holds a moved file under each profile's record bound
    (`../llm.md`, 5.6); startup refuses a configuration whose moved file's
    bound would pass it. The engine refuses a key past the bytes `me`
    states, and a moved file past its form's bound, as a limit.
- **A request whose answer carries a value,** an account moved out
  (`../../client/domain/contract/requests.md`, 2), goes the same way: sent once,
  never kept, never sent again by the client, since its value is given
  once and a second answer would not carry it.
  - *The engine's side* fills the value as it encodes the answer for
    the exchange that asked, never before the core's commit: the core's
    outcome names what holds it (the moving source), and the clients'
    component asks the application's root for it, which takes it from
    the accounts' protocol layer (`../llm.md`, 5.6), once. The value is in
    no commit, no answer kept for the key, no log and no progress
    report, as a service's minted credential is not.
  - *A second answer under the key,* to the request sent again or to a
    join, is the kept answer, its value empty: the value went with the
    first. The client answers its caller *lost* for it (as for an
    exchange that ended with no whole answer): the engine made the
    request, the value is gone with the exchange that carried it, and
    the account's move is cancelled at its deadline
    (`../../domain/accounts.md`, section 6). Its caller asks again under a
    new name once it is.

## 4. Reads

- **One exchange per read,** or one member of a batch (`wire.md`, section 2),
  answered once: the document, a page with its
  cursors, or why there is none (unknown, busy, signed out, version not
  served) (`../../client/domain/contract/reads.md`, section 1). A page's cursor
  is the engine's, opaque bytes the client gives back. Inside they are a
  record of the family `jig-engine-cursor`
  (`jig/crates/jig-protocol-engine-clients/schema/cursor-v2.schema`),
  whose fixed-rule values are built from its schema text in tests
  (skein's `data/codec/golden.md`, 1), with a key inside one in the store's
  codec. A cursor of another version does not decode, and its read is answered unknown (`../README.md`, section 7).
- **Credential boundaries, on the client,** visibly end pending reads
  in the domain's `Page` forms: jig's `ReadNone::SignedOut` or
  a family's `None`, once each, on sign-out and unconditionally on
  successful sign-in, before the first `me` too
  (`../../client/domain/client/README.md`, section 6, and `../../client/domain/client/sign-in.md`, section 1). The link erases
  old exchange names; the domain retains pending token and optional
  family and fences late completions with its token match. The
  boundary needs no link output, public `Readout` variant or wire
  record of its own, and never replays a read. Supersession and `Unwatch` remain abandonment.
- **A task query's page** carries its two groups, live then ended, each
  with its cursor, and says whether the ended group was cut by a filter
  beyond its index's prefix, or has none for the query's keys
  (`../../client/domain/contract/reads.md`, 1.1).
- **An application's read** names its scope and its window, and its
  connector's read, in the application's wire, and is answered with the
  application's document, or unknown, or not allowed.
- **A turn read** (`turn{n, attempt, turn}`) is answered with the
  turn's bytes, or its size where the turn alone passes the read's byte
  bound, or gone past its transcript's horizon, or why there is none, as
  above (`../../client/domain/contract/reads.md`, section 1; `../store/transcripts.md`, 3).
- **Held until durable** on the engine's side, as every read's answer is
  (`../../domain/engine/decisions.md`, 1.2); the exchange waits for it.

## 5. Watches

- **Opening a watch** is an exchange, or one member of a batch (`wire.md`, section 2):
  the stream's name, the client's name
  for the watch, its subject and window. It is answered admitted, or
  refused (unknown, not allowed, busy, signed out, version not served);
  the first delivery follows on the stream. A subject an application's
  connector judges is admitted at the end of the step that takes it, and
  the connector's refusal, if it comes later, ends the watch with its
  reason (`../../client/domain/contract/watches.md`, 1.2). A subject of the party's
  own (`me`, their inbox, an application's subject in the party's own
  scope) names no party, on the wire or in the core's events: the
  protocol layer hands the core the sign-in's number, and the core names
  the subject's party from it, as it does a read's, from the sign-ins its
  people child keeps (`../../domain/engine/decisions.md`, 1.4). The protocol layer
  never keeps a sign-in's party, which a restart would lose.
- **A delivery** is one streamed message: the watch's name, and the
  document, whole, in its owner's wire. The component hands it to the
  transport when the core releases it (held until durable,
  `../../domain/engine/views.md`), and tells the core it was taken
  once the transport says `StreamTaken`. Until then nothing more is
  streamed to that client, and the views coalesce what changes
  meanwhile. A client is never told it missed anything.
- **Ending:** a streamed message naming the watch and why
  (`../../client/domain/contract/watches.md`, 1.2); or closing it by an exchange,
  answered once the engine has ended it. A stream that ends ends its
  watches. Once its connection is closed, the component sends each
  watch's unwatch and outstanding delivery acknowledgements, then drops
  the watch without waiting for the core's ending. Late core replies
  cannot retain the closed stream's slot. A document or run text released
  after its watch is dropped is acknowledged as dropped when it arrives,
  so the core can finish its watch and drain. A name the component has
  never issued is ignored. The core answers an unwatch
  whose watcher it does not know with an unwatched ending.
- **A stream's own refusal is observable.** When its queued records pass
  its byte ceiling, or a delivery, run text or ending fails translation,
  the component emits one `StreamFailed` diagnostic as it starts closing.
  It carries the sign-in number and either the queue ceiling or the
  translation's schema path and reason, never document bytes. The host
  writes this to its operator journal; later deliveries on that closing
  stream add no diagnostic.
- **A closing follows its opening's answer.** Exchanges cross
  independently, and a closing that overtook its opening would find no
  watch, be answered closed, and leave the watch admitted after it, which
  nothing closes but its stream's end. So the client sends a watch's
  closing only once its opening is answered admitted, and none once it
  is refused. An opening or a closing that gets no answer leaves the
  client unsure what the engine holds: it drops the stream (`wire.md`, section 2),
  whose end ends every watch on it.
- **Each delivery is settled once, while the engine runs.** The core
  makes a watch's next delivery, or ends the watch, only once the
  component has told it of the last one: taken, once the transport took
  its message; or dropped, when its stream ended first. So a watch's
  end, by the client, by the engine or with its stream, never excuses
  telling the core of a delivery the client took. A restart does: the
  core restarted cold holds no watch (`../../domain/engine/views.md`
  ), and is told nothing of deliveries made before it; the client
  watches again (`wire.md`, section 2), and its first delivery is the document
  whole.
- **A run's text** is a watch of the run: each `RunText` delivery carries
  the turn's name, text after the client's offset, and its end offset
  (`../../client/domain/contract/watches.md`, 1.3). `writing` carries the current call
  being written whole: provider id, tool, input suffix, byte offsets and
  `cut`. `call` remains the call being made. The client keeps text and
  replaces `writing` whole on an accepted delivery. A new turn or retry
  replaces the provisional document; a committed turn clears it.
  Input is carried from the first retained character boundary through the
  last complete UTF-8 character; an incomplete final character waits for
  the next piece, and invalid UTF-8 omits `writing`. These deliveries leave
  without a commit, marked provisional. Reasoning has no client surface.
  Only conversations are asked for text and input (`../channel/owners.md`, 1.4).

## 6. Versions

Moved to [`versions.md`](versions.md), section 1.

## 7. The small store

Moved to [`small-store.md`](small-store.md), section 1.

## 8. An application's documents and requests

- **On the same messages and stream:** a keyed request names its owner,
  jig or the application's family; a read and a subject name theirs; a
  streamed message's record is its watch's owner's.
- **Scopes:** an application's request, read or subject names its
  scope, a project, the deployment, or the party's own, and a read or
  subject its window (`../../client/domain/contract/requests.md`, 3, and `../../client/domain/contract/watches.md`, 1.5).
  The party's own names no party: the core names it from the sign-in
  (`README.md`, section 5), and admits it on the credential alone.
- **The engine's side** decodes an application's record with the
  application's wire, through the protocol layer's routing, before the
  engine's root sees it; on the way out it encodes the connector's
  document and outcome the same way.
- **The client's side:** `jig-client` carries an application's records as
  bytes under their owner's number (`../../client/domain/root.md`, section
  15), and the application decodes them with its wire.
- **An application's wire,** `ops`'s for one: `../../examples.md`,
  section 8.

## 9. Shells

Moved to [`shells.md`](shells.md), section 1.

## 10. Limits and the worst case

- **The client's:** exchanges in flight, the stream's heartbeat
  tolerance, a record's bytes on each side (from the wire's worst cases
  at the client's limits), the stream's message bytes; account receiving
  counts for accounts per list, sources per account and sign-ins per
  list, checked by the link within the held document's decoded room
  (`../../client/domain/client/README.md`, sections 11 and 13).
- **Transport credit lasts through the terminal answer.** Every emitted
  exchange counts until its `Reply`, `Done` or `Lost`, a batch one
  whatever its members, including a cancelled
  watch's opening and cleanup calls. Ending a logical attempt does not
  free its transport credit. `Link::in_flight` is the unfinished
  transport exchanges; a late terminal frees one without ending a
  logical operation twice. The logical attempts, a batch's members each
  one, are bounded by the client's reads in flight and the engine's
  watches it counts (`../../client/domain/client/README.md`, section 11), not by its
  exchanges, so a batch carries what one settling gathered.
- **A watch's opening and closing wait for credit,** as a read does: one
  that finds every exchange in flight is held, in the order asked, and
  sent when a terminal answer frees credit, never ended busy or dropped
  for the client's own admission. A held opening the client closes ends
  closed, unsent (section 5). Only the engine's refusal ends an opening
  busy.
- **A held read the client cancels is dropped unsent.** A read the
  domain supersedes or ends while it waits for credit leaves the link's
  queue: `jig-client` tells the link with `Link::cancel_read(read:
  Token)`, an input with no output, public only because the link and the
  client are separate crates. One already sent keeps its transport
  credit to its terminal, whose answer fails its token match
  (`../../client/domain/client/sign-in.md`, section 1). So the reads the
  link holds stay within the client's reads in flight, and the held
  queue keeps the order asked across reads, openings and closings.
- **The transport's:** an exchange's deadline, and what it buffers of the
  stream and of the small store's writes.
- **The component's:** streams in all, whatever their sign-ins (`wire.md`, section 2
  ); the watches one stream holds, each until the engine has ended it,
  at least the engine's watches one client opens at its limit, as a
  party's room is (`../../client/domain/contract/watches.md`, 1.2), and past them
  an opening is refused busy, within the views' limits on a party's
  watches and on all (`../../domain/engine/README.md`, section 7); exchanges in
  flight per sign-in, which its clients share, since an exchange names
  no stream, and in all, a batch one whatever its members (`wire.md`, 2.2);
  per sign-in, reads in flight, `reads_per_client`, and watches'
  openings in flight, `openings_per_client`, each a read's or an
  opening's whether alone or a batch's member, past which it is
  answered busy, sized to the client's own reads and watches at once
  (`../../client/domain/client/README.md`, section 11), 32 and 31 in
  both stock profiles and the dotted overrides
  `clients.reads_per_client` and `clients.openings_per_client`; a
  batch's members, 64, its schema's bound; a message's bytes, a batch's among them, which bound a change of
  settings (`../../domain/engine/settings.md`, 8), what a stream may have
  queued before nothing more is handed out, the heartbeat's span, mints
  in flight; and the public reads' own: in flight, per second and its
  burst, their answers' bytes per second and its burst, and the answer
  ceiling (`public-reads.md`); and the approvals', fixed: 4 live, 2 of
  them waiting per source by the public name, 8 ended remembered, each
  10 minutes long, and 3 takes with a grant that doesn't match
  (`sign-in.md`).
- **The public answers' `max-age`,** the deployment's, at most a bound
  of jig's (`default-http.md`, 5).
- **The default mapping's,** at each end: its HTTP machines', TLS's, and
  the stream's buffer, at those limits.
- **Internal hand-offs are uncapped.** The client retains each pending
  read's token and optional family metadata. A credential boundary can
  emit one cancellation `Page` per admitted read, independently of
  watch handles; this bounds the event's fan-out without reserving an
  output queue (`../../client/domain/client/README.md`, section 11). The kernel's
  queues and transport edges keep their limits; fixed scenarios check
  their measured heap peaks against ceilings
  (`../../client/domain/testing.md`, section 6).

## 11. Testing

Moved to [`testing.md`](testing.md), section 1.

## 12. Open questions

- **Waiting for people during sign-in** (8, 18): whether authorization,
  minting and approval sign-in should wait within a sign-in bound while
  people restore. Local transport admission (section 10) does not settle
  that readiness policy. No retry interval or wait bound is chosen here;
  `openid.md`'s stopping-only refusal remains the contract.
- **Remembering why a printed link failed** (8): recognising a used or
  expired link, printed or local, across a restart, or beyond the last 8
  remembered links, would need longer-lived history. Default: the
  bounded in-memory digests; a forgotten link is `link_unknown`, with no
  stored-format change. A forgotten local link's sign-in is then any
  unknown secret's, and draws a fresh printed link when the printed link
  is neither live nor reserved, so the startup prints again for a native
  client's try: one that read the local link before a restart, or before
  8 other links were spent or expired.
- **Several tabs:** each tab a client with its own stream, which a hidden
  tab lets go (`../browser/README.md`, section 10); whether tabs of one person
  share one stream through the platform's means.
- **A page that is left sending its stream gone as it goes** (a
  `keepalive` request at `pagehide`), so that a tab whose next page
  never opens a stream frees the room before the browser ends the
  connection. The next client's gone (`wire.md`, section 2) frees it today.
- **A bound on one sign-in's streams,** for an engine serving many
  people between hosts. On one person's machine, the browser's
  connections to one origin and the engine's streams in all bound them
  (`wire.md`, section 2).
- **The reference server at a full room** (`default-http.md`, 4): whether it holds an
  ordinary call's head until room is free, as that section allows, in
  place of refusing it. Default: it refuses, and the client takes the
  `503` as that section says.
- **Bundling parts** in a jig read's answer (`../../client/domain/contract/README.md`,
  section 9).
- **A binary stream** in the default mapping: a streamed response body
  of the wire's bytes in place of server-sent events and base64, a third
  smaller, when browsers' streaming fetch serves every platform the
  client targets.
- **HTTP/2:** one connection for the stream and the exchanges, when
  skein's HTTP has it.
- **JSON records:** the same schemas written as JSON, for clients not
  built on skein, when the generator writes them (skein's `data/codec/run-time.md`,
  section 1).
- **A sign-in helper for OAuth or OpenID:** a generic flow, configured
  by its issuer and its client, which an application's server would run
  on its own routes before its `Mint`, so an application whose people
  sign in through a standard server writes no flow of its own (`sign-in.md`
  ). It is sugar over `Mint`, which jig adopts once a second
  application needs it (`../../client/domain/client/helpers.md`).
- **A person's sign-ins listed and each ended** from a browser, the
  approved ones among them. Default: none; signing out ends one's own.

## 13. What it owes and is owed

- **`jig-client`** is owed by its design (`../../client/domain/client/README.md`):
  the link's states, out of date among them (`sign-in.md` and `versions.md`, section 1 here); a
  kept request's exchange sent only once its record is stored, with
  `Seen` memory-only (`small-store.md`, section 1); a credential held, or carried by the transport (`sign-in.md`); reads and
  watches answered signed out, a reopened run's watch taken from its
  turn's beginning, and a run's document with no text and no call, for
  a run whose host gives neither (`../../client/domain/contract/watches.md`, 1.3);
  a request carrying a value, sent once, never stored, its value taken
  out and kept before the core sees it (`README.md`, section 3).
- **The application owes:** its transport, or jig's default mapping; its
  sign-in flows, with `Mint` sent from its server alone (`sign-in.md`); the
  printed link's URL, printed from the secret the component gives; its
  small store's storage, answering each write; and its server's guards
  (`../browser/serving.md`, 1.2).
- **The engine's domain** is owed nothing beyond
  `../../client/domain/contract/README.md`, section 7: sign-ins numbered by their
  commit, made for an identity the application verified, and a party
  ending theirs (`../../domain/people/README.md`, section 3); watches and reads as
  `../../domain/engine/views.md` has them.
- **skein owes,** for one person's machine: skein-lib, skein-codec and
  skein-http, with its server-sent events reader and writer, built for
  `wasm32-unknown-unknown` (skein's `runtime/lib.md`, section 2; skein's
  `testing/layout.md`, section 1); for the reference UI, its UI kit's browser
  parts, its bundler, and its face and harness for the browser tier
  (skein's `ui/web/README.md`, 4.1, and `ui/web/bundle.md`, 2, and `ui/web/testing.md`, 2), over its browser kit
  (skein's `testing/browser.md`, sections 5 and 6); a client compiled to
  WebAssembly in its model (`skein/docs/foundation/programming-model.md`, 2.1). Its HTTP
  server and client serve as they are. Between hosts, TLS's server side
  (`../needs.md`, section 1).
