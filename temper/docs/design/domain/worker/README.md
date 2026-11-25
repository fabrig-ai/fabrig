# The worker

Provisional, 2026-10-04, revised 2026-10-29. What temper's worker does,
as a domain layer: its root, which composes jig's hub with smith's host
domain and temper's checkout; the items an assignment gives a run's
workspace; and the checkout, which is temper's own. The worker hosts the
runs of agent tasks (jig's `domain/engine/executors.md`). Its hub, a
hosted run's lifecycle, its failures, turns, facts and losing contact
are jig's (jig's `domain/hosts/README.md`); the agent it starts is smith's, as
../agent/README.md says, and the worker is smith's host (smith's `domain/host.md`).
The mechanics are those of skein's `../foundation/programming-model.md`. What is still
open is listed in `README.md`, section 10.

- `hosted-runs.md`: assignments, lifecycle and failures.
- `agent-processes.md`: the worker's agent processes and their bounds.
- `checkouts.md`: workspaces, their stores and holds, a workspace kept for its run, and the checkout's open questions.
- `checkouts-prepare.md`: checkout preparation, starting points and failure diagnostics.
- `checkouts-push.md`: delivery, saving and their open questions.
- `jobs.md`: the worker's job capability, its kinds, commands and failure diagnostics.

## 1. In one page

- **A worker is an execution host,** jig's third shape: agents on
  workers (jig's `domain/hosts/README.md`, section 3). The engine decides what
  work exists and keeps it in its store; the agent does the LLM work. The
  worker prepares workspaces, starts agent processes and keeps them in
  bounds, pushes the changes they make, and relays everything else
  between a run and the engine.
- **The worker keeps nothing durable.** What must survive lives in the
  engine's store (claims, transcripts, results) or on the forge (pushed
  branches, saved work). A worker can be killed at any moment and any
  worker can take any run; its checkouts are caches.
- **It hosts runs, not jobs.** A run may finish once, or wait for
  messages, take them and park (smith's `domain/run-messages.md`, section 1): it
  starts fresh or from its transcript, takes messages, makes calls, sends
  turns, and parks or ends (jig's `domain/hosts/hub.md`, 2).
- **Policy is data.** The worker interprets no workflow vocabulary: no
  task, role, verdict or tool names. An assignment says which
  repositories to check out and how, what to save, and the charter to
  hand the agent, which is smith's and which it carries as bytes.
- **Its one forge write is a git push:** a change its run delivered, or
  saved work. Push credentials stay in the worker's protocol layer; an
  agent never sees them. Everything else that touches the forge goes
  through the engine.
- **The checkout is temper's** (`checkouts.md`): git workspaces cached by
  workstream, a merge in progress as a place to start, a push that
  commits exactly the tree the run checked and is never forced, and no
  credentials on disk.
- **The rest is jig's and smith's:** admission, fenced attempts, turns
  and answers kept until the engine acknowledges them, losing contact and
  shutting down are the hub's (jig's `domain/hosts/hub.md`); the
  agent process, its channel, its watchdog and cancel then kill are
  smith's host domain's (smith's `domain/host.md`, section 4).
- **The domain is complete** (skein/docs/foundation/programming-model.md, section 4). A world of
  domains and fakes runs everything the worker does, with no protocol and
  no io (section 9).

## 2. The worker in the system

```
engine   jig's core, the forge connector, the store: what runs, with what; commits what runs say
worker   jig's hub: workspaces, agent processes, pushing their changes; relays the rest
agent    smith: LLM work, one run per agent process, reporting to the worker
```

- **The contract is jig's** (jig's `domain/hosts/README.md`, section 2). The
  worker dials the engine and keeps one channel open, through jig's
  worker link in its protocol layer (jig's `protocol/channel/worker-link.md`,
  section 1); it says its hello first; the engine holds the claim and
  fences attempts; the worker keeps turns and answers until the engine
  acknowledges them, once committed (jig's `domain/hosts/hub.md`, 1 and
  4). temper adds its parts beside jig's records: the workspace items
  down, what a run left on the forge up (`hosted-runs.md`, 1.1; `../../protocol/channel.md`,
  sections 4 and 5).
- **The engine never reads a repository.** It works from what the forge
  connector reads; what a checkout holds (a repository's checks, its
  `AGENTS.md`) is for the run to find (../agent/README.md, section 2).
- **The stop bound** the hello states is jig's (jig's `domain/hosts/hub.md`,
  6), with temper's push as the delivery and temper's save as the save:
  the longer of the time the link takes to notice the loss with the
  grace after it, and the drain's bound; then the longer of the time
  smith's agent may take to be gone after a stop and a push's deadline,
  then a save's commit and push. The worker's configuration derives it
  (`../../protocol/configuration/worker.md`, section 1); the domain takes
  it whole, with the drain's bound beside it, and keeps it as a deadline.
