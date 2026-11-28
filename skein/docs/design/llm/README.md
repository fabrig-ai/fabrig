# LLM calls

Provisional, 2026-10-09, revised 2026-10-20. `skein-llm` is a reusable
composition of HTTP, SSE and a provider's JSON dialect. It depends on
`skein-lib`, `skein-http` and `skein-json`, and knows nothing of a
service's domain, sockets, credentials store or tools. This is a
composition crate, rather than a primitive protocol machine: the lib-only
rule for the individual machines remains in force.

Providers are OpenAI's public Responses API and Anthropic's Messages API.
OpenAI's default endpoint is `api.openai.com/v1/responses`, called with an
API key or with the access token of a sign-in that pays with the
person's plan, both sent as a bearer (`wire.md`, section 1.8). Anthropic's is
`api.anthropic.com/v1/messages`, called with an API key in `x-api-key`
and an API version (`wire.md`, section 1.7). A credential is a value and how it is
sent (`vocabulary.md`, section 1.6): the caller obtains and renews it, and sign-in and
refresh are outside this crate (../oauth/README.md). Transport connects to the
endpoint's authority and provides a secured plaintext stream. Endpoint
overrides support fakes and explicitly chosen deployment routes; the
caller must bind the stream to the endpoint it supplied. Client identity
strings are optional data, never installed as defaults. Version overrides
are explicit endpoint headers; reserved credential and framing headers
are refused. The caller selects the model and any required compatibility
headers.

- [`vocabulary.md`](vocabulary.md): provider-neutral types and ownership.
- [`wire.md`](wire.md): bounds, encoding, decoding and provider dialects.

## 1. In one page

- **One call, one terminal.** A client prepares one call from a neutral
  prompt, measured before its body is allocated, drives it over a
  caller-owned plaintext stream, and ends it with exactly one terminal:
  completed, failed or cancelled. It keeps no timer and retries nothing.
- **A neutral vocabulary.** Prompts, messages, blocks, tools, tool choice,
  affinity and usage name no provider. Provider metadata travels as
  tagged opaque replay. Two dialects render the vocabulary: OpenAI's
  Responses and Anthropic's Messages.
- **A credential says how it is sent.** A bearer or a key, which the
  route puts where its provider takes it. A credential carries no
  account id. A call to an endpoint
  that takes none, such as a local model's server, carries none
  (`vocabulary.md`, section 1.6).
- **Per-call outcomes are not failures.** A tool call too large to keep
  arrives as an oversize block with its byte count; a tool call the
  provider cut at its output cap arrives marked as cut. The completion
  completes, and the caller tells the model.
- **Failures are typed.** A local limit names itself and its bound, an
  HTTP failure carries its status, and a timeout names the deadline that
  passed. A provider's answer is classed: exhausted, with a reset only
  where the provider gives one; refused; unsupported; unauthorized; and
  the rest. Its error code, request id and the field it names travel
  with it as bounded evidence (`vocabulary.md`, section 1.5).
- **Limits are derived.** `Limits::derive` turns a few declared
  quantities (window, output, reasoning item, tool payload, calls per
  response, conversations) into every limit of the client, per dialect,
  with checked arithmetic. Each relationship between them is checked at
  startup by name. skein ships the derivation, not a model catalogue.
- **Retained, not buffered.** Provider events are decoded as they
  stream, keeping only what the dialect reads; everything else is
  scanned and counted, never stored. Echoed instructions, tools and
  metadata cost a scan, not memory.
- **A stable prefix.** Encoding is byte-deterministic, so a conversation
  that only appends sends requests that only grow. OpenAI routes requests
  by the prompt's affinity; Anthropic's cache breakpoints sit at positions
  fixed by the prompt's shape.

## 3. Driving the stack

`client::down` accepts `Start`, `Next`, `Cancel` and `Close`.
`client::up` accepts the plaintext stream's events. `resume` drains bounded
internal routing work while `has_work` says there is runnable work. Every
entry point declares maximum output counts; the owner reserves them first.

