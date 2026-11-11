# The client

Provisional, 2026-10-09, revised 2026-10-23. `jig-client`: the client an
application holds, as a component, to reach jig's engine. It holds the
link, the watches the application names by handle and the latest
document of each, the application's keyed requests in skein-lib's
request table, the work kept across a restart, and the application's own
families, carried as bytes. It knows no page and no HTTP: what a party
sees is the application's presentation, or jig's reference UI
(../pages.md, ../objects.md, ../view.md); what the client says is jig's messages,
which the application's transport carries (`../../../protocol/client/README.md`).
How an application's client root holds it is ../root.md; the API it speaks
is ../contract/README.md; jig's helpers are modules of it (`helpers.md`). What is
still open is listed in `README.md`, section 15.

## 1. In one page

- **A step machine the application holds** (skein/docs/foundation/programming-model.md,
  section 3): `jig_client::Client`, with its own vocabulary, limits
  and world. The application steps it with each event, and
  it gives back what to send and what to tell. It runs no loop of its
  own, and it never calls the application (section 4).
- **Presentation-agnostic.** It knows no page, place, card, confirmation
  or draft. A browser, a terminal, a chat bot, a one-shot command and a
  service with no presentation hold the same client (section 16). jig's
  reference UI is one presentation, built on the public client alone
  (../pages.md, ../view.md).
- **Watches by handle.** The application names each watch and read by a
  handle. The client keeps the latest document of each watch, and says
  when it changed; the application reads it when it likes. Several
  handles on one subject share one watch on the engine (`watches.md`).
- **Keyed requests, through skein-lib's request table.** Ordinary
  requests (an operation, a compound request, or a family's request as
  bytes) are kept in the small store before sending. Top-level `Seen`
  stays in memory only. Both use the table's keys, joins, conflicts,
  names, retries and one answer while the client lives (`requests.md`).
- **Work kept across a restart.** The small store holds kept requests
  not yet answered. At start, each is restored and told to the
  application as kept. An unanswered `Seen` is silently lost on closing,
  reload or crash; the next opening checks and sends the forward-only
  watermark again (section 8).
- **The link is the client's to judge,** from the stream and its own
  watch of `me`: signed out, starting, live, offline, or out of date
  (`watches.md`, 2).
- **Signing in by `SignIn { with }`:** a printed link's secret, a
  service's credential, a credential the application obtained by its own
  sign-in, or one its transport carries unseen. jig owns credentials;
  every sign-in flow beyond its two is the application's (section 9).
- **The application's families travel as bytes,** under their owner's
  number and version, on the same connection and through the same
  table. The application's wire crate encodes and decodes them; the
  client never reads them (`README.md`, section 4 and `requests.md`; ../root.md, section 7).
- **jig's messages, not HTTP.** The client gives jig's messages to a
  transport the application chooses: jig's default HTTP mapping, a local
  socket, or an in-process channel (section 13).
- **Built for the host and for wasm32,** as step code under the strict
  subset (skein/docs/foundation/programming-model.md, section 10).
- **Helpers beside it, optional:** shapes of work from templates, facts
  over documents, a conversation's thread, and jig's words, each built
  only from the public client (`conversations.md` and `helpers.md`).

## 2. The client in the system

```
presentation   the application's: routes, screens, words, look, confirmations, drafts; or jig's reference UI
helpers        jig's, optional: shapes, facts, thread, words (section 19)
jig-client     the link, watches by handle, the request table, kept work, families as bytes
transport      the application's: jig's default HTTP mapping, a socket, or in-process (section 16)
the API        jig's messages: operations, subjects and reads, documents with revisions and offers (contract.md)
the engine     the application's root around jig's core, and jig's clients' component
```

- **Two translations, one per side.** Toward the engine, the client
  speaks jig's messages, encoded by `jig-client-protocol-link`. Toward
  the application, it speaks the API's typed values: operations,
  subjects, reads, documents and answers.
- **Nothing below the presentation knows a page,** and nothing of the
  client's knows HTTP. The helpers use the public client only, as the
  application does.
- **The client has no store of the truth.** The engine's store is the
  truth. The client holds the latest document of each watch, and a small
  store of what it must not lose across its own restart (section 8).
- **A party may have many clients:** tabs, devices, a command. Each is
  its own client, with its own keys and its own small store. Two of them
  deciding one thing are two requests to the engine: the first commit
  decides, and the second is told who decided and how
  (`../../../domain/people/acts.md`, 1).

## 3. Structure

```
jig-client                        jig_client::Client: the domain and the link, composed; the helpers as modules
├── jig-client-domain             the client's domain
└── jig-client-protocol-link      jig's messages and the small store's records, encoded and decoded; the stream's events
    ├── jig-client-protocol-wire  jig's wire records, for both ends (../../../protocol/client/wire.md, section 1)
    └── jig-protocol-transcript   turns decoded with smith's transcript crate (section 10)
```

- **`jig-client-domain`** is a step machine like every jig domain:
  `#![no_std]`, depending on skein-lib alone. It holds:
  - *the API's types:* operations, subjects, reads, documents and
    answers, as ../contract/README.md gives them;
  - *the link:* its state, the sign-in it is on, the stream (watches.md, 2, and README.md, 9
    ). The credential's value is never domain state
    (`../../../protocol/README.md`, section 8): `jig-client` holds it
    beside the domain, as an engine's protocol layer holds its secrets,
    and puts it on each `Exchange` and `OpenStream`;
  - *handles:* each watch's and each read's, with its asker (section
    4), each watch mapped onto a watch of the engine's with its latest
    document (`README.md`, section 6 and `watches.md`);
  - *the request table,* skein-lib's `RequestTable` (skein's `runtime/lib.md`,
    section 12), with each request's key and asker (`requests.md`);
  - *kept work:* the requests restored at start (section 8);
  - *its timers and its seed,* from which the table's keys, the engine
    watches' names and every jitter are drawn.
