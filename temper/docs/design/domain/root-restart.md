# Restarting and stopping the root

This document holds restarting and stopping the root, split from [`root.md`](root.md).

## 1. Restart and stopping

### 1.1 Restart

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
   clears any other in that decision (`forge/adoption-person.md`, 1). Then **the switch** (jig's `start::Phase::Switch`), at
   every start: over a generation the store holds pending (jig's
   `protocol/store/opening.md`, 2, step 7) the root asks the store to switch
   and goes on once it answers; then it commits the start's wires (jig's
   `protocol/store/records.md`, 3). Nothing leaves the process before that
   commit is durable, the accounts' driver made after it (jig's
   `protocol/llm.md`, 5.1); over a pending generation the restores'
   commits before it land in the copy alone, and a refusal before the
   switch leaves the source generation active (jig's
   `domain/engine/restart.md`). A rehearsing root
   (`../protocol/binaries.md`, `temper-box upgrade --check`) takes
   this step inside its scratch and runs on with no network to the
   opening (step 6), where it ends, reporting the restores whole, the
   opening and what the forge would catch up (jig's
   `protocol/store/opening-rehearsal.md`, 1).
4. **Runs adopted:** the claims to the fleet, those on in-process
   workers settled at once (`root.md`, section 2).
5. **The calendar is given the periods** (`routing.md`, 7): for each restored
   project, its current period's number, start and next start, and its
   policy's spend per period, from the core; it says what each needs and
   arms its deadline.
6. **Decisions open:** every due task is activated, and the service
   admits its clients.
7. **The forge catches up** (`forge/catching-up.md`): as the core asks
   it at the opening, the root gives the forge `Event::CatchUp`, and
   passes its `RestartDone` reports of its fresh reads and its settle
   to the core, which records them by connector; the forge reads afresh
   and settles its outbox (forge/effects.md, section 2) while the engine
   serves. The root's own procedures on the forge's facts, the trains'
   forming (`form_landings`) and the limit holds' reconcile, wait for
   the forge's fresh reads (`forge/catching-up.md`, 4).

Until decisions open, the forge's hints are dropped; its own timers
fire from the beginning of its catching up.

**A step that runs long.** The root keeps the time the core asked for
each step, and what the step waits on: the store's page or switch, its
commits in flight, and in step 5 the forge's reads and finds still owed
(`forge/effects.md`, section 2). Past the report bound, and again at each
doubling of the step's age, it reports them by kind, with counts and the
oldest by name, as a fact the service writes
(`../protocol/service-process.md`, 1.3). The report is no deadline: the step
goes on as before.

### 1.2 Stopping

A stop settles first (`../draft/upgrades.md`, section 8), then the core
drains as jig's `domain/engine/restart.md`, 1 says.

