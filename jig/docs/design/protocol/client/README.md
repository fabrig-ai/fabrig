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
protocol layer and a part of `jig-client` (`../README.md`, section 5). What
is still open is listed in `README.md`, section 12.

## 1. In one page

- **jig's messages, transport-agnostic.** A keyed request, a read,
  opening and closing a watch, a stream gone, signing in and signing
  out: each an exchange, one record each way, answered once. And one
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
  versions, its heartbeat), carries a heartbeat while nothing else is
  sent, and its loss drops every watch on it, which the client reopens.
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
  envelope every version reads (section 6).
- **The small store is the client's records,** the request table's and
  its own, per owner, versioned, each kept request's stored before it
  leaves. A top-level `Seen` stays in memory, and a request carrying a
  value is sent once and never stored (section 3). The application keeps
  its own (section 7).
- **Shells are the application's.** jig ships a native shell,
  `jig-client-shell-native`, and its browser parts,
  `jig-client-shell-web` (section 9).
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
  (`../../README.md`, section 10).
- **`jig-client-protocol-wire`** holds jig's records, for both ends
  (`wire.md`, section 1).
- **The transport** is the application's: what carries a message's bytes
  and its credential to the engine and back, and the stream's messages
  as they come. HTTP by jig's default mapping (`default-http.md`), a local
  socket for a command line, or an in-process channel for one process
  and the worlds.
- **The shell** is the platform's: its HTTP stack or socket, its
  storage, its clock and its entropy, around the application's loop
  (section 9).

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

- **One exchange per read,** answered once: the document, a page with its
  cursors, or why there is none (unknown, busy, signed out, version not
  served) (`../../client/domain/contract/README.md`, section 4). A page's cursor
  is the engine's, opaque bytes the client gives back. Inside they are a
  record of the family `jig-engine-cursor`
  (`jig/crates/jig-protocol-engine-clients/schema/cursor-v1.schema`),
  whose goldens the families' drift check holds, with a key inside one in
  the store's codec. A cursor of another version does not decode, and
  its read is answered unknown (`../README.md`, section 7).
- **Credential boundaries, on the client,** visibly end pending reads
  in the domain's `Page` forms: jig's `ReadNone::SignedOut` or
  a family's `None`, once each, on sign-out and unconditionally on
  successful sign-in, before the first `me` too
  (`../../client/domain/client/README.md`, sections 6 and 9). The link erases
  old exchange names; the domain retains pending token and optional
  family and fences late completions with its token match. The
  boundary needs no link output, public `Readout` variant or wire
  record of its own, and never replays a read. Supersession and `Unwatch` remain abandonment.
- **A task query's page** carries its two groups, live then ended, each
  with its cursor, and says whether the ended group was cut by a filter
  beyond its index's prefix, or has none for the query's keys
  (`../../client/domain/contract/README.md`, 4.1).
- **An application's read** names its scope and its window, and its
  connector's read, in the application's wire, and is answered with the
  application's document, or unknown, or not allowed.
- **A turn read** (`turn{n, attempt, turn}`) is answered with the
  turn's bytes, or its size where the turn alone passes the read's byte
  bound, or gone past its transcript's horizon, or why there is none, as
  above (`../../client/domain/contract/README.md`, section 4; `../store/transcripts.md`, 3).
- **Held until durable** on the engine's side, as every read's answer is
  (`../../domain/engine/README.md`, 5.2); the exchange waits for it.

## 5. Watches

- **Opening a watch** is an exchange: the stream's name, the client's name
  for the watch, its subject and window. It is answered admitted, or
  refused (unknown, not allowed, busy, signed out, version not served);
  the first delivery follows on the stream. A subject an application's
  connector judges is admitted at the end of the step that takes it, and
  the connector's refusal, if it comes later, ends the watch with its
  reason (`../../client/domain/contract/README.md`, 5.2). A subject of the party's
  own (`me`, their inbox, an application's subject in the party's own
  scope) names no party, on the wire or in the core's events: the
  protocol layer hands the core the sign-in's number, and the core names
  the subject's party from it, as it does a read's, from the sign-ins its
  people child keeps (`../../domain/engine/README.md`, 5.4). The protocol layer
  never keeps a sign-in's party, which a restart would lose.
