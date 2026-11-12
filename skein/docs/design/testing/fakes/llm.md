# Shared scripted LLM peer

Provisional, 2026-10-05, revised 2026-10-19. The scripted peer that worlds
and an application's real loop call in place of an LLM provider.

## 1. Role

Skein owns reusable provider-neutral LLM calls and their independent wire
peers. `skein-fake-llm-domain` depends only on `skein-lib`; it knows no HTTP,
JSON, provider client, application tools, checkout paths or credential policy.
`skein-fake-llm-protocol` depends on that domain and the shared HTTP/SSE/JSON
and LLM codecs; provider codecs are `skein-llm`'s (../../llm/README.md). Application
worlds supply their own tool schemas, inputs, invalid calls, scripts and
effect expectations, and interpret the answers themselves.

## 2. Script domain

`Config` supplies fixed pending-call and owned-byte limits, seeded failure
chances, injected latency and output limits, and the cache, usage and
tool-choice settings of section 2.1. `Domain::configured` consumes
scripts and `Menu` together under `script_bytes`. The menu supplies complete
argument bodies and deliberately invalid names/bodies. An empty menu produces
no random tool calls. No application body or unknown tool name is invented.
`Line::Opaque` carries complete caller-supplied provider replay bytes; the
domain counts and preserves them without parsing.

`Event::Call` transfers a query and one reply right. A request is refused or
produces one delayed `Request::Reply`; `fire` uses only injected time.
Conversation validation requires every result ID to answer exactly one prior
call ID; tool calls belong to assistant messages and results to user messages.
Generated provider-call names use checked counters; exhaustion
refuses rather than reusing an identity. Query, script/menu and generated
answer ownership is checked before effects or payload copies.

### 2.1 The cache, usage and tool choice

- **A query says how it may be cached,** in the domain's own terms, which
  the byte peer fills from the wire (section 3.1): automatically, by
  prefix, within a scope, the opaque key a provider routes by, or none;
  or only up to marks, the positions the request asked to cache.
- **The domain keeps what calls wrote:** a bounded table of prefixes, each
  with its scope, its length in tokens and a digest of its content,
  written at injected time and expired after the configured lifetime. The
  oldest entry gives way when the table is full.
- **A query's prefix** is its model, its tools and its system text,
  then, for a marked query alone, its tool choice, then its messages. A
  changed choice therefore keeps an automatic query's whole prefix, as
  the Responses wire does, and only a marked query's tools and system text,
  as Anthropic's does.
- **Reads.** An automatic query reads the longest prefix written in its
  scope that it begins with. A query without a scope reads only with the
  configured chance, as a provider's random routing does, so a world can
  tell an affinity that reaches the wire from one that does not. A marked
  query reads the longest prefix, ending at one of its marks, that an
  earlier call wrote, and never past its last mark.
- **Writes.** An automatic query writes its whole prompt. A marked query
  writes the prefix at each of its marks; what it writes past what it
  read is its cache write.
- **Usage is optional, field by field.** The domain's usage has the same
  five fields as skein-llm's `Usage` (../../llm/README.md, section 2.3), each present
  or not. The
  configuration says which the wire reports, per dialect; the peer omits
  the rest, so worlds meet usage not reported as well as usage of zero.
- **Tool choice.** A query carries its choice: with `None` the domain's
  random answer calls no tool, and with `Only` it calls only the tools
  named. With the configured chance it calls outside the choice anyway,
  as a provider may, so a world sees its caller answer such a call as not
  run. Scripted turns are the application's, and call what they name.
- **Cut and oversized calls** need nothing new: an answer longer than the
  query's output cap is cut short inside its first tool call, which the
  peer sends as its dialect's own truncation, and a menu body longer than
  the client's input limit is an oversized call.

### 2.2 A reply in pieces, and a hold

- **A scripted reply can come in pieces.** The script gives the reply's
  text as an ordered list of pieces, and the peer sends each as a delta
  of its own, in order, in the dialect's own streaming events, so the
  client under test sees the reply arrive by parts, as it does from a
  provider. A reply scripted whole is one piece. The pieces' bytes are
  the reply: concatenated, they are the text the committed answer holds.