- **The stop bound is printed** at each start, once the engine's figures
  apply, on standard error after the figures' line: `stop bound S s,
  drain D s`, S the stop bound and D the drain's bound, in whole seconds
  rounded up. A service manager's stop timeout sits above it
  (`../../draft/archive/redeploys.md`, 6.1): temper's help gives the worker's
  unit `KillMode=mixed`, so that a stop signals the worker alone, and a
  `TimeoutStopSec` of the printed bound and a margin, past which the
  unit kills what is left, an agent or a command among them
  (`../../../help/server.md`, section 5).
- **Losing contact and shutting down** are the hub's (jig's
  `domain/hosts/hub.md`, 6 and 7). Past the grace the worker cancels its
  runs itself, saving their work first. Shutting down, it still says its
  slots and refuses every assignment as busy; it gives up the turns and
  answers it keeps, counted, once no run is left and the engine is still
  out of reach past the grace, or at its stop bound.
- **Told to stop, the worker drains** (`../../draft/archive/redeploys.md`, 6.1),
  a worker alone or a cluster's at its first termination signal, which
  its unit sends it alone (`KillMode=mixed`). At
  once it asks its hub to drain (jig's `domain/hosts/hub.md`, 7): each
  agent stops at its answer, the LLM's answer in flight committed and
  the calls it made cut after the drain's grace, `graces.drain_grace_ms`,
  the turn left open for its next attempt (smith's
  `domain/turn-parts.md`, section 4), or at the end of its current turn
  where the link carries no parts; and a run not yet live is
  cancelled. From that moment it keeps two deadlines. At the drain's
  bound, it cancels what is left for shutdown, a run still waiting for
  its answer among them, whose next attempt plans that turn again.
  At its stop bound, it abandons what its stop still runs: every agent's
  process group killed and every git operation in flight cut, so nothing
  of an attempt writes after it, then gives up what it keeps, counted,
  and exits, as at a second signal. Between, its runs end as cancels end
  them: each agent gone, its relays and deliveries settled, its work
  saved to its saved-work branch and its workspace kept for its next
  attempt (`checkouts.md`, *A workspace kept for its run*), the answer
  `Cancelled(Shutdown)`, so each attempt ends suspended with no try spent
  and its window open, and every turn, part and answer acknowledged
  before it exits. Its git
  component takes every save and push a run's end asks for until then,
  and no new preparation. A job runs on through the drain; cancelled
  after it before its push began, it ends interrupted too (jig's
  `domain/hosts/hub.md`, 9).
- **Asked over its link, the worker pauses or drains, and stays up**
  (jig's `protocol/channel/pause.md`; jig's `domain/hosts/hub.md`, 7,
  *Paused over the link*; `../../draft/archive/worker-restarts.md`, section 2).
  Paused, it takes no new run or job, and its runs go on. Draining, it
  takes nothing and drains as a stop does: each agent stops at the end
  of its current turn, a run not yet live is cancelled, and at the
  drain's bound, counted from the ask, what is left is cancelled for
  shutdown, a check in flight cut; each run's work is saved and it
  answers `Cancelled(Shutdown)`, an interruption with no try spent. Its
  git component takes every save and push a run's end asks for, and no
  new preparation. Once nothing is left and the engine has every turn
  and answer, it says it has drained, and stays linked, taking nothing,
  until it is told to take work again or stopped; its stop then finds
  nothing to drain. A stop while it drains is a stop: the stop's
  deadlines, from the signal, then hold alone. It writes a line at each
  change: `paused: no new run or job, as the engine asked`; `draining:
  each run stops at its turn's end, the rest cut in D s, as the engine
  asked`, D the drain's bound in whole seconds; `drained: no run or job
  left; a stop now loses nothing`; `taking work again, as the engine
  asked`. So a redeploy drains a worker, then stops it
  (`../../draft/archive/redeploys.md`, 7.2), and an operator keeps one from new
  work without a stop.
- **A signal to its agents is the worker's to decide.** Its agents run
  in process groups of their own, inside its unit. A termination signal
  one of them hears while its channel is open is told to the worker
  rather than cancelling the run (smith's `domain/host.md`, section 4),
  and the worker leaves the run to its own stop, writing `RUN: its agent
  heard SIGNAL from pid P; its stop is the worker's`, RUN the run as
  the worker's other lines name it, SIGNAL `SIGTERM` or `SIGINT`, and
  `from an unknown sender` where the system names none. So a stray
  signal, as a unit's whose `KillMode` signals every process in it,
  cuts no run; an agent whose worker is gone, its channel ended,
  cancels on its own as before.
