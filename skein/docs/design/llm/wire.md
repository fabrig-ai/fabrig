# LLM calls: bounds and wire format

## 1. Bounds and wire format

Every limit of the client is derived from declared quantities (sections 1.1 to 1.3
). Limits are per route: `Limits` holds a set for OpenAI's
public route (`openai`) and for Anthropic's (`anthropic`), as their
heads, events, replay and output caps differ. Startup checks
ensure SSE demands fit HTTP's read cap and that each piece size fits the
transport's (connection.md, section 7). JSON is decoded as each
event's data streams, selectively (`wire.md`, section 1.4). HTTP read/room maxima are
exported for the stream owner to check.
`client::valid` checks the edge bounds and derived capacities, including
HTTP, SSE and JSON demands fitting their lower streams, tagged projections,
error documents and tokenizer lengths. Internal handoff queues grow with
admitted work. There is no client heap sum; worlds check measured peaks
under scenario ceilings. The edge formulas name what they bound:
`client::encoder_bound` and `wire`'s `encoder_bound` bound request
scratch;
`openai::decoded_bound` and `anthropic::decoded_bound` bound
retained decoded output, and `anthropic::decoder_bound`
bounds its response decoder. Anthropic's is OpenAI's with its own decoder
and marks added, so it is written where both dialects are known, in
`dialect`, and `anthropic` re-exports it. `replay_bound` bounds the raw/envelope
handoff. The neutral document's internal `document_bound` and OpenAI's
response `decoder_bound` and `decoder_fixed_bound` size their retained
tables and scratch.

### 1.1 Declared quantities

```rust
pub struct Declared {
    pub window: u32,              // prompt tokens the largest model accepts
    pub output: u32,              // completion tokens, reasoning included
    pub reasoning_item: u32,      // bytes of one opaque reasoning item as the provider sends it
    pub tool_payload: u32,        // bytes of one tool call's arguments once unescaped
    pub calls_per_response: u32,  // tool calls in one completion
    pub conversations: u32,       // calls outstanding at once, across the owner
    pub tools: u32,               // tools one request offers, at most
    pub history_items: u32,       // messages and blocks one request carries, at most
}
```

- **Per model:** window, output and reasoning item. An endpoint serves the
  models its owner sends to it, and its declaration takes the largest of
  each. Head and idle, also per model, are deadlines the owner gives per
  call (connection.md, section 6), not inputs to byte limits.
- **Per deployment:** tool payload, calls per response and conversations.
- **Counts, from the owner:** the tools one request offers and the history
  items it carries. The owner knows both from what it holds in memory:
  an agent's, for instance, are its offered tools, and its history's
  bytes over the smallest block it keeps.
- **skein ships no catalogue.** A model's window and output are the
  provider's facts and change too often for a kit. The application
  declares them, from its own presets and settings.
- **Configuration declares quantities, never byte limits.** `Limits`
  stays a plain value, so a test may build one by hand or lower any limit
  after deriving it (../../foundation/testing-strategy.md, section 3).

### 1.2 Derived limits

`Limits::derive(&Declared) -> Result<Limits, Violation>` computes every
limit, for every route, with checked arithmetic: `Limits` holds one set
per route, and a connection takes its endpoint's. An overflow, or a relationship of section
1.3 that does not hold, is a `Violation` naming it. Two factors are the
derivation's own:

- **`ESCAPE` = 6:** the most bytes JSON escaping makes of one byte
  (`\u00XX`).
- **`TOKEN_BYTES` = 8:** the bytes of text one token is allowed to stand
  for. Text averages about four bytes a token; the factor is generous
  because memory is bounded by admission (connection.md, section 7),
  not by these caps.

