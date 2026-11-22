# temper's root

Provisional, 2026-10-04, revised 2026-11-04. What temper's engine root
is and does: `temper-engine-domain`, the top-level domain of temper's
engine, which composes jig's core with the forge connector in the shape
jig's `domain/root.md` gives every application's root. It says what the
root composes and carries, its vocabulary toward the protocol layer, the
one shape as temper holds it, how it routes, translates and puts outputs
together, its store vocabulary, its part in restart and stopping, its
limits, and what it never does. The core is jig's `domain/engine/README.md`;
the forge is forge/README.md; the worker the root assigns runs to is worker/README.md,
and the agent agent/README.md. What is still open is listed in section 12.

## 1. In one page

- **temper's root composes jig's core and the forge.** Its children are
  jig's core, `jig-domain-core`, the forge connector's crates, and the
  calendar, `temper-engine-domain-calendar`, which renews each project's
  funding period (`routing.md`, 7). It alone faces the engine's protocol layer
  (section 3; `../protocol/service-routing.md`, section 1).
- **It routes and translates; jig's core decides.** What commits
  together, what waits for a commit, the order of a restart and the check
  on every effect are the core's or the journal's (jig's
  `domain/root.md`, section 1). The root carries them through skein-lib's
  journal, its only way out (section 4).
