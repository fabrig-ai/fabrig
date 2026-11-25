# Limits: the derivation

## 1. The derivation

Each layer derives its own `Limits` in its own crate, from the
declarations and from the limits of the layers it depends on, with
checked arithmetic: `None` on overflow, which refuses the configuration
naming the limit it was deriving. A derivation documents its factors
(a byte per token, an escape's expansion, a frame's overhead) where it
uses them. The service composes the chain once at startup and checks the
relationships between layers (`limits.md`, section 4). Domains derive from what they
are given and never from a lower layer's types: the service passes each
the numbers it needs.

**How bounds compose.** Formulas size an edge or the ownership admitted
for one value, never a process-wide sum (skein/docs/foundation/programming-model.md, 6.3).
Every derivation follows these rules:

1. **Joint bounds are charged at their shared total.** Items drawn from
   one shared quantity are charged at that quantity, plus a fixed cell
   per item, never as their count times the largest item. A completion's
   text, call arguments and copied identifiers all come out of one
   answer, `output` tokens at `TOKEN_BYTES` each. A batch's problems
   all come out of one contract's names.
2. **Escape factors apply only to encodings.** `ESCAPE` and a
   rendering's expansion apply to bytes on the wire or in rendered text.
   Decoded content held in memory is charged as decoded. The ×4
   rendering of invalid UTF-8 applies only to bytes that may be invalid.
3. **Counts come from in-memory items.** A count bounded by a byte
   budget is that budget over the smallest item as it is held in memory
   (its cell), never over the smallest item as encoded on the wire.
   Containers allocate as they fill, not at their declared cap.