| Limit | Rule | What it bounds |
|---|---|---|
| request | window × `TOKEN_BYTES` × `ESCAPE` | one encoded request; a prompt within its window meets it only if its text passes `TOKEN_BYTES` a token with every byte escaped |
| answer | output × `TOKEN_BYTES` | what the model writes in one completion: text, refusals, call IDs, names and arguments |
| input | tool payload × `ESCAPE` | one call's argument text as received; past it, an oversize block (`vocabulary.md`, 1.4) |
| reasoning | reasoning item | one reasoning item: Responses' encrypted content, Anthropic's thinking and signature |
| metadata | the dialect's constant | one block's other replay metadata: item IDs, phase |
| output items | 2 × calls per response + 2 | blocks in one completion: each call with a reasoning block before it, a text and a refusal. A reasoning block is a run of consecutive reasoning items, and a part's index within an item is not an item (`vocabulary.md`, section 1) |
| strings | the larger of answer and reasoning | the longest single string the decoder keeps |
| retained tokens | the dialect's fixed fields + reasoning | tokens kept from one event: an opaque value takes at most a token per byte |
| retained text | the largest of answer, input and reasoning, + the dialect's constant for its short kept fields | bytes of text kept from one event, unescaped: the collector's `text` |
| receiving | answer + output items × reasoning + one event's retained values | what one call holds of its answer at once: its memory reservation, beside its measured request |
| skip | request + `ESCAPE` × receiving | bytes of one event the dialect scans without keeping: an echo of the request and of the whole answer, escaped once more |
| tools | the declared `tools` | tools in one request |
| history items | the declared `history_items` | messages and blocks in one request |

- **Counts are memory, so they are declared.** skein's tables allocate
  their capacity when made, so a count is memory. A count derived as the
  request over the smallest encoded item gives counts that only a
  request of empty items reaches, a million history items for one
  272,000-token window, and allocates them all. The owner declares them
  instead. A request within its byte limit that would carry more is
  refused by name, as any count past its bound is.
- **Replay values are charged at their total.** What a completion keeps
  for replay, each block's item IDs, phase and opaque values, is held at
  its own length and charged at the total its blocks keep, within the
  receiving reservation that already holds the answer and the reasoning
  share. It is never reserved as output items × one block's largest:
  output items each holding a reasoning item's worst case is memory no
  completion holds, and each call in flight would reserve it.
- **The encoder makes no copies.** A request is written from the values
  its owner holds, borrowed, straight into the request's bytes, which
  the memory pool reserves:
  - tool schemas, replayed reasoning values and other compact documents
    are serialized by reference, never cloned into a compact copy;
  - JSON text a request carries, such as a call's arguments that a
    dialect sends as an object, is checked by a streaming pass of the
    tokenizer, which writes each token in the compact serializer's
    spelling as it reads it. That means no insignificant whitespace, and
    escapes as the serializer writes them. So the bytes sent, and what
    is admitted against the request bound, are exactly those of parsing
    and serializing it, without a compact document. What fails the pass
    is handled as the dialect already handles invalid arguments.

  So the encoder's scratch is its fixed state and a stack bounded by the
  dialect's nesting depth, held once by the component. Dense JSON, many
  tokens in few bytes, costs no memory, and no count of JSON tokens is
  admitted or refused. Sizing the scratch from the encoded request's
  bytes, a token per byte, charged 577 MB for one 272,000-token window.
  A per-item token constant would undercount dense JSON, so neither is
  used.
- **Sent text has no cap of its own.** A string the caller sends is
  bounded by the request as a whole; what the caller renders into it is
  the caller's to bound.
- **The SSE reader's** line and event limits are the skip cap, counted
  and never held (http.md, section 4.2). Its chunk, and HTTP's read and
  send, follow the transport (connection.md, section 7).
- **Heads.** The request head's limit is measured per endpoint
  (connection.md, section 3). The response head and its field count
  are the route's constants, sized for its provider's header set with
  headroom (`README.md`, section 7): the provider and every proxy in front of it add
  headers without notice.
- **The report** (`vocabulary.md`, section 1.5) holds three values, each within the
  route's constant, 128 bytes (`README.md`, section 7). The failure event holds them,
  so the worst case counts them once per call.
- **The error body** and the failure detail are the route's constants.
  An error body is evidence for the failure's detail and its finer class
  (a context too long, a retry hint in the body). It never decides
  whether a failure is retryable: the status does. A body past its bound
  is kept up to the bound and not parsed. The call fails with the class
  its status and head give, as a non-JSON body's does, its detail says
  that the body was cut, and the connection closes rather than drain the
  rest. So a large proxy page in front of a 503 stays `Unavailable`, not
  a local `Limit`. `Cap::ErrorBody` still names a zero bound refused at
  admission.

### 1.3 Relationships checked at startup

Each is checked when the limits are derived or composed, and a violation
refuses the configuration and names the relationship:

- **input ≤ answer:** a call within the tool payload is never cut by the
  answer limit;
- **output items ≥ calls per response,** and strings ≥ input and
  reasoning, which the rules above give by construction and the check
  keeps true when a test lowers one;
