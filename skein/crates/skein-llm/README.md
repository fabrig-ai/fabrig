# skein-llm

A provider-neutral LLM client following Skein's step model: no async, runtime,
sockets or hidden retries. Its routes are OpenAI's public Responses (`Endpoint::openai()`)
and Anthropic's Messages. It composes
`skein-http` (HTTP/SSE) and `skein-json` over a caller-owned plaintext stream,
normally supplied by TLS.

`Prompt` contains instructions, model, conversation messages and function
tools and `ToolChoice` (`Auto`, `None`, or distinct offered names in `Only`).
Both dialects render `Only` as automatic selection; callers enforce tool policy.
Ordered `Block` values describe text, refusals, tool calls/results and
opaque reasoning. Responses expose text, reasoning and tool-argument deltas,
completed blocks, and one final `Completion` with stop reason and usage.
Arguments past the input cap arrive as `Oversize`; provider output cuts arrive
as `Cut`. Both complete normally and are refused in history until the caller
replaces them; only `ToolCall` represents an executable call.
Completed blocks preserve provider replay metadata for the next turn.

For OpenAI, use `Endpoint::openai()` with a key or a caller-obtained access
token, both sent as a bearer. The caller obtains and refreshes its credential;
the crate reads no auth file and initiates no sign-in. Responses requests use
`store:false`, `stream:true` and full history; they omit unsupported token
caps and sampling settings.

To drive one call:

1. Configure `client::Limits` with `client::valid`; check
   `largest_read`/`largest_room` against the lower stream's caps.
2. Build a `Call` with its owner token, endpoint, credential and prompt.
   `Client::prepare(call, &limits)` validates it before touching the stream.
3. Use uncapped upper and lower queues for each
   entry point. Call `client::down(..., Request::Start, ...)`, and forward its
   `stream::Down` records to the lower stream.
4. Feed lower `stream::Up` records into `client::up`. Drain `resume` while
   `has_work()` is true before delivering another input.
5. Request one data event with `Request::Next`. A delta, completed block or
   completion consumes that demand; progress/extension events do not. Stop
   asking to stop further response reads. Failures need no demand.
6. Execute application tools using the completed `ToolCall` block. Argument
   deltas are incomplete JSON. Keep the final completion's replay metadata
   when appending its assistant message to history.
7. After `Completed`, wait for `Reusable` before `next_call` on the same
   authority/provider. It preserves the HTTP machine and buffered carry-over.
8. On `Event::Close`, close the actual lower stream and call `client::closed`
   only after it settles. Cancellation ends with `Cancelled`, then `Closed`.

The caller schedules deadlines using `waiting()`, and reports them through
`abort(..., Failure::TimedOut { phase }, ...)`. The phase names connect, handshake,
head, idle or whole. There are no automatic retries; failures
include structured classification and evidence of whether the request may
have reached the provider. Limits remain fixed across all entry points and
connection reuse. Admission `Error::Limit` and active `Failure::Limit` carry
the exact `Cap` and its configured `bound`. Failure detail is derived from
that evidence; `Evidence::Response { status }` preserves the HTTP status
on every failure after a response head, including non-JSON error bodies.

For Anthropic, use `Endpoint::anthropic()` and `Credential::key(value)`.
Key acquisition and secure storage remain with the caller. The default route is
`api.anthropic.com/v1/messages`; requests send `x-api-key` and
`anthropic-version: 2023-06-01`, with neither `Authorization` nor a beta flag.
A bearer is refused at admission as `Invalid`. Version and beta headers can be
set through `Endpoint::headers`. Credential and framing headers cannot be overridden.

Anthropic uses native ordered message blocks, tool schemas and tool-result
error flags. `Prompt::max_output_tokens` sets its required `max_tokens`, defaulting
to `Limits::declared_output_tokens`; zero is invalid. The public Responses route requires this setting to be `None`.
`reasoning_effort` accepts `off` or adaptive-thinking effort `low`, `medium`,
`high`, `max`; model compatibility is the caller's choice. Anthropic accepts
`Prompt::affinity` with no effect on its wire bytes. Completed thinking and
redacted-thinking blocks retain signatures/data as Anthropic-tagged replay.
Received tool arguments remain raw; resending requires valid JSON objects and
native Anthropic identifiers. Provider replay cannot cross dialects.

The public route renders `Affinity::key` as a lowercase UUID in the body's
`prompt_cache_key`. Without affinity it sends no key. `session-id`, `thread-id`
and `chatgpt-account-id` are reserved endpoint-header names.

