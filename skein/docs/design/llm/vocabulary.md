# LLM calls: vocabulary and ownership

## 1. Vocabulary and ownership

A `Prompt` holds instructions, a model, conversation messages, tool
definitions, a tool choice, reasoning effort, an affinity and an optional
output-token cap. A message holds ordered text, refusals, tool calls,
tool results and reasoning blocks; a completion may also hold the
per-call outcomes of section 1.4. Provider-owned
`Replay` metadata travels with the block it belongs to and is tagged with
its provider. `Replay` can be retained as a bounded durable envelope without
exposing provider fields to a domain. The exported seven-byte header is
additional to the raw opaque cap; `replay_bytes` and `replay_bound`
state complete receiver/transit bounds. A replay value for a different provider
is refused. Native tool calls keep the application call ID and provider item
ID in separate owned fields. Both are bounded opaque strings: pipes, quotes and backslashes
are admitted without imposing an additional identity grammar. The call ID
pairs the exact result; the item ID moves into complete replay metadata and
returns unchanged on a continued request. Neither field is joined or split.
The native answer budget charges both ID payloads, name and input. Its slot
bound uses the enlarged `Part`/`Opened` wrapper size, with active ID/kind bytes
and admitted ready payloads priced simultaneously.

Generic tool calls retain raw JSON argument text; the application validates and
translates it into its own tool types before execution. Responses history carries
the exact bounded UTF-8 argument string, including malformed JSON, so the
application can send an error result and let the model correct its call. The
Client does not interpret that feedback. Anthropic's native `tool_use.input`
is an embedded JSON object: replay refuses malformed or non-object input
rather than normalizing or discarding the original call. A call the
provider cut short never reaches history as a call (section 1.4), so this
refusal only meets a caller's own malformed history. Schemas are bounded JSON
values, independent of the application's tool vocabulary.

Every emitted replay also fits the complete serialized raw opaque cap, including
synthesized text/refusal item IDs, phase fields and tool item IDs. An oversized
native metadata value produces actual Client `Failure::Limit` before a completed
block or Completed terminal, with lower settlement still owed. An admitted value
at that exact cap fits its exported durable envelope without dropping metadata.
JSON admission measures the complete escaped metadata under the smaller document
and opaque cap, without allocating a serialized buffer just to count it. Actual
Client controls cover exact/one-over escaped text, refusal and tool metadata.
Replay metadata and reasoning have separate caps (`wire.md`, section 1.2): a large
reasoning item never widens what a text block's metadata may hold.

`Json`, `DocumentLimits` and `DocumentError` expose bounded document admission
without naming a wire dialect. `Json` is a compact document (../data/json.md,
section 5.2). `document_error` retains receiving overflow as
`Error::Limit` and malformed grammar, missing fields or wrong types as
`Error::Invalid`. It reuses the Client's classification; it starts no call,
emits no terminal and supplies no retry policy.

`Client::prepare` validates and measures the whole request before allocating
its exact body. Admission is fallible and does not touch the stream; a
refusal names its cause (admission.md). An
accepted client has one call, named by the caller's opaque token. Every
accepted call has one terminal: `Completed`, `Failed` or `Cancelled`.
Completed blocks and deltas are separate events; completion includes all
completed blocks in provider order, its stop reason and token usage.
Delta indexes are the provider's output/content indexes, not the order in
which completed blocks are delivered. A part's index within an item, a
message's content part or a reasoning summary's part, is not an output
item: parts are streamed as deltas and their bytes are within the answer,
so no count bounds them, and a reasoning summary has as many parts as the
model writes. A run of consecutive reasoning items is one block: Responses
writes a reasoning item for each segment of its reasoning, many before one
call, and Anthropic may write thinking blocks back to back. The block
holds the run's items in order, each as it came, and replays them as they
came, so a completion's blocks follow its calls and texts, not its
reasoning's segments. Each item is within the reasoning limit; the run's
items together are within the completion's share of reasoning, output
items × reasoning, which the receiving reservation already holds, and
their replay values within one block's retained tokens (`wire.md`, section 1.2). Each
tool call opened with an ID and name gives one `Delta::ToolStart` before
its `ToolArguments`, with the call ID and tool name its completed
block carries. Responses heads missing either identity field give no start;
their argument deltas remain unnamed. Non-empty initial Responses arguments
follow the start as an argument delta; Anthropic start input is a placeholder
and gives none. Tool arguments are incomplete JSON until the whole call
block arrives. A local size limit is a failure, not a
provider token-limit stop. A tool call past the input limit is the one
exception: a per-call outcome (`vocabulary.md`, section 1.4). A Responses reasoning item past
its limit is dropped instead only where the owner opted in (`wire.md`, section 1.8).

