# Limits

Provisional, 2026-10-09, revised 2026-10-20. How an agent's limits are
made:

- the quantities a deployment declares;
- how every limit below them is derived, layer by layer;
- the relationships checked at startup, each by name;
- memory, as dynamic ownership, edge reservations and measured ceilings;
- what happens to one call at a limit, and the deadlines that measure
  progress;
- how a limit that fires is reported.

The derivation's skein half is skein's `llm/README.md`. The values a product
ships are its presets, in `../shell/README.md`. This document holds
the rules and the arithmetic's shape, not the values.

## 1. In one page

- **Declared, never set by hand.** A deployment declares a few
  quantities: per model, its window, its output, its largest reasoning
  item and its progress deadlines; per deployment, its concurrency, its
  tool payload, its calls per response, what a tool's output may hold,
  the jobs a session may hold, its LLM pool and its memory (section 2).
  No byte limit is configuration. A test may override a derived limit;
  nothing else may.
- **A profile is a named bundle** of declared values and policy values.
  It is step code in the service crates (README.md, section 5), so a
  profile is tested where it is made, and its values are the product's
  presets.
- **Every limit is derived, layer by layer,** with checked arithmetic:
  skein-llm's `Limits::derive` gives the LLM path's, and smith derives
  its session's, transcript's, rendering's, machine's and channel's from
  the same declarations and from the layers below (`limits-derivation.md`, section 1). An
  overflow refuses the configuration.
- **The window bounds the material, not the reverse.** A byte counts as
  at most one token. What a window holds at its opening, and what a
  compaction carries, is bounded from the model's window, so every
  window can open and close within it (`limits-derivation.md`, 1.9). The session's check
  estimates appended bytes at a measured ratio; a guard holds each
  prompt sent within the line below which a compaction always fits
  (`limits-derivation.md`, 1.9).
- **Every relationship is checked at startup, by name** (section 4). A
  violation refuses the configuration and names the relationship and the
  values that broke it. What startup can find is never found in the
  middle of a run.
- **Memory follows ownership** (section 5). Containers grow as they
  fill, and internal queues have no cap. LLM calls reserve against a
  shared pool and wait while it is full. Edge formulas must be
  representable, and fixed scenarios' measured peaks stay under ceilings;
  there is no process-wide heap sum or startup memory fit check.
- **A limit meets one call, not the conversation** (section 6). An
  oversized call is that call's problem, told to the model as one it can
  fix. Oversized rendered output is cut with a marker. A call that finds
  the pool full waits.
- **Deadlines measure progress** (section 7): connecting, the handshake,
  the response's head and the gaps between its events. The whole call is
  bounded only by the session's remaining time.
- **A limit that fires says which** (section 8): `Limit { which, bound }`
  in the failure, in the answer, in the event stream and in the
  operator's words.

## 2. Declared quantities

### 2.1 Per endpoint

Declared once for every model an endpoint serves, in the agent's
configuration (agent.md, section 4): the largest a charter may name on it,
and what any of them takes. A provider preset supplies known values;
settings override them (`../shell/README.md`).

| Quantity | Unit | What it is |
|---|---|---|
| `window` | tokens | the usable input window: the most one prompt may hold, with a completion of `output` beside it |
| `output` | tokens | the most one completion may produce, thinking included: sent where the dialect takes it, what a completion's budget reserves (../domain/budget.md), and what the window above its threshold holds for the answer a compaction carries (`limits-derivation.md`, 1.9) |
| `reasoning_item` | bytes | the largest opaque reasoning item the model returns for replay |
| `head` | duration | from the request sent to the response's first byte |
| `idle` | duration | the largest gap between two events of a streaming response |

- **The charter carries the effective values.** A host puts a model's
  `window` and `output` in each charter's model entry, at most what the
  configuration declares for its endpoint (charter.md, section 2). The agent
  derives its limits from the configuration's, the largest any charter
  may name; the session enforces the charter's.
