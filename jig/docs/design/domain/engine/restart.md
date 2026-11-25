# The engine's restart

A restart is a cold start that rebuilds everything a decision reads from
the store and the connectors, and makes nothing twice. The core runs it
as a script: it asks the root for one step at a time, and asks for the
next only once the root says this one is done (../root.md, section 8).

## 1. The restart script

1. **The core's records:** the deployment's numbers; the settings,
   checked whole (`settings-store.md`, 1), before every other record; projects, their
   policies and parties' roles; parties with a live sign-in or a role,
   and their requests' keys within the retention; parties' read
   positions over live conversations, never ended ones' (`views.md`, 4); live
   tasks with their inboxes, subscriptions, holds and queues, and their
   dependencies with when and by whom each was added, the waits among
   them judged whole, so that a queue's waiter left on a circle of waits
   is held (../tasks/holds.md, 7); pending proposals; the calls committed since each live task's last turn;
   each live agent task's conversation; the funders' numbers; LLM
   accounts. The core's limits bound what is live (../tasks/README.md,
   section 6), so what is loaded fits; a live set past limits lowered
   since refuses the start, saying which. A live task names its
   revision, which is held (`settings.md`, 3).
2. **Each connector's records,** connector by connector: it is given
   the application's defaults of its kinds and says which differ from
   those it stored (`settings-application.md`, 1); then the root loads that connector's live
   ranges and restores them into it (../connectors.md, section 14). Once
   every connector has, new defaults, jig's and the connectors', make
   the release's revision in one decision, with the connectors' new
   defaults saved in it, checked against the limits with the live
   tasks' revisions held (`settings-store.md`, 1).
3. **The runs adopted.** Every task claimed or running names its attempt
   and the kind of host it was placed on. Claims placed in the engine
   died with it, and are settled at once as lost, or as not started if
   they committed nothing (../tasks/lifecycle.md, 2); so are claims on workers in
   the engine's own process, which no worker outlives
   (`../../protocol/service.md`, section 10), when its configuration says
   its workers are in process. A stop that reached its first decision
   settled its agent claims of both kinds as interrupted already (1),
   so those left are a crash's, or a job's. Claims on other workers reach the
   fleet, which adopts each attempt a worker reports as it dials in,
   keeps waiting for one within its grace, or its hold after a stop
   that said it restarts (1, *Held across starts*), and cancels any it
   does not name (`fleet.md`). A claim whose stored run proof is
   `crossed` is never adopted (`executors-transcripts.md`, 1): one in the engine or
   on in-process workers is settled at once, as above; one on another worker ends lost as a
   claim lost with its worker does, its run stopped when its worker
   lists it, and the claim lost once that run has stopped or once the
   engine's grace has passed with none listing it, so that no next
   attempt starts while it may still write (../hosts/hub.md, 6; ../tasks/holds.md,
   3). The fleet holds it in a state of its own (`Event::Cross`, the
   attempt `Fenced`), and one no worker listed that committed nothing
   ends not started, as an engine claim does (../tasks/lifecycle.md, 2). Nothing new is assigned before every claim loaded has
   reached the fleet.
4. **Open.** Once the runs are adopted, decisions open: the engine's
   own records are in, and nothing it serves waits for a connector.
   People sign in and out, chats, settings, task and census reads,
   worker links and adopted runs go on; every agent task due gets a
   run; every inbox's messages are relayed or wake as their policy
   says.
5. **Each connector catches up** from the opening, while the engine
   serves (../connectors.md, 14), its two halves together:
   - **it reads afresh,** for the live tasks that name them, the
     resources its system owns;
   - **it settles its outbox,** each entry not committed as made
     resolved as its recovery class says (../connectors-effects.md, 1.3): looked
     for by its key or the state it leads to, written again if it is a
     set; one not found waits for the retry deadline committed with its
     last attempt, which the restart keeps; one of a kind that cannot be
     recovered holds its task for a person.

   It reports each half, and has caught up once both have ended.
   Nothing bounds it, and nothing of the engine's waits on it but what
   step 6 names.
6. **Decide afresh, a resource at a time.** Every procedure steps from
   the opening as its wakes come; a decision on a fact a connector owns
   waits for that fact's first fresh read, or for the connector's fresh
   reads (../connectors-facts.md, 1.1), and never judges a resource from what
   the store held alone; a write waits for the uncertain entries before
   it on its lane (../connectors-effects.md, 1.2). A run's call that needs a
   connector waits in that call, within the run's own deadlines.