**The dialects share one vocabulary.** The crate's own `Stop`, `Usage` and
`Failure` (1.5) are what both dialects' decoders produce: no copy of them, and
no mapping between copies. What both dialects need below that, the documents'
limits, the failure classes and rate headers, the JSON value layer and the
reasoning replay's fields, sits in a private module `wire`, which names neither
dialect; `openai` and `anthropic` each re-export the names they publish, and
Anthropic's modules name nothing of `openai`.
`wire::Output<P>` is one stream of decoded events over a dialect's own `Part`.
It is this step crate's one generic item, a departure from
`skein/docs/foundation/programming-model.md`, 10.3, for this reason: every
variant but `Part` is the same in both dialects, the dialects' `Part`s are not,
and a data type with no methods hides no call, which is what the rule guards.
Whatever reads it is written per dialect, on concrete types. The Client's own
stream in `dialect` isn't a copy: its `Part` is a translated block or an error,
and its failure carries a report.

### 1.1 Affinity

```rust
pub struct Affinity {
    pub key: [u8; 16],   // opaque: the tree of conversations that share a prefix
    pub thread: u32,     // one conversation within the key; the first is 0
}
```

- **`Prompt.affinity: Option<Affinity>`** says which conversation a
  request continues, so a provider that routes by it can serve the
  conversation's prefix from one cache. It is the only cache key: no
  endpoint carries one of its own.
- **The caller owns it.** The caller supplies the key, minted at random or
  from a seeded generator, never derived from credentials, paths or
  content, and puts the same affinity, unchanged, into every prompt of a
  conversation, resumed ones included. Conversations that share a prefix, such as a run and its
  children, share the key and differ in the thread.
- **The dialect renders it** (`wire.md`, sections 1.7 and 1.8): OpenAI routes by it;
  Anthropic accepts it with no wire effect. A prompt without affinity
  renders none of it.
- **Fixed size:** it allocates nothing, and its rendering has a fixed
  length, which the request head's limit counts.

### 1.2 Tool choice

```rust
pub enum ToolChoice {
    Auto,                    // the model may call any offered tool, or none
    None,                    // the model may call no tool
    Only(Box<[Box<[u8]>]>),  // the model may call only these offered tools, or none
}
```

- **`Prompt.choice`** says which of the offered tools the model may call.
  The offered definitions (`Prompt.tools`) stay the same in every prompt
  of a conversation, whatever the choice: only the choice changes, so the
  definitions stay in the cached prefix.
- **`Only` names offered tools,** each once, at least one; admission
  refuses anything else as `Invalid`.
- **A dialect that cannot express `Only` renders `Auto`.** No dialect
  expresses it (`wire.md`, sections 1.7 and 1.8).
- **The client does not filter.** A call outside the choice arrives as any
  call does, and the caller answers it, as not run.
- **One call at a time where the caller takes one.** Where the declared
  `calls_per_response` is 1 (`wire.md`, 1.1), every dialect asks the model for at
  most one call per completion (`wire.md`, 1.7, 1.8), so no model is invited past
  what its caller declared; above 1, parallel calls are allowed. A
  completion that still makes more is the caller's to answer, as any
  call outside the choice is.

### 1.3 Usage

```rust
pub struct Usage {
    pub input: Option<u64>,        // prompt tokens neither read from nor written to a cache
    pub cache_read: Option<u64>,   // prompt tokens read from a cache
    pub cache_write: Option<u64>,  // prompt tokens written to a cache
    pub output: Option<u64>,       // completion tokens, reasoning included
    pub reasoning: Option<u64>,    // the part of output spent reasoning
}
```

- **`None` means not reported;** zero is a report of zero. A completion
  whose provider reported no usage has every field `None`.