- **receiving + request ≤ memory pool:** one largest call fits the pool
  (connection.md, section 7);
- **connections ≥ conversations** (connection.md, section 7);
- **demands fit their caps:** the SSE reader's chunk within HTTP's read,
  the tokenizer's largest demand within the reader's data face, HTTP's
  largest read and room within the transport's. These depend on the
  transport's piece sizes, so they are checked where the connection
  component is built (connection.md, section 7), not in `derive`;
- **a model's output** is at most what its route accepts as `max_tokens`,
  per model, checked where an endpoint of that dialect is composed: a
  declaration for one dialect is never refused for another's route.

### 1.4 Selective, streamed decoding

- **Each event's data streams** from the SSE reader (http.md, section 4)
  into a JSON tokenizer and the selective collector (../data/json.md, section
  5.1). The dialect's filter names the paths it reads. Every other value
  is scanned, checked as JSON, counted against the skip cap, and never
  stored.
- **Responses keeps** an event's `type`, its indexes, its item ID and delta;
  of an item, its type, IDs, name, status, phase, content parts' text,
  arguments (under the input limit) and encrypted reasoning (under the
  reasoning limit), and an unknown item whole, as opaque replay under the
  reasoning limit; of `response`, its status, the known usage fields,
  error and incomplete details. It never keeps `response.output`,
  `instructions`, `tools` or per-item usage attribution, which echo what
  it already has or does not read.
- **Of an error,** each dialect keeps its type, code, message, the field
  it names (`param`) and the reset it gives, and of the response head
  the route's request id and limit fields; the report (`vocabulary.md`, section 1.5) is
  copied from them.
- **Anthropic keeps** each event type's known fields; a thinking block
  whole, its extension fields included, under the reasoning limit, so
  that its replay is exact; tool input under the input limit; and an
  unknown native block whole. Its events are told apart by their SSE
  event name, so its filter's root is chosen per event, and need not be
  `Tagged`.
- **Items and blocks are tagged** (../data/json.md, 5.1): a Responses output item by
  its `type`, an Anthropic content block by its `type`. A known type
  keeps its fields; an unknown one is kept whole under the reasoning cap,
  whatever the order of its fields. The input, reasoning and string caps
  are the filter's named caps, set from the derived limits.
- **Retained values are bounded** by the retained-token and string limits.
  A value past its limit is `Limit { which }`, except a tool call's
  arguments, which become an oversize block, and a Responses reasoning
  item under the opt-in of section 1.8, which becomes a dropped block.
- **Memory is one event's retained values,** the tokenizer's text buffer
  and depth stack, and the reader's chunk: never an event's wire bytes.
  The size of what a provider echoes changes no bound.
- **Arguments stay text.** Responses' arguments are a string holding JSON;
  the decoder keeps the string's text and never parses it. The caller
  does.

### 1.5 Byte-deterministic encoding

- **The same input encodes to the same bytes.** The same prompt,
  endpoint, credential and limits always give the same request head and
  body: fixed key order, compact JSON, no clock, no randomness, no
  iteration over unordered state. Per-call headers are a function of the
  prompt alone.
- **Replay goes back unchanged.** The dialect writes provider metadata,
  reasoning and arguments from what it kept, the same way every time, and
  never rewrites their content.
- **So a prefix that is kept is sent again unchanged.** When a caller
  keeps the model, endpoint, affinity, instructions and tools of a
  conversation fixed and only appends to its history, each request's
  instructions, tools and history items are the previous request's,
  followed by what was appended. Anthropic's breakpoints are a pure
  function of the prompt's shape (section 1.7): only the tail marker
  moves, and a marker is not part of the content the provider caches.

### 1.7 The Anthropic dialect

Anthropic sends `stream:true`, native `messages`, tool `input_schema`, and
`max_tokens`. Adaptive thinking maps explicit effort values.
Signed/redacted thinking is preserved
as native opaque replay, tool names/IDs remain unchanged, and tool results
retain their native `is_error`. The bounded decoder verifies message/block
ordering, delta types, cumulative usage patches and completion at `message_stop`.