- **Past the grace,** lost contact keeps the rest of the stop bound as a
  deadline from the grace's end, the longer of the agent's stop and a
  push's deadline, then a save: at it, each run still stopping or
  saving has its agent's group killed and its git operations cut, and
  is answered as its save or push ended (jig's `domain/hosts/hub.md`, 6).
- **A run loses at most a turn's tail** that the drain's bound cut, and
  what its saved work leaves out, ignored files and build outputs: its
  conversation, its committed calls and its saved work go on in its next
  attempt, on any worker (jig's `domain/engine/executors-transcripts.md`, 1;
  `../../draft/archive/redeploys.md`, 6.2).
- **In one process,** the worker is stopped after the engine has drained
  and settled its claims, so it does not drain: it cancels at once
  (`../../protocol/service-process.md`, section 1).
- **An engine's restart costs a run nothing.** The grace is the
  engine-away bound, `graces.grace_ms` in the worker's configuration
  (`../../protocol/configuration/worker.md`, section 1), sized to outlast an
  engine's restart. While it runs, the worker keeps hosting every run,
  keeps what they send, and redials no further apart than a quarter of
  the grace; it stops only once the engine admits a hello (jig's
  `protocol/channel/worker-link.md`, 1.1). The restarted engine holds that hello
  until its stored settings are restored, so its check is the store's
  (jig's `domain/engine/fleet.md`), then adopts each attempt the
  worker lists that its store holds as claimed: the run goes on in the
  same attempt, its kept turns and answers committed as they arrive, and
  its relayed calls answered on the new channel (jig's
  `domain/hosts/hub.md`, 6). Nothing is cancelled and nothing starts
  again.
- **A restart said is waited for longer.** An engine's settled stop
  tells its workers it restarts, with its restart wait, 20 minutes by
  default (`../../protocol/configuration/worker.md`, section 1): a
  worker that heard it keeps its runs, at its link's loss, for the
  longer of that wait and its grace, then the same tail as past the
  grace (jig's `protocol/channel/restart.md`, 4). Meanwhile its runs'
  turns, checks, saves and pushes go on; a relayed call the link never
  carried is asked again within its deadlines with no uncertain effect,
  and a completion whose grant lapsed waits for its renewal (jig's
  `domain/hosts/hub.md`, 6; `../../draft/archive/redeploys.md`, 5.3). The engine
  holds their claims as long (jig's `domain/engine/fleet.md`, *A restart
  said holds its runs*).
- **A start that fails is the engine still away.** An engine that
  exits before it admits a hello, refused at its start's checks or
  failing as it starts, and is restarted, is one absence to the worker:
  every failed dial, refused opening and channel lost before the
  engine's first record leaves the grace's clock running from the first
  loss (jig's `protocol/channel/worker-link.md`, 1.1), and the start that
  admits the hello within the grace adopts every run, as above, however
  many starts failed before it. One of the worker's dials is due at the
  grace's end less the link's handshake deadline (jig's
  `protocol/channel/worker-link.md`, 1.1), so an engine back by then, however
  late in the grace, finds the worker's hello before the runs are
  cancelled.
- **Past the bound, an interruption.** A run cancelled for lost contact
  saves its work first and answers `Cancelled(Contact)`; the engine ends
  the attempt interrupted, with no try spent, and the next attempt, on
  any worker, resumes from its transcript (jig's `domain/engine/executors-transcripts.md`,
  1) and starts its checkout from the work it saved (`checkouts.md`).
- **A worker's restart loses no work.** Stopped, it cancels each run for
  shutdown, saving its work first, and answers `Cancelled(Shutdown)`, an
  interruption as above, before it gives up its link (jig's
  `domain/hosts/hub.md`, 7). Killed, it takes the same path without the
  settle: back as a new process, it keeps each workspace a run held at
  the kill, its hello names their workstreams, and the engine ends
  each attempt it no longer lists suspended at once (jig's
  `domain/engine/executors.md`, 6.5). Each next attempt on this worker
  saves its workspace before its agent starts and resumes from its
  committed answers and calls (`checkouts.md`, *A workspace kept for
  its run*; smith's `domain/turn-parts.md`, section 5); one on another
  worker, or back after the window, resumes from its committed
  transcript and its last saved work, as before. Either way the next
  attempt may be on this worker once it is back, or on another. A run's start messages leave its task's inbox
  only through a committed turn's read fence (jig's `domain/tasks/messages.md`,
  2), so a run lost before its first turn hands them all to the next.

## 3. Structure

```
temper-worker-domain                 the worker's root: faces the protocol layer; routes and translates
├── jig-domain-host                  jig's hub: hosted runs, admit, prepare, start, relay, park or end
├── temper-worker-domain-checkout    the workspace: prepare, merge, commit, push, save; the cache
└── smith-host-domain                the agent: processes, spawn, channel, watchdog, cancel then kill
```

The tree is jig's worker root (jig's `domain/hosts/README.md`, section 4), with
temper's checkout as its workspace capability and smith's agent
processes as its agent capability. The root owns its children's state
and routes between them. The three share no types, so the root
translates between them through small total functions, keeping for each
workspace what each side names it by, and temper's own parts: the
workspace items down, what a run left on the forge up. smith's payloads
are handed out to the protocol layer to decode and encode (jig's
`domain/hosts/hub.md`, 8); the root links no codec of smith's. A hand-off
goes from a capability to the hub, from the hub to a capability, and at
most once more back and forth, so each entry point completes them before
it returns, with no ready list.

The root's vocabulary toward the protocol layer (`Event`, `Request`) is
what crosses: jig's link records, with temper's parts beside an
assignment and an answer; smith's payloads by hand-out; the agent
processes' io, as smith's host domain defines it; and the checkout's git
and file operations.

### 3.1 Admission and its orders

The worker's service loop admits by the root's watermark (skein's
../foundation/programming-model.md, 7.1). The queues between the root's children are
uncapped and no route reserves room. The held entrance is the link's
capped edge; a host's channel and the kernel's queues stay capped.

- **The link is read and fired on every pass,** whatever the entrance
  holds (jig's `protocol/channel/liveness.md`, 1, *Reading never waits for the
  root*): its pings come and go under any load, and the engine's records
  wait in the entrance, never unread in the socket. A socket left unread
  starves the pings both ways, and the engine loses a healthy link at
  its silence deadline.
- **The entrance holds the engine's cap:** as many records as the
  engine's output cap to a worker has frames (jig's
  `jig_protocol_worker_wire::sizes::engine_cap`, from the link's sizes),
  and the link's own state beside them (ready, lost, a grace passed). The
  engine never has more new work outstanding on a worker than that cap,
  so the entrance never fills while the engine keeps it; a record past
  it breaks the engine's contract and faults the link.

- **The backlog** is the entries in the uncapped queues between the
  service's components and the root's children; **the watermark** a
  field of the root's `Limits`, defaulting to
  `temper_worker_domain::WATERMARK`, 10,198 entries, set where the
  worker's limits are, with no key in temper's documents.
- **New work waits:** an assignment, a message, a check and a sign-out
  from the engine wait at the watermark, in the link's capped queue in
  order. **Everything else is always taken**: the children's
  completions, the agent processes' io, the checkout's operations, the
  sources and the resume (bounded by the accounts).
- **What passes a held assignment:** the engine's answers about admitted
  runs' turns and calls (a turn kept, an acknowledgement, a relay and
  its cancel, a resend), in order, up to the next new record. **What
  waits behind it** as new work does: a cancel or a grant naming a run,
  and the link's own state (ready, lost, a grace passed), since each must
  follow an assignment sent before it: a cancel that passed would be
  dropped as unknown and its run started after, and a run would start
  after its grace passed or across a new hello. The service's entrance
  predicate, `held.room() > 0 || root::passes(head)`, lets a passing
  answer through even when `held` is full; a new record at the engine
  head waits, with everything behind it. The service world holds this
  with a full held entrance in `temper-service`,
  `tests::worker_admission::a_full_held_entrance_lets_a_passing_answer_pass`.
- **A link invocation's output precedes the next invocation.**
  `drain_link` dispatches the whole batch, bounded by `link_out.len()`
  at entry; dispatch does not invoke the link. The service holds the
  complete batch before the next invocation in `temper-service`,
  `tests::worker_orders::a_link_invocations_whole_output_precedes_the_next_invocation`,
  and the reconnect order in
  `tests::worker_orders::the_hello_precedes_the_replayed_turns_at_a_reconnect`.
- **Its orders, stated** (skein/docs/foundation/programming-model.md, 7.2), each a world test
  under load: a hand-off before the next entrance's; a capability's
  hand-outs in order, and an end after its last hand-out; one entrance
  dispatched before the next; a run's progress before its watchdog; and
  the link's records after the assignment they name, as above.

## 4. Hosted runs

Moved to [`hosted-runs.md`](hosted-runs.md), section 1.

## 5. Agent processes

Moved to [`agent-processes.md`](agent-processes.md), section 1.

## 6. Facts

- **A run's facts** go up to the engine as jig's activity, through the
  hub (jig's `domain/hosts/hub.md`, 5): smith's channel facts, decoded by
  hand-out, which also feed the watchdog. A run's live text goes up
  beside them when its assignment asks for it: smith's pieces, handed
  from its channel to the hub as jig's pieces (jig's `domain/hosts/hub.md`,
  5), each also progress for the watchdog, and best effort as facts
  are.
- **The worker's own facts:** each child domain's, content-free (tokens,
  kinds and counts, never a name, a branch or a message), gathered after
  every entry point into one capped queue: a sink that drops and
  counts (skein's ../foundation/programming-model.md, section 3), with no production
  consumer.

Nothing the worker or the engine decides depends on a fact arriving, and
no fact holds a slot.

## 7. Turns

Turns are the hub's, as jig's `domain/hosts/hub.md`, 4 says: smith's
bytes, of which the worker reads only what comes beside them, kept until
the engine acknowledges them, and lost with the run when the worker
dies, so the task's next run starts from the last committed turn.

## 8. Below the domain

What the protocol and io layers owe the domain. The worker's protocol
layer is the worker's service (`../../protocol/service-routing.md`, section 2): jig's
worker link, smith's host half and agent processes, temper's identities'
table and the git component.

- **The engine:** jig's channel (jig's `protocol/channel/README.md`), dialled by
  the worker and authenticated, carrying the hub's link vocabulary, with
  temper's parts as temper's bytes under temper's payload version
  (`../../protocol/channel.md`). The run's facts go after what the domain
  emitted, so none goes before the hello. The root keeps each run's
  items until the engine has its answer, so the parts going up name each
  repository by its tag, and the engine's protocol layer, which keeps no
  state, needs nothing else to read them. In one process the link joins
  the engine by an in-process address (`../../protocol/service-process.md`,
  section 1).
- **Agents:** spawning the agent within a deadline, in a contained
  process tree, with proof that the tree is empty after it stops; where
  skein gives no contained trees (jig's `protocol/needs.md`,
  section 1), each agent in a process group of its own, which a stop
  signals whole. smith's channel over its pipes (smith's host half); an
  environment without forge credentials. smith's payloads (the charter,
  the transcript, turns, facts) are decoded and encoded by hand-out (jig's
  `domain/hosts/hub.md`, 8). The root completes the decoded start with
  where the repositories sit, as the checkout prepared them, and, for a
  merge in progress, which files are in conflict.