- **`jig-client-protocol-link`** (`../../../protocol/client/wire.md`, 2.3)
  translates between jig's wire records
  and the domain's types, encodes and decodes jig's messages and the
  small store's records, judges the stream's hello against the versions
  the client speaks, and decodes turns through `jig-protocol-transcript`.
  It knows no HTTP.
- **`jig-client`** composes the two into `jig_client::Client`, one step
  machine, and holds jig's helpers as modules: `shapes`, `facts`,
  `thread` and `words` (`helpers.md`). It depends on `skein-ui` for its
  `words` module alone, whose phrases are skein-ui's phrasebook types;
  it builds no tree, so the client stays presentation-agnostic.
- **Internal hand-offs are uncapped.** The domain's and link's output
  queues and the deferred-read queue grow as they fill. Each event's
  fan-out is bounded by admitted state, so the settlement drain ends
  (skein/docs/foundation/programming-model.md, 7.2).
- **Link consequences settle before the next entrance.**
  `link_requests_dispatched` requires link outputs to be dispatched
  before the next domain request, including a credential boundary's
  consequences before a deferred read is sent. `client_settled`
  requires both output queues to be empty when the step returns; only
  deferred reads may wait on exchange credit. The client world holds
  this under load in `jig-client-world`,
  `orders::link_consequences_settle_before_the_next_entrance_under_load`.
- **Builds:** the three crates and the helpers are step code under the
  strict subset, built for the host and for wasm32 (skein/docs/foundation/programming-model.md,
  section 10). Only `jig-protocol-transcript` links smith, the client's
  only use of smith (`../../../README.md`, section 10).
- **Its handles inside are skein-lib's:** its watches, reads and
  exchanges are `Id`s in slabs sized at startup, and what crosses to the
  transport is their tokens (skein/docs/foundation/programming-model.md, 4.2). A terminal is
  never stale: a slot is retired only once its terminal has come.
- **Beside it, the application's to use or not:**
  `jig-client-protocol-http`, the default HTTP mapping's client end;
  `jig-client-shell-native`, a native shell over skein's HTTP client;
  and `jig-client-view`, the reference UI (../README.md, section 2).

## 4. Its vocabulary toward the application

```
jig_client::Client: step(now, wall, Event, &mut Queue<Request>), fire(now, wall, &mut Queue<Request>), next_deadline()

Event, in
  from the application:
    Start { kept }                                the small store's records, as the host loaded them
    SignIn { with } | SignOut                     with: Link { secret, held } | Service(credential) | Held(credential) | Transport
    Watch { handle, subject, window } | Unwatch { handle }
    Read { handle, read, cursor }
    Send { key, name, request }                   an operation, a compound request, or a family's request as bytes; name optional
  from the transport:
    Reply { exchange, message } | Lost { exchange }
    Streamed { message } | StreamGone
    Stored | NotStored
Request, out
  to the transport:   Exchange { exchange, credential, message } | OpenStream { credential } | CloseStream | Save { records } | Erase { records }
  to the application: Link(state) | Changed { handle } | Ended { handle, why } | Page { handle, page }
                      Pending { key } | Answered { key, outcome } | Kept { key, request } | Dropped { count } | Text { handle }
read by the application: client.document(handle), client.text(handle), client.pending(key), client.link()
```

- **`now` and `wall`** are the monotonic instant and the wall time, as
  every step machine takes them (skein/docs/foundation/programming-model.md, section 9:
  `env.now` and `env.wall`).
