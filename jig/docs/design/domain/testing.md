# Testing the domain

Provisional, 2026-10-07, revised 2026-10-30. How jig's domain is tested,
and how it tests an application's: the worlds of jig's own parts, the
core's world, and the conformance world every application runs its
domain on, with the referee that holds it to jig's promises. It applies
testing-strategy.md to jig's domain layer; the tiers below the domain
(protocol, simulated, the real loop) are `../protocol/service.md`'s,
section 8. What is still open is listed in section 10.

## 1. In one page

- **Domain worlds first,** as testing-strategy.md, 2.2 has them: every
  child domain of the core has its world, the core has its own, and so
  does the hub, none of them needing a protocol layer.
- **The conformance world is jig's gift to applications.** It runs an
  application's whole engine domain, its root over jig's core and its
  connectors, against jig's fakes for the store, the hosts and the
  parties and the application's fakes for its systems; it crashes the
  engine at every commit, holds and fails commits, loses hosts; and
  jig's referee checks, from outside, every promise of model.md,
  section 11.
- **Generic in the world, concrete in the domain.** Worlds are ordinary
  Rust (programming-model.md, 10.2), so the conformance world is generic
  over the application, through a trait of the test kit's. No step code
  is.
- **A test connector** stands in for an application in jig's own worlds:
  resources, holds and pools, priced effects, requirements, procedures,
  topics and faults, all drawn from a seed.
- **Faults everywhere, limits tiny,** replayed from a seed, on skein's
  world harness, within the suites' time budgets.

## 2. jig's tiers

| Tier | Real | Faked |
|---|---|---|
| step tests | one step function; authority's checks | its events, by hand |
| a child's world | one child domain of the core, or the hub | its parent, scripted; its neighbours |
| the core's world | the core with every child, under a root of jig's own | the test connector, scripted hosts, scripted parties, the store |
| the conformance world | an application's engine domain, root and all | jig's fakes, the application's fakes, the faults |
| an application's system worlds | its engine, its workers and its agents' domains together | its systems' fakes, at their domain faces |

Every world runs on skein's world harness: one loop, one seed, the
referee in the loop, the heap counted at every iteration
(testing-strategy.md, sections 6 and 7).

## 3. The children's worlds

- **tasks:** batches, lifecycle, holds and queues, inboxes, wakes,
  proposals; a task's label, flags and last activity, cancels with a
  grace across a restart, goals per project, the ended indexes saved as
  tasks end, what parties and agents ask alike, and each task's
  revision, its moves and its charter version's models (tasks/README.md,
  section 7).
- **authority:** step tests only, as a decision over data; one batch
  under a task and under a party answering alike, and `AsAllowed`
  resolved over generated pools, roles and caps (authority.md,
  section 11).
- **people:** sign-ins and credentials minted, roles, requests,
  inboxes; operations alone and in compound requests, made whole or not
  at all; read positions over conversations, live and ended, across a
  restart (people/README.md, section 10). Its request step tests and its world
  fill completed-answer capacity: a `Seen` progresses without saving an
  `Answer` on success or any routed refusal, while a new ordinary key is
  busy. Ordinary reservations exclude `Seen` flights exactly, and the
  pending, flight and waiter bounds apply. Duplicate waiters each get one
  terminal; pending conflicts, retained replay and conflicts, and
  reprocessing to a maximum watermark are checked in the step and world
  tiers (people/acts.md, 1, and people/README.md, 5).
- **fleet:** hosts of both kinds, placement, fencing, graces, adoption;
  host needs per charter version, a run of an older version placed on a
  host its version allows and the newest's does not (engine/settings.md, 3).
- **brief:** sections planned, gathered, cut by size, a required one
  missing; connectors' sections as sizes and tokens only.
- **views:** watches of jig's subjects marked by generated commits: a
  task query by the tasks it matches before a decision or after it, `me`
  and an inbox by their party's changes; changes coalesced, one delivery
  in flight per watch, and the watches' limits refused at the entrance
  (engine/views.md).
- **notes:** its own concern.
- **accounts:** its world checks source deadline creation,
  merge and restore with injected monotonic/wall clock pairs: later,
  earlier and equal resets, the old pair kept on a tie, a no-reset
  limit overriding a timed one to `None`, and indefinite limits kept
  until resumed. Restore preserves the exact saved wall; timers are
  checked just before and at the monotonic deadline (accounts.md, 3.5).