- **A key.** `Endpoint::anthropic()` is `api.anthropic.com` and
  `/v1/messages`. A key goes in `x-api-key`, with no `Authorization`
  field, and `anthropic-version` goes with it (`2023-06-01` unless the
  endpoint's headers set it). No beta flag is sent unless the endpoint's
  headers name one.
- **`max_tokens` comes from the declared output.** It is the prompt's
  output cap, which the caller sets from its model's declared output, or
  lower; a prompt without one sends the endpoint's declared output. There
  is no fixed default. A cap above the declared output is refused at
  admission.
- **Affinity** is accepted and has no wire effect: a prompt encodes to the
  same bytes with and without it.
- **Breakpoints.** `system` is a block array, and its last block carries
  `cache_control: {"type":"ephemeral"}`. Anthropic orders its cache as
  tools, then system, then messages, so this marker caches the tools with
  the system text. A second marker, the tail, goes on the last block of
  the last message that may carry one: text, tool use or tool result,
  never thinking or redacted thinking. No `ttl` is written, so each entry
  lives the provider's default of five minutes. There are never more
  than four markers; these two leave room. Empty instructions have no
  system marker, and a last message without an eligible block has no
  tail.
- **The markers move as the conversation grows.** The system marker stays
  where it is; the tail moves to the new end, so each request reads up to
  the point the previous one wrote.
- **Tool choice.** `Auto` sends no `tool_choice`, which is the provider's
  default; `None` sends `{"type":"none"}`; `Only` renders `Auto`
  (`vocabulary.md`, section 1.2): Anthropic's `tool` and `any` force a call, which `Only`
  does not mean, and do not combine with thinking. Where
  `calls_per_response` is 1, `Auto` and `Only` send
  `{"type":"auto","disable_parallel_tool_use":true}` instead (`vocabulary.md`, section 1.2
  ), a deployment's constant, so the cache holds. A change of choice
  keeps the cached tools and system text but not the cached messages.
- **Usage.** `input_tokens` is input, `cache_read_input_tokens` cache
  read, `cache_creation_input_tokens` cache write and `output_tokens`
  output. Reasoning is `None`: the provider counts thinking within output
  and does not report it apart.
- **Cut calls.** A `tool_use` whose input is unfinished when the message
  stops at `max_tokens` arrives as `Block::Cut`, with the partial input
  text. It is never parsed into an object, so it cannot make a history
  unreplayable.
- **Thinking past the reasoning limit** is always `Limit`: Anthropic's
  thinking is replayed whole or not at all, so the Responses opt-in does
  not apply.
- **Thinking is adaptive, always.** A prompt's effort, one of `low`,
  `medium`, `high`, `xhigh` and `max`, sends `thinking:
  {"type":"adaptive"}` and `output_config: {"effort": EFFORT}`; a prompt
  with no effort sends neither, the provider's default, for a model that
  declares no efforts. No effort turns thinking off: `off` is refused at
  admission as `none` is, so no request carries `{"type":"disabled"}`,
  which current Claude models (Opus 5.5, Sonnet 5.5 and Fable 5.1) refuse
  at every effort. A caller whose model declares efforts always sends
  one, since a model's own default may sit below the caller's: Opus 5.5's
  is `medium`.
- **No sampling settings.** No request carries `temperature`, `top_p` or
  `top_k`: a prompt has none to give, and current Claude models refuse
  them beside thinking. With tool choice's (above), never `any` or a
  named tool, these are the shapes current Claude models refuse, and a
  focused test of the encoder holds each one absent.
- **History only grows.** An Anthropic account made on or after
  2026-08-31 refuses a request whose earlier turns were edited while it
  replays their thinking. The dialect writes each earlier turn's
  thinking as it was given, whole and in order, and its markers move
  without touching a block's content; its caller appends turns and never
  edits one it replays, and smith's own compaction opens its new window
  with no thinking (smith's `domain/session.md`, 8.2).
- **Failures, by the error's type before the status.**
  `authentication_error` is `Unauthorized`; `permission_error` is
  `Refused`; `rate_limit_error` is `RateLimited`; `billing_error` is `Exhausted`
  with no reset (`README.md`, section 7); `overloaded_error` is `Overloaded`;
  `api_error` is `Unavailable`; a context too long, an
  `invalid_request_error` whose message says `prompt is too long`, or a
  `request_too_large`, is `ContextTooLong`; any other
  `invalid_request_error` is `Invalid`. A type it does not name falls
  to the status, as on the public route (1.8). An `error` event in the
  stream, after a head that said 200, of a type this list does not name
  is `Unavailable`, its type kept in the report (`vocabulary.md`, section 1.5). Anthropic
  names no field, so its report's field is empty, and a field it will
  not take is `Invalid`. The request id is the head's `request-id`.

Bodies are uploaded in bounded pieces using HTTP room.
Responses must be identity-encoded SSE on success; non-success responses
are parsed as bounded JSON errors. Provider-defined retry hints are data
in structured failures, never automatic retries.

### 1.8 The public Responses route

OpenAI's public Responses API, `Provider::OpenAi`: `Endpoint::openai()` is
`api.openai.com` and `/v1/responses`. It is called with an API key, which
pays at API prices, or with the access token of a sign-in that pays with
the person's plan (../oauth/README.md). Both go as `Authorization: Bearer`, and the
request is the same for both: one rendering serves a key and a plan.

Requests are measured JSON: `store:false` and `stream:true` on every
request, the whole context in `input` (never `previous_response_id`), the
caller's instructions in `instructions`, client-side function tools, and
encrypted reasoning included for replay. A request never carries a field
a plan's route refuses: `background`, `conversation`,
`max_output_tokens`, `max_tool_calls`, `metadata`, `moderation`,
`multi_agent`, `prompt`, `prompt_cache_retention`, `safety_identifier`,
`temperature`, `top_logprobs`, `top_p`, `truncation` or `user`. So
admission refuses an explicit output-token cap, and the local output
bounds still hold.

- **Response media type.** The public route accepts a missing
  `Content-Type` and validates SSE events through their terminal. An explicit
  type must be `text/event-stream` (case-insensitive, parameters ignored);
  duplicate types are refused. This follows the provider dialect regardless
  of credential kind or endpoint override. A refusal names the media type
  within 64 bytes, or says missing, duplicate, invalid or too long, and
  carries neither parameters nor body text.
- **Affinity.** The key renders as 36 bytes: lowercase hexadecimal in the
  8-4-4-4-12 form of a UUID, in the body as `prompt_cache_key` and in the
  call's `session_id` header. The header is what keeps a conversation's
  calls on its cached prefix: without it, a plan's route spread one run's
  calls across machines and cached under a fifth of its input, against
  95% with it (measured 11-09). The thread is not rendered: conversations
  that share a key share its cache routing. No `thread-id` header is
  sent. `session_id`, `session-id` and `thread-id` stay reserved names,
  which an endpoint's configured headers may not use. A prompt without
  affinity sends neither key nor header, and model-list reads carry no
  `session_id`.
- **Tool choice.** `Auto` sends `"auto"`, `None` sends `"none"`, and
  `Only` sends `"auto"` (`vocabulary.md`, section 1.2). `parallel_tool_calls` is false
  where `calls_per_response` is 1, and true above it (`vocabulary.md`, section 1.2).
- **Usage.** `input_tokens_details.cached_tokens` is cache read;
  `input_tokens_details.cache_write_tokens`, when present, is cache write;
  `input_tokens` less the parts that are reported is input, and `None`
  when `input_tokens` is below their sum (`vocabulary.md`, section 1: usage never fails a
  completion). A part the wire does not carry stays `None` and is not
  separated: its tokens, if any, stay within input. Nothing is invented
  for it; `output_tokens` is output;
  `output_tokens_details.reasoning_tokens` is reasoning. A field absent
  from the wire is `None`.
- **Cut calls.** A response that ends incomplete for its output limit
  with a function call unfinished delivers that call as `Block::Cut`.
- **Reasoning past its limit** is a `Limit` failure naming the reasoning
  limit. An owner may opt in to dropping it instead: the item is scanned,
  never stored, and `Block::Dropped { bytes }` takes its place in the
  completion. The dialect sends nothing for a dropped block, so later
  requests go without the item. The route keeps no state (`store:false`),
  so a conversation continues with less of its reasoning, not wrongly.
- **Success is `response.completed`,** which completes with
  `Stop::EndTurn`. `response.incomplete` completes with the stop it
  names, `MaxTokens` or `Refusal`. `response.failed`, or an `error` event,
  fails as its error's code says, as an HTTP error does; a stream that
  ends without one of them fails as `Protocol`.
- **A compressed body.** An endpoint may say that
  it takes a request body in gzip (`Content-Encoding: gzip`, RFC 9110,
  8.4.1). The client then measures and renders the body as before,
  compresses it whole with skein-http's gzip encoder (http.md, section
  3.1) and sends the compressed bytes with their length. Every bound
  stays on the body as rendered; the compressed body is never larger
  than its worst case, which the encoder states from the rendered length.
  The compressed copy needs room at that worst case beside the rendered
  body while it is made, so the client compresses a body only while the
  two together stay within the request bound; a larger body, past about
  half the bound, is sent as rendered. One call then never holds more
  than a body at the bound, whatever its endpoint's coding, and the
  largest call's reservation is the same on every route (connection.md,
  section 7).
  `Endpoint::openai()` does not,
  until a capture shows the public endpoint takes it. A local model's
  server never does.