- **A window is a prompt's, never its output's.** No request's output
  takes room in `window`: the session's check bounds the next prompt,
  and a compaction's summary is its output (../domain/session-context.md, 1.1 and
  1.2). Where a provider counts a request's output within its context, a
  model's `window` is declared as that context less `output`.
- **An oversized reasoning item fails by default.** An endpoint's
  declaration may opt into dropping it instead: the completion is kept,
  the item is left out of later requests, and the event stream says so
  (events.md, section 3.9).

### 2.2 Per deployment

| Quantity | Unit | What it is |
|---|---|---|
| `charter_models` | count | the most models one charter's LLMs name, main's among them; `run.models` is one less |
| `conversations` | count | the most conversations open at once across a run's tree: main, its sub-agents and compactions; so the most LLM calls in flight |
| `tool_payload` | bytes | the most a tool exchanges in one piece: a `read`'s largest window, a `write`'s content, the arguments of one call |
| `file_bytes` | bytes | the largest file the tools hold whole: a file `read` pages through in windows, and one `edit` loads and stores. It is at least `tool_payload`, so a file can grow past one piece: written once, then extended by edits |
| `calls_per_response` | count | the most tool calls one completion may make, and so the most result blocks the next message holds |
| `jobs` | count | the most jobs one session may hold at once: a job counts from the answer that names it `running` until an answer reports its end, or its session closes (domain/tools.md, section 5) |
| render caps | bytes, count | what a tool's output may hold (below) |
| `llm_pool` | bytes | the memory every LLM call in flight shares, sending and receiving (section 5) |
| `memory` | bytes | declared process memory, carried without a startup fit check (section 5) |

The render caps, each the default or the ceiling a tool states to the
model (domain/tools.md, section 4):

| Cap | What it bounds |
|---|---|
| `read_window` | a `read`'s window when the call asks for none; the ceiling is `tool_payload` |
| `shell_output` | a command's kept output when the call asks for none |
| `shell_head`, `shell_tail` | the ceiling on a command's kept output, and how it splits between head and tail; a check's output keeps the same tail |
| `search_hits`, `search_bytes` | a search's matches and their bytes |
| `list_entries` | a listing's entries |
| `guide` | a guide kept whole in the system text before it is cut (../domain/run-start.md, 1.3) |

### 2.3 Policy values

Values of a profile that bound no bytes, checked like any. Each has the
name `smith check` and the event stream's header print it under
(section 9); the product's values are `../shell/settings.md`'s, section
2.5.