- **A delivery** is one streamed message: the watch's name, and the
  document, whole, in its owner's wire. The component hands it to the
  transport when the core releases it (held until durable,
  `../../domain/engine/views.md`), and tells the core it was taken
  once the transport says `StreamTaken`. Until then nothing more is
  streamed to that client, and the views coalesce what changes
  meanwhile. A client is never told it missed anything.
- **Ending:** a streamed message naming the watch and why
  (`../../client/domain/contract/README.md`, 5.2); or closing it by an exchange,
  answered once the engine has ended it. A stream that ends ends its
  watches. Once its connection is closed, the component sends each
  watch's unwatch and outstanding delivery acknowledgements, then drops
  the watch without waiting for the core's ending. Late core replies
  cannot retain the closed stream's slot. A document or run text released
  after its watch is dropped is acknowledged as dropped when it arrives,
  so the core can finish its watch and drain. A name the component has
  never issued is ignored. The core answers an unwatch
  whose watcher it does not know with an unwatched ending.
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
  (`../../client/domain/contract/README.md`, 5.3). `writing` carries the current call
  being written whole: provider id, tool, input suffix, byte offsets and
  `cut`. `call` remains the call being made. The client keeps text and
  replaces `writing` whole on an accepted delivery. A new turn or retry
  replaces the provisional document; a committed turn clears it.
  Input is carried from the first retained character boundary through the
  last complete UTF-8 character; an incomplete final character waits for
  the next piece, and invalid UTF-8 omits `writing`. These deliveries leave
  without a commit, marked provisional. Reasoning has no client surface.
  Only conversations are asked for text and input (`../channel/README.md`, 5.4).


## 6. Versions

- **Per owner:** jig's wire and each application's carry their own
  version; every message names its owner and the version of its record,
  and a streamed message's record is in the version its watch was opened
  in.
- **The envelope is fixed.** A message's owner and version, and the
  answer *version not served*, are encoded the same in every version of
  jig's wire, so any engine reads them from any client, and any client
  reads that answer from any engine. *Version not served* is the first
  variant, tag 0, of each answer that carries it (a keyed request's, a
  read's, a watch's opening's, and a watch's end), and its bytes never
  change: a later version adds its variants after it.
- **The engine serves a range** of each owner's versions, which its
  stream's hello states (`../../client/domain/contract/README.md`, section 6). To
  serve a version is to keep its generated codecs and their translations
  beside the current ones. Pre-release, each range is one version
  (`../README.md`, section 7).
- **A version not served** is answered so, typed, before the record is
  read: *version not served*, naming the owner and the ranges the engine
  serves. A keyed request, a read and a watch's opening are each
  answered so; nothing is done.
- **Until the first release, versions stay fixed** (skein's `data/codec/README.md`,
  section 5). A change to a record, the template preview's among them
  (`wire.md`, section 1), changes its schema, codec, translations and one set of
  golden bytes in place. Versions move at a release; family owners keep
  their own versions.
- **The client's versions:** jig's is its wire's; each family owner's,
  the application declares as it makes the client, and the client
  judges each hello against them, whole (`../../client/domain/client/watches.md`,
  2). A hello names at most its wire's bound of ranges, jig's among
  them: an engine serves no more owners than that, and a client
  declares no more family owners than that bound less jig's.
- **A client outside a range,** from the hello or from such an answer,
  sends nothing more of that owner: its requests stay kept and unsent,
  its reads are answered at once, unsent, and its watches end, as
  version not served. Outside jig's range, the link is out of date, and
  the client sends nothing at all (`../../client/domain/client/watches.md`, 2).
  The application says how to update; updated, the client sends what it
  kept in a version it speaks, under its keys (`README.md`, section 7).