- **A piece the script marks held is not sent until its owner releases
  it.** Pieces before it are sent at their latency; the held piece and
  those after it wait, and the stream stays open, with the dialect's
  keep-alive if it has one. A world therefore observes a reply partway:
  what reached the client is exactly the pieces before the hold.
- **The owner releases by name.** The script gives a hold a name, and
  the world that owns the domain releases it by that name, at any time,
  in its own iteration (the domain stays pure: a release is an event,
  not a timer or a callback). Releasing sends the held piece and the
  pieces after it up to the next hold, each at its latency. A release
  of a name nothing holds changes nothing. A reply cut or refused by
  a fault while held ends as the fault says; the hold is dropped with
  the reply.
- **Holds and the peers.** The domain keeps the holds, so the byte peer
  and the worlds that host it (the one-process world and the real loop:
  ../../../foundation/testing-strategy.md, 4 and 4.1) take the release as the domain's
  event, never from the wire. The peer as a process of its own has no
  owner to release a hold unless its host gives it one, a control of
  the host's beside the provider's listener that takes the release as
  the domain's event (`Peer::release`), so without one it refuses a
  script with a hold at its start. A world without a hold sees every
  piece sent at its latency.

### 2.3 Scripted failures

- **A turn may fail.** A failing line, `Line::Fail`, answers with a
  provider failure: its class, as the domain's error; the provider's
  error code; its request id; and the field it names, for `Unsupported`.
  The domain's error has `Refused` and `Unsupported`, and `Exhausted`'s
  reset is optional, as skein-llm's failure has them (../../llm/README.md, section
  2.5). Exhausted with a reset or without, refused, unsupported and
  unauthorized are its point; any class with a row on its route may be
  scripted (below).
- **Where it falls.** A failing line that names no credential ends its
  turn: a line after it is refused when the script is configured. One
  that names credentials (below) ends the turn for their calls alone,
  and may be followed by lines, which the calls that pass over it
  answer. Reached before any other line of its call's reply, a failing
  line answers before any stream, with its route's status and error
  body; after other lines, it ends the stream as its route fails one
  (`response.failed` on Responses, an `error` event on Anthropic), once
  their pieces are sent, a hold's included when released (2.2).
- **As the route says it.** The byte peer renders the class as its
  route's section names it (../../llm/README.md, sections 4.7 and 4.8): the status, a
  code the section maps to that class unless the script gives one, the
  reset as the next bullet says, and the field as `error.param`. Where
  the section reads the class from the error's message, as Anthropic's
  context too long (../../llm/README.md, section 4.7), the peer sends that message.
  So the client's own classification reads back the class the script
  gave.
- **A reset, per route.** On the public route,
  `Exhausted`'s reset goes in the error's `resets_in_seconds`, in whole
  seconds rounded up, never in `retry-after`, before a stream as in
  `response.failed`. Anthropic's `Exhausted` has no reset; a script that
  supplies one is refused at configuration (../../llm/README.md, section 4.7).
- **A class needs its row.** A class has a row on a route where the
  route's section names an error for it, not a status alone; a failing
  line of a class without one on its peer's route is refused at
  configuration. `Unsupported` has none on Anthropic, whose refused
  fields are `Invalid`. `Overloaded` has one on every route: on
  Responses, a 503 whose error is `overloaded_error` (../../llm/README.md, section
  4.8).
- **Whose calls it fails.** A failing line may name the accepted
  credentials it answers, by index (section 3), which the byte peer puts
  in each query; it fails only their calls, and a call with another
  credential, or with none, passes over it to the rest of its turn.
  Naming none, it fails every call. An `Exhausted` line with a reset
  fails the calls it names until its reset has passed since the first
  of them it failed; at the reset and after, they pass over it to the
  rest of its turn, as a provider serves the account again after its
  reset. The domain keeps that first failure's injected monotonic time
  per line and named credential; a line naming none shares one time
  across all calls. This state fits under `script_bytes` with the script.
  A line without a reset, and every other class, fails each call made
  again. A failure adds no assistant message, so the same turn answers
  the call made again: a story fails one account's calls, lets another's
  through, and so drives each account's state deterministically.
- **The evidence is the script's.** The code, the request id and the
  field are sent as given, past the client's bounds too, so that a story
  sees the client keep each or leave it empty (../../llm/README.md, section 2.5). An
  empty request id is the peer's own (section 3.1).