- **The request id** is the head's `x-request-id`.
- **Failures, by code before status.** The error's `code`, else its
  `type`, decides where the table names it. Otherwise the status does:
  401 is `Unauthorized`; 403 `Invalid` where its body decoded, else
  `Unavailable` (`vocabulary.md`, 1.5); 404 and 422 `Invalid`; 400 and 413
  `ContextTooLong` where the error says so, else `Invalid`; 429
  `RateLimited`; a 5xx `Unavailable`, or `Overloaded` where it is 529 or
  its error's code, else type, is `overloaded_error`. In the stream,
  after a head that said 200, no status applies: a `response.failed` or
  `error` event whose code, else type, this section does not name is `Unavailable`, the code kept in the
  report (`vocabulary.md`, section 1.5). A well-formed failure event is the provider's
  answer, never `Protocol`.

| Code (status) | Failure |
|---|---|
| `subscription_sharing_usage_limit_exceeded` (429, or in `response.failed`) | `Exhausted`, its reset only where the response gives one: the error's `resets_at` or `resets_in_seconds`, or `retry-after` |
| `insufficient_quota` (429) | `Exhausted`, with no reset: a key's quota is spent |
| `rate_limit_exceeded` (429, or in `response.failed` or `error`) | `RateLimited`, its delay as rate-limits.md reads it |
| `subscription_sharing_user_not_eligible`, `chatpass_v2_scope_not_authorized`, `chatpass_v2_invalid_authorization_context` (403) | `Refused` |
| `subscription_sharing_usage_unavailable`, `subscription_sharing_user_unavailable` (503) | `Unavailable` |
| `subscription_sharing_unsupported_capability` (400) | `Unsupported`, the report's field from `error.param` |
| `subscription_sharing_route_not_supported` (403) | `Invalid`: the method or the endpoint is wrong |
| `subscription_sharing_invalid_user`, `invalid_api_key` (401) | `Unauthorized` |
| `context_length_exceeded` (400) | `ContextTooLong` |