- **One jig pin:** an application's `jig-client` and its engine come from
  one jig pin, so a client the application's server serves is in range
  unless the engine was updated under a client still running; reloading
  or restarting the client updates it. A client that does not come from
  the engine's application (a phone's, a command line's) may lag it.
- **How jig is released and pinned** is an open question
  (`../../README.md`, section 14).

## 7. The small store

- **The client's records** (`../../client/domain/client/README.md`, section 8):
  the request table's, each a whole request, jig's or an application's,
  with the key the engine knows it by, its party, the application's key
  for it, its owner and version, and when it was first sent; and the
  name of the client's stream (`wire.md`, section 2), under a key of its own, which
  no request's key can be (a request's is 16 bytes); nothing else of the
  client's, and never a credential, an answer, a request carrying a
  value, or one whose answer carries one (`README.md`, section 3). A top-level `Seen` has no stored record. Each
  kept record is encoded with jig's wire, with its store version.
- **The stream's name** is saved at each hello, in a write of its own,
  and replaces the last; nothing waits for it, and a write of it not
  stored retires nothing. It is not a credential: ending a stream takes
  its sign-in's credential as well (`wire.md`, section 2). It is not one of the
  request table's records, and the records read at start are bounded by
  what the table holds and this one.
- **The application's own records are its own.** What its presentation
  keeps (where the person was, their drafts; the reference UI's places
  and drafts among them) it keeps itself, where it likes. The client's
  `Save` carries only the client's records.
- **Kept requests stored before sent.** The transport answers each `Save`
  and `Erase` `Stored` or `NotStored`. A kept request's exchange never
  leaves before its record is stored: the client sends it only once its
  `Save` is answered `Stored`. `NotStored` (a quota full, a disk failing)
  is each request
  whose record that write was the last to carry not sent, parked or not,
  answered *not kept* (`../../client/domain/client/README.md`, section 8). A
  request carrying a value, or whose answer carries one, is never
  stored, and leaves at once, once (section 3). An account's move
  *written* is an ordinary request: kept, and sent again under its key.
- **Held exchanges wait for durability and leave in order.**
  `request_records_durable` holds keyed exchanges behind pending writes;
  `held_exchanges_fifo` releases the surviving attempts once, in arrival
  order, skipping cancelled attempts even when their credit is reused.
  The client protocol world holds this under load in
  `jig-client-protocol-world`,
  `orders::held_exchanges_wait_for_durability_and_leave_in_order_under_load`;
  it holds failure and late completions without a send in
  `orders::failed_persistence_never_releases_held_exchanges_under_load`.
- **`Seen` has no save of its own.** The client domain consumes its
  table save before the composed client or link registers a store write
  (`../../client/domain/client/requests.md`). Its exchange still waits
  behind writes already given, and is eligible only for the confirmed
  party on the current link. A failed mixed write reports not kept only
  for the ordinary requests whose records it carried; a held `Seen`
  exchange is lost and retried normally. Consuming the save creates no
  tracked write, and its store acknowledgements are any write's. Every terminal
  still emits `Erase`, including an absent `Seen` key, with the usual
  acknowledgement.
- **Read at start,** the records the application's shell loaded handed
  to `jig-client` as it starts, each request's to the request table, and
  the stream's name to the link (`wire.md`, section 2); one of a
  version the client does not read is dropped, and the application told,
  so a request kept across an update that changed its record's version
  is lost, not sent wrong. A family's request whose owner and version
  are not among those the application declares is one
  (`../../client/domain/client/README.md`, 8). Pre-release, nothing is
  translated.
- **An unanswered `Seen` is lost** on closing, reload or crash, since
  it is in memory alone; the next opening checks and sends the
  watermark again, with no notice from a record of its own
  (`../../client/domain/client/README.md`, section 8).
- **Where** is the shell's: a browser tab's session storage, as text
  skein's small store part encodes (skein's `ui/web/README.md`, section 8); a
  file a native client writes whole and syncs; a service's file, synced
  before its requests leave; a world's memory.

## 8. An application's documents and requests