Workers keep the answers and turns they were not acknowledged for, and
send them again after their hello; the core commits any it does not
have, and acknowledges each.

## 2. After an upgrade

**After an upgrade.** At or above the floor and at the build's owner
levels, jig's store registers the build's layouts and runs declared
jobs in a new generation before the root starts (`../../protocol/store/opening.md`,
2). Values are converted at read from the registry's texts
(`../../protocol/store/layouts.md`, 4). A store without the floor,
below it or behind the levels is refused, naming `FLOOR_BRIDGE`; a
non-empty store without a registry is refused too (`../../protocol/store/opening.md`,
1.8). The restart restores from the upgraded copy before it becomes the
generation `active` names: the store's open answers that a generation
is pending, and the script asks, after the connectors' records (step
2) and before the runs adopted (step 3), for one step more,
`Phase::Switch`, which the root serves by asking the store to
switch. Every engine's start takes that step, pending or not (a worker
alone keeps no store): once the store is
active the root commits the start's wires (`../../protocol/store/records.md`, 3),
the start's mark, and only once it is durable are secrets served, runs
adopted and connectors let reach their systems. Before it the restores may commit, as they do at any start
(people's expired rows erased, the accounts' wants saved, the release's
settings revision, `settings-store.md`, 1), and their commits land in the copy alone; but
nothing leaves the engine's process: no secret is served, no run
adopted or placed, no connector reads or writes its system
(`../../protocol/store/opening.md`, 2, step 7). So the restores load and link
before the switch, and what acts on a record comes after it: the tasks
child's links of its live tasks (stubs, writers, waits and funders),
which adopting the runs needs, are made at `Phase::Link`, the link
pass, before the sweep and the switch, and step 3 adopts. The
mark is a root decision carrying the store's `Write::Wires`, and the
store opens its secrets' gate on it. The fleet's checks of a host's
programs, which read what a worker reports and start nothing, may go
before it. Over a generation with nothing pending, the step switches
nothing and commits the wires alone.

## 3. Load, then link

**Load, then link** (the user, 10-27; temper's `draft/upgrades.md`,
6.4). A start, over the active generation or an upgraded copy alike,
judges no record. It loads every record of the live ranges that decodes
under its codec within its bounds, and the live sets must fit the
limits: a set past limits lowered since refuses the start, saying
which, the one refusal beside a store that does not decode. Then it
links: each child attaches each record to what it names, once every
family whose facts the link consumes is loaded: a task to its project, parties and parent, an account's source
to its account, a read position to its conversation, a run proof or a
projection to its task, an application's records to theirs (its
root's, by the same rule). Nothing is checked again: no family's rules,
no record's shape beyond its codec, and never who may have written it,
which authority judged at its commit (../authority.md). Writers are
trusted, so every link is made. What a start does with what it finds,
one rule for every child and the store:
- **It refuses,** naming it, only what this binary and its
  configuration cannot hold: a record that does not decode under its
  codec (its key and its value disagreeing among them, a part past its
  bound); a live set past the limits; a saved value past its byte
  budget or that cannot be accounted (`ProjectPolicy`, below); the
  current settings failing their check whole against the document
  they are read with (`settings-store.md`, 1); a retained LLM choice the configuration's
  executors no longer admit (`../../client/domain/contract/README.md`, section
  7, item 7). Each is the store against the configuration, not a
  record judged.
- **It rebuilds** derived state, never refusing: the waits judged
  whole and a waiter on a circle held (../tasks/holds.md, 7), people's expired
  rows erased, the release's settings revision, and projection plans. After
  assembling every projection, a plan row at a waiting, active or held phase
  (except a held closing phase) whose task has no live row takes its own
  projection header's phase when that projection is retained. Without one,
  the row leaves the plan and both its stored TaskHot and TaskWords are
  erased. A changed plan marks its goal changed, so connectors receive the
  repaired whole plan. These writes commit in Link, before the sweep and
  switch, through the start's ordinary commits; the next start finds the
  repaired rows. This is derived feed state (`../connectors.md`, 8), repaired
  at every start rather than by a job pending once per store
  (`../../protocol/store/jobs.md`, 2).
