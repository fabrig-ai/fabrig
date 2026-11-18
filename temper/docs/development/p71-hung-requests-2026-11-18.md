# P71: is a hung model request cut?

2026-11-18. What each provider path the agents use does with a model
request that hangs, read from the code, and what that says of P71: on
the development deployment, every agent stopped for no progress since
the redeploy of 2026-11-17 19:48Z was gpt-6.1-sol on OpenAI's sign-in,
seven kills on four tasks, one task four times. In each, the run sent a
tool's result back and the model's next reply never came, no first
token and no byte, for 2 to 37 minutes before the kill.

No code on these paths changed between 2026-11-16 18:33Z and this
report, so the deployment ran what is read here.

## 1. What each path does

| Path | Backend | Head and idle |
|---|---|---|
| OpenAI's sign-in | smith's direct backend, skein's LLM client, the Responses dialect at `api.openai.com/v1/responses`, a bearer | cut |
| OpenAI's API | the same, a key | cut |
| Anthropic's API | the direct backend, the Messages dialect on the `anthropic-api` endpoint, a key | cut |
| Anthropic's sign-in | Claude Code, smith's program backend | cut on the program's stream |

- **The figures reach every call.** The worker writes each endpoint's
  `head_ms` and `idle_ms` into the agent's configuration
  (`temper/crates/temper-worker-shell/src/agent/figures.rs`,
  `configuration`); smith reads them
  (`smith/crates/smith-agent-document/src/config/endpoints.rs`,
  `build_limits`), the domain selects them for the conversation's
  endpoint (`smith/crates/smith-domain/src/config.rs`, `Config::progress`),
  and the session gives them with every completion
  (`smith/crates/smith-domain-session/src/session.rs`, `complete`). The
  direct component passes both to skein's connection as that call's
  deadlines (`smith/crates/smith-protocol-llm/src/component.rs`,
  `Component::start`). Neither is ever absent: profile validation refuses
  zero. Stock: `head` 60 s; `idle` 300 s for OpenAI, 120 s for Anthropic
  (`temper/crates/temper-engine-service-stock/src/stock/agents.rs`).
- **Every request alike.** The first request after a tool's result
  (`finish_tools`, then `ask`, then `complete`, in `session.rs`), a
  compaction's, a sub-agent's and a resumed run's first request (the
  restored history opens through `smith/crates/smith-domain/src/route.rs`,
  `open`, then the same `ask`) take the same deadlines. A resumed run
  takes the figures of the worker that started it.
- **How the cut works** (skein's `llm/connection.md`, 6):
  `head` is armed at each piece of the request sent and runs to the
  response's HTTP head (`skein/crates/skein-llm-connection/src/call.rs`,
  `Connection::route`; `skein/crates/skein-llm-connection/src/deadlines.rs`,
  `Table::sent`); `idle` runs from the head and is re-armed only by an
  event carrying the model's output, never by a ping, a keepalive,
  `response.created` or `response.in_progress` (`Table::activity`;
  `skein/crates/skein-llm/src/client.rs`, `sse_event`). A request that
  gets no byte is cut at `head`; one with a head and no output, or output
  then silence, at `idle`. The failure is *timed out* with its phase.
- **Retried within the session's bounds**
  (`smith/crates/smith-domain-session/src/session/retry.rs`, `class`,
  `decision`, `backoff`): `Head` and `Idle` are transient, retried 3
  times (4 attempts) after 0.1 to 0.8 s of jittered backoff, the stock
  of `smith/crates/smith-domain-session/src/limits.rs`. Past the last, the
  session ends failed, timed out; the run answers as its model's
  failure, and the worker records `RunFailure::Model` with
  `ModelFailureKind::TimedOut`
  (`temper/crates/temper-worker-domain/src/translate.rs`, `run_failure`,
  `model_failure_kind`), never `NoProgress`. At stock that is at most
  4 × 60 s for a request that gets no byte, and 4 × (60 + 300) s for
  OpenAI's worst stall.
- **Claude Code** (`smith/crates/smith-protocol-program/src/backend/current.rs`;
  `conversation.rs`, `Backend::resume`): `head` runs from the request
  to the program's first stream record, `idle` between records; each
  ends the completion timed out with its phase, and the session restarts
  it as above
  (`smith/crates/smith-protocol-program/src/backend/recovery.rs`,
  `restartable`). A request hung
  inside Claude Code is cut when the program's stream goes quiet.

## 2. Why no hung completion is stopped for no progress

An issued completion is a long operation to the host from its issue to
its terminal, its span reaching the session's whole deadline
(`smith/crates/smith-domain/src/operation.rs`, `begin_completion`,
`completion_done`), and the watchdog is armed at
`max(progress, open spans' end) + no_progress`
(`smith/crates/smith-host-domain/src/long.rs`, `Long::stretch`;
`smith/crates/smith-host-domain/src/domain.rs`, `follow`). Each attempt's
terminal and each retry also report facts, which are progress. So while
a direct completion is in flight the watchdog cannot fire, whatever
the no-progress bound is; a request hung past its deadlines is cut and
retried, and fails as the model's.

## 3. What this explains of P71

None of the seven kills. Each was stopped for no progress, so at each
the host held no open completion: the run was not waiting on an issued
request, or the host never heard of it. The paths that can wait before
or between requests with no long operation, read but not shown to stall:

- a retry's backoff, whose floor is a provider's `Retry-After`,
  uncapped by `backoff_max` (`retry.rs`, `backoff`); this needs a
  response, so it fits "no byte" poorly;
- the agent service's queues before a completion's long operation
  reaches the host: the event sink's and the channel's credit
  (`smith/crates/smith-agent-service/src/service/down.rs`,
  `domain_down`);
- a completion held at the run's gate, for its turns' credit or while
  the run winds down (`smith/crates/smith-domain-run/src/run/gate.rs`,
  `completion_held`);
- a long operation the channel refuses to send, which the service
  records and goes on from
  (`smith/crates/smith-agent-service/src/service/link.rs`, `request`).

What settles it: for each kill, whether the worker heard a completion's
`Long` with no `LongDone` before the stop, and whether the agent's event
stream shows the request sent (its `llm.request` and first byte). With no
`Long`, the request was never issued and the stall is before it; with
one, the host lost it.

## 4. The tests that pin it

- smith's protocol world: a request answered nothing, cut at `head`; one
  stalled after its first output, cut at `idle`; each retried and then
  failed timed out; a restored session's first request cut the same way
  (smith's `protocol/llm.md`, 10).
- temper's real loop: on a worker alone, a request held after a tool's
  result, cut, retried and answered; held on every attempt, the run
  failed as its model's timeout, no agent stopped for no progress
  (`temper/docs/design/testing/real-loop.md`).