- **On the same messages and stream:** a keyed request names its owner,
  jig or the application's family; a read and a subject name theirs; a
  streamed message's record is its watch's owner's.
- **Scopes:** an application's request, read or subject names its
  scope, a project, the deployment, or the party's own, and a read or
  subject its window (`../../client/domain/contract/requests.md`, 3, and `../../client/domain/contract/README.md`, 5.5).
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

- **A shell is the application's:** its loop around `jig-client` and its
  presentation, its transport, its storage, its clock and its entropy,
  on its platform. `jig-client` runs no loop of its own, and never calls
  the application (`../../client/domain/client/README.md`, section 1).
- **jig's native shell,** `jig-client-shell-native`: skein's HTTP client
  on skein's io, with the default mapping's client end,
  `jig-client-protocol-http`, which cuts the stream's body into
  messages; a file for the small store, and a private file for the
  credential, which it writes when its host gives it one to keep, or
  none, and reads back at start for its host to give as held (`sign-in.md`
  ); for Linux and for the worlds that make bytes real. With no
  presentation, it is a service's shell, and a one-shot command's.
  Its public `Transport` owns the socket loop and admission, including
  sockets still closing, for both this shell and a caller's shared io,
  with a caller-chosen owner tag.
- **jig's browser parts,** `jig-client-shell-web`: the client's
  adapter, which carries the default mapping's calls over skein's
  fetch, the printed link's sign-in, and jig's look, over skein's
  `skein-ui-web` (`../browser/README.md`, 5.1), for `ops` and jig's test
  application, whose browser shell is `jig-client-test-shell-web`
  (`../browser/README.md`, 3.1). An application's own browser shell uses them,
  skein's parts alone, or neither.
- **Its pages are served by the application's server,** through skein's
  serving contract (skein's `ui/web/bundle.md`), under its own
  policy: jig serves no page, no bundle, and reserves no path
  (`../service.md`, section 2; `../browser/README.md`, section 12).

## 10. Limits and the worst case

- **The client's:** exchanges in flight, the stream's heartbeat
  tolerance, a record's bytes on each side (from the wire's worst cases
  at the client's limits), the stream's message bytes; account receiving
  counts for accounts per list, sources per account and sign-ins per
  list, checked by the link within the held document's decoded room
  (`../../client/domain/client/README.md`, sections 11 and 13).
- **Transport credit lasts through the terminal answer.** Every emitted
  exchange counts until its `Reply` or `Lost`, including a cancelled
  watch's opening and cleanup calls. Ending a logical attempt does not
  free its transport credit. `Link::in_flight` is the greater of the
  logical reservations and unfinished transport exchanges; a late
  terminal frees the latter without ending the logical operation twice.
- **A watch's opening and closing wait for credit,** as a read does: one
  that finds every exchange in flight is held, in the order asked, and
  sent when a terminal answer frees credit, never ended busy or dropped
  for the client's own admission. A held opening the client closes ends
  closed, unsent (section 5). Only the engine's refusal ends an opening
  busy.
- **The transport's:** an exchange's deadline, and what it buffers of the
  stream and of the small store's writes.
- **The component's:** streams in all, whatever their sign-ins (`wire.md`, section 2
  ); the watches one stream holds, each until the engine has ended it,
  at least the engine's watches one client opens at its limit, as a
  party's room is (`../../client/domain/contract/README.md`, 5.2), and past them
  an opening is refused busy, within the views' limits on a party's
  watches and on all (`../../domain/engine/README.md`, section 7); exchanges in
  flight per sign-in, which its clients share, since an exchange names
  no stream, and in all; a message's bytes, which bound a change of
  settings (`../../domain/engine/settings.md`, 8), what a stream may have
  queued before nothing more is handed out, the heartbeat's span, mints
  in flight; and the public reads' own: in flight, per second and its
  burst, their answers' bytes per second and its burst, and the answer
  ceiling (`public-reads.md`); and the approvals', fixed: 4 live, 8 ended
  remembered, each 10 minutes long (`sign-in.md`).
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

Each within jig's suites' budgets.

