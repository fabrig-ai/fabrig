# The wire and the connection

## 1. The wire

- **`jig-client-protocol-wire`,** a codec crate generated from schemas
  (skein's `data/codec/README.md`), holds jig's records for both ends:
  - the envelope of every message: its owner and its version (`README.md`, section 6
    );
  - the operations of `../../client/domain/contract/requests.md`, 2, one record
    each, by their names: `Create`, `Delegate`, `Conclude`, `Propose`, `Withdraw`, `Message`,
    `Amend`, `Cancel`, `Release`, `Stop`, `Move`, `Decide`, `Answer`,
    `Take`, `HandBack`, `Seen`, `MakeProject`, `RemoveProject`,
    `ChangeSettings`, `MoveToNewest`, and the project's own; and
    `Compound { project, operations }` (3.4 of it). A change of settings
    carries each value as the configuration's member describes it, field
    for field, a charter's instructions as text and an application's
    kind's value as its bytes (`../../domain/engine/settings.md`, 1 and 5);
  - their answers, with refusals' facts (section 2 of it), *version not
    served* among them;
  - the reads of its section 6 and their pages, the task query among
    them (6.1); the subjects of 7.1 and their windows;
  - the documents of 5.1: `me`, the task line, and the rest;
  - a run's text (7.3); the stream's hello, heartbeat and endings;
  - signing in and out (`sign-in.md`), and the small store's records
    (`README.md`, section 7).
- **In general form.** No record names a shape of work: a chat is a
  `Create` filled from a template; closing it, a compound request; a
  list of conversations or of goals, a task query whose rows are task
  lines; the charters offered for chats, templates in `me`. A new shape
  is data in an application's configuration, never a change to the
  wire.
- **The records are the contract's,** field for field: the joint world's
  translation (`../../client/domain/testing.md`, section 5) is the map each
  end's translation follows, made bytes. Where the joint world finds the
  contract short, the contract changes first, and the wire follows.
- **Each application's wire crate** holds its own: its families' asks and
  their outcomes, its subjects, reads and documents. It is generated the
  same way, and depends on no crate of jig's but `skein-codec`.
- **Translation at each end:** `jig-protocol-engine-clients` between the
  wire and the core's vocabulary; `jig-client-protocol-link` between the
  wire and `jig-client`'s; the application's protocol layer between its
  wire and its connectors' vocabularies, and the application between
  its wire and its own. Each is a module of small total functions,
  exhaustive both ways, as the channel's are (`../channel/README.md`, section 3).
- **A project document** carries each adopted resource's connector part,
  role and independent `home` boolean. A fork retains its fork role.
  Ownership and live hold reports imply no home (`../../domain/model.md`,
  6.1; `../../domain/connectors.md`, 12 and 14).
- **Documents are decoded whole,** into the client's types, and checked
  against the client's limits before they reach `jig-client-domain`: a
  document past its bytes, rows, a path's segments, an authority's
  entries or texts is refused at decoding, as too large, never cut
  silently (`../../client/domain/client/README.md`, section 13), and a chip whose
  phase lacks what it names, an ended chip's end time or a backing-off
  chip's notice, is refused as malformed. An authority
  carries no cut mark (`../../client/domain/contract/documents.md`, 3). The wire's
  generated worst cases price each document at the client's limits.
  Connector hold accounts retain their independent 512-byte bound
  (`../../client/domain/contract/documents.md`, 3), including when the client's general
  text preview limit is smaller. The wire admits that room for document
  texts; conversion still checks every other text against the client's limit.
- **The schemas' ceilings bound the client's limits.** Each bounded
  field has a fixed ceiling in its schema, and the client's limits stay
  within them. A list the client bounds by its rows (a task's
  dependencies, a role's holdings, a pattern's segments) takes at most
  32 items, so 32 is the most the client's `rows` may be, and with it
  the rows of its pages and windows. More changes the wire's schema,
  and so moves its version (`README.md`, section 6). A client whose rows pass it is unusable: the link
  refuses its limits at construction, and nothing is cut to fit.
- **A document's nested collections cross as pieces.** The client bounds
  a document by its bytes in all, not each list by its count times its
  largest item. So a list whose items hold lists of their own (a page's
  rows; the rows of a table in which a contract type that nests itself
  is flattened, each row with its parent's index) crosses as one piece
  bounded by the client's document limit (skein's `data/codec/README.md`, section 3,
  "A sum crosses as one piece"). The client decodes each row as it
  splits the piece, counting it against that limit, so a page of many
  small rows and one of a few large ones cost what they hold. Lists of
  flat items stay lists.
- **Turns cross as their bytes,** smith's, inside the documents that
  carry them, and are decoded by `jig-protocol-transcript` in the
  client's link (`../../client/domain/client/conversations.md`); a turn that
  does not decode crosses up as such, never dropped. An activity line's
  turn crosses beside the line's references: its bytes; its size, where
  the turn alone passes the read's byte bound; or unread (`../store/loads.md`,
  3).