- **Handles and keys are the application's.** Each names its asker and a
  number of the asker's. The asker is the owner number of the child of
  the client root that asked (../root.md, section 15; jig's reference UI's
  is 65536): the client keeps it, hands it back with everything it says
  of that handle or key, and never reads it, as skein-ui keeps an owner
  (skein's `ui/README.md`, 3.3). The client root routes by it, and keeps no map
  from handles or keys to its children (../root.md, section 5).

### 4.1 What it hears

- **From the application:**
  - *`Start { kept }`:* the small store's records, as the host loaded
    them, before any other event (section 8);
  - *`SignIn { with }` and `SignOut`* (section 9);
  - *`AskApproval`, `ReadApproval { code }` and `DecideApproval { code,
    approve }`:* an approval's ask, read and decision (section 9);
  - *`Watch { handle, subject, window }` and `Unwatch { handle }`:* a
    subject of jig's (../contract/README.md, 5.1), or a family's as bytes, and the
    window its document holds (`watches.md`);
  - *`Read { handle, read, cursor }`:* one document, or one page of a
    list from a cursor (section 6);
  - *`Send { key, name, request }`:* an operation, a compound request,
    or a family's request as bytes, with a name or none (`requests.md`).
- **From the transport:**
  - *`Reply { exchange, message }`:* the one record that answers an
    exchange; *`Lost { exchange }`:* none came whole;
  - *`Streamed { message }`:* one event of the stream, in order: the
    hello, a delivery, a heartbeat, a watch ended; *`StreamGone`:* the
    stream ended;
  - *`Stored` and `NotStored`:* the last `Save` or `Erase` written, or
    not (section 8).
- **Time:** `fire(now, wall, ..)`, at the deadline `next_deadline()` gave
  (section 10).

### 4.2 What it asks for

- **Of the transport:**
  - *`Exchange { exchange, credential, message }`:* one of jig's messages
    (`../../../protocol/client/wire.md`, 2.1) with the credential held, or
    none; answered by one `Reply` or one `Lost`;
  - *`OpenStream { credential }` and `CloseStream`:* the one stream every
    watch's deliveries come on;
  - *`Save { records }` and `Erase { records }`:* the small store's,
    written before the requests they keep are sent (section 8).
- **Of the application,** told, never asked to act:
  - *`Link(state)`:* the link's state, whenever it changes, and when a
    sign-out does not reach the engine (watches.md, 2, and README.md, 9): what a
    presentation shows of the link, which it reads with `client.link()`;
  - *`Changed { handle }`:* a newer document for the handle, or the
    handle behind or live again (`watches.md`, 1);
    *`Text { handle }`:* more of a run's text (`watches.md`, 3); *`Ended { handle,
    why }`:* the handle's one terminal, and why (`watches.md`, 1);
  - *`Approval(answer)`:* an approval's ask, read or decision answered,
    once each (section 9);
  - *`Page { handle, page }`:* a read's one terminal (section 6);
  - *`Pending { key }`:* the request's progress changed; *`Answered {
    key, outcome }`:* its one answer; *`Kept { key, request }`:* a
    request restored at start (`requests.md` and `README.md`, section 8); *`Dropped { count
    }`:* how many kept records were dropped at start, unread, under no
    asker (`README.md`, section 8).

### 4.3 What it never asks

- **It never calls the application.** What it tells is a value in the
  queue, which the application takes when it steps it. What it holds,
  the application reads:
  - `client.document(handle)`: the handle's latest document, and its
    state (`watches.md`, 1);
  - `client.text(handle)`: the run's text it holds for the handle, the
    one turn being written (`watches.md`, 3);
  - `client.pending(key)`: the request's progress, its party and its
    owner (`requests.md`);
  - `client.link()`: the link's state, and the credential held (watches.md, 2, and README.md, 9
    ).
- **It never reads an application's value.** A family's request, read,
  subject, page and document are bytes under their owner's number, which
  the application's wire crate encodes and decodes; the client reads
  their owner, their version and their size, and nothing more. A part
  that one of jig's documents names stays a reference (../root.md, section
  6).
- **It never decides for a presentation:** what to show, when to
  confirm, what to keep of a draft. It gives the facts as data: the
  documents with their revisions and offers (../contract/documents.md, 3), and each
  request's progress and answer.

### 4.4 Without a link

`Client::unlinked` makes a client that holds no link: its host carries
the domain's work in the link's place, and answers it with `Up`. Each
mode is stepped by its own pair, and each pair has its own request type:

```
Client::new:      step(now, wall, Event, &mut Queue<Request>), fire(now, wall, &mut Queue<Request>)
Client::unlinked: step_unlinked(now, wall, Event, &mut Queue<UnlinkedRequest>),
                  fire_unlinked(now, wall, &mut Queue<UnlinkedRequest>)

UnlinkedRequest, out
  Told { told }               as a linked client's
  Authorized(authorized)      a printed link's authorization, always lost: no link signs in
  ApprovalAct(act)            an approval's ask, read or decision, which the host carries (section 12)
  Approval(answer)            its answer, as the host gave it back
  Down { down }               the domain's work, which the host carries: a save, an erase, a send,
                              a watch opened, continued or closed, a read
  SignOut                     the sign-out, which the host answers
Event, in, besides the application's: StartUnlinked { kept } in place of Start, Up { up }, SignOutLost,
  ApprovalAnswer { answer }