`Next` asks for one delta, completed block or completion. It is idempotent
while one demand is outstanding. JSON/SSE progress and unknown extension
events do not consume that demand. The stack reads the next SSE event only
while demanded and after buffered output has been delivered. Transport
faults and cancellations may end the call without a `Next`. A terminal
answers an outstanding demand and ends all future reads for that call.

The connection's states and transitions are:

| State | Holds / waits for | Input and result |
|---|---|---|
| Prepared | sized request head/body | Start -> Head; Cancel/Close -> Closing |
| Head | HTTP upload and final head | room -> next upload piece; response -> Streaming or ErrorBody |
| Streaming | SSE reader, selective decoder, completion, at most one demand | Next -> consume one output or demand a read; provider terminal -> Draining; fault -> Closing |
| ErrorBody | bounded HTTP error document | bytes/end -> classified Failed; past its bound -> classified by its status and head alone, its detail saying it was cut, and Closing |
| Draining | terminal already emitted, HTTP body discarded | HTTP Done(Keep) -> Idle; Done(Close) -> Closing |
| Idle | retained HTTP client | replace with admitted call -> Prepared; Close -> Closing |
| Closing | machines stopped, actual stream still owned below | caller's settled `closed` -> Closed |
| Closed | no running machines | late inputs ignored |

`Close` on an unfinished call is cancellation. `Close` is a lifecycle
notification asking the owner to close the actual lower stream; stopping
the protocol machines does not claim that socket/TLS ownership has settled.
Cancellation's terminal comes only after the owner calls `closed`, then
`Closed` follows. A previously completed/failed call emits no second
terminal while closing. Completion may precede HTTP drainage; `Reusable`
is emitted only once drainage finishes. Reuse preserves HTTP carry-over
and checks the next call's provider and authority against the binding.

There are no hidden retries or timers. `waiting()` describes the stack's
current demand, and the owning protocol layer arms the connect,
handshake, head, idle and whole deadlines (connection.md, section 6).
It uses `abort` with `TimedOut { phase }` to report its timeout, or with
the transport's failure. Failures include evidence: unsent, possibly
sent, or a provider response received with its status. This does not
instruct a caller to retry.

## 5. Verification

Focused tests cover encoding, replay, concurrent output ordering, deltas,
limits, error classification and every lifecycle boundary. Protocol worlds
drive the real HTTP/SSE/JSON stack over a demand-checking in-memory stream,
with fragmentation, slow reads, early response, truncation, transport faults,
cancellation and reuse. Memory tests compare the counting allocator's high
water with named scenario ceilings with headroom. Fuzzy cases live in the separate nextest profile.
Archived provider transcripts retain their provenance; synthetic cases
are labelled as such. Live admission, with a key or a plan's token, is not
implied by offline tests and requires a caller-provided current credential. Worlds answer the
actual Client with the [shared scripted peer](../testing/fakes/llm.md); applications
supply its schemas, body menus, invalid inputs and scripts.

