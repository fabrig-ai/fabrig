# Testing the domain: fakes and kits

## 1. Fakes and kits

What jig ships for tests, as ordinary Rust or as step machines where
more than one world needs them (skein/docs/foundation/testing-strategy.md, section 4):

- **the store:** jig's store component over skein-kv's in-memory mode,
  with an in-memory stand-in for transcripts' files, the world holding
  commits, failing one, and serving pages slowly
  (`../protocol/store/testing.md`, section 1);
- **scripted workers:** each a host that speaks the contract of
  hosts/README.md, section 2, playing a run's script, keeping turns and answers
  until acknowledged, refusing messages and leaving them unread, losing
  its channel and coming back, or vanishing;
- **`smith-inline-agent`'s runs:** smith's real domain over skein's fake
  LLM, with a script per charter, for the hub's world and for an
  application's worlds that run smith's own domain;
- **scripted parties:** sign-ins, operations alone and in compound
  requests, reads and watches, by script or at random within their role;
  a template filled into a `Create` as a client fills it from `me`;
- **the test connector:** resources named by paths, kinds of hold
  (exclusive and pooled, refusing and waiting, pools that shrink),
  priced effects of every form and every recovery class, with keys,
  conditions, late copies and uncertainty, requirements guarded and
  observed, judged on its facts, procedures that step, wait, delegate,
  amend, cancel with a grace, message and finish, topics with classified
  news, some offered to agents, drift, kinds of settings of its own,
  judged on its facts, accepting or refusing as the seed draws, and a
  fake system beneath it that reports what it saw. The core's world runs two of them, so that
  judges, resources and kinds cross connectors, with names and kinds
  generated from the seed;
- **the referee and the harness** of the conformance world;
- **what several worlds drive,** each a kit under `jig/testing`: the test
  connector's paths and kinds (`jig-test-connector-kit`), the core's driver,
  peers and referee (`jig-core-kit`), the client's harness, fronts and shared
  stories (`jig-client-kit`), the engine clients' peer
  (`jig-engine-clients-kit`), the protocol client (`jig-client-protocol-kit`),
  the accounts' fixtures (`jig-protocol-accounts-kit`), the channel's
  (`jig-channel-kit`), the people's referee (`jig-people-kit`) and the tasks'
  world and referees (`jig-tasks-kit`);
- **what one package's test binaries share,** a kit too, compiled once for
  them all (`tools/crates/design.md`, 3.2): the reference binary's stories'
  rig, its binary, native person and provider on skein's real ring
  (`jig-test-kit`);
- **jig's inspection adapter,** `jig/testing/jig-inspect`, of skein's form
  (`skein/docs/design/testing/inspection.md`, 2;
  `temper/docs/design/draft/failure-worlds.md`, 3.1 and 3.2), over its
  crates' `pub` reads, each private table or alarm it needs gaining a
  `pub` read documented as its adapter's:
  - `jig_inspect::tasks(tasks: &jig_domain_tasks::Domain, into: &mut
    Inspection)` gives each live task's waits and the tasks child's
    tables;
  - `jig_inspect::core(core: &jig_domain_core::Core, into: &mut
    Inspection)` calls `tasks` over the core's and adds the core's parked
    batches and the people's kept answers;
  - `jig_inspect::engine_clients(clients:
    &jig_protocol_engine_clients::Component, into: &mut Inspection)` gives
    the clients' component's waits and a sign-in's pending exchanges,
    which that component holds, not the core;
  - what each reports is 1.1, where the worlds call them 1.2; an
    application's worlds call `tasks` over the arena their root holds, or
    `core` over the core, beside their own adapter.
