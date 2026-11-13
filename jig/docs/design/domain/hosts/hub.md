# The hub

`jig-domain-host` hosts runs on a fixed number of slots: a worker's, or
the engine's own, a configured number. It is the same hub in both
places; what differs is its root and its capabilities. One hub, rather
than one per shape, buys:

- **one lifecycle:** admission, fencing, relays, turns and answers kept
  until acknowledged, and cancel are written, reviewed and tested once,
  in one world, with smith's agent processes over skein's fake LLM and
  with smith's real domain; the open questions of `README.md`, section 10 are answered once for
  both shapes;
- **one root shape:** the engine's root and a worker's compose the same
  three kinds of child: the hub, an agent capability, and a workspace
  where there is one;
- **one process when wanted:** the engine and its workers share a
  process (`README.md`, section 3), the workers' agents still child processes, with
  no code of their own.

## 1. The link to the core

- **On a worker, the worker dials the engine** and keeps one channel
  open, through its protocol layer. The engine never connects to a
  worker, so a worker can run wherever it can reach the engine
  (`../../protocol/channel/README.md`, 9.1).
- **In the engine, the link is direct:** the engine's root connects the
  hub to the core's fleet from construction, with one hello, and it is
  never lost. A restart settles its claims at once (../engine/restart.md
  ); its slots never dial in.
- **Hello first,** as `README.md`, section 2 says, on every link. Facts go only after
  it.
- **One vocabulary toward the link,** `README.md`, section 2's whole, on a worker as
  in the engine: the hello as one value, which a worker's root assembles
  from the hub, its workspace, its agent capability's bounds and its
  link's graces; notices relayed from the agent; facts as outputs, best
  effort, as well as by query. The values are one crate's, `jig-domain-link`, which
  the hub and the core's fleet share, so the engine's root passes its own
  hub's link to the fleet unchanged, as a worker's protocol layer decodes a
  worker's into the same values (`../../protocol/channel/README.md`, section 1); its
  hello states the bounds of its `smith-inline-agent`'s configuration.
  In the engine, `jig-domain-hub-link` is that link: it keeps the hub's assignments, briefs, payload hand-outs and relayed calls beside the fleet's claims, and the root composes it and routes to it.