- **The client link's admission:** a watch ended by its stream while its
  opening and cleanup HTTP calls remain in the browser's fetch table.
  Authorization waits for credit, then receives its code; both replies
  and failures release credit, with one watch terminal throughout
  (`jig-client-protocol-link`'s stream table).
- **Step tests** in `jig-client-protocol-wire` and each application's
  wire: generated, as skein's `data/codec/README.md`, section 6 has them; the fixed
  envelope read across versions among them.
- **The client's wire world,** `jig-client-wire-world`: the client
  side's generated codecs (jig's wire, the reference UI's records,
  `ops`'s wire and jig's test family's) checked for drift against their
  schemas, and jig's wire's decoders run over arbitrary and mutated
  bytes in the fuzzy suite (skein's `data/codec/README.md`, section 6).
- **The template preview** in the client-protocol carriage tests and
  wire goldens: both configured intent and `resolved_default`, including
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
  sent once, never resent nor in the small store (section 3); a slow
  stream never holds the engine up.
- **The browser tier:** the reference UI's browser shell in Chromium,
  against jig's test application, `jig-test` (`../browser/testing.md`
  ).

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
  built on skein, when the generator writes them (skein's `data/codec/README.md`,
  section 7).
- **A sign-in helper for OAuth or OpenID:** a generic flow, configured
  by its issuer and its client, which an application's server would run
  on its own routes before its `Mint`, so an application whose people
  sign in through a standard server writes no flow of its own (`sign-in.md`
  ). It is sugar over `Mint`, which jig adopts once a second
  application needs it (`../../client/domain/client/helpers.md`).
- **A browser's approval between hosts** (8): over TLS, at a public
  name, a client asking from another machine, the code typed in the
  browser rather than carried in its address, and the page saying where
  the ask came from and how long ago. Default: loopback alone, as the
  printed link.
- **A person's sign-ins listed and each ended** from a browser, the
  approved ones among them. Default: none; signing out ends one's own.

## 13. What it owes and is owed

- **`jig-client`** is owed by its design (`../../client/domain/client/README.md`):
  the link's states, out of date among them (`sign-in.md` and `README.md`, section 6 here); a
  kept request's exchange sent only once its record is stored, with
  `Seen` memory-only (`README.md`, section 7); a credential held, or carried by the transport (`sign-in.md`); reads and
  watches answered signed out, a reopened run's watch taken from its
  turn's beginning, and a run's document with no text and no call, for
  a run whose host gives neither (`../../client/domain/contract/README.md`, 5.3);
  a request carrying a value, sent once, never stored, its value taken
  out and kept before the core sees it (`README.md`, section 3).
- **The application owes:** its transport, or jig's default mapping; its
  sign-in flows, with `Mint` sent from its server alone (`sign-in.md`); the
  printed link's URL, printed from the secret the component gives; its
  small store's storage, answering each write; and its server's guards
  (`../browser/README.md`, 12.2).
- **The engine's domain** is owed nothing beyond
  `../../client/domain/contract/README.md`, section 7: sign-ins numbered by their
  commit, made for an identity the application verified, and a party
  ending theirs (`../../domain/people/README.md`, section 3); watches and reads as
  `../../domain/engine/views.md` has them.
- **skein owes,** for one person's machine: skein-lib, skein-codec and
  skein-http, with its server-sent events reader and writer, built for
  `wasm32-unknown-unknown` (skein's `runtime/lib.md`, section 2; skein's
  `testing/README.md`, section 6); for the reference UI, its UI kit's browser
  parts, its bundler, and its face and harness for the browser tier
  (skein's `ui/web/README.md`, 4.1, and `ui/web/bundle.md`, 2, and `ui/web/testing.md`, 2), over its browser kit
  (skein's `testing/browser.md`, sections 5 and 6); a client compiled to
  WebAssembly in its model (`skein/docs/foundation/programming-model.md`, 2.1). Its HTTP
  server and client serve as they are. Between hosts, TLS's server side
  (`../README.md`, section 10).