- **An invalid run end carries its cause.** `RunEndFailed.invalid_reason`
  is present only for the invalid failure class. It names the saved
  contract-result kind: form, verdict, words, change or followups
  (`../../domain/tasks/lifecycle.md`, 7). With no saved kind it says unreadable
  result, for an adapter's decode or translation failure. Every other
  class omits it. This is the ended attempt's immutable fact, not the
  task's current retry context. Wire 20 carries charter overlays under `README.md`, section 6.
- **Every result kind a task hears crosses.** A delegate's result reaches
  its requester as a word whose kind names the result
  (`../../domain/tasks/messages.md`, 1), and `WordResult` has a form for each
  kind the core records: a job delegate's (`../../domain/engine/executors.md`,
  5) is `job`, with no payload, after `cancelled` among its tags. The word,
  or the `Words` line, carrying a job's result crosses with empty words: a
  job's result is bytes its procedure reads, not words, and need not be
  text (`../../client/domain/contract/documents.md`, 7). So the engine's
  translation of an activity line, a word or a question's answer refuses no
  kind the core records, and no line a task heard keeps its activity's page,
  or a question's answer, from being served. Wire 20 takes `job` in place,
  before the first release (`README.md`, section 6).
- **No time crosses, only walls.** Documents carry wall times, which the
  store keeps (`../../client/domain/contract/documents.md`, 3); durations (a
  retention, a heartbeat, a grace, a deadline a request gives as a span,
  `../../client/domain/contract/requests.md`, 2) cross as durations. A deadline the
  engine keeps on its own clock (an account's next ask, a grace's end)
  crosses as the wall it falls at, which the engine's translation makes
  from its clock and its wall when it makes the document, so a client
  shows it as it shows every other moment.

Authority selectors preserve omission, ALL and named sets (including
empty), at most 64 names of 4,096 UTF-8 bytes each. Both public
authority forms carry them. Authority documents carry `cut`
independently from the whole authority payload used for edits
(`../../client/domain/contract/requests.md`, 2); decoding retains the flag, and
`whole()` refuses a cut projection.

`me` carries the public provider catalogue and the startup key bound,
and `accounts` the provider and payer readiness. `Deployment` includes
the same catalogue and viewer readiness for the reference fragment.
Each deployment host's `HostRule` is `open`, `person` with a
`HostPerson` party number, `several`, or `unknown`, recording its
current kept-login placement rule. The `position` hello refusal names a
worker position that is unconfigured or held by a live channel. These
are bounded secret-free configuration and derived facts
(`../../client/domain/contract/documents.md`, 1, and `../../client/domain/contract/README.md`, 5.1).

The wire carries added dependencies (`../../domain/tasks/lifecycle.md`, 8 and `../../domain/tasks/holds.md`, 7 and `../../domain/tasks/changes.md`
): an `Amend` names a task's dependencies as one whole list;
the refusals *circle*, naming its tasks in order, cut at its limit,
which a batch's cycle becomes, *dependency* and *kept*; a task
document's dependencies, each with its phase, its wall and maker when
it was added, and whether they keep the task; a kept task's pending
dependencies in what it waits on; and a hold for a circle of waits,
naming it (`../../client/domain/contract/requests.md` and `../../client/domain/contract/README.md`, section 3 and `../../client/domain/contract/documents.md`). The
owner's activity also carries the parked run's kept line, its attempt
and pending dependencies, and the owner's message kinds the met
notice, the dependency and whether it was removed
(`../../domain/tasks/lifecycle.md`, 8, and `../../domain/tasks/messages.md`, 1; `../../domain/engine/executors.md`).