| Name | Unit | What it is |
|---|---|---|
| `max_turns`, `max_spend`, `max_time`, `max_waiting` | count; the host's unit; duration; duration | the most a charter's budget and waiting time may ask for (../domain/budget.md) |
| `sections`, `host_tools`, `verdicts`, `items`, `fields` | count | the most of each a charter may hold, at most the charter schema's (charter.md, section 6) |
| `inbox` | count | the messages a run holds unread (../domain/run-messages.md, section 1) |
| `unacknowledged` | count | the most turns a start's window may ask to hold unacknowledged (channel.md, section 7) |
| `compaction_threshold` | fraction | the share of a window at which a session compacts (../domain/session-context.md, section 1) |
| `parallel_tools` | count | the most adjacent reads of one completion that run at once, as one batch; the reads after them wait, in call order, until every call of that batch has answered, and then form the next batch. A session uses the smaller of it and `calls_per_response` (domain/session.md, section 5). It is from 1 to 8, the most calls a session runs at once |
| `connect`, `handshake` | duration, per endpoint | connecting, and TLS's handshake (section 7) |
| `connection_keep` | duration, per endpoint | how long an idle connection stays open in a live service; never how the process ends, since the pool closes at the answer (skein's `llm/connection.md`, section 7) |
| `tool_deadline` | duration | a tool call's longest deadline: the most a command's call may ask for, and the most any tool call's deadline may be, a host tool's included (domain/tools.md, section 5) |
| `shell_timeout` | duration | a command's deadline when its call asks for none |
| `job_poll` | duration | a `job` call's wait when it asks for none: the default of `wait_seconds` (domain/tools.md, section 5) |
| `job_wait` | duration | the longest a `shell` call's yield or a `job` call's wait may be: the ceiling of `yield_seconds` and of `wait_seconds` (domain/tools.md, section 5) |
| `check_timeout` | duration | the checks' deadline where the charter names no bound of its own (../domain/run-delivery.md, 1.1; charter.md, section 2), cut at the run's end; a check that ran out of time runs once more at twice its bound (../domain/run-delivery.md, 1.1); a charter that allows a change keeps at least this much time in its reserve (../domain/budget.md, 1) |
| `group_stop` | duration | each of the three steps that stop a command's group (agent.md, section 2) |
| `close` | duration | io's close deadline (section 7) |
| `write_deadline` | duration | the most one write of the event stream may stay in flight before its destination is given up (events.md, section 5.2) |
| `wrap_up` | duration | the most time a sub-agent keeps for its final turn (domain/run.md, 5.3); a child's allowance is the smaller of this and a quarter of its effective time |
| `cancel_grace` | duration | the inline agent's grace for a cancel, before it drops a run (domain/host.md, 9.2); a host's for its spawned agent's cancel, before it terminates the tree; and an agent's own, once its host is gone, before it drops its run (domain/host.md, section 4) |
| `live_interval` | duration | the longest the agent holds live bytes before sending them as a piece (channel.md, section 7); the default is 100 ms |
| `exit_grace` | duration | a host's grace for a spawned agent to exit after its answer, above the agent's teardown bound (section 4) |
| `refresh_lead` | duration | how long before a sign-in record's access token expires the local host refreshes it: skein's `oauth/README.md` refresh lead, shared by every process on the record (`../shell/settings.md`, 2.5, and `../shell/credentials.md`, 3). A story raises it so that its run refreshes at once (`../testing-tiers.md`, 3) |

### 2.4 Fixed by the vocabulary

A few bounds are the vocabulary's own, sealed by its schemas and the same
in every deployment:

- **a delivery's diagnostic:** the last 512 bytes of the host's
  diagnostic output, with the count of bytes dropped before them
  (../domain/run-delivery.md, 1.2);
- **a run's identity:** 16 opaque bytes (../domain/run-start.md, 1.2;
  domain/host.md, section 2);
- **a credential:** a grant's value and what travels beside it, such as
  an account's id, within the bound the channel's schema seals for a
  grant (channel.md, section 6). Every source the local host lends from
  is read within it, and the request head's limit counts it at that
  bound (`limits-derivation.md`, 1.1);
- **an operator's line:** one line on standard error, a failure's
  detail clipped to its first 512 bytes at a character's boundary
  (agent.md, section 6);
- **a piece of live content:** at most 4 KiB of bytes, plus its fixed
  fields (a turn, a retry, a kind, an offset, and for an input the
  call's provider id and tool's name, each within the vocabulary's
  bound), the channel schema's (channel.md, section 4);
- **a model's effort:** a name of at most 32 bytes, the charter schema's
  (charter.md, section 2). The list of names a model accepts is its
  declaration (`../shell/settings.md`, 2.1), never the vocabulary's;