- **An expired row,** one whose time has passed at the start's time
  (people's expiries among them), is decoded, never counted against the live
  set's limits, never linked and never held past its page, and the sweep
  erases it; its links are the tests' to check (`world.md`, section 1), so a store
  that fits once cleaned is never refused for what it held before.
- **The start's time** is the wall time observed once, when the start
  begins; both walks judge expiry against it and nothing else in the
  start reads the clock for expiry, so a clock set back during the start
  changes nothing. A row whose time passes after it is live, loaded and
  linked, and expires through the ordinary timers once the engine serves.
- **It refuses a link it cannot make:** a record naming one the start
  did not load (a task's party, project, placed hold or task, a claim's
  proof), a task or a claim naming a revision not held or a charter
  that revision lacks (`settings-store.md`, 1), a name reserved by another source
  (../accounts-sources.md, 1), an ended read position whose task is missing
  or whose task's record is not that party's conversation (the census's
  pair, `../../protocol/store/loads.md`, 2), a projection header or part
  naming what it lacks after the derived plan repair, a connector's row
  naming what it lacks, a person
  whose identity another person's row holds (the second row naming
  the first).
- **It does not check** a family's rules, a record's shape beyond its
  codec, who wrote it, or a job's counts, which the manifest keeps
  and the tests compare (`../../protocol/store/opening.md`, 2).
- **A link that cannot be made** refuses the start as `Failure::Link`,
  naming the record's key and what it links to (`Linked`: a record's
  key, a charter at a revision, a project, a connector, a settings
  proposal numbered past what its settings key holds, the hold owning a
  writer's resource), the first one found, and the engine stops with
  that reason, never a panic (the user, request `store-crash-safe` of
  11-09, which replaces the assertion of 10-28). Each child's link
  check answers the same way, through its own restore refusal, and the
  core names it in `Failure::Link`. A refused start leaves the active
  store unchanged. Derived projection plans are repaired as above;
  leaving any other unlinked record out so the start can open remains an
  open question (`README.md`, section 10). Every link is made before any sweep runs
  (`../../protocol/store/secrets.md`), so the refusal erases no value,
  and a release that repairs the writer, or the store, starts it.
- **The checks are tests** (`world.md`, section 1): every world story ends with a
  cold restart that restores and reads back what it wrote, and a record
  left out fails a test build; registry changes and declared jobs are
  proved over stores at the floor
  (`../../protocol/store/opening.md`, 2, and `../../protocol/store/testing.md`, 1).

A restart refused before the switch leaves the source generation
active and unchanged (`../../protocol/store/opening.md`, 2, step 7);
one refused after it, at a later step, leaves the upgraded generation
active and the one before it as `active` − 1, for the operator to go
back to before its first commit of its own (temper's
`draft/upgrades.md`, 3.2). The restart restores a stored closed window
as closed and adopts no claim whose proof is crossed (step 3); the
next attempt starts fresh (`executors-transcripts.md`, 1). Registration and the
jobs' runner do not infer those marks or close windows.

## 4. The start's interface

