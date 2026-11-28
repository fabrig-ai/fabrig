# The run

Provisional, 2026-10-05, revised 2026-11-02. What a smith run is, as a
domain layer: one activation of an agent, from the charter its host gives
it to the answer it gives back. It is the child domain `smith-domain-run`,
composed with its sessions by `smith-domain` (README.md, section 2). The
mechanics are those of skein's `programming-model.md`. What is still open
is listed in section 13.

## 1. In one page

- **A run is one activation.** A host starts it with a charter, a
  workspace, the messages that triggered it and, when it resumes, a
  transcript; it answers once. Between runs nothing of the agent is live:
  continuity is the transcript, which the host keeps turn by turn.
- **Finite or open.** The host selects `Mode::Finite`, with a result accepted
  through `finish`, or `Mode::Open`, whose text replies wait and whose
  activation parks. Open requires `wait` and forbids `deliver`; it offers
  neither terminal tool. The host decides what ends the conversation.
- **A flexible LLM driver.** A chat with a person, research, a triage, a
  review, operating a system through its host's tools, a coordinator
  revising a plan, a code change: what a run is for is its charter's. The
  mechanics are the same for all.
- **Tools from four sources:** the workspace's, when it has one, which
  its sessions run;
  the run's own (`finish`, `deliver`, `wait`, sub-agents); the host's,
  declared in the charter and relayed; and, later, MCP servers. To a
  session they differ only in who answers.
- **Results are judged against a contract** given as data: a report, a
  verdict from a closed list, a change, or a declared failure, each with
  the fields it requires.
- **A change is checked, then delivered,** when a run has a writable
  workspace: the run runs its checks; its host makes the checked state
  durable and says how that went. smith knows directories, git working
  trees and checks; what delivering means is the host's. A run granted
  `save` may also hand its tree over unchecked, marked so, to keep its
  work before it asks a question (`run-delivery.md`, 1.5).
- **One budget,** in the host's unit, across every session the run opens,
  each completion priced as it is made. Time and spend plan the work;
  turns only guard the loop. A reserve keeps main's last word: when the
  rest is spent, its sub-agents close and it finishes with what it has.
- **Every message ends once:** read by a turn, refused at the entrance, or
  unread when the run answers, and the answer names the last one read.
- **Fixed, then appended.** What a conversation's model is told first
  (its system text and its tools) never changes while it lives; what
  changes as the run goes is appended, so each request extends the last.
- **Failures are feedback.** A failing check, a broken contract, a
  delivery refused or a host tool's error goes back to the LLM, whole and
  bounded, for it to fix. Only what it cannot fix ends the run.

## 2. The run in the agent

```
host      starts runs; serves host tools and deliveries; keeps turns; hears the answer
run       charter, served tools, results, delivery, budget; owns its sessions
session   one conversation with an LLM; runs the workspace's tools
```

- **The run acts on the world only through its host and its
  workspace:** files written in its writable directories, deliveries the
  host makes, host tools the host answers. It never commits, pushes or
  reaches a system on its own authority.
- **Credentials never reach the domain.** A charter names endpoints; the
  protocol layer holds what they need, lent by the host or configured in
  the agent (host.md, section 7).
- **Retrying a failed run is decided above the run,** by its host, across
  attempts. A run answers once and is done.
- **Only a run opens sessions,** so ownership is a tree: a run owns its
  sessions, and which session is whose sub-agent is the run's
  bookkeeping.

## 4. What a run does

- **Inputs precede alarms.** `run_inputs_drained` requires admitted input
  events to be handled before expired timers, so a check terminal at its
  deadline is handled before the alarm. The run world holds this under
  load in `smith-run-world`,
  `orders::run_inputs_drained_before_alarms_at_the_check_deadline`.
- **Inputs precede deferred work.** `run_inputs_drained` also requires
  input events to be handled before resuming deferred wind-down work.
  The run world holds this under load in `smith-run-world`,
  `orders::run_inputs_drained_before_deferred_wind_down`.

