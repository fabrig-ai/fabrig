# LLM calls

Provisional, 2026-10-03, revised 2026-10-22. Where temper's LLM calls
are made, and what temper adds around them: the host tools its engine
serves, whose declarations, decoding and rendering are temper's; and the
charter and a run's result, which `temper-engine-smith` translates. The
calls are smith's, made by agent processes on the worker, over skein's
LLM client or through a program. The LLM providers, their endpoints and
models are jig's configuration on the engine and smith's on the worker
(jig's `protocol/llm.md`, sections 4 to 6); LLM accounts are jig's, made
from the client (credentials.md, section 3). What is open is listed in
section 7.

## 1. In one page

- **temper's engine makes no LLM call.** Its runs are on workers, jig's
  third shape (jig's `domain/hosts/README.md`, section 3), each in an agent
  process: `temper-box internal agent`, smith's agent shell over skein's LLM
  client (section 2).
- **LLM providers, endpoints and models are configuration,** jig's `llm`
  member on the engine and smith's agent configuration on the worker, by
  the same names, checked at start (section 3). **Accounts are not:**
  jig's, made from the client and kept by jig (credentials.md, section
  3).
- **Two backends, one agent process.** smith's agent runs a task's loop
  itself over skein's LLM client, the direct backend, for a signed-in
  source, a key or none; or adapts a program that runs it, the program
  backend, for a program's source: Claude Code, on its own login
  (smith's `protocol/programs.md`, section 2). temper's host tools, the
  charter and the result are the same either way (sections 4 and 5).
- **Host tools are temper's:** the root's `Tool` vocabulary, declared,
  decoded and answered by `temper-engine-smith`, which the engine's
  routing calls after the fleet has fenced each call (section 4).
  Nothing is parsed in a domain.
- **The charter and a run's result are temper's translation:**
  `temper-engine-smith` writes smith's charter from the root's
  assignment, and reads a run's result into the root's ending of its
  task (section 5).
- **One translation.** The engine's protocol layer is the only place where
  temper's charter, tools and messages become smith's, and smith's calls
  become the engine's (`../domain/agent/README.md`, section 1).
- **jig's charter translation, temper's host tools.** `temper-engine-smith`
  uses `jig-protocol-charter` for the charter, and keeps the root's own
  `Tool` vocabulary rather than `jig-protocol-tools` (sections 4, 5 and 7).

## 2. Where LLM calls are made

```
engine   no LLM call: accounts and grants, host tools, charters, results
worker   no LLM call: grants put in smith's form for smith's host half;
         programs' logins checked by smith's host half
agent    every LLM call: smith's agent shell, over skein's LLM client
         (the direct backend) or through a program (the program backend)
```

- **The agent process** is the worker's agent's program, the box's
  `temper-box internal agent AGENT.json` or a deployment's `temper-agent
  AGENT.json` (binaries.md, 3), spawned by the worker's protocol layer for
  each run
  (configuration/README.md, section 4; jig's `protocol/service.md`, section 6).
  It reads smith's agent configuration, dials the endpoints it declares,
  and takes its run's grants from smith's host half (smith's
  `protocol/host.md`, section 2).
- **What a call does is smith's:** its session retries a failed
  completion, and its run holds the budget (jig's `protocol/llm.md`,
  sections 1 and 3).
- **The program backend** runs the program as the agent process's
  child, with the source's configuration directory (smith's
  `protocol/programs.md`, section 5): the host tools reach it through
  the agent, as the only tools server it is given, so the engine decides
  each host tool's call as it does a direct run's; and each call of the
  program's built-in tools is asked of smith's policy before it runs, on
  the run's workspace.
- **Limited, refused and unauthorized** come back from the agent as
  notices that reach jig's accounts child, fenced by attempt
  (credentials.md, section 3; jig's `domain/accounts.md`, 3.5).
- **Agents in a host's own process** (`smith-inline-agent`), in the
  engine or on a worker, are not used (section 7).

## 3. LLM providers, endpoints and models

- **On the engine, jig's `engine.llm`** (jig's `protocol/configuration.md`,
  section 3; jig's `protocol/llm.md`, sections 4 and 5): the LLM
  providers, each with its endpoints, models and the sources it offers.
  The engine uses the models, their prices and what each is good at,
  which its charters and the brief's catalogue carry, and the endpoints'
  names, which its charters name; its grants name an account's source.
  It dials no LLM endpoint; its only exchanges for LLM accounts are its
  accounts driver's with an LLM provider's sign-in, token and
  revocation endpoints (credentials.md, section 3).
- **On the worker, smith's agent configuration** (smith's
  `protocol/agent.md`, section 4; configuration/README.md, section 4): the
  endpoints the agents dial, the models they serve, and the programs
  they may run, each with its programs directory. The worker's routing
  decodes each charter against these endpoints (jig's
  `protocol/channel/README.md`, 5.1).
- **The same names on both.** `temper-box serve` refuses a start whose
  engine names an endpoint, a model or a program the agent's
  configuration does not declare the same way (configuration/README.md, section
  4), so no run's charter fails for a name.
- **Which LLM providers:** OpenAI, with keys and ChatGPT's sign-in, in
  the owned example (setup/init.md, 1), Anthropic, and any other smith's dialects speak over skein's LLM
  client, such as a local model's server, whose source is *none*. Claude
  Code is the one program, through smith's program backend. temper
  keeps no provider's dialect, no program's adapter and no fake provider
  of its own: the tiers use skein's fake LLM peer and smith's fake
  `claude`, and the live tier a real model (`../testing/process-tier.md` and `../testing/live-tier.md`
  ).

## 4. Host tools

- **The vocabulary is the root's `Tool`:** the core's tools and the
  forge's, as `../domain/agent/tools.md` lists them. `temper-engine-smith`'s
  `tools` declares each with its JSON Schema, its description, its
  effect (a read or a write) and the charter's call timeout, and the
  charter carries the declarations (`llm.md`, section 5). Each of the forge's reads
  is a tool of its own (`read_items` to `read_permission`), decoded as
  `Tool::ReadForge`, naming the read.
- **A forge subscription selects an adoption in the task's project.**
  `subscribe_forge`'s `topic.repository` accepts `owner/name` or the
  numeric `{forge, repository}` pair. Omitted or empty names select the
  project's sole readable adoption, as the read tools do; no adoption or
  an ambiguous choice returns the readable names to choose from. A
  numeric pair must also be readable in that project. A task's packed
  resource tag is not a repository ID (`../domain/forge/adoption.md`).
- **Subscription admission names the failed check.** Refusals identify
  the task, repository pair and topic: an unadopted or unreadable pair,
  a duplicate topic (with its existing subscription number), malformed
  branch segments, oversized paths or branch, exhausted forge or task
  subscription room, or exhausted per-task resource names. A bounded
  refusal names the bound and limit. A duplicate keeps the existing
  interest; changing it requires unsubscribing its number first. These
  checks precede a backend subscription. Settled named calls retain
  their rendered feedback for replay (`../domain/routing.md`, 2).
- **A task read refused as unknown says what to read instead.** jig
  refuses `read_task` as `unknown` for a task it has no row of, an offset
  outside the text, and a `result` no ended task kept, a running one's
  among them (jig's `domain/engine/executors.md`, 3). The words say each, that a
  task's result reaches its requester as a `result` message when it
  ends, and that its specification, `field` left out, shows a running
  task's state, holds and questions. The other refusals keep their words.
- **Decoded in the routing** (jig's `protocol/channel/README.md`, 5.3), in the
  step that routes the call, keeping no state:
  1. the fleet fences the call, and the root hands it out as
     `HostRequest::Decode { to, task, attempt, call }`;
  2. the routing decodes the call's name with smith's channel codec
     (`smith_channel::CallName`), whose activation must be the attempt,
     and its input with `temper_engine_smith::call`;
  3. it hands the typed call back as `WorkerInput::DecodedHostCall { to,
     body }`.
- **A call that does not decode** is answered at once as an error that
  names what is wrong (its `temper_engine_smith::Problem`: malformed,
  too large, a member missing, of the wrong type or out of range, each
  naming the member by its name, `repository` or `number`, an
  unknown tool), and decides nothing: the routing renders the error as
  smith's text and hands it to the root as `WorkerInput::RenderedHostCall
  { to, answer }`, with the answer marked an error, and the root relays
  it to the run with no commit and no record (`../domain/routing.md`, 2);
  the run may ask again with a corrected input under a new name.
- **Answered in the routing:** once a decided call's commit is durable,
  the root hands its answer out (`HostDelivery::Render`); the routing
  renders it with `temper_engine_smith::answer_bounded`, as smith's text with
  whether it is an error, and hands it back as
  `WorkerInput::RenderedHostCall { to, answer }`. An answer that cannot
  be rendered becomes a bounded error, so no decided call is left
  unanswered. A job log's suffix is fitted after its metadata and cut
  marker, with byte escapes priced, to `core.call_answer_bytes` less the
  settled tool name; the configured bound is never raised.
- **Not `jig-protocol-tools`.** temper's root takes its own `Tool`
  vocabulary, not the core's tool events, so `temper-engine-smith` stays
  (section 7).

## 5. The charter and a run's result

- **The charter.** As the routing encodes an assignment,
  `temper_engine_smith::charter` writes smith's charter from the root's
  `Assignment` (`../domain/agent/charter.md`: the instructions, the
  brief's sections, the conventions, the host tools, the result contract,
  the budget and its reserve, the models and their prices), and smith's
  codec encodes it (`smith_protocol_channel::encode_charter`) against the
  engine's endpoints. The charter carries the models jig's claim chose
  for the run, the task's LLM choice among those its charter allows,
  with an account usable for each (jig's `domain/accounts.md`, 4.3); the
  run's grants carry one credential, or one program source's name, per
  LLM provider it uses (jig's `protocol/llm.md`, section 6). It crosses
  as smith's charter bytes in jig's assignment, and the worker's
  routing decodes it against the agent's configuration as the agent
  starts (jig's `protocol/channel/README.md`, 5.1). A resumed run's charter
  resumes when its policy allows and the task has kept turns.
- **Turns** cross and are kept as smith's bytes, encoded on the worker;
  the core reads only what jig's records carry beside them (jig's
  `protocol/channel/README.md`, 5.1).
- **A run's result.** The worker's routing encodes smith's result before
  the hub keeps the answer. Once the fleet has fenced an ended answer,
  the root hands its result out as `WorkerRequest::ReadResult { payload,
  outcome }`; the routing decodes smith's result
  (`smith_protocol_channel::decode_declared`), translates it with
  `temper_engine_smith::result` into the root's ending of the task, and
  hands it back as `WorkerInput::ResultRead { payload, end }`. A change's
  resource comes from what the worker and the forge settled, never from
  the LLM's fields, and a change with none is invalid. A result that
  does not decode or translate fails its attempt as invalid (jig's
  `protocol/channel/README.md`, 5.1).
- **Charter policy is jig's.** `temper-engine-smith` supplies the rendered
  brief and host-tool declarations to `jig-protocol-charter::assignment`,
  sets the conventions and workspace delivery fact, then calls its
  `charter` translation (jig's `domain/hosts/charters.md`). A repository's
  delivery fact is set only after its push destination survives the root's
  authority and hold checks (`../domain/forge/branches.md`, 1). This grants a
  Finite report's delivery with the policy's `modify`, without changing its
  result contract. The tool vocabulary and result translation remain
  temper's.

## 6. Testing

Within the suites' budgets (`../testing/README.md`, section 2):

- **Step tests of `temper-engine-smith`:** each tool's least input
  decoded and a wrong shape refused; each `Problem`; each answer rendered
  as bounded text; the charter from an assignment, resuming only when its
  policy allows and turns are kept; each result's translation, a change
  with and without its settled resource.
- **Step tests of the routing,** in `temper-service`: a call decoded into
  the root's `Tool`; a call of another attempt, and each `Problem`,
  answered as an error; a decided answer rendered; a result that does not
  decode failing its attempt as invalid.
- **The engine's domain world:** a call that does not decode answered at
  once, with no commit and no record.
- **Through bytes:** smith's agent processes over skein's fake LLM,
  calling host tools through bytes, in the one-process world
  (`../testing/README.md`, 2.5); the process and live tiers
  (`../testing/process-tier.md` and `../testing/live-tier.md`).
- **The program backend,** in the real loop: a run on a program's source
  through smith's fake `claude`, calling host tools that the engine
  decides as a direct run's, its result read the same way; a chat, an
  Open run, replying to two of its person's messages, one program
  process each, parking between them and resuming by its session, since
  the real loop's charters park at once; two replies on one process are
  smith's agent-process world's to show (smith's
  `protocol/programs.md`, 5.9 and 8); and a limit the fake program
  reports making the source limited (`../testing/real-loop.md`).
- **Notices:** a limited, refused and unauthorized notice from a run,
  each scripted on skein's fake LLM peer, reaching the accounts child
  fenced by its attempt, and one of a fenced attempt dropped
  (`../testing/real-loop.md`).
- **No token across a domain boundary** (credentials.md, section 7).

## 7. Open questions

- **jig's host tools** (`jig-protocol-tools`). temper's root takes its own
  `Tool` vocabulary while sharing jig's charter policy (section 5).
  Default: keep that vocabulary and its translation in `temper-engine-smith`
  until the root's tools are next reconciled with jig's.
- **A run's live text** on a conversation's page: smith's channel
  carries it at the host's ask, but jig's assignment carries no ask and
  its workers' facts no content, so agent processes give no text until
  jig's transport is built (jig's `protocol/channel/README.md`, 5.4) and
  temper's assignment then asks for it and its worker passes the pieces
  on. Default: a run is shown by its activity, and its words as each
  turn commits.
- **Agents in the worker's process,** `smith-inline-agent`, for one
  person's machine, with no containment and no process per run (jig's
  `protocol/service.md`, section 6). Default: agent processes.
- **A run's idle bound** (smith's `protocol/limits.md`, 2.1 and 7): the request
  `parallel-faults` of 2026-11-06 saw 19 runs fail `timed out during response
  idle (HTTP 200)` in 73 minutes, on two tasks with requests of 456 to 801 KB,
  where a run at 823 KB worked, and reading doesn't settle why. The bound is per
  model in a deployment's `agent.json`: `temper-box init`'s template has 30 s and
  smith's local presets 120 s and 300 s, so a raise in `smith/` might not reach a
  deployment. It arms at the response's head and only an event carrying output
  re-arms it, so a 200 with no event, a stream of pings and a stall partway read
  alike. About 230 s a run fits four tries at 30 s and not at 120 s, by arithmetic
  alone, and says nothing of the run at 823 KB. What settles it: that deployment's
  `idle`, and a rerun with `trace` set, comparing each attempt's
  `response.completed` on those tasks and that run: `duration_ms` less
  `first_byte_ms` near `idle` says nothing re-armed it, and `largest_gap_ms` how
  quiet a stream ran. A new figure, the user's, then comes from those gaps and a
  sourced figure for a comparable client. Default: the figures as they stand.
