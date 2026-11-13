# Agents and their hosts

Provisional, 2026-10-07, revised 2026-10-23. Where an application's
agents run, and what hosts them: the contract between the core and a
host, the three shapes an application may take, the agent as a
capability of two kinds, both smith's, jig's one host for runs wherever
their agents run, smith's payloads it carries, the application's
workspaces, and how a run's charter, tools, contract, messages and
delivery become smith's. The agent is smith, whose design is its own
(smith's `docs/design/`, cited by path: smith's `domain/run.md`, smith's
`protocol/channel.md`). The core's side of runs is ../engine/executors.md and ../engine/fleet.md
, and the LLM accounts that pay for them are ../accounts.md's. What is
still open is listed in `README.md`, section 10.

## 1. In one page

- **smith is the agent.** Nothing in smith assumes code, repositories or
  a workspace, so the same agent serves every application, told apart by
  its charters. jig builds no code into it.
- **The core needs a host for each run, not a worker** (section 2). A
  host starts a run with an assignment, relays messages and calls, keeps
  turns and the answer until the core acknowledges them, and hears how
  the run ends. smith's host contract carries the same (smith's
  `domain/host.md`, section 2).
- **Three shapes** (section 3): no agents; agents inside the engine;
  agents on workers. An application may mix the last two, a charter
  saying which hosts may run it.
- **One host** (`hub.md`): `jig-domain-host`, the hub of hosted runs,
  composed by every root that hosts runs: the engine's, when the
  application runs agents there, and a worker's. It knows a hosted run's
  lifecycle (admission, fencing, relays, turns and the answer kept until
  acknowledged, cancel) and nothing of how the agent runs, where its
  workspace is, or how it reaches the core. Those are its root's and its
  capabilities'.
- **The agent is a capability of two kinds, both smith's** (section 5):
  agent processes, `smith-host-domain`, one process per run; and
  `smith-inline-agent`, smith's domains in the host's own process. Both
  speak smith's parent face. The engine's hub has `smith-inline-agent`; a
  worker's has agent processes, or `smith-inline-agent` by its root's
  choice. jig ships no agent of its own.
- **The link to the core is the root's:** over a channel the worker
  dials, through its protocol layer; or, in the engine, straight into the
  core's fleet, linked from construction and lost only with the engine.
- **Workspaces are the application's** (section 6): a capability of the
  root that prepares what the connectors named, saves unfinished work,
  delivers what a run hands over, and says what the run left. jig defines
  the vocabulary; what a workspace holds is the application's. The engine
  has none, by policy: it runs no commands for its agents.
- **Neither the core nor the hub depends on smith** (`hub.md`, 8). The core
  keeps turns as bytes and its charter in its own terms, which
  `jig-protocol-charter` puts into smith's (`charters.md`); the hub carries
  smith's payloads as bytes, which its root hands out to its protocol
  layer to be decoded for the agent, or encoded from it.
- **A run does what a party does** (`charters.md`). The engine's host tools
  that act on tasks, `introduce` aside, speak a party's operations: a
  run's amendment and release name the revision it was shown, it
  proposes an action beyond its authority, explicitly, and it subscribes
  to the topics a connector offers to agents.
- **Every message ends once** (section 7): read, refused or unread. The
  start carries the messages waiting in the task's inbox; a refusal
  travels inside the run's next turn, or its answer; a message the run
  refused or left unread stays in the inbox and wakes its task again.
- **A run's text comes from a host whose agent gives it** (`hub.md`, 5): every
  host tells a run's activity, without content; a run's text, when its
  assignment asks for it, comes from any host, an agent process or
  `smith-inline-agent`; the calls a run makes, named as they begin, only
  from the second.
- **Committed turns survive.** Turns and answers are kept until the core
  acknowledges them, once committed, and sent again after every hello.
  What a host loses before it was committed (its turns, when a worker
  dies) is done again by the task's next attempt, which starts from the
  last committed turn and is told of the calls committed after it
  (../engine/executors.md, 1 and 3).

## 2. The contract

Between the core's fleet and a host, whatever its shape:

- **Down, from the core:**
  - an **assignment:** the run and its attempt; the workspace's items and
    its workstream key; the charter; when it resumes, the transcript's
    current window and the calls committed since its last turn (`hub.md`, 4);
    the start's messages (section 7); the run's identity; its grants:
    for each LLM account its check chose, its source's credential or a
    program source's name, and the application's credentials its
    workspace needs (../engine/executors.md, 1); and the **ask** for live content:
    one boolean per kind, `text`, `input` and `reasoning`, the first two
    set by the core for a run whose task is a conversation (../engine/executors.md,
    1), which the hub passes into smith's start (`hub.md`, 2, 5);
  - **messages** for the run, named, in order (../tasks/messages.md, 2);
  - **answers** to the run's relayed calls;
  - **grants** renewed while the run lives: each new generation of a
    credential it holds;
  - to a worker, the **program sources** it may be given, by name, after
    its hello and as one is added or removed; a holder's **check** of a
    program source's login; and a **sign-out** of one, which the worker
    runs (../accounts.md, 3.4 and section 5);
  - **acknowledgements** of turns and of the answer, once committed;
  - at most one **cancel.**
- **Up, from the host:**
  - a **hello:** the host's slots, how long it may take to stop a run it
    lost contact for, the workstreams it holds, the runs it hosts and
    those whose turns or answers wait for acknowledgement; and its
    agent's bounds, by which the fleet admits it (../engine/fleet.md):
    the limits a run's start must meet (smith's `protocol/limits.md`):
    the endpoints it serves, each with the largest window and output a
    run's models may take on it; the most turns, spend, time and waiting a run
    may have; its checks' timeout; the largest message a run takes, and
    how many its inbox holds; and how many turns it keeps unacknowledged.
    Its agent's bounds also name, if the application gives one, the
    longest host answer its agent takes, in bytes: its **reply bound**,
    by which the engine sizes what it answers this host's runs
    (../engine/fleet.md). It is the application's figure, never a
    limit a run's start must meet, and no reason to refuse the hello.
    A worker's program logins are not in its hello: they follow it, as
    `logins` (below);
  - **turns,** numbered within their attempt, each with its spend, the
    last message it read, whether it opens a window, and the messages
    refused since the run's previous turn, each by name with smith's
    reason (`hub.md`, 4);
  - **calls,** named by the run, each answered once;
  - **facts,** best effort: the run's activity; a run's live content,
    each piece named by its turn, retry, kind and offset, when the
    assignment asked for it and the host's agent gives it; and, from a
    host that has them, a tool begun with its call (`hub.md`, 5);
  - one **answer:** finished with its result, parked, or failed with a
    typed failure; with the turns it took, what it spent, the last
    message it read (the unread fence, section 7), the messages refused
    since its last turn, what it left on resources (the application's
    part), and how its save went;
  - **notices,** relayed from the run's agent: a source *limited*,
    with how long until its limit resets where its LLM provider gave
    that, or *refused*, and a generation *unauthorized* (../accounts.md,
    3.5; smith's `protocol/channel.md`, section 4);
  - **logins,** from a worker, after its hello: the state of each
    program source's login on it, as the check of the sources the engine
    sent answered (the fleet places no run on the worker until the
    first), then each one whose check answers otherwise than before, and
    a sign-out's outcome (../accounts.md, 3.4; `../../protocol/channel/README.md`,
    4.3);
  - **refusals:** busy, a duplicate among them (`hub.md`, 1), or invalid.
- **Fenced.** Every message names the run's task and attempt; the core
  drops what a stale attempt sends, and a host drops an attempt the core
  has finished with.

## 3. Three shapes

```
no agents           procedures and people only             no host, no smith, no LLM accounts
agents in-engine    the hub and smith-inline-agent         no workspace; tools are host tools only
agents on workers   the hub and agent processes            workspaces, commands, many machines
```

1. **No agents.** An approval pipeline, an operations runbook of
   procedures. The deployment configures no charters; a batch naming an
   agent executor is refused, and the fleet and accounts stay idle.
   Nothing links smith.
2. **Agents in the engine.** A triager, a researcher, a chat assistant,
   whose tools are the engine's and its connectors' reads. The engine's
   root composes the hub and `smith-inline-agent`, with no workspace
   (`README.md`, section 5 and `hub.md`). The costs:
   - no process containment, which is fine when nothing runs commands;
   - the engine's worst case includes its runs;
   - LLM traffic goes through the engine's protocol layer, on its loop.