A task's run wait carries `TaskRunWait.refused`: a hello refusal,
`TaskWaitHello`, the `HostKind`, the wall receipt time, the
`HelloRefusal` and `figures: option ShortfallFigures`; or a bounds
refusal, `TaskWaitBounds { kind: HostKind why: option AgentShortfall
figures: option ShortfallFigures }`, `ShortfallFigures { required: u64
offered: u64 }` (`../../client/domain/contract/documents.md`, 1). The server and
client share the deployment's hello-refusal translations. A checks
shortfall carries the failing charter number as `AgentShortfallChecks {
charter }`, identifying its `reserve_time` member, and turns, time and
waiting carry theirs as `AgentShortfallTurns`, `AgentShortfallTime` and
`AgentShortfallWaiting { charter }`. The v21 schema takes these in
place.
`TaskWaitSource { state }` names a source's state. An account wait's
why appends `untooled`, `TaskWaitUntooled { worker: u32 }`, the worker
setting's number whose login of that source started its program
without its tools (`../../domain/engine/fleet.md`); the client wire
moves its version for it, unbridged (temper's `draft/redeploys.md`,
section 8). `TaskRunWaitRefused { value }` wraps the enum payload
for the generator, an unversioned record that adds no tag or header
bytes. Shared admission types precede their task-wait consumers in the
schema, as the generator requires. Status and person-facing words are
`../../client/domain/contract/documents.md`, 1's.

A task's ending authority (`TaskFlags.ended_by`, *result* or
*requester*) and its refusal status travel in jig's wire and in the
tasks and people families; the worker's `GrantValue` keeps tags 0 to 2.