**The start's interface** (`jig_domain_core::start`). The root drives
the start; the core owns its order. `Core::start_begin(pending, at)`
takes the store's pending flag and the start's time; the core then
answers `Action::Step(phase)` for each `Phase` in turn: `LoadCore`,
`LoadInbox`, `RestoreConnector { connector }` for each connector,
`Link`, `Sweep`, `Switch`, `Wires`, `AdoptRuns`, and `Open`. As `Open`
is done the core asks each connector to catch up, `Ask::CatchUp` (step
5), and the root passes each connector's report of a half back with
`catch_up_done(connector, stage)`, `stage` its
`effects::connector::RestartStage`, `ReadAfresh` or `Settled`;
`caught_up(connector)` says whether both have come since the opening.
No phase follows `Open`, so nothing the start gates waits for a
connector. The root serves the
phase and reports it with `start_done(env, phase, out)`, and gives the
sweep its people rows with `start_sweep(env, SweepInput, out)`; it never
picks a successor. `Action::Commit(continuation)` asks the root to put
the writes the core pushed and the `Continuation` in one decision, and
to call `start_resume` once that decision is released; a cold restart
drops a held continuation with the rest of the volatile state, and a
stale one is an assertion. `Wires` always answers a commit, so nothing
leaves the process before the start's mark is durable. A row whose owner
is missing reaches the core as `start_missing(record, target)`, which
the link refuses as `Failure::Link` naming both keys, as any link it
cannot make (above). `Action::Refused(failure)` stops
the start, `start_phase()` naming the phase: a `start::Failure` is the
store against the configuration, never a record judged: `Settings`,
`ProjectPolicy { project }` (a saved policy past `limits.policy_bytes`
or that cannot be accounted), `HeldPastLimit`,
`NeedsPastLimit { held, limit }` (the stored settings' agent needs, in
owned bytes, past the fleet's needs bound, `fleet.md`), `AccountsPastLimit`,
`LiveSet { set, key, held, limit }` (a live set past its limit, `set` a
`LiveSet`), `Accounting` and `LlmChoice`, each reached by a test from a
stored record. `Action::Idle` follows `Open`, and `start_opened()` says
decisions are open. The store's side is `Store::start_restore()` before
its open, then typed pages, `Event::Page` and `Event::PageRefused`
(`../../protocol/store/loads.md`, 1); a refused page, like an open refused or a
connector's own fit, is the application's cause and is never turned
into a `start::Failure`.

## 5. Stopping

The engine stops when its service is told to (a signal, an operator), and
the core hears it from its root before anything closes
(`../../protocol/service.md`, section 5). From then on it takes nothing that
would make a commit:

- **a party's request** is answered not ready, and an application's
  request likewise, and so are a sign-in, a credential to mint and a
  sign-out, which commit too (`../../protocol/client/sign-in.md`);
- **a run's turn or call** is answered busy, and its answer left
  unacknowledged, so its host keeps them and sends them again, to this
  engine or the next;
- **a run in the engine's own process** cannot send them to the next
  engine, since it ends with this one (`../../protocol/service.md`, section
  10): one in its own slots, or on a worker in its process when its
  configuration says its workers are, as `restart.md`, step 3 counts them.
  So on hearing it stops, before anything else, the core settles each such
  agent task's claim as **interrupted**, in one decision: its attempt
  ends with no try spent and no backoff, keeping the turns and calls it
  committed, and the task is due again when an engine next starts
  (../tasks/lifecycle.md, 2). The fleet forgets the attempt, sending its host
  nothing, and whatever the run sends after is for an attempt the core no
  longer holds, and dropped. Its writer slots are released: the run may
  still write until its process ends, but nothing is placed while the
  engine stops, and the next start places no run that writes a
  resource before its connector has read it afresh (`restart.md`, steps 5
  and 6; `../tasks/holds.md`, 3). A job's claim is left as it stands, for the
  next start to settle as a crash leaves it, since its one effect may
  have happened. A crash, or a second signal before that decision
  commits, leaves every such claim to the next start, which settles it
  lost or not started (`restart.md`, step 3), so a run that brings its
  engine down still spends its tries;
- **a connector's events and outcomes** stay with the connector, as when
  the journal is full;
- **no run is placed, and no procedure stepped.**

It ends every watch saying the engine is stopping
(`../../client/domain/contract/watches.md`, 1.2), and says once when nothing it holds
waits for a commit, so the service can close its components. A restart
then starts from the store as `restart.md` says.

**Settling before the stop.** A root that settles its effects before
it stops (temper's `domain/root-restart.md`, 1.2; temper's `draft/archive/redeploys.md`,
5.2) first tells the core so, `Event::Settling`, at once on hearing the
stop, and its stop, `Event::Stopping`, once settled. From `Settling` on,
the core makes no new work and takes every commit:

- **no run and no job is placed:** the fleet's resume places no
  waiting attempt, whatever slot frees or hello is heard, while it
  releases held turns, reads and completions as ever; an attempt it
  had placed in a decision before `Settling` is still assigned once
  that decision is released, since its claim is durable and the next
  start adopts it;
- **no task is activated and no procedure stepped:** a due task, or one
  whose backoff or wake passes meanwhile, stays due, for the next start
  to decide afresh (`restart.md`, step 6); a connector's event commits
  its facts and steps nothing;
- **every commit is taken:** workers' turns and answers are committed
  and acknowledged, connectors' outcomes commit, and a run's call is
  the root's to answer (temper's answers busy).

`Core::settling()` says whether it holds. `Settling` twice is once;
`Stopping` after it goes on as above, and `Stopping` alone, for a root
that does not settle, is as before. A crash while settling leaves the
store as any crash does.