- **The chances stay,** for soak runs: a seeded failure carries its
  route's code for its class and the peer's own request id, and names no
  field.

### 2.4 Which script answers

- **A system cue chooses first,** bytes the script looks for in the
  query's system text. The cue that starts earliest there answers, and
  of two at one position the first listed. Its turns count the query's
  assistant messages; past its last turn it answers "done".
- **A request may cue one too.** `Scripts { cued, asked, otherwise }`
  holds the scripts cued by the system text or the request, `cued`;
  those cued by the request alone, `asked`; and the turns that answer
  when neither cues one, `otherwise`. A request cue from either group
  is looked for in each text part of the last user message that holds
  text, the person's latest request. Parts are searched in order;
  within each part the earliest cue wins, then the first listed, with
  `cued` before `asked`. Its turns count the assistant messages after
  that message, so each new request starts its script again, including
  a `cued` script selected by the request; `otherwise` counts the same
  way. Past their last turn, both answer "done".
- **In that order:** a system cue, then a request's, then `otherwise`
  when it holds a turn, then the random answer, each after the seeded
  chances (section 2.3). A world that gives neither `asked` nor
  `otherwise` is answered by a system or request cue, or at random.
- **`Domain::played(&Config, seed, Scripts, Menu)`** checks and consumes
  them as `configured` does its scripts, all under one `script_bytes`,
  and `configured` is `played` with the other two empty. Their turns
  are checked as a cued script's are (section 2.3), and a hold in one
  is placed and released as in a cued script (section 2.2): its
  `Place`'s script counts the cued scripts, then the asked ones, then
  `otherwise` as the last.
- **Who uses it:** an application's demo model, whose scripts its
  document gives (temper's `protocol/setup/playground.md`), and a world that
  answers a person's words rather than a charter's.

`Domain::played_by_input(&Config, seed, Scripts, Menu, Rules)` admits
optional turn rules beside the scripts. `Rules.cued` and `Rules.asked`
are index-parallel to their script groups; missing or empty entries keep
the counting above. Script selection still follows system, request, then
fallback order. Each nonempty rule list is searched in declared order;
the first matching `Rule.cue` selects its zero-based `Rule.turn`. Exactly
one empty cue must end the list and always matches as its default. A
table longer than its group or a turn outside its script is refused.
Rule arrays and cue bytes share `script_bytes` with scripts and menu;
empty tables charge nothing. `played` supplies empty rule tables.

Fresh input is the parts of messages after the latest assistant message,
or all conversation messages when there is no assistant message. Rules
search each text part's text and each tool output's output, never system
text, older history or opaque blocks. A cue never spans two parts. Rule
order takes precedence over part or message order, so expected input can
win when delivered in the same fresh batch as other news.

An opted-in script has no count or conversation cursor: a retry or repeated
request plays the same selected turn, and an older message never selects
a turn. Its answer, failures, pieces, holds, accounting and `(script,
turn)` place belong to that turn; selection draws no randomness. A recap
is not understood, but a known recap prompt can have its own rule.

### 2.5 Strict

- **A switch, off by default.** `Config.strict`, a `bool`. Off, a turn
  no script answers takes "done" or the random answer, as 2.4 has it.
  On, it fails instead, so that a world or a QA box whose scripts miss a
  turn sees the turn fail rather than drift (temper's
  `draft/landed-qa.md`, 6).
- **What no script answers,** under the switch: a script that 2.4
  selects, by a system cue, a request cue or as `otherwise`, called past
  its last turn; and a call that nothing selects, no cue matching and
  `otherwise` empty, which would take the random answer. A script with
  turn rules always plays one of its turns (2.4), so it never runs past
  its last.
- **The failure,** in place of the answer, after the seeded chances
  (2.3), as a failing line falls before any other line of its reply:
  class `Invalid`, its report's code `no_script`, the peer's own request
  id (3.1), no field, and provider text that names the conversation's
  last request:
  - `no script answers: past the N turns of the script cued "CUE"; last
    request: "TEXT"`, for a cued or asked script past its last turn, N
    its turns and CUE its cue; for `otherwise`, `past the N turns of
    otherwise`;
  - `no script answers: no cue matches; last request: "TEXT"`, for a
    call nothing selects.

  TEXT is the text of the query's last user message that holds text,
  its text parts joined by a line feed: the person's latest request,
  where 2.4 looks for a request's cue. At most its first 512 bytes, cut
  back to a character boundary and followed by `…` when cut; empty when
  no user message holds text. The text is one line of the message as
  given, quotes and line feeds included, never escaped.
- **No assistant message,** as with any failure (2.3): the same call made
  again fails again, until the conversation changes.
- **On the wire,** the byte peer renders it as its route renders
  `Invalid` before a stream (2.3; ../../llm/README.md, sections 4.7 and
  4.8): the route's status and error type, the code as the error's code,
  and the provider text as the error's message, in place of the message
  a code alone gives. So the client reads `Failure::Invalid`, its
  report's code `no_script`, and the text in its detail
  (../../llm/README.md, section 2.5).
- **Nothing when off:** no script byte and no state; `configured`,
  `played` and `played_by_input` take the switch from the `Config` they
  are given, and every world that sets none keeps 2.4's answers.

### 2.6 Values taken from the conversation

- **A turn may take a value from the conversation** it answers, so that
  a script made from one run replays on a deployment whose numbers and
  ids differ: a task's number, a pull request's, a commit's id (temper's
  `draft/landed-qa.md`, 6 and 10). A take is a line,
  `Line::Take { name, before, after }`, that adds nothing to the reply:
  it binds `name`, for the lines after it in its turn, to the value the
  conversation holds between `before` and `after`.