The live suite (`skein-llm-world`'s `live`) lends the public route a Codex
`auth.json`: its `OPENAI_API_KEY` where a string, refused empty, else
`tokens.access_token`. The daily live run lends an API key's, which
`codex login --with-api-key` writes and which holds `api.responses.write`.
A lent file is read once, within 64 KiB, only at mode 0600; no refusal
holds a value read. What the suite admits it decides over what its read
hands it: whether the named path is a directory, the opened entry's kind
and mode, and the bytes. Each admission and refusal, a Codex login's and
an Anthropic key file's, is checked in the focused suite over such facts
(`skein-llm-world`'s `suite`); only the providers' calls are live.

The contracts of `vocabulary.md`, section 1, and `wire.md`, section 1 add these, all in the focused suite
unless marked:

- **Affinity:** the public route carries the key in the body and in the
  `session_id` header, with no `thread-id`; no affinity sends neither; an
  endpoint header named `session_id`, `session-id` or `thread-id` is
  refused on every route; Anthropic encodes
  byte-identical requests with and without affinity. Through the fake peer, every request of one
  conversation carries the same affinity (../testing/fakes/llm.md, section 3.1).
- **Determinism:** each request of a growing conversation, encoded twice,
  gives the same bytes, and its instructions, tools and history are the
  previous request's, the moved tail marker aside, followed by what was
  appended.
- **Breakpoints:** markers at exactly the system's last block and the
  tail, never on thinking, at most four, the system marker unmoved as the
  conversation grows; the fake peer reports cache reads up to the
  previous request's tail.
- **Tool choice:** each choice's rendering per dialect; `Only` with an
  unknown, repeated or empty name refused.
- **Usage:** each field present, absent and zero, per dialect.
- **Per-call outcomes:** an oversized call next to a normal one completes
  with one oversize block and one call, the high water independent of the
  oversized call's size; a cut `tool_use` arrives as `Cut` and a history
  holding it is refused; the Responses reasoning opt-in drops an item at
  one byte over and fails without the opt-in.
- **Parts:** a Responses reasoning summary of more parts than output items,
  and a message of more content parts, complete, their deltas forwarded
  with their indexes, under limits whose output items are two.
- **Reasoning runs:** under limits whose output items are two, a Responses
  completion of more reasoning items than that before its final text,
  and one of more reasoning items than output items before one call,
  complete with one reasoning block per run; each block replays its
  items in order, byte for byte; an Anthropic completion of thinking
  blocks back to back does the same; a run whose items pass the
  completion's share of reasoning fails as `Limit`, naming it.
- **Typed failures:** each limit, at its edge and one over, names itself
  and its bound; a non-JSON error body, or JSON of another shape, carries
  its status and its own clipped text as the detail; each deadline's
  phase reaches the failure.
- **Failure classes:** each row of `wire.md`, section 1.8's table, as an HTTP error
  and, for the plan's limit, inside `response.failed`, and each of
  `wire.md`, section 1.7's types; a code
  the table names wins over its status;
  `Exhausted` carries a reset where the response gives one and `None`
  where it does not, never zero; each route's request id reaches the
  report, with or without a decodable body; the report's code, request id
  and field at their bound are kept and one over are empty; a failure
  before a head has an empty report, and a local one after it the
  request id alone; in the stream after a 200 head, a `response.failed`
  or `error` event of a code `wire.md`, section 1.8 does not name, and an Anthropic `error` event of a type
  `wire.md`, section 1.7 does not list, are `Unavailable` with that code or type in
  the report.
- **Credentials:** a key on Anthropic goes as `x-api-key` with neither an
  authorization field nor a beta, and a bearer is refused at admission; the public
  route sends a key and a token alike as a bearer, the two requests'
  bodies byte-identical; an endpoint header named `x-api-key` or
  `Authorization` is refused; a value past its bound is refused by name; a call
  without a credential sends no credential field on the public and
  Anthropic routes.
- **The public route:** over drawn prompts, no request carries a field
  of `wire.md`, section 1.8's refused list or `previous_response_id`, and every one
  carries `store:false` and `stream:true`; an explicit output cap is
  refused at admission; `response.completed` completes, `response.incomplete`
  completes with its stop, and a stream that ends without either fails
  as `Protocol`. Its cases are synthetic, and labelled so, until a
  capture of the route replaces them.
- **Derivation:** properties over drawn declarations: `derive` either
  refuses, naming a relationship, or returns limits for which every
  relationship of `wire.md`, section 1.3 holds; no product wraps.
- **Selective decoding:** a protocol world under tiny limits, a handful of
  retained tokens and a few hundred bytes of strings, decodes events that
  echo instructions, tools and attribution far larger than both, with a
  high water independent of the echo's size. A synthetic echo-heavy
  `response.completed` event, labelled so, is kept as a golden event. In the fuzzy
  suite, random extension fields never change what is decoded.

## 6. Later

- **Later pages of a provider's list** (`wire.md`, 1.9). One request reads the first
  page and keeps at most the configured row bound, 256 by default, in
  provider order. Following pagination needs a consumer's design for
  bounded requests and combined rows; jig defers it in its
  `jig/docs/design/domain/engine/README.md`, section 10.
- **Routing state.** A Responses route's turn-state token, which a
  response returns and the client replays within a turn, and its
  `reasoning.context`, which ChatGPT's backend offers to keep reasoning
  across user messages, are
  adopted on an OpenAI route that takes them only if a consumer's
  benchmarks show a material gain. Adopted, routing state is
  an opaque value the dialect returns with a completion and the caller
  hands back within the same turn, and the reasoning context is a dialect
  option chosen per model.
- **Incremental transport.** Responses over WebSocket send only the new
  items after a previous response on the same connection, which would
  decouple a request's size from its history's. It is on skein's roadmap
  as a transport of its own, with a full resend when a connection is lost.
- **How the owner runs Anthropic's live stories.** Anthropic's live
  stories need an API key that no live run lends, so the owner runs them,
  one at a time (section 5). How the owner runs them is decided later.
- **Anthropic's decoded bound.** `anthropic::decoded_bound` is OpenAI's with
  Anthropic's added (`wire.md`, 1), so it counts OpenAI's request items, which Anthropic
  never holds. Dropping them changes what a public function returns. Its one
  reader outside the crate, `skein-fake-llm-protocol`, asks only whether it
  is `Some`, so it can shrink when a consumer wants the room.
- **A client's handlers** (3): each event is one function that matches the
  state inside, where ../../foundation/programming-model.md, section 5.4 has one handler per
  cell, and `Client` keeps its machines, its `Upload` and its `outcome` beside
  `State`. They stay until a debt item names them.

## 7. Open questions

- **Responses' cache writes.** Whether `cache_write_tokens` is counted
  within `input_tokens`, as cached tokens are, is not yet seen on the
  wire. `wire.md`, section 1.8 assumes it is; a capture that shows otherwise
  changes only the dialect's arithmetic.
- **`TOKEN_BYTES`** and the dialects' smallest tool and item are set
  from captures, and checked against fresh captures as models change,
  which a consumer's benchmarks can take.
- **A bound on what the provider sends has headroom over its captures.**
  The response head's bytes and fields are the provider's and its
  proxies' to grow. Each is at least four times the largest captured,
  and never below 16 KiB and 128 fields: 16 KiB and 128 fields for Anthropic, and the floor, 16 KiB
  and 128 fields, for the public route until a capture of it sets its
  bound. Each capture must fit
  within a quarter of its bound. A fresh capture past a quarter is a
  warning to raise the bound, and is never met by a live failure first.
- **The error body's bound is a policy value,** since a body past it
  changes no class (`wire.md`, 1.2): 16 KiB for each route. That holds any
  provider's JSON error with room to spare, at a worst case the memory
  pool counts per call. Named captures of non-success answers, where
  they exist, check that each fits within a quarter of it. The failure
  detail's bound is the same kind of value where no capture settles it.
- **The report's bound is a policy value:** 128 bytes for each of the
  code, the request id and the field, on each route. Codes, request ids
  and field names are short identifiers; a capture of one past a quarter
  of it raises the bound.
- **`Only` on OpenAI.** The public Responses API expresses `Only` as an
  allowed-tools choice; whether a plan's token may use it, rather than
  being answered `subscription_sharing_unsupported_capability`, waits for
  a live test. Until then the public route renders `Auto`.
- **The plan's reset.** Whether OpenAI gives a reset with
  `subscription_sharing_usage_limit_exceeded` is not yet seen; the class
  carries one only where the response does (`wire.md`, section 1.8).
- **A key's spent credit on Anthropic.** `billing_error` is `Exhausted`
  with no reset. A spent credit answered with another type, such as an
  `invalid_request_error` naming the balance, stays `Invalid` until a
  capture shows its shape.
- **Anthropic's lookback.** The provider looks for a cached prefix only a
  bounded number of blocks before a marker. A turn that appends more
  blocks than that would miss the previous tail; a third marker on the
  previous tail would cure it within the limit of four. Whether a turn's
  calls per response can reach it decides whether the dialect adds it.
- **Anthropic's lifetime.** Five minutes until a consumer's benchmarks
  measure what a conversation loses when it pauses for longer, as one
  does while its caller waits on a long piece of work; a longer
  lifetime on the system marker alone is the candidate.
- **Calls past the declared count.** A completion with more calls than
  output items allow fails as `Limit`. Whether the calls past the count
  should instead be per-call outcomes, told as not run, waits for a
  completion that meets it.
