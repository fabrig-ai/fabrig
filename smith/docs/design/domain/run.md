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
  work before it asks a question (8.5).
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

## 3. Starting a run

### 3.1 The charter

What the host gives a run to set it up:

- **Mode:** `Finite` or `Open`, stated by every creator. Open requires
  waiting authority and no delivery grant, otherwise admission refuses
  it as a malformed charter. Its bounded result contract remains present
  and is still validated, but is never offered or priced as a finish schema.
- **Instructions:** the role, as text, the first part of main's system
  text (3.3): a chat, a researcher, a coordinator, a reviewer with a
  lens, an operator, a producer of a change. A host gives its own;
  smith's shell gives the task role, one text for chats and for tasks,
  when its settings give none (`../shell/settings.md`, sections 1.2 and
  2.6).
- **Brief:** titled sections of text, in order, that the host wrote for
  why the run runs, already fitted to the host's budget for them. A host
  that keeps typed context renders it; smith reads only titles and text.
- **Tools:** the families the LLM may call (section 5): inspect, modify
  and shell, on the workspace, if it has one; sub-agents; `deliver`;
  `save`, only with `deliver` (8.5); `wait`; and each host tool, declared
  (5.2). Data, never derived from a
  role's name. A run may have no tools but `finish`.
- **Result contract:** what counts as done (section 7).
- **Conventions:** where a workspace directory keeps its guide and its
  checks (8.1); smith's defaults when absent.
- **Budget:** what the whole run may spend across all its sessions, in
  the host's unit, with each model's prices; its wall time; its turns, a
  loop guard; and the reserve it keeps for finishing (`budget.md`).
- **LLMs:** the endpoint and model the main session runs on, and the
  others sub-agents may use, each with its prices, the window of input it
  may use and the most output a completion may have, as the host chose
  them within the agent's configuration (session.md, section 8). The
  agent is configured with endpoints; a charter only names them.
- **Waiting:** how long the run may wait for a message before it parks
  (section 6), its first message included (3.4); zero parks as soon as it
  waits. A run is *wait-granted* when its charter's tools include `wait`,
  whatever its waiting time: a headless host grants `wait` with a waiting
  time of zero, so a run that would wait parks at once and its host
  reports that it needs input (host.md, section 8).
- **Resuming:** whether the main session opens from the transcript in
  the start, when there is one.

### 3.2 The start

What comes with the charter:

- **The workspace,** if any (tools.md, section 2): its directories, each
  with the name the LLM calls it (one safe path component), where it
  sits, whether it may be written and whether it is a git repository; and
  for a repository that starts from a merge in progress, the files the
  merge left in conflict (8.3).
- **Messages,** if any: those that triggered the run, such as a person's
  first line or a headless run's prompt, each named and labelled like any
  message (section 6). They enter the inbox when the start is admitted,
  before any message relayed after it, and a start refused at the
  entrance refuses them with it.