`me` carries `resolved_default: Option<LlmChoice>` for each effective
template with one, in its project's `defaults` by the template's number, a
template's own record being a setting's too; and in each project's `models`
the offer alone, each `MeModel` with its `effort`, `shown`, `accounts` and
`picker` (`../../client/domain/contract/documents.md`, 1). The schema, generated
codecs, translations and served-version declarations carry those fields
(`README.md`, sections 6 and 11). Tests build their values from schema text
or with public fields, without golden files (skein's `data/codec/README.md`, 6). The
preview says what a run's check would take now for a task keeping no
account (`../../domain/accounts.md`, 4.3), never task intent or a reservation;
no store family carries it or the offer, and application families' versions
are their own.

The account's move (`../../domain/accounts.md`, section 6) is in jig's
wire: the account changes `move_out { account }`, `written { number }`
and `move_in { digest }`, the last carrying the moved file as the
keyed request's value (`README.md`, section 3), whose bound is 128 KiB; the answer
`Made::moving_out { account, number, record }`, the record a secret of
at most 128 KiB filled for the asking exchange alone (`README.md`, section 3), and
the outcome `account_moved_in { account, renewed }` (*written* is
answered `account_changed`); the refusal `account_move { why }`, `why`
one of `not_signed_in`, `not_ours`, `held`, `cancelled` and `moving`;
and `moving` in each source state an account document or a task's wait
shows.

## 2. The connection

A client keeps one connection to the engine, whatever carries it, of
two parts: exchanges, each one message and one answer; and one stream,
the engine's messages to the client as they come.

- **Exchanges.** Every one of jig's messages but the stream's crosses as
  an exchange: one record each way, answered once. Each carries its
  credential (`sign-in.md`) and names its owner and its version (`README.md`, section 6
  ).
- **A batch** is one exchange carrying many messages: reads, watches'
  openings and watches' closings, each the record its own exchange
  carries, at most 64, under the batch's one credential. Its answer is
  one reply per member, each naming its member, handed out in the
  order they are released: a read's once every commit it reflects is
  durable, an opening's once admitted or refused, a closing's once the
  engine has ended the watch; so one slow read holds back no other
  member's reply. Each member is the message it would be alone: checked,
  admitted, reserved and answered as on its own exchange, its reply the
  record its own exchange's would be, so a member answered busy,
  refused, signed out or not served is that member's alone. A batch
  cut short leaves the members answered as they were; each unanswered
  one is lost as its own exchange would be (2.3). Keyed requests,
  signing in and out, and approvals keep their own exchanges.
- **The stream** is one per client, for as long as the client keeps it.
  Its opening names the version of jig's wire the client speaks. Its
  first message is the hello: the stream's name, the versions the
  engine serves of jig's wire and of each application's, and the
  heartbeat; then, where the opening named a version the engine serves,
  `me`'s value (below). Every watch the client opens names its stream.
- **`me` comes with the stream.** The engine opens a watch of `me` for
  every stream whose opening names a version it serves, as it would open
  a client's watch of `me`, the core naming the party from the stream's
  sign-in (`README.md`, section 5), and names it `0` on that stream. It sends nothing on the stream before
  the hello, and the hello waits for that watch's first delivery, held
  until durable as every delivery is: it is the stream's
  `StreamEvent::Opened`, `Opened { hello, me }`, its `hello` the hello
  alone's record, so the ranges come first, and its `me` the document
  in jig's versioned envelope, `Owned`, at the opening's version, whose
  record is what a delivery of `me` carries. `me`'s later values are
  deliveries of watch `0`, paced and coalesced as any watch's. The
  client opens no watch of `me` and never names a watch `0`; an
  opening naming `0` is refused as a name the stream already holds, and
  a closing of it is answered as one the client never opened. The watch
  holds its place in the party's room and the stream's as any watch
  does (`../../client/domain/contract/README.md`, 5.2), and ends with
  its stream. Refused or ended, it ends its stream: signed out where
  the sign-in ended, and otherwise busy (2.2), which the client takes as
  the stream dropped and opens again after its backoff
  (`../../client/domain/client/watches.md`, 2).
- **The hello alone,** `StreamEvent::Hello`, is what the engine sends a
  stream whose opening names no version, or one it doesn't serve: the
  stream's name, the ranges and the heartbeat, with no `me` and no watch
  of it, encoded the same in every version (`README.md`, section 6). So
  an earlier client, whose opening names none, reads the ranges and is
  refused by name. A client whose version the hello alone's ranges
  serve takes it as a record out of its order on the stream.
- **A client is one run of `jig-client`,** from its start to its end: a
  native process; a browser page from its load until it is left. A link
  followed, or back and forward, within the page's history keeps it
  (`../browser/README.md`, section 9); a reload, an address typed or opened, or
  back and forward to an entry another load made, starts a new client.
  Each client holds a stream of its own, so two clients of one sign-in,
  two tabs or a page and the page that follows it, hold two. The engine
  never refuses or ends a stream for another stream of its sign-in:
  streams are bounded in all, by the component and by the transport
  (`README.md`, section 10), and an opening past either bound is refused as full,
  naming the bound reached and its size: the client keeps trying by its
  backoffs, as for a dropped stream, since a stream may free, and tells
  its caller the streams are full as it does, so a caller whose own
  deadline passes meanwhile is answered full, naming the bound, never
  a silent timeout.
- **The stream names the one it replaces.** A page that was left, or a
  process that died, may hold its stream, and its watches their places
  in the party's room, until the engine sees the stream's connection
  end: seconds later for a page whose browser drains it, and later
  still for a connection cut half-open; so may this client's own
  stream that dropped. So the client keeps its stream's name in the
  small store, saved at each hello (`README.md`, section 7), and each opening of
  its stream names the last stream it knew: at its first opening, the
  one its kept records name, if any; after, its own last hello's.
  The engine checks the opening's credential, then ends the named
  stream, when it is the sign-in's and still open, as a stream gone
  (2.1) ends it, before it admits the new one against the streams'
  bounds; so a name kept by another sign-in, or of a stream already
  ended, does nothing, and the opening waits on no other exchange. A
  store copied (a browser tab duplicated) ends the original's stream
  once, which the original takes as any drop. A stream gone stays a
  message any client may send for a stream of its sign-in's.
- **Heartbeats.** The engine sends a heartbeat on the stream whenever it
  has sent nothing for the heartbeat's span. The link tells each of the
  stream's watches it is alive at each heartbeat and each message; past
  twice the span with nothing read, it takes the stream as dropped.
- **A dropped stream** drops every watch on it, each told so, and the
  engine ends them as the client left; until then, each holds its place
  in the party's room (`../../client/domain/contract/README.md`, 5.2). The client
  reopens them, and the stream with them, after its backoffs
  (`../../client/domain/client/watches.md`, 2). A watch's opening or closing
  that gets no answer drops the stream too (`README.md`, section 5).

### 2.1 jig's messages

| Message | Carries | Answered with |
|---|---|---|
| a keyed request | its key; the request's owner, version and record: one of jig's operations, a compound request, or an application's request naming its scope | an answer (`README.md`, section 3) |
| a read | its owner, version and record: one of jig's reads, or an application's naming its scope and window | the document, a page, or why there is none (`README.md`, section 4) |
| opening a watch | the stream's name; the client's name for the watch; the subject, with its owner, version and window | admitted, or refused (`README.md`, section 5) |
| closing a watch | the stream's name and the watch's | closed, once the engine has ended it |
| a batch | `Batch { members }`: at most 64, each `read: Owned`, `open: Open` or `close: WatchClose`, the record of that message alone | one `BatchReply { member, reply }` per member, in the order released: `member` its place, from 0; `reply` the record its own exchange would answer, or none where that exchange would be answered no record (section 2; `default-http.md`, section 6) |
| a stream gone | the stream's name | nothing more: the engine ends the stream and its watches, when it is the sign-in's and still open (section 2) |
| signing in | a link's secret, the printed link's, the local link's or an asked one's; or, as a service, nothing more than the service's credential, as the message's credential; or an approval's secret, with the grant its loopback return brought or none | a sign-in's credential, refused, not ready, or why a printed link's secret or an approval's can't sign in, a grant that doesn't match among them (`sign-in.md`) |
| asking an approval | its loopback return's port, or none | the approval's secret, its code and its expiry; refused, not ready or busy (`sign-in.md`) |
| asking a link | nothing more than the asking sign-in's credential | the link's secret and its expiry; refused, signed out, not ready or busy (`sign-in.md`, the asked link) |
| reading an approval, deciding one | its code; and approve or refuse, to decide | what the approval is, waiting with its kind, its source and the reader's; or the decision's answer, an approval with a loopback return approved with its port and grant; not a browser, signed out, not ready or busy (`sign-in.md`) |
| signing out | nothing more than the credential of the sign-in it ends | signed out, or not ready |

On the stream, in its owner's version: the hello, with `me` or alone
(section 2); a delivery (a watch's document, whole, in its owner's
wire), `me`'s among them as watch `0`'s; a run's text; a heartbeat; a
watch ended, with why.

A batch's records, in `jig-client-protocol-wire`'s schema, from jig's
client wire 28:

```
enum Member { read: Owned, open: Open, close: WatchClose }
record Batch { members: list 64 Member }
record BatchReply { member: u16 reply: option bytes R }
```

`R` is at least the largest encoded bound of `ReadAnswer`, `OpenAnswer`
and `CloseAnswer`, the records a member's own exchange answers. A
batch names no owner or version of its own: its members name theirs,
as alone, and a closing none, as alone.

### 2.2 The clients' component's messages

```
in, from the transport:
  Exchange { exchange, credential, browser, via, source, message }   one of jig's messages (4.1), an application's record inside it where its owner is the application; the credential as opaque bytes, or none; browser, whether the transport carried that credential itself, as a browser's cookie (17.3); via, whether the exchange came by the public name or by the listener's own address, and source, its address or none for unknown (`default-http.md`, 3)
  StreamOpen { stream, credential, version, replaces }  a client opened its stream, naming jig's wire version or none, and the 16-byte name it replaces or none (section 2)
  StreamTaken { stream }                       the transport took the last message streamed
  StreamGone { stream }                        the stream's connection ended
  ExchangeGone { exchange }                    an exchange's connection ended before its reply: a keyed request's, a read's, a public read's
in, from the application's server code (never from a client):
  Mint { exchange, identity }                  an identity the application verified: a provider's number, a subject, a login, a name
out, to the transport:
  Reply { exchange, message }                  one record: an answer, a page, a watch admitted or refused, a sign-in's credential; for a batch, only its refusal whole, not a record
  Part { exchange, message }                   one BatchReply of a batch's, as its member's reply is released
  Done { exchange }                            a batch's last part handed: its exchange is answered
  Streamed { stream, message }                 the hello, a delivery, a run's text, a heartbeat, a watch ended
  StreamEnded { stream, why }                  signed out, the engine stopping, or busy: the stream's `me` refused or ended for want of room (section 2)
  Minted { exchange, credential, expires } | MintRefused { exchange, why }
out, to the application's startup:
  Link { secret }                              the printed link's one-time secret, each time one is drawn (section 8)
  LocalLink { secret }                         the local link's one-time secret, each time one is drawn, kept and never printed (section 8)
  LinkSignedIn                                 the printed link signed a person in, each time (section 8)
```

- **An exchange's way and source** are `via: Via`, `Public` or
  `Listener`, the component's type, which the engine end decides and
  names, and `source: Option<IpAddr>`, core's address, canonical
  (`default-http.md`, 3). The component reads them for an approval's
  kind and source alone (`sign-in.md`, *A sign-in approved by a
  browser*); a public read and a stream carry neither, and a batch's
  members are the batch's exchange's.
- **`Minted` carries the sign-in's expiry,** its wall time, beside the
  credential, so the server gives the credential the sign-in's life, as
  the default mapping's cookie's `Max-Age` (`default-http.md`, 3).
- **Names.** `exchange` and `stream` are the transport's, which the
  component gives back and never reads. The stream's name that watches
  carry is the component's, drawn when the stream opens and sent in its
  hello.
- **Its side toward the engine's root** is the protocol layer's routing,
  as for every component: the core's events and outputs, an
  application's records decoded and encoded through its wire, and the
  secrets' store's records (`../service.md`, section 2).
- **One reply per exchange,** made once the core has answered and, for a
  keyed request or a read, once that answer is durable. A transport that
  has lost its client drops the reply.
- **A batch's replies are parts.** The component decodes a batch whole,
  within the message's bytes (below) and its 64 members, or answers it
  `Reply` not a record, with nothing done. It checks the batch's
  credential once, then takes each member, in the batch's order, as the
  message it would be alone, and hands its reply out as a `Part`, a
  `BatchReply` naming it, when that reply is released; a member
  answered at once (busy, not served, signed out, unknown) is released
  at once. A member its own exchange would answer with no record (a
  malformed member, a closing that conflicts) is a part whose reply is
  none. Once every member's part is handed, `Done`. `ExchangeGone`
  lets the batch's caller go, as any exchange's: its members' work
  settles as it would have, and their parts go nowhere.
- **A batch is admitted as one exchange, its members as many.** It
  takes one exchange of the sign-in's and of all (below) from its
  admission until `Done`. Each member reserves what it would alone: a
  read, a place among the sign-in's reads in flight; an opening, a
  place among its openings in flight, a watch's place on its stream,
  and the core's and the views' rooms as alone; a closing, nothing, as
  alone. A member that finds its room full is answered busy, as alone;
  the others go on. A batch that finds no exchange's room, the
  sign-in's or all's, is answered as its messages sent apart would be
  when each found none: each read and opening busy at once, and each
  closing taken as alone. A batch admits nothing the same messages sent
  apart could not: every check of a member's own exchange stands.
- **A caller gone lets its exchange go.** The transport says
  `ExchangeGone` when an exchange it handed the component ends before
  the component's reply reaches it, its connection ended or failed
  (`default-http.md`, 4). The component lets go of that caller at once and keeps what
  the exchange started: the core's or the application's answer, when it
  comes, settles as it would have (a link spent, a carried value used or
  erased, a committed effect kept), and its reply goes nowhere. A close
  waits for the replies owed to callers still there, never for a caller
  gone, so a request admitted before its caller left never holds the
  engine's drain (`../service.md`, section 5).
- **One streamed message in flight per stream.** The component hands the
  transport the next only after `StreamTaken`, which the transport says
  once it has written the message to its connection, or into a bounded
  buffer of its own (`README.md`, section 5).
- **The credential is checked** on every exchange and every stream
  opened (`sign-in.md`). A stream opened with none, or with one the engine
  does not know, ends at once, `StreamEnded` signed out.
- **Bounded:** exchanges in flight per sign-in and in all, a batch one
  whatever its members; reads in flight and watches' openings in flight
  per sign-in, a batch's members each one as alone; streams in
  all, never per sign-in (section 2); a message's bytes, a batch's
  among them, and a batch's 64 members; what a stream
  may have queued, mints in flight (`README.md`, section 10).
- **A stream's opening that names one it replaces** (section 2), once its
  credential is checked, ends the named stream first when it is the
  sign-in's and still open, as a stream gone does, then is admitted or
  refused full as any opening; a name of another sign-in's stream, or of
  none open, changes nothing.

### 2.3 The link

`jig-client-protocol-link` is `jig-client`'s half toward its transport.
`jig-client`'s surface, as the application holds it, is
`../../client/domain/client/README.md`'s; toward the transport it is these
messages:

```
emitted:  Exchange { exchange, credential, message } | OpenStream { credential, version, replaces } | CloseStream | Save { records } | Erase { records }
heard:    Reply { exchange, message } | Part { exchange, message } | Done { exchange } | Lost { exchange } | Streamed { message } | StreamGone | Stored | NotStored
```

A batch is one more of `Message`'s kinds, `Batch`, its record a
`Batch`'s bytes, beside `Read`, `Watch`, `Unwatch` and the rest;
`replaces` is the 16-byte name of the stream the opening replaces, or
none.

- **Messages as bytes.** The link encodes each of jig's messages, and
  decodes each reply and each streamed message, with
  `jig-client-protocol-wire`; an application's records travel inside
  them as bytes under their owner's number, which the application's wire
  encodes and decodes. There is no method, path or header here: those
  are the transport's.
- **One attempt per exchange,** whose terminal is a `Reply`, or `Lost`:
  the exchange failed, or ended without a whole reply within the
  exchange's deadline, which the transport keeps
  (`../../client/domain/client/README.md`, section 13). A batch's terminal is `Done`
  or `Lost`, after a `Part` for each reply that came whole. Retrying is
  the client's.
- **Sent at the settling.** The link hands out no `Exchange`,
  `OpenStream` or `CloseStream` while the client steps: it gathers them,
  in the order they were asked for, and hands them out when the client
  settles (`../../client/domain/client/README.md`, 4.1). Then each read, watch's opening and
  watch's closing that may leave (an opening once the stream has its
  name, a closing once its opening is admitted, `README.md`, section 5) goes in a
  batch while more than one does: its members in the order asked, at
  most 64 and at most `BATCH_BYTES`, 131,072, of the batch's record, the
  rest in the next batch; a message whose record alone passes
  `BATCH_BYTES`, and one with no other to go beside it, goes alone on its
  own exchange. Every other message goes alone, a keyed request with
  its key, as before. Each exchange, a batch one, takes one of the
  link's exchanges in flight (`README.md`, section 10); what finds none
  waits, in order, for a later settling.