- **It speaks for the forge to the core.** The forge's crates name no
  type of jig's core: the root makes, from what the forge says, what the
  core takes (an effect's description, a verdict, a procedure's step and
  its delegates, a hold's reason, a result), and back (`routing.md` and `root-translation.md`, section 1).
- **Every run is on a worker.** temper runs no agent in the engine: the
  root has no hub child, and speaks jig's hub's link vocabulary to
  workers, which in standalone share the engine's process (section 2).
- **The owner keeps the value.** The root puts each output together from
  its owners' parts: an assignment from the core's part and the forge's
  workspace and brief sections, a call's answer from the core's part and
  the forge's (`root-translation.md`, section 2).
- **One store vocabulary.** Its keys, records and ranges wrap the core's
  and the forge's; it has no records of its own (section 7).
- **Admitted by a watermark.** New external work waits while the root's
  backlog (its journal's, its work queue's, its queued store answers') is
  at its watermark; whatever finishes admitted work is always taken, and
  an event that carries both is split. No route reserves room, and its
  orders are stated and tested (section 4; skein's programming-model.md,
  7.1 and 7.2).
- **On jig's rails.** The engine's world runs the root with its real
  children over jig's store; the conformance world runs it on jig's
  harness, under jig's referee (section 11).

## 2. What the root is

```
temper-engine-domain                       the root: the journal, routing, translation, putting outputs together
├── jig-domain-core                        jig's core and its children (jig's domain/engine/README.md, section 3)
├── temper-engine-domain-store             the root's store vocabulary: keys, records, ranges and pages (section 8)
├── temper-engine-domain-forge             the forge connector's top (forge/README.md, section 3)
│   ├── temper-engine-domain-forge-vocabulary  what crosses the top's edge (forge/README.md, section 3)
│   ├── skein-forge-client                 the working set, keeping up, fresh reads, the request budget, effects' keys, write lanes (skein's forge-client.md)
│   ├── temper-engine-domain-forge-change  the change procedure and the landing queues
│   ├── temper-engine-domain-forge-landing the landing by turn (landing.md)
│   └── temper-engine-domain-forge-issues  the goals' issues
├── temper-engine-domain-calendar          each project's funding period, renewed on a span (5.7)
└── temper-engine-domain-trials            a model's trials (trial.md)
```

- **One connector, the forge.** Configuration names the core one
  connector, the forge, and a start naming another is refused
  (`../protocol/configuration/README.md`, section 2). The root knows the forge
  by that number and matches on its variants exhaustively.
- **No hub, no agent in the engine.** The root links `jig-domain-host`
  for the hub's link vocabulary alone: a worker's hello and its records
  up and down (section 3). The engine has no slots of its own
  (`fleet.engine_slots` 0) and temper's charters run on workers
  (`hosts: "workers"`), so the fleet places every run on a worker (jig's
  `domain/hosts/README.md`, section 3).
- **Workers in the engine's process, in standalone.** The worker joins
  the engine by an in-process address (`../protocol/service-process.md`, section
  1), so the core is configured with its workers in process
  (`workers_in_process`, which jig's configuration sets from an
  in-process hosts' listener), and a restart settles their claims at once
  (jig's `domain/engine/restart.md`). temper's domain worlds run their
  workers apart.
- **Its state:** its children's; the journal; what it keeps of its
  configuration (the forge's number); its facts
  waiting to be drained; and the values in flight between a hand-off and
  its answer, each bounded (section 9): a worker's record until the
  fleet takes it, a run's brief sections and workspace while its claim is
  prepared, its assignment until the claim's decision, the loads it makes
  for the core, host calls being decoded and rendered, and the forge's
  reads, effects, subscriptions and adoptions in flight; the calendar's
  record per project and its one deadline (`routing.md`, 7). None of it is stored.
- **Siblings share no types.** The core never names the forge's types,
  nor the forge the core's; the forge's top names jig's people child's
  outcomes alone, for its keyed adoptions (forge/adoption.md).

The crate, one module per part:

```
temper-engine-domain/src/
├── lib.rs              the tree, re-exports
├── boundary.rs         the root's vocabulary: events in, requests out (section 3)
├── admission.rs        what the journal takes before any child changes (section 4)
├── domain.rs           Domain, step, fire, release: the one shape (section 4)
├── decision.rs         the decision and the journal's limits (section 4)
├── route.rs, route/    which child each event and hand-off goes to (section 5)
├── translate.rs        Config and the small total functions (section 6)
├── assemble.rs         assignments and briefs from their owners' parts (section 7)
├── link.rs             the hub's link records, and what a run left (sections 3 and 7)
├── store, loads        temper-engine-domain-store: keys, records, ranges and pages (section 8)
├── archive.rs          loads outside the working set (section 8)
├── restart.rs          the restart script's steps (section 9)
├── stopping.rs         the drain (section 9)
├── deadline.rs         the children's timers
├── limits.rs           the root's entrance limits and validity (section 10)
└── tests.rs
```

## 3. Its vocabulary

What crosses the domain boundary, in `boundary.rs`: an exhaustive enum
per direction, one variant per peer. The engine's service on the other
side, its components and what its routing does with each, is
`../protocol/service-routing.md`, section 1; in standalone it shares its process
with the worker's (`../protocol/service-process.md`, section 1).

- **In, `Event`:**
  - the store's answers (`StoreInput`): a commit durable, with the
    store's fill; a commit failed; a page loaded, or not; a core read's
    page; the store's census, at most once a minute, which the root
    hands the core as its `Event::Census`, keeping the last while it
    starts as it keeps the fill (jig's `protocol/store/README.md`, 6.3);
  - a worker's channel (`WorkerInput::Link`): a hello, a record up with
    temper's `Parts` beside an answer, a loss; and the protocol layer's
    hand-backs: a host call decoded into the root's `Tool`, a decided
    answer rendered, an ended run's outcome read;
  - a party's (`PartyInput`), among them signing in and out, a keyed
    request, an application request, reads, watches and their
    deliveries, a project's period;
  - accounts' events, and a generation of a git identity
    (`Event::Credential`, `root-translation.md`, section 2.1);
  - the forge's events: its API's answers and its webhooks' hints;
  - a released output addressed to a child (`Event::Released`), and the
    service's `Restart`, `Resume` and `Stop`.
- **Out, `Request`:** to the store, a commit, a page to load, a core
  read's load; to a worker, a link record, an assignment, a refusal, a
  turn answered busy, a call to decode or dropped, an answer to render,
  an ended outcome to read; to a party, replies, read answers, documents,
  a watch refused or ended, a run's text; the forge's API calls
  (`Request::Forge`); accounts' requests; a released output back to its
  child (`Request::ToChild`); a forge event the drain hands back
  (`Request::Busy`); `Drained`; `Stop`.
- **temper's parts on the hub's records** (`../protocol/channel.md`,
  sections 4 and 5): the assignment's `ForgeWorkspace` down; up, beside
  an answer, `Parts`: the heads the run pushed, the repositories it
  saved, a permanent failure to prepare one.
- **Facts,** which decide nothing: the root's, among them a failed
  preparation (`Fact::PreparationFailed`, `root-translation.md`, section 2.4), a brief's read
  refused and a brief failed (`Fact::BriefReadRefused`, `Fact::BriefFailed`;
  forge/change/repairs.md) and a branch head refused at a full table
  (`Fact::HeadRefused`, forge/keeping-up.md, 1), at most 32 waiting to be drained, the rest counted as
  lost; and the children's own.
- **Tools.** The root's `Tool` is a host call in typed form, as the
  protocol layer decodes it (`../protocol/llm.md`, section 4): jig's
  operations on tasks, notes, the forge's reads, effects and topics, and
  the refusal of a call whose shape is past its bounds. Which tools
  temper's agents get, and what each answers, is agent/tools.md.

## 4. The one shape, and the journal

jig's shape (jig's `domain/root.md`, section 3), with temper's
admission. The journal's `held` field and memory estimates do not
admit work; the watermark paces new external work (section 9).

```
try_step(root, event):
    a store's answer (committed, loaded, unloaded), a released output, Resume:   always taken
    a completion or an advance of admitted work (below):   always taken, once the work queue is empty
    a core read's page, a load switched:   taken in arrival order, after the work queue and earlier answers
    new external work (below):             handed back while the backlog is at the watermark,
                                           or while the work queue holds a hand-off
step(root, event):
    route the event to its child; drain every hand-off it starts, through the root's work queue
    the core finishes its decision: its writes and held outputs into one decision, which cannot fail
    journal.decide: at most one commit, the deployment's record its first write
    outputs marked now: through the door
release(root):   (the service's iterate)
    after a step: the door's outputs and the commit
    otherwise:    one held output whose commit is durable, then the next commit
```

- **The backlog,** in entries: the journal's (`Journal::backlog`: the
  commits made and not yet answered, the outputs held and those at the
  door), the work queue's and the queued store answers'. Its watermark is
  section 9's. The journal and the work queue are uncapped: a decision
  always completes, and nothing between the root's children reserves
  room.
- **Its orders, stated** (programming-model.md, 7.2), each a world test
  under load in the root's world (section 11):
  - **hand-offs first:** a hand-off a step queued is finished before the
    next entrance is admitted (`work.is_empty()`);
  - **store answers in order:** a read's page and a load's switch are
    stepped in arrival order, queued behind earlier work;
  - **releases in order:** an older release steps before a later one;
    releases are always taken.
- **Classed by variant.** No event carries new work beside admitted
  work, so none is split; each variant is one or the other.
  - **Always taken:** a worker's link coming up or lost, and every record
    it sends but a call (a turn, an answer, pieces, logins, a sign-out,
    activity, limits), with the payloads' room where a record takes a
    slot; a host call decoded or rendered; a result read; a party's
    acknowledgements and unwatch, and a procedure step's answers; an
    account's notifications; setup and stop; the forge's answers and
    hand-offs. The new work a completion makes possible (a placement, a
    due task's activation, the forge's continuation) waits at `Resume`'s
    own watermark check.
  - **New, waiting at the watermark:** a worker's call, a party's
    requests, a credential, a restart, the forge's bind, installation and
    hint.
- **A held call lets completions pass.** The service holds one handed-back
  new event per source, and lets the events behind it that the root
  always takes pass it, in order, up to the next new one
  (programming-model.md, 7.1). A call held at the watermark never acts
  for an attempt that ended behind it, and completions taken under
  pressure release nothing before their commit.
- **Handed back, not answered.** An input the journal cannot take returns
  to its owner unchanged, who offers it again; the root answers nobody
  busy itself. Busy answers are the core's (jig's `domain/engine/README.md`,
  5.1).
- **The deployment's record rides every commit:** the core's numbers,
  with the commit's own, so a restart reads the last durable commit from
  it.
- **Released in order:** one held output per call, and the door's outputs
  first, after the step that staged them. A released output addressed to
  a child (a fleet's start, a relayed message, a forge entry to make, a
  load) comes back as `Event::Released`, one more step of this shape.
- **A released batch precedes the next release.** `requests.is_empty()`
  requires every request from the previous release to be dispatched
  before the service calls the root's release again. The service world
  holds this with a multi-request release and a later input waiting in
  `temper-service`,
  `tests::engine::a_released_batch_is_routed_before_the_next_release`.
- **An io continuation precedes a later completion.** `!io.is_ready()`
  requires io's ready continuations to finish before the service takes
  another queued completion. The service world holds this with kernel
  output credit held in `temper-service`,
  `tests::system::an_io_continuation_reaches_its_owner_before_a_later_completion`.
- **Queued completions precede a due io timer.**
  `completions.is_empty()` requires the service's queued completions to
  be taken before `io::fire`. The service world holds this with a full
  completion batch and a timer due in `temper-service`,
  `tests::system::queued_completions_are_taken_before_a_due_io_timer`.
- **Io events keep their arrival order.** `destination_can_take(told_head)`
  requires the head of `told` to be dispatched before a later event,
  even one for another destination. A blocked head holds the FIFO. The
  service world holds this with a diagnostics sink full in
  `temper-service`,
  `tests::system::a_blocked_told_head_holds_later_events_for_other_destinations`.
- **A source's closing state keeps its close.**
  `close_enqueued_before_source_marked_closing` requires the service to
  hand the close to io in the same stage that marks its source closing;
  without output credit the source stays open. The service world holds
  this with kernel output credit held in `temper-service`,
  `tests::system::a_source_is_marked_closing_only_once_its_close_is_queued`.
- **An unpublished cancellation keeps its target's route.**
  `queued_cancel(target) => retained_target_route` requires a completed
  target's external route to survive until its queued cancellation is
  published, so the cancellation's terminal still reaches its owner.
  The service world holds this with publication stalled in
  `temper-service`,
  `tests::engine::a_cancellation_waiting_unpublished_keeps_its_completed_targets_route`.
- **A productive root yields between scheduling quanta.**
  `RELEASE_CYCLES` bounds one service pass to 22 release/resume cycles.
  Exhausting the quantum leaves `ready` set, so the loop stays awake;
  the next pass takes waiting answers before resuming root work. The
  service world holds this with the root productive across the quantum
  in `temper-service`,
  `tests::engine::a_root_productive_past_one_quantum_lets_a_waiting_answer_in`.
- **Each secret request keeps its own terminal.** `secret_owners`
  grows with the requests in flight and retains each routing token's
  owner until its terminal, without a fixed concurrency cap. The
  service world holds this with more requests in flight than client
  staging slots and terminals returned in reverse order in
  `temper-service`,
  `tests::clients::secret_asks_in_flight_each_get_their_own_terminal`.
- **Owner progress is a step of its own.** `Resume` admits one: the
  forge's continuation when it has one, else a due task's activation,
  else the fleet's.
- **Timers** fire through the same admission (`fire`), waiting at the
  watermark as new work does: accounts' and views' from the start; the
  forge's from its restart step on; the core's others once decisions
  open.
- **Stopping of its own:** a root that cannot go on (a counter at its
  end, a decision past the store's commit edge) stops its journal
  (`Journal::stop`, skein's `runtime/lib.md`, 11), and nothing held or at the
  door leaves again; a failed commit fails it as jig's journal says
  (jig's `domain/root.md`, section 4).
- **A stopped journal ends the engine, naming why.** A store answer the
  journal refuses (skein's `runtime/lib.md`, 11: an answer outside the order the
  commits were sent) stops it as the stops above do. Whichever stopped
  it, the root emits `Stop` in the same step, with what stopped it: the
  commit that failed, the answer refused with the commits sent and
  durable, the decision past the store's edge, or the counter at its
  end. A root whose journal has stopped never stays up undeciding: the
  engine ends with that reason in its log (`../protocol/service-process.md`,
  1.2).

### 4.1 The byte budget

The engine meters the transcripts and turns it holds in memory against
one budget, `engine.memory_bytes` (skein's programming-model.md, 6.1),
counted at the root, which sees them all.

- **What counts,** each byte once, while it is in memory:
  - a worker's turn body, from the moment the root takes its link's
    record until its commit is issued to the store (the write leaves the
    root) or it is dropped, wherever the root holds it on the way (its
    payloads, the work queue, a decision, the journal's held write): a
    running total of the root's, raised where it takes the body and
    lowered where it lets it go, so a move between its owners changes
    nothing;
  - a run's preparation, from its start: the core's
    `Core::transcript_bytes`, which counts each preparation at its resume
    allowance (its turns' and its text's) until its load is in, then at
    the bodies and text it keeps, never the loaded window's framing or
    its evicted history; a sum over its preparations, which its task
    limit bounds;
  - an assignment's turns until the worker takes them, and a brief's
    sections from their completion at the root until its assignment goes
    out, in the root's running total.
  A preparation's loaded pages are taken into the core at once, counted
  by its allowance; what waits in the store answers is a person's read,
  which no budget counts.
- **What waits:** a run's preparation, the one entrance that brings the
  most. While the held bytes are at or past the budget, every activation
  waits, the task staying due in jig: `Resume`'s second choice (it goes
  on to the fleet's progress), an activation that comes ready of itself,
  and one queued as the root's work. A restart's restored activations (a
  task due or preparing when the engine stopped) are no longer drained
  all at once when decisions open: `Resume` takes them within the
  budget, as bytes drain. Since a
  preparation counts its whole allowance from its start, the held bytes
  pass the budget by at most one preparation. A worker's turn of a run
  already admitted is never refused for bytes: the fleet's limits and
  the wire's turn size bound it.
- **What does not wait:** a person's reads and calls that start no run,
  the fleet's progress, the store's answers, completions. The budget
  limits pacing in bytes as the watermark does in entries (section 9),
  and the held bytes drain without a new preparation: assignments are
  delivered, preparations fail or finish, turns are committed. With no
  worker to take assignments, preparations stop, and nothing else does.
- **Its value** is the document's `engine.memory_bytes`: 1 GiB in
  `temper-box init`'s template and the owned example, a budget for the
  transcripts and turns the engine holds, never a cap on the process's
  memory (the help pages say so). No start checks it against a sum
  (skein's programming-model.md, 6.3); a deployment that declared a
  far larger figure before keeps it, and the budget then rarely binds.
  It is positive: a budget of zero would hold every preparation for
  ever, so the engine's limits refuse it at start, as they refuse a
  zero watermark.
- **Its test,** in the root's world: with a budget of about two
  preparations, three due tasks activate two at a time as earlier runs
  are assigned and their turns committed, while the fleet's placements
  go on; a restart with three tasks due activates them the same way; a
  person's read meanwhile is answered at once; and nothing deadlocks
  with no worker connected.

## 5. Translation

Moved to [`root-translation.md`](root-translation.md), section 1.

## 6. Putting outputs together

Moved to [`root-translation.md`](root-translation.md), section 2.

## 7. Its store vocabulary

- **`Key`, `Record`, `Write`** wrap the core's and the forge's, one
  variant each, the core's first; a write saves a record under its own
  key or erases a key (jig's `domain/engine/README.md`, 5.6).
- **No records of its own.** The deployment's record is the core's; a
  call's record is the core's part and the forge's (`root-translation.md`, 2.3).
- **`Range`:** the core's ranges the root loads; the census of ended
  owners (jig's `protocol/store/loads.md`, 2); and `Range::Forge`, every
  record of the forge's, all live (`../protocol/store.md`, section 2).
  The core's: at restart, the deployment's record, people, live tasks and
  goals' projections, claims and calls; on demand, a task's result, an
  ended read position, a task's current window, an attempt's turns, a
  decided escalation or proposal, a page of notes.
- **Pages,** bounded in rows and bytes, are taken in the order jig's
  walk gives them (jig's `protocol/store/loads.md`, 2), each row within its
  bytes; a page past its bytes comes back cut, saying how much. Their
  range, order and cursors are the tests' to assert, never refused at a
  start (jig's `domain/engine/restart.md`).
- **Loads for the core.** A run's preparation (its transcript's window,
  its dependencies' and inputs' results), a delegate batch's inputs, and
  the core's historical reads (ended conversations, decided escalations
  and proposals) are loads the root makes for the core and answers back
  in its terms; a full table answers busy all but an ended task's load,
  which waits (below).
- **An ended task's load waits for a slot.** The core's load of an
  ended task (jig's `Now::EndedTask`: a run's dependencies' and inputs'
  results, and the ended task an amendment, a batch or a decision
  reads) that finds the archive's read table or the load slots full, a
  retired slot awaiting its reclaim among them, is not answered busy:
  the root keeps it, in the order asked, and issues it once a slot
  frees, before any read asked after it, still held behind the commits
  made before the core asked (jig's `domain/engine/README.md`, 5.3).
  Busy is left for a load that failed: a page refused or cut, or an
  answer the archive cannot read. The core's other loads (an ended
  conversation and its read position, a decided escalation or proposal,
  a batch's inputs, a person's ended task) keep busy on a full table.
- **An issued transcript load that fails** answers the core's
  `TranscriptLoad::Unread`, retiring its read once; an answer to a stale
  preparation is ignored. The core owns the window policy (jig's
  `domain/engine/executors.md`, 2): a closed window starts fresh with an empty
  tail and no added failure or refusal, while an open window keeps its
  transient failure policy. A transcript load is issued only with read
  room: without it the root answers the activation `busy` and the task
  stays due (jig's `domain/root.md`, 3). A window's turn the store hands
  on cut (jig's `protocol/store/transcripts.md`, 3) reaches the core as
  `TranscriptLoad::Cut`: its place and its length, never its bytes. So
  does a turn the store carried whole but past the root's own page bounds
  (`root.loads.transcript_bytes` for a row, `root.loads.bytes` for a
  page): the service hands it on cut, alone in its page, rather than end
  the page, so no turn's size fails a window or ends the engine's
  journal. Other successful-page cuts and the transcript row and text
  validation guards keep their existing preparation-failure translation.
- **A window's pages keep their load slot.** A transcript's window is
  read a page of `root.loads.rows` turns at a time. A page that names a
  next one hands its load slot on: the next page is issued in that slot,
  in the pass that took the page, so a window of any length loads
  whatever else holds the table, and room never fails it. Only its first
  page waits for room, at the activation, as above. The slot frees with
  the window's last page, or with its read when the preparation it
  served ends between pages. A restart's due tasks, reading their
  windows together, wait for their first pages and never fail on the
  later ones.

## 8. Restart and stopping

Moved to [`root-restart.md`](root-restart.md), section 1.

## 9. Limits and validity

The root carries configured `core.policy_bytes` and `core.call_answer_bytes`
unchanged and derives one effective core envelope for runtime, store opening,
sizing and the service wire. The journal transcript bound is independent;
transcript capacity never enlarges the configured call-answer bound. Startup
checks that envelope against the codec and checks that a policy at its
configured bound plus its owned row carrier fits the journal. Transcript
capacity never enlarges the configured policy budget. Restore refuses an
over-bound stored settings policy by setting number and source, and an
over-bound project policy by project number.
A change of settings whose charter's stored policy, its parts with it,
passes `root.journal.run_bytes` is refused as that charter's item out of
shape, with the facts jig's refusal carries for it (jig's
`domain/engine/settings.md`, 9): the policy's stored bytes and the bound. ctl
and a run's settings tool word them alike, as
`out of shape; item 1; charter 5: shape; stored policy: supplied 70112; maximum root.journal.run_bytes 65536`,
and the client's JSON names the member `stored_policy`.
Startup also checks the seed policy after enforced landing requirements are joined, before constructing the core.

- **The restart's workspace** holds a cold restore page, which holds
  any row a writer commits (jig's `protocol/store/loads.md`, 1). It lives
  only before the engine serves; a startup answer goes straight to
  the restart, one at a time, and occupies no retained read-answer slot.
- **The core's children's limits** are `engine.limits`, jig's part:
  tasks, authority, people, the fleet, accounts, notes, documents,
  templates and call records (jig's `protocol/limits.md`
  ).
- **The root's own,** `root.` in temper's limits
  (`../protocol/configuration/limits.md`):
  - the journal (temper's `JournalLimits`, its decision `Limits`):
    commits in flight, which is the store's edge (jig's store queues
    that many commits, so the root sends
    a commit only while fewer are in flight, the store's credit, and the
    rest wait in the journal); the service passes `root.journal.commits`
    and `root.journal.writes` as jig's store `sizes::Commits`'s
    `in_flight` and `writes` (`../protocol/store-versions.md`, 1); with the door's
    outputs, it gives the watermark; a row's bytes, a result's bytes
    and a charter's bytes,
    which size the store's records (edges); the journal itself, its
    decisions and its held outputs are uncapped;
  - loads: in flight, rows per page, the bytes kept and a page's bytes;
  - briefs: gathers, sections per gather, parts per section, a section's
    read bytes, a budget per kind of section (the task, dependencies, CI,
    reviews, the pull request, attempts, the plan, notes), a brief's
    bytes and the gather's deadline.

    The gather's deadline bounds a connector that never answers. Its
    default, 30 seconds, outlasts a brief's sequential forge reads: a
    pull section reads Item, Pull, PullFiles and Compare, each expanded
    into GitHub requests, with file contents adding reads. Even six to
    eight sequential requests at a slow but healthy two seconds each,
    with a token mint first and one retry after backoff, can occupy
    about 20 seconds. Thirty leaves room that ten cannot, rather than
    failing a healthy brief for waiting, and still ends a stuck connector
    within the same sitting. These are planning latencies, not a bound
    on every forge response.

  They give the core its resume limit, within a row's bytes, its
  charters' bytes, its load slots and its briefs'.
- **Checked at start where a limit sizes an edge:** a row's bytes at
  least the largest task row and the forge's largest answer; a result's
  bytes at least a brief, a task's result and a party's words; the
  forge's brief sections and bytes at least what the briefs may plan. No
  limit has to cover another's room: no decision is capped in outputs,
  nor in writes but at the store's commit edge (jig's store refuses a
  commit of more than `journal.writes`), and no event reserves a route,
  so the held outputs need not cover the door's, nor the forge's named
  calls the call records.
- **The watermark,** a field of the root's `Limits`, in entries: the
  journal's commits plus the door's outputs (`commits` and `deliveries`),
  derived where the root's limits are, with no key of its own in
  temper's documents. That is what a route could hold before the
  watermark (its commits in flight and the door's outputs), so pacing
  stays close to it: at the commits alone, a person's watch waited
  twice as many offers behind admitted work.
- **The watches' rooms follow the streams:** the root's `streams` is
  `clients.streams`, and `core_limits` gives the views `streams` times
  `WATCHES_PER_STREAM`, 26, one shipped client's bound, in all and to a
  party, above jig's profiles' 4 (`../protocol/configuration/limits.md`
  ). The rest of the views'
  limits are jig's small profile's: 4 runs watched, 65,536 bytes of a
  run's text, 32 subjects marked per change, 32 facts.
- **The store's fill mark and a read's bytes** are jig's small profile's
  numbers, 268,435,456 and 65,536 bytes, held as the root's constants
  (section 12).
- **Validity:** `temper_engine_domain::valid` checks child validity,
  entrance constraints, payload capacities and count representability.
  The journal requires nonzero `commits`, `writes`, `deliveries` and
  `run_bytes`; `writes = 1` admits a header-only commit. `deliveries`
  caps no queue (decision deliveries queue uncapped): it is the outputs
  at the journal's door, and the default watermark is `commits` plus
  `deliveries` (`default_watermark`). Its `held`
  field and memory estimates impose no admission condition. The root
  accepts `forge.output = 0`: the forge's internal output envelope
  does not constrain the parent. No heap sum admits the root.

## 10. What the root never does

- decide anything of the core's or the forge's: what to commit, what
  waits, whether an effect may be made, what a change does next, the
  order of a restart, whether a setting of temper's kinds is well
  formed;
- send an output any way but through the journal;
- keep anything between steps but its children, the journal and the
  values in flight of section 2;
- read or rewrite a value it carries, beyond wrapping and translating
  it and joining a landing rule's approvals to the core's identities
  and roles (`routing.md`, 6);
- give the core a type of the forge's, or the forge one of the core's;
- route by content beyond the variant and the approval eligibility
  check of `routing.md`, section 6;
- price a forge effect, or grant the forge's API token, App key or
  webhook secret.

## 11. The world

The root has no child domain of its own; its worlds are the engine's
(`../testing/tiers.md`, section 1):

- **The engine's world,** `tests/engine/domain`: the root with jig's real
  children and the forge's crates, over jig's store through the service's
  store routing, an independent fake forge below, a scripted person and
  worker, and the protocol layer played (the hub's records, host calls
  decoded and rendered, ended outcomes read, accounts). The engine
  restarts cold at drawn moments and at each commit a scenario chooses.
- **The conformance world,** `tests/conformance`: the root on jig's
  harness and its outside referee, with jig's shared scenarios bound to
  the forge and temper's own stories, crashed at every commit (jig's
  `domain/testing.md`, sections 5 and 8).
- **The system worlds:** the root with the worker and smith's agent
  (`tests/system`), and with temper's client (`tests/client`)
  (`../testing/tiers.md`, 1.1).

Its stories, as the root meets them: a chat that answers, parks and
resumes from its transcript's window; a chat past its resume limit
starting fresh, its tail in its brief; a tool call whose answer is lost
with the channel and the engine, asked again after a restart and
answered from its record, made once; an undecodable call answered with
no commit and no record, then decided once under a new name; a lost
attempt told of the calls committed after its last turn; calls under
backpressure handed back and decided once; a failed preparation reported
as a fact of its attempt and retried by its answer's class; an answer
pushing more heads than its task may hold, failing as invalid; saved
work reaching the next attempt's checkout; an adoption refused for a
forge not configured, a repository not found or not readable, and one
adopted with its collaborators seeded into roles, sent again and
answered its kept outcome, across a restart too; a forge effect
committed before its write and made once, and one proposed and kept
until its holder accepts; a stop with a committed forge effect, making
no commit and no write; a start refused for live tasks past lowered
limits, or for a malformed page; a CI repair's brief carrying its failed
step and its log's end, which `read_ci what=job` reads again, on GitHub's layer
as on Forgejo's (`forge/change/repairs.md`), and a failed repair released
at a head CI has not passed at, its replacement's brief carrying them
again and nothing merged. The forge's own stories are forge/README.md,
section 10's.

Its Open chats' stories (`agent/charter.md`, 5), in the engine's world on
a chat ended by its requester, as temper's is configured (temper-init's
golden shows the flag shipped): a chat whose run answers and
finishes parks, its reply committed and no try spent, still at rest
past its idle interval; its person's next words wake it once; its
person's Close, the client's `close_chat`, ends it with its committed
turns kept and its delegates' fates as the Close names them; a goal's
delegate in the same world still ends by its result; and a chat whose
ordinary source is a program endpoint its worker's hello declares is
refused with its typed cause, starts nothing, and reads `HostRefused`.
In the root with the worker and smith's agent, an Open chat's text-only
reply ends its turn after one call, with no nudge and no failed run.
The engine's other stories run its walking chat, ended by its result
as jig's default chat is: what they test is a result's terminal path,
which temper's goals still take.

Its referees hold the root to jig's promises from outside (jig's
`domain/model.md`, section 11): the walking referee reads durable rows,
never the root's state; a referee per promise rejects a fault injected
against it; jig's conformance referee checks that nothing a party, a
host or the forge saw depended on a lost commit, and that every effect
the fake forge saw, in its own terms, is the one the core decided. Each
step's memory is measured against the world's ceiling (skein's
testing-strategy.md, 6).

## 12. Open questions

- **A party's fair share of the watch rooms** (section 9): the
  deployment's room is also one party's. Whether it should have less is
  `../protocol/configuration/README.md`, section 9's question. Default: the whole
  room, so one person's admitted tabs can each hold their watches.
- **What the root keeps between steps** (section 2). jig's root keeps
  nothing between steps but its children and the journal (jig's
  `domain/root.md`, section 7). Whether each value in flight moves to its
  owner. Default: kept, each bounded.
- **The store's fill mark and a read's bytes** (section 9): taken from
  `engine.limits` as the core's other limits are, or kept as the root's
  constants. Default: constants.
- **Procedures other than the forge's change:** the boundary carries a
  procedure's step out and its decision back (`Delivery::Procedure`,
  `PartyInput::ProcedureStep`), which temper's worlds use; temper's
  service steps none, and stops if one is asked. Default: none.