- **The run's identity:** opaque and of the fixed size the vocabulary
  seals (protocol/limits.md, section 2.4), the same for every
  activation and resume of the run, which its host keeps with the run.
  A start without one takes the identity its transcript records (the key
  of main's affinity), so a minted identity is also the run's for its
  whole life. Only a start with neither, a run's first activation without
  a host identity, gets one the run draws from its seeded randomness as
  it admits the start. Every conversation of the run carries it, with an
  ordinal of its own (main's is zero), as its affinity (session.md,
  section 2), so a provider can route the run's requests to where their
  prefixes are cached; a main session resumed from a transcript keeps the
  affinity its transcript records. It is never derived from a credential
  or a path.
- **The transcript,** when the run resumes, and the calls its host
  answered after the last of its turns, with their answers (section 6).
- **Credential grants** for the endpoints that need them (host.md,
  section 7).
- **An activation number,** new for every start of the run and never
  reused; every call's name carries it (5.2).

More messages may follow at once, before the run has read its start.

### 3.3 The prompt

The run composes its main session's system text: the instructions, the
role (3.1), first; the brief's sections; then the sections about its
own mechanics, because those are what it enforces: the workspace, if
any, and what it may write; the tools; each directory's guide; the
checks; the budget (`## Budget`, `budget.md`, 2); sub-agents, when granted (5.3);
how to finish in Finite (the contract, as the LLM must meet it), or how to
reply in Open; and how waiting works, when `wait` is granted. The domain decides what each says and in which order, and
writes it as text, as it writes the results and problems of the run's
own tools and the host's, and the notes it appends (`budget.md`, 2), so a
transcript keeps exactly what the LLM was told. The workspace tools'
outcomes stay typed (tools.md, section 4).

- **The working directory, named.** The workspace's section says
  concretely where paths start (tools.md, section 2). With one directory,
  it is the working directory: paths are relative to it, and its name
  appears in no path the prompt shows. With several, the first is the
  working directory, and the prompt says how each other is reached, as
  `/name/...`.
- **No workspace, no word of one.** A run with no workspace has neither
  the workspace's section nor the section on its tools that act on it:
  nothing it could read, write or run is named, so that its system text
  is its instructions, its brief and the mechanics it has (its budget,
  how it finishes or replies, and waiting), as a run that only talks or
  reads through host tools needs (the user, 2026-11-07).
- **Fixed for the conversation's life.** The system text is written once,
  when the run opens a session, and does not change while that
  conversation lives, across its windows (session.md, sections 3 and 8):
  it holds no clock and nothing else that moves. What moves is appended
  to the conversation as text the run wrote (a nudge, a sub-agent's usage
  line, the wind-down's note, a message), never placed in the system text
  or a tool's description, so each request extends the one before and a
  provider's cache keeps it. A session that resumes the conversation in a
  later activation has it written afresh, from the start that resumes it.
- **The summary instruction and its preamble.** With each session it
  opens, the run gives the instruction the session sends when it
  compacts its conversation into a new window, and the preamble that
  opens the summary in that window (session.md, section 8.2). The
  session decides when; the run owns the words, as it owns every
  prompt. The instruction must say:
  - that the work is not finished, and that a new window continues it,
    opening with the task's own words;
  - to answer with text and to call no tool;
  - to write what the next window needs that the task does not say: what
    was done and found, with exact values, names and paths as they were
    read; what remains; and the decisions taken;
  - not to restate the task or its required answer;
  - to keep within 1,000 words, since a summary is generated at the
    model's output rate.

  The preamble says that what follows is the model's own summary of an
  earlier window, and that the work goes on. Both are the same whatever
  the charter says, so they count in smith's own part of a window
  (protocol/limits.md, section 3.9).

A guide is read from each workspace directory at the path the
conventions name, as text, UTF-8: whole within its share of the host's
part, at most the guide limit, which the product's presets size so that
a guide is normally whole (protocol/limits.md, 3.9), and past it cut at
a character boundary, the prompt saying that the file goes on; a file
that is not text counts as no guide.

A sub-agent's system text is its brief, then the same mechanics for the
tools it has, with the same guides: `## Your share` in place of
`## Budget`, and how to answer, within its answer cap, in place of how
to finish (5.3).

### 3.4 The opening

A run opens its main session once it has prepared and has work: a brief
that is not empty, or a message.

- **The opening** of a fresh conversation is main's first user message:
  the instruction to begin the work the brief describes, when the brief
  is not empty, then every message queued, in arrival order, at most one
  offer's worth (section 6); the rest stay queued for main's next yield.
  An Open run's brief is its person's first words, not work handed to
  it, so its opening carries the brief itself, whole, as that first
  user message, before the messages queued, and its system text keeps
  its instructions alone, with no instruction to begin: a model that
  reads a person's request as its own system text plans the work
  itself where its instructions would have it delegate. A Finite run's
  opening, a sub-agent's and a resumed window's are as above.
  When concrete history is selected for resume, the waking prompt contains
  an instruction that starts a fresh bounded activation and supersedes prior
  exhaustion, followed by the offered messages without a new instruction to
  begin. With no messages, that activation instruction stands alone. It first includes
  any host answers absent from the transcript (section 6). The session
  supplies this prompt as the result of a withheld `wait`, or as user text
  otherwise (section 6). The first turn told names the last offered message
  as read (section 6).
- **Awaiting.** A run with an empty brief, no message and `wait` granted
  opens nothing: no session, and no request to a provider. It tells its
  host it is waiting, and the first message opens main, as its opening.
  Its time awaiting counts against its wall time (`budget.md`), and it
  parks at its charter's waiting time like any waiting run, having taken
  no turn; its host starts it again with the message that wakes it
  (3.2). A run whose ordinary time runs out while it awaits parks then,
  as any waiting run does, having nothing to finish (`budget.md`, 1).
- **Instructions only.** A run with an empty brief, no message and no
  `wait` has only its instructions to go on, and opens on them, with the
  instruction to begin the work they describe.

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
   checks (8.1).
3. **Equips** its main session: prompt, tools, workspace authority, LLM,
   the run's identity, a share of the budget, the summary instruction
   and its preamble, and the transcript it resumes, if any; and opens it
   once there is work, or awaits its first message (3.4).
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
   message it read.
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
prompt tool (protocol/programs.md, 5.3). The charter's families say which of
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
(3.2). When the child ends, its final message is the parent's tool
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
  opens (protocol/limits.md, section 3.9).
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
  told, `answer_bytes`, the outcome's room (protocol/limits.md, 3.10), then
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
  workspace (8.1).
  - **Jobs give way.** A session's running jobs are its writes in
    progress (tools.md, section 5), so whatever takes the lease first
    cancels the running jobs of every other session, and waits for them
    to settle: a writing `sub_agent` call cancels its parent's jobs
    before its child opens, and the checks and the delivery cancel every
    running job before they run (8.1).
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
- **On a program backend** (protocol/programs.md, 5.10), the run serves
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

## 6. Messages, waiting, parking, resuming

- **Messages in.** A host relays messages to a live run as they arrive,
  each named, in order, with a sender's label and its text, which the
  host rendered; a start may carry the first (3.2). smith interprets none
  of them: a person's words, a delegate's result and a decision are all
  text to the LLM. The run admits each at its entrance or refuses it
  there, saying why: too large (its label and text as they will be
  rendered, against the run's message limits), the inbox full, a name
  still in use, or the run ending, its answer decided. Each refusal goes
  to its host, typed, with the message's name and the reason, and is
  that message's terminal (host.md, section 2). Host messages reach
  main only; sub-agents never receive them.
- **Host-requested finish.** Each message carries `finish_requested`, false
  for ordinary words. Only admission of a true message puts Open into the
  existing wind-down (`budget.md`, 1), waking Listening or Waiting. Start-carried and
  preparation-time requests retain this effect until main can act. The next
  successful text reply publishes its fence and parks; unread words retain
  their existing terminal. Refusals change no policy, and repeats never renew
  the allowance. Finite ignores the bit and processes an ordinary amendment,
  retaining its terminal tools. Cancel and all hard limits remain unchanged.
- **Offered in order.** Admitted messages queue in arrival order. The run
  offers them at main's opening (3.4), with a `wait`'s result, and
  whenever main yields: every message queued, in order, at most one
  offer's worth, a number of messages and of their rendered bytes that
  the run's limits give, sized so that a window's opening can hold an
  offer (protocol/limits.md). What an offer leaves stays queued, in
  order, for the next. A compaction carries the messages offered and not
  yet read into its new window (session.md, section 8).
- **The fence.** With each turn, and when it waits, the run names the last
  message read: its fence. Messages go in order, so one name covers every
  message before it. A message is read when a told turn holds it, so the
  transcript the host keeps has what was read.
- **One terminal each.** Every message the run accepts ends exactly once:
  - **read:** a told turn's fence names it or a later message;
  - **refused:** at the entrance, as above;
  - **unread:** the run answered without reading it, whether it was still
    queued or offered to a turn that was never told.

  The answer carries the final fence (section 9), so a host in any
  placement knows which went unread; what it does with them is its policy
  (host.md, section 5). No message is dropped in silence.
- **A finish crossing a message.** In a run whose charter grants `wait`,
  a `finish` while messages are queued is not accepted: the run answers
  the call saying that messages came, and gives the model the messages
  in the same request, so it decides with them in view. In a run without
  `wait`, a finish is judged as it comes, and what was queued goes
  unread.
- **A cancel is not a message.** It is the host's, and ends the run
  (section 9).
- **Waiting.** A run whose work depends on what comes next (a chat on its
  person, a coordinator on its delegates) calls `wait`. With messages
  queued, the call is answered at once, with them. Otherwise its result
  is withheld until a message arrives, and the result and the messages
  then go to the model in one request, so an exchange costs one
  completion. Meanwhile the run tells its host it is waiting, naming its
  fence, and its host holds it, the watchdog paused; the run is idle only
  when that fence covers every message its host sent (host.md, section
  4). Time spent waiting counts against wall time (`budget.md`).
- **Open text replies.** Main ending a turn with nonempty assistant text
  under an EndTurn stop has replied. Reasoning, metadata and refusal blocks
  do not count as assistant text. After the real turn is told, queued
  messages take precedence; otherwise the run tells Waiting with its read
  fence and listens for the charter's waiting interval. A new message
  continues main as a user message, once, without a tool result. No tool
  call or result is invented, and a successful reply spends no nudge.
  A zero interval parks immediately. Explicit `wait` can park silently;
  empty, truncated, refused and no-calls ends keep their unsuccessful-end
  policy (7.2). In the wind-down a successful text reply tells its fence
  and parks at once; queued unread words belong to a later activation,
  never to work beyond this budget.
- **Awaiting** a first message is waiting without a session (3.4): told,
  timed and parked the same way. It has read nothing, so its waiting
  names no fence.
- **Parking in Finite wind-down.** A granted main `wait` parks immediately,
  including on the last reserved completion. Queued words stay unread for the
  next activation; no further completion spends this activation's reserve.
  This does not grant wait to a child or bypass hard monetary or wall limits.
- **Parking.** Past its charter's waiting time it parks: every turn it
  took is sent, and it answers parked. A run whose ordinary time runs out
  while it waits parks too: it has nothing to finish, so it does not
  wind down (`budget.md`, 1). Its state is its transcript, which the host already
  has, so it hands over nothing else. A run that parks while its `wait`
  is withheld tells its last turn without that call's result. Resumed, it
  supplies that result in its first request, with the messages that wake
  it, as a live `wait` is answered, just as it supplies the answers its
  transcript does not hold (below; session.md, section 3).
- **Resuming.** A run whose charter resumes opens its main session from
  the transcript in its start (its current window, session.md, section
  8), its turns as they were, provider blocks
  included; tells the LLM, as text in its waking prompt, of the calls its
  host answered that the transcript does not hold as answered (one whose
  turn was never told, the agent having stopped between the two, or one
  the run withdrew and its host decided later), with their answers, so the
  LLM does not ask them again; and goes on with the messages that woke
  it, which its start carries (3.2). Each activation starts with fresh bounded
  turns and wall time; its waking instruction explicitly supersedes exhaustion
  and finish-now instructions retained from earlier activations. The host
  supplies only the task funding still available, so monetary spend stays
  cumulative and bounded across activations. Its waking prompt follows section
  3.4's resume form; it does not add a fresh begin instruction. A transcript
  it cannot use (another version, malformed, unresolved, too large) fails
  the run as transient, saying which (session.md, section 3), and the host decides
  what the next run starts from. A window that ran on another endpoint,
  dialect, account or backend is not refused: it is carried over
  (session.md, 8.2). A run that does not resume starts fresh from its
  brief, which carries what the host thinks it needs.
- **Messages mid-work.** Between
  completions, not only when main yields, the run offers its queued
  messages to the working session, which appends them after the results
  of the calls it just ran, in its next request; the turn that reads them
  is told with them, its fence naming the last. Nothing in flight is
  interrupted and nothing already sent changes, so the prefix holds and
  every rule above stands. The run's threshold notes on its budget take
  the same path, as text the run wrote, with no name (`budget.md`, 2).

## 7. Results

### 7.1 The result contract

A contract allows any of four forms:

- **A report:** text, bounded, with the fields the contract requires.
- **A verdict:** one label from a closed list, each label with a contract
  of its own: which fields it requires, and which items it may carry, how
  many (a least and a most) and of which kinds, each kind with the fields
  it requires.
- **A change:** the workspace's changes, delivered (section 8), with the
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
wait, is answered with the messages instead of judged (section 6).

An accepted report, verdict or failure ends the run. A change is
delivered first (section 8), and ends the run once its delivery lands.

### 7.3 What is checked twice

The run checks the form of a result; the host may check it again against
what it knows, as it commits it, and refuse it as a failed attempt for
its next run to hear of. smith does not make the host's judgement.

## 8. Delivering a change

### 8.1 Checks

- **A directory says what its checks are,** by convention: an
  executable at the path the conventions name, `.smith/check` unless the
  charter says otherwise (temper says `.temper/pre-pr`), looked up in each
  writable directory when the run starts. A directory without one has no
  checks, and nothing above the agent needs to know either way.
- **Checked is delivered,** but for a save, which runs no check (8.5).
  The checks run as contained processes with
  deadlines, and the delivery that follows is made, under the lease over
  the whole workspace (5.3): nothing else holds write authority from the
  checks' start to the delivery's answer, so the host makes exactly the
  tree that was checked durable. That holds by construction: a write runs
  alone in its session, a sub-agent lives only as long as the call
  that asked for it, and no job runs through a check or a delivery.
- **Jobs are cancelled first.** A `finish` that delivers, a `deliver` and
  a `save` cancel every running job, and wait for each to settle, before
  the checks start, or before the delivery when no directory has checks
  or the call is a save
  (5.3; tools.md, section 5). Only their own session's can be running:
  they run alone, and a child lives only as long as its call. A job so
  cancelled ends `cancelled`, which the next `job` call naming it
  answers.
- **A long operation is said.** The run tells its host when checks start,
  with their deadline, and when they end, as it does for each LLM
  completion, each command its sessions run, until it ends or is
  answered `running`, and each `job` call's wait, so a long test suite,
  a slow completion or a long build is not taken for a hang (host.md,
  section 4).
- **The checks' bound** is the charter's, where it names one
  (`protocol/charter.md`, section 2), and the run's `check_timeout`
  otherwise: each check is given that much from its start, or what is
  left of the call's own deadline when that comes first, cut at the
  run's end. A host that knows a run's work is large (temper, for a
  project's goal-sized landings) gives its charter a larger bound; the
  reserve still holds one run at `check_timeout` alone (budget.md, 1),
  so a larger bound's check in the wind-down is cut at the run's end.
- **Out of time, once more.** A check stopped at its deadline (it timed
  out: it neither exited, nor failed to start, nor was cancelled) runs
  once more at once, on the same tree under the same lease, given twice
  the time its first run was given, cut at the call's deadline and at
  the run's ordinary deadline, so the retry never spends the finishing
  reserve. The host is told the second check's start with its deadline,
  as the first's. There is no retry while the run winds down, nor where
  the cut leaves no more than the first run was given; and never a
  third. Each directory's check retries on its own as it times out; a
  check that fails otherwise refuses at once, as before. A check that
  ran out of time a second time, or could not be retried, is feedback
  as below, `timed-out`, with the time its last run was given: a check
  that only ran out of time is told as that, never as a failing check's
  exit.
- **A failing check is feedback,** with its output's tail, like any
  other. It names its directory and the check it ran there (the
  convention's path), how it ended, and the time it was given, for
  every exit: the checks' bound, or what was left of the call's
  own deadline when that came first. A stopped check keeps its output's
  bounded tail through the last byte read before its pipes close,
  including output read during stopping, as a check that exits does.
  An empty tail with no discarded bytes records no output read before
  the pipes closed.
- **Not a security boundary.** An agent can change what a check runs;
  whatever guards the host's systems (temper's CI and its authority's
  requirements) guards them. The checks catch failures early, inside the
  run that can fix them.

### 8.2 Delivery

`deliver` is the run's request to its host: make the checked state
durable, with the result's fields. The host takes exactly that state, in
each writable directory that changed, and does whatever delivering means
to it: temper commits and pushes a branch; the local host commits a
repository in place and keeps a plain directory's files as they are;
another host might copy files out or attach them to a ticket.

The host answers in smith's terms:

- **delivered,** naming per directory what it made, as text the run
  passes on (a commit, a branch's head, files kept);
- **nothing,** when no directory changed, which is how a declared
  change is checked to exist: the LLM is told its delivery failed;
- **refused,** for something the LLM can fix, naming it: a conflicted
  file that still holds a marker (8.3);
- **failed,** with a reason (unreachable, refused by the target, timed
  out, broken, too large, missing, busy, unavailable, cancelled, unknown)
  and the tail of the host's diagnostic output with the number of bytes
  dropped before it, within a cap the vocabulary seals
  (`protocol/limits.md`); with
  several directories, the first that failed in workspace order;
- **stale:** where the change goes has moved under the run, so every
  later delivery of this run would find the same. It ends the run as
  stale, and the host decides afresh.

The delivery value, `Delivered`, reports what it owns through
`owned_bytes()`; `owned_bound()` bounds its receipt cells and opaque
receipt bytes (programming-model.md, 6.3).

Every answer but delivered and stale goes back to the LLM. A delivery
that lands is the result, even if the run was ending meanwhile; a
delivery in flight is never abandoned (host.md, section 2).

### 8.3 Merges in progress

A repository may start from a merge in progress, the host having merged
and left the conflicts in the working tree; the start names the files in
conflict, and the run tells the LLM. It also tells it what a merge in
progress holds, before it looks: `HEAD` is its work so far, `MERGE_HEAD` the
tip the host merged in, and every other staged file is what that merge
brought, which it leaves in place. The LLM edits the files in conflict and
runs the checks; it has no git writes (tools.md, section 5). Delivered, the host
commits the tree as the merge, and refuses while a file the merge left
in conflict still holds a marker, naming it.

### 8.4 Delivering mid-run

A run whose charter grants `deliver` may deliver before it finishes, as
a chat does with a small fix before handing it on: checked, then
delivered, the same way, and the run goes on. A finish with a change
delivers what is there at finish. A mid-run delivery is not the run's
result: a run ended while one is in flight waits for its answer, then
answers as it was ending; the host has the delivery's own answer.

### 8.5 Saving unchecked

A run whose charter grants `save` may hand its tree to its host at any
time with no check run, so that work in progress is kept before the run
asks a question and waits (the owner, 2026-11-09). A host that keeps
work on a branch has the run's tree there to read while the question is
open.

- **Granted only with `deliver`,** so only in a Finite run: a charter
  that grants `save` without `deliver` is refused at the entrance
  (charter.md, section 2). smith's local host grants none
  (../protocol/host.md, 5.5).
- **No input.** `save` takes an empty object: it carries none of the
  result's fields, since it is no result.
- **A delivery without its checks.** A save takes the lease over the
  whole workspace and cancels every running job, as a delivery does
  (8.1), and asks its host to make the tree durable at once, as a save:
  the host knows it from a delivery by its ask (protocol/channel.md,
  section 4), and marks what it makes as unchecked, in its own terms (a
  commit's message, a receipt's words). Its answer is a delivery's
  (8.2), told the LLM the same way; stale ends the run, as a delivery's
  does.
- **Never a result.** A save does not finish the run and is not its
  change. A `finish` with a change, and a `deliver`, run the full checks
  whatever was saved before them, so only a tree that passed them is
  delivered as checked. A host that merges a run's work merges what a
  checked delivery made, never a save alone.
- **Main's alone, and not in wind-down.** Like `deliver`, a save is an
  exclusive write of main's; a child is never offered it, and a run
  winding down offers it no more: a save asked while main is closing or
  the run is winding down is answered cancelled.

## 9. The answer

A run answers once, after every turn it counts, with its final fence:
the last message it read (section 6).

- **accepted,** with its result, its turn count and what it spent; also
  when the budget forced its finish, the result accepted in the wind-down
  (`budget.md`, 1);
- **parked,** with its turn count and what it spent;
- **failed,** typed so the host can act without reading prose: the model
  (a provider's failure past its retries, with its class, the endpoint
  and model of the last completion and how many attempts it made; an
  account limited, naming the account and, when its provider said, how
  long until it may be used again, measured when the run answers; an
  account refused, naming it and the provider's code; a program
  source's login lost, naming the account; a program that could not
  run or exited before its answer, naming the program, with its detail
  (protocol/programs.md, 5.6); a limit that fired, naming the limit and
  its
  bound; or a fault of the model's own answer, *refused*, *truncated* or
  *malformed*, when it kept refusing, stopping at its output limit, or
  asking for tools and naming none past its nudges (7.2), or when a
  compaction's answer gave no summary (session.md, section 8.2)), the
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
finish delivered meanwhile is still its result (8.4).

**A drain from the host** (protocol/channel.md, section 3) ends the run at the end of its current turn, rather than in the middle of it:

- **No new completion** of the main conversation starts once the run has heard it, a finishing one of a wind-down (`budget.md`, 1) included. A completion in flight goes on to its answer, and each call that answer makes runs to its result, a sub-agent's whole conversation and a check included, so the turn is whole.
- **Then the run ends as a cancel ends it:** the turn told, its sessions closed, what is in flight settled, and the answer *failed: cancelled*, with every turn it told and its fence. Messages that arrived meanwhile stay unread, named once as any are (`host.md`, section 4).
- **At once** when no completion of main is in flight: a run awaiting its first message, waiting, listening after a reply, or still preparing ends as a cancel ends it.
- **A turn that ends the run anyway** answers as it would have: a result accepted, a park.
- **A cancel after a drain** cuts the turn, as any cancel does. The drain's wait is the host's to bound: smith arms nothing for it.

## 10. Facts

Each child domain pushes what happened as typed, content-free facts into
an uncapped queue (programming-model.md, section 3), so no fact is lost
inside the process. The run exposes its queued count through
`pending_facts()` for its parent to drain, without an allowance for one
entrance. The service dispatches those facts before the next
domain entrance (protocol/limits.md, 3.8). Each carries when it happened,
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
  domain wrote (3.3), placed in each provider's request.
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

- **A silent check** (8.1): feedback carries only what its process
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
  LLM reads (3.3).
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