4. **One factor per direction.** Where a token count bounds bytes (a
   window's history at its threshold, an answer, a request), a token is
   `TOKEN_BYTES` bytes. Where a byte count bounds tokens (a fresh
   window, a room), a byte is one token. One comparison never mixes the
   two. `TOKEN_BYTES` is skein's decoded bytes per declared token;
   `smith-domain-session::ENCODED_BYTES` is the 5/4 allowance for escaping
   and framing in a batch's estimate of room below the line
   (../domain/session-context.md, 1.1). The guard still measures the exact encoding.
5. **An edge queue's bound follows what can be queued at once:** its
   cells and the payload its producers can have queued together, never
   its capacity times the largest payload. Internal queues grow without
   a cap and take no part in an edge reservation.
6. **Frames follow this side's limits.** The channel's and io's limits
   are derived from the agent's own derived limits and from the
   direction a frame travels. A stream's output holds the largest frame
   this side writes. A schema's ceilings bound what may be declared,
   never what is reserved. The product's values set them: where a
   profile's value would pass a ceiling, the ceiling rises in place while
   formats are pre-release (README.md, section 8), as far as the codec's
   encoded and decoded bounds at the ceilings allow (skein's `data/codec/README.md`,
   section 4).
   Many large items are bounded jointly, as one field of bytes at their
   total, as the start's transcript is (1.6), where a list's count times
   its largest item would pass that bound.

Each derivation states which rule it follows where it combines bounds.
The service's tests hold, for the product's values, that every edge
compatibility check passes and each bound is representable. Fixed
scenarios measure heap peaks separately (`limits.md`, section 5).

### 1.1 The LLM path: skein

`skein_llm::Limits::derive(&Declared)` (skein's `llm/wire.md`, sections 1.1
and 1.2) runs once per endpoint. Its `Declared` takes the endpoint's
maxima, the largest `window`, `output` and `reasoning_item` of the models
the configuration declares for it, and the deployment's `tool_payload`,
`calls_per_response` and `conversations`. Its two factors, an escape's
expansion and the bytes a token may stand for, are skein's.

It gives every limit of the LLM path, per dialect: the request, the
answer, one call's input, a reasoning item, replay's metadata, a
completion's output items, the strings it keeps, the tokens it retains,
a call's receiving reservation, the bytes it skips, the tools and the
history items. Events are decoded as they stream and selectively
(skein's `data/json.md`), so what a provider echoes that smith never reads
costs a scan, not memory, and no limit is sized by it. Text smith sends
has no cap of skein's: smith bounds what it renders (1.4), and the
request bounds the whole.

skein's LLM connection component is given, from the same declarations
(skein's `llm/connection.md`, sections 3 and 7):

- **`calls`:** `conversations`, the calls it holds at once, running or
  waiting;
- **`connections`:** `conversations` across the pool, and as many per
  endpoint. skein checks connections ≥ calls when the component is made
  (skein's `llm/wire.md`, section 1.3), so the service refuses to start
  otherwise;
- **`memory`:** `llm_pool`, the bytes calls may reserve at once;
- **`idle keep`:** each endpoint's `connection_keep`;
- **the request head:** measured from the endpoint's headers, the
  affinity's fixed rendering and a credential at its bound (`limits.md`, 2.4).

### 1.2 The session

`smith-domain-session` derives, from skein's limits and the
declarations:

- **a completion's blocks:** its calls, at most `calls_per_response`,
  and the few text, refusal and reasoning blocks a dialect adds;
- **one completion's output:** the charter's `output`, at most the
  model's;
- **a window's history:** the messages, blocks and bytes one window may
  hold, set so that the window's check is reached before any of them
  (../domain/session-context.md, 1.1). Its bytes come from the window's side, by
  rule 4: the line's tokens at `TOKEN_BYTES` bytes each (1.9), plus what
  the window must receive before its next check, which is one
  completion's whole reserve, its answer included, since no output is
  reserved below the threshold, and one batch's results: the
  calls that run together, at most `parallel_tools` (../domain/session.md,
  section 5), each at its render bound. A completion's later batches run
  within what is left (../domain/session-context.md, 1.1), never in more history. So `window-first`
  holds by construction. Its messages and blocks are
  those bytes over a block's cell in memory, by rule 3, and they are
  also skein's declared history items (skein's `llm/wire.md`, 1.2). A prompt
  the session sends is within skein's request, because the line bounds
  it in tokens and the request is derived from the same window. A
  request-size `Limit` cannot follow a passed admission: should one
  fire, a fault in this derivation, the session takes it as *context too
  long* (../domain/session.md, section 4). With
  compaction, a session holds one window, so this is what a session
  holds, not what a run accumulates;
- **what a window holds at its opening, and what a compaction carries:**
  section 1.9;
- **the prompt's allowance:** what the protocol layer and the provider add
  to a prompt beyond the bytes the session counts, from the LLM
  component's framing, for the window's check and the budget's
  reservation (../domain/budget.md). The guard needs no share of it:
  it measures a request's new part with its framing, as encoded (1.9);
- **a completion's bytes,** by rules 1 and 2:
  - its answer once, for its text and its calls' encoded arguments;
  - once more for the decoded arguments and the identifiers copied from
    it, unescaped and at most the answer's bytes in all;
  - its reasoning at skein's share for it, output items × the reasoning
    bound, each item within that bound. A run of consecutive reasoning
    items is one block (skein's `llm/vocabulary.md`, section 1), so one block may
    hold the whole share;
  - a cell per block.

  Its reserve is the completion's bytes plus one answer's bytes for the
  details copied from it, and a slot per block, never twice the
  completion;
- **a decoded call's bytes:** `calls_per_response` decoded calls, each
  within one call's input and all within one answer's bytes (rule 1);
- **a session's jobs:** at most `jobs`, each with its number, the
  provider's id of the `shell` call that started it, its start's time,
  and whether its end has come; a cell each, since their kept output is
  the machine's (1.5). The session also keeps the highest number it has
  given, so that none is reused (../domain/tools.md, section 5);
- **a batch's results:** each result within its render bound. Problems
  that name a contract's fields are charged at that contract's total
  (rule 1), rendered ×4 only where bytes may be invalid UTF-8 (rule 2);
- **a failure's detail,** the last failure's, kept for the operator: the
  provider's text within the largest failure detail skein's dialects
  keep (skein's `llm/wire.md`, 1.2), or a limit's name and bound. The session
  keeps one, each later failure replacing it (../domain/session.md, section
  4); the event stream carries it whole under content capture, and the
  operator's line clips it (`limits.md`, 2.4).

### 1.3 The transcript

`smith-transcript`'s schema seals ceilings; the agent's limits sit within
them (transcript.md, section 7):

- **a message's blocks:** from `calls_per_response`, the largest of:
  - a completion's blocks, skein's output items: twice
    `calls_per_response`, and two;
  - a message of results: `calls_per_response` results, then what is
    appended after them, one offer of messages (1.9) and the run's
    notes;
  - a window's opening: the task's blocks, its instruction and one
    offer, then the summary preamble's block, the summary's and one
    offer of held-back messages.

  So one declared number bounds both the completion and the message of
  its results, in the transcript (transcript.md, section 6);
- **a turn's bytes:** from a completion's answer, its calls' inputs, its
  results at their render bounds and its reasoning, one reasoning block
  holding at most the completion's share (1.2), which the schema's
  opaque block holds by rule 6;
- **a transcript's,** the resume limit: at least one window's turns, as
  a session holds them, and their bytes, since a transcript carries only
  the current window (transcript.md, section 7). Its turns are counted
  from the window's messages, each turn holding at least one, never from
  a turn budget: a window may outlast an activation.

### 1.4 Rendering

`smith-protocol-llm` derives one rendered result's bound (llm.md,
section 8):

- **each workspace tool's outcome at its caps,** rendered: its kept bytes,
  with every escape at its largest expansion, and what the rendering
  adds (line numbers, paths, the head and tail's separator, a clamp's
  note, a job's number and how to read it). A `running` answer and a
  `job` call's answer are outcomes like these: a job's kept output at
  `shell_head` and `shell_tail`, and its end or `running`;
- **a host tool's answer,** with the prefix the rendering adds; its text
  is at most `tool_payload`, like any piece the model reads at once;
- **a message,** with its label's prefix, at its bound (1.9);
- **a sub-agent's result:** its answer cap, the marker that cuts it, and
  the usage line;
- **the cut's marker,** which `limits.md`, section 6's truncation adds.

The system text is not a result: the run writes it, and a window's room
bounds it (1.9).

### 1.5 The machine

`smith-protocol-machine` derives from the tools' limits, never beside
them (agent.md, section 2):

- **a command's captured output:** at least `shell_head` and
  `shell_tail`;
- **its processes:** at least the listings, searches, commands and
  checks the run's calls may have running at once, a batch of at most
  `parallel_tools` in each of `conversations`, and beside them `jobs`
  for each of `conversations`, each job with its process record;
- **a job's kept output:** `shell_head` + `shell_tail` bytes, the head
  and the tail of what it wrote since its last answer, and the count of
  bytes dropped between them, held from its `running` answer until an
  answer reports its end or its session closes;
- **a file's load and store:** at least `file_bytes`;
- **a scan and a search:** at least `list_entries`, `search_hits` and
  `search_bytes`;
- **paths:** at least the tools domain's path bound;
- **the environment:** the configured environment's encoding, exactly
  (agent.md, section 3).

### 1.6 The channel

`smith-protocol-channel` derives its half's limits, and with them its
terms (channel.md, sections 2 and 7):

- **a turn's frame:** at least the largest turn (1.3);
- **a start's frame:** the charter at its run bound, one window's
  transcript, the answers it lacks and the start's messages. The
  transcript is one piece of bytes, bounded at the resume limit's bytes
  with a length for each turn; the agent counts its turns against the
  resume limit's turns as it splits it (channel.md, section 3). The
  parts a resumed start carries are one piece too, counted with the
  transcript's turns against the same limit, in turns and bytes; the
  agent drops parts past it, never refusing the start for them, and
  resumes from the window alone. Main's parts fit beside its window,
  since a session's window held them when they were told; sub-agents'
  may not;
- **a part's frame:** as a turn's, its body bounded as a turn's;
- **a message's frame:** a message at its bound (1.9);
- **its queue:** the window's turns, the calls in flight (at most
  `conversations`), the notices, the long operations, the answer and a
  reserve for facts and pieces, charged by rule 5 of section 1;
- **live content,** channel.md, section 7. Three numbers, all for the
  run's main conversation, the only one that gives any:
  - **the piece:** at most 4096 bytes (`limits.md`, 2.4), a frame of that and its
    fixed fields. A delta longer than it goes in several pieces;
  - **the interval:** `live_interval`, 100 ms (`limits.md`, 2.3). A piece goes when
    its bytes reach the bound or the interval ends since the oldest
    unsent byte, whichever comes first;
  - **the held bytes:** four pieces, 16 KiB, by rule 5: what can wait
    at once for a channel with no room is the retry's shared text and call
    bytes. Each piece contains one stream only. Past it the newest bytes are dropped
    and counted (channel.md, section 7). A piece is cut from the held
    bytes, so nothing else in this path is charged. The queue's reserve
    for pieces is one piece's frame, since the agent sends one piece at
    a time. The held bytes have one owner (1.8, `limits.md`, section 5).
    Identities are bounded at 16,385: one per retained byte plus the open
    call, each with at most 256 id bytes and 64 tool bytes. Their map
    cells, payload and the held bytes' stream/offset cursors belong to
    the scenario's measured total; the host keeps one bounded input identity;
- **by direction,** by rule 6. What the agent writes is bounded by its
  own derived limits. What it reads is bounded by what it admits: a
  start within the charter's run bound, a message within its bound. The
  schema's ceilings only bound what may be declared.

One entrance at the host-channel seam needs `EdgeRoom { to_domain: 1,
below: 6 }`: one owner event and six lower operations. `EDGE_ROOM` holds
that edge credit, exposed by `edge_room()`. An entrance waits
until both destinations have that room; `send_live` and `offer_fact`
instead drop the piece or fact and count the loss.

The host's terms are checked against these at the opening, not at
startup: a host that takes less finds out before a run starts.

### 1.7 The event sink

The largest record follows from the capture policy (events.md, section
5): under `everything`, a response's prompt delta is at most one turn's
growth, its results at their render bounds. The sink's output cap holds
that record and the room it reserves for authoritative records.

### 1.8 The service

`smith-agent-service`, and `smith-local-service` for a run in one
process, compose the chain and check the edge bounds before the loop
(`limits.md`, section 5). Each holds its profiles as step code (README.md, section 5).

**Admission by a watermark** (skein's ../foundation/programming-model.md, 7.1). The queues between the service's own children (the
domain's events and requests, the LLM's, the machine's, IO's and the
file driver's) are uncapped: no child reserves room in another, and
startup does not ask `queue` to hold any child's largest output. Two
kinds of queue keep their caps, as edges: the three the host's channel
fills and drains, and the kernel's submissions and completions, which
the shell fills from outside the service. New work, the host's `Start`
and each `Message`, waits in the channel's queue while the service's
backlog, the entries in its uncapped queues, is at `queue`; everything
that advances admitted work is taken at any backlog, still waiting on
real credit (a destination's, the channel's to the host, the file
table, the kernel's queue). `queue` also bounds each drain's work per
iteration. The agent service ships `DEFAULT_PACING = 2_302` as the
backlog watermark, per-stage work quantum and pending-message room
(skein/docs/foundation/programming-model.md, 7.1). It is a pacing choice, independent of a
domain's largest output. At the host-channel seam, the service keeps
`edge_room()`, `EdgeRoom { to_domain: 1, below: 6 }` (1.6), before
entering the channel; this credit belongs to that edge.

- **Requests precede the next domain entrance.**
  `domain_requests_dispatched` requires the previous entrance's requests
  to be dispatched before a resume, event or timer enters the domain,
  so later facts cannot overtake the turn they follow. The agent process
  world holds this under load in `smith-agent-process-world`,
  `orders::domain_requests_dispatch_before_a_later_entrances_facts`.
- **Facts precede the next domain entrance.**
  `domain_facts_dispatched` requires the previous entrance's facts to
  reach the event sinks before the next entrance. The domain's facts
  queue is uncapped; waiting entrances contribute to the backlog. The
  agent process world holds this with a complete sink held in
  `smith-agent-process-world`,
  `orders::domain_facts_reach_the_sink_before_the_next_entrance`.

The children below (the domain, the inline agent, the host domain)
reserve no room for their own children; each exposes `backlog()` for the root above it,
and only the service has a watermark.

In the local service, a person's line or slash command is its new work:
it waits at the terminal's head while the backlog is at the watermark,
`service.watermark` (by default `queue`); the answers to admitted work
never wait. A queue the shell reaches only through a service method is
internal and uncapped; its edges are the queues it hands out (`&mut
Queue`) and the terminal's input. The backlog counts the queues between
the service's children, and what a child holds through its `backlog()`
(the local domain's counts its agent's, and drains its spawned agent's
host facts, which nothing local reads); the inline effects expose
theirs, and the credential owner keeps its queues. The terminal's
hand-off to the local service is uncapped: consuming a completed input
never waits for internal output room.

The local service retains caps where excess input has a defined result.
Two edges are filled by the agent process: the live text slots
(`LIVE_SLOTS = 4`), which drop text past them or for another run and count
it in `lost`, and received-turn measurements, bounded by the host's
unacknowledged-turn credit. Reasoning and input pieces are never kept.
An over-credit turn is not measured, never gets a `SaveTurn`, and is
faulted by the domain. Sent-message timestamps, filled by the service's
own sends, queue without a cap until a read fence releases them. Before an
account pass, the private `ACCOUNT_EDGE_ROOM` requires six free slots in both shell and
terminal destinations. This is destination credit, not a child's output
reservation.

- **Requests precede the next local entrance.**
  `local_requests_dispatched` requires the previous local entrance's
  requests to be dispatched before a later entrance; an interrupt or
  drop passes this wait. The local service's simulated world holds this
  under load in `smith-local-service`,
  `tests::a_later_entrance_keeps_its_facts_behind_the_earlier_entrances_requests`.
- **Store actions precede the next file terminal.**
  `store_actions_dispatched` requires the store's actions to reach the
  chat before another file terminal or marker result enters. The local
  service's simulated world holds this in `smith-local-service`,
  `process::tests::simulated_store_failure_reaches_chat_before_a_later_marker`.
- **Account file events precede another private action.**
  `account_file_events_dispatched` requires a pending account file event
  to be dispatched before another account's private action, even with
  file credit available. The credential world holds this in
  `smith-local-service`,
  `credentials::tests::private::pending_account_file_events_precede_another_accounts_private_action`.

### 1.9 A window's room

A window is counted in tokens and most of what fills it in bytes. A
token is at least one byte, so a byte count bounds a token count: these
bounds count each byte as one token, and so does the guard, which
measures a request as encoded. The session's check estimates instead:
it counts the bytes appended since its last prompt at that window's
measured ratio (../domain/session-context.md, 1.1). Where the material, a byte a
token, would not fit, the derivation bounds what a window holds, never
the window, which is the model's.

For each endpoint the configuration declares:

- **The room** is its `window` at `compaction_threshold`, less two
  `output`s: one for the summary that opens a new window, one for the
  output of that window's first completion, so that the check holds it
  below the threshold (../domain/session-context.md, 1.1). The summary counts as
  the tokens its compaction reported, or `output` when none was, never
  as its bytes.
- **The line** is its `window` less one `output`, the summary
  instruction and a cut result's marker for each of `calls_per_response`
  calls. No ordinary prompt is sent past it: the guard measures what a
  request adds since its window's last prompt as encoded, its framing
  included, a byte a token: the request's encoded body less that of the
  same request cut where the new part starts. It refuses, unsent, a
  request whose last prompt and that measure pass the line
  (../domain/session-context.md, 1.1; llm.md, section 6). The compaction after a
  prompt within the line, carrying that prompt's answer of at most
  `output`, fits the window.
  - **The material above the line,** the summary instruction's bytes
    plus `calls_per_response` cut markers' bytes, a byte a token, is one
    derived limit of the run's, `run.compaction_material`. The run gives
    each conversation its line when it opens it: the charter's effective
    `window` for its model, less its `output` and that material. So a
    reader of the event stream finds each conversation's line from its
    model's `window_tokens` and `output_tokens` in `conversation.opened`
    and the material among the effective limits (`limits.md`, section 9; events.md,
    3.4 and 3.10).
- **The spare** is what lies between the threshold and the line: the
  window above `compaction_threshold` less one `output`, the summary
  instruction and the markers, what `compaction-headroom` leaves (`limits.md`, section 4
  ). It is the room the session's estimate may miss by before the guard
  refuses a request. A charter's effective `window` and `output` give the
  line and the spare its session uses.
- **smith's own part** is taken first: the run's sections at their
  largest, smith's tools' schemas, the opening's and the summary's
  instructions, the summary preamble, and the prompt's allowance (1.2).
  smith's code fixes them, and the service measures them at startup. Only what is the same
  whatever the charter says is smith's own. A number a section shows,
  such as a budget's, counts at its largest.
- **The rest is split in halves,** a factor of the derivation's own:
  - **the host's part:** every byte that is there because of what the
    charter chose, and its directories' guides. That includes:
    - the charter's instructions, the task role among them when the
      shell gives it (`../shell/settings.md`, 2.6), its brief, and its
      host tools' descriptions and schemas;
    - what smith renders from the charter's choices, in the system text
      and in the schemas it generates for `finish` and `deliver`: the
      contract's forms, fields, rules, verdict labels and items; the
      workspace's directory names, conflicts and convention paths;
      everything smith adds around each of them.

    It is measured as rendered, at the entrance, never at the
    vocabulary's maxima. A start whose charter alone passes it is
    refused at the entrance.
  - **The guides share what the charter leaves.** Each of the start's
    directories keeps its guide whole up to its share: the host's part
    less the charter as rendered, split evenly among the start's
    directories, and never more than `guide`. Past its share a guide is
    cut as past `guide` (../domain/run-start.md, 1.3). So a start with one
    directory keeps the guide the product sizes, and a start with
    several fits whatever their guides hold: more directories cut their
    guides sooner, never refuse the start. The count is the start's
    directories, never the run's `directories` limit, which bounds only
    how many a start may name.
  - **How the run counts what the protocol renders.** The system text
    is the run's own to write, so the run counts it directly. The
    generated schemas are `smith-protocol-llm`'s to render, and the run
    cannot call the renderer without a cycle, so the run counts them
    from plain numbers in its limits:
    - **what the numbers are:** the schemas' fixed bytes, each kind of
      element's bytes besides its own text, and each byte's escaped
      length;
    - **where they come from:** the service measures them at startup
      with the protocol's renderer, as it measures smith's own part;
    - **every entrance uses them.** The root's, the inline agent's and
      the spawned agent's entrances all run the run's admission, so
      each is guarded, and none needs a measurement from the protocol
      at the entrance;
    - **a property keeps them exact:** the count equals the rendered
      bytes for drawn contracts. A sub-agent's brief takes the charter's place, and a
    `sub_agent` call whose brief passes it is answered as too large
    (llm.md, section 4);
  - **the window's messages:** two offers (below). The task is the
    opening's instruction and one offer, and a new window carries the
    task, the summary preamble, counted in smith's own part, the summary
    and at most one offer of held-back messages.
- **One offer** is the most the run gives the model at once (../domain/run-start.md, section 1.4, and ../domain/run-messages.md, section 1
  ): at most `inbox` messages, and at most half the
  window's messages in rendered bytes. What an offer leaves stays
  queued, in order, for the next one.
- **A message's bound** is at most one offer's bytes, and at most
  `tool_payload`. So every message a run admits fits an offer and can be
  read, and a new window always has room for one.

The derivation takes the smallest room among the declared endpoints, so one
set of bounds holds whichever model a charter names. A charter may lower
a model's `window` or `output`, but not so far that its room falls below
the one the limits were derived for (charter.md, section 6).

A window closes by compaction: its history, the results owed to it, and
the summary instruction, then a summary of up to `output`, the
compaction's output, beside its prompt (`limits.md`, 2.1; ../domain/session-context.md, 1.2).
The guard keeps each prompt sent within the line, and its output is at
most `output`, so the history a compaction carries leaves above it at
least the summary instruction and a marker for each owed result. **The results owed to a
compaction** are rendered within what the window leaves after the
history before them and the summary instruction; each is cut with its
marker where they would pass it (llm.md, section 5). History before
them is unchanged, so the compaction still reads the cached prefix, but
for what its tool choice costs on the Anthropic dialect (llm.md, 2.2).

### 1.10 A run's charter and outcome

Both follow `tool_payload`, so a host that hands a run one call's worth of
words keeps them whole, and a run can answer in one call's worth:

- **The charter's allowance,** `run_bytes`, is the larger of 65,536 and
  twice `tool_payload`: room for a whole spec of one call beside the
  charter's instructions and tools. It bounds a charter's decoded ownership
  with its workspace's (../domain/run.md, section 4), and a host's
  `charter_bytes` takes it.
- **The outcome,** `outcome_bytes`, is the larger of 4,096 and
  `tool_payload`: what one `finish` or `deliver` call can say. It bounds the
  declared values together, their names and cells among them, so a field's
  own bound may be near it and leave the rest to the others. A host's
  `outcome_bytes` takes it, and the host's own bound must hold it.
- **A sub-agent's answer,** `answer_bytes`, is the outcome's room, the
  larger of 4,096 and `tool_payload`. A child's answer is its run's result
  to its asker, so it has the room one `finish` has, and a survey a run
  asks for comes back in one call's worth. The child is told it in its
  share, and an answer past it is cut (../domain/run.md, 5.3). `smith check`
  prints it as `run.answer_bytes` (`limits.md`, section 9).

Each enters T through a sub-agent's feedback, which holds four times the
larger of them (../domain/run.md, section 5): up to a `tool_payload` of 32,768
the allowance stays 65,536, the outcome stays below it, and T doesn't move.
The answer's feedback, the answer at `answer_bytes` with its cut's words
and its usage line, is within that feedback, so the answer moves no bound
the outcome doesn't, and `render-in-result` holds it (`limits.md`, section 4).
