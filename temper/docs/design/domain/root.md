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
  (section 3; `../protocol/README.md`, section 4).
- **It routes and translates; jig's core decides.** What commits
  together, what waits for a commit, the order of a restart and the check
  on every effect are the core's or the journal's (jig's
  `domain/root.md`, section 1). The root carries them through skein-lib's
  journal, its only way out (section 4).
- **It speaks for the forge to the core.** The forge's crates name no
  type of jig's core: the root makes, from what the forge says, what the
  core takes (an effect's description, a verdict, a procedure's step and
  its delegates, a hold's reason, a result), and back (`routing.md` and `root.md`, section 5).
- **Every run is on a worker.** temper runs no agent in the engine: the
  root has no hub child, and speaks jig's hub's link vocabulary to
  workers, which in standalone share the engine's process (section 2).
- **The owner keeps the value.** The root puts each output together from
  its owners' parts: an assignment from the core's part and the forge's
  workspace and brief sections, a call's answer from the core's part and
  the forge's (section 6).
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
├── temper-engine-domain-forge             the forge connector's top (forge/README.md, section 3)
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
  the engine by an in-process address (`../protocol/README.md`, section
  6), so the core is configured with its workers in process
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
├── store.rs, loads.rs  keys, records, ranges and pages (section 8)
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
`../protocol/README.md`, section 4; in standalone it shares its process
with the worker's (`../protocol/README.md`, section 6).

- **In, `Event`:**
  - the store's answers (`StoreInput`): a commit durable, with the
    store's fill; a commit failed; a page loaded, or not; a core read's
    page;
  - a worker's channel (`WorkerInput::Link`): a hello, a record up with
    temper's `Parts` beside an answer, a loss; and the protocol layer's
    hand-backs: a host call decoded into the root's `Tool`, a decided
    answer rendered, an ended run's outcome read;
  - a party's (`PartyInput`), among them signing in and out, a keyed
    request, an application request, reads, watches and their
    deliveries, a project's period;
  - accounts' events, and a generation of a git identity
    (`Event::Credential`, section 6.1);
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
  preparation (`Fact::PreparationFailed`, section 6.4), a brief's read
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
  engine ends with that reason in its log (`../protocol/README.md`,
  6.2).

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

Small total functions, one per pair of vocabularies, matching
exhaustively with no `_` arm (jig's `domain/root.md`, section 6).

- **Configuration,** `Config`, field by field as
  `../protocol/configuration/engine.md` says: `core` to the core
  whole, with jig's kinds' stock defaults and temper's kinds'
  declarations; `forge` to the forge, its namespace the deployment's id,
  with temper's kinds' stock defaults (`../protocol/configuration/stock.md`
  ); `identities`, each forge's git identity, to the forge. The
  landing rules and the forge's charter are settings of temper's kinds
  (`routing.md`, 6), not members of `Config`. The root holds its configuration to
  its limits as it is made, and a start that breaks one is refused,
  naming it (`../protocol/configuration/README.md`, section 2): one connector;
  the configured workers within the fleet's configured count and name
  bytes (jig's Membership); the deployment's provider other than 0, the
  forge's sign-in; each charter's run within an assignment's bytes; the
  forge's charter one of the core's; the resume limit within a row's
  bytes; each seeded owner's project naming its escalation role; the
  landing rules and the permission mappings within a row's bytes, and
  within the forge's judges' tables.
- **The landing policy:** the deployment's rules and each project's,
  each a `LandingRule` on a pattern of landing branches, whose gates
  every change into a covered branch asks for (forge/change/gates.md, 1). Nothing
  is compiled: the rules are values of temper's kinds, which the forge
  holds as it is told them (`routing.md`, 6) and judges (forge/change/gates.md, 2); the stock
  rules carry two standing requirements on the forge's merge, and no
  rule or policy of jig's changes with a landing rule
  (`../protocol/configuration/settings.md`, 3).
- **Names.** The root makes two names from the forge's own (the forge,
  the repository's id, what in it): a path for authority (`forge`, the
  forge's host, the owner, the name, then `branch` and its segments, or
  `pull` or `issue` and a number; forge/README.md, section 2); and, for a hold,
  three segments, the forge's number, the repository's id and the
  branch, which it reads back for the forge's branches only. A saved
  repository's tag becomes the task's saved resource, its path the
  forge's number and the repository's id (forge/adoption.md), and
  back.
- **The change procedure's decisions** become the core's procedure
  steps:

  | The change decides | The core hears |
  |---|---|
  | finished, merged | its result: the change and its pull request, the merge's id its words (below) |
  | a delegate: producing, a repair (CI, a semantic conflict, a gate's remarks, a person's request on the forge), a resolution, a gate | a batch of one delegate (below); the step waits |
  | ready; a wait | a wait; ready steps it again |
  | a queue repair | the repair started (`routing.md`, 4) and a wait, or, when it cannot start, held for its effects |
  | held: its branch moved or missing, its pull request retargeted or closed | held for drift |
  | held: stalled | held, stalled |
  | held: past its repairs, resolutions or updates | held by its procedure |
  | held: a delegate or an effect failed for good; an approval declined | held, an effect failed |

- **A landed change's result** is a change of the forge's connector and
  kind, its resource the pull request, and its words the merge commit's
  id as text: in lowercase hexadecimal, as a delegate's commits are
  (below), shortened to its first digits, as a short id is, when the
  task's contract bounds its words under the id's 40 or 64. Where the
  bound leaves room, verification words follow the id, whole or not at
  all: `Cleared by TIER run N.`, the repairing run and landing where
  applicable, or `Not cleared: ...` (`after-landing.md`, 6). These words
  precede the optional pull request line so they take room first within
  a task document's text window. Next comes the pull request in the
  forge's words (`../client/forge/family.md`, 1) and its address on the
  forge (`../client/forge/served.md`, 2), `pull request #7 of acme/app on
  github.com: https://github.com/acme/app/pull/7`, or the words alone
  where the forge gives no address or it is past its bound; the line
  cut whole, never in part, where the bound leaves no room for it.
  After the pull request line, where the bound leaves room for it whole,
  a line says `Landed: pull request #7 merged into main as 0123456789ab.`:
  the pull request's number, the branch it landed into and the merge
  commit's first twelve digits. The run that asked hears which pull
  request landed, since jig gives a parent the result's words and never
  its resource. The reported evidence of what the landing passed
  (`forge/change/README.md`, 2) follows, fitted to what the bound leaves. A client
  reads the id from the first line. A result's
  words are text that summarises it (jig's `domain/tasks/README.md`, section 3,
  and `jig/docs/design/client/domain/contract/documents.md`, 3), never the commit's bytes; the
  whole id stays the change's and its pull request's (`forge/change/README.md`, 2).
  The short id keeps any bound the task's creator chose: a bound under
  the id's digits is not refused, and the result never passes it.
- **A change's delegates,** in the core's terms: an agent task under the
  forge's charter for producing, repairs, resolutions, and agents' and
  checks' gates (the charter the change was made with, kept on it, which
  its procedure's delegation grants; `../protocol/configuration/settings.md`,
  2); a person task for the project's maintainers for a person's
  gate. Its parameters name the repository, its base and the change's
  branch, and a resolution's base commit or a gate's number and head,
  each commit as text: its id in lowercase hexadecimal, 40 digits for a
  SHA-1 id and 64 for a SHA-256 one, as the forge's ids are written
  (skein's `forge/forgejo.md`, 1.1), since a task's parameters are text (jig's
  `client/domain/contract/documents.md`, 3); its contract is a change, or a
  gate's verdict; its authority is the change's, its budget what the
  change has left, with no delegation of its own.
- **A collaborator's permission** on the forge (read, write, admin)
  becomes the role its project's configured mapping names, as a
  repository is adopted (forge/adoption.md).
- **A repository as a notes scope** is jig's resource scope on the
  forge's connector, its pattern the repository's path; the root
  translates nothing of it (jig's `domain/engine/briefs.md`, section 2).
- **The hub's link vocabulary** to the fleet's and back: a hello with its
  bounds, the worker's stop bound among them (worker/README.md, section 2), a
  run's phases, activity and counts. A run's live piece goes to the
  core as the run's text instead (jig's `domain/engine/views.md`):
  one of a kind other than text, or past the offsets the core keeps, is
  dropped and counted. A message relayed to a run
  keeps its kind in its words: a result, a proposal decided, a person
  task stalled and an effect not made each lead with what it is and its
  source's number.

## 6. Putting outputs together

### 6.1 An assignment

The core's part (the run, its attempt and conversation, its ask for the
run's live content, its charter, its brief's core sections, its start's
messages, its transcript's current window, the calls committed since its
last turn; jig's `domain/engine/executors.md`, 1) and the forge's (the
workspace, the brief's forge sections), put together while the task is
preparing, held until its claim commits:

- **The repositories:** for a change's delegate, the change's
  repository; for a run whose charter is an asking one, naming the asking family
  and no family but settings beside it, none, as
  for the status agent's (forge/attention.md, 3; `agent/ask-temper.md`); otherwise the
  project's home repository, those its spec names and those it saved
  work in; and, for every run but those with none, the project's context
  repositories, read only. Each item is the forge's: where it
  starts, and the branch it pushes to, the change's branch for a
  change's delegate and the task's own `r` branch for any other run
  (forge/branches.md; worker/README.md, 4.1), pushed only when the task
  holds that branch, or the change it works for does, and its authority
  grants the push; otherwise read only.
- **The workstream:** the change's task for its delegates, so a change's
  producing, repairs and resolutions find its checkout cached; the task's
  own otherwise.
- **Saving:** the task's saved-work branch (forge/branches.md) for a
  run that may write, so the unfinished work it leaves is saved and the
  next run starts from it (jig's `domain/engine/executors.md`, 4; worker/README.md,
  4.2).
- **Grants:** the LLM accounts its charter's models need, the core's;
  and each repository's forge's git identity, one of temper's
  application credentials, by number and generation (jig's
  `domain/engine/README.md`, 6.1). Their values come from the forge's records
  in jig's secrets' store, generation 1 at each start and the next at
  each change of value, each announced before the restart
  (`../protocol/credentials.md`, section 4). The forge's API token or App
  key, and a Forgejo forge's webhook secret, stay in the protocol layer,
  and are never granted.
- **The writes it claims:** the branches it pushes, the core's writer
  slots in the claim's commit and the forge's after it (`routing.md`, 3).

### 6.2 The forge's brief sections

- **For a change's delegate,** once the change's pull request is open,
  a pull section at the change's last head, required, naming its pull
  request before its contents (forge/change/gates.md, 1 and forge/change/repairs.md): against its base
  for a producer, gate or repair, with what landed in the base and the
  files in conflict for a resolution, and with what its clean update
  merged in for a semantic repair. A repair also has a required reason
  section: CI's failures, with the failed step and its output's end, for
  a CI or semantic repair; reviews' remarks for a gate's repair; the
  forge's reason for a merge-queue ejection. A read of one of a repair's
  two that fails becomes words in its place, the other standing for it
  (forge/change/repairs.md). Each kind has its budget (`ci`, `reviews`, `pull`).
- **For a task holding no run branch in its project's adopted home**
  (forge/branches.md), one section, required: why its runs read the
  home without pushing, bounded by its words. A goal's design writer
  (forge/branches.md, 3) needs no run branch to push, and its brief has none.
- **For a goal's design writer** (forge/branches.md, 3), one line beside that
  section's place: that `deliver` pushes the checkout's Markdown
  straight to the goal branch, merging its moved tip first, and that
  code goes through `change`. No other run's brief has it.
- **For each document its charter names** (jig's `domain/engine/settings.md`,
  9; `../protocol/configuration/settings.md`, 6), one section, optional,
  titled by the repository, the path and the heading: the file, or its
  part, at the branch's head when the run is prepared, within the
  `document` budget, or why it could not be read.
- **Planned with the core's.** The brief child places, cuts and orders
  them with the core's sections (jig's `domain/engine/briefs.md`, section 1);
  the forge gathers each and keeps it, cut to its allotment, and the root
  takes it once the brief is complete. A read refused at the forge
  client's admission (`Busy`) is retained and retried after the client
  reclaims, within the gather's deadline. A read sent but unanswered
  at that deadline leaves its required section late: the brief fails,
  counting as transient, and no attempt is spent. A section whose bytes
  the forge no longer has fails the brief too, counting as permanent
  (jig's `domain/engine/briefs.md`, 1.1).

Required forge sections take room from optional core sections alone, never
from a charter's document sections. The small root's eight section slots
hold three required core sections, two documents and three forge sections
(2026-11-04).

A charter document that names another connector or no forge branch is
answered missing at its gather without asking the forge. Its section is
titled `Document PATH` and says its read failed (2026-11-04).

### 6.3 A call's answer

The core's part (its status, an effect's entry, why it was refused) and
the forge's (what an effect made, a read's answer, the forge's refusal),
which the forge keeps under the call's key and the root writes beside
the call's record in the same commit. Asked again, the answer is put
together again from both. A run's read is answered through the door.

Read refusals also emit volatile root facts naming task, attempt, tool,
refusing layer and reason (`agent/agents.md`). Authority findings survive as
`ToolDenied`; local routing errors and forge errors retain their layer in
the fact. Call admission distinguishes call-record room from archive-read
room while keeping the host's busy response. The service drains the new
facts into operator notes as note room returns; a full diagnostic sink
changes no tool decision. The root's bounded fact queue counts overflow,
and no diagnostic variant enters the store or client wire.

Final forge-read rendering checks the settled tool-name and body bytes against
`core.call_answer_bytes`, then those bytes plus any retained typed answer
against `root.journal.transcript_bytes`. An exceeded bound returns an error
within both settled-answer limits, naming the bound, measured size and a
narrower read route (`agent/agents.md`). The root retains the original tool
identity for its `host-render` refusal fact even as it removes the flight.
A rendering size refusal is terminal; forge-client admission busy retains
its bounded retry path.


### 6.4 What a run left

- **Pushed heads,** from the answer's parts, kept until the core says the
  attempt has ended; then given to the forge against the branches the
  attempt was granted, which it reads afresh before their writer slot
  frees (forge/change/README.md, 2).
- **Saved work:** the repositories saved, as the task's saved resources,
  which its next run starts from.
- **A failed preparation** names its repository and why: what the forge
  lacks (the repository, the branch, the commit), that it refused the
  identity, or the head that did not merge into a base with heads merged
  in (`../protocol/channel.md`, section 5; worker/checkouts.md). The
  root reports it as a fact of the attempt: its task, its attempt, the
  repository's tag, the reason; a conflict it gives the forge too, for
  the change whose producer the run is (forge/change/questions.md, 1). Whether the
  task tries again follows the answer's class, as for any failure
  (worker/README.md, 4.3).
- **Past their bounds,** an answer's parts fail its run as invalid,
  whatever the answer says.

## 7. Its store vocabulary

- **`Key`, `Record`, `Write`** wrap the core's and the forge's, one
  variant each, the core's first; a write saves a record under its own
  key or erases a key (jig's `domain/engine/README.md`, 5.6).
- **No records of its own.** The deployment's record is the core's; a
  call's record is the core's part and the forge's (6.3).
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
  in its terms; a full table of them answers busy.
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

### 8.1 Restart

The core's script (jig's `domain/engine/restart.md`); the root does
each step the core asks for:

1. **The core's records:** every configured account wanted; the
   deployment's record, which binds the forge's namespace to the
   deployment's id and gives the journal its last durable commit;
   people, the configured projects and owners seeded at a first start;
   live tasks and goals' projections; claims; calls. A page past its
   bytes, or a record past the limits, refuses the start, naming its
   step.
2. **The inbox:** the census of ended owners.
3. **The forge's records,** `Range::Forge`, a page at a time; then the
   forge links them, judging none: a record whose forge, repository or
   task the restart does not hold is a writer's bug, and its link
   refuses the start, naming the record and what it links to, as jig's
   children's do (jig's `domain/engine/restart.md`).
   Before them, the forge loads its kinds' values from its family
   `Setting`, is given temper's stock defaults, and tells the core which
   differ (`routing.md`, 6). Once both are loaded, the root hands the forge the
   deployment's sole owner, the party jig's people answer as such, or none
   (jig's `domain/people/README.md`, section 4), before the forge's restore
   decision: a forge's link to a box's person holds only while the forge
   runs under the box and that person is the sole owner, and the forge
   clears any other in that decision (`forge/adoption.md`, 6). Then **the switch** (jig's `start::Phase::Switch`), at
   every start: over a generation the store holds pending (jig's
   `protocol/store/opening.md`, 2, step 7) the root asks the store to switch
   and goes on once it answers; then it commits the start's wires (jig's
   `protocol/store/records.md`, 3). Nothing leaves the process before that
   commit is durable, the accounts' driver made after it (jig's
   `protocol/llm.md`, 5.1); over a pending generation the restores'
   commits before it land in the copy alone, and a refusal before the
   switch leaves the generation patched from active (jig's
   `domain/engine/restart.md`). A rehearsing root
   (`../protocol/binaries.md`, `temper-box upgrade --check`) ends
   before this step, reporting the restores whole.
4. **Runs adopted:** the claims to the fleet, those on in-process
   workers settled at once (section 2).
5. **The forge reads afresh,** then **settles its outbox** (forge/README.md,
   section 9).
6. **The calendar is given the periods** (`routing.md`, 7): for each restored
   project, its current period's number, start and next start, and its
   policy's spend per period, from the core; it says what each needs and
   arms its deadline.
7. **Decisions open:** every due task is activated.

Until decisions open, the forge's hints are dropped and its timers fire
from its restore on.

### 8.2 Stopping

A stop settles first (`../draft/upgrades.md`, section 8), then the core
drains as jig's `domain/engine/restart.md`, 1 says.

- **Settling.** Told to stop, the root tells the forge to settle
  (forge/README.md, section 4) and holds the core's stop until the forge says
  it has. Meanwhile:
  - the forge starts no write and retries none: what it was told to
    make stays in its outbox, durable, for the next start;
  - the writes it sent before finish as ever, each with its answer
    and, for a review, a merge or an update, the read that proves it,
    and each outcome commits as any outcome does;
  - the root activates no due task, so no run is placed and no
    procedure steps, and answers every run's call busy, since a call may
    start an effect: its host sends it again, to the next engine;
  - workers' turns and answers, and the commits they make, go on; the
    listeners already refuse new openings and parties' requests
    (jig's `protocol/service.md`, section 5).
  With no write lacking its outcome, the forge has settled at once.
- **Bounded** by its client's (skein's `forge/client.md`, 7.8): twice
  the writes' lifetime and their clock margin from the stop
  (`forge.client.lifetime_ms`, `forge.client.clock_margin_ms`), by
  which each write sent before has its answer and its proving read, or
  their timeouts. Past it, the forge names each write still without its
  outcome, by its entry and its task, and the stop goes on unsettled
  (`Request::Unsettled`);
  the service prints each as it ends (`../protocol/binaries.md`
  ), and the next start settles it as its class says (forge/README.md,
  section 9).
- **A second signal** aborts the wait, as it aborts the drain.

Then the drain. What the root adds to the core's:

- **the forge's events** go back to their owner undecided
  (`Request::Busy`), and no change steps;
- **a relayed call** is answered busy;
- **a committed forge entry** is not made: its notification is retired,
  its outbox row stays durable, and the next start's settlement makes it;
- **`Drained`** once nothing waits for a commit.

### 8.3 Each service restarted alone

An engine, a worker or a forge may each stop, fail, or be restarted on
its own, in any order, and whatever needed it carries on once it is
back:

| Restarted or away | What carries on |
|---|---|
| the engine | each worker keeps hosting its runs within its engine-away bound, redials, and its hello waits for the stored settings; the restart adopts each run its store holds as claimed, by its attempt, and the run goes on (`worker/README.md`, section 2; jig's `domain/engine/restart.md`, step 3 and `root.md`, section 7). Past the bound a run is interrupted, never failed, and resumes. The forge reads afresh and settles its outbox, each write in flight found or sent again as its class says, a goal's own branch made again (`forge/README.md`, section 4, and `forge/branches.md`, section 1, and `forge/README.md`, section 9) |
| a worker | its runs, stopped or lost, are resumed on it or another worker from their transcripts and saved work, their start messages kept (`worker/README.md`, section 2) |
| a forge | its reads and writes wait out the outage and its CI is read again (`forge/keeping-up.md`, 5); an owned Forgejo on a box gets its sign-in source once the engine answers, without its own restart (`../protocol/setup/box.md`, 2) |

The root itself adds no step: steps 4 and 5 of 8.1 are where the engine's
side happens, and the rest is the worker's, the forge's and the box's.

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
    `in_flight` and `writes` (`../protocol/store.md`, 4); with the door's
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
(`../testing/README.md`, section 2):

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
  (`../testing/README.md`, 2.1).

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
limits, or for a malformed page. The forge's own stories are forge/README.md,
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

- **A public read returning 503 after a project is made** (`routing.md`, 7, `root.md`, 8.1):
  the `core-after-project` request reports it alongside clients whose
  links never come up, across a restart. A client rejecting `me` for too
  many memberships explains those links, but not the service refusing
  that public read. Default: no further core repair without evidence of
  a shut gate; the service's reason line (`../protocol/README.md`, 6.2)
  must identify what holds it before attributing another occurrence to
  the calendar, startup or the journal.
- **A party's fair share of the watch rooms** (section 9): the
  deployment's room is also one party's. Whether it should have less is
  `../protocol/configuration/README.md`, section 9's question. Default: the whole
  room, so one person's admitted tabs can each hold their watches.
- **What the root keeps between steps** (section 2). jig's root keeps
  nothing between steps but its children and the journal (jig's
  `domain/root.md`, section 7). Whether each value in flight moves to its
  owner. Default: kept, each bounded.
- **The forge's work done in the root:** choosing a run's repositories,
  planning the forge's brief sections from a change's state, making a
  change's delegates (sections 5 and 6). jig's connectors gather their
  workspace items and plan their sections themselves (jig's
  `domain/connectors.md`, sections 9 and 10), but temper's forge names no
  type of jig's tasks. Default: translation in the root.
- **The store's fill mark and a read's bytes** (section 9): taken from
  `engine.limits` as the core's other limits are, or kept as the root's
  constants. Default: constants.
- **A failed preparation's reason on the task's page** (6.4): jig's core
  keeps an attempt's end as a class only; temper would ask jig to keep
  its detail. Default: the fact alone.
- **Procedures other than the forge's change:** the boundary carries a
  procedure's step out and its decision back (`Delivery::Procedure`,
  `PartyInput::ProcedureStep`), which temper's worlds use; temper's
  service steps none, and stops if one is asked. Default: none.