- **The link vocabulary names only what crosses.** It names a run and an
  attempt by their numbers, and a call, a message or a turn by the name
  or number the run or the engine gave it, which mean the same on both
  ends. A handle one side mints for itself (a delivery's token, a
  request's reply handle) is not in it: an assignment is answered by its
  run and attempt, and a root that wants a handle of its own keeps it
  beside them.
- **A relayed call is named by its bytes both ways,** the run's name for
  it as its root had it encoded (`../../protocol/channel/README.md`, section 6),
  its answer and its cancel included. A delivery's token is the hub's own
  handle and never leaves it, so neither a worker's link nor a root that
  composes a hub, the engine's own included, keeps a table from names to
  tokens.
- **Turn busy is not in it.** The engine's answer of busy to a turn is
  the fleet's word to what keeps the backoff below the hub, which arms no
  timers (2): a worker's link, or `jig-domain-hub-link` for the engine's own hub, on the timers of the layer that holds it. At the backoff's deadline it asks its root
  for the hub's kept copy of the turn, which goes up again as any turn
  (`../../protocol/channel/records.md`, 2).
- **The engine holds the claim.** It commits a claim before assigning a
  run, and decides what follows: commit its result, run it again on any
  host, or hold its task. The hub never retries a run and never decides
  that work is done.
- **Attempts are fenced.** Once the engine has cancelled or reassigned an
  attempt, whatever it still sends is dropped; on the hub, a cancelled
  run's calls are answered as unavailable, and an attempt assigned again
  while the hub hosts it is dropped. A fresh assignment for an attempt
  already hosted is refused as busy.

## 2. Hosted runs

```
admit ─► prepare ─► start ─► active ⇄ waiting
                               │  turns: kept until acknowledged
                               │  calls: delivery served here; the rest relayed to the engine
                               ▼
                         park or end ─► stop ─► save ─► release ─► answer
cancel, from any state ──────────────► stop ─► save ─► release ─► answer
```

1. **Admit** the assignment, or refuse it at the entrance: busy (every
   slot taken, an answer not yet acknowledged keeping its own; the run
   hosted under another attempt; the host shutting down) or invalid
   (beyond the limits).
2. **Prepare** the workspace from its items (`README.md`, section 6), if the run has
   any and the root has a workspace.
3. **Start** the agent through the agent capability (`README.md`, section 5), with
   what the assignment gives it, smith's payloads decoded by hand-out on
   the way (8), and its ask for live content, which goes into smith's
   start as it is (5).
4. **Relay** while the run is live:
   - **messages** that arrive after the assignment go down as they
     arrive, each named, in order, a bounded few held until the run is
     live; one the hub cannot hold, or that comes as the run ends, it
     refuses, full or ending (4). One that arrives once the answer is
     sent is past its fence, and so unread;
   - **turns** come up as the run ends each, kept until acknowledged
     (4);
   - **calls** come up, a bounded number per run and one delivery at a
     time; deliveries are served by the workspace (`README.md`, section 7); every
     other call goes to the engine and its answer comes back. A call the
     run withdraws past its own deadline is answered as withdrawn, and
     may be asked again by the same name once its relay has ended;
   - a run that **waits** holds its slot until its next message, naming
     the last message it has read.
5. **Park or end,** as the run decides. A run parks only once every turn
   it took is sent; its transcript is its state.
6. **Stop:** answer the run's relayed calls as unavailable, and wait until
   the agent capability says the agent is gone (`README.md`, section 5), and until a
   delivery in flight has settled: a delivery cannot be abandoned half
   way, and what it made counts. Nothing touches the workspace while
   anything of the run is still running.
7. **Save** unfinished work, if the assignment asks: at park, at an
   unfinished end and at cancel, where the workspace's items say
   (`README.md`, section 6).
8. **Release** the workspace to the cache.
9. **Answer** the engine, exactly once (`README.md`, section 2). The engine decides
   what happens next, and its acknowledgement frees the slot. A start
   refused, at the entrance (step 1) or by the agent capability (3),
   is answered once as refused: the hub frees what it took for it at
   once, keeps no copy and lists it at no hello, and the fleet closes it
   as it hears it, never acknowledging it (../engine/fleet.md).

A run with no workspace skips preparing, saving and releasing. A cancel
from the engine, a lost link or shutdown, a drain's turn end (7), or a
fault of the agent, takes the same tail: stop, save, release, answer. A run that says how it
finishes as it winds down has that as its answer; only a run that says
nothing is answered as the host stopped it. The hub arms no timers: each
wait is bounded below it.

## 3. Failures

Typed, so the engine can act on them without reading prose:

- **refused** at the entrance: busy; or invalid, beyond the hub's
  limits, with smith's parts that do not decode (8), or as a start
  smith refuses, naming what (`charters.md`); a start the agent refuses for
  want of room is refused busy, as at the entrance (`charters.md`); or
  for a version, a charter, a turn or the application's items carrying
  a version this host does not read, naming the wire, that version and
  the range it reads (`../../protocol/channel/wires.md`, 6);
- **preparation failed:** transient (the system could not be reached,
  the worker's side failed or ran out of time, another run of the
  workstream holds its workspace), or permanent until something changes
  on a system, naming the resource the workspace says. The engine counts
  a transient one, a run's or a job's, as an infrastructure failure
  (../tasks/lifecycle.md, 6), its detail the task's reason, cut to its
  last `REASON_BYTES` as a failed job's is (9), so that its hold names
  what failed; a permanent one as permanent;
- **the run failed,** as it reports it (smith's `domain/run.md`,
  section 9), a failure of its model naming the model call where it can
  (below), or because the source of an LLM account it needs turned
  limited, refused or in need of a sign-in, naming the account, which
  is no failure of its task's (../tasks/lifecycle.md, 6; ../accounts.md, 3.5);
- **the agent failed:** it could not be started, it exited or was dropped
  without answering, it broke the channel's rules, or the watchdog or its
  bound on wall time stopped it;
- **cancelled:** by the engine, for lost contact, or for shutdown; for
  lost contact or shutdown, an interruption, never a fault of the run's
  (6).

Each carries bounded detail for operators; none of it is shown to an LLM.

An agent fault may arrive before admission. While starting, the hub keeps
its first typed fault and waits for the agent's `Gone`, without acknowledging
admission, delivering held messages or sending a cancel to an agent token
it has not received. `Gone` then answers an agent failure with that fault
and the bounded diagnostic tail, refusing held messages as ending. Without
a fault, disappearance while starting remains an unstarted agent failure.
A cancel decided before the fault keeps its cancellation outcome. Smith
refuses admission after a fault; a later `Started` cannot admit that run.

**Unusable resume history** retains its own typed reason. A transcript
that cannot be decoded, a start Smith refuses for its transcript, and a
run reporting a transcript failure all end as a transient task failure.
The fleet closes these as any refused start (2; `../engine/fleet.md`),
never acknowledging them; the task's terminal is the transient failure.
The next preparation starts fresh with the transcript tail when
readable and keeps unread inbox messages (`../engine/executors.md`, 2, and `../engine/briefs.md`, 1);
only such history closes the window, and every other failure leaves
the next attempt to resume it;
an unread tail does not spend another transient try. It does not retry the same
history as an invalid start or a model failure.

**A configured bound** is `RunFailure::Exceeded(Exceeded)`, where
`Exceeded { bound: Bound, value: u64 }` names the bound and its configured
value in that bound's unit, never a measured size. The payload is `Copy`
and inline. For `History`, the value is the ceiling met, whose unit the
tag does not identify. Its bounds are:

| Bound | What is bounded | Unit |
|---|---|---|
| `Reasoning` | the model's reasoning, per item or in all | bytes |
| `Metadata` | the model's other replay metadata | bytes |
| `Items` | the items of the model's answer | count |
| `Answer` | the model's answer | bytes |
| `Tools` | the tools offered to the model | count |
| `HistoryItems` | the messages and blocks one request carries | count of both |
| `History` | retained history that compaction could not bring under a configured ceiling | messages, blocks or bytes; the tag does not identify which |
| `Window` | the model's window | tokens |
| `Output` | the model's declared output ceiling | tokens |

The tasks child holds this failure at once (../tasks/lifecycle.md, 6). It is not an
account wait and is not rewritten to an account Recheck. The host, tasks,
people and client domains each own their `Exceeded` and `Bound`; every
translation between them is exhaustive, without a wildcard.

**A model call's failure is named,** by structured fields alone, so
that the engine can class it and a hold can say what failed without
any prose. A run that failed for its model (`RunFailure::Model`)
carries, beside that failure, the model call whose failure ended it,
where its application can name one:

```rust
pub struct ModelFailure {
    pub endpoint: Box<[u8]>,     // the endpoint's configured name, as the run's charter names it
    pub model: Box<[u8]>,        // the model's name, as the run's charter names it
    pub kind: ModelFailureKind,
    pub evidence: ModelEvidence,
    pub status: Option<u16>,     // the HTTP status the call's answer carried, where one came
}
impl ModelFailure {
    pub const NAME_BYTES: u32 = 40;     // each name's bound
    pub fn outage(&self) -> bool;       // the provider unreachable (below)
    pub fn words(&self) -> Box<[u8]>;   // its words (below)
}
pub enum ModelFailureKind {
    Unavailable, Overloaded, RateLimited, Exhausted, TimedOut, ContextTooLong,
    Invalid, Refused, Unsupported, Unauthorized, Protocol, Cancelled, Limit,
}
pub enum ModelEvidence { Unsent, Unknown, Response }
```

- **Where it rides.** `jig-domain-host` owns the three types. The agent
  capability hands it to the hub in `Finish::Failed { failure, model }`,
  and the hub's answer carries it in `Ending::Failed { failure, detail,
  work, model }`, to the engine on the channel (`../../protocol/channel/records.md`,
  1). `model` is `Some` only beside `Failure::Run(RunFailure::Model)`,
  and `None` for every other failure, and for a model failure its
  application names no call for (a provider failed for good, a model
  that kept declining, a program that could not run). `RunFailure` and
  `Failure` stay `Copy`.
- **What it names.** Each name is at most `NAME_BYTES`, cut by its
  application where longer; neither is ever a provider's text, a body,
  a diagnostic or a tool's output. The kind is the provider's failure
  class, one to one, a local limit's being `Limit`; the evidence says
  whether the call's request went out (`Unsent`: no call reached the
  model), may have (`Unknown`), or was answered (`Response`).
- **Its words** are `<endpoint> <model>: <kind>, status <status>,
  <evidence>`, the status part only where one came, and each name's
  bytes outside printable ASCII written `?`:
  `anthropic claude-opus-5-5: unavailable, status 403, unsent`. The kinds
  are written `unavailable`, `overloaded`, `rate limited`, `exhausted`,
  `timed out`, `context too long`, `invalid`, `refused`, `unsupported`,
  `unauthorized`, `protocol`, `cancelled` and `limit`; the evidence
  `unsent`, `unknown` and `response`. They are at most 123 bytes, within
  the tasks child's `REASON_BYTES` (128).
- **An outage** is a failure of kind `Unavailable` or `Overloaded` whose
  evidence is `Unsent`: the provider could not be reached, and no call
  reached the model. The engine counts it an infrastructure failure
  (../tasks/lifecycle.md, 6), as `End::FailedWithReason { class:
  Infrastructure, reason }`, its words the reason. Any other named
  model failure, one answered (`Response`) or perhaps sent (`Unknown`)
  among them, is the run's own: `End::FailedWithReason { class: Run,
  reason }`, its words the reason. A model failure with none named is
  `End::Failed(Run)`, as before. So the retry notice and, past the
  class's tries, the hold name the call, and nothing a provider echoed
  reaches their readers.

## 4. Turns

- **A turn** is the bytes of one turn of the run's conversation, in
  smith's vocabulary (smith's `domain/session.md`, section 3; smith's
  `protocol/transcript.md`), encoded by hand-out (8), numbered within
  its attempt, with what it spent, the last message it read, and whether
  it opens a window: a window turn, or a conversation's first. The hub
  reads only its number, its size, its spend, that name and that flag,
  which come beside the bytes. A resumed run is given its current
  window's turns only (../engine/executors.md, 2; smith's `domain/host.md`,
  section 6).
- **Refusals ride with turns.** A turn's fence covers every message
  relayed to its run before it, and takes only those: a message its run
  isn't offered, by its charter's parts or a pace, is never relayed, so no
  fence takes it (../engine/executors.md, 2; ../tasks/messages.md, 6). The core must know of a
  refused message before it commits a turn whose fence passes it, across
  an engine restart too. No
  refusal crosses on its own: the hub keeps each, its name and smith's
  reason, with the run's next turn, or its answer if none follows, and
  the core records them in the commit that takes the turn's read
  messages (../engine/executors.md, 2). An agent refuses a message only before a
  fence covers it (smith's `protocol/channel.md`, section 5), and the hub
  makes its own refusals before it relays a later message.
- **Kept until acknowledged,** like an answer: sent at once, sent again
  after every hello, and after a backoff when the engine answers it busy.
  A run may have a bounded number of turns, and bytes, unacknowledged;
  past it, the hub stops taking the run's turns, and the run waits, its
  watchdog paused.
- **The hub holds them, not the agent.** The hub acknowledges a turn to
  its agent capability as soon as it has taken it, and keeps it until the
  engine acknowledges it: for either kind of agent capability, the hub is
  the host that says a turn is safe (smith's `domain/host.md`,
  section 6). Its room is what it lets the agent send: past it, the hub
  takes no more turns and withholds the agent's acknowledgements, so the
  run waits.
- **Stopping never waits for the engine.** Once a run is stopping, the
  hub takes every turn the agent has sent, past its room if need be,
  since the turns the agent may leave unacknowledged are bounded by its
  start, so the agent can be gone (`README.md`, section 5) while the engine is out of
  reach, and the stop bound holds (6).
- **Before the answer.** The answer says how many turns the run took,
  and is sent after them.
- **Lost with the worker.** A turn not acknowledged when a worker dies is
  lost with the run; the transcript ends at the last turn committed, and
  the next run starts from there. In the engine, turns die only with the
  engine, and a restart starts from the store.

## 5. Facts

What the run reports as it goes (smith's `domain/run.md`, section 10)
goes up to the engine for live views, and, from agent processes, feeds
their watchdog. Forwarding is best effort: a bounded queue, what does not
fit dropped and counted, sent on an open link only, after the hello.
Nothing decides on a fact.

- **Activity, from every host,** carries no content: the run's phase;
  its long operations, each begun with what it is (its checks, an LLM
  completion, a command) and its span, and ended (smith's
  `domain/host.md`, section 2); and counts from smith's channel facts,
  each a kind, its time and a count (smith's `protocol/channel.md`,
  section 8), which the root decodes by hand-out before the hub sees them
  (8). Activity is the host's observation, fenced like everything a
  host sends (`README.md`, section 2). The engine shows a run as writing, or as
  running a tool, only on such a fact, never inferring it from an
  attempt being placed.
- **A run's text, at the assignment's ask.** A run whose assignment
  asks for text and input hands up, as best effort content, its main conversation's text and call being written as the
  model writes it, from any host: pieces, each naming its turn, its
  retry, its kind and its offset (below). In an agent process the pieces
  come up smith's channel, which the host's half decodes (smith's
  `protocol/channel.md`, sections 3 and 4); in `smith-inline-agent`'s
  host they are drained in the process (`README.md`, 5.2). A run whose assignment does
  not ask gives none: its words appear as each
  turn commits. A sub-agent's or a compaction's completion gives no text.
- **A tool begun with its call, only from a host that has it:** a hub
  whose agent is `smith-inline-agent` hands up a tool begun naming its
  call and its tool, with the call's input within the views' bound, so a
  run's document shows the call it is making. smith's channel carries no
  such record, so runs in agent processes give none; whether it is asked
  of smith later is open (`README.md`, section 10).
- **A piece names its turn, its retry, its kind and its offset,** whatever
  its source: the turn it belongs to, by its attempt and its number
  within it (an attempt is one activation of smith's run, whose turns
  smith numbers); the retry of the turn's completion, which smith
  retries within the turn, each retry's content starting again at zero;
  its kind: `text`, `reasoning`, or `input`, a tool call's input as it
  streams, named by the call's provider id and its tool; and the offset
  of its first byte within that retry's stream of that kind
  (`../../protocol/llm.md`, section 3), counted as smith counts it, in 64
  bits, in the hub's vocabulary and on jig's channel alike. Smith produces text and input from an agent process. Input names the
  call being written; it does not attest that a tool began. The root
  routes it to `RunInput`. Reasoning and offsets past the views' 32 bits
  are dropped and counted (../engine/views.md).
- **No piece after its turn.** A piece of a turn goes up before that
  turn's record; once the turn is sent, none of it follows, and a retry's
  content starts again at offset 0. The engine's views rely on it, and
  keep a run's text by turn, retry and offset (../engine/views.md).
  The hub holds to it, whatever its agent sends: it forwards a piece
  only from the agent of its run's current attempt, while that attempt
  is live (active or waiting), of a kind its
  assignment asked for, within its bound on a piece's bytes, and of a
  turn it has not yet taken from the agent; and it checks the last
  again as the piece leaves its queue, since its root drains facts at
  its own pace. It drops any other piece, counted apart from the pieces
  it had no room for.
- **Best effort, and gaps.** A piece goes up only while the link leaves
  room for everything else, as every fact does. In the hub, it waits
  with the run's activity in the same bounded queue, and takes room
  there only while half of it is free, so that a run's text never
  crowds out its activity; it is dropped and counted otherwise. smith
  holds a bounded amount of unsent content and drops the rest, counted
  (smith's
  `protocol/channel.md`, section 7). A drop shows as a gap, a piece
  whose offset is past the last one's end, which the views mark as cut.
- **One text, whatever its host.** Above the hub, the core's views, the
  client and the application see a run's text the same way from an agent
  process and from `smith-inline-agent`.
- **What jig asks of smith, later:** the LLM component's abort and
  closed terminal (`../../protocol/README.md`, section 11).
- **A conversation's delegated tasks give no text.** Only the run of the
  conversation's own task is asked; a task it delegates streams nothing
  (`README.md`, section 10).

## 6. Losing contact

On a worker; in the engine the link is never lost, and none of this
happens.

- **Runs go on for a grace** if the channel drops, and what they send
  waits, never dropped: turns, with the refusals they carry, and relays,
  unless their call has closed meanwhile. On reconnecting, the worker
  says what it hosts and the engine keeps or cancels each run.
- **The grace is the engine-away bound.** It is the application's to
  size so that it outlasts an engine's restart: the restarted engine
  adopts each attempt its store holds as claimed and the worker lists
  (../engine/restart.md, step 3), sends again what it had not
  acknowledged, and the run goes on with no new attempt. The clock
  stops only once the engine admits a hello, and the worker redials
  within it (`../../protocol/channel/README.md`, 9.1).
- **A loss is told, with why,** on each side, once, in its log: silence,
  stall, the stream, a refusal and its reason (`../../protocol/channel/README.md`,
  section 8). A healthy link is never lost for its load: each side reads
  it and keeps its pings whatever its root has yet to take.
- **A worker comes back as often as it is lost.** Nothing a lost link
  leaves holds its return: the engine closes the old connection within
  its stall deadline (`../../protocol/channel/README.md`, 9.2), the fleet frees the
  worker's position as it hears the loss (../engine/fleet.md), and each
  hello is taken as the first was.
- **Past the grace,** the hub cancels them itself, saving their work
  first, and keeps their turns and answers for the next channel. The
  answer, `Cancelled(Contact)`, is no fault of the run's, and neither is
  `Cancelled(Shutdown)` (7): the core ends the attempt interrupted,
  with no try spent and no backoff (../tasks/lifecycle.md, 2), and the next attempt
  resumes from its transcript and its saved work (../engine/executors.md, 2).
- **The stop bound** the worker says at its hello is the longest it may
  take to stop a run, counted from the last moment the engine may have
  heard from it, or from its own shutdown: the longer of the time its
  link takes to notice the loss with its grace after, and its drain's
  bound (7), then the longer of the time its agent capability may take
  to be gone after a stop (`README.md`, 5.1 and 5.2) and a delivery's
  deadline, then a save. A drain under a lost link ends at the grace's
  end, if that comes first, as any run is cancelled there, so the bound
  holds whichever comes first. The engine's grace, counted from when the
  engine notices the loss, is strictly longer (../engine/fleet.md), so no
  next attempt starts while this one may still write, however each
  side's link notices a partition (`../../protocol/channel/README.md`, 9.1).
- **It is kept, not only stated.** The root that composes the hub holds
  the timers (2), and keeps the stop bound as a deadline over the drain,
  every cancellation, delivery and save, and its agents' teardown: from
  its shutdown, the whole bound; from its grace's end, what is left of
  it after the notice and the grace. What a stop still runs at the
  deadline is abandoned: each agent's process group killed, each of the
  workspace's operations in flight cut, a save or a delivery among them,
  so nothing of the attempt writes after it. A run whose save was cut is
  answered as its save failed, with the saved work it named before; a
  job whose delivery was cut is answered as its delivery failed. A
  worker shutting down then exits, giving up what it keeps, counted
  (7).

## 7. Shutting down

A hub shuts down in up to two steps, which its root times, since the
hub arms no timers (2):

1. **The drain,** if its root asks for one. The hub admits no more, and
   asks each live run's agent, active or waiting, to stop at the end of
   its current turn (`README.md`, section 5; smith's `domain/run.md`,
   section 9): the run goes on until then, its turns, calls and messages
   as ever. A run not yet live, preparing or starting, is cancelled at
   once, having no turn to end. A drained run's end takes the cancel's
   tail (2): stop, save, release, and the answer `Cancelled(Shutdown)`,
   unless the run said in that turn how it finished, which is its
   answer. A job runs on (9): its work is never saved, so a job that
   finishes within the drain is not run again.
2. **The cancel** of every run and job still going, for shutdown: at
   the drain's bound, or at once for a root that does not drain. A run
   whose stop has begun goes on with it.

Either step says the hub drains, on its link, before anything its runs
send after it, and again after each hello while it shuts down, so that
the fleet places nothing more on it (../engine/fleet.md). The host is
done once the engine has every turn and answer. A worker gives up what
it keeps, counted, once no run is left and the engine is still out of
reach past the grace, or at its stop bound (6). A worker's root drains;
the engine's own hub has no drain, since the engine's stop settles its
claims at once (../engine/restart.md, 1).

## 8. smith's payloads

- **Bytes in the hub.** The hub links no smith crate. It carries as
  bytes it never reads the charter, a resumed run's transcript, each
  turn's body, call names and a run's result, and the run's identity,
  reading only what `README.md`, section 2 puts beside them. A host tool's input
  stays the bytes the LLM wrote, for the engine's protocol layer to
  decode (`charters.md`).
- **Typed in the agent.** smith's agent capabilities take and give them
  typed, through smith's parent face, never encoding or decoding them
  (smith's `domain/host.md`, sections 3 and 9.4).
- **Translated by hand-out.** The root that composes the hub with an
  agent capability, a worker's or the engine's for its own hub, hands
  each payload out to its protocol layer, whose routing decodes or
  encodes it with smith's codecs (`smith-protocol-channel`'s functions,
  and `smith-channel`'s records for a call's name and a fact), keeping
  no state,
  and hands it back as an event of the root, in the order it went out:
  the path of host tools' calls (`../../protocol/channel/README.md`, 5.3).
  - **Down,** the charter, the transcript and the committed calls'
    names are decoded, with the endpoints of the agent's configuration,
    which resolve those the charter names, and smith's start goes to the
    agent typed. One that does not decode fails the start as invalid
    (3).
  - **Up,** each turn, call name and result is encoded for the hub, and
    each of smith's channel facts is decoded into jig's typed activity
    (5), so the hub and jig's channel carry jig's activity, never
    smith's facts. Each live piece from smith's channel goes to the hub
    as jig's typed piece, and to the agent process's watchdog as its
    progress (`README.md`, 5.1).
- **Who links smith:** `jig-protocol-charter` (`charters.md`), a worker's
  agents part, `jig-protocol-worker-agents`, which an application whose
  workers run no agents leaves out (`../../protocol/service.md`, 11.2), a
  client's decoding of turns (`../../client/domain/client/conversations.md`),
  and an application's roots that compose smith's agent capability, with
  their protocol layers' routing, which also runs smith's components
  where it composes them. Never the hub, the core, or jig's channel's components,
  which pass smith's bytes through (`jig/docs/design/protocol/channel/README.md`, 5.1),
  translating them by smith's bridges, as bytes, between versions
  (`../../protocol/channel/bridges.md`, 5.3).
- **A call carries its charter's contract.** The worker's routing, as it
  decodes a start, hands the hub the version of the host's tools
  contract its charter names (smith's `protocol/charter.md`, 2), a
  `u16` the hub keeps with the hosted run and never reads; the hub puts
  it on each of that run's calls, and jig's channel carries it up
  (`../../protocol/channel/records.md`, 1), so the engine answers the
  call by the contract the run was written for, whichever engine wrote
  its charter (temper's `protocol/wires.md`, 7).

## 9. Jobs

A job task (../model.md, 3.2) is hosted as a run is,
with no agent:

- **Its assignment** is a run's but for the agent's part: the job's
  kind, a number the application gives; its spec, the application's
  bytes, carried and never read; its workspace's items and workstream;
  its deadline; and the application's credentials its workspace needs,
  which the workspace keeps and the job is never given. No LLM account,
  charter, transcript, message or ask for live content.
- **Its lifecycle** is 2's with steps 3 to 5 replaced, and no save:
  1. admit; 2. prepare the workspace;
  3. **run:** the root's **job capability** runs the job of that kind
     in the prepared workspace, with its spec, until it ends or its
     deadline passes;
  4. **deliver** what the job made, through the workspace (`README.md`, section 7's
     delivery, asked by the job capability rather than by a run's
     call), when the job ended well; a delivery once begun settles, and
     what it made counts;
  5. **stop:** wait until the job capability says every process of the
     job is gone;
  then release and answer as ever. A cancel, a lost link, shutdown or
  a passed deadline stops the job at once, kills its processes, makes
  no delivery that has not begun, and takes the same tail. A job saves
  nothing: a job cut short is run again from the start.
- **On shutdown,** a job runs on through the drain (7), and the cancel
  for shutdown reaches it only after. The job's delivery is its one
  effect on a system, since the job has no credential: a job that
  cancel stops before its delivery began answers `Cancelled(Shutdown)`,
  which the core ends interrupted, spending no try
  (../tasks/lifecycle.md, 2). One whose delivery began lets it settle
  and answers as it ends, finished or failed as the delivery says,
  never cancelled, so `Cancelled(Shutdown)` always means that nothing
  of the job reached a system. A delivery the stop bound cuts (6) is
  answered as its delivery failed, or, by a worker exiting at the bound,
  not at all, its attempt then lost: either way a try is spent, as after
  any loss, and the next attempt recovers what it made.
- **Its answer** is finished, with a result of the application's bytes
  within the result's bound, or failed: the job's own failure (it said
  so), its deadline passed, a workspace that would not prepare or
  deliver, or the host's loss, each typed as a run's are (3), and
  retried by the core within the task's tries (../tasks/lifecycle.md, 6).
- **The job capability is the application's,** a capability of the
  worker's root beside the agent capability: it knows the kinds it
  runs and declares them in the host's hello, and the fleet places a
  job only on a host that declared its kind (../engine/fleet.md). It
  runs nothing the assignment did not name, and gives the job no
  credential: what reaches a system is the workspace's, by identity.
- **No turns, no relay, no calls:** a job reports nothing until its
  answer; the hub keeps that answer until it is acknowledged, as a
  run's.
- **A root with no job capability** refuses what the hub asks of one:
  that is its choice, and `ToJob::refused` spells the answer, a
  `StartRefused` to a run and a `Gone` to a stop.
