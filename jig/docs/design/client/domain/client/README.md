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
  own, and it never calls the application (`vocabulary.md`, section 1).
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
  watermark again (`small-store.md`, section 1).
- **The link is the client's to judge,** from the stream, whose hello
  brings `me`: signed out, starting, live, offline, or out of date
  (`watches.md`, 2).
- **Signing in by `SignIn { with }`:** a printed link's secret, a
  service's credential, a credential the application obtained by its own
  sign-in, or one its transport carries unseen. jig owns credentials;
  every sign-in flow beyond its two is the application's (`sign-in.md`, section 1).
- **The application's families travel as bytes,** under their owner's
  number and version, on the same connection and through the same
  table. The application's wire crate encodes and decodes them; the
  client never reads them (`vocabulary.md`, section 1 and `requests.md`; ../root.md, section 7).
- **jig's messages, not HTTP.** The client gives jig's messages to a
  transport the application chooses: jig's default HTTP mapping, a local
  socket, or an in-process channel (section 13).
- **Built for the host and for wasm32,** as step code under the strict
  subset (skein/docs/foundation/programming-model.md, section 10).
- **Helpers beside it, optional:** shapes of work from templates, facts
  over documents, a conversation's thread, and jig's words, each built
  only from the public client (`conversations.md` and `helpers.md`).

The client's documents include:

- [`vocabulary.md`](vocabulary.md): the client's vocabulary toward the application.
- [`small-store.md`](small-store.md): kept work and the small store.
- [`sign-in.md`](sign-in.md): credentials and signing in.

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
  store of what it must not lose across its own restart (`small-store.md`, section 1).
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
  - *the link:* its state, the sign-in it is on, the stream (watches.md, 2, and sign-in.md, 1
    ). The credential's value is never domain state
    (`../../../protocol/README.md`, section 8): `jig-client` holds it
    beside the domain, as an engine's protocol layer holds its secrets,
    and puts it on each `Exchange` and `OpenStream`;
  - *handles:* each watch's and each read's, with its asker (`vocabulary.md`, section 1
    ), each watch mapped onto a watch of the engine's with its latest
    document (`README.md`, section 6 and `watches.md`);
  - *the request table,* skein-lib's `RequestTable` (skein's `runtime/lib.md`,
    section 12), with each request's key and asker (`requests.md`);
  - *kept work:* the requests restored at start (`small-store.md`, section 1);
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
  only use of smith (`../../../client-stack.md`, section 1).
- **Its handles inside are skein-lib's:** its watches, reads and
  exchanges are `Id`s in slabs sized at startup, and what crosses to the
  transport is their tokens (skein/docs/foundation/programming-model.md, 4.2). A terminal is
  never stale: a slot is retired only once its terminal has come.
- **Beside it, the application's to use or not:**
  `jig-client-protocol-http`, the default HTTP mapping's client end;
  `jig-client-shell-native`, a native shell over skein's HTTP client;
  and `jig-client-view`, the reference UI (../README.md, section 2).

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
  turn read alone, ../contract/reads.md, section 1; malformed for a turn that does not decode, and
  for any read whose answer does not, `README.md`, section 13). A read is one attempt
  (`../../../protocol/client/README.md`, section 4): the application reads again
  if it wants.
- **What is live is watched; what is past is read.** The engine holds
  live work only (`../../../domain/engine/decisions.md`, 1.3), so ended tasks, history
  and transcripts are pages read from its store, a page at a time, as
  the application asks.
- **A window** is what a watch's document holds of a list, or a read's
  page: its size, its order and where it starts, by cursor. The
  application gives it; the engine bounds it in bytes and rows, never in
  rows sized for one client's screen (../contract/documents.md, 3, and ../contract/reads.md, 1.1, and ../contract/watches.md, 1.5).
  A window past the engine's rows is answered within them, never
  refused: the document or the page holds the rows the bound admits and
  says there are more, by its cursor for the next and, for a tree, by
  its folded counts (../contract/watches.md, 1.2). So an application asks for the
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
  deployment, or the party's own) and its record's bytes (../contract/watches.md,
  1.5); its page is its owner's, as bytes.
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

- handles; the engine's watches open, the stream's `me` always one of
  them, each until its terminal (`watches.md`, 1); reads in flight; exchanges in
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
  start, no more than the table holds and the stream's name (`small-store.md`, section 1
  );
- backoffs' first and most spans, the heartbeat's tolerance, the
  retention's margin;
- the family owners the application declares, each once, with its
  version (`watches.md`, 2): no more than a hello's ranges hold beside jig's
  (`../../../protocol/client/versions.md`, section 1).

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
  `Propose` (../contract/requests.md, 6; `../../../protocol/configuration-templates.md`, 1).
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
  within the exchange's deadline, the transport's. A batch's is
  answered by a `Part` for each of its replies as it comes whole, in
  the order they came, then one `Done` at its answer's end, or one
  `Lost` when the answer was refused or cut short, after the parts that
  came whole (`../../../protocol/client/default-http.md`, section 6). The transport makes
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
- **A run's wait on a summary row** (`helpers.md`, 6): a row holds no run wait, since
  a task query carries no run's check per row, so a chat that can't start
  says starting in a list, and its document alone says why. Default: so.
- **Several clients of one party,** such as two tabs: whether pending
  requests should be shared between them, which they are not; and the
  party's room for watches on the engine (../contract/watches.md, 1.2), which they
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
  (`sign-in.md`, section 1).
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