3. **Agents on workers.** When runs execute commands, hold workspaces, or
   need many machines. Each run is its own agent process, so stopping a
   run ends its process tree. A worker's root may compose
   `smith-inline-agent` instead, for one person's machine: its runs are
   then in the worker's process, with no containment (5.2). Only an
   agent process runs a program backend, such as Claude Code on a
   person's plan, so a program's source pays only for runs on workers
   (5.1).

A charter says which kinds of host may run it, and the fleet places by it
(../engine/fleet.md). `ops` is the second shape; an application whose
agents change code in workspaces is the third.

One process may run the engine and its workers, for one person's
machine, development and tests, from the same parts
(`../../protocol/service.md`, section 10). Its workers' agents still run as
child processes, as between hosts, spawned by smith's host half; the
agent may be the application's own binary, running smith's agent shell
as a command of its own (section 8).

## 4. Structure

In the engine, when it hosts runs:

```
<application>-domain           the application's root
├── core                       jig's core
├── jig-domain-host            the hub: hosted runs on the engine's own slots
├── smith-inline-agent         the agent, smith's: its domains in this process, one per run in flight
└── …
```

On a worker:

```
<application>-worker           the worker's root: the application's; routes among its children
├── jig-domain-host            the hub: hosted runs, admit, prepare, start, relay, park or end
├── smith-host-domain          the agent, smith's: processes, spawn, channel, watchdog, cancel then kill
├── <workspace>                the application's: prepare, save, deliver, release (section 7)
└── <jobs>                     the application's job capability, if it has one: its kinds, run, stop (6.9)
```

A worker's root may compose `smith-inline-agent` in place of
`smith-host-domain` (5.2); nothing else in its tree changes.

Each tree follows skein/docs/foundation/programming-model.md, 4.5. The hub knows a hosted run's
lifecycle and nothing of the application's workspaces, of processes, of
channels or of the core, and links no smith crate; the agent, the
workspace and the job capability are capabilities, which the root translates to and from with
small total functions, the same functions for both kinds of agent, since
both speak smith's parent face, smith's payloads among them by hand-out
(`hub.md`, 8). A hand-off goes from a capability to the hub, from the hub to a
capability, and at most once more back and forth, so each entry point
completes them before it returns.

A worker's root admits new work by a watermark, with no store: a worker
keeps nothing durable (skein/docs/foundation/programming-model.md, 7.1).

- **The reference worker's backlog** counts the root's `hub_out`,
  `agent_out` and `pending`, and the service's `out`, `process_events`
  and `io`. These queues are uncapped. Its watermark defaults to 2,048
  entries and paces admission, not memory; one admitted event's work
  may take the backlog past it. The root reserves no room for outputs.
- **The waiting room** is the service's uncapped `events` FIFO, outside
  the admitted backlog; counting it would hold admission shut. A held
  new record lets admitted records pass it in order, until the next new
  record, a link record naming a held assignment's run and attempt, or
  any record of a held process record's agent. The walk stops there and
  keeps the remaining records in order. smith reads one record per
  process; a later record of that agent cannot pass its held record.
  Completions and hand-out returns for admitted work continue to drain
  the backlog. Only a real sink's credit stops their output.
- **Due source checks** are a separate new event, held at the watermark;
  the timer continues to advance admitted work without starting them
  (../accounts.md, 3.4).
- **A link's records follow the assignment they name.** `must_follow`
  prevents a link record naming a held assignment's run and attempt
  from passing it; a held process record also fences later records of
  that agent. The channel world holds the assignment order under load
  in `jig-channel-world`,
  `worker_orders::link_records_follow_the_assignment_they_name_under_load`,
  and the process fence in
  `worker_orders::a_held_call_never_acts_after_its_run_ended`; the
  reference service holds the fence in `jig-test-service`,
  `worker::admission_tests::a_process_record_stops_the_walk_behind_its_held_call`.
