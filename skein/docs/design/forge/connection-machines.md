# The forge component: entities, limits and state machines

## 1. Entities and limits

### 1.1 Entities

Each is in a slab of the component: a **call**, a **forge's start**, a
**forge connection** (bound to io's socket and one forge, or one of its
log origins, with its stack: TLS, HTTP client, JSON tokenizer), the
**tables** (repositories, users, page caps,
credentials, secrets, and a GitHub forge's installations and log
origins), each forge's **own connection**, which its start, its mints,
its installations' reads and its puller share, one request at a time, a
GitHub forge's **puller** (the guid and validator kept, where its poll
is, and the hint it holds), the **listener**, and a **webhook
connection** (its stack: HTTP server,
tokenizer, the delivery's decoder; the HMAC state, the fields decoded
and the hint it holds until the owner has room).

Each configured forge has one **entry** in the component: what its owner
configured (`connection.md`, 3.1), its start (2.3), its own connection, and its dialect's
state, whose variant holds the forge's own plan and exchange (2.1) and,
for a GitHub forge, its installations and its puller. The own exchange
lasts as long as its connection is the forge's own: it goes where that
connection stops being so, and when the component sheds what it holds
(2.5). An entry is made when its forge is
added and goes whole when it is removed. What the forges share (the
connections, the calls, the tables, the owner's word) sits beside the
entries, and a step on one forge is handed that forge's entry. A forge's
dialect is known from its entry and is not asked again: a call's plan
sits in a variant of its own dialect, chosen once where the call meets
its forge, and a call whose variant is not its forge's is answered as a
call to a forge that has gone (`connection.md`, 3.1).

### 1.2 Limits

The component's own; their values are the owner's.

- **Forges and tables:** `forges`; `token_bytes`, the largest token, an
  installation's too; `key_bytes`, the largest App key; `repositories`
  and `users`, the two tables' rows; `installations`, a GitHub forge's
  installations' rows (`connection-calls.md`, 1.4); `log_origins`, at least 1, a GitHub forge's
  job log origins, and the most origins not configured it remembers
  having said (`connection-calls.md`, 1.5).
- **Connections:** `connections`, equal to the owner's `calls`, so no
  call waits for one and the component never says `Busy`; an idle
  connection of one forge closes for a call to another when none is
  free. Each forge has one more, its own (1.1). `idle` is how long one
  is kept: below the keep-alive time of
  whatever serves the forge's API by more than a request's round trip.
- **What is read:** `head_bytes`, the largest response head, and
  `headers`, the most fields of a head, a delivery's too; `body_bytes_in`,
  the body read, streamed, so it caps time and bytes, not memory;
  `depth`, the JSON nesting followed (Forgejo's documents need about 8);
  `chunk`, the most text of a string demanded at once, and the most one
  read of a delivery takes. A JSON response intake holds `max(chunk, 5)`
  bytes, enough for a tokenizer chunk or its largest literal; each refill
  is bounded by its remaining room.
- **What is kept,** the dialect's (dialects.md, section 1.1):
  `string_bytes`, `body_bytes`, `inner` and `target_bytes`; and
  `kept_bytes`, the conditional reads' entries in all (`connection-calls.md`, 1.6), which the
  owner sets to its `calls` times its `answer_bytes`.
- **Time:** `deadline`, a call's and each request's of a forge's own,
  from when it is asked; `start_backoff` and `start_backoff_max`;
  `renew_before`, how long before an installation token's expiry it is
  minted anew (`connection.md`, 5.1).
- **The delivery log** (`connection-webhooks.md`, 1.5): `log_poll`, the time between a puller's
  polls; `log_catch_up`, how far back a poll reads; `log_pages`, the
  most pages it reads; `delivery_bytes`, the largest delivery read
  whole.
- **Webhooks:** `hooks`; `hook_head_bytes` and `hook_body_bytes`, the
  largest delivery's head and body; `hook_deadline`, a request's read
  and answer, from its connection's accept or its last answer;
  `hook_secret_bytes`, the largest secret, at least 32.
- `tls`, and what HTTP and JSON take as their own (../net/tls.md, http.md,
  ../data/json.md).

### 1.3 Shared values

The owner hands the component, as plain numbers, what it shares with its
domain: `calls`, `rows` (the most rows of a listing it keeps),
`answer_bytes`, `op_bytes` (an operation's text and key together),
`lifetime` (the longest a sent write may still arrive), `name_bytes`
(the longest name its domain keeps), `repositories` (the most it
watches), and io's caps. A plan is handed only what it reads: a
listing's page size, `body_bytes`, `rows` and `answer_bytes`. A page
size is the least of `rows`, the forge's page cap, and `answer_bytes`
over the listing's largest row, measured from `string_bytes`,
`body_bytes` and `inner`. Compare reads one page and keeps its paths while they fit `rows` and `answer_bytes`, counting the rest (github-operations.md, 1.1).

### 1.4 Checked at start

Each refusal names both limits:

- every required limit is above 0, and derived capacities are representable;
- `connections` equals `calls`, and `deadline` is at most `lifetime`;
- `body_bytes` is at least 2 * `op_bytes` + the word's length + 15: a
  marker writes the key twice over, in hexadecimal (README.md, 5);
- `string_bytes` is at least `name_bytes`, and `repositories` at most
  the owner's;
- each listing holds at least one of its largest rows in `answer_bytes`,
  by the measure of vocabulary.md, section 1.7, so no page size is 0;
- the dialect's own `check` of its limits (dialects.md, section 1.1);
- for each forge, the request head fits skein-http's `request`, the send
  cap of the stream below: `target_bytes` of target, its `Host`, its
  credential (a GitHub forge's the longer of `token_bytes` and
  `github::jwt::JWT_BYTES`, github-app.md, section 1.1), the user
  agent (the owner's, of any length) and 192 bytes of framing; and for
  each log origin, its target and host likewise, with no credential;
- with a GitHub forge configured: `deadline` at most 270 seconds, half a
  JWT's life (`connection.md`, 5.2); `renew_before` at least twice `deadline`, so a mint
  that fails is asked again before its token expires, and at most 30
  minutes, half an installation token's hour; and `key_bytes` at least
  `skein_tls::RS256_KEY_BYTES`, the largest 2048-bit key's DER (../net/tls.md,
  section 4);
- `hook_secret_bytes` is at least 32;
- each stacked machine's largest demand fits the cap of the stream
  below it: HTTP over TLS and io, TLS over io, JSON over HTTP, and the
  listener's server over io and JSON over it.

### 1.5 Memory at the edges

The configuration checks every stack's demands and room, the admitted
operation and answer sizes, the per-forge credentials and tables, and
the listener's input. A long thread or a large payload is streamed
within those caps. Internal collections grow with admitted work; a
sum of all their possible allocations is not an admission check.
Worlds hold measured peaks under named scenario ceilings with headroom.

## 2. State machines

### 2.1 A forge connection

| State | Holds, and demands | On |
|---|---|---|
| Closed | nothing | a call, or a request of its forge's own: Connecting |
| Connecting | the holder | connected: Handshaking, or Sending without TLS; failed: `Unseen`, Closed |
| Handshaking | the holder; TLS's demand | done: Sending; failed: `Unseen`, Closed |
| Sending | the holder, its request; room | sent: Reading; a fault: `Unseen` if skein-http says `Closed`, else `Seen`; Closed |
| Reading | the holder, the decoder; the head, then the body as demanded | answered: the next step's Sending, or Idle with the answer up; a fault: `Seen`, Closed; a malformed answer is heard as such (dialects.md, 1.2), Closed |
| Idle | nothing; none | a call to its forge: Sending; `idle`, the forge closing, or a call to another forge with none free: Closed |

A connection the forge marked `Connection: close`, or that read a raw
body past its bound, goes to Closed after its answer, not to Idle.

The holder is a call, or a request of its forge's own (1.1): a start
read (`connection.md`, 3.3, 3.4), a mint (`connection.md`, 5.1), an installations' read (`connection-calls.md`, 1.4) or the
puller's (`connection-webhooks.md`, 1.5), whose failures go to what asked it. Deadlines are the holder's: every phase but
Closed and Idle waits until it, so no exchange outlives what it serves.
At it the exchange fails `Unseen` if no byte of it was written, else
`Seen`, and the connection is aborted. Which deadline runs is a function
of the phase, set in one place after each transition
(../../foundation/programming-model.md, section 5.4). A connection stands on a link
(../net/link.md): a closing one sends TLS's `close_notify`, and is bounded by
the link's close deadline and then io's.

A forge connection speaks no dialect. Its stack carries an exchange's
bytes, tokens and ending; the exchange itself (the table's request and
decoder, and what was heard of it) is its holder's, in its holder's dialect, made new
each time the holder takes a connection. So a plan and a connection of
different dialects cannot meet, and the connection's own table above
holds for both.

### 2.2 A call

Waiting for its forge's start; Resolving a path or a login (`connection-calls.md`, 1.2, 1.3);
running its plan's steps on its connection; then answered, once.

### 2.3 A forge's start

| State | On |
|---|---|
| Unconnected | a connect: Checking, or Listing for GitHub; one `connection.md`, 3.5 refuses: Stopped |
| Checking | the identity read: the configured user: Settings; another user, `401`, `403`, any other `4xx` but `429`: Stopped; unreachable, `429`, `5xx`, malformed: Waiting |
| Settings | the page-cap read: Ready; a `4xx` but `429`: Stopped; unreachable, `429`, `5xx`, malformed, a cap of 0: Waiting |
| Listing | GitHub's installations read: Minting; `401`: Stopped, `KeyRefused`; another `4xx` but `429`: Stopped; unreachable, `429`, `5xx`, malformed: Waiting |
| Minting | each accepted installation's tokens (2.6): Identity, or Ready with none accepted; refused: as `connection.md`, 5.1 says |
| Identity | the viewer read: the access's user: Ready; another user, a `4xx` but `429`: Stopped; unreachable, `429`, `5xx`, malformed: Waiting |
| Waiting | the backoff: Checking, or Listing for GitHub |
| Ready | serves calls; a connect: Checking, or Listing for GitHub |
| Stopped | says `stopped` once, naming the forge, and holds no access; a connect: as Unconnected |

A token has only one state, held. An installation's tokens are 2.6's.

### 2.4 A webhook connection

The answers are `connection-webhooks.md`, 1.3's.

| State | Holds | Deadline | On |
|---|---|---|---|
| Head | the head so far | `hook_deadline` | a signed `POST` to a forge's path within `hook_body_bytes`: Body; any other head: its answer, Closing; past the deadline: Closing, no answer |
| Body | the HMAC state, the fields decoded | `hook_deadline` | read and verified: Answering; a wrong signature or a body that does not decode: its answer, Closing |
| Answering | the hint, or none | `hook_deadline`, from its `204` | `204` queued: the hint offered up, held while the owner has no room (`connection.md`, 2.2); then Head (kept alive) or Closing |
| Closing | nothing | io's close deadline | closed |

A fault or the deadline in any state closes the connection with no
answer and no hint.

Answering has no moment before its `204` is queued that its owner can
hold. With the room the owner leaves each entry point (`connection.md`, 2.1), the step
that verifies the body queues the `204`, sent from the room skein-http
set aside for its head when the request came, and then offers the hint,
in that entry point (`connection-webhooks.md`, 1.2). Answering lasts only while its hint is held, so
its deadline and the owner's close meet it with its `204` queued and its
hint held: then the hint is dropped, never offered. A world proves the
order inside the step, the `204` queued before the hint and one of each,
and crosses Answering's deadline and the close with a held hint.

### 2.5 Ending

- **The close is the owner's last act,** after its domain's drain
  (../../foundation/programming-model.md, section 5.2). By then the domain takes no
  answers.
- **A call after the close is refused at the entrance,** until `closed`:
  it ends at once `Unavailable`, at no cost.
- **It ends what is open.** Each call still open ends with its
  connection and has no answer: no terminal is sent for it. It drops
  every answer and hint it holds, and every answer the owner returned,
  and offers none again. The next process reads afresh (client.md,
  section 8).
- **It closes what it keeps:** its connections, the listener and each
  webhook connection. **`closed`** goes up once, when none is left and
  io has settled every socket. It is the last event.

### 2.6 An installation's tokens

| State | On |
|---|---|
| Unminted | accepted, not suspended, its forge past Listing: Minting |
| Minting | minted: Current, with a `git token` up; `401`: the forge Stopped, `KeyRefused`; `403` or `404`: the installations read again; another `4xx` but `429`: the forge Stopped, `TokenRefused`; unreachable, `429`, `5xx`, malformed: Waiting |
| Current | due, `renew_before` its expiry: Minting, the old token still sent until it expires; a `401` on it in a call: Minting |
| Waiting | the backoff: Minting; calls wait once the old token expires |
| Gone | deleted, suspended, or no longer accepted: its tokens dropped; accepted again, or unsuspended: Unminted |

Unminted and Gone also run while the forge is Ready: a newly accepted
or unsuspended retained row becomes Unminted. A committed deletion or
suspension, or an accepted-set removal at once, drops its Full token
and both expiries and invalidates its mint, Identity and held git
output. Identical sets, metadata-only reads and renames keep Current
tokens. Eligibility revisions reject an older answer even after loss
and reacceptance; an already-written exchange settles before the own
connection is reused. Events use the newest account.

Ready-time mints, reads and Identity share the reserved own connection,
one exchange at a time. Public Ready and other Current rows survive
private backoff. A mint's transient failure retries its selected scope;
`403`/`404` drop its tokens and wait before a whole installations read,
with no mint from stale metadata. Backoff doubles up to
`start_backoff_max`, and a longer Retry-After wins beyond that maximum.
The wait wakes at its future deadline. Changes and a held git token
share the remaining output reservation; eligibility is checked again
when the token is released.

When Ready was reached without an eligible installation, the first
later Full/Contents sequence runs `connection.md`, 3.4's Identity on Full. Until it
succeeds the candidate is ineligible for calls. A matching bot sets
identity_checked without another `connected`; a refusal may emit the
generation's one `stopped` after `connected`. A transient Viewer retry
retains its valid candidate and repeats neither Contents nor its git
event. An expired candidate starts fresh mints, and loss invalidates
the check so another eligible row may supply it. Replacement resets
the check; Close discards all private work and token values.

### 2.7 The delivery log's puller

| State | On |
|---|---|
| Resting | `log_poll`, its forge ready: Reading |
| Reading | a page's rows, each as `connection-webhooks.md`, 1.5 says; the guid kept, `log_catch_up`, `log_pages` or the log's end: Resting, the newest guid kept; a row to read whole: Delivery; `304`: Resting; `401`: the forge Stopped, `KeyRefused`; anything else: Resting, the guid kept as it was |
| Delivery | read and decoded to a hint: Holding; to none, past `delivery_bytes`, or failed: Reading |
| Holding | the hint taken: Reading |