- **Git:** each typed operation is a fixed sequence of git invocations
  under the operation's one deadline (`../../protocol/git-operations.md`, section 1),
  its output parsed into a typed outcome. A commit is named by its full
  object id (a SHA-1 id zero-padded to a SHA-256 id's 32 bytes); a merge
  leaves its conflicts in the working tree and says which files; a
  commit may have several parents. An operation that ran out of time or
  was cancelled ends only once its process has exited, so nothing of it
  touches the workspace afterwards. A repository's identity is mapped to
  credentials and an author from the identities' table, per invocation,
  in git's environment only, never in an argument, a URL or a file
  (`../../protocol/git.md`, section 5).
- **Files:** workspace directories under the worker's `root`
  (`../../protocol/configuration/worker.md`, section 1). Making one empties it
  first, so what the cache evicted, or a worker left behind before a
  restart, cannot fail a prepare. The stores sit beside them, one bare
  repository each, and outlive a restart (`../../protocol/git.md`,
  section 2), a cache as the workspaces are: what must survive still lives
  in the engine's store or on the forge, and a store removed is made again
  from the forge (`checkouts.md`).

## 9. The world

The worker's worlds run the domain against neighbours that share none of
its types (skein/docs/foundation/testing-strategy.md, section 4). `../../testing/README.md` places them
among temper's tiers and keeps their fakes.

- **The checkout's world** (`tests/worker/checkout`): the checkout, the
  world its parent, against a fake forge and a fake disk. Its stories: a
  change prepared, edited, pushed and landed; a workstream reusing its
  workspace, and others evicting the least recently used; a held
  workstream busy, and a full cache refused; a spec beyond the limits
  refused; a spec naming what the forge lacks failing for good; an
  unreachable forge failing for now and a refusing one for good; a
  branch another party advanced making the push moved, nothing forced;
  saved work where a later run resumes; a new workstream, an evicted one
  and a saved start fetching from the forge only the commits the worker's
  store lacks, with every fetch's imports counted; an abort ending a prepare under
  way; a producer's clean merge committed before it is ready; merges that
  conflict, keep their markers until resolved and
  commit both parents, a stale expected head refused, and a push that
  lands but times out; a first hold pushing, releasing, and a new hold
  delivering the same existing diff without another edit; landing ancestry,
  missing landing branches, moved remote heads and unchanged save semantics.
  Those stories run on the fakes, a domain world's, and are a binary of
  their own, `fakes`, in the domain layer, their CPU budget moved down
  with them (`tools/tiers/budgets.md`, 10). The stories that run real git
  against an owned Forgejo (a design's and a change's delivery reaching
  their landing branch, and a moved goal) stay the binary `suite`, in the
  real loop's layer (skein/docs/foundation/testing-strategy.md, 2.8 and
  8.1: a binary's tests are in one tier).
- **The worker's world** (`tests/worker/domain`): jig's hub, smith's host
  domain and the checkout, against scripted engine and agent peers, the
  protocol layer's routing played with smith's codecs, and the fake
  forge and disk. Its stories: a prepared run delivering its complete
  tree; a cancel mid-delivery waiting for the landing; contact restored
  within the grace, and lost past it, saving before the cancellation's
  answer; the hello carrying the configured stop bound whole and the
  workstreams the checkout holds; a branch the forge lacks failing the
  prepare, naming the repository; a save saying whether each repository
  started from saved work; turns surviving busy, reconnects and crossed
  acknowledgements; a worker shutting down past the grace giving up what
  it keeps, and a channel reopened in time getting every turn. Its
  runtime world's referee checks the whole boundary (`../../testing/checks.md`,
  2).