- **A run's end follows its last hand-out.**
  `payload_handouts_complete`, the absence of a pending hand-out for the
  run (`!hand_out_pending`), requires the final turn and result to
  return before the hub hears that the agent is gone. The root drains
  hub outputs before agent outputs, and hand-out returns follow their
  pending FIFO (`hub.md`, 8; skein/docs/foundation/programming-model.md, 7.2). The channel world holds
  this under load in `jig-channel-world`,
  `worker_orders::a_run_ends_after_its_last_hand_out_under_load`.

In the engine, the hub and `smith-inline-agent` are children of the
engine's root, beside the core: what the hub sends up goes to the core's
fleet, and what the fleet sends down for the engine's slots goes to the
hub (../root.md, section 5).

## 5. The agent

The hub's agent capability is one of smith's two crates. Both speak
smith's parent face (`smith_host_domain::parent`; smith's
`domain/host.md`, sections 4 and 9.4), its payloads typed (`hub.md`, 8): down, a
start (what the assignment gives the agent, with the workspace's
directories), messages, answers to calls, acknowledgements of turns,
grants, a drain (the run stops at the end of its current turn, `hub.md`,
7), a stop and a drop; up, in order, message refusals, calls and
withdrawals, turns, waiting, long operations, notices, the unread names,
one answer, and the agent's failures and end, typed. Both refuse a
message at the entrance, typed, with smith's reasons (section 7).

### 5.1 Agent processes

`smith-host-domain` runs each run as an agent process (smith's
`domain/host.md`, section 4). It is a worker's agent capability unless
its root chooses otherwise:

- **one process per run,** its process tree the run's containment
  boundary, its environment free of credentials, which come as grants on
  its channel. A program backend is the agent's child, inside that tree,
  started with its source's configuration directory and no credential
  variable (smith's `protocol/programs.md`, sections 2 and 4.2);
- **one channel** over the process's pipes, whose rules smith's host
  domain enforces, its host half in the worker's protocol layer
  (section 8);
- **a watchdog on progress,** paused while the run waits for a message,
  for a call's answer, or for room to send a turn, and stretched to the
  span of a long operation the run reports; a separate bound on wall
  time;
- **cancel, then kill:** a cancel down the channel first; past the host
  domain's grace, the tree terminated; past its `kill_after`, killed.
  Only an exited process and an empty tree say the agent is gone;
- **LLM completions** are the agent process's own io, with the
  credentials it is granted, or the program's, for a grant that names a
  program source (smith's `protocol/programs.md`, section 3);
- **the programs directory** is the worker's, private to its user, with
  one configuration directory per program source, named by the source,
  holding the program's own login and never a value of jig's
  (../accounts.md, 3.4). The worker's host half checks each login, at the
  engine's asking or its own interval, runs a sign-out, and reports each
  answer (section 2). A run's process tree, and a program's own tools,
  run as the worker's user and can read every login there, so until a
  run's tree is contained from the programs directory a worker that
  keeps a person's logins takes only runs that person's accounts pay
  for (../engine/fleet.md; ../accounts.md, 3.4).

A read of an account's list (../accounts.md, 3.7) takes the same queue: the engine
names an endpoint and a grant, and the root's agent capability has smith's host
half run the agent's one request (smith's `protocol/host.md`, section 2). It
takes no slot and no workspace, and its answer, the rows or a typed failure, goes
up with the read's number. The hub hears the grant's name alone; its value comes
from the worker's table, as a run's does. A link lost cancels it and sends nothing.

The hub's source machine owns when to check: on a new list, an explicit
check and a positive interval. An explicit check queues a fresh command;
requests for that queued command coalesce, while a command already running
finishes independently. Sign-out precedes a pending check.
Commands are serialized across the worker so one command owns the injected
resources and settles before reuse. Its whole first inventory waits for
every source's answer. The protocol layer drives the real host half's
command I/O and deadlines, with an explicit executable, programs parent
opened by the shell, the effective user and a bounded environment below
the root. A check first asks smith's host half to inspect both directory
levels without making anything or following links at either name. A
missing level answers not signed in without a child. An unsafe existing
level or another file error fails as unreadable, with detail, without
changing permissions or launching the program. Only two private levels
permit the status command; inspection uses the same deadline as that
command. Linked parents are accepted by the shell's parent open. Without
a configured program or an opened parent, the check fails as unconfigured.
The worker closes its parent descriptor when it closes.
These commands take no run slot. An unconfigured program counts as an
absent login: it cannot sign in or use a program login. A failed check after
an exit or timeout keeps withholding new placement until a later check
establishes absence, because a lost reply does not prove the login absent.
Shutdown reports an active sign-out's actual result once it settles; only a
queued sign-out that never started fails as closed. Channel loss suppresses
its old-channel reply, while the replacement channel starts with a fresh
whole inventory.

### 5.2 `smith-inline-agent`

`smith-inline-agent` runs smith's domains in the host's own process
(smith's `domain/host.md`, 9.1 to 9.4). It is the engine's hub's agent,
with no workspace, and a worker's by its root's choice:

- **a run is a composed smith domain,** one per run in flight, on the
  hub's slots, not a process: the contract crosses as typed values, with
  no pipe and no frames;
- **it hands its parent** what smith's host domain would, in the same
  order, through the same parent face, so the root translates both the
  same way;
- **a stop** is smith's cancel: the run winds down and answers. Past the
  inline agent's cancel grace, the run is dropped: its relayed calls are
  withdrawn, and what it holds below it is ended. The agent is gone once
  each has its terminal (smith's `domain/host.md`, 9.2);
- **requests below the run go up:** its LLM completions, and with a
  workspace the machine's operations, leave as requests that the root
  routes to its protocol layer, which runs them on smith's components
  with the grants' values (section 8), the run holding only their
  names. Their terminals come back to the run that asked, never through
  the hub;
- **content and facts** are drained from each run in the host's process
  (smith's `domain/host.md`, 9.1), and reach the hub, through the root,
  as a run's text and activity (`hub.md`, 5);
- **no watchdog on progress:** a composed domain's waits are bounded by
  its own deadlines and those of the requests it hands up; a bound on
  wall time stays;
- **bounded:** each run's worst case is smith's, and the inline agent's
  is its slots' sum, part of the root's;
- **no program backend:** it has no child of its own, so it runs the
  direct backend only, and the fleet gives it no grant naming a program
  source (../engine/fleet.md; smith's `protocol/programs.md`,
  section 2).

What it gives up is said in smith's `domain/host.md`, 9.3: no process as
the run's containment boundary. That is why the engine runs no commands
for its agents, and why a worker composes it only by its root's choice.

## 6. Workspaces

A workspace is what a run works in, prepared from the items the
connectors named (../connectors.md, section 10). Its kinds are the
application's: a capability of the root that composes the hub, speaking
jig's workspace vocabulary with the hub, through the root:

- **prepare** these items under this workstream key: answered ready,
  with what the agent is to be told of it (directories, names, what may
  be written, a merge to resolve), or failed, transient or permanent,
  naming the resource;
- **deliver** what the run hands over, from its `deliver` or its
  `finish` (section 7), checked, with its title and body, or from its
  `save`, unchecked, with neither (smith's `domain/run.md`, 8.5), where a
  checked delivery goes: answered with the delivery's outcome in smith's
  terms, and, for the engine, what it left on the resources. What marks
  an unchecked one is the application's;
- **save** unfinished work where the items say: answered saved, with
  where, or failed;
- **release** the workspace to its cache, or **discard** it when it may
  be damaged.

- **A cache, keyed by workstream:** the runs of one piece of work find
  their workspace ready. A workspace is reused only if it holds exactly
  the items named, and is reset to their starting point: nothing local is
  authoritative.
- **One operation at a time** per workspace, each bounded by deadlines
  given as data; the workspace arms no timers.
- **No credentials on disk:** what reaches a system names an identity,
  which the protocol layer maps to credentials per operation.
- **A run with no items** needs no workspace, and its host prepares
  nothing; a worker may still run it, for containment.
- **None in the engine,** by policy: the engine's root composes no
  workspace, and a charter whose runs need one is placed on workers only.

## 7. Messages and delivery

- **Messages** go to smith as messages, rendered by the same translation
  (`charters.md`): a sender's label (its requester, a delegate by number, a
  party by name, a subscription's topic) and the message's words, whole.
  Proposals and escalations waiting for the task are relayed the same
  way. What a decision about a task names travels with it: a notice of
  a delegate held, and an escalation, carry the hold's revision
  (`charters.md`); a question carries its kind and its number, the one an
  answer to it names (../tasks/messages.md, 1), and an answer the question it
  answers, so a run that relays or answers a question never guesses
  its number. The run names the last it read with each turn and when it
  waits. A proposal or an escalation relayed to a run and decided before
  its turn commits, often by a call in that very turn, still ends read
  when the turn names it (../tasks/proposals.md).
- **A grace asks the run to finish.** jig keeps `FinishRequested { revision }`
  as a durable inbox kind, separately reserved from merged amendments. Its
  live message and its carried Start message translate to smith's
  `finish_requested: true`; every other kind translates to false. The
  owned kind travels through held deliveries, without querying later task
  state. In an Open run, admission enters smith's wind-down, so its next
  text parks. Finite runs hear an ordinary amendment, and the flag changes
  nothing of cancellation, refusals or read fences. The hub carries the
  flag without linking smith; application roots only translate it.
- **The start carries the inbox,** as many of the messages the run is
  offered as the run's inbox takes, oldest first; the rest it is offered
  are relayed as the run reads (../tasks/messages.md, 2 and 6). The hub relays what
  the core sends it and chooses nothing. The run opens on them (smith's `domain/host.md`,
  section 2).
- **Every message fits** a run's message bound (../tasks/messages.md, 2), and the
  fleet admits no host whose agent takes less (../engine/fleet.md), so
  no run refuses a message as too large.
- **Each message ends once** (smith's `domain/host.md`, section 2):
  - **read:** a turn's last message read names it or a later one;
  - **refused:** by the hub or by the agent, by name, with smith's
    reason: too large, the run's inbox full, a name in use, or the run
    ending; the refusal travels inside the run's next turn, or its
    answer if no turn follows (`hub.md`, 4);
  - **unread:** the run answered without reading it, which the answer's
    last message read, the unread fence, says: every message relayed
    after it, and not refused, went unread.
- **Refused or unread, it stays** in the task's inbox, untaken, and
  wakes the task again when the run ends, as it would had it arrived
  then (../tasks/messages.md, 2 and 3). Nothing a host does drops a message.
- **Delivery** is the workspace's: smith's `deliver` is served by the hub
  through the application's workspace (section 6), which makes it. Its
  outcome is smith's (smith's `domain/run.md`, 8.2): delivered, nothing,
  stale, refused, failed. A `save` is served the same way, as an
  unchecked delivery with no title or body, one delivery in flight at a
  time with the run's others; its answer is a delivery's, which the
  worker wire and the store keep as they keep any (`../../protocol/store/records.md`,
  1), so neither changes. A delivery that was made is the run's result
  even if the run was winding down; what it left on resources reaches
  their connectors with the answer.

## 8. Below the domain

- **The engine's channel,** jig's: framed, dialled by the worker and
  authenticated, carrying the contract of section 2, versioned.
- **Agent processes:** spawning smith's agent in a contained process
  tree, with proof that the tree is empty after it stops; smith's channel
  over its pipes, `smith-channel` its codec, its host half
  `smith-host-protocol`, a component of the worker's protocol layer
  (smith's `protocol/host.md`, section 2). The agent may be a command of
  the application's own binary that runs smith's agent shell
  (`smith-agent-shell`).
- **Workspaces' io:** the application's.
- **In a host's process:** `smith-inline-agent`'s LLM completions and,
  with a workspace, the machine's operations, on smith's components
  (`smith-protocol-llm`, `smith-protocol-machine`) in the protocol layer
  of the process that runs it, each slot's result contract kept beside
  them (`../../protocol/llm.md`, section 3).
- **smith's payloads, by hand-out:** the protocol layer's routing under
  each root that composes an agent capability (`hub.md`, 8).

## 9. The world

- **The hub's world:** its root scripted; an engine that assigns,
  relays, cancels and acknowledges once its fake store commits; a link
  that drops for less and for more than the grace, and one that never
  drops, as in the engine; agent capabilities of both kinds, smith's:
  `smith-host-domain` over smith's agent processes over skein's fake LLM,
  which
  progress, hand up a conversation's text in pieces at their
  assignment's ask, call, take turns, some opening a window, wait, park
  and end,
  refuse messages and leave them unread, but also hang, crash, break
  rules and ignore cancels; and `smith-inline-agent`'s runs, smith's real
  domain over skein's fake LLM, which answer, wait, park and resume from
  their current window, stream text, are cancelled mid-turn, and ignore
  a cancel past the grace, to be dropped; and a scripted workspace that
  prepares, delivers, saves and fails, or none. It checks that every
  turn and answer reaches the engine once and is acknowledged once,
  every relayed call is answered once, every message relayed ends once,
  read, refused or unread, each refusal inside a turn or answer no later
  than the first whose fence passes it, nothing touches a workspace
  while a run's agent may, no piece goes up after its turn or for a kind
  its assignment did not ask for, and nothing is live once it settles.
- **The reference worker's orders** are checked under admission
  pressure: a link's records follow their assignment, and a run's end
  follows its last hand-out. A low-watermark service test checks that
  waiting records do not hold admission shut and resume as admitted
  work drains. Admitted records pass a held new record in order; the
  walk stops at the next new record, a link record naming a held
  assignment's run and attempt, or any record of a held process record's
  agent. Due source checks wait at the watermark while the timer
  advances admitted work (section 4).
- **The application's worlds** run its hosts, with its own workspace or
  none, against jig's core on the conformance world (../testing.md,
  section 5).

## 10. Open questions

- **A delegated task's text.** A task a conversation delegates streams no
  text for now (`hub.md`, 5). Later the conversation's run could ask for it from
  its parent, so the person sees the delegate write; whether that asks
  of the assignment, or of the views, is open.
- **Reasoning as it streams.** Its kind is in the wire (`hub.md`, 5), but smith
  has no producer for it. Agent processes produce input; reasoning needs
  each provider asked for readable reasoning.

- **Fairness on the engine's loop:** whether runs in the engine need a
  share of each iteration, so many chats never slow the engine's
  decisions.
- **Saving periodically,** so a dying worker loses less than the work
  since the run's last save.
- **Before the first channel:** whether a worker's failed dials, before
  it ever reached the engine, want a grace of their own.
- **The hand-out's cost in the engine.** smith's payloads cross the hub
  as bytes (`hub.md`, 8), so for runs in the engine some are encoded and decoded
  once more than they need to be: the charter, encoded by
  `jig-protocol-charter` and decoded again for `smith-inline-agent`; a
  result, encoded for the hub and read back by `jig-protocol-charter`.
  Each turn is encoded once, as the core keeps it anyway. Whether that
  ever shows, and if so whether the hub carries tokens its root
  resolves.
- **Workspaces in the engine.** Whether a deployment may give in-engine
  runs a workspace, for one person's machine, knowing it has no
  containment; the mechanism would allow it.
- **Containing a run from the programs directory,** so that a worker
  keeping several people's program logins may take each one's runs:
  ../accounts.md, section 7.
- **A failed login check's restriction:** whether a check that failed
  after an exit or timeout (5.1) withholds new placement only from the
  runs its source's holder pays for, by a check joined with the holder's,
  rather than from every run on the worker.
- **Agents other than smith:** the contract of section 2 is jig's; an
  agent that spoke it through a capability of its own could run beside
  smith. Not wanted yet.
- **What a root outside the charter shares.** `hosted` takes jig's tools and
  jig's charter, so a root with its own keeps its own hand-over. Whether the
  parts of it that are no application's, the answered calls and the start's
  messages, come apart from the charter, so that such a root shares them and
  keeps its charter, workspace and message translations, is open
  (`jig/docs/development/debt/D004-ops-root-jig-wiring.md`). Default: it
  keeps its own.
- **A message's rendering allowance.** A message admitted at
  `tasks.message_bytes` reaches a run with its sender's label, kind and
  number before its words (section 7), and the host and smith measure
  all of it against their own bounds (`hub.event_bytes`, the run's
  message bound), so a message admitted at the bound is refused by the
  run, against ../tasks/messages.md 2's promise. The likely answer: words and
  questions admitted against the message bound less an allowance the
  application declares for its rendering (label, kind, number and
  smith's separator), with the host's, the fleet's and the agent's
  bounds checked against it at configuration.
