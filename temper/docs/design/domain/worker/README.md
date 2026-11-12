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

- `checkouts.md`: workspaces, preparing, delivering and saving, and the checkout's open questions.
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
  messages, take them and park (smith's `domain/run.md`, section 6): it
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
  worker link in its protocol layer (jig's `protocol/channel/README.md`,
  section 9); it says its hello first; the engine holds the claim and
  fences attempts; the worker keeps turns and answers until the engine
  acknowledges them, once committed (jig's `domain/hosts/hub.md`, 1 and
  4). temper adds its parts beside jig's records: the workspace items
  down, what a run left on the forge up (4.1; `../../protocol/channel.md`,
  sections 4 and 5).
- **The engine never reads a repository.** It works from what the forge
  connector reads; what a checkout holds (a repository's checks, its
  `AGENTS.md`) is for the run to find (../agent/README.md, section 2).
- **The stop bound** the hello states is jig's (jig's `domain/hosts/hub.md`,
  6), with temper's push as the delivery and temper's save as the save:
  the time the link takes to notice the loss, then the grace, then the
  longer of the time smith's agent may take to be gone after a stop and a
  push's deadline, then a save's commit and push. The worker's
  configuration derives it (`../../protocol/configuration/README.md`, section 3);
  the domain takes it whole.
- **Losing contact and shutting down** are the hub's (jig's
  `domain/hosts/hub.md`, 6 and 7). Past the grace the worker cancels its
  runs itself, saving their work first. Shutting down, it still says its
  slots and refuses every assignment as busy; it gives up the turns and
  answers it keeps, counted, only once no run is left and the engine is
  still out of reach past the grace.