- **the store check** (`temper/docs/design/draft/failure-worlds.md`,
  3.7): a start over a committed snapshot, apart from the live world, so
  that a record a start would refuse fails the world, naming the record
  and the checked commit. Under `every()` it fails at the commit that
  made the record:
  - **What it runs:** the world's store copied at a commit (`Durable::fork`,
    `jig_test_store_files::Store::fork`: the durable image, cold-opened),
    a root of its own built from the world's configuration, and a start
    over the copy through every phase before the switch: the core's
    records, each connector's, the link pass and what the start derives
    after it, the sweep among them (`engine/restart.md`). It stops where
    the core asks for `Phase::Switch`, so nothing a start does after its
    mark runs: no wires, no run adopted, no connector reading or writing
    its system. The start's commits land in the copy alone, and no peer,
    fake or referee of the live world sees it.
  - **Its verdict:** the root reaching the switch passes. A start refused
    (a `start::Failure`, a page refused, a connector's restore refused)
    is a finding, `store check: commit <n>: start refused at <phase>:
    <refusal>`, the refusal in its `Debug` form, naming the record's key
    wherever it names one (`Failure::Link`'s `record`, `LiveSet`'s `key`,
    a page refusal's row); a start that stops short of the switch with
    nothing refused is one too, `store check: commit <n>: start stalled
    at <phase>`. A connector's restore refusal in jig's test root is a
    start refusal (`Failure::Application`), never a panic.
  - **Which commits,** `jig_conformance::store_check`:

    ```rust
    pub struct StoreChecks { /* policy, commits, checked, kept */ }
    impl StoreChecks {
        /// Every commit: the worlds whose budgets carry it.
        pub fn every() -> StoreChecks;
        /// One of the run's commits, drawn from its seed, checked as the
        /// run ends: the worlds' default.
        pub fn one(seed: u64) -> StoreChecks;
        /// Each commit with one chance in `one_in`, drawn from the run's seed.
        pub fn sampled(seed: u64, one_in: u64) -> StoreChecks;
        /// None, for a story whose own input is a record a start refuses.
        pub fn none(reason: &'static str) -> StoreChecks;
        /// Counts commit `number`; whether the world checks it now.
        pub fn due(&mut self, number: u64) -> bool;
        /// After `due`: whether the world keeps that commit's store, in
        /// place of any it kept, for the run's check as it ends.
        pub fn keeps(&self) -> bool;
        /// As the run ends: the kept commit, now counted checked, which the
        /// world rehearses over the store it kept; `None` when it kept
        /// none, under any other policy, and at every later call.
        pub fn end(&mut self) -> Option<u64>;
        /// Counts commit `number` checked by a restart the run made from it anyway.
        pub fn restarted(&mut self, number: u64);
        pub fn checked(&self) -> u64;
        pub fn commits(&self) -> u64;
        /// `store check: 3 of 41 commits checked (one in 16, seed 7)`;
        /// `store check: 1 of 22 commits checked (one per run, seed 45, commit 17)`.
        pub fn report(&self) -> String;
    }
    pub fn finding(commit: u64, phase: &str, refusal: &str) -> String;
    ```

    Under `one`, the `k`th commit `due` counts is kept when
    `Rng::new(draw ^ k).below(k)` is 0, so the first is always kept and
    each of a run's `n` counted commits ends kept with one chance in
    `n`; `due` answers false, and `restarted` counts no candidate. Its
    report names `one per run, seed <s>`, and `, commit <k>` once `end`
    has answered `k`. A sampled commit `n` is due when
    `Rng::new(draw ^ n).below(one_in)` is 0. `draw` is the seed's
    `store check` part of `Stream::Scheduling`
    (`skein/docs/design/testing/sweeps.md`, 2): a part of its own, so
    its draws move none of the world's, and a seed replays its sample.
    A world writes its report to stderr as its run ends, after its kept
    check. A run is a world's story from its construction, or its last
    `store_checks` setter, which starts the count afresh and drops what
    was kept, to its end; a commit-cut child keeps nothing, its restart
    its check.
  - **The core's world:** `jig_core_kit::World::check_store(&self) ->
    Result<(), String>`, over jig's test root rehearsing, called once
    each commit is durable as the world's `StoreChecks` say
    (`World::store_checks`), one per run from the world's seed by
    default: the kept commit's fork is rehearsed in `World::finish`,
    after the closing oracle, and from `Drop` where no `finish` ran.
  - **The conformance harness:** `Harness::check_store(&self) ->
    Result<(), Violation>`, over a cold snapshot (`cold_snapshot`), with
    four methods each application gives: `rehearse(domain: &mut
    Self::Domain)`, before its start begins, which makes the start stop
    before the switch; `rehearsed(domain: &Self::Domain) -> bool`, it
    stopped there; `refused(domain: &Self::Domain) -> Option<String>`,
    its start's refusal in its `Debug` form; and `start_phase(domain:
    &Self::Domain) -> String`, the phase its start is in, which names a
    refusal's phase and a stalled start's, so that no finding parses a
    phase out of a refusal's text. Every application of the harness
    gives them from its own root: jig's test root, the example
    application's root (`../examples.md`, 4) and temper's. The per-commit
    check clones the application's configuration, so that associated
    type is `Clone`, and builds its peers and systems afresh
    (`Application::peers` and `Application::systems`, from the
    configuration and the harness's seed), never a copy of the live
    ones: the cold readback (`Application::cold`) gives the fresh peers
    what the store holds, its start stops before the switch, so no
    connector reaches its system, and a fake's pending calls and a peer's
    held outputs hold reply capabilities no copy may duplicate. Its
    verdict is the start's alone: it runs no outside inspection of the
    referee's, whose systems it would not match.
    `Harness::store_checks(&mut self, StoreChecks)` sets the commits,
    one per run from the harness's seed by default, and `Outcome` gains
    `checked: u64`. The kept commit's store (`Durable::fork`, with the
    configuration and clock at that commit) is rehearsed at the story's
    end, `Harness::finish(&mut self) -> Result<(), Violation>`, which the
    story and scenario runners call before they return their outcome,
    and from the harness's `Drop` where no `finish` ran.
  - **A restart the run makes anyway is its commit's check:** in a
    commit-cut sweep (`Cut::EachCommitFrom`, `fork_cuts`, the core
    world's `World::sweep`), each child's cold restart is a start over
    its commit's store, so a refusal there is the finding, read through
    `refused` (the core world's `start_failure()`), naming the commit it
    restarted from, and the parent counts each such commit `restarted`:
    those sweeps check every commit, at no second start's cost. The core
    world's sweep takes a commit's cut as `Cut::Durable(n)`, one per
    commit it sweeps, and its child resumes the story, whose first cold
    restart is the check: a refusal there is written to the child's
    stderr as the finding, naming `n`, and the child exits failing, so
    the parent fails naming the cut (`forked cut Durable(n) failed`),
    the refusal in its child's output beside it; no channel carries the
    finding's words back to the parent. A
    story's own restart (`World::restart`, `Harness::crash` called by a
    story) keeps its own reading of a refusal, since a story may be about
    one.
  - **A story about a refused start,** whose own input is a record a
    start refuses, reads the check's `Err` as its outcome, or sets
    `StoreChecks::none` with a reason naming the refusal it tests.
  - **A sampled world's finding** names the commit it checked, at or
    after the one that made the record, a kept commit's as the run ends;
    the same seed under `every()` names the commit that made it.
  - **Tests,** in the core's world and the conformance world: a committed
    record a start would refuse (a task naming a project the store
    doesn't hold, written by a fixture's commit as a writer's defect
    would) fails at its commit under `every()`, naming the record and the
    commit; the same story without it passes with every commit checked;
    a commit-cut sweep's child refused names its commit; a run under the
    default checks exactly one commit as it ends, reports it and replays
    it from its seed, and the defect's story at a seed keeping a commit
    at or after the defect's fails as it ends, naming the kept commit; a
    sampled run reports its count and replays it from its seed.
  - **Coverage:** `StoreChecks::rehearse` records one `store-check`
    when a rehearsal of a committed snapshot returns its verdict.
    For a commit-cut restart the parent records the check through
    `StoreChecks::fork_restarted` on the child's successful exit.
    These counts show when a change leaves fewer committed snapshots checked
    (`skein/docs/design/testing/checks.md`, 1; `tools/tiers/budgets.md`).

They stay in jig, and an application's worlds reuse them. What jig finds
missing in skein's world harness goes into skein first.

**Their checks return their findings,** as skein's do
(`skein/docs/design/testing/service-tiers.md`, 1;
`skein/docs/foundation/programming-model.md`, 10.4), and nothing in jig's
tests catches a panic:

- **Every world's referee and judge** implements `watch`, and keeps what
  its `act` or `idle` finds for its next `watch`.
- **The conformance observer** (`jig-conformance`'s `core::Observer`, and
  `jig-core-kit`'s over it) returns its findings rather than panicking:
  each method that observes or checks returns `Result<_, String>`, its
  `Err` the referee's error on an observation, with the time, a durable
  row changed or gone, or a reply ahead of its commit's fence, in the
  words it panics with now. Its callers pass it up, so no finding is
  dropped: the core's world, the conformance harness and the
  conformance world fail with it at their top, naming the seed and the
  trace's tail, and a test of a check reads the `Err`.
- **The views' referee** records what breaks in skein's domain judge
  (`Judge::check`, `Judge::fail`), never asserting; its last delivery
  following the subject's final change is a returning check too. Its
  world fails on the verdict (`check_holding`); its tests read it.
- **The client's model** (`jig-client-kit`) has a returning form of
  applying a page's patches, whose finding is a patch that breaks the
  model, a value patch that doesn't move its write count among them; the
  world fails with it.
- **The client's world** (`jig-client-kit`'s `World`) has a returning
  form of each entry point that observes: `try_client`,
  `try_client_with`, `try_event`, `try_send_raw` and `try_run`, each
  answering `Result<_, String>` beside its form that fails, which keeps
  its signature. The `Err` is the referee's first finding
  (`check_holding`, a missed deadline among them) with the seed and the
  trace's tail; the first stops the world, which observes nothing after
  it. The forms that fail call the returning ones and panic with the
  `Err`, and no returning form calls a form that fails. A test of a
  check, its heap envelope's among them, reads the `Err`.
- **The restart oracle** (`jig-core-kit`'s `World`):
  `World::finish(&mut self) -> Result<(), String>` runs the closing cold
  restart and readback once and returns the oracle's first finding, a
  durable document's field that differs, a row left out, a header or a
  transcript among them, each comparison returning rather than
  asserting. `close` and the world's `Drop`, which runs only when
  nothing is unwinding, panic with it at the top. A test of the oracle
  calls `finish` and reads the `Err`.
- **The store's files read back** (`jig-test-store-files`'s `Store`):
  `retained`, `readback`, `readback_of` and `transcript` answer
  `Result<_, ReadbackFinding>`, the finding a store that is not quiet
  (`NotQuiescent`), an application key no record names
  (`ApplicationKey { key }`), a value that does not decode
  (`ApplicationValue { key }`), or a transcript that does not read whole
  (`Transcript { task, backwards, .. }`), naming what it read. A
  readback that finds one changes nothing it reads, and its callers, the
  restart oracle and the crash's recovery check among them, pass it up.
- **The host's and the notes' worlds:** `jig-host-world`'s refusal
  referee (`Referee::request`) and its scripted turn engine
  (`receive_turn`: a replay that changes its turn, a turn past a gap),
  and `jig-notes-world`'s referee (`before_commit`, `after_commit`)
  answer `Result<(), String>`, returning before they record what they
  refuse; their worlds fail with it at their top.
- **No test triggers an assertion** (`skein/docs/foundation/programming-model.md`,
  10.4). A limit an assertion guards is tested at its bound, comparing
  what the build used with its price; a call out of a step's order, a
  stale continuation or a duplicate key is tested through the outcomes
  around it; a cause outside the process is a refusal, and its test
  reads the refusal.
- **A harness that names a seed, a story's choices, a scenario or a
  case** writes it to stderr before it runs it, and adds the trace's tail
  to a returned finding. A table of cases, the step crates' and the
  core's bundle (`tests/core/tests/support/bundle.rs`) alike, names each
  case, and the bundle its seed, before running it, and stops at the
  first that fails. The runner prints a failed test's output. What a
  harness writes before a case names the case, never its input: a
  seeded sweep writes the seed, the case (its kind, number or mutation)
  and the input's length, and the assertion that fails names the input
  in its message.

**A world's library is its own.** No package depends on a world; its own suite
reads its kit through its library, which re-exports it.
`jig/tests/boundary/src/tests/kits.rs` holds this. Two packages named as worlds
are kits by design: skein's world harness, `skein-world`
(`skein/docs/design/testing/layout.md`, 1), and smith's `smith-agent-process-world`.

### 1.1 What jig's adapter reports

Written from `temper/docs/design/draft/failure-worlds.md`, 3.1 and 3.2.
`tasks` and `core` fill one inspection, component `jig`, whose kinds are
`jig_inspect::KINDS`, the strings below; `engine_clients` fills its own,
component `jig engine clients`, kinds `jig_inspect::ENGINE_CLIENTS_KINDS`.
A waiter is `task N`, `batch N` for a batch the core has parked, or
`exchange N` for a clients' exchange, by its number, so that an
application's adapter names a task as jig's does.

**Waits:** each live task's, from its phase and its record
(`tasks/lifecycle.md`; `tasks/holds.md`), one for each reason it waits:

| Kind | Who is in it | Wakes | Bound |
|---|---|---|---|
| `on its dependencies` | waiting, or kept, with pending dependencies | each pending dependency, `Ended` once it has ended | none |
| `in a resource's queue` | waiting for holds, its dependencies done | each holder of a resource it needs that isn't free; its hold deadline | that deadline, `tasks.hold_wait` after its wait began |
| `on its delegates` | closing on its delegates, or idle with live delegates | each live delegate | none |
| `on its requester` | idle, its question to its requester unanswered | its requester | none |
| `on a watch` | idle, for each task watch and topic | the watched task, `Ended` once it has ended; a topic's connector | none |
| `on a timer` | idle, for each timer | the timer | the timer's deadline |
| `on its party` | idle: a conversation, a standing task, or one its requester ends | the person, or the party, it waits on | none |
| `on a person's answer` | a person task | the person or role it waits on; its stall deadline | none |
| `on its proposal's decision` | its proposal pending | the holder deciding it; its stall deadline | that deadline |
| `held: <hold>` | held: one kind for each `Hold`, in its design's words (`held: holds waited`) | the holder its escalation reaches; what lifts that hold by itself, as a circle's tasks or a connector's facts; its stall deadline | that deadline, where its design arms one |
| `closing on its run` | closing, its run ending | the run's host | none |
| `closing on its effects`, `closing on its releases` | closing on its connectors | each connector it waits on | none |
| `backing off` | active, between tries | its retry | the retry's deadline |
| `suspended` | due, its run suspended | the suspension's end | that end |
| `paced` | idle, with messages its wake policy holds back | its wake | the wake's deadline |
| `a batch awaiting its creation reports` | a batch the core parked (`tasks/holds.md`, 2) | each connector whose report it lacks; its deadline | that deadline, `tasks.hold_wait` after it parked |

- **Wakes:** a task is a waiter, `task N`; a person, role, host or
  connector a party (`person 3`, `role 1 of project 2`, `connector 2`); an
  armed deadline a timer; a task that has ended `Ended`.
- **A bound is the component's own armed deadline,** read from the alarm
  the tasks child or the core keeps for that wait. A wait its design
  bounds with no deadline armed is reported under its kind followed by
  `, with no deadline armed`, which no inspection declares, so that the
  shared check finds it (`skein/docs/design/testing/inspection.md`, 4.1).
  A task's grace, and the authority's deadline that will hold an idle
  task, are a wake and a bound of each of its waits.
- **A timer the core withholds bounds nothing.** From `Event::Settling`
  the core fires no `Timer::Tasks`, and from `Event::Stopping` none of
  the timers `vocabulary::waits` names: a backoff or a wake that passes
  stays due for the next start to decide afresh
  (`engine/restart.md`, *Settling before the stop*). `core` asks the core
  which timers it withholds, by the rule its routing fires them by, never
  a copy of it. While a timer is withheld, each wait and each queue's
  waiter that timer's alarm would bound has no bound and the added party
  wake `the next start`, and a kept answer past its expiry, once the
  people's timer is withheld, is kept, not due. `tasks` alone knows no
  core, so it reports every alarm as a bound.
- **A rejected escalation** leaves its task held (`tasks/proposals.md`,
  *An escalation's decider has its kind*): it is `held: <hold>`, its wake
  the decider the escalation records, `task N` or `person N`, as a
  waiting escalation's holder, beside the hold's own sources. It has no
  bound unless an alarm is armed for it. A `Routing` escalation is
  resolved in the same decision that makes it, so no quiet point sees one:
  `routing escalation` stays undeclared, and so does `unheld escalation`.
- **A held task in no project** has no one to escalate to: its hold
  "stays with its task until the application cancels it"
  (`authority.md`, 6.1), its escalation left `Routing` or `Unheld`. It
  is `held: <hold>`, its wake its requester, `deployment of project 0`,
  with no bound unless an alarm is armed for it. Only a task of project
  0 is read so; elsewhere both stay undeclared.
- **A proposal past its last holder** stays with that holder once its
  stall finds no one further who covers it, its stall spent
  (`tasks/README.md`, 8: "a pending proposal whose holders go" is an
  open question). It is `on its proposal's decision`, its wake that
  holder, with no bound, as at a policy's parties. `core` asks the core
  whether anyone is further, by the rule its stall routes by, through a
  read, never a copy of it; a proposal with someone further and no
  alarm armed is still `, with no deadline armed`. `tasks` alone, which
  knows no routing, reports every holder's missing alarm so.
- **The parties a policy names** are a party wake, `policy <kind> of
  project P`, in the component's words for the proposal's kind: the tasks
  child keeps the policy's project and kind, not its roles. A proposal or
  an escalation waiting there has no bound, since it waits for them
  however long (`tasks/proposals.md`: "At the parties the policy
  names") and its design arms no alarm.
- **A closing task's parties** are named by what knows them. The tasks
  child keeps a run's attempt, not its host, and not the connectors its
  effects and releases wait on, so `tasks` names them in the task's own
  words: `host of task N`, `connectors of task N`. `core` walks the same
  tasks with the core's placement and closing connectors at hand, and
  names each exactly, `host 4`, `connector 2`, one wake for each; where
  the core knows none, it keeps the task's words. An application's
  adapter that calls `tasks` alone takes the task's words.
- **Words are no wake.** A parked agent task that is no conversation,
  doesn't stand and isn't ended by its requester waits on what it asked,
  delegated, watches or proposed (`tasks/lifecycle.md`, 2): words may
  come, and nothing promises them. One idle with none of these is
  reported `idle with nothing left to await`, which no inspection
  declares: its design makes it due again.
- **Not waits:** a task whose executor is engaged or owed (due, preparing,
  claimed, running), an ended task, and a procedure's own step, which its
  application's adapter reports (`temper/docs/design/testing/checks.md`,
  6.1): of a procedure task jig reports only the waits above, so that
  jig's wake never stands in for its connector's.
- **The clients' component's waits:** each exchange `awaiting the
  engine's answer`, its wake the party `engine`; and each wait it arms a
  deadline for, in `../protocol/client/README.md`'s words, that deadline
  its bound.

**Tables,** as the draft's 3.2 names them, each entry keyed in its
table's words, its rule the table's:

| Table | Capacity | Entries and their rule | Release | Refused newcomers |
|---|---|---|---|---|
| `subscriptions`, one for each subscriber | the allowance admission refuses past: `tasks.subscriptions` with its reserved one (`subscriptions::kept_capacity`), which the tasks `Domain` keeps from its limits as a count no store or wire carries | each subscription: `the subscriber's end, or unsubscribing` | pending; a one-shot timer with no timer armed, due | none: refused `Busy` |
| `creation reports` | `tasks.resource_reports` | each report: `the batch's answer; pinned while a batch that hasn't answered needs it; evictable otherwise` | pending while a live task's holdings name it or a batch pins it, kept otherwise; pinned by a batch the core no longer holds, due (`core` alone) | none: refused `Busy` |
| `pools`, one for each pool | its slots, as last reported | each holder: `the holder's end` | pending; a holder that has ended, due | each task in its queue waiting on that pool alone, its bound its hold deadline |
| `write holds`, one for each held resource | 1 | its holder, as a pool's | as a pool's | as a pool's |
| `kept answers` | `people.requests` | each kept answer, a connector act's among them: `their retention's expiry`; each pending request's reservation: `its answer` | an answer kept, due once its expiry is at or before the core's clock; a reservation pending | none: refused `Full` |
| `a sign-in's pending exchanges`, one for each sign-in | `clients.exchanges_per_client` | each exchange: `each request's answer` | pending | none: refused busy |

- **Admission's progress** is checked where newcomers wait, the pools' and
  write holds' queues. A waiter takes its holds all at once
  (`tasks/holds.md`, 2), so it is refused only in a table it waits on
  alone, where room that frees must admit it. Its bound is its hold
  alarm's due; its `since` is `Time::ZERO`, the start unknown, because
  the tasks child keeps a wait's start as wall time alone and the
  checks read `since` only to describe a finding.
- **jig's kept answers** go by their retention alone: an unexpired one is
  never evicted (`people/acts.md`, 1); the draft's eviction is a
  connector's own.

### 1.2 Where the checks run

- **The core kit's `World`** (`jig_core_kit::world::World`) calls both
  shared checks (`skein/docs/design/testing/inspection.md`, 4) over
  `jig_inspect::core` at its driver's quiet point: where its loop returns
  settled, nothing left to step before time moves, with no restore held:
  a start the world holds at one of its steps, its root not yet ready,
  is a restore held, so its deadlines not yet rebuilt are no findings.
  A timer the core reports due at the world's time is something left to
  step: before it returns settled, the loop fires each due timer the core
  does not withhold (1.1), one at a time and draining after each, until
  none is due, in every drain and not only `advance` and `fire_due`. A
  withheld timer stays unfired. Each check's own tests judge a snapshot
  before the world fires it.
  Its moment is the world's seed, its time and a count of its iterations.
  A finding is the loop's `Err`: a drain fails with it and the trace's
  tail, and the restart oracle's settling returns it from `finish`. Every
  world that drives the core through that `World` runs them: the core
  world, and stories of the engine-clients, client, client-protocol and
  accounts worlds and of temper's client world.
- **The world's clock:** with no clock supplied, each drain iteration moves
  the clock by one implicit tick. A restart ticks through its load and switch
  phases, so slow pages still take simulated time, then holds the clock from
  the claim's adoption until the restart returns. `advance` holds the clock
  pair it reached throughout its draining; ordinary drains still tick. Thus
  a deadline the restart arms at adoption, and a boundary `advance` is given,
  are not moved past by the driver's own iterations. The core world's
  `joint_clock::restart_and_advance_keep_the_implicit_clock_at_their_boundary`
  pins this rule.
- **The engine-clients world:** `jig_engine_clients_kit::Peer` calls both
  over `jig_inspect::engine_clients` where its pump finds nothing left to
  do; its moment is the peer's seed, its time and a count of its passes.
  The peer fails with a finding at the entry point that pumped.
- **Outside the heap's count:** the inspection is made and dropped between
  the world's heap readings.
- **The bounded waits' deadlines** are the component's own alarms, checked
  at each quiet point. These worlds settle within each drain or fail, so a
  wait starved past its bound is found at the first quiet point after it.
- **Each check's own tests,** in the core world, withhold a deadline: a
  wait, and a queue's waiter, judged past their bound are findings, and
  pass once it has fired; a kept answer past its retention is a finding,
  and passes within it and once the people's timer has fired. Small
  stories pin the adapter's record of each kind and table.

### 1.3 The open problems

Each open problem of the draft's section 7 that a jig world shows has its
scenario, named as the draft names it, ignored with the problem and the
work that fixes it (`#[ignore = "P43, open: bounded waits
(jig/docs/design/domain/testing-fakes.md, 1.3)"]`), and an exclusion of
skein's form (`skein/docs/design/testing/sweeps.md`, 5) in the kit of the
world whose sweeps draw its fault: `jig_core_kit::open::EXCLUSIONS` and
`jig_engine_clients_kit::open::EXCLUSIONS`, each fault's kind beside them,
its family as the draft's section 4 names it. The sweeps that draw these
faults take them (`testing-workloads.md`, 7), and a step test holds that
a sweep configuring each fault accepts each.

| Problem | Scenario | World | Fixed by | Fault left out, and while |
|---|---|---|---|---|
| P43 | `a_coordinator_watching_its_asker_is_woken` | core | bounded waits | runs that park, resume and move: `a run parks with a question to its requester open`, while `its requester watches it` |
| P117 | `a_parked_act_holds_no_exchange` | engine-clients | bounded waits | people and parties: `a sign-in sends a request again while the first is pending`, while `the first is parked` |
| S15 | `a_party_extends_its_goals_deadline` | core | provider failures | people and parties: `amending a deadline after it has passed`, while `the amending party started the goal` |
| P77 | `an_owners_widening_is_never_proposed_to_itself` | core; its reading and decision, client-protocol | provider failures | people and parties: `a party widens a goal's authority`, while `the widening party is the project's owner` |
| P76 | `a_new_party_gets_account_uses` | core | provider failures | people and parties: `a new service party runs on an account`, while `the account's uses were set before the service was made` |
| P93 | `a_start_without_a_budget_leaves_room` | core | provider failures | the engine's own bounds: `a start with no budget`, while `its pool funds another request` |
| P98 | `a_full_period_is_named_as_funding` | core | provider failures | the engine's own bounds: `a new party asks for funds`, while `its period is fully allocated` |
| P82 residual | `a_small_delegating_task_funds_its_delegates` | core | provider failures | the engine's own bounds: `a delegating task's run at its minimum`, while `half its budget is at most the minimum run spend` |

- **The reason cites this section,** whose table leads to the draft's
  rows: jig's sources name no temper path or word, in an attribute, a
  literal or a comment (jig's vocabulary boundary,
  `jig/tests/boundary`), so neither a scenario nor its doc comment cites
  the draft.
- **What each asserts** is the draft's row. P76's runs in the core world,
  where the accounts and the runs' coverage are: the people world answers
  account changes with a fixed outcome.
- **P77's client part** reads a party's widening proposal and decides it
  through the client's wire (`Read::Proposal`, `Ask::Decide`). Where it
  passes today, it runs, as a regression of P77's repaired part, and the
  row's ignored scenario is the core world's.
- **The fixing work** removes the `#[ignore]` and the exclusion together
  (the draft, 6).