- **The system world** (`tests/system`): the engine's and the worker's
  roots, smith's real agent domain over skein's fake LLM, jig's fake
  store and parties, and the fake forge, which the engine and the
  worker's git share, so a branch the worker pushes is the head the
  engine reads. Its stories: a chat's report through the worker and
  smith; a host call and its committed answer across the agent's
  channel; a person's words and stop reaching smith; an adopted
  repository prepared and mounted in smith; a parked run resuming
  through both roots; a parked producer's delivered and saved edits
  surviving a moved landing branch, clean and in conflict, to its next
  delivery; a withheld commit holding the assignment. And section 2's
  absences, the engine's root stopped and reopened on its store while
  the worker's root keeps hosting, the link between them cut and joined
  again, the grace passing on the world's clock: an engine back within
  the grace adopting the run in its same attempt, one assignment and
  one run, the model asked once a turn and the reply read once; the
  same after two starts refused before it admits the hello, one absence
  to the worker; past the grace, the run cancelled for contact, its
  attempt ended interrupted with no try spent and the next resuming
  from its transcript, the committed turn in its messages once; and a
  worker stopped before its run's first turn and started again, the
  next attempt reading the chat's first message once. The link's own
  grace, its redials and refused hellos, is jig's worker link's world's;
  a start's refusal, `temper-service`'s.
- **The one-process world** (`tests/one-process`): temper's services on
  skein's simulator, the worker's agents smith's processes over skein's
  fake LLM (`temper/docs/design/testing/tiers.md`, 1.5).

Across them, each world checks what is its: that the engine commits each
attempt's turns and answer once and acknowledges them in time; that each
relayed call reaches the engine once and is answered once; that what
lands is exactly the tree the agent left (a merge with both parents when
it started from one) and saved work exactly the tree at the stop; that
no git operation runs while a run's agent may, but the push it asked
for; and that nothing is live once it settles.

## 10. Open questions

- **Several workers in one world:** the whole worker's world runs one;
  placement across several is exercised with scripted workers in the
  engine's world.
- **Checks without an LLM:** validating an exact head needs a checkout
  and the repository's checks but no conversation. It could be a run
  whose outcome is its checks' result, for repositories without CI
  (../forge/README.md, section 11).
- **jig's:** saving periodically, and a worker's failed dials before its
  first channel (jig's `domain/hosts/README.md`, section 10).