- **Where it looks:** the query's messages from the last to the first,
  in each its parts from the last to the first, each text part's text
  and each tool output's output; then the system text. Tool calls and
  opaque blocks are not read. In each, the occurrences of `before` from
  the last to the first: an occurrence's value is the bytes from its end
  up to the first `after` after it, and it holds one when that value is
  1 to 256 bytes with no line feed. The first that holds one gives the
  value, so the newest wins. A value never spans two parts.
- **Its uses,** in the lines after it in the same turn, replaced as the
  reply is made:
  - in a text line's text, `{{NAME}}` by the value as it is;
  - in a call's arguments, first `"{{#NAME}}"`, its quotes included, by
    the value as a JSON number, which must then be 1 to 20 ASCII digits,
    with no leading zero but `0` alone; then `{{NAME}}` anywhere else by
    the value escaped as a JSON string's content: `"` and `\` after a
    backslash, and each byte below 0x20 as `\u00XX`, so that a
    placeholder inside a JSON string leaves it one.

  A `{{...}}` that names no take earlier in its turn is left as it is,
  as are call names and every line before the take.
- **Checked when configured,** under `script_bytes` with the rest of
  the scripts: at most 8 takes in a turn; `name` 1 to 32 bytes of ASCII
  letters, digits and `_`, distinct within its turn; `before` and
  `after` each 1 to 256 bytes. A turn whose lines hold pieces (2.2)
  holds no take, since its pieces stream from the script as written.
- **Bounded before it is copied:** a turn's answer counts each use at
  its replaced size, an escaped byte at six at most, against
  `answer_bytes` with checked arithmetic, before any of it is copied, as
  `limits::scripted_answer` counts an answer now.
- **No value fails the turn,** whether or not strict is on (2.5): a take
  that finds none, or a number's use whose value is no number, answers
  in place of the turn's reply as 2.5's failure does: class `Invalid`,
  its report's code `no_take`, the peer's own request id, no field, and
  provider text
  - `no value taken for NAME: "BEFORE" … "AFTER" is nowhere in the
    conversation; last request: "TEXT"`, or
  - `the value taken for NAME, "VALUE", is no number; last request:
    "TEXT"`,

  TEXT as 2.5 gives it. A turn that cannot play as its run did fails
  rather than drift. No assistant message is added, so the call made
  again fails again until the conversation holds the value.
- **No state:** a take reads only the query, as a turn's selection does
  (2.4): a retry takes the same values, and a deployment restored from a
  copy takes what its own conversation holds. Turn rules (2.4) select
  the turn; its takes then play as in any other.
- **Who uses it:** temper's twins as QA's setups, through its document
  form's `{"take", "before", "after"}` (temper's
  `protocol/setup/playground.md`; `testing/fakes.md`, 1).

## 3. Byte peer