- **an account's refusal:** the refused notice's code, text of at most
  128 bytes, skein's report bound, left empty past it, never clipped
  (skein's `llm/vocabulary.md`, 1.5, and `llm/README.md`, 7); and its reason, text of at most 512
  bytes, the failure's detail cut at a character's boundary. Both are
  the channel schema's, and a failed run's refused account carries the
  code within the same bound (channel.md, section 4);
- **a host's input and answer:** at most 262,144 bytes each, the most a
  deployment's `tool_payload` can make them: a start with more is refused,
  whatever the channel's own larger ceiling (domain/run.md, section 5).

## 4. Checked at startup

Each relationship has a name. A configuration that breaks one is
refused before the loop starts, and the refusal names the relationship,
the limits on each side and their values (agent.md, section 6;
`smith check` in `../shell/README.md`).

skein checks its own as it derives and composes, each refused by its
name (skein's `llm/wire.md`, section 1.3): one call's input within the
answer; the output items and strings its rules give; one largest call's
request and receiving reservation within `llm_pool`; at least as many
connections as `conversations`; each demand within its cap; and
Anthropic's output within what its route accepts. smith checks the rest:

| Name | Relationship |
|---|---|
| `derived` | every derived limit fits its type; the first that overflows is named |
| `fresh-window` | for each declared endpoint, a new window at its largest (smith's own part, the host's part, the task as its instruction and one offer, a summary of `output` tokens, and one offer of held-back messages) and one `output` fit within the window at the compaction threshold, counting a byte as a token (`limits-derivation.md`, 1.9). The parts are derived so that this holds whenever smith's own part fits the room, for every shipped preset among them (`../shell/settings.md`, 2.1); the check refuses a model whose room smith's own part fills |
| `host-part-holds-the-role` | for each declared endpoint, the host's part (`limits-derivation.md`, 1.9) holds the task role as written, its bytes as the shell's code holds them, not its 4 KiB cap (`../shell/settings.md`, 2.6), beside one guide at `guide`, the shell's own start's one directory, so that the product's own default never cuts its guide or makes a start too large for the entrance; a start with more directories shares the rest (`limits-derivation.md`, 1.9) |
| `compaction-headroom` | for each declared endpoint, the window above its threshold holds one `output` (that of the completion whose results a compaction carries), the summary instruction and a cut result's marker for each of `calls_per_response` results: the line is at or above the threshold, and the compaction the check or the guard calls for always fits, its results cut to what is left; what it leaves is the spare (`limits-derivation.md`, 1.9; ../domain/session-context.md, 1.1) |
| `output-in-window` | each model's `output` is less than its `window` |
| `window-first` | a window's history reaches the compaction threshold before its message, block or byte bound: the line at `TOKEN_BYTES` bytes a token, plus one completion's whole reserve and one batch's results, within the history's bytes (rule 4 of `limits-derivation.md`, section 1). It holds by construction for `limits-derivation.md`, 1.2's derivation, and guards it |
| `calls-in-transcript` | a message's blocks, derived from `calls_per_response`, are at least a completion's blocks and within the transcript schema's ceiling |
| `resume-holds-window` | the resume limit holds the most a session holds in one window |
| `turn-in-transcript` | the largest turn fits the transcript schema's ceilings |
| `turn-in-channel` | the largest turn, start and message fit the channel's frame ceilings |
| `render-in-result` | every tool's outcome at its caps, a host's answer, a message and a sub-agent's result (its answer at `run.answer_bytes`, its cut's words and its usage line, `limits-derivation.md`, 1.10) render uncut within one rendered result's bound: the cut is a safeguard, never what a cap relies on (llm.md, section 5). A bound held below the derived render bound by any other limit breaks it |
| `read-in-payload` | `read_window` is at most `tool_payload` |
| `payload-in-file` | `tool_payload` is at most `file_bytes` |
| `machine-holds-tools` | the machine's output, file, scan, search and path limits hold the tools', and its processes hold the listings, searches, commands and checks that run at once, and `jobs` for each of `conversations` beside them (`limits-derivation.md`, 1.5) |
| `environment` | the configured environment's names are valid and distinct |
| `request-head` | a request's head holds a credential at its bound (2.4), the affinity's headers and the endpoint's configured headers, none of which is a reserved name |
| `ceilings-in-vocabulary` | the policy ceilings fit the charter's and the channel's schemas |
| `deadlines` | every deadline is positive, `shell_timeout` ≤ `tool_deadline`, so no command's deadline, asked for or not, passes what the session allows one tool call, and `job_poll` ≤ `job_wait` ≤ `tool_deadline`, so no yield or wait does either, the default included |
| `cancel-over-close` | `cancel_grace` is at least twice io's `close`: a cancelled call settles within one `close` against a peer that keeps its connection open (skein's `runtime/shell.md`, section 12), so a grace no longer than `close` drops a run whose cancel would have settled; the second `close` is the margin |