- **the hub,** with agent capabilities of both kinds, smith's: agent
  processes, `smith-host-domain` over smith's agent processes over skein's
  fake LLM;
  and, as the in-process kind, `smith-inline-agent`'s runs over skein's
  fake LLM; with turns that open windows, and messages refused and left
  unread: hosts/README.md, section 9.

Each child's world plays its parent and the store as engine/README.md, 5.6
says: it keeps saved records, commits at the end of each step, and
restarts by restoring them, so restart is tested in each child before the
core exists.

## 4. The core's world

The core with every child beneath it, under a root of jig's own for
testing, against fakes that share none of its types:

- **jig's test connector** (section 7);
- **one to three scripted workers and the engine's own slots,** whose
  runs play scripts: turns, calls, waits, parks, answers, crashes and
  silence;
- **scripted parties,** making requests and answering what waits for
  them;
- **the store,** skein-kv's in-memory mode, slow, holding commits, or
  failing one, which stops the engine;
- **restarts,** cold, at drawn moments or at one a scenario chooses, from
  the store's last durable commit.

Its stories are engine/README.md, section 9's; its referee is section 6's.
Beside the referee, the documents the core delivers are held to a
reference made from outside: a task query's rows against every task the
scenario made, filtered and ordered as the query asks, its ended part
within what its index serves; what each task waits on against the
tree and the entries the scenario made.

The `joint_clock` and documents fixtures compare a source's
deadline in task waits, accounts and deployment documents and in its
stored reset. Different injected clock pairs between reads leave those
wall facts exactly equal, including after restore: literal equality,
never a real-clock sample or tolerance (accounts.md, 3.5; engine/README.md, 5.4).

The charters and templates stories check `me`'s
`resolved_default` with catalogue and charter orders opposed, explicit
and omitted intent, a chosen executor, no eligible model, and account,
policy, membership and effective-executor changes that refresh the
preview, and each project's offer: a provider with no usable account gives
no model, and an account's change changes it. The rule is a run's check's
(accounts.md, 4.3), and the payer the current receiving party (engine/views.md, 1).

Settings (engine/settings.md) are tested where each part lives, most
of it below this world: the current settings from defaults and changes,
the check, a charter's version for a revision, the held revisions'
accounting and the bounds' arithmetic as step tests of the core's
settings functions, pure and fast; the records' round trip and
goldens in the store's tests (`../protocol/store/README.md`, section 9); the
document's members read as defaults, and the settings' limits, in the
configuration's tests; the change's and the move's records in the
client wire's. This world holds what needs the decision and the
children together: the stories of engine/README.md, section 9. None needs a
process: the world's cold restart shows what a restart keeps.

First-start and restart stories use people roster capacities equal to
and above the actual configured project count; they seed the whole set
and keep it across restart. Invalid programmatic configuration is
`Core::new`'s precondition, never an invalid world constructor or a
catch-panic scenario (engine/README.md, section 8).

The inbox and ended-mark fixtures control the current subject,
role and requester and the interleaving of ended-record and position
loads with overlapping marks and forget. A completed `Seen` retried
after mark and forget answers `Unknown`, writes nothing and never
recreates history; lost authority can answer `Role`. Holding the journal
commit proves no external success leaves early. Crashing and restoring
recovers only committed read positions or committed absence, with the
mark and any consequent forget atomic (people/README.md, section 5). These
proofs are in the step and world tiers, within their budgets.

## 5. The conformance world

What every application runs, and what makes jig's promises its own.

- **What jig gives:**
  - the harness, generic over the application: it builds the
    application's engine domain from its configuration, feeds it events,
    takes what it asks for from the journal, and plays its peers;
  - the fakes for jig's peers (section 7);
  - the faults: a crash after any commit, the engine restarted cold from
    the last durable one; commits held, so the engine runs ahead; a
    commit failed; hosts lost, slow, or back after their grace; parties
    sending a request twice;
  - the adversarial scenarios every application runs, with tiny limits:
    - an effect's copy arriving late, after its retry deadline and after
      a retry; a target state another hand reached too;
    - the engine restarting again and again while an effect is
      uncertain, before its deadline passes;
    - a judge's facts changing between its verdict and the effect;
    - a pool shrinking while its slots are held and a task waits;
    - the policy narrowing while a proposal waits, an entry is committed
      and a run is live;
    - a standing task running across many periods;
    - many sessions and sub-agents of one run near the end of its
      budget;
    - a compound request, the engine crashing before and after its
      commit and the request sent again: made whole once, or not at all,
      and answered once;
    - a cancel with a grace across restarts: the task finishing within
      it, or cancelled once when it ends, from the grace kept with it;
    - a party's and a run's decisions about one task racing: the one
      named with a revision that has moved refused, never made over the
      other;
    - a conversation ending while its party reads it, the engine
      restarting between: no turn read comes back unread, and a task
      forgotten once read leaves nothing a page shows;
    - a task query watched while its tasks end, and watched again after
      each restart: every delivery after its commit, the ended ones
      found through their indexes;

    Each runs with the application's own kinds. One the application has
    no kind for (a pool, where its connectors take no pooled holds) is
    reported as not applicable, never faked with a different mechanism;
    jig's own worlds still cover it;
  - the referee (section 6).