`Service` owns bounded routing across connections; `Server` owns one HTTP
exchange and one SSE writer. Start/up/down/resume entrances reserve their
published output maxima. Lower room and reads are explicit. A complete
provider request becomes the neutral domain query, and an actual domain
terminal becomes a bounded native stream.
`Service::target` consumes a whole domain terminal and returns it alongside
the actual connection token; an unmatched/closed route returns the same
owned terminal. Looking up a route never discards its reply right.

**Credentials.** The peer accepts several credentials, each a
`skein_llm::Credential` with its scheme (../../llm/README.md, section 2.6), so that one
peer serves several accounts. `Peer::new` takes one, the index 0, and
`Peer::accept` adds another and returns its index. Every credential is
accepted before the peer serves its first request, and none after. Each
entry receives the accepted credentials, borrowed. A request is
authorized when it presents one of them as its route sends it: on
Responses, either scheme as `Authorization: Bearer`; on Anthropic, a key
as `x-api-key`; the peer refuses a bearer on Anthropic, as the client does
(../../llm/README.md, section 4.7). A peer whose wire configuration takes
calls with no credential, as a local model's server does, also
authorizes a request that presents no credential field (../../llm/README.md, section
2.6). Anything else, a credential it
does not hold or no credential without that configuration, is answered
as unauthorized.
The fake compares bytes; it owns no sign-in, refresh, JWT parser,
credential generation or durable credential record. The caller remains
the credential owner. The peer's provider selection is wire
configuration, never application policy.

**No observation holds a credential's value.** The head observation
leaves out the credential's fields (`Authorization`, `x-api-key`). `Observation::Credential` records, for each
request, its route (`skein_llm::Provider`) and what it presented,
each distinct: an accepted credential, by its scheme and its index; a
credential that matched none, by the scheme its field shows (a key in
`x-api-key`, else a bearer) and no index; or no credential field at
all, as the scheme none and no index.

`Close` requests lower settlement; `closed` acknowledges actual settlement.
A lost connection cannot authorize effects or turn a withdrawal into a
provider answer. The independent script domain still produces its actual
terminal; old connection routing may no longer deliver it.

**On io,** the byte peer also runs as a process of its own. It listens on
loopback, in plaintext, or in TLS with skein's test certificates, so that
a simulated world and the real loop host it as they host a service
(../../../foundation/testing-strategy.md, 4 and 4.1). The fake issuer runs the same way
(../../oauth/README.md, 5). An application that serves it to a person, as a demo's
model, makes it keep no records (`without_records()`, forge.md,
section 5).

**The test root is exported,** so that a client in another process can
trust a TLS peer: `skein_fake_peers::transport::ROOT`, the DER bytes of
the root that signs the fixed chain the peers present, whose leaf names
`skein.test` and `127.0.0.1`. The owner writes it to a file the client
is given (temper's fake LLM document's `tls.root`, temper's
`protocol/setup/playground.md`); the client trusts that root
alone, and reaches the peer as `https://127.0.0.1:<port>`. A test root
vouches for nothing outside a test: its key is in the tree.

### 3.1 Request heads, affinity and echoes

- **The peer records each request's head:** its fields, names and values,
  within the observation caps, beside the decoded query, so a world can
  assert what reached the wire: that every request of a conversation
  carries the same affinity.
- **It reads a gzip body back with an inflater of its own.** A body sent
  with `Content-Encoding: gzip` (../../llm/README.md, section 4.8)
  is inflated beside its member, in room the server holds from the start:
  the gzip bound of the request cap, twice. The inflater is test code,
  simple over fast, and shares nothing with the client's encoder. A body
  in any other coding, under two codings, or a member that does not
  inflate whole to its CRC and length is refused as invalid.
- **It reads the dialect's cache instructions into the query.** The
  public route: `prompt_cache_key` must be the key in its rendered form,
  and a `session-id` or `thread-id` header is refused. The key becomes the
  query's scope. Anthropic: each `cache_control` marker becomes a
  mark; there may be at most four, never on thinking or redacted
  thinking. A request that breaks a rule the dialect promises is refused
  as invalid: the fake checks its client (../../../foundation/testing-strategy.md, section 6).
- **It refuses what a plan's route refuses.** On the public route, a
  request carrying a field of ../../llm/README.md, section 4.8's refused list is
  answered as unsupported, naming that field.