The low-level `openai` and `anthropic` modules expose bounded document codecs
for fakes and specialized protocol stacks. Both share `Json` and dialect limits.
The Anthropic event vocabulary follows the
[Messages streaming protocol](https://platform.claude.com/docs/en/build-with-claude/streaming).

The [design](../../docs/design/llm.md) defines lifecycle and memory contracts.
`cargo nextest run -p skein-llm -p skein-llm-world` checks codecs and the full
stack; the world crate's fuzzy profile varies fragmentation, flow control and
close races. The archived response fixtures say where they came from.

The opt-in live suite exercises `client::Client` against the public OpenAI Responses and
Anthropic Messages endpoints over certificate-verified TLS. It checks streamed
text and token usage, multi-turn replay, tool-argument streaming and tool-result
replay, HTTP connection reuse, and invalid-token failures for each provider.
There are no retries or credential refreshes. OpenAI tests use subscription
quota and require a valid, unexpired OAuth access token. Anthropic tests use an API key.

Name a Codex login and a private plaintext Anthropic key file, then run:

```sh
export SKEIN_TEST_LIVE_OPENAI_AUTH="$HOME/.codex/auth.json"
export SKEIN_TEST_LIVE_ANTHROPIC_KEY="/path/to/private/anthropic-key"
heavy cargo nextest run -p skein-llm-world --profile live-skein
```

`SKEIN_TEST_LIVE_OPENAI_AUTH` names a Codex `auth.json`, or a directory
holding one (a `CODEX_HOME`); the tests read its `.tokens.access_token` and pass it as
`Credential::bearer` to `Endpoint::openai()`. `SKEIN_TEST_LIVE_ANTHROPIC_KEY` names a mode-0600
plaintext file containing the API key. Each Anthropic call reads it once, trims
surrounding ASCII whitespace, and sends it as `Credential::key` on
`Endpoint::anthropic()`. An empty, oversized or non-private key file is refused.
The tests never print credential values or write credential files. A missing
variable, file or login field fails the live run, naming the variable, file
or field. The default and fuzzy profiles exclude the `live` binary, and its
provider tests perform no network I/O outside `NEXTEST_PROFILE=live-skein`,
including under `cargo test`; its offline tests use synthetic credential files
in a scratch directory.
The `live-skein` profile selects only this binary, runs its tests serially,
and allows twenty minutes (ten minutes before the user's doubling of 10-22,
provisional) overall with a six-minute (three-minute before the user's
doubling of 10-22, provisional) per-test limit and 60-second (30-second
before the user's doubling of 10-22, provisional) socket timeouts.

Optional `SKEIN_TEST_LIVE_OPENAI_MODEL` and `SKEIN_TEST_LIVE_ANTHROPIC_MODEL`
override `gpt-5.5` and `claude-haiku-4-5`. `SKEIN_TEST_LIVE_CA_FILE` overrides
the default Linux CA bundle at `/etc/ssl/certs/ca-certificates.crt`.
Public Responses accepts a missing `Content-Type` while validating any
explicitly supplied type.

Applications can retain `Replay::to_bytes` envelopes without owning provider
codecs. `replay_bytes` bounds the receiving envelope: its seven-byte header
is additional to `DocumentLimits::bytes`. `Prompt::output_ceiling` handles the
configured dialect's supported wire option, while local output limits remain
caller-owned. Shared independent scripts and HTTP/SSE peers are described in
[fake-llm.md](../../docs/design/fake-llm.md).

Owners may enable `client::Limits.drop_reasoning` to receive `Block::Dropped`
for Responses reasoning beyond the opaque cap. Dropped blocks emit no history item;
Anthropic thinking and other opaque Responses items retain typed limit failures.
`Client::prepare_with_reasoning_drop` selects the policy explicitly for one
call, overriding that default in either direction. Connection owners supply
`Request::Start.drop_reasoning` from the selected model's policy. Calls with
different policies can reuse the same endpoint and connection; the receiving
bounds stay unchanged. `Component::admit_with_reasoning_drop` supports the same
explicit policy when checking admission before submitting Start.

Anthropic output uses `client::Limits.declared_output_tokens` when the prompt
has no cap. A prompt cap above that declaration is refused as `Cap::Output`.
Native Anthropic request encoding and measurement take the declaration explicitly.

`Usage` exposes optional `input`, `cache_read`, `cache_write`, `output` and
`reasoning` counts. `None` is unreported; `Some(0)` is a reported zero.
Inconsistent accounting preserves the completion and leaves derived input absent.
The fake protocol's `Config.usage_fields` selects the fields it writes.

Anthropic requests encode system instructions as ordered text blocks, marking
its last block and the final message's last text, tool call or result with
`cache_control: {"type":"ephemeral"}`. Reasoning replay remains unchanged.
`anthropic::decode_request` returns `Request { prompt, marks }`; the independent
peer rejects excessive or invalid markers before dispatching a query.