```

- **A linked client's `Request` holds no `Down` and no `SignOut`:** what
  its domain asks goes to the link, and its sign-out is an exchange. A
  linked host matches only what a linked client asks.
- **A pair given the other mode's client is a fault** (an assertion), as
  an event of the other mode is.
- **An application root stepping either mode** keeps the two apart in its
  own requests: what the linked client asks, and what the unlinked one
  asks, each under its own variant.

## 5. Pages and the working set

Pages, places, navigation and the working set are the reference UI's,
`jig-client-view`: ../pages.md, sections 2 and 12; ../objects.md, section 2;
and ../view-state.md, 2 and 3. The client holds no object by key: it holds
the latest document of each watch, by handle (`watches.md`).

### 5.1 Sections

Sections and arrangements are the reference UI's: ../pages.md, 12.2, 12.3
and 12.5. What a section reads and watches, it opens through the client's
handles, as every presentation does (`README.md`, section 6 and `watches.md`).

## 6. Reads and windows

- **Deferred reads leave in order.** `deferred_reads_dispatched`
  requires current deferred demands to keep FIFO precedence over a
  fresh read when exchange credit returns. Cancelled or obsolete
  demands cannot return. The client world holds this under load in
  `jig-client-world`, `orders::deferred_reads_leave_in_order_under_load`.

- **A read** asks for one document (a task, a proposal, `me`, a
  family's) or one page of a list (a task query's lines, a task's
  activity or history, the inbox's group), and has one terminal, `Page
  { handle, page }`: the document, or the page with its cursors, or why
  there is none: unknown, gone, too large with its size, malformed,
  refused, busy, signed out, version not served, or unreachable, when
  its exchange is lost or not sent for want of room (gone and too large for a
  turn read alone, ../contract/README.md, section 4; malformed for a turn that does not decode, and
  for any read whose answer does not, section 13). A read is one attempt
  (`../../../protocol/client/README.md`, section 4): the application reads again
  if it wants.
- **What is live is watched; what is past is read.** The engine holds
  live work only (`../../../domain/engine/README.md`, 5.3), so ended tasks, history
  and transcripts are pages read from its store, a page at a time, as
  the application asks.
- **A window** is what a watch's document holds of a list, or a read's
  page: its size, its order and where it starts, by cursor. The
  application gives it; the engine bounds it in bytes and rows, never in
  rows sized for one client's screen (../contract/documents.md, 3, and ../contract/README.md, 4.1 and 5.5).
  A window past the engine's rows is answered within them, never
  refused: the document or the page holds the rows the bound admits and
  says there are more, by its cursor for the next and, for a tree, by
  its folded counts (../contract/README.md, 5.2). So an application asks for the
  window it can show, and never needs the deployment's bounds first.
  Keeping rows across pages, as a list slides while a person scrolls, is
  the presentation's.
- **One read in flight per handle.** A `Read` on a handle whose read is
  in flight abandons that read, and so does `Unwatch` on the handle; an
  abandoned read's terminal is dropped when it comes. A full slab of
  reads answers the read at once, refused by the client's limit.
- **Credential boundaries end reads visibly.** On sign-out and on
  every successful sign-in, the domain removes each pending read and
  emits its one `Page`: `Jig(ReadResult::None(ReadNone::SignedOut))`,
  or `Family { family, record: None }`. This is unconditional, including
  before the first `me`. There is no automatic replay; whoever asked
  must ask again. A failed, refused, lost or not-ready sign-in that
  installs no credential leaves old reads pending. Reads made during
  an exchange use the old credential until successful installation.
- **Pending ownership** is internal: `PendingRead { token: Token,
  family: Option<u16> }`. A response before the credential boundary
  wins; the boundary before a response emits cancellation. Token
  matching prevents a late reply or loss from completing a
  replacement read under the same handle. A read under an old credential
  never answers under the new one. Supersession and `Unwatch` still
  abandon reads as above; credential boundaries do not.
- **A family's read** names its owner, its scope (a project, the
  deployment, or the party's own) and its record's bytes (../contract/README.md,
  5.5); its page is its owner's, as bytes.
- **A read of an owner not served** (`watches.md`, 2), a family's outside its
  owner's range or any read while the link is out of date, is answered
  at once and never sent: version not served, naming the owner and the
  range the last hello named of it, or none where it named none.

## 7. Drafts and confirmations

Drafts and the confirmation are the presentation's: the application's
own, or the reference UI's (../view-state.md, 5; ../objects.md, section 2). What
a confirmation says is data in jig's documents (../contract/documents.md, 3 and
4); what holds below it is the revision a decision names and the
request table's once (`requests.md`); jig's check is ../testing.md's, 6.2.

## 8. The small store

- **What the client keeps across its own restart:** the requests not yet
  answered, except top-level `Seen` and requests carrying a value
  (`requests.md`), as the request table's records, each with the key the
  engine knows it by, its party, the application's key, the request (jig's record, or a family's
  bytes under its owner's number and version), and the wall time of its first
  sending; and the name of its stream, which the client that follows
  sends gone before it opens its own
  (`../../../protocol/client/wire.md`, section 2, and `../../../protocol/client/README.md`, section 7). Nothing else of the
  client's: everything shown is read again. A presentation's own records
  (the reference UI's drafts and its place) are the application's, which
  it keeps beside the client's (../root.md, section 8; ../view-state.md, 8).
- **Kept requests before sent.** An ordinary request's record leaves in a
  `Save`, and the client gives its first `Exchange` only once the transport
  says `Stored`. `NotStored` (a quota full, a disk failing) retires the
  requests the write carried, unsent, each answered *not kept*, by its
  key (skein's `runtime/lib.md`, 12, not kept): parked, or with its `Exchange`
  waiting for the write, since a parked request's record leaves at its
  admission too. Its erase leaves as any retired request's, since an
  earlier write may hold its record; if that erase is not stored either,
  a restart tells the request `Kept` again, under its key. A record is
  whole in each write, so a request whose record a later write carries
  again is decided by that later write, and one whose record the write
  did not carry is sent once its own write is stored. So a restart never
  loses an ordinary kept request the engine may have made. A request
  carrying a value is sent once without a record, and its caller asks
  again (`requests.md`).
- **A later write holds its requests.** `request_records_durable`
  requires pending writes to finish before held requests are sent; an
  earlier write's completion cannot release a later write's requests.
  A failed write ends its own obligations as not kept. The client
  world holds this under load in `jig-client-world`,
  `orders::a_later_write_holds_its_requests_under_load`.
- **The durability hold is uncapped.** Held exchanges are bounded by
  admitted attempts, never refused as not kept for a full hold. Once
  the write is durable, live attempts leave in FIFO order; stale
  attempts are skipped. Not kept reports a store failure, not internal
  queue room (skein/docs/foundation/programming-model.md, 7.2).
- **`Seen` has no save of its own.** Closing, reload or crash silently
  loses an unanswered in-memory mark, with no `Kept`, `Dropped` or not
  kept notice from its own store. The next opening checks and sends the
  watermark again; repeating a forward-only read position is safe. Its
  exchange still waits behind writes already given. A failed mixed
  write retires only the ordinary requests whose records it carried as
  not kept; a `Seen` exchange held behind it is lost and follows its
  normal retry. Consuming the table's save creates no tracked store
  write and changes no store acknowledgement protocol.
- **Every terminal erases.** The table's terminal `Erase` holds for
  `Seen` too: erasing an absent key is harmless, and its store
  acknowledgement completes normally. A `Seen` record the small store
  holds is not read: it is dropped, counted in `Dropped`, and its key
  erased at start.
- **Restored at `Start { kept }`,** before any other event: each
  request's record admitted to the table under its key, parked until
  the link is live, its party confirmed and the retention known; and
  each told to the application as `Kept { key, request }`, which the
  client root routes by the key's asker (../root.md, section 8).
- **It names its party.** Each record carries the party it is for, and
  its request is sent, under its original key, only once `me` confirms
  that party. While another party is signed in, it stays parked and
  unsent, until its party returns or the retention passes and it is
  answered unknown. Whether, and how, to warn of someone else's kept
  work is the presentation's (the reference UI's: ../pages.md, section 8).
  A `Send` before `me` has named the party is answered unconfirmed at
  once, and nothing of it is kept (`requests.md`).
- **Versioned** with jig's store version, so a client updated since reads
  what an older one kept. A record of a version it does not read is
  dropped, and the application told (`../../../protocol/client/README.md`, section
  7). A family's request is kept in its owner's version: one whose
  owner the build does not declare, or whose version is not the one
  declared (`watches.md`, 2), is such a record. So is one past the records the
  table holds (section 11). The client tells only how many, `Dropped {
  count }`, once at start, under no asker: nothing of a record it
  does not read is read, its key's asker included. The client root
  gives the count to the presentation that says so (../root.md, section
  8): the reference UI's notice (../view-state.md, 7).
- **Where it lives is the host's:** a browser tab's session storage, a
  file written whole and synced, a world's memory. A host whose store is
  copied (a browser tab duplicated) sends the same keys from both
  copies, whose ordinary deciding acts the engine makes once, while each
  copy draws new keys from a seed of its own; the copy sends the original's
  stream gone, once, which the original takes as a drop (`watches.md`, 2).
- **It is never the truth.** An ordinary deciding request kept is
  answered by the engine by its key, made once whatever the client did
  before.

## 9. Signing in

- **jig owns credentials, not sign-in flows** (../contract/README.md, section 2).
  Its own three ways of signing in are the printed link's one-time
  secret, on loopback, a service's credential, and a sign-in approved
  by a signed-in browser (below). Every other way is
  the application's sign-in, on its own routes, ending in a `Mint` its
  server gives the clients' component (`../../../protocol/client/sign-in.md`
  ).
- **`SignIn { with }`:**
  - *`Link { secret, held }`:* the printed link's secret, beside the
    credential the application holds from an earlier sign-in, if any.
    The client sends the component's sign-in with the secret, carrying
    `held` as its credential, and holds the credential the answer gives;
    an answer that gives none keeps `held`, the sign-in the engine kept
    (`../../../protocol/client/sign-in.md`), unless the transport keeps
    it: a browser's `held` is none, its cookie carried by the transport.
    An answer refused, not ready, busy, full, why a link can't sign in
    or lost installs nothing, and `held` stays the application's to give
    again. Not ready, busy and full leave the link's secret usable until
    its expiry (`../../../protocol/client/sign-in.md`), and the link's
    view says which, as `not_signed_in`: not ready, busy, or full with the
    deployment's room for sign-ins, the earliest saved expiry among
    them and the link's own expiry, for the application's words. It says
    too why a link can't sign in, `link_used`, `link_expired` or
    `link_unknown`, as the answer did, and these are not sent again. The client keeps no secret and
    signs in again by itself never: the person's retry is the
    application's, the same `Link { secret, held }` given again, as a
    native shell does, or the printed link opened again, as a browser
    does. The next `SignIn`, and signed in, clear `not_signed_in`;
  - *a service's busy* ends its signing as a refusal does: a service
    waits for no room, and is never answered full;
  - *`Service(credential)`:* a service's credential, sent the same way,
    and answered with a sign-in's credential, held the same way;
  - *`Approval { secret }`:* an approval's take, its secret the one
    `Asked` gave (below), sent with no credential. Answered signed in,
    the client holds the credential the answer gives, as a link's. While
    it is answered waiting, the client sends the same take again by
    itself after a second, `APPROVAL_AGAIN`, and tells `not_signed_in`
    `Approval(Waiting)`; not ready, busy and lost take the backoff a
    link's sign-in takes, and `not_signed_in` says not ready for not
    ready and busy for busy and for lost, so its application tells a
    lost take from one not yet answered. Any other answer ends it,
    installing nothing:
    `not_signed_in` says `Approval` with why (refused, expired, used or
    unknown), or full with the deployment's room, the earliest saved
    expiry and the approval's own expiry in place of the link's. A next
    `SignIn` or a `SignOut` ends it too. The client keeps the secret in
    memory while it takes, never in the small store, and drops it at
    the end;
  - *`Held(credential)`:* a credential the application obtained by its own
    sign-in, which the client holds as it is;
  - *`Transport`:* the transport carries a credential the client never
    sees, as a browser carries a cookie its server set.

  The `credential` on `Exchange` and `OpenStream` is the one held, or
  none; a `Link` sign-in's is its `held`. Signed in, the client opens
  the stream, watches `me` (`watches.md`, 2), and opens the watches asked before
  (`watches.md`, 1); under `Transport` it is signed in from its `SignIn`. So a host
  may sign in after `Start { kept }`, its children watching from their
  start.
- **An approval, asked and decided** (`../../../protocol/client/sign-in.md`
  ), each one exchange whose one answer the client tells the
  application, `Approval`, and keeps nothing of:
  - *`AskApproval`,* sent with no credential, by a client signed out:
    `Asked { secret, code, expires }`, the secret the application's to
    give back in `Approval { secret }` and never show, the code to
    show; or `NotAsked`, refused, not ready, busy or lost;
  - *`ReadApproval { code }` and `DecideApproval { code, approve }`,*
    sent with the credential held, or none under `Transport`, by a
    browser's client: `Read { code, state }`, the approval waiting
    with when it was asked, approved, taken, refused, expired or
    unknown; `Decided { code, decided }`, approved, refused, used,
    expired or unknown; either not a browser, signed out, not ready,
    busy or lost. Answered signed out, the client is signed out, as by
    any exchange (below);
  - *the code* is `ApprovalCode`, the wire's (`../../../protocol/client/sign-in.md`
    ): eight letters of its alphabet, read from what a person
    types or an address carries, either case, hyphens and spaces
    ignored, and shown as four letters, a hyphen and four.
- **The credential is never in the small store.** It is the link's, and
  `client.link()` gives it: a host that keeps a sign-in across its own
  restart keeps the credential where its platform keeps secrets, and
  gives it back with `Held`.
- **Signed out:** an answer, a read's terminal, a watch's end or the
  stream saying so drops the credential held, ends every watch handle
  (`Ended`, signed out), visibly ends every pending read (section 6),
  parks every request, and tells `Link` signed out. The application
  signs in again as it likes, and the parked requests are sent once the
  same party is confirmed, within the
  retention (`../../../domain/people/acts.md`, 1).
- **`SignOut`** sends the component's sign-out with the credential held,
  or none under `Transport`, to end the sign-in on the engine. Answered
  signed out, the client drops the credential and is signed out, as
  above.
- **A sign-out that does not reach the engine** (lost, or answered not
  ready while the engine stops) leaves the party signed in, as they
  still are on the engine: the client keeps the credential and its
  handles, and the link says the sign-out did not reach the engine.
  `SignOut` again sends it again, which is safe: a sign-out whose
  sign-in has already ended is answered signed out, the end asked for.
  Signed out meanwhile, whatever ended the sign-in, the client is signed
  out, as above, and the link no longer names the sign-out.
- **A sign-out's answer is its sign-in's.** The client knows which of
  its sign-ins each exchange belongs to. Once a newer sign-in is
  live, an earlier one's sign-out answer is dropped, and the newer
  stands. Nothing else the earlier sign-in owes reaches the newer one:
  going signed out parks every request, visibly ends every pending read
  and ends every watch handle. Successful sign-in also ends all pending
  reads before opening `me`, even before the first sign-in's `me`.
  Thus an earlier attempt's answer is stale to the request table, a
  completed or cancelled read's late terminal fails its token match,
  and an ended watch's deliveries are dropped by their names (`requests.md` and `README.md`, section 6 and `watches.md`
  ). The link erases old exchange names; the domain owns read
  cancellation, with no new link output, public `Readout` variant or
  wire record.
- **A service's credential, when an owner makes one,** reaches the
  application once, in that request's `Answered` outcome; the client
  keeps no copy of it, and the small store holds no answer
  (`../../../protocol/client/sign-in.md`).
- **A bot relaying a person's decision** holds that person's own
  credential, from the application's sign-in (`Held`), and the decision
  is the person's request, naming the revision the bot showed them: jig
  adds no acting for another (`../../../domain/people/README.md`, section 3).
- **Keys are the party's across sign-ins,** so the parked requests are
  sent again once the same party is confirmed, within the engine's
  retention.

## 10. Time and notices

- **Deadlines are the client's:** the backoffs of requests and of
  opening the stream and its watches again, the stream's heartbeat
  tolerance, and the retention's cutoff, on the monotonic clock
  (skein/docs/foundation/programming-model.md, section 9). `next_deadline()` gives the
  earliest, and `fire(now, wall, ..)` runs what is due.
- **Wall times are for records and for showing.** The request table
  records the wall time of a request's first sending, so its cutoff holds
  across a restart; documents carry wall times, which a presentation
  shows (../contract/documents.md, 3).
- **Notices,** a presentation's short-lived words for what has no other
  place, are the reference UI's (../view-state.md, 7). The client tells
  only the facts they are made from: answers and the link.

## 11. Limits and envelopes

Each is configured and bounds what enters or is retained:

- handles; the engine's watches open, `me`'s included, each until its
  terminal and one kept for `me` (`watches.md`, 1); reads in flight; exchanges in
  flight;
- rows: the most items of a page, a window and lists without a bound
  of their own, at most 32, the ceiling of the wire's lists
  (`../../../protocol/client/wire.md`, section 1); a document past it is refused at decoding (`README.md`, section 13),
  and limits above it at construction (below, on a knob and its class);
- a path's segments: the most segments of a resource's path or a
  pattern a document may carry, a bound of its own, never a window's
  rows; the application sets it at least the most its engine admits in
  a name (../contract/documents.md, 3), and a document past it is refused at
  decoding (`README.md`, section 13);
- an authority's entries: the most entries of one of an authority's
  lists (its grants, its executors, its notes' scopes) or of an amending
  offer's (its grants listed, its implications) a document may carry, a
  bound of its own, never a window's rows; the application sets it at
  least the most its engine admits in each of them (../contract/documents.md, 3),
  and a document past it is refused at decoding (`README.md`, section 13);
- account counts: accounts per list, sources per account and sign-ins
  per list, independent of rows. The application sets each at least its
  engine's document lines (`../../../domain/accounts.md`, 3.6); a document
  past one is refused at decoding (section 13). Uses keep their constant
  bound of 16 (`../../../domain/accounts.md`, section 3);
- bytes of a document held, of a page handed over, and of a run's
  provisional turn, each priced from the wire's worst cases at the
  client's limits (`../../../protocol/client/README.md`, section 10);
- the request table's: requests held, and their bytes, a family's
  included (skein's `runtime/lib.md`, section 12); the records restored at
  start, no more than the table holds and the stream's name (section
  8);
- backoffs' first and most spans, the heartbeat's tolerance, the
  retention's margin;
- the family owners the application declares, each once, with its
  version (`watches.md`, 2): no more than a hello's ranges hold beside jig's
  (`../../../protocol/client/README.md`, section 6).

`DomainLimits.turn_text` is one committed turn's shared text budget in the
transcript, priced in this order: its calls' names; its texts, an opaque block's
dialect with them; its calls' outcomes, a line each within `text`; its calls'
inputs; its results' summaries; then its retained system context. An outcome comes before the inputs because a
call's input can fill the budget, a note's whole body, and what reads a call's
outcome needs its line. It is at least the receiving field bound `text` and at
most `turn_bytes`; limits outside those bounds are refused. It bounds the
turns a page carries; a turn read alone is decoded whole (`conversations.md`).

`DomainLimits.long_text` bounds what a person may read or send whole past a
field's `text`: a note's body, a document's text and a task's words, the words of
an activity line (a `Words` line, a word heard or notified) and of a `Message` or
an answer to a question, and what one delivery of a run's text carries, within the
wire's ceiling for it. It is at least `text` and at most `document_bytes`, and a
request's `ask_bytes` holds it. A line holds what the store took, whoever wrote it,
so a deployment whose `tasks.message_bytes` or `tasks.result_bytes` pass `long_text`,
or whose page of lines passes `document_bytes`, has that page refused (section 13).

Reads have room independently of watch handles; a read need not own a
watch. Pending entries price both their token and optional family. A
credential boundary may emit one `Page` per pending read, in addition to
watch and request outputs. This fan-out is bounded by admitted reads;
it sizes no internal queue and reserves no room.

- **No client watermark.** Each event's fan-out is bounded by admitted
  state, and new work enters through capped edges: the link's exchanges,
  the kernel's queues, and terminal or browser input. The reference
  client root, browser host and application's scratch queues use
  uncapped internal hand-offs. The native shell's io, file and SSE
  hand-offs are uncapped too; only the kernel's queues and their staging
  stay capped (skein/docs/foundation/programming-model.md, 7.1).

Containers grow as they fill; fixed scenarios check their measured heap
peak against a named ceiling (../testing.md, section 6). Each held document
is charged at twice its wire byte limit as room for its decoded
representation: fixed-size
records and lists reserved at their bounds. A deployment of 169 wire
bytes holds 512 decoded bytes, so a document limit of 169 gives only
338 bytes of room and refuses it; it needs a document limit of at least
256. The link refuses an accounts or deployment document whose decoded
charge passes its room (section 13).
Each helper's limits are its own (`helpers.md`); the reference UI's derive
from the client's and its own (../view-state.md, 9).

**A knob sizes a class of the wire's bounds.** Rows, a path's segments, an
authority's entries, text and a document's bytes each set every wire bound
of their class at once, in one derivation by class. A knob past its class's
ceiling is refused at construction. A member the schema bounds below its
class keeps its own ceiling: that ceiling is the schema's, not a limit of
the client's, and no configuration is refused for it.

## 12. What it never knows

- **HTTP and its platform:** no path, cookie, header, host or event
  stream; no browser, window or terminal. The transport and the host are
  the application's (section 13).
- **A page:** what is shown, where and how; what a person is in the
  middle of.
- **An application's values:** its families' requests, subjects, pages
  and documents are bytes under their owner's number; a part named in
  one of jig's documents is a reference (../root.md, section 6).
- **Templates:** a template is the application's data, which `me` serves
  and a helper or the application fills into an ordinary `Create` or
  `Propose` (../contract/requests.md, 6; `../../../protocol/configuration.md`, 3.1).
  The client sends what it is given.
- **The engine's internals:** tasks by number, documents by subject and
  revision, outcomes and refusals as the contract gives them. Authority
  is never computed here: what a party may do is what the engine's
  documents offer, and the engine checks it again.

## 13. Below the client

What the transport and the clients' component owe the client, which
the client's protocol designs (`../../../protocol/client/README.md`):

- **The transport is the application's:** jig's default HTTP mapping
  (`jig-client-protocol-http` at the client's end,
  `jig-protocol-engine-clients-http` at the engine's;
  `../../../protocol/client/default-http.md`), a local socket for a command
  line, or an in-process channel for one process or a world.
- **One terminal per exchange:** each `Exchange` answered by one `Reply`,
  with the record that came, or by one `Lost` when none came whole
  within the exchange's deadline, the transport's. The transport makes
  one attempt; retrying is the client's.
- **The stream in order:** its events from the hello on, each a
  `Streamed`, in the order the component wrote them; then one `StreamGone`,
  whatever ended it.
- **The small store in order:** each `Save` and `Erase` written, synced
  as its platform allows, and reported `Stored` or `NotStored` before the
  next.
- **The credential as given:** the one on `Exchange` and `OpenStream`
  carried with its message, or, under `Transport`, the transport's own.
- **The clients' component,** at the engine's end, takes jig's messages
  with their credentials, answers each exchange with one record, and
  writes the stream (`../../../protocol/client/wire.md`, 2.2). It knows no
  HTTP. A read's answer it cannot encode within its wire's bounds is
  answered malformed: closing the exchange would read as a lost one, which
  a client asks again.
- **Documents decoded whole,** into the client's types, and checked
  against the client's limits before they reach its domain
  (skein/docs/foundation/programming-model.md, 4.4); a document past them is refused at
  decoding, as too large, never cut silently. A path is held whole,
  within the client's bound on its segments: only the view cuts what it
  shows of a long one (../view.md, section 7). An authority is held whole,
  within the client's bound on its entries, and a draft is made from it
  whole: only the view cuts what it shows of a long one (../view.md,
  section 7). An account list is held whole, past the client's rows,
  within its account, source and sign-in counts and its decoded room:
  what its rows decode to is charged against the bytes the domain holds
  for a document, twice the client's document limit (section 11). A
  document past a count or that room is refused as too large.
- **An answer that came and does not decode ends its read.** Whatever the
  cause, malformed, past the client's bounds or of another version, a jig
  read ends malformed, beside the link's `UndecodableDocument` fact: asking
  again would get the same answer. A family's read ends with no record, as any
  of its failures does. A lost exchange, or one not sent for want of room, ends
  unreachable, which an application may ask again by itself.

## 14. The world

`jig-client-world` (`tests/client`) runs the client through its surface
alone, as an application holds it, with the engine scripted (../testing.md,
section 4), and joins the engine's domain in the joint world
(../testing.md, section 5). Its stories: signing in by the printed link and
starting a chat by `shapes::chat`; a held chat amended and released in
one compound request; a proposal accepted while two handles watch it;
two clients deciding one thing; a restart with requests pending, the
kept work told and sent; the engine restarting under a pending request;
the stream dropped, every handle behind, then live; a handle given
another subject while a delivery for the old one is in flight; a store
write that fails; another party signing in on a client with kept work; a
family's request, read and watch as bytes; a version not served; a
sign-out lost, then sent again; a one-shot client that sends, waits for
the durable answer and stops.

Its referee is ../testing.md's, 6.1 and 6.2; the reference UI's stories,
with a person in front of a tree, are ../view-state.md's, 10.

## 15. Open questions

- **Browser approval between hosts and listing sign-ins** (12): the
  public-name flow and ending another sign-in from the browser remain
  `../../../protocol/client/README.md`, section 12's work. The client takes an
  approval on loopback and signs out its own sign-in.

- **A failed link beside a live transport sign-in** (12): whether to
  fall back to the transport after `link_used`, `link_expired` or
  `link_unknown`. Default: the link's reason remains visible, with no
  fallback; a browser reload signs in by its cookie.
- **A busy watch's cause** (`watches.md`, 2): `watches_full` records the refusal,
  which carries no count or separate capacity cause. Whether to distinguish
  watch room from another admission bound is a protocol decision. Default:
  the existing Busy refusal and a boolean, read on demand by the application.
- **Activity past a client's read bounds** (sections 11 and 13): a line past
  `long_text`, or a page past `document_bytes`, ends the read malformed without
  cutting its words. Whether an application grows its bounds for a larger
  deployment, or the engine pages to a client's negotiated bounds. Default: the
  read ends; no bound is negotiated.
- **A run's wait on a summary row** (`helpers.md`, 1): a row holds no run wait, since
  a task query carries no run's check per row, so a chat that can't start
  says starting in a list, and its document alone says why. Default: so.
- **Several clients of one party,** such as two tabs: whether pending
  requests should be shared between them, which they are not; and the
  party's room for watches on the engine (../contract/README.md, 5.2), which they
  share while each counts its own, so that one may find its watches
  busy, behind until the other's end.
- **A window's turn:** how a thread shows a turn that opens a window (a
  compaction's summary of what came before it), beside the earlier
  window's turns it summarises.
- **A thread's `lines`** (`conversations.md`): a second bound beside its `bytes`, for a
  presentation that prices its tree by rows, as the reference UI and an
  application's tools do. Whether they price theirs by bytes, so that it goes.
  Default: it stays.
- **Signing out with the engine out of reach:** whether a client should
  keep the sign-out and send it once the engine is reached, or a host
  forget its credential there, leaving the sign-in live on the engine
  until it expires. A browser's cookie only its server clears
  (section 9).
- **The generic half in skein:** keyed requests until durable,
  level-triggered watches and the link are jig's while they have one
  user; whether they move to skein, by a rename and a crate move, once
  a second user wants them.
- **A root's two modes:** ../root.md, section 14's question.
- **A class's members in the schema:** the derivation by class lists each
  class's members by hand, once per class (section 11). Bound classes in the
  schema would generate the lists. Default: by hand until the schema has them.
- **A stream record's bound:** its body's bound plus 512, written at each
  shell, in `jig-client-shell-native` and in an application's. Default: each
  keeps its expression; one helper once those shells hold still.
- **A turn being written past what the engine retains** (`watches.md`, 3): the engine's
  `views.text_bytes` and the wire's `run_text_text` stop at one bound, so a client
  following a turn from its start holds it whole, and one opened later holds the
  retained end until its committed turn comes. Holding the whole takes a delivery in
  pieces, a change of a protocol, the user's. A thread whose next suffix
  skips bytes likewise keeps its prefix marked cut until a full-origin
  snapshot or its committed turn replaces it (`conversations.md`); it asks for no
  missing provisional bytes. Default: as it stands.
- **A committed turn past a client's document bound** (`conversations.md` and `README.md`, section 13): a turn
  past `document_bytes` on the wire is refused as too large, with its size, so it
  shows no calls' names, whatever `turn_text` would keep of it. Reading it takes a
  page past that bound. Default: too large, and the tools read it by its parts.
- **A run that committed no turn** (`conversations.md`) names no model to its client,
  and a line of activity past a thread's first page can leave its last turn
  unread. An undecodable or nameless newest turn likewise supplies no model,
  so the thread can retain an earlier completion's model and effort. Keeping
  the latest run's model and effort independently of readable turns is
  deferred. A task's claim could keep them on the task for every page, at
  the price of a stored field and a field of the client's wire; it would
  record what the run was given, not prove what its completion used.
  Default: as it stands, from the turns, retaining the last named model.

## 16. Modes

- **Long-lived:** a browser or a terminal. The host drives its loop, and
  the application presents.
- **A service:** the same, with a service's credential (`SignIn { with:
  Service }`) and no presentation; its client root holds the service's
  own logic (../root.md, section 11). Its small store is a file, written
  and synced before any request it keeps is sent, so a restart of the
  service sends what is pending again, under its keys.
- **One-shot:** sign in, read or send, wait for the durable answer, and
  stop, with the small store in a file: a request sent by a run that
  stopped before its answer is sent again by the next run, under its
  key.
- **The client runs no loop of its own in any mode.** The host steps it,
  fires its deadlines and carries its requests; the modes differ in the
  host alone, and each follows from the client's shape.