- **It answers with a request id,** in each response's head under its
  route's name (`x-request-id` or `request-id`):
  numbered as its response ids are, unless a failing line gives one
  (section 2.3).
- **It echoes as a provider does.** Its Responses events can carry
  the request's instructions and tools and a usage
  attribution entry per input item, sized by configuration, so worlds
  exercise a client that keeps only what it reads (../../llm/README.md, section 4.4).

### 3.2 A list

- **The route's `list`,** bytes its configuration gives, answer `GET /v1/models`
  (and with a query) as a provider's body, under the same credentials as a
  call (section 3). A route with none answers `404`, as a local model's server
  may. The bytes are the world's own, so that a parser's reading of them is
  proved and not assumed.
- **A failure:** `list_status`, a status it answers in place of them, so a world
  can have a read fail and then succeed, by `Peer::set_list` between requests.
- **Its observation:** `Observation::List`, the route and the credential it
  presented, as `Observation::Credential` says it. A list spends no script, and
  counts no call.

## 4. Shared codec and replay

`skein-llm::DocumentLimits` is the neutral document-bound vocabulary. Native
Anthropic peer decode/encode entrances live in the shared codec, not an
application's adapter. Tool schemas remain whole bounded JSON values.
Signed/redacted thinking preserves unknown extension fields. Required known
fields and complete JSON shape/bounds are checked; thinking/signature deltas
replace only their own values in the retained provider head.
The native request decoders read the dialects' cache instructions and
tool choice into the neutral query (section 3.1), and the usage encoders
write only the fields the configuration reports (section 2.1).
Unknown nonempty assistant-native block kinds keep their complete object,
including nested proof fields, without inferred effects or deltas. Known
text/tool/thinking kinds cannot bypass their admission rules by masquerading
as the opaque variant. Duplicate queried known fields are refused; unknown
extension keys remain uninterpreted, including duplicates.

`Replay::to_bytes` and `Replay::from_bytes` provide a versioned opaque durable
envelope. Its seven-byte version/tag/length header is additional to the raw
`opaque_bytes` allowance. `replay_bytes` exports the complete receiver cap;
`replay_bound` includes temporary JSON and simultaneous raw/envelope
storage. A complete raw replay admitted at its cap still fits its envelope.
Provider compatibility is validated by the real Client when replay is used.
`Prompt::output_ceiling` applies configured dialect support: Responses omits
its unsupported wire field, the public route refuses an explicit cap at
admission (../../llm/README.md, section 4.8), and the caller still bounds local output.

## 5. Outside stories and bounds

`skein-llm-kit::World::prepared` adopts one freshly prepared Client and
caller-owned literal HTTP/SSE response bytes. The caller supplies the same
limits used at preparation. Adoption has no stream entrance, send, read or
terminal effect and constructs no second Client. `World::new` prepares once
and delegates to this identical initialization. Demand and intake,
fragmentation, queue limits, terminal checks and lower settlement are shared;
the constructor neither parses fixtures nor controls application policy.
The original Client keeps its callback owner and prepared request bytes.
Its price is counted once; the world's source tape, request tape, terminal
observations, queues and intakes remain separate owned storage. Its stories
expect literal bytes, never the peer's response encoder's output, and
establish no live provider admission.

`skein-llm-kit::fake::Exchange` connects the actual shared Client to the
independent byte peer and real script domain. It records requests and
their heads, queries, response bytes and actual terminals. Its transport uses bounded intakes and
checks every read/room/send; it owns no provider parser. Its stories
carry whole schemas, literal argument bytes, exact paired feedback and
continuation through both wire configurations, then corrupt provider IDs.
Cancellation has no terminal before actual lower settlement; repeated close
settlement produces no second terminal.

`Exchange::prepared` adopts an application's already prepared actual Client.
The caller moves the exact endpoint, credential, if any, and unchanged
receiving limits used at admission as independent peer metadata, released
after configuring the peer. It creates no second Client and performs no
second preparation; `Exchange::new` prepares once and delegates to the
same constructor. The Client is priced once. Peer credential bytes and
the retained target are separate ownership, as are the caller's retained
application declarations, decoded results and observation buffers.

`Exchange::at(now, wall)` installs the caller's one iteration snapshot in
the actual Client, independent script domain and byte peer before their
entries run. The caller supplies nondecreasing monotonic `Time`; `Wall` may
jump independently and never arms deadlines. Installing time neither starts
the exchange nor fires timers, delivers bytes or settles effects; the caller
still drives the existing entrances. It adds no retained state or memory
allowance.