`jig_conformance::core` translates the core's rows and released outputs into the referee's snapshot, and answers the core's peers as every application's adapter would, so an adapter writes only its own root's and connectors' part.

An adapter's view of its store is made for a reader, from the rows as they are, and never kept: `Got` changes rows outside commits, so a commit's number doesn't name their state. `Observer::inbound` asks for one only for an event whose check reads a row (a party's request, a proposal of settings, a run's accepted proposal), and `Observer::submitted` only for a commit that erases a task's activity or an ended task. A new commit always has one.

The core's observer keeps keyed row evidence: submitted keys replace or
erase their contributions after durability. Receipts retain each contributing
row, and person spend sums current ledgers. Observations remain complete,
ordered snapshots once per commit, checked against whole-store projection
in focused tests. Cold observation reads current rows even without a new
commit; a missing feed or a bulk mutation rebuilds the projection whole.

- **What the application gives,** through the harness's trait:
  - its engine domain, built from configuration;
  - its systems' fakes, as step machines, each reporting what it saw in
    the referee's terms: which effects reached it, under which keys, on
    which resources, of which kinds, at what condition. The fake reports
    what it observed in its own system's terms, mapped by the fake, never
    by the root, so a root that mistranslates an effect is caught when
    the observed effect differs from the one the core decided;
  - its scenarios: the parties' scripts, its systems' starting state and
    faults, its agents' scripts;
  - the policy each scenario runs under, which the referee reads as its
    independent statement of what is allowed, and whose kinds hold each
    kind of effect's recovery class, which the referee holds the fake
    system's observations to.
- **Crashing at every commit.** For a scenario, the focused suite crashes
  after a few chosen commits and checks the story still ends as it
  should; the fuzzy suite sweeps every commit of the scenario, and draws
  cuts at random across random worlds. A sweep using `Cut::EachCommitFrom`
  holds its prefix once: the run forks at each commit from the one
  named, and the child takes that commit's crash as `Cut::AfterCommit`
  does, through the same cold restart, and ends the story; the parent
  waits for it and goes on uncut, so no cut repeats the commits before
  it. A child that fails fails the run, naming its cut, and the run's
  `Outcome` lists the cuts whose children passed, so that a sweep checks
  it missed none.
- **What it catches,** where its scenarios reach: a root that sends a
  held output early, splits a decision across commits, admits after a
  child changed, restarts out of the core's order, lets an effect reach
  a connector unchecked, or translates one wrongly (root.md,
  section 11); and a connector that repeats an effect beyond its
  recovery class, gives a verdict on facts older than their freshness,
  forgets a retry deadline across a restart, or decides differently when
  stepped twice on the same facts.

## 6. The referee

jig's referee watches from outside (testing-strategy.md, section 7): what
the fakes saw, what the store made durable, and the facts the domain
emits, never a domain's state. Each promise of model.md, section 11
becomes observations:

- **Authority holds:** every effect a fake system saw is covered by the
  scenario's policy as it stood when the effect was decided, for the
  task that asked, or by a proposal whose acceptance the store made
  durable first; every effect whose kind has requirements saw, before
  it, a met verdict from each judge, each observed one within its
  freshness when decided, each guarded one still true when applied; the
  effect a fake observed is the effect the core decided, in kind and
  resources; nothing waits as a proposal that its proposer did not ask
  for: a run's `propose`, a step's proposal, a party's `Propose`, or a
  result's follow-ups.
- **Spend is bounded:** recorded spend never passes a task's budget,
  and actual spend, the fake LLM's own count, passes it by no more than
  the turns lost with workers allow.