- **A plan's request is never paid another way.** The client sends the
  request it is given with the credential it is given, and never moves a
  call from a plan's token to a key, or back, whatever it fails with.

### 1.9 A provider's list

`skein_llm::list` says what a route's list of models looks like. It holds no
connection (`connection.md`, 4.3), and is pure.

- **The request:** `GET /v1/models` on the route, its credential sent as a
  call's is (`vocabulary.md`, 1.6). Anthropic's takes `?limit=1000`, and carries its version
  header as a call does. It is one request: a page past the first isn't read.
- **What it gives:** rows of `Listed`, in the provider's order: a `name`, and,
  where the provider says them, a `shown` name, a `window`, an `output` and the
  `efforts`, low to high.

  | Route | The rows | Each row |
  |---|---|---|
  | Responses, a plan's token | `models` | `slug`, `display_name`; a row whose `visibility` isn't `list` is left out (jig's `domain/providers.md`, 2.4) |
  | Responses, a key | `data` | `id` |
  | Anthropic | `data` | `id`, `display_name`, `max_input_tokens`, `max_tokens`; an effort, of `low`, `medium`, `high`, `xhigh` and `max` in that order, where `capabilities.effort.<effort>.supported` is true |

  Responses tells the two forms apart by which member the document has, and a
  document with neither is unreadable. A list's body is read for what it says
  and nothing else: an unknown member is passed over, a row with no name or one
  past 128 bytes is left out, and so is a field past its bound, with the row kept.
- **Its bounds,** `list::Limits`: the body, 1 MiB by default, where a longer one
  is unreadable; the rows kept, 256 by default, the provider's first; a shown
  name of 64 bytes and 8 efforts of 16 bytes each.
- **A number** past `u32`, or of 0, is left out, as a field past its bound is.
