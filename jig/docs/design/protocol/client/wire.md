# The wire and the connection

## 1. The wire

- **`jig-client-protocol-wire`,** a codec crate generated from schemas
  (skein's `data/codec/README.md`), holds jig's records for both ends:
  - the envelope of every message: its owner and its version (`README.md`, section 6
    );
  - the operations of `../../client/domain/contract/requests.md`, 2, one record
    each, by their names: `Create`, `Propose`, `Withdraw`, `Message`,
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
`AgentShortfallWaiting { charter }`. The v20 schema takes these in
place.
`TaskWaitSource { state }` names a source's state. `TaskRunWaitRefused { value }` wraps the enum payload
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
codecs,
translations, golden records and served-version declarations carry those
fields (`README.md`, sections 6 and 11). The
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
- **The stream** is one per client, for as long as the client keeps it.
  Its first message is the hello: the stream's name, the versions the
  engine serves of jig's wire and of each application's, and the
  heartbeat. Every watch the client opens names its stream.
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
- **The predecessor's stream is sent gone.** A page that was left, or a
  process that died, may hold its stream, and its watches their places
  in the party's room, until the engine sees the stream's connection
  end: seconds later for a page whose browser drains it, and later
  still for a connection cut half-open. So the client keeps its
  stream's name in the small store, saved at each hello (`README.md`, section 7).
  The first time it would open a stream, a client whose kept records
  name one sends that stream gone (2.1), and opens its own once that
  exchange has its terminal, a reply or `Lost`. The engine ends a stream
  named gone only when it is the sign-in's and still open, so a name
  kept by another sign-in, or of a stream already ended, does nothing.
  A store copied (a browser tab duplicated) ends the original's stream
  once, which the original takes as any drop.
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
| a stream gone | the stream's name | nothing more: the engine ends the stream and its watches, when it is the sign-in's and still open (section 2) |
| signing in | the printed link's secret; or, as a service, nothing more than the service's credential, as the message's credential; or an approval's secret | a sign-in's credential, refused, not ready, or why a printed link's secret or an approval's can't sign in (`sign-in.md`) |
| asking an approval | nothing | the approval's secret, its code and its expiry; refused, not ready or busy (`sign-in.md`) |
| reading an approval, deciding one | its code; and approve or refuse, to decide | what the approval is, or the decision's answer; not a browser, signed out, not ready or busy (`sign-in.md`) |
| signing out | nothing more than the credential of the sign-in it ends | signed out, or not ready |

On the stream, in its owner's version: the hello; a delivery (a watch's
document, whole, in its owner's wire); a run's text; a heartbeat; a
watch ended, with why.

### 2.2 The clients' component's messages

```
in, from the transport:
  Exchange { exchange, credential, browser, message }   one of jig's messages (4.1), an application's record inside it where its owner is the application; the credential as opaque bytes, or none; browser, whether the transport carried that credential itself, as a browser's cookie (17.3)
  StreamOpen { stream, credential }            a client opened its stream
  StreamTaken { stream }                       the transport took the last message streamed
  StreamGone { stream }                        the stream's connection ended
  ExchangeGone { exchange }                    an exchange's connection ended before its reply: a keyed request's, a read's, a public read's
in, from the application's server code (never from a client):
  Mint { exchange, identity }                  an identity the application verified: a provider's number, a subject, a login, a name
out, to the transport:
  Reply { exchange, message }                  one record: an answer, a page, a watch admitted or refused, a sign-in's credential
  Streamed { stream, message }                 the hello, a delivery, a run's text, a heartbeat, a watch ended
  StreamEnded { stream, why }                  signed out, or the engine stopping
  Minted { exchange, credential, expires } | MintRefused { exchange, why }
out, to the application's startup:
  Link { secret }                              the printed link's one-time secret, each time one is drawn (section 8)
  LocalLink { secret }                         the local link's one-time secret, each time one is drawn, kept and never printed (section 8)
  LinkSignedIn                                 the printed link signed a person in, each time (section 8)
```

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
- **Bounded:** exchanges in flight per sign-in and in all; streams in
  all, never per sign-in (section 2); a message's bytes, what a stream
  may have queued, mints in flight (`README.md`, section 10).

### 2.3 The link

`jig-client-protocol-link` is `jig-client`'s half toward its transport.
`jig-client`'s surface, as the application holds it, is
`../../client/domain/client/README.md`'s; toward the transport it is these
messages:

```
emitted:  Exchange { exchange, credential, message } | OpenStream { credential } | CloseStream | Save { records } | Erase { records }
heard:    Reply { exchange, message } | Lost { exchange } | Streamed { message } | StreamGone | Stored | NotStored
```

- **Messages as bytes.** The link encodes each of jig's messages, and
  decodes each reply and each streamed message, with
  `jig-client-protocol-wire`; an application's records travel inside
  them as bytes under their owner's number, which the application's wire
  encodes and decodes. There is no method, path or header here: those
  are the transport's.
- **One attempt per exchange,** whose terminal is a `Reply`, or `Lost`:
  the exchange failed, or ended without a whole reply within the
  exchange's deadline, which the transport keeps
  (`../../client/domain/client/README.md`, section 13). Retrying is the client's.
- **The credential** on `Exchange` and `OpenStream` is the one the
  client holds, or none when the transport carries one the client never
  sees (`sign-in.md`).
- **The stream:** its hello judged against the client's versions
  (`README.md`, section 6); each delivery and run's text routed to its watch by name;
  heartbeats keeping the watches alive; `StreamGone`, or silence past
  twice the heartbeat, dropping it; the stream refused signed out, a
  `Streamed` message the transport makes, ending the sign-in. A record
  of it that the link refuses as malformed or of another version, or a
  watch's answer whose bytes don't decode, drops it and leaves the
  client out of date, `unreadable`
  (`../../client/domain/client/watches.md`, 2); one refused for a bound
  or for its order drops it, to open again after its backoff.
- **The predecessor's stream:** the stream's name saved at each hello;
  the name kept at start sent gone before the first `OpenStream`, which
  waits for that exchange's terminal (section 2).
- **The small store's records** encoded and decoded (`README.md`, section 7).