- **In every step of the start** (1.1) a stop is heard: the service
  routes the root's stop as soon as it is told, opened or not, before
  anything else it would hand the root (`../draft/reconciling-start.md`,
  4). What it does depends on the start's mark, the wires commit of step
  3:
  - **Before the mark,** the start ends at once. The root tells the core
    `Stopping` alone and the forge nothing, since nothing has been read
    or written to settle; the core says it has drained once the start's
    commits in flight are answered (jig's `domain/engine/restart.md`, 1,
    *A stop during the start*). The service makes none of the components
    that wait for the mark and closes each at once as closed (jig's
    `protocol/service.md`, 5, step 4), the store last, and the engine
    ends drained. No forge is read, no secret served, no run adopted, no
    worker told; the next start begins again, a pending generation still
    pending (jig's `protocol/store/opening.md`, 2, step 7). A stop heard
    while the wires commit is in flight is before the mark: the commit is
    answered, and nothing that waits for the mark follows it.
  - **After the mark and before decisions open,** the start ends at the
    end of the step the root serves, then settles as below, the restart
    word kept and said for the runs step 4 adopted. Adopting runs and
    opening decisions end at once; so do reading the forge afresh and
    settling its outbox, whose reads and finds are the next start's to
    do again (`forge/effects.md`, section 2), a write the settle sent
    finishing within the settled stop's bound.
  - **After decisions open,** the settled stop below, as before.
  - **While the store opens,** before the root's start, a stop ends the
    engine once the store lets it close, with no step of the start.

  A second signal aborts in every one of them, as below.
- **Settling.** Told to stop, the root tells the core it settles
  (jig's `domain/engine/restart.md`, 1, *Settling before the stop*),
  then tells the forge to settle (forge/effects.md, section 1), and holds
  the core's stop until the forge says it has. The service routes the
  root's stop before anything else it would hand the root once told, so
  that nothing is decided between the signal and the settling. Meanwhile:
  - the forge starts no write and retries none: what it was told to
    make stays in its outbox, durable, for the next start;
  - the writes it sent before finish as ever, each with its answer
    and, for a review, a merge or an update, the read that proves it,
    and each outcome commits as any outcome does;
  - the root activates no due task and the core places nothing, so no
    run or job is placed, not even where a worker's slot frees, and no
    procedure steps, jig's or the forge's own (a change's step, a
    closer's): what would have stepped is decided afresh at the next
    start (1.1). The root answers every run's call busy, since a call
    may start an effect: its host sends it again, to the next engine;
  - workers' links stay up, so their grace has not begun
    (jig's `domain/hosts/hub.md`, 6), and the runs they host go on;
  - workers' turns and answers, and the commits they make, go on; the
    listeners already refuse new openings and parties' requests
    (jig's `protocol/service.md`, section 5).
  With no write lacking its outcome, the forge has settled at once.
- **Bounded** by its client's (skein's `forge/client-effects.md`, 1.8): twice
  the writes' lifetime and their clock margin from the stop
  (`forge.client.lifetime_ms`, `forge.client.clock_margin_ms`), by
  which each write sent before has its answer and its proving read, or
  their timeouts. Past it, the forge names each write still without its
  outcome, by its entry and its task, and the stop goes on unsettled
  (`Request::Unsettled`);
  the service prints each as it ends (`../protocol/binaries.md`
  ), and the next start settles it as its class says (forge/effects.md,
  section 2). The bound is `skein_forge_client::stop_bound`, 602 s at
  the stock limits; `temper-engine versions` prints it, and a unit takes
  its stop timeout from it (`../protocol/configuration/derived.md`,
  5.6).
- **Then it says it restarts** (jig's `domain/engine/restart.md`, 1,
  *Saying it restarts*; `../draft/redeploys.md`, 5.2). Settled or past
  its bound, the root tells the core it stops; the core keeps the word
  in its stop's decision and, once durable, asks `Held::Restarting {
  wait }`. The root hands it to its service as `Request::Restarting {
  wait }`, and the service has its workers' component tell each worker
  (`host::Workers::restarting`, jig's `protocol/channel/restart.md`, 3),
  logging each worker told or not (`../protocol/wires.md`, section 4).
  The workers' links stay up until `Drained` closes them, after it, so
  the word drains before each close.
- **A second signal** aborts the wait, as it aborts the drain.

Then the drain. What the root adds to the core's:

- **the forge's events** go back to their owner undecided
  (`Request::Busy`), and no change steps;
- **a relayed call** is answered busy;
- **a committed forge entry** is not made: its notification is retired,
  its outbox row stays durable, and the next start's settlement makes it;
- **`Drained`** once nothing waits for a commit.

### 1.3 Each service restarted alone

An engine, a worker or a forge may each stop, fail, or be restarted on
its own, in any order, and whatever needed it carries on once it is
back:

| Restarted or away | What carries on |
|---|---|
| the engine | each worker keeps hosting its runs within its engine-away bound, or the restart wait its engine's settled stop told it (`worker/README.md`, section 2), redials, and its hello waits for the stored settings; the restart adopts each run its store holds as claimed, by its attempt, and the run goes on (`worker/README.md`, section 2; jig's `domain/engine/restart.md`, step 3 and `root.md`, section 7). Past the bound a run is interrupted, never failed, and resumes. The forge reads afresh and settles its outbox, each write in flight found or sent again as its class says, a goal's own branch made again (`forge/effects.md`, section 1, and `forge/branches.md`, section 1, and `forge/effects.md`, section 2) |
| a worker | its runs, stopped or lost, are resumed on it or another worker from their transcripts and saved work, their start messages kept (`worker/README.md`, section 2) |
| a forge | its reads and writes wait out the outage and its CI is read again (`forge/keeping-up.md`, 5); an owned Forgejo on a box gets its sign-in source once the engine answers, without its own restart (`../protocol/setup/box-supervision.md`, 1) |

The root itself adds no step: steps 4 and 5 of 1.1 are where the engine's
side happens, and the rest is the worker's, the forge's and the box's.

## 2. Open questions

- **A public read returning 503 after a project is made** (`routing.md`, 7, `root-restart.md`, 1.1):
  the `core-after-project` request reports it alongside clients whose
  links never come up, across a restart. A client rejecting `me` for too
  many memberships explains those links, but not the service refusing
  that public read. Default: no further core repair without evidence of
  a shut gate; the service's reason line (`../protocol/service-process.md`, 1.2)
  must identify what holds it before attributing another occurrence to
  the calendar, startup or the journal.