- **An engine's restart costs a run nothing.** The grace is the
  engine-away bound, `graces.grace_ms` in the worker's configuration
  (`../../protocol/configuration/README.md`, section 3), sized to outlast an
  engine's restart. While it runs, the worker keeps hosting every run,
  keeps what they send, and redials no further apart than a quarter of
  the grace; it stops only once the engine admits a hello (jig's
  `protocol/channel/README.md`, 9.1). The restarted engine holds that hello
  until its stored settings are restored, so its check is the store's
  (jig's `domain/engine/fleet.md`), then adopts each attempt the
  worker lists that its store holds as claimed: the run goes on in the
  same attempt, its kept turns and answers committed as they arrive, and
  its relayed calls answered on the new channel (jig's
  `domain/hosts/hub.md`, 6). Nothing is cancelled and nothing starts
  again.
- **A start that fails is the engine still away.** An engine that
  exits before it admits a hello, refused at its start's checks or
  failing as it starts, and is restarted, is one absence to the worker:
  every failed dial, refused opening and channel lost before the
  engine's first record leaves the grace's clock running from the first
  loss (jig's `protocol/channel/README.md`, 9.1), and the start that
  admits the hello within the grace adopts every run, as above, however
  many starts failed before it. One of the worker's dials is due at the
  grace's end less the link's handshake deadline (jig's
  `protocol/channel/README.md`, 9.1), so an engine back by then, however
  late in the grace, finds the worker's hello before the runs are
  cancelled.
- **Past the bound, an interruption.** A run cancelled for lost contact
  saves its work first and answers `Cancelled(Contact)`; the engine ends
  the attempt interrupted, with no try spent, and the next attempt, on
  any worker, resumes from its transcript (jig's `domain/engine/executors.md`,
  2) and starts its checkout from the work it saved (`checkouts.md`).
- **A worker's restart loses no work.** Stopped, it cancels each run for
  shutdown, saving its work first, and answers `Cancelled(Shutdown)`, an
  interruption as above, before it gives up its link (jig's
  `domain/hosts/hub.md`, 7). Killed, it saves nothing more: the engine
  holds its runs for its grace, then ends them lost, and each next
  attempt resumes from its committed transcript and its last saved work
  (`checkouts.md`). Either way the next attempt may be on this worker once it
  is back, or on another. A run's start messages leave its task's inbox
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
  holds (jig's `protocol/channel/README.md`, 8, *Reading never waits for the
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

### 4.1 The assignment

What the engine gives the worker for one run is jig's assignment (jig's
`domain/hosts/README.md`, section 2; jig's `domain/engine/executors.md`, 1), with
temper's workspace items beside it:

- **Run and attempt.** The engine's names for the run (its task's number
  and the run's) and for this attempt at it; every message about the run
  carries both.
- **Workspace.** A workstream, which names the checkout so later runs of
  the same work find it cached: the number of the task that holds what is
  written (jig's `domain/engine/executors.md`, 1), by which the checkout keys its
  cache. And the repositories, each with:
  - its deployment tag, by which what the run left names it;
  - its name, the directory it sits in and what the agent calls it, one
    safe path component;
  - its remote: the URL git fetches from and pushes to, which the
    engine's protocol layer makes from its forge's configuration
    (`../../protocol/channel.md`, section 4), so a worker needs no forge
    configuration;
  - its git, where its forge has its own: the program every git
    operation on it runs, a fake forge's, which the domain carries to
    each operation and never reads; otherwise the worker's own git
    (`../../protocol/git.md`, section 3);
  - where to start: a base branch, a branch, a commit by its full object
    id, saved work, **a merge in progress**: a branch, and the commit
    of its base to merge into it, or **a base with heads merged in**: a
    base branch, and the commits to merge into its tip, one by one in
    their order, each by its full object id (a change's early start:
    `../forge/change/questions.md`, 1) (`checkouts.md`);
  - whether it may be written, and if so the branch a change is pushed
    to, and the head that branch must still be at, if the assignment
    names one (`checkouts.md`);
  - a landing branch, the branch its change lands on, for each writable
    repository of a change's producer, whatever its start, and of a goal's
    later run; no other item names one. Prepare pins its tip as
    `refs/temper/landing` for the project's checks (`checkouts.md`); a writable
    item that names none (a repair, a resolution, a merge job, a chat's
    run) has the head its delivery replaces pinned there instead
    (`checkouts.md`, *The replaced head's pin*). For an
    inherited producer that starts on its push branch it is also the
    comparison for existing work, separate from the checkout start and the
    push's expected head; and on a merge in progress it marks a producer's
    start, as a goal's later run's does (`checkouts.md`);
  - the identity for every git operation on it: one of temper's
    application credentials (jig's `domain/engine/README.md`, 6.1), granted
    with the assignment, which the protocol layer maps to a token and a
    commit author; and on GitHub the account whose entry of that
    identity's value authenticates git on it (`../../protocol/channel.md`,
    sections 4 and 6), carried with the identity into each git operation
    that reaches the forge, which the domain copies and never reads.
- **Saving.** The assignment's typed resume policy requests saving
  unfinished work from the first attempt, before a transcript exists
  (a chat's, a coding agent's: ../agent/charter.md). Each writable
  repository has its own optional saved-work destination, belonging to
  the running task (../forge/branches.md), and the next
  attempt starts from the saved work: the tree follows the transcript,
  so a retry that remembers a write finds the file it wrote, and a run
  that starts fresh from its brief starts from a clean checkout. A run
  that ended with its change landed saves nothing, and one holding a merge
  in progress saves it as a merge commit only once its conflicts are resolved
  (step 7).
- **Charter.** smith's, as the engine wrote it (../agent/charter.md),
  carried as bytes. The worker gives it to the agent with the workspace
  it prepared: where the repositories sit, which may be written, and for
  a merge in progress the files in conflict (smith's `domain/run.md`,
  3.2).
- **Transcript,** for a run that resumes: its current window, and the
  calls committed after its last turn (jig's `domain/engine/executors.md`, 2;
  jig's `domain/hosts/hub.md`, 4). Across the link, those calls' answers
  fit its room: the newest whole, each older host answer cut to words
  saying the call ran and how many bytes its answer had (jig's
  `protocol/channel/README.md`, 8, *A resumed start fits its room first*), so a
  producer whose program worked an hour since its last turn resumes,
  told of every call it made. A delivery among those calls comes with
  its evidence, smith's record of the push's terminal, which must decode
  and agree with the outcome the engine committed (4.3;
  `../../protocol/channel.md`, section 2). The worker carries the rest
  opaque, within its limits.
- **Grants.** The credentials the run's LLM accounts and git identities
  need, as names the protocol layer fills (`../../protocol/credentials.md`,
  sections 3 and 4). The hub hands the agent its LLM accounts' grants
  only; the worker keeps the git identities' for its own git operations.

Its sizes and names are checked at the entrance against the worker's
limits, and so is a repository named twice: what is beyond them is
refused as an invalid start that names it (jig's `domain/hosts/hub.md`, 3).
A run with no repository needs no workspace: the worker prepares nothing
and starts smith with none (jig's `domain/hosts/README.md`, section 6). A chat
that names no repository runs so.

### 4.2 Lifecycle

A hosted run's lifecycle is the hub's, in nine steps (jig's
`domain/hosts/hub.md`, 2). What the worker puts in them:

- **Step 2, prepare:** the checkout prepares the workspace from the items
  (`checkouts.md`).
- **Step 3, start:** smith's agent process (section 5), with the charter
  and the transcript decoded by hand-out, and the workspace as the
  checkout prepared it.
- **Step 4, relay:** smith's `deliver` is a push, which the checkout
  serves, one at a time (`checkouts.md`; ../agent/README.md, section 5); every host tool
  goes to the engine, and its answer comes back.
- **Step 6, stop:** a push in flight goes on to its end, and what it
  lands counts (`checkouts.md`).
- **Step 7, save,** if the assignment asks, at park, at an unfinished end
  and at cancel: commit what the writable repositories hold and push it
  to each repository's own saved-work destination, where one is named,
  so a later run on any worker starts from it.
  A run that ended with a landed change has nothing left to save, and
  nor has one whose agent never started. A run that started from a merge
  in progress saves it as a merge commit once its conflicts are resolved;
  unresolved conflict markers refuse the save. One that delivered mid-run,
  then parked, failed or was cancelled saves its completed work, so its
  next attempt retains that delivery's history.
- **Step 8, release** the workspace to the cache (`checkouts.md`).
- **Step 9, answer:** jig's answer, with temper's parts beside it: what
  the run left on the forge (the last commit landed in each repository,
  the head a pull request is to show) and how its save went, each
  repository named by its tag (`../../protocol/channel.md`, section 5).

A push that lands while its run winds down is the run's result (smith's
`domain/run.md`, 8.2). The hub arms no timers: each wait is bounded
below it, by the deadlines of git operations, the agent's watchdog and
wall time, and the grace of cancel, then kill.

### 4.3 Failures

jig's types (jig's `domain/hosts/hub.md`, 3), with temper's detail:

- **Refused** at the entrance: busy, or invalid, saying what is beyond
  the limits or which item the workspace cannot take. Once prepared, a
  start smith refuses, or whose charter, transcript, call names or
  delivery evidence do not decode or agree, is refused as invalid too,
  which the engine counts as permanent (jig's `domain/tasks/lifecycle.md`, 6). A
  start smith refuses as busy, an agent or a run it has no room for, is
  refused busy, as at the entrance (jig's `domain/hosts/charters.md`).
- **A start the link cannot carry** never reaches the worker. An
  assignment or job whose frame is past the room the engine's stream
  grants at once, `io.largest_room` (its `io.output`), is answered on the
  engine's side by jig's workers' component, refused as an invalid start
  whose words name the record, its frame's bytes and the room (jig's
  `protocol/channel/records.md`, 2, and `jig/docs/design/protocol/channel/README.md`, 8). The run fails as any invalid start
  does, with serve's operator line below, and serve writes the link's
  `worker unsent` line too (`../../protocol/cluster.md`, 3). The link, its
  pings and the worker's other runs go on; nothing waits for an answer
  the worker was never sent. A resumed start's committed calls are cut
  to the room first (4.1, *Transcript*), so a start fails so only when
  it is past the room with every answer cut, by its charter, window,
  inbox and grants, as `standard` admits by its inbox alone. Such a
  start fails so, with its numbers, rather than silencing the link
  until the worker's silence deadline drops it and every run on it.
  Limits under which even the smallest resumed start, every call
  `core.call_records` admits cut beside an assignment's fixed fields,
  passes the engine's room are refused at start, naming the bound
  (`../../protocol/configuration/limits.md`).
- **An answer past the agent's reply bound** (`Limits::reply_bytes`,
  smith's `host_reply_bytes`) is given to smith as its `TooLarge`,
  never as a body its host codec would refuse: such a send would
  leave the call unsent and the agent draining.
- **Preparation failed:** transient (the forge could not be reached, the
  worker's side failed or ran out of time, another run of the workstream
  holds its checkout, or every checkout is held), or permanent until
  something changes on the forge, naming the repository: the forge does
  not have it, a branch or a commit, or it refuses the identity.
- **The run failed,** the **agent failed,** or the run was
  **cancelled,** as jig's say, each with bounded detail for operators
  that no LLM is shown.

When the hub accepts a run's failed or refused answer, serve writes one operator line
on stderr, which the box keeps in `<data>/box/logs/serve.log`
(`../../protocol/setup/box.md`, 2). The line names the task, run, attempt and
failure class or typed refusal reason. A refusal distinguishes a busy host,
an oversized charter, transcript, grants or messages, and an invalid start;
the invalid start names its cause and carries its decoding words. The line
includes the last at most 4,096 bytes of the worker's bounded failure detail
or start words. Newlines, control bytes and non-ASCII bytes are escaped,
so the record is one line. Acceptance emits it once; resending a kept
answer emits no second line. A full operator-note sink retains the diagnostic
until the sink drains. In-process and socket workers share this
path. The evidence is volatile operator output, outside turns, facts,
`TaskResult` and the review's reason sentence; no wire or stored field
carries an extra copy.

**A configured model-call bound** is reported as jig's
`RunFailure::Exceeded(Exceeded { bound, value })`. Smith's
`CompletionFailure::Limit { which, bound }` supplies `value` unchanged:
the configured value in the bound's own unit, never a measured size.
The worker classifies both `Limit` and `Limit::Path(Cap)` exhaustively,
without a wildcard, so an added smith cap requires a classification.

| Smith limit | Jig bound | Unit |
|---|---|---|
| `Limit::Path(Cap::Reasoning)` | `Reasoning` | bytes, per item or in all |
| `Limit::Path(Cap::Metadata)` | `Metadata` | bytes |
| `Limit::Path(Cap::OutputItems)` | `Items` | answer items |
| `Limit::Path(Cap::Answer)` | `Answer` | bytes |
| `Limit::Path(Cap::Tools)` | `Tools` | tools |
| `Limit::Path(Cap::HistoryItems)` | `HistoryItems` | request messages and blocks, counted together |
| `Limit::History` | `History` | a retained-history ceiling on messages, blocks or bytes; the tag does not identify which |
| `Limit::Window` | `Window` | tokens |
| `Limit::Path(Cap::Output)` | `Output` | declared output-ceiling tokens |

For `History`, the value is the ceiling that compaction could not bring
the retained history under. It is carried unchanged, without assigning
it a unit the tag does not identify.

- **The included bounds name the call's allowance.** The answer's
  reasoning, metadata, items and bytes, and the request's tools, history,
  window and declared output ceiling become `Exceeded`. The tasks child
  holds the run at once without counting a try (jig's `domain/tasks/lifecycle.md`,
  6). The classification does not infer a measured size from the
  failure or require proof that every provider response will repeat.
- **Other limits stay `Model`.** `Cap::Request` stays `Model` rather than
  being identified with a retained-history or model-window bound.
  `Cap::RequestHead` depends on the credential. `Limit::Call` and
  `Limit::Message` have no production constructor. `Cap::ResponseHead`,
  `ResponseFields`, `Skip`, `Field`, `Retained`, `Tokens`, `Depth`,
  `Strings`, `Number` and `ErrorBody` can describe framing or a transient
  error body, rather than the call's configured allowance.
  `Cap::AccessToken` depends on credentials.
  `Limit::Pool` is a connection reservation, which can depend on
  credential-bearing request headers; `CompletionFailure::ApplicationLimit`
  also covers temporary call-slot refusal. None is classified as one of
  the nine call bounds. Other completion failures retain their `Model`
  classification.

## 5. Agent processes

The agent child is smith's host domain, `smith-host-domain` (smith's
`domain/host.md`, section 4): jig's agent processes (jig's
`domain/hosts/README.md`, 5.1). One process per run, whose process tree is the
run's containment boundary (section 8); one channel over its pipes,
whose rules smith's host domain enforces; a watchdog on progress and a
bound on wall time; cancel, then kill; the slot back only once the
process has exited and its tree is empty. temper's choices:

- **Agent processes, always.** Every run is a process of its own, also
  when the engine and the worker share a process
  (`../../protocol/README.md`, section 6); `smith-inline-agent` is not used
  (`../../protocol/llm.md`, section 7).
- **temper's binary is the agent:** the box's `temper-box internal agent`,
  or a deployment's `temper-agent` (`../../protocol/binaries.md`, 3, and `../../protocol/README.md`, 5
  ), smith's agent shell. temper builds no code into it.
- **No credentials in its environment.** The tokens its LLM calls use
  come as grants on its channel (`../../protocol/credentials.md`,
  section 3); git's never reach it.
- **The repository's checks are a long operation** the run reports, so
  the watchdog stretches to their span: a two-hour test suite is not a
  hang.
- **A check's build directory is held for the whole check.** A
  repository's check builds and then runs what it built, after cargo has
  exited. A deployment whose cargo on the agent's `PATH` lends a build
  directory only while a cargo command runs could reclaim it between the
  two, and a test's executable would vanish under the check. The worker
  gives a check nothing but the agent's `environment`
  (`../../protocol/configuration/README.md`, section 4), so the hold is the
  check's and that environment's: one hold for the check, taken by its
  outermost entry point from before its first build to its last test,
  released at its end or by its process's death. fabrig's own check takes
  the hold its environment names in `TIERS_HOLD` (`tools/tiers/design.md`,
  section 3; `tools/pre-pr/design.md`, section 4), and a deployment that
  lends build directories names its hold there.
- **What a test writes reaches the checkout.** A run's commands and
  checks run in place, in the workspace's checkout (smith's
  `protocol/agent.md`, section 2), and its delivery takes that tree, so a
  file a test writes under its package, a recorded fixture re-recorded by
  `SKEIN_CODEGEN_WRITE=1` among them, is in the change as any edit is,
  once the test finds its package when it runs
  (skein/docs/foundation/testing-strategy.md, section 8, *A test reads its
  paths when it runs*). A path compiled in names the checkout the
  executable was built from, which a build cache shared between
  checkouts, or a target directory kept apart from the checkout, can make
  another one's: the file goes there, and never reaches the change. A
  merge job re-records the recordings it knows in its own job's checkout
  and delivers them itself (`../../protocol/tooling.md`, 4.1).
- **Its limits** are smith's host limits, and the bounds the hello
  states, from the agent's configuration (`../../protocol/configuration/README.md`,
  section 4). The one number that isn't the agent's is how many agents
  the host holds at once: a run is one agent process, so that number is
  the worker's slots, set where the worker composes its limits, and
  nothing in the agent's configuration raises it. Each of the host's
  other limits is one agent's, whatever the slots; an agent process
  holds one run.
- **Its reply bound,** in the hello's bounds: the agent's
  `host_reply_text`, its `agent.json` `profile.declared.tool_payload`,
  the same figure as the worker's `Limits::reply_bytes` (4.3), so that
  the engine sizes a task read and renders an answer for this worker's
  runs within what its agent takes (`../routing.md`, 2; jig's
  `domain/hosts/README.md`, section 2). Every temper worker names it, the
  engine's in-process local agent among them.
- **What rises with its slots:** the worker's `hub.slots`, the engine's
  `fleet.slots` and the checkout's `checkout.workspaces` are each at
  least the slots, and `git_component.operations` at least the
  workspaces. A start refuses a worker past any of them it can see,
  naming the limit to raise. An engine alone can't see a worker's slots:
  it uses them up to `fleet.slots` and turns away a hello listing more
  runs than that (jig's `domain/engine/fleet.md`).
- **Its program endpoints,** in the hello's bounds: the endpoint numbers
  the agent's configuration serves by a program, its `program_endpoints`,
  as jig's reference launcher declares them (jig's
  `domain/hosts/charters.md`, 1). temper's stock agent serves none, an empty
  list that proves no endpoint is a program; a chat whose ordinary
  source is one is refused before it starts, `HostRefused` in its
  status (`../agent/charter.md`, 5).

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
layer is the worker's service (`../../protocol/README.md`, section 5): jig's
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
  the engine by an in-process address (`../../protocol/README.md`,
  section 6).
- **Agents:** spawning the agent within a deadline, in a contained
  process tree, with proof that the tree is empty after it stops; where
  skein gives no contained trees (jig's `protocol/README.md`,
  section 10), each agent in a process group of its own, which a stop
  signals whole. smith's channel over its pipes (smith's host half); an
  environment without forge credentials. smith's payloads (the charter,
  the transcript, turns, facts) are decoded and encoded by hand-out (jig's
  `domain/hosts/hub.md`, 8). The root completes the decoded start with
  where the repositories sit, as the checkout prepared them, and, for a
  merge in progress, which files are in conflict.