- **Once, as each kind allows:** a keyed or conditional effect takes
  effect on its fake system at most once, late copies included; an
  idempotent one ends in the decided state; an unrecoverable one whose
  outcome was uncertain is never repeated before a person decides; every
  batch, and every compound request, is in the store whole or not at
  all; every uncertain entry is resolved, or held, within a bound
  however often the engine restarts.
- **Nothing ahead of its commit:** every effect a fake system saw, every
  assignment a host saw, every answer a party or a run saw, and every
  document a party's watch or read was given followed a durable commit
  that holds it; after a crash, nothing seen is missing from the store.
- **Order:** no task started before its dependencies were done and
  closed; no two runs held one resource's writer slot at once; no pool
  admitted a holder beyond the slots it was known to have; no task ended
  before its delegates.
- **Nothing lost:** liveness deadlines for every result to reach its
  requester, every person's words to reach their task or the task to end,
  every proposal to be decided, withdrawn, or waiting where a holder sees
  it; no turn a party has seen comes back unread, across its task's end
  and restarts (people/README.md, section 5).
- **Nothing written over:** no effect on an owned object that the
  application did not make it, but as a participant or through its
  connector's mechanics; no amendment or release made against a
  revision that had moved since the one its party, run or procedure
  named.
- **A run reads its revision:** every assignment a host saw carries the
  charter version of the revision its run was prepared under, and every
  task's revision in the store was the newest when it was made or
  moved; every change of settings is in the store whole or not at all,
  and a refused one left nothing.
- **Bounded:** each fixed scenario's measured heap peak stays under
  its named ceiling at every iteration (testing-strategy.md, section 6),
  and every story ends within its bound. The core world's empty closure
  uses `EMPTY_CLOSURE_HEAP_CEILING`, 1,300 bytes of additional heap.
  An uncapped queue grows from a floor of four entries, so one output
  need not allocate only one entry; the ceiling includes that growth.

## 7. Fakes and kits

What jig ships for tests, as ordinary Rust or as step machines where
more than one world needs them (testing-strategy.md, section 4):

- **the store:** jig's store component over skein-kv's in-memory mode,
  with an in-memory stand-in for transcripts' files, the world holding
  commits, failing one, and serving pages slowly
  (`../protocol/store/README.md`, section 9);
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
  (`jig-test-kit`).

They stay in jig, and an application's worlds reuse them. What jig finds
missing in skein's world harness goes into skein first.