- **The three prompt fields are disjoint.** When all three are reported,
  their sum is the prompt's size as the provider counted it.
- **Usage never fails a completion.** A dialect's arithmetic is checked.
  A report that contradicts it keeps what was reported, and the derived
  field is `None`; the completion completes. An example is a total below
  the sum of its parts. Usage is accounting: a mismatch never discards
  what the model said, and a consumer charges for a missing count as for
  an unreported one.
- **`reasoning` is part of `output`,** never added to it.
- Each dialect maps its own fields (`wire.md`, sections 1.7 and 1.8). A cumulative
  patch replaces a field only when it reports that field.

### 1.4 Per-call outcomes

- **Oversize.** A tool call whose argument text passes the dialect's
  input limit (`wire.md`, section 1.2) is a completed block of its own,
  `Block::Oversize { id, name, bytes }`: the call's ID, its name, and the
  byte count of its argument text as received. The arguments were scanned
  and counted, never stored (`wire.md`, section 1.4), so the call's size changes no
  memory bound. The completion completes, and the answer budget charges
  only the block's ID and name. The caller tells the model that the call
  was too large, by how much, and how to make it smaller.
- **Cut.** A tool call the provider cut off at its output cap is a
  completed block of its own, `Block::Cut { id, name, arguments }`: the
  argument text received so far, which need not be JSON. The completion's
  stop says why (`Stop::MaxTokens`).
- **Dropped.** A Responses reasoning item past the reasoning limit, where
  the owner opted in, is a completed block of its own, `Block::Dropped {
  bytes }`: the item's byte count as received, scanned and never stored
  (`wire.md`, section 1.8). The dialect sends nothing for it, so a history may keep
  it as it came.
- **Neither oversize nor cut is a call.** A caller that matches only
  `Block::ToolCall` never runs one. Neither can be replayed as a call: a
  history holding either is refused at admission as `Invalid`. The caller
  replaces each in its history, as its own policy decides, with a note
  and an error result for instance, so the history it sends stays
  replayable.
- **Deltas** of an oversized or cut call include its `ToolStart`, before
  its argument deltas, with the same ID and name as its outcome block.
  Arguments still go up as they come: they are for display, never a call.

### 1.5 Failures

```rust
pub enum Failure {
    Unauthorized,
    Exhausted { retry_after: Option<Duration> },  // the reset, only where the provider gives one
    RateLimited { retry_after: Duration },
    Refused,       // the provider refuses this account what was asked
    Unsupported,   // the provider will not take a field of the request
    Overloaded,
    Unavailable,
    ContextTooLong,
    Invalid,
    Limit { which: Cap, bound: u64 },   // a local limit fired: which one, and its value
    Protocol,
    Cancelled,
    TimedOut { phase: Phase },          // the owner's deadline passed: Connect, Handshake, Head, Idle or Whole
}

pub struct Report {
    pub code: Box<[u8]>,     // the provider's error code, else its error type
    pub request: Box<[u8]>,  // the provider's request id, from the response head
    pub field: Box<[u8]>,    // the field of the request the provider names
}
```

- **The provider's classes.** Each route maps its provider's answers
  onto these by the error's code first, and by the status where the code
  is absent or not one the route names (`wire.md`, sections 1.7 and 1.8):
  - `Exhausted`: the account's usage or quota is spent, a plan's usage
    limit or a key's quota. `retry_after` is the reset the response
    gives, and `None` when it gives none: the client never guesses a
    reset and never reports an unknown one as zero.
  - `RateLimited`: a short-term limit; its delay is the provider's hint,
    read as rate-limits.md says, zero when it gives none.
  - `Refused`: the provider refuses this account what was asked: not
    eligible, a scope not granted, a permission it lacks. Neither the
    same request again nor a refreshed credential changes the answer.
  - `Unsupported`: the provider will not take a field of the request,
    which the report names. It is the request's error: the same body is
    never sent again unchanged.
  - `Unauthorized`: the provider did not accept the credential.

  The client retries none of them; what each does to an account is its
  owner's policy.