- **Git:** each typed operation is a fixed sequence of git invocations
  under the operation's one deadline (`../../protocol/git.md`, section 4),
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
  (`../../protocol/configuration/README.md`, section 3). Making one empties it
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
  delivery; a withheld commit holding the assignment.
- **The one-process world** (`tests/one-process`): temper's services on
  skein's simulator, the worker's agents smith's processes over skein's
  fake LLM (`temper/docs/design/testing/README.md`, 2.5).

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
- **A worker past what no document raises.** A standalone start counts its
  slots and git operations against the process's sockets, a constant
  (`../../protocol/configuration/limits.md`), and the link's caps are `u32`s
  that grow with the slots, so past them a start's refusal names a limit no
  one can raise, or none. A refusal saying what to lower, a key for the
  sockets or wider caps would close it. Default: as they stand, until a
  deployment asks for more than they hold. The user's to decide.
- **jig's:** saving periodically, and a worker's failed dials before its
  first channel (jig's `domain/hosts/README.md`, section 10).
- **A hold naming an agent run's last failure.** An agent task held
  `out of tries: run failures` names no failure: the reason is 4.3's
  operator line alone. Job failures carry their bounded cause into the
  hold (`jobs.md`). Default for an agent run: the line alone, which `task
  show`'s way out points to and `temper help hold` and `temper help
  server` say how to read, until that proves too little. Keeping agent
  text means one last failure per hold, replaced on each failure, with
  a byte bound and readers the user decides first. The operator line's
  4,096-byte tail is no choice of that stored bound. Task readers could
  then see text now confined to the engine's log, including tool output,
  paths, provider errors or echoed credentials, which cannot be reliably
  redacted. Restricting it to owners and maintainers needs a reading path
  of its own. An agent ending's producer and its payload must carry the
  cause, as the job's do. The user's to decide.
- **What a deployment's operators may read of a run's prompts.** A run's
  prompts hold repository text and anything an agent read, and a real
  design of keeping them starts from who may read them
  (`../../draft/dogfood-hacks.md`, 2.12 and 2.13). Default: nothing past what an
  agent's document asks its trace to keep.
- **A start past the link's room, made to start.** A resumed run's
  committed calls are cut to the room (4.1, *Transcript*), but
  `standard` admits a start past its 1,048,576-byte room by a full run
  inbox alone, 82 messages of up to 262,144 bytes (4.3). Making such a
  start run takes a figure or a form (jig's `protocol/channel/README.md`,
  section 13). temper/migration answered on 2026-11-09 that a later plan
  chooses from the deployment's refusal line: a start near the room
  sizes `io.output` from the configured room, and one far past it bounds
  what the start carries, as the committed calls' cut does. Default: it
  fails with its numbers.