**Their checks return their findings,** as skein's do
(`skein/docs/design/testing/README.md`, 5;
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
- **A harness that names a seed, a story's choices, a scenario or a
  case** writes it to stderr before it runs it, and adds the trace's tail
  to a returned finding. A table of cases, the step crates' and the
  core's bundle (`tests/core/tests/support/bundle.rs`) alike, names each
  case, and the bundle its seed, before running it, and stops at the
  first that fails. The runner prints a failed test's output.

**A world's library is its own.** No package depends on a world; its own suite
reads its kit through its library, which re-exports it.
`jig/tests/boundary/src/tests/kits.rs` holds this. Two packages named as worlds
are kits by design: skein's world harness, `skein-world`
(`skein/docs/design/testing/README.md`, 6), and smith's `smith-agent-process-world`.

## 8. What an application adds

- **its connectors' worlds,** each with its system's fake below and the
  root scripted above (connectors.md, section 15);
- **its conformance scenarios,** which are its stories: `ops`'s are
  `../examples.md`, section 7;
- **its system worlds,** if it has workers: its engine, its workers with
  their workspaces, and smith's agents, together;
- **its own tiers below the domain,** with its protocol layers.

## 9. Budgets

Every tier follows testing-strategy.md, 8.1: CPU budget, wall hard limit,
starved warning and the three-question repair rule.

jig's tests fit the suites' budgets (testing-strategy.md, section 8):
the focused suite runs a few cuts and seeds per scenario, the fuzzy
suite sweeps. jig's tests count against jig's budgets; an application's
conformance scenarios count against the application's. The stories of
the plumbing (operations alone and compound, a grace, task queries,
read positions, revisions named by runs and procedures) join the
scenarios of the worlds they belong to, a few cuts each in the focused
suite and swept with the rest in the fuzzy one: none needs a suite of
its own.

The core world's deletion, startup, effect-and-terminal, failed-commit
and decision-read stories keep their whole sweeps in fuzzy. Focused
chooses the decision, family-step, sign-in, chat, claim, effect and
terminal commits for what a crash there can lose. The store world's
commit cuts remain focused.

The stock profiles' room for a day of deciding requests is arithmetic
over both profiles in the configuration's step tests. Focused drives
one world to `Small`'s request bound and restarts it; fuzzy drives
`Standard`'s whole room under the request world's heap ceiling. Both
worlds check every answer, the last request and the first request's
replay after a cold restart.

The store's generation, patch, check and frame scenarios are focused tests
in `jig-store-world::suite`, on the simulated disk. Focused cuts each a few
times, chosen for what a cut there can lose: a first start's first directory
and its switch, a stray's first and last removal, the drop, the copy and the
switch of a patch, the seal and the check, a patched read's first and last file
operation. Their whole sweeps are under `fuzzy::` in the same binary, each with
its focused cuts' body. Layout,
manifest and admission cases are data rows of their behaviour's table;
copy mutations and check/switch cuts have one owner each. These are
component-world scenarios, not a separate tier. `jig-test::process` also
runs in focused: three paths through the shipped binary against fakes,
including its bundle page under the reference policy. Startup, HTTP and
child-exit waits use the shared real loop's `Deadline`, with the measured
starvation verdict (testing-strategy.md, 2.10). Offline live-configuration
and refused-plan diagnostics run in focused; real live stories remain
opt-in.

Settings' tests are priced so (engine/settings.md). Their step tests
are pure functions over a few settings, each well under a millisecond.
The core's world gains about a dozen focused scenarios (half with the
change, half with revisions and moves), the tasks child's and the
fleet's a few each, each as long as the scenarios beside it, a few cuts
each: about what the runtime projects' stories added, a small part of
the focused suite's 30 seconds. The fuzzy suite gains no sweep: a change
of settings, and a move, become two more operations its random worlds
draw, at the cost of the steps they take. Each change that adds them
measures both suites before and after on the gate; past a budget, it
asks the three questions in 8.1 in order, taking the measurements to
the user if neither placement nor an efficiency fix settles it, rather
than cutting the cover.

## 10. Open questions

- **How many cuts the focused suite takes** per scenario, and which: the
  commits around an effect, a claim and an answer are the likeliest to
  show a bug.
- **Coverage of the root's arms:** whether the harness should report
  which of the root's routes a suite never exercised, as transition
  coverage does for state machines.
- **A world's bare numbers** (7): the rows of
  `jig/docs/development/debt/D035-bare-numbers-in-worlds.md` take a name or say
  why the number is the point. A request's key stays literal: a retry repeats
  it, and its `[key; 16]` and reply token follow from it. A seed stays per file,
  naming its run.
- **The core's arms in an adapter** (5): whether a boundary lets the kit answer
  the core's peers itself, with a trait kept for `Harness<A>`, the charter's
  decoding outside the kit, since it links smith, and each adopter's first
  period and `Drained` as the referee sees them. Default: as they stand, until
  `jig/docs/development/debt/D030-conformance-not-drop-in.md` is planned.
- **Owned conformance observations** (5, 6): reducing snapshot copies needs
  an ownership design that keeps every referee check and complete ordered
  observations. Required cold restarts and independent public readback stay;
  cheaper construction is lower-layer work. Further naming and word-check
  savings below the roughly 2% per-story stopping rule stay deferred. The
  instruction pairs, overlapping shares and retained causes are in
  `temper/docs/development/limits-worlds.md`; changed-row-only verdicts
  cannot replace the whole observation contract.
- **Further world-cost repairs** (7, 9): which remaining representations and
  algorithms need to change for the focused suite to fit its 30 CPU s budget.
  Read whole-suite cost from the queue after landing, including fuzzy's moved
  core and store sweeps and `Standard` drive. A world's real first start and
  every story's cold restart stay; further changes to the product's catalogue
  construction are separate work. The protocol kit's socket walk may give
  way to an ordered set of active sockets only with the same cuts, seeded
  schedule and event order. A due agent's readiness is recomputed: a stored
  answer needs visible invalidation for every input, including clock-driven
  account and credential expiry. Apply testing-strategy.md, 8.1 in order;
  neither budgets nor build settings change to make a cost repair pass.
- **Smith's world kit** (7): `smith-agent-process-world` is a kit by name, as
  smith has no `testing/`. Default: as it stands, until a debt item of smith's
  names it.