Two more hold across processes, and the side that knows both checks
them:

| Name | Who checks | Relationship |
|---|---|---|
| `grace-over-teardown` | a host that writes its agent's configuration | `exit_grace` exceeds the agent's teardown bound (section 7), with a margin, as skein's `runtime/shell.md`, section 12 has a supervisor derive its grace (agent.md, section 6; domain/host.md, section 4) |
| `watchdog-over-silence` | the host kit | its no-progress deadline exceeds the longest deadline the run may be silent for between reports: io's short operations and its own steps (domain/host.md, section 4) |

What a run asks for is checked at its entrance, not at startup
(domain/run.md, section 4): an endpoint the configuration does not declare;
each charter model entry's `window` and `output` against its endpoint's
declaration, its room against the one the limits were derived for, and
its window above the threshold against one `output` and
`run.compaction_material`, so that its spare is not negative (`limits-derivation.md`, 1.9),
refused as a window or an output above its declaration is (charter.md,
section 6); the charter's text against the host's part (its directories' guides
take their shares of what it leaves, `limits-derivation.md`, 1.9); the budget against the ceilings; and the
reserve against the budget.

## 5. Memory

- **Dynamic ownership.** Slabs and queues grow as they fill; internal
  queues have no cap. Owned bytes arrive at their final length and leave
  with their owner: a compacted window's history once the new window
  opens, a job's output once its answer carries it, a closed
  conversation's history and calls. The declared `memory` is carried
  without a startup fit check.
- **Formulas size edges and admitted values** (programming-model.md,
  6.3). A codec's workspace, a completion's receiving bound, a rendered
  result and a delivery's ownership each have their own bound, with
  checked arithmetic. `valid(limits)` checks representability and
  compatibility; it does not sum the heap. Limits whose edge bounds
  overflow are invalid.
- **Use is measured.** The counting allocator holds each fixed
  scenario's total peak under its named ceiling with headroom, throughout
  construction, iterations and teardown (testing.md, section 6). The
  total includes caller-held outputs and transient allocations; no
  doubled formula or separate held/peak allowance substitutes for it.
  A ceiling moves by hand when a real change passes it, never with a
  type's size. Benchmarks measure peak resident memory beside time and
  cost (../benchmarks/codex-parity.md, 1).