**A stop during the start** (temper's `draft/reconciling-start.md`, 4).
A stop may come in any phase of the start's script (`restart.md`). Told
`Stopping` while the script runs, the core asks no further phase
(`start_done`, `start_sweep` and `start_resume` answer `Action::Idle`)
and commits no mark; a continuation it holds is dropped once its
decision's commit is answered. Then:

- **before the mark** (the `Wires` commit durable), it has adopted no
  run and placed none, so it settles no claim; a root tells it
  `Stopping` alone there, since nothing has left the process to settle,
  so it keeps and says no restart word;
- **after the mark,** a root that settles tells it `Settling` first,
  once the phase it serves has ended, and the stop goes on as at any
  settled stop, the word kept and said for the runs step 3 adopted;
- **either way** it ends every watch saying the engine is stopping (none
  is open before decisions open) and asks `Held::Drained` once, when
  nothing it holds waits for a commit, the start's own commits in flight
  answered first, as at any stop. The root's service then closes its
  components, those that wait for the mark closing at once as closed
  (`../../protocol/service.md`, 5, step 4).

A stopped start is no refused start: it names no `start::Failure`,
`start_phase()` still names where it stopped, and the next start begins
again from the store.

**Saying it restarts** (temper's `draft/archive/redeploys.md`, 5.2). A settled
stop tells the workers that the engine restarts, so that each keeps its
runs through the restart wait rather than its grace
(`../../protocol/channel/restart.md`):

- **Kept first.** At `Stopping` after `Settling`, the core keeps the
  word in the decision its stop makes, with the claims it settles as
  interrupted or alone: the longer of the configured restart wait
  (`fleet.restart_wait`, `fleet.md`, *A restart said holds its runs*)
  and a word still kept from an earlier stop (*Held across starts*,
  below), so that a later word never shortens a hold outstanding.
- **Then said.** Once that decision is durable, the core asks its root
  to tell the workers, `Held::Restarting { wait }`, `wait` the
  configured restart wait, a `Duration`. The root has its workers'
  component send it on each channel whose worker's layout has it
  (`../../protocol/channel/layouts.md`, 4.1), and closes
  the workers' part only at `Held::Drained`, which the journal releases
  after it (temper's `domain/root-restart.md`, 1.2).
- **Kept as** the deployment's numbers record's `restart_wait_ms`, the
  wait in milliseconds, `option u64 = none` in `jig-store-core`: a
  registered family's field with its default, so no version moves and
  no patch comes (`../../protocol/store/layouts.md`). It is saved with
  the decision's header, as the numbers are.
- **Only a settled stop says it,** and only to workers that may outlive
  the engine: `Stopping` alone keeps and says nothing, as a crash says
  nothing; nor does a stop whose configuration says its workers are in
  its process (step 3), or whose restart wait is zero.

**Held across starts** (temper's `draft/archive/redeploys.md`, 5.3):

- **Read with the core's records** (step 1): a word kept is the start's
  hold.
- **Each claim on a worker the start adopts or crosses** (step 3) is
  held for the word's wait and the fleet's grace after it, counted from
  its adoption, rather than for the grace alone, and keeps that hold
  through its worker's hello and any loss after it, until its worker
  has heard the engine again (`fleet.md`, *A restart said holds its
  runs*). So no next attempt is placed while a worker that heard the
  word may still host the last.
- **Discharged** once no claim the start held is held any more: each has
  ended, been lost at its hold's end and so placed again, or had its
  worker hear the engine. The fleet says so once, and the core clears
  the kept word in a decision. A start that holds none clears it once
  the runs are adopted. A discharge heard after this stop's word is kept
  clears nothing.
- **A crash or a stop before then** leaves the word kept, so the next
  start holds every claim it restores on a worker again, for the
  longest wait said, from its own start: two restarts in a row, a crash
  between them or a shorter wait on the second, shorten no hold.
- **A build that doesn't know the word** holds for its grace alone, so
  going back past the build that brings it is every part together, each
  worker drained first (temper's `draft/archive/redeploys.md`, 7.3).

Finishing continues while stopping. After routing previously accepted work,
a Resume at which the core owes finishing closes without taking a new input,
placing a run, stepping a procedure or resuming a connector. Follow-up watch
endings and the one drain notification remain held behind the last commit.
The continuation makes no new commit (../root.md, section 3).