1. **Admits** a start, or refuses it at the entrance: busy, or invalid
   (a charter in a version it does not read, a charter that does not
   decode, a window of unacknowledged turns that cannot hold one turn,
   beyond the agent's `Limits`, an endpoint it is not configured with, a
   model its configuration does not declare for that endpoint, a window
   or an output above what the configuration declares for the model or
   too small for the agent's limits, charter text too large for a
   window's host part, a host tool whose name is smith's or appears
   twice, a contract that cannot be met, a model priced at zero at any
   rate, a budget that cannot hold its reserve, an LLM's account the
   start carries no grant for, a grant the agent cannot install
   (protocol/channel.md, section 6), a program source granted for a
   sub-agent's LLM and not for main's (protocol/programs.md,
   section 2)). The start's messages are admitted
   or refused with it. A host that builds a brief measures its room
   first: `brief_room` is the window's host part less the exact host
   bytes of everything but the brief, for the charter and the workspace
   (the empty guide's scaffolding included), and `brief_bytes` counts the
   brief's sections and framing as the run renders them, so a brief at
   the room fits the host part and one byte more is refused.
   Separately, `charter_room` subtracts the prepared workspace's ownership
   from `run_bytes`, using admission's accounting. The charter's
   `owned_bytes()` must fit that remainder; rendered room does not replace
   this combined ownership check. A workspace over the bound or an
   overflowing count leaves no allowance.
2. **Prepares:** reads each workspace directory's guide and looks for its
   checks (`run-delivery.md`, 1.1).
3. **Equips** its main session: prompt, tools, workspace authority, LLM,
   the run's identity, a share of the budget, the summary instruction
   and its preamble, and the transcript it resumes, if any; and opens it
   once there is work, or awaits its first message (`run-start.md`, 1.4).
4. **Drives** the session. A session *yields* when the LLM stops calling
   tools, and the run decides what happens next: nudge the LLM ("you have
   not finished"), pass on the messages that arrived, wait for one, or
   close it.
5. **Serves its tools and relays the host's** (section 5). It runs each
   call's deadline: a call that times out returns only once what it
   started has settled (a sub-agent ended, a delivery answered), so
   nothing it started is still writing.
6. **Tells each turn** to its host as it ends (session.md, section 3):
   what the LLM said and called, the results, what it spent, and the last
   message it read; and, to a host that asks, its parts as they come
   (turn-parts.md).
7. **Judges** a declared result against the contract (section 7). A
   violation goes back to the LLM as a tool error it can fix; it does not
   end the run.
8. **Accounts** every session it opens against one budget (`budget.md`),
   its sub-agents' included, keeps a reserve for finishing, and winds
   down when the rest runs out (`budget.md`, 1).
9. **Ends once.** It closes its sessions, waits for what is in flight to
   settle, and answers its host (section 9). A cancel from the host takes
   the same path.
10. **Reports** facts as it goes (section 10).

## 5. Tools

### 5.1 Where tools come from

| Source | Tools | Who answers | Effect |
|---|---|---|---|
| the workspace | `read`, `list`, `search`; `write`, `edit`, `remove`; `shell`, `job` | the session, through its tools (tools.md) | reads; writes; `job` reads, and writes with `stop` |
| the run | `finish`, `deliver`, `save`, `wait`, sub-agents | the run | `finish`, `deliver` and `save` write; a sub-agent's follows its tools |
| the host | declared in the charter | the host, relayed | as declared |
| MCP servers, later | as each server lists them | the protocol layer, as opaque calls | as the charter declares |

Names are unique across sources; a declared tool may not take one of
smith's, nor `permission_prompt`, which a program backend keeps for its
prompt tool (../protocol/programs-claude-code.md, 1.3). The charter's families say which of
smith's tools the LLM sees; a host tool is offered exactly when it is
declared.

### 5.2 Host tools

A host tool is an action of the host's that a run offers its LLM: in
temper, delegating a task, messaging, deciding a proposal, reading a
connector's system.

- **Declared as data:** a name, a description for the LLM, an input
  schema (JSON Schema, bounded in bytes), an effect (read or write) and a
  deadline. The protocol layer hands the schema to the provider as it
  came.
- **Relayed whole.** A call carries the run's name for it, the tool's
  name and the input the LLM wrote, as bytes, checked only to be a JSON
  object within the limits. The host decodes it, decides, and answers
  with text for the LLM, as a result or as an error. smith never
  interprets a host tool, and never retries one on its own judgement of
  what it did.
- **Asked again, never anew.** A call left unanswered (withdrawn past its
  deadline, or lost with a channel) or answered busy is asked again with
  the same name while the run lives, after a backoff, once the earlier
  relay of it has ended. The backoff is `host_backoff` before the second
  ask and doubles before each later one, to at most 64 times
  `host_backoff`; an ask that would start at or past the caller's
  deadline is not made, and the call returns busy, or unknown where an ask went
  unanswered. A host is owed that a name is decided once and
  answered from its record when asked again (host.md, section 2); a name
  carries the start's activation number, so no later activation reuses
  it. Only
  when the run cannot learn the outcome is the LLM told that it is
  unknown, and then it may look (with a host tool that reads) before
  acting again.
- **Unsent is busy.** A withdrawal the host answers unsent (its link
  never carried the call, as when the engine is away: jig's
  `domain/hosts/hub.md`, 6) leaves no uncertain effect: the call is
  asked again as one answered busy is, within the caller's deadline and
  the run's, and returns busy, never unknown, when no ask is left. So a
  relayed call waits for an engine that restarts within its own
  deadlines, and past them fails to its LLM as any slow call does
  (temper's `draft/archive/redeploys.md`, 5.3).
- **Scheduled by effect,** like any tool (session.md, section 5): a write
  runs alone, adjacent reads together.
- **Refusals are the host's words:** a call beyond what the host allows
  is answered as an error saying what it lacked, and the LLM may act on
  that, by asking otherwise if the host offers a way.

### 5.3 Sub-agents

A sub-agent is a served call that the run answers by opening another
session: the same workspace; its own tools, from the run's workspace
families (often read-only), never the run's served tools or the host's;
a share of what is left above the run's reserve; and one of the LLMs
the charter lists, which the asking LLM may name, or else main's. Its
conversation carries the run's identity with an ordinal of its own
(`run-start.md`, 1.2). When the child ends, its final message is the parent's tool
result; its spend is counted once, through that result (session.md,
section 6).

- **The call:** `sub_agent { brief, tools, llm?, max_turns?,
  max_seconds? }`: the brief the child works on; the tool families it
  gets, no wider than the asker's; the LLM it runs on; and the turns and
  seconds it asks for. Its effective share is what it asked, each part
  clamped to what is left above the reserve when it asks (`budget.md`, 1), or all of
  that when it asked for none. Its spend share follows the same rule:
  never asked for, so all of the spend left above the reserve when it
  asks, and never shown, since the host's unit means nothing to a model.
  No formula ties spend to the turns or seconds asked:
  - asked turns already bound a child's spend, since each completion
    reserves its maximum cost (`budget.md`);
  - children open at once share the ordinary pool, and every completion
    reserves from the run's one budget, so together they never pass it.
  The brief takes the charter's place in the child's window, so a brief
  larger than a window's host part is answered as too large, and no child
  opens (../protocol/limits-derivation.md, section 1.9).
- **The room.** A run holds at most `conversations` conversations at
  once (protocol/limits.md, 2.2), main among them, so at most
  `conversations` less one children are open at once. A `sub_agent`
  call asked while they are open is refused as too many
  (`refused reason=too-many`), and one asked while the agent's own
  tables of conversations or calls are full answers busy; either way no
  child opens and its parent goes on, and a later call may open a child
  once one has answered. A message may ask more calls than the room
  holds: the run takes them one at a time, in the order it is asked
  them, opening each that fits and refusing the rest, so a burst of
  calls is answered whole, call by call, whichever backend asked it. A
  run's room is as configured; no burst changes it.
- **No nesting.** A child never opens children, and is never offered
  `sub_agent`. A call that asks for nesting anyway is refused with a
  typed problem saying that a sub-agent does its work itself
  (protocol/llm.md, section 4).
- **Told its share.** The child's system text has `## Your share`: its
  effective turns and seconds, the most its answer may hold, and that its
  last turn has no tools. Like main's `## Budget` (`budget.md`, 2), it says what was
  set at open, never what is left.
- **A final turn.** The run gives each child, at open, a final-turn note
  and a wrap-up allowance of time: the smaller of the policy value
  `wrap_up` (protocol/limits.md, 2.3) and a quarter of the child's
  effective time. A short share then keeps most of its time for
  ordinary turns. When the child's own turns are down to
  one, or its time to that allowance, the session gives it its final turn
  (session.md, section 7): the note, and no tool to choose, the tools
  offered unchanged. The child answers with what it has, and the run
  gives that answer to its parent with `stop: budget`: an answer that its
  share ended, not one the child chose to give. Its parent hears it like
  any other answer. A child returns unanswered only when it fails or is
  closed (`budget.md`, 1).
- **The result.** The child's final message, up to the answer cap it was
  told, `answer_bytes`, the outcome's room (../protocol/limits-derivation.md, 1.10), then
  a usage line the run writes: whether the child stopped for its budget,
  what it used, in turns and time, and the turns and time the run has left
  above its reserve (`budget.md`, 2).
  - **A cut answer** keeps its head to the cap, cut at a character's
    boundary, and its marker says how much was left out and how to read
    it. The child has closed when it answers, so nothing continues it: the
    rest is read by asking a new child, narrower. Its marker is
    `[child-result cut=N stop=S]`, then on a line of its own `The answer
    above is cut: its last N bytes were left out. The sub-agent has closed;
    to read the rest, ask a new one for it with a narrower brief.`, N the
    bytes left out and S the stop. An answer cut by nothing, whose stop is
    not `end-turn`, keeps the bracket alone, `[child-result cut=0
    stop=S]`.
- **One writer at a time: the lease.** Write authority over overlapping
  directories has at most one holder at a time. A child granted modify or
  shell holds the run's lease for its call's life: its call is a write,
  which runs alone (session.md, section 5), so its parent waits and writes
  nothing meanwhile. Children granted only inspect hold no lease, and run
  side by side, with each other and with the reads around them in their
  parent's turn. Checks and delivery hold the lease over the whole
  workspace (`run-delivery.md`, 1.1).
  - **Jobs give way.** A session's running jobs are its writes in
    progress (tools.md, section 5), so whatever takes the lease first
    cancels the running jobs of every other session, and waits for them
    to settle: a writing `sub_agent` call cancels its parent's jobs
    before its child opens, and the checks and the delivery cancel every
    running job before they run (`run-delivery.md`, 1.1).
  - **A child's jobs end with it.** A child granted shell may start
    jobs; its session's close cancels them (session.md, section 5), so
    none outlives the call that asked for it.

  The lease follows from how calls are scheduled and from these cancels;
  naming it lets a later step change who holds it while keeping what it
  means. The first such step, a read-only parent working beside one
  writing child, is a later document's (section 13).
- **Focused briefs.** Main's system text says how to delegate: a child
  starts cold, knowing only its brief, the guides, its tools and its
  share, so a brief names the paths, the decisions already made and the
  scope; independent reading splits among inspect-only children, which
  run side by side; a writing child runs alone.
- **On a program backend** (../protocol/programs-claude-code.md, 1.10), the run serves
  `sub_agent` through the agent's MCP server and answers it as here,
  each child a program session of its own at the model and effort of
  its LLM, or a direct session where its LLM's endpoint names no
  program. Its children are granted inspect alone: none holds the
  lease, and they run side by side. Its share, its final turn, its
  result and its spend are this section's; how the program is told
  them is the program's document's.

Within a run, sub-agents: cheap, sharing its workspace and budget, ended
with the call that asked for them. Work that should outlive the run, run
elsewhere or have its own authority is the host's, which may offer a
host tool for it, as temper's `delegate` does.

## 7. Results

### 7.1 The result contract

A contract allows any of four forms:

- **A report:** text, bounded, with the fields the contract requires.
- **A verdict:** one label from a closed list, each label with a contract
  of its own: which fields it requires, and which items it may carry, how
  many (a least and a most) and of which kinds, each kind with the fields
  it requires.
- **A change:** the workspace's changes, delivered (`run-delivery.md`, section 1), with the
  fields the contract requires, and whether the checks must pass.
- **A failure:** the LLM declares it cannot do the work, with a reason.

A result is its form, its label for a verdict, its text, its fields (a
name and text each) and its items (a kind and fields each). Fields,
labels and kinds are names the host chose: temper's change requires a
title and a body, its review's verdicts are an approval and a request
for changes whose items are follow-up tasks. smith knows only that a
field is required, non-empty and within its bound.

### 7.2 Finishing

`finish` is a tool, not a convention about the last message. Its input
schema is generated from the contract by the protocol layer, which
decodes the LLM's input into a result. The run judges it and answers
with acceptance or with what is wrong; what is wrong is a list of typed
problems (a form not allowed, an unknown label, a missing field, too few
or too many items, an item of a kind not allowed), each named, which the
LLM can fix. An LLM that stops without finishing is nudged, within the
budget. A `finish` that crosses a queued message, in a run that may
wait, is answered with the messages instead of judged (`run-messages.md`, section 1).

An accepted report, verdict or failure ends the run. A change is
delivered first (`run-delivery.md`, section 1), and ends the run once its delivery lands.

### 7.3 What is checked twice

The run checks the form of a result; the host may check it again against
what it knows, as it commits it, and refuse it as a failed attempt for
its next run to hear of. smith does not make the host's judgement.

## 9. The answer

A run answers once, after every turn it counts, with its final fence:
the last message it read (`run-messages.md`, section 1).

- **accepted,** with its result, its turn count and what it spent; also
  when the budget forced its finish, the result accepted in the wind-down
  (`budget.md`, 1);
- **parked,** with its turn count and what it spent;
- **failed,** typed so the host can act without reading prose: the model
  (a provider's failure past its retries, with its class, its evidence,
  the endpoint and model of the last completion, how many attempts it
  made and its report: the provider's code, request id and field, and the
  HTTP status a program reported for its call, which its evidence does
  not carry (../protocol/programs-claude-code.md, 1.6); an
  account limited, naming the account and, when its provider said, how
  long until it may be used again, measured when the run answers; an
  account refused, naming it and the provider's code; a program
  source's login lost, naming the account; a program that started
  without smith's tools, *untooled*, a fault of its host's program
  rather than the run's, naming the program, with its detail
  (../protocol/programs-claude-code.md, 1.6); a program that could not
  run or exited before its answer, naming the program, with its detail
  (../protocol/programs-claude-code.md, 1.6); a limit that fired, naming the limit and
  its
  bound; or a fault of the model's own answer, *refused*, *truncated* or
  *malformed*, when it kept refusing, stopping at its output limit, or
  asking for tools and naming none past its nudges (7.2), or when a
  compaction's answer gave no summary (session-context.md, section 1.2)), the
  budget (naming what ran out, how much of it the run used and its
  bound, each in its own unit: turns, spend in the host's unit, or the
  run's wall time as its time budget counts it, `budget.md`), policy (a
  call or result the run refused
  to make), cancelled, stale, or a transcript it could not resume; each
  with what it spent.

The type is also what a host shows: a host words a failure from its
type, in every placement, rather than saying only that the run failed
(host.md, section 8).

Refused at the entrance, it answers refused: busy, or invalid, saying
what is beyond the limits; the start's messages are refused with it. A
cancel from the host ends the run along the same path; a change its
finish delivered meanwhile is still its result (`run-delivery.md`, 1.4).

**A drain from the host** (protocol/channel.md, section 3) ends the run at the end of its current turn, rather than in the middle of it:

- **No new completion** of the main conversation starts once the run has heard it, a finishing one of a wind-down (`budget.md`, 1) included. A completion in flight goes on to its answer, and each call that answer makes runs to its result, a sub-agent's whole conversation and a check included, so the turn is whole.
- **Then the run ends as a cancel ends it:** the turn told, its sessions closed, what is in flight settled, and the answer *failed: cancelled*, with every turn it told and its fence. Messages that arrived meanwhile stay unread, named once as any are (`host.md`, section 4).
- **At once** when no completion of main is in flight: a run awaiting its first message, waiting, listening after a reply, or still preparing ends as a cancel ends it.
- **A turn that ends the run anyway** answers as it would have: a result accepted, a park.
- **A cancel after a drain** cuts the turn, as any cancel does. The drain's wait is the host's to bound: smith arms nothing for it.
- **A drain with a grace,** from a host that asked for parts, ends the run at its answer instead: the calls in flight get the grace, then are cut, and the turn stays open for the next activation (turn-parts.md, section 4).

## 10. Facts

Each child domain pushes what happened as typed, content-free facts into
an uncapped queue (programming-model.md, section 3), so no fact is lost
inside the process. The run exposes its queued count through
`pending_facts()` for its parent to drain, without an allowance for one
entrance. The service dispatches those facts before the next
domain entrance (../protocol/limits-derivation.md, 1.8). Each carries when it happened,
taken by the step that emits it, never when a sink takes it:

- a run admitted or ended;
- a conversation opened or closed (main, a sub-agent or a compaction, a
  compaction's with the conversation it summarises), with its parent; a
  sub-agent's with its effective share and the reasons each part of it
  was clamped, and on its return what it used: turns, time and spend;
- an LLM call started, retried or finished, with its usage;
- a tool or a check started (with its deadline) or finished, a tool's
  with the provider's id for its call; and a job's end that no answer
  reported, as its session closes (tools.md, section 5; `job.ended`,
  protocol/events.md, section 3.6);
- a message received, or refused with its reason, then read by a told
  turn or unread at the answer, so every message's terminal is a fact;
- the budget, as it stands at each charge, and its stage, ordinary or
  winding down, when it changes (`budget.md`, 1); and, as a sub-agent's return
  writes its usage line, the values that line tells, with the call
  (`budget.md`, 2);
- text arriving from the LLM, counted, never carried. Live content
  (protocol/channel.md, section 4) is not a fact: it goes beside the
  domain, at the host's ask.

The protocol layer turns them into the event vocabulary
(`protocol/events.md`), and each sink applies its own policy and counts
what it loses: the event stream (a trace file, or a headless run's
output) is complete by default, held back by its stream rather than
dropping; the channel's projection, for the host's liveness, drops what
does not fit, counting it as not projected, and never holds the run.
Content (prompts, completions, also as their text streams, and
tools' input and output) goes only to the event stream, as its capture
policy allows, and as live content to the host that asked (never through
the domain). The records that say how the run and its process ended are
built from the answer and the process, not from facts, and are never
dropped. Nothing the agent decides depends on a fact: a sink may slow the
run, never change what it does. What must be delivered (turns, calls,
the answer, what was spent) is not a fact.

## 11. Below the domain

- **Schemas and decoding:** the protocol layer owns the schemas of
  smith's tools, `finish`'s generated from the contract, and passes the
  host's on as given; it decodes owned and served calls into typed
  entities, and checks host tools' inputs only for being JSON objects
  within limits.
- **Text as written:** the system text, and the results and problems the
  domain wrote (`run-start.md`, 1.3), placed in each provider's request.
- **Affinity and tool choice:** the run's identity with each
  conversation's ordinal, and the tools a turn may choose among, rendered
  for each dialect by skein's LLM client (protocol/llm.md, sections 2.1
  and 3).
- **Events:** facts encoded for each sink (protocol/events.md).
- **The channel** to the host (host.md, section 3).
- **Files and processes,** through io, when there is a workspace: guides
  read as text, checks run
  as contained processes with deadlines and their output's tail.

## 12. The world

The run's world runs the domain against neighbours that share none of
its types (testing-strategy.md, section 4): a scripted session, whose
yields, turns and served calls a script draws; the machine's faces for
guides and checks; and a scripted host: messages, host tools' answers
(busy, errors, lost), deliveries' outcomes, grants, cancels. The agent's
top-level world puts the run with real sessions and tools, a fake LLM
provider and the scripted host; a host's own system world (temper's
worker's) puts it under that host. Its typed provider adapter decodes
owned file calls under the caller's session answer bound and tool
declarations, so a deployment profile is exercised without a smaller
fixture's receiving limit.

Its stories: a change produced, checked and delivered; a verdict judged
against its contract, corrected and accepted; a chat that waits, hears
its person's words, parks, and resumes from its transcript; a run whose
host tools answer busy, asked again under their names; a resolution from
a merge in progress; a sub-agent on another model; a delivery found
stale; a chat with no workspace, only host tools; a cancel in the middle
of a turn.

For jobs:

- a build run as a job while main reads and edits, read to its end;
- main's jobs cancelled before a writing child opens, and before a
  `finish`'s checks and a `deliver`'s; a writing child's job ended with
  the child;
- a wind-down with a job running, cancelled by the `finish` that
  delivers;
- a run parked with a job running, the job told ended, and a `job` call
  after the resume answered as invalid.

For messages:

- a start that carries messages, opened with the brief's instruction and
  then the messages in order;
- an empty brief that awaits, asking no provider anything until its
  first message, then opening on it; one that awaits past its waiting
  time and parks with no turn; one whose ordinary time ends first,
  parked the same way;
- a headless run granted `wait` with a waiting time of zero, parking as
  soon as it would wait;
- more messages queued than one offer holds, the rest offered at the
  next yield, in order;
- an empty brief without `wait`, opened on its instructions;
- a message arriving at each point of a run: as it prepares, as it
  awaits, during a completion, while main waits on a sub-agent, as main
  finishes, as a change lands, as it parks, after a cancel;
- a finish crossing a queued message, given the messages instead;
- a `wait` answered with its message in one request, and one withheld
  through a park and supplied on resuming.

For budgets and sub-agents:

- a sub-agent asking more than is left above the reserve, clamped, and
  told its share;
- a child that reaches its final turn and answers for its budget;
- a child asking for sub-agents, refused;
- a sub-agent that spends the ordinary pool: it is closed, then main
  finishes in its final turns, accepted;
- the ordinary time running out during a sub-agent;
- the ordinary time running out while main waits, parked, not wound
  down;
- final turns ignored, failing once for the budget;
- a compaction, counted as a turn and as spend, the system text
  unchanged, given the run's summary instruction and preamble;
- a compaction refused, failing the run as *refused*, the message it
  held back unread;
- inspect-only children side by side, and a writing child alone.

Its referee: one answer per run, after every turn it took; turns in
order; nothing written outside the writable directories; budgets
exceeded by at most one completion per open session, spend never; every
call answered once; every host call asked under one name. And:

- every message accepted ends exactly once, read, refused or unread;
  fences never go back; the answer's fence is the last told turn's;
- no request to a provider before the run has work;
- a session's system text and offered tools never change, and each
  request in a window extends the one before;
- a sub-agent's effective share is within what was left above the
  reserve when it was asked;
- at most one lease holder over overlapping directories, and checks and
  deliveries only under the whole workspace's; no job running in
  another session while a writing child lives, and none while a check
  or a delivery runs;
- every job's end told once, by an answer or as its session closes;
- with a reserve of a turn or more, no budget failure before main's last
  final turn.

The agent's top-level world checks the same against the fake provider:
requests carry `## Budget`, `## Your share` and the usage lines, and
final turns offer no tool to choose, or only `finish` and `deliver`;
compactions offer no tool to choose, and each new window opens with the
task, the preamble and the summary.
These are domain-speed stories under a deterministic clock and seed,
within the focused suite's budget; the fuzzy suite draws message
timings, budgets and shares within its own (`docs/design/testing.md`).

## 13. Open questions

- **A silent check** (`run-delivery.md`, 1.1): feedback carries only what its process
  wrote and the machine read. Progress messages belong to the repository's
  check, so a timeout before it writes can still have an empty tail.
  Default: preserve that evidence, with the path and given time; no
  generated progress output or longer timeout in the run.
- **A sub-agent's result:** its final message; whether a sub-agent
  may be given a contract of its own.
- **A cut answer's rest** (5.3): read by asking a new child, narrower.
  Default: the run keeps nothing of a cut answer past its cap. Whether it
  keeps the rest for its asker to read by a call, a continuation, waits on
  runs where asking again costs more than the rest is worth.
- **A run's refusal on the channel:** a run that refuses its start for
  policy answers failed; whether refusal wants a kind of its own.
- **Conventions' home:** in the charter, as here, or in the agent's
  configuration, with the charter overriding.
- **Spend in `## Budget`:** none is shown (`budget.md`, 2), since the host's unit
  means nothing to a model; whether a share of it, as a fraction, would
  help a model plan.
- **A forced acceptance, marked:** whether the answer says that an
  accepted result came in the wind-down, so a host can tell a finished
  task from one the budget cut short.
- **Threads across activations:** a sub-agent's thread is its ordinal
  in its activation, so a later activation's children reuse earlier
  threads under the run's one key. Affinity is a routing hint, so a
  reused thread can cost a cache miss, never correctness. Whether
  ordinals should continue across activations, which would need the
  count kept with the transcript, waits on a capture showing a provider
  keeps state per thread.
- **The workspace's word in the system text:** whether it says
  "workspace", as this design does, or "checkout": a change to what the
  LLM reads (`run-start.md`, 1.3).
- **A child's context:** whether a sub-agent may open from its parent's
  transcript, bounded by its limits and counted in its share, so it
  starts warm.
- **Work beside sub-agents,** a later document's: a read-only parent
  working beside one writing child that holds the lease, which needs
  messages mid-work, a rule for when results join, `finish` and `deliver`
  waiting for or closing live children, and no parking with them live;
  then write scopes over directories, with contained trees; then
  asynchronous children, whose spend shares become reservations and to
  which messages may go, under the same terminals.