- **Admission to the LLM pool.** Every LLM call in flight draws on
  `llm_pool`, which skein's connection component keeps (skein's
  `llm/connection.md`, section 7). At admission a call reserves its
  request, measured exactly before encoding, and its derived receiving
  bound (skein's `llm/wire.md`, 1.2), releasing both at its terminal. A call
  that fits alone waits for room, in arrival order and within its
  deadline (`limits.md`, section 6).
- **One process owns each part once.** The product in one process
  (`../shell/README.md`, section 5) has one loop, one ring and no host
  channel. Its local domain owns the inline agent; their effects, io,
  chat store, terminal and delivery are built directly, without building
  an agent service and stripping it. Scenario measurements cover this
  composition as one process, without a synthetic agent or channel sum.

## 6. One call at a limit

| What | Outcome |
|---|---|
| a tool call whose arguments pass one call's input | the call arrives as an oversize block with its byte count; the completion completes; the model is told the size, the bound, and how to make it fit (llm.md, section 4) |
| a rendered result past its bound | cut at a character's boundary with an explicit marker saying how many bytes were left out; the request goes (llm.md, section 5) |
| a sub-agent's answer past `answer_bytes` | kept to it, cut at a character's boundary, with words saying how many bytes were left out and that a new sub-agent, asked narrower, reads them: the child has closed, so nothing continues it (domain/run.md, 5.3) |
| a call that finds the pool's connections or memory at capacity | it waits, in order of arrival, bounded by `conversations`, within its deadline (llm.md, section 6) |
| a `sub_agent` call while its run holds `conversations` conversations, or while the agent's tables of conversations or calls are full | refused as too many, or answered busy; no child opens and the parent goes on, however many calls its message asked (domain/run.md, 5.3) |
| a reasoning item past `reasoning_item` | a typed limit failure, unless the model's declaration drops it (2.1) |
| a tool call cut off by the output limit | marked by skein; the session keeps a note and an error result in its place (llm.md, section 4) |
| history that reaches the window's threshold | a compaction, not a failure (../domain/session-context.md, section 1) |
| results owed to a compaction past what its headroom leaves | cut with their markers in the compaction's request only (`limits-derivation.md`, 1.9) |
| a completion's later batch past what is left of the history | each call runs with its share of what is left as its cap, its result saying what it left out; the next completion is the compaction (../domain/session-context.md, 1.1) |
| a completion's batch past what is left below the line | each call runs with its share of what is left as its cap, its result saying what it left out; the next completion is sent when the guard's exact measure fits (../domain/session-context.md, 1.1) |
| messages queued past one offer | they wait, in order, for the next offer (`limits-derivation.md`, 1.9) |
| a provider's "context too long" | one compaction; a second in the same window fails the session |
| a completion past `calls_per_response` | a typed limit failure of the completion; the tools' descriptions state the bound |
| a `shell` call with `yield_seconds` while its session holds `jobs` jobs | answered as an invalid call that says at most `jobs` may run and how to stop one; the command is not run (domain/tools.md, section 5) |
| a yield or a wait asked past `job_wait` | clamped to it, and the outcome says so |
| what a job writes between two answers, past its kept output | its head and tail kept, and the bytes between them counted and said (`limits-derivation.md`, 1.5) |
| a message, a host's answer, a charter or a start past its bound | refused at its entrance, naming the bound (../domain/run.md, section 4, and ../domain/run-messages.md, section 1) |

## 7. Deadlines

| Deadline | Runs | Its value | On expiry |
|---|---|---|---|
| connect | from the connection's start to its socket connected | the endpoint's | transient |
| handshake | from connected to TLS ready | the endpoint's | transient |
| head | from the request sent to the response's first byte | the model's `head` | transient |
| idle | between two events of the response | the model's `idle` | transient |
| whole | the call | the session's remaining time | not retried: the session's time is up |

- **Progress, not duration.** A response that keeps streaming is never
  cut by a fixed bound. There is no ceiling per attempt.
- **`TimedOut` carries its phase:** `Connect`, `Handshake`, `Head`,
  `Idle` or `Whole` (skein's `llm/connection.md`). The session retries by
  phase (domain/session.md, section 4).
- **A call waiting for the pool** is within its whole bound; past it, it
  fails unsent.
- **Tools and checks** have the domain's deadlines: a host tool's from
  the charter, at most `tool_deadline`; a command's from its call, or
  `shell_timeout`, clamped to `tool_deadline`; the checks' is the
  charter's bound, else `check_timeout`, cut at the run's end, and twice
  that for a check's one retry after it ran out of time (../domain/run-delivery.md,
  1.1; domain/tools.md, section 5).
- **A job keeps its command's deadline,** counted from the command's
  start, whatever its yield. A `shell` call's yield and a `job` call's
  wait bound how long that call waits before it answers, never how long
  the job runs, and are at most `job_wait`.
- **A cancel of jobs settles within the group stop.** A session's close,
  like whatever takes the lease (../domain/run.md, section 5.3, and ../domain/run-delivery.md, section 1.1),
  stops the groups of the jobs it cancels together, each in the three
  steps of agent.md, section 2, so it waits at most three `group_stop`s
  for them, however many there are.
- **The end of the process** is bounded only by io's close deadlines
  (skein's `runtime/shell.md`, section 12). After its answer the agent closes its
  components together, its LLM connections, its machine's roots and its
  channel, and then its event stream, after `session.ended` (agent.md,
  section 6): two `close` deadlines in turn, the stream's
  `write_deadline` for its last records, and the moment its cancels take
  to settle, which io's close deadlines already bound and which adds
  nothing to them. That is its **teardown bound**, from which a host
  derives `exit_grace` (section 4).
- **An agent whose host is gone** exits within **`orphan_exit`** of the
  loss: the link's `cancel_grace` plus its `exit_grace`, as
  `host_limits` derives both from the agent's own configuration, the
  same derivation its host makes (domain/host.md, section 4). It is what
  the host would have given the agent from a cancel to its exit: the
  cancel's grace to answer, then the grace to exit. It is derived, never
  configured, its sum checked, an overflow refused as `derived` naming
  `orphan_exit`, and it is an effective limit under that name (section
  9). The agent drops its run at the cancel's grace at the latest; what
  the drop ends settles within one `close`, as a machine's close stops a
  job (above), and the teardown follows within its bound: together
  `exit_grace` as `host_limits` derives it, the teardown bound plus one
  `close`. The silence that marks a host gone is timed by the link's
  `no_progress`, the figure the host times the run by, derived the same
  way and an effective limit under that name.
- **Jobs add nothing to the teardown bound.** A session's close cancels
  its running jobs and waits for them to settle (domain/session.md,
  section 5), and the run answers
  only once its sessions have closed and what was in flight has settled
  (domain/run.md, section 4), so the machine holds a running job at its
  close only after a run dropped without an answer. Its close stops each
  such job with io's close of the job's child, the group stop's third
  step, within one `close` (agent.md, section 2).

## 8. A limit that fires

- **Typed.** A limit that fires is `Limit { which, bound }`: which limit,
  and its value. The LLM path's kinds are skein's (skein's `llm/README.md`);
  smith's are its own layers' (a session's history, a decoded call, a
  message). An HTTP failure carries its status.
- **Carried whole.** The session keeps it in its failure, the run in its
  answer (domain/run.md, section 9), the event stream in
  `response.completed` and `run.completed` (events.md, section 3), and
  the operator sees it in words: on standard error (agent.md, section
  6), and in the local host's account of the failure (host.md, section
  5.1).
- **Refusals name what they compared.** A startup refusal names the
  relationship (section 4). An entrance refusal names the bound and the
  value asked.

## 9. Effective limits

The declared values, the policy values and every derived limit, each
under its name in its unit, are the **effective limits**. The declared
and policy values are named as section 2 names them; a derived limit is
named by its layer and its field in that layer's `Limits`, such as
`llm.request` or `session.history_bytes`. A value per endpoint or per
model adds them by their configured names, never by a number:
`llm.request[endpoint=openai]`, `declared.head[endpoint=openai,model=gpt-6-luna]`,
so a reader joins them to the names in `conversation.opened`. Sizes are
in bytes or tokens as their names say; durations in whole milliseconds. `smith check` prints them (`../shell/commands.md`, 4), and the
event stream's header carries them (events.md, section 3.2). Their names
and units are part of the event vocabulary (events.md, section 6).
`domain.content_edge_records` caps the root's optional content capture edge
in records, with overflow dropped and counted. Internal run, session,
tools and inline lifecycle facts have no configured allowance and drain
losslessly by count.

## 10. What the domain is owed, and what skein owes

- **The domains are owed** their limits as numbers, derived before the
  loop, with every relationship between layers already checked, so no
  domain meets another layer's limit in the middle of a run.
- **skein owes:**
  - `Limits::derive` for the LLM path, with its documented factors and
    its own relationships checked (skein's `llm/wire.md`, section 1);
  - selective, streamed decoding of provider events (skein's `data/json.md`
    and `http.md`);
  - an oversized call as a block of its own, typed `Limit` kinds and
    HTTP statuses, and a reasoning bound apart from replay's metadata
    (skein's `llm/README.md`);
  - waiting for a connection or for room in the memory pool, the expired
    phase in `TimedOut`, and a pool sized from the owner's concurrency
    (skein's `llm/connection.md`, sections 4.1, 6 and 7);
  - encoded, decoded and workspace bounds for edges, and allocation
    prices charged at admission (programming-model.md, 6.3).

## 11. The world

- **Derivation properties,** in each deriving crate's step tests: for
  declared values drawn at random within their types, derivation either
  refuses, naming one relationship, or gives limits for which every
  relationship of section 4 holds. Nothing panics or wraps.
- **Each relationship's refusal:** one configuration per row of section
  4 that breaks it, refused with its name.
- **Every profile:** each shipped profile, with every preset model
  declared, derives, passes every check, `fresh-window` and
  `compaction-headroom` included; its edge bounds are representable.
- **A window's room,** in the session's world with small windows: a
  window just opened at its largest, its first completion made; more
  messages queued than one offer holds, the rest offered at the next
  yield; a compaction whose owed results pass its headroom, cut and
  fitting; a charter too large for the host's part, refused.
- **No pinned values.** Tests assert properties of the derivation, never
  that a limit equals a number. Worlds declare tiny values
  (testing-strategy.md, section 3), so every limit is reached in a world.
- **The LLM path,** in the protocol world `tests/protocol-llm` (llm.md,
  section 10): calls waiting at a full pool, then served; an oversized
  call beside an ordinary one; a reasoning item at its bound and one past
  it; a response streaming past any fixed bound; a stall timed out
  `Idle` and retried; the session's time ending once, without a retry.
- **Rendering:** a result past its bound cut with its marker; random
  outcomes at random caps rendered within the bound.
- **Live content:** a stream longer than the held bytes against a
  channel with no room, dropped past the bound and counted; pieces never
  larger than the piece bound; a piece sent at the interval's end with
  fewer bytes than the bound (channel.md, section 10).
- **Memory:** skein's counting allocator under fixed scenarios' measured
  ceilings, with the LLM pool full and every conversation open, each
  holding `jobs` jobs at their kept output's cap, in the agent's simulated
  world (agent.md, section 8). Domain output is uncapped; worlds test the
  dispatch orders and edge credit, never domain output flow control.
- **Memory returns** (section 5), with the same allocator, in the
  agent's simulated world:
  - **a compaction:** the live heap falls, once the new window opens, by
    at least the payload bytes of the compacted history's blocks less
    those of the new window's opening, cells left out, since a window's
    message list keeps its capacity;
  - **repeated uses:** across a story that compacts, reads jobs to
    their end, and opens and closes conversations, several times each,
    the live heap at the same point of each repetition is no larger than
    at the first.
- **Jobs and batches,** in the session's world with small values: a
  `shell` call with a yield refused while `jobs` jobs are held; a yield and a
  wait clamped to `job_wait`; a wait asked for none, `job_poll`; more
  adjacent reads than `parallel_tools`,
  the rest run as the next batch once the first has answered.

## 13. Open questions

- **A cut sub-agent answer's rest** (`limits-derivation.md`, 1.10; section 6): retaining it for a
  continuation is domain/run.md, section 13's question. Default: keep
  only the capped answer and tell its asker to ask a new child, narrower;
  no additional retained-answer room is declared.
- **Calls past `calls_per_response`:** whether skein marks the calls
  beyond the bound as calls of their own, answered as not run, so the
  completion completes, as an oversized call's does.
- **A ceiling per attempt:** whether a model's declaration may add one,
  for a provider that streams without end.
- **A host answer's own bound:** whether host tools' answers get a
  quantity of their own rather than `tool_payload`. Messages take
  theirs from one offer (`limits-derivation.md`, 1.9).
- **A window's halves:** the room left after smith's own part goes half
  to the host's text and half to messages. Proposed as a first split;
  the history probe measures whether a host's text or a person's
  messages want more of it; what it gives the presets is
  `../shell/settings.md`'s, 2.1.
- **The translation limits' names:** whether `tool_bytes` and
  `history_bytes` are named under `translation`, their layer's field:
  a change to the event vocabulary (events.md, section 6), which recorded
  runs hold.
- **The rest of the codecs' ceilings:** whether the channel's and the
  transcript's decoding also draws on a shared pool, as LLM calls do, or
  its decoded bound at its derived limits stays small enough.