- **Each member is its own.** The link keeps each member's attempt as
  it would alone, and hands each `Part`'s reply to its member, by the
  part's `member`, as the reply of that member's own exchange: a
  record decoded and judged as on its own route; none, as `Lost` on its
  own route. So a read answered busy ends busy, an opening answered busy
  is opened again after its backoff, and an opening refused or not
  served is that watch's alone. At `Lost`, or at `Done` with members
  unanswered, each unanswered member is lost as alone: a read ends
  unreachable, and a watch's opening or closing with no answer drops
  the stream, whose watches the client opens again (`README.md`, section 5). A part
  naming no member of its batch still unanswered, or whose record does
  not decode, is the engine's fault: the link takes the batch as lost
  there, each member then unanswered as above, and reads nothing more of
  it until its terminal.
- **The credential** on `Exchange` and `OpenStream` is the one the
  client holds, or none when the transport carries one the client never
  sees (`sign-in.md`).
- **The stream:** opened naming jig's version, the link's own; its
  hello judged against the client's versions
  (`README.md`, section 6), and its `me` handed on as the first value of
  the client's `me`, watch `0`'s deliveries its later ones (section 2);
  each delivery and run's text routed to its watch by name;
  heartbeats keeping the watches alive; `StreamGone`, or silence past
  twice the heartbeat, dropping it; the stream refused signed out, a
  `Streamed` message the transport makes, ending the sign-in. A record
  of it that the link refuses as malformed or of another version, or a
  watch's answer whose bytes don't decode, drops it and leaves the
  client out of date, `unreadable`
  (`../../client/domain/client/watches.md`, 2); one refused for a bound
  or for its order drops it, to open again after its backoff.
- **The stream it replaces:** the stream's name saved at each hello;
  each `OpenStream` naming the last stream the link knew, the name
  kept at start before the first hello, its own last hello's after
  (section 2). No stream gone is sent, and no opening waits for one.
- **The small store's records** encoded and decoded (`README.md`, section 7).