`Exchange::observe(ObservationLimits)` opts a fresh, unstarted exchange into
fixed observation counts and whole-record byte caps. Events, decoded queries,
request heads, manual pending calls and both wire tapes reserve their exact
wrapper capacity.
Payload ownership is checked before cloning or appending, including public
native replay token wrappers and their owning bytes. Drained records become
caller ownership; a moved vector's replacement capacity is reserved at the next
physical entrance. Without `observe`, observation is unconstrained.

The driver reclaims retired service routes after draining their actual outputs;
`Service::calls()` reports allocated live and retired slots without content.

Memory stories hold the counting allocator's high water under named
scenario ceilings with headroom: replay transit, extended opaque heads,
a maximum-size scripted response, the joint script/menu cap with every
delayed-call slot at its full answer cap (one over each cap is refused),
and two connections sharing one delayed script domain. They establish shared
Client/peer ownership only: application worlds add their schema, result and
replay copies and their outside effect ledger to this envelope.

**The accounts' stories,** through `Exchange`, which accepts further
credentials for its peer as `Peer::accept` does: a peer holding
credentials of both schemes on one endpoint answers each as its own, and
observes which by index; each scripted failure reaches the client's
terminal with its class and report, and a code, request id or field past
the report's bound arrives empty; a failing line naming one credential
fails its calls and lets another's through to the lines after it; a
scripted reset reaches the client's `Exhausted` in whole seconds, on
each route that carries one; a line after a failing line naming no
credential, a class without its row, and an Anthropic reset on a key or
after the stream has begun are each refused at configuration; a call
with no credential is observed as the scheme none, and answered by a
peer configured to take such calls and as unauthorized by another; a
credential the peer does not hold is observed apart from both, by its
scheme with no index, and answered as unauthorized; a public-route
request with a refused field is answered as unsupported, naming it; and
no observation holds a credential's value.

The redacted Anthropic captures beside the shared codec keep their
provenance. They flow through the actual Client under fragmentation and
prove wire preservation, not current live subscription admission.

## 6. The real loop

An application's real loop (../../../foundation/testing-strategy.md, section 2.8) runs the
peer in its own process, on loopback, with no injected latency, as it runs
the fake forge (forge.md) and the fake issuer (../../oauth/accounts.md, section 8).
Its script is the same `Script`, `Turn` and `Line`, and the referee reads
the same observations.

- **Most stories give an account a key** (../../llm/README.md, section 2.6), which the
  peer accepts: no sign-in is driven.
- **Stories of an account with no credential** configure the peer to
  take calls with none (section 3).
- **Stories that sign a person in** run the fake issuer beside the peer,
  and the test drives the redirect. The issuer mints no token: its
  script lists every access token it grants, each refresh's included
  (../../oauth/README.md, section 5). The world that runs both gives the peer that
  list, as bearers, when it starts it (`Peer::new` and `Peer::accept`,
  before it serves, section 3). So a signed-in account's calls are
  accepted without the test reading a token. A token the issuer revokes
  stays accepted by the peer: a story that wants it refused scripts an
  `Unauthorized` line naming its index (section 2.3).
- **Stories of a program that signs in by itself** use a fake program; the
  credential it hands over is one the peer accepts.
- **An account's state is the script's:** failing lines that name a
  credential exhaust, refuse or unauthorize one account and not another
  (section 2.3), so a story asserts what its application does with each.

## 7. Open questions

- **Cache granularity.** Providers cache only past a minimum length and in
  steps of tokens; the fake caches any prefix at a message's or a mark's
  end. Default: it stays so, until a world needs the providers'
  granularity to catch a regression.
- **Digests.** The cache table compares prefixes by a digest of their
  content, not the content itself, to keep its bound small. A collision
  would show as a false hit; with a 64-bit digest and a world's few
  prefixes, it is accepted rather than paid for in memory.
- **A failure's reset as a header.** Section 2.3 renders a reset on the
  public route in the error's `resets_in_seconds`,
  though the client also reads `retry-after` (../../llm/README.md, section 4.8). What
  stays open is which the public route sends. Default: the error's field
  alone, until a capture shows it.