- **The report is evidence.** `Event::Failed` carries a `Report` beside
  the evidence and the detail: the provider's error code, its request id
  and the field it names, each as the provider sent it and empty where it
  sent none. Each is within the route's report bound (`wire.md`, section 1.2); a
  value past it is left empty, never clipped, since a clipped code or id
  names nothing, and the detail still holds the provider's text. The
  request id is read from the response head, under the field each route
  names, so a failure after a head carries it whether or not its body
  decoded. A failure before a response head has an empty report; a
  local failure after one (a limit, a timeout, a malformed stream)
  carries the request id alone.
- **`Limit { which, bound }`** names the local limit that fired and its
  value: the request, the response head or its field count, an event's
  skipped bytes, retained bytes or tokens, nesting depth, a string, the
  answer, a reasoning item, replay metadata, the completion's items, or
  an error body. Admission's `Error::Limit` names its limit the same way.
  The failure's text detail is derived from it; a limit is never reported
  as a malformed event.
- **HTTP status.** `Evidence::Response { status }`: a response head
  arrived, with this status. Every failure after it carries it, a
  non-JSON error body's included. The other evidence is unsent, or
  possibly sent.
- **A cut exchange is `Unavailable`.** The
  connection closing or failing before the request went out, after it
  and before a response, or within a response's head or body (http.md,
  section 3.4: `Closed`, `Truncated`, `Stream`) is no answer of the
  provider's: the call fails `Unavailable`, which an owner may retry,
  with the evidence of how far it went. Its detail names the cut: closed
  before the request was sent, closed before a response, closed within
  the response, or failed below. A response that is not HTTP (its status
  line, a header, its framing, a chunk or a trailer malformed, another
  version, an upgrade) stays `Protocol`.
- **`TimedOut { phase }`** names the deadline that passed
  (connection.md, section 6). The client arms none: the owner aborts
  with it. Head and idle say the peer stalled; whole says the owner's own
  time ran out. That is the distinction an owner's retry policy needs.
- **Provider text** stays in the failure's detail, within the detail
  limit. It is data: what to show, and where, is the owner's policy.
  An error body that does not decode as the dialect's error document
  (another JSON shape, plain text, a proxy's page) is provider text
  too: its detail is the body itself, clipped to the detail limit, as a
  cut body's is. The client never puts a diagnostic of its own in its
  place; the status is in the evidence already.
- **A 403 that is not the provider's own** is `Unavailable`: its body
  did not decode as the dialect's error document (an edge's or a proxy's
  page, another shape, none, or one cut at its bound), so the provider's
  edge refused the call before the provider saw it. A 403 whose body did
  decode stays as its code or the status says (`wire.md`, 1.7, 1.8).

### 1.6 Credentials

```rust
pub enum Scheme {
    Bearer,   // an access token, or a key the provider takes as one
    Key,      // a key, sent where its provider takes keys
}

pub struct Credential {
    pub value: Box<[u8]>,       // the secret
    pub scheme: Scheme,
}
```

- **A credential is a value with its scheme, `bearer` or `key`, and no
  account id.** The caller obtains it, a key from its holder or an access
  token from a sign-in (../oauth/README.md), and hands it with each call that has
  one (connection.md, section 4.2). The client keeps it no longer than
  its call, and no `Debug` shows it. `Credential::bearer` and
  `Credential::key` make one.
- **The route puts it where its provider takes it.** OpenAI's public
  route sends either scheme as `Authorization: Bearer`, and renders the
  same request for a key as for a plan's token (`wire.md`, section 1.8). Anthropic
  takes a key in `x-api-key`, with no authorization field, and refuses a
  bearer at admission, as `Invalid`.
- **Or none.** A call's credential is optional (`Option<Credential>`),
  for an endpoint that takes none, such as a local model's server on
  loopback (connection.md, section 3). A call without one sends no
  credential field: no `Authorization` on the public route, and neither
  `x-api-key` nor `Authorization` on Anthropic, whose `anthropic-version`
  is sent. The client never makes one up: a provider that wants one
  answers `Unauthorized`.
- **Its bounds.** The value is within the bounds the owner gives when
  the component is made (connection.md,
  section 3), which the request head's measure counts; a value past its
  bound is refused at admission, by name.
- **Reserved names.** `Authorization` and `x-api-key` are credential fields,
  and `chatgpt-account-id` stays reserved: an endpoint's configured
  headers may not use them, as they may not use the framing fields.
