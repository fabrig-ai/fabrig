# failure-worlds: what failed on the reference deployment, found in deterministic worlds first

Draft, 2026-11-20, on the user's word. In the week from 13 to 20 November the reference deployment's goals hit 125 problems in goal execution (`../../development/failures-1113-1120.md`). Almost every fix that followed came with a test reproducing its failure, written after the incident, in a world that could already express it: the worlds had the machinery, but nothing made them meet the failure before production did. This draft gives temper's worlds checks that hold for every scenario, the faults production showed, a workload that reaches the shipped limits, and sweeps that draw both from the seed, so that a failure of this kind is found at the gate. It builds no fix: each problem still open keeps an ignored scenario naming the work that fixes it, and the sweeps leave out only the narrow fault combination that scenario reproduces (section 6).

## 1. In one page

- **Checks every world holds** (section 3), each stated as what it observes and through what: every wait has a wake; every resource is released by its own lifetime rule; blame and tries follow the injected cause; finite goals end, or are held with words, and goals a fault doesn't touch keep going; a restart changes no outcome it shouldn't; the shipped settings are consistent; and every commit leaves a store a start would open.
- **The faults production showed** (section 4), each marked as a new behaviour of a fake, a new script of an existing one, a new composition of existing faults, or a new assertion over faults that already fall; and each as reachable by a legitimate sequence of events or as a synthetic fixture of a state no such sequence makes.
- **A workload that reaches the shipped limits** (section 5): a random workload in each root's world, and the composed system world running several goals, a milestone and a landing train at once.
- **Open problems are ignored scenarios** (section 6), as the strategy keeps any finding not yet fixed. A sweep excludes only the fault combination an ignored scenario reproduces, by name; the draft that fixes the problem enables it. No finding is ever whitelisted.
- **The catalog** (section 7) names each open problem's scenario, what it asserts, its harness, how well its cause is established, and the work that fixes it. Problems already repaired are held as active regressions under the new faults, with no exception.
- **The budget is planned first** (section 8): per tree and layer, from the suites' own reports, before any sweep is added. Moving budget between layers is the user's decision.
- **Restarts are the restart drafts'** to fix (section 9), and their faults and checks are this harness's (4.8, 3.3, 3.4, 3.5, 3.7).

## 2. Where it starts

- **The strategy asks for this already** (`skein/docs/foundation/testing-strategy.md`): deterministic worlds below the real loop, faults drawn from the seed in every state (section 3), tiny limits, contracts as a world goes and invariants over its frozen state once it settles (section 6), and a referee with safety and liveness expectations over what it observes from outside (section 7).
- **temper's worlds have most of the machinery** (`../testing/README.md`):
  - the composed system world runs the real engine and worker with smith's agents over skein's fake LLM and fake forge (2.1);
  - every engine world has a referee, whose stimuli include an engine restarting cold and a worker vanishing (4.2);
  - scenarios run at the shipped `small` limits (4.1);
  - the one-process world runs the service on skein's simulator, its clock and its seeds (2.3, 2.5).
- **The fakes have most of the faults.** The fake forge fails requests, rate-limits, conflicts merges, delays webhooks, and runs CI that passes, fails, never reports, loses its runner or is cancelled (`../testing/fakes.md`, 2; `skein/docs/design/testing/fakes/forge.md`, section 2). The fake LLM exhausts an account with no reset, and its failure scripts can answer with a provider's own codes (`skein/docs/design/testing/fakes/llm.md`, section 2). smith's fake `claude` can answer with any status and error text (`../testing/fakes.md`, 1). Oversized pull requests already have focused and fuzzy cases.
- **jig's conformance world** crashes the engine at every commit, and its referee checks jig's promises from outside: bounded delivery, order and ownership, and nothing lost across restarts (`jig/docs/design/domain/testing.md`, sections 5 and 6).
- **The design states rules the checks hold:**
  - a landing's step never waits on what nothing will answer (`../domain/landing-waits.md`, section 1; `../domain/landing.md`, 4.2);
  - a red that isn't the change's asks no repair (`../domain/landing-proof.md`, 2.4);
  - jig's promises (`jig/docs/design/domain/model.md`, section 11).
- **Most of what failed can be shown in memory.** Of the 125 problems, 79 have a mechanism a tier below the real loop can show: 50 in step tests, 24 in domain worlds, and 5 in system, machine or protocol worlds. 26 need real machines, real providers or a person's action, and 20 have no established cause (`../../development/failures-1113-1120.md`). The faults that broke things mostly fall already. What the worlds lack is three things:
  - the combinations of lifecycles they exercise;
  - what their checks require of every scenario;
  - a workload that reaches the shipped limits.
- **What is missing, then:**
  - no world holds *no silent wait* for every kind of wait temper has, outside the landing's own steps;
  - nothing checks that a bounded table's entries are released by their own lifetime rules, outside jig's conformance;
  - nothing compares what the engine blamed, and what tries it spent, with what the world injected;
  - no world runs several goals at once at the shipped limits;
  - the start's link checks run only at a scenario's final cold restart.

## 3. The checks

Each check is one of three kinds, as the strategy separates them (`skein/docs/foundation/testing-strategy.md`, sections 6 and 7):
- **A referee expectation** observes only what the fakes saw, the facts the engine and the worker emit, and the durable records the store committed, never a domain's state. It is written once and runs in every world that has a referee.
- **A harness invariant** inspects a world's frozen state between iterations, through an **inspection adapter** each component exposes to its worlds alone: the engine's for its landings, changes and tables, jig's for its tasks, waits and tables, the worker's for its workspaces and disk. An adapter is test code, built with its component, and reads what its component keeps without changing it.
- **A settings test** runs over shipped settings, not in a world.

Each returns its findings, as every check of temper's worlds does (`../testing/README.md`, 4.2), naming the object, the moment and the seed. A check that can't read what it needs is a gap in its adapter, to be built, never a finding of the product's.

| Check | Kind | Reads |
|---|---|---|
| 3.1 | harness invariant | each component's waits, through its adapter |
| 3.2 | harness invariant | each component's bounded tables, through its adapter |
| 3.3 | referee expectation | the fault ledger, the engine's and smith's emitted facts, the fakes' observations |
| 3.4 | referee expectation | the goals' durable records and emitted facts |
| 3.5 | referee expectation | two runs' durable outcomes, normalized |
| 3.6 | settings test | the stock profiles, policy and catalogues |
| 3.7 | harness invariant | a committed snapshot, restored apart from the live world |

### 3.1 Every wait has a wake

- **What holds:** every wait a live task, change or landing is in has a documented kind and a wake source that is live. The documented kinds come from the design:
  - waiting on its requester, standing, on its dependencies or delegates, on a timer or a watch (`jig/docs/design/domain/tasks/lifecycle.md`);
  - held, with its hold's kind and words (`jig/docs/design/domain/tasks/holds.md`);
  - a landing's named waits (`../domain/landing-waits.md`, section 2), and a change procedure's.
- **A finding:** a wait of no documented kind, or one whose wake source has ended or can't wake it, such as a parked coordinator whose only watched task waits on its answer. The finding names the waiter, the wait and its wake source.
- **When:** at each driver's own quiet point, as that driver defines it: the direct driver's fence once it has drained its immediate work (`temper/tests/engine/domain/src/direct.rs`), the harness's run once nothing is due before time moves. It never relies on the whole world being quiet, since another task's steady work can keep a world busy for good (the reference deployment's busy ready set, S9).
- **A wait with a bound** also has a deadline armed in the referee, from the bound its design gives, so that a wait starved while other work goes on is found at its bound.

### 3.2 Every resource is released by its own lifetime rule

- **What holds:** each bounded table or slot names, for each entry, the rule that releases it, and the adapter reports the rule's state. The rules differ by table:

  | Table | Released by |
  |---|---|
  | forge write entries | the write's settlement, or its owner's end with the entry withdrawn |
  | forge read admissions | the read's answer |
  | branch holds and landing rows with their asks | the closing settlement of the landing or change that holds them |
  | kept answers to change requests | their retention's expiry, or eviction when room is needed (`jig/docs/design/domain/people/acts.md`) |
  | creation reports | the batch's answer; pinned while a batch that hasn't answered needs them; evictable otherwise (`jig/docs/design/domain/tasks/holds.md`) |
  | subscriptions | the subscriber's end, or unsubscribing |
  | a sign-in's pending exchanges | each request's answer |
  | build places, workspaces and per-run scratch | the run's end, or the workspace's release |
  | worker Git stores' packs | the packing their own counts trigger |

- **A finding:** an entry its rule should have released, and hasn't, once the rule's moment has passed; or a newcomer refused for room that frees and never admitted within its bound. Admission's progress is checked as well as release.
- **Ended records kept on purpose** (an ended task's row, a kept answer within its retention) are no findings.

### 3.3 Blame and tries follow the cause

- **The fault ledger:** every fault the world or a fake injects is recorded with its kind, its interval (when it starts and stops), and the identities it strikes: the account, the runner and gate run, the candidate and the base, the request. A change's tree that the scenario made wrong is recorded with the candidate it is wrong in, and a base made red is recorded as the base's, so that blame can be judged against the candidate and the base alike.
- **The counters, each its own:** a provider request's retries; a session's retries within one call; a task's tries; a gate's retries; a change's repairs; a person's asks. Each failure class has its expected transition on each counter, in a table the check holds:

  | Failure | Provider retries | Task tries | Gate retries | Repairs | Hold names |
  |---|---|---|---|---|---|
  | a rate window | wait out the window | none | n/a | n/a | the account, if it persists |
  | spent credits, a billing refusal, an unserved model | none after the first | none | n/a | n/a | the account and model |
  | a runner stopped mid-job, a gate with no report | n/a | none | one each, to their bound | none | the runner |
  | a starved gate with no failing test | n/a | none | as the runner's | none | the runner |
  | the candidate's own red | n/a | n/a | none | one | the change |
  | a red inherited from the base | n/a | n/a | none | none | the base |
  | a settled stop, or a restart announced to the workers | none | none | none | none | none |
  | a crash, or a worker lost mid-run | as the design says (today a lost run spends one) | as the design says | none | none | the run, if its tries run out |
  | a stream cut after its head | with backoff, to their bound, acting on no partial output: a cut tool call never runs, nor runs twice | none | n/a | n/a | the provider, if it persists |

  Where today's policy differs from a row (a lost contact with a job spends a try today, `jig/tests/core/tests/suite/jobs.rs`), the row is the target, and the difference is a scenario of section 7 with the draft that settles it. A stream cut after its head is one: on 2026-11-20, between 20:09 and 20:15Z, ten tasks' Responses streams answered 200 and ended before their completion event. skein-llm reports that as `Protocol` ("incomplete Responses stream", `skein/crates/skein-llm/src/openai/response.rs`; Anthropic's alike), which smith's retry classes as lasting (`smith/crates/smith-domain-session/src/session/retry.rs`), while a connection closed within a response is `Unavailable` and transient. So the tasks spent their tries in minutes and needed a person's release. Its scenario is `a_cut_stream_is_retried_not_failed`, fixed by *provider failures*.
- **Controls:** a scenario with two faults on one change at once (a wrong tree and a stopped runner), and one with a red base under a green candidate, so that the check is shown to tell them apart.

### 3.4 Finite goals end, or are held with words, and others keep going

- **Which goals:** those whose work is finite in the scenario, whose parties are scripted to answer, release and decide within the script, and whose faults stop by a moment the scenario names. A goal a party holds, or whose party is scripted never to answer, may stay held or waiting; that is an allowed outcome, named in the scenario.
- **What holds:**
  - each such goal ends (landed, or ended as its procedure says) within its bound past the faults' last moment;
  - a goal struck by a fault that never stops is held, within the same bound, with words naming the fault;
  - a goal the fault doesn't touch (another account, another repository) keeps going as if it weren't there;
  - after every restart (4.8), the forge down through it included, the engine opens within a bound, and every goal goes on as above.
- **The bounds** are explicit and nonzero: a number of simulated minutes and a number of steps, each named by the scenario, or derived from the design's own bounds for the steps a goal takes. The step bound catches a loop that works without moving time.

### 3.5 A restart changes no outcome it shouldn't

The restart drafts' oracle (section 9).
- **Comparable runs:** the workload, the faults and the scheduling draw from independent streams of the seed, so a run with restarts and one without draw the same goals, the same faults and the same parties' actions. A party's actions are replayed as logical events (answer this question, cancel that goal), not at fixed instants.
- **What holds:** the same goals end the same way, with the same trees landed and the same holds' kinds, compared after normalizing ids and times. Tries differ only where the design says a restart spends one, and a settled stop spends none.
- **Which cases compare:** a scenario whose outcome depends on a deadline racing a restart is left out of the comparison by name; its recovery is 3.4's.
- **Crash safety and recovery liveness** stay separate checks: 3.7 and 3.4.

### 3.6 The shipped settings are consistent

Settings tests, in four parts, each stating its relations and their valid exceptions, and over generated mutations of the stock settings, the decision each mutation must get:
- **The stock profiles:** each figure a design derives from another is derived and not set, unless an explicit override is given (a component's deadline from the client's lifetime, `temper-engine-service-vocabulary`'s limits); a table too small for what it must hold at the profile's other figures is a finding only where its design says it must hold all of it, since refusing a batch is a valid answer.
- **The policy's references:** every producer, model and charter a stock template names exists and is live, and settings apply refuses retiring one a template still names.
- **Provider capability fixtures:** a model a stock template names has efforts and limits in the catalogue a provider preset gives. What a real account serves stays a live fact, outside these tests.
- **Amendment admission:** no proposal is addressed to a role whose acceptance the same check would refuse. Judging an amendment by what it widens (the user, 2026-11-20) is the fix of section 7's S15 row, not a check here until it is built.

### 3.7 Every commit leaves a store a start would open

- **What holds:** after each commit, a store restored from that committed snapshot, apart from the live world, passes everything a start checks: jig's checks of its children, its link pass and what a start derives after it, and temper's connector checks above them. A record a start would refuse fails the world at the commit that made it, naming the record and the commit.
- **How:** jig gives an isolated restore-and-check operation over a committed snapshot, which never touches the running world; temper's connector checks run on what it restores.
- **Why every commit:** run only at a scenario's final cold restart, the same checks let three writers' defects reach the reference deployment that week, each found at its next restart (2026-11-15, 11-18 and 11-19).
- **Its cost** grows with the store at each commit. It runs at every commit in the worlds whose budgets carry it. Elsewhere it samples commits from the seed and reports how many of each run's commits it checked; an exhaustive claim is only a verified commit-cut sweep's (`jig/docs/design/domain/testing.md`, section 5).

## 4. The faults

Each fault is drawn from the seed in every state it can fall in, as the strategy's are (`skein/docs/foundation/testing-strategy.md`, section 3), and a sweep asserts that each fault it configures fell at least once. Each is marked:
- **[behaviour]:** a fake does something it can't today;
- **[script]:** a fake can already, and a scenario's script must ask it;
- **[composition]:** faults that already fall, combined in one run;
- **[assertion]:** faults that already fall, with a check that doesn't hold them yet.

A fault is **reachable** when a legitimate sequence of events makes it, or a **fixture** when it stands for a state no legitimate sequence should make, built as an explicitly synthetic store, to test recovery from it. Where a fault models what only a real machine does, the modelled outcome is what the world tests, and the real mechanism stays outside its proof.

### 4.1 The forge

- **Busy and slow** [script]: read admission refusing busy; a write answered after its client's deadline, its outcome unknown, the merge made or not; a merge past the component's deadline.
- **Lagging views** [behaviour]: a pull request's head or base behind its branch for a while; a merge recorded after a later write.
- **Past its bounds** [composition]: a pull request's file list, a comparison or a job log past the forge's answer bounds, while the change's repair needs them.
- **Refusing** [script]: a close refused by branch policy; a pull request merged or closed by someone else between two reads.
- **CI** [composition, assertion]: a runner lost or cancelled mid-job while its gate's change has a wrong tree too; a gate that never reports; a gate whose report artifact is missing; one runner shared by every gate.

### 4.2 LLM providers

skein's fake LLM (`skein/docs/design/testing/fakes/llm.md`) and smith's fake `claude` (`../testing/fakes.md`, 1), per account:
- **A shared rate window** [behaviour]: tokens per minute shared by every task on an account, answered `429` with the provider's own reset headers and no `retry-after`. Today the native fake's rate limit always sends a `retry-after`.
- **Spent credits** [script]: `insufficient_quota`, which never returns in the scenario.
- **A billing refusal** [script]: a documented synthetic envelope as Claude Code reports a refused subscription, told apart from an edge's or a proxy's refusal as smith's program classification does (`smith/docs/design/protocol/programs.md`). It stands for the class, and doesn't claim the reference deployment's own reply, which wasn't kept.
- **An unserved model** [script]: the provider's refusal of a model an account doesn't serve.
- **An expired sign-in** [script].
- **A request that hangs** after its head, or goes idle mid-stream [script].
- **A stream cut after its head** [behaviour]: a response that answers `200` and ends before its completion event, before any output, mid-text and mid-tool-call, in the fake LLM's domain and its peer (section 3.3's last row).
- **A saved window bound to another endpoint** [composition].

### 4.3 The worker and its machine

- **A workspace fetch timing out partway**, leaving git's lock and index [behaviour], as the fake machine models them; real git's locking stays outside the proof.
- **The fake disk filling** [behaviour], and a trash entry that can't be removed [behaviour]; a real filesystem's pressure stays outside.
- **A program started without its tools** [script], in smith's worlds.
- **A placement refused as busy** while the builder's build places are full [composition].
- **A delivery check's clock while it waits for a build place** [assertion], in smith's run world.

### 4.4 People and parties

Scripted parties, each action at any step [composition]:
- cancelling a goal, a change or a landing;
- holding and releasing a member of a live landing;
- amending a deadline after it has passed;
- answering late, never, twice, or to a run that has ended;
- words to a change whose producer has ended;
- a sign-in sending a request again while the first is pending.

### 4.5 Stale, reordered and rewritten state

Each from legitimate events [composition]:
- a forge's view behind temper's own last write, and a record of an earlier write arriving after a later one;
- a dependency landing squashed, so an early checkout's parent becomes unreachable;
- a saved resolution or save whose base has moved since;
- a pull request closed, merged or reopened by someone else between two reads.

### 4.6 Runs that park, resume and move

Each from legitimate events [composition]:
- a run parking with a question open, its answer arriving while it is parked, after it ended, or to its successor;
- a run that delivered before parking, finishing in its next run;
- a run resumed on another worker or another endpoint, with its saved window;
- a coordinator ending while a child waits for its answer.

### 4.7 The engine's own bounds

Every scenario runs at the shipped `small` profile (`../testing/README.md`, 4.1). The workload (section 5) runs enough goals at once to reach each bound of 3.2's table [composition]. Funds run at the stock policy's figures, so a goal's run and its delegates share its budget as production's did.

### 4.8 Restarts, stops and upgrades

The restart drafts' faults (section 9), each drawn from the seed at any iteration, not only between operations:
- **The engine restarted cold** at any iteration, as a crash [composition]: what was in flight is dropped, what wasn't committed is lost, and the next start runs from the store.
- **The engine stopped** [behaviour]:
  - a settled stop at any moment;
  - a stop in each phase of a start: before the start's mark, after it and before the opening, and after the opening while the forge still reconciles;
  - a second signal during a stop's settle.
- **The forge down through a start** [script]: every call refused from the restart until a moment the scenario names, or for good.
- **Tables full at a restart:**
  - [fixture] the forge's write entries all held by owners that have ended;
  - [fixture] a held goal branch the forge doesn't have, with no creation entry;
  - [script] an uncertain write whose find keeps failing;
  - [script] a read rate-limited at every attempt.
- **A worker stopped mid-turn,** in an LLM call, a tool call, a delivery check or a save, in two ways [behaviour]:
  - asked to drain over its link;
  - signalled by its service manager: the worker alone, or, in the simulated tier, every process of its unit, as a control-group kill does.
  
  The real service manager stays outside the proof.
- **A worker crashed mid-turn** at the same points, with no settle, coming back as a new process on the same worker or another [composition].
- **The engine restarted while workers are mid-turn,** announced or not [composition].
- **An older layout** [fixture]: a store and configuration as the previous release writes them, from fixtures that release's build writes in a world, never a deployment's own data.

### 4.9 What the faults cover

The mapping of every problem to its lowest tier found ten ingredients a reproducer most often needed (`../../development/failures-1113-1120.md`). Each is covered here, or said not to be:

| Ingredient | Covered by |
|---|---|
| A: bounded progress, or a useful end, once outside activity stops | 3.1, 3.4 |
| B: bounds saturated, with reservations and resources following their owners | 3.2, 4.7 |
| C: stale, reordered or rewritten state | 4.5 |
| D: questions and instructions carried through park, finish, resume and endpoint changes | 4.6, 4.4 |
| E: complete, encodable and accessible evidence across bounded boundaries | 4.1 (past its bounds), with each catalog row's own assertion on the evidence a repair reads; a structured record of what a check found is another draft's |
| F: peer outcomes beyond success | 4.1, 4.2 |
| G: deadline ownership: separate queue and execution clocks, derived bounds, retry windows | 3.4's bounds, 4.2's rate window, 4.3's check clock |
| H: blame and outcome matching the injected fault and the right base | 3.3 |
| I: observation of absent data, pagination and actionable states in watchers | not covered: the team's watchers are outside temper (section 10) |
| J: consistency across profiles, catalogues, templates, permissions and budgets | 3.6 |

## 5. The worlds and their sweeps

- **A random workload in each root's world:** the engine's domain and forge worlds, jig's core world and the worker's domain world each draw a workload from the seed, and run the faults their domain alone shows. They are the cheapest, so most sweeps run there:
  - the engine's worlds for the landing train and the change procedure;
  - jig's core world for waits, questions, words, funding and authority;
  - jig's engine-clients world for a sign-in's exchanges;
  - the worker's domain world for workspaces, disk and scratch;
  - smith's worlds for providers, programs and a check's clock.
- **The composed workload** is the composed system world (`../testing/README.md`, 2.1) running:
  - several goals at once, from two parties;
  - a milestone whose goals depend on one another;
  - a goal whose changes form a landing train of several members;
  - an after-landing tier following main;
  - one project on one fake forge, with a fake LLM of two accounts.

  Each goal's tree cues its agents' scripts and its CI, so its work is the same at every seed.
- **Independent streams:** the workload, the faults, the parties' actions and the scheduling each draw from their own stream of the seed, so a change to one leaves the others' draws as they were (3.5).
- **Sweeps:**
  - one per fault family, in the cheapest world that reaches it;
  - one in the composed workload that mixes every family;
  - each over the seeds its budget allows (section 8).
- **A failing seed** is fixed and kept, as a scenario if it shows behaviour worth naming, or among the sweep's pinned seeds (`skein/docs/foundation/testing-strategy.md`, section 8).

## 6. Open problems are ignored scenarios

- **Each open problem has a scenario** that reproduces it, in its lowest harness, ignored with a reason naming the work that fixes it, and run by that work's goals until they land it (`skein/docs/foundation/testing-strategy.md`, section 8).
- **A sweep leaves out exactly that scenario's combination** of fault, workload and state, by its name in the sweep's configuration, and nothing broader: cancelling a landing stays in the sweeps, while cancelling one with a gate's dispatch in flight is left out until the leak is fixed. Each exclusion names its ignored scenario.
- **The fixing work removes the exclusion** and the `#[ignore]` in the change that fixes it. From then on its sweep holds the fix.
- **Every other finding fails the sweep,** at the gate like any red, and the harness stops at it, as it does at any finding today. Nothing is matched by category, and no finding passes.
- **A problem fixed before this draft lands** has its own focused scenario from its fix, and gets no exclusion.

## 7. The catalog

The test contracts for the problems still open that a world can show, with their evidence: an **established** mechanism (the code shows it), or a **symptom** whose cause no record establishes. For a symptom, the scenario shows the handling the design wants of that symptom, and doesn't claim to reproduce the original cause. The work that fixes each is named by subject:
- *bounded waits:* every wait bounded, named and visible, and every table's admission rule;
- *landing train:* one owner for a landing's members, and a landing that can always move;
- *a change's reds:* one way to classify a red, one release rule, a record of temper's own writes;
- *provider failures:* failure classes per account and model, choosing again, funding by need, amendments judged by what they widen;
- *check record:* one structured record of what a check found;
- *worker disk:* the worker owning what it creates on disk.

| Problem | Evidence | Scenario | It asserts | Harness | Fixed by |
|---|---|---|---|---|---|
| P124 | established | `cancelled_landing_frees_its_write_entries` | a landing cancelled with a gate's dispatch in flight withdraws its write entries, and a change waiting for room is admitted once room frees | engine forge world | landing train |
| P124 | established | `a_change_waiting_for_room_has_a_bound` | a wait for write room past its bound holds with words | engine forge world | bounded waits |
| S7 | symptom | `a_landing_without_its_prepare_task_moves` | a live landing whose prepare child is gone, with a member held by a person, releases its turn or holds with words | engine domain world | landing train |
| P88 | symptom | `a_refused_release_is_accounted` | a resource release the forge refuses holds with words naming the operation, the resource and the forge's answer | engine forge world | landing train |
| P90 | symptom | `a_refused_close_of_an_open_pull_holds_with_words` | a close refused by policy on a pull still open holds with the forge's answer | engine forge world | landing train |
| P67 residual | established | `a_no_change_landing_repair_regates_once` | a landing repair ending "no change" leads to one regate, not a fresh repair | engine forge world | a change's reds |
| P37 | established | `a_starved_gate_is_the_runners` | a gate red with no failing test and unobserved tests retries as a runner's, and asks no repair | engine forge world | a change's reds |
| P53 | established | `a_resolution_keeping_a_deleted_path_is_refused` | a resolution that brings back a path deleted on the base, without naming it, is refused at delivery | engine domain world | a change's reds |
| P43 | established | `a_coordinator_watching_its_asker_is_woken` | a coordinator whose watched child waits on its answer is woken, and escalates past its tries | jig core world | bounded waits |
| P85 | symptom | `a_held_answer_tells_its_requester` | an answer held by a relay tells its requester what holds it, and reaches the producer when released | engine domain world | bounded waits |
| P113 | established | `a_change_waits_on_another_goal` | a change can wait on another goal's end without holding a question open, and its parent isn't held | engine domain world | bounded waits |
| P120 | established | `the_newest_ruling_reaches_the_resolution` | a held change's next resolution follows its newest authorized ruling | engine domain world | bounded waits |
| P117 | established | `a_parked_act_holds_no_exchange` | a parked act and a resend leave the sign-in's other requests answered | jig engine-clients world | bounded waits |
| P63 residual | established | `a_stored_charter_is_checked_at_start` | a stored charter whose fixed part no longer fits its room is refused at start with its words, not at its first assignment | engine domain world | bounded waits |
| S15 | established | `a_party_extends_its_goals_deadline` | the party that started a goal extends its passed deadline, judged by what the amendment widens | jig core world | provider failures |
| P77 | established | `an_owners_widening_is_never_proposed_to_itself` | an owner's widening within its own authority applies; none is proposed to a role whose acceptance would be refused; a proposal reads and decides through the client | jig core and client protocol worlds | provider failures |
| P26 | established | `a_limited_account_frees_its_tasks` | tasks kept on an account limited with no reset choose again when another account can serve them | system world | provider failures |
| P72 | established | `a_billing_refusal_marks_the_account` | the synthetic billing envelope marks the account refused, spends no task try, and moves the task | smith session world | provider failures |
| P127 | established | `a_rate_window_is_waited_out` | a 429 with reset headers and no `retry-after` waits out the window, and spends no task try | smith protocol world | provider failures |
| P126 | symptom | `an_unserved_model_moves_the_task` | a model refused for an account narrows that account's models, and the task chooses again | smith session world | provider failures |
| cut streams, 11-20 | established | `a_cut_stream_is_retried_not_failed` | a stream that answered 200 and ended before its completion event, before any output, mid-text or mid-call, is retried with backoff, spends no task try, acts on no partial output, and resumes without a person (3.3's last row) | smith protocol world | provider failures |
| P128 | established | `a_discovered_model_has_efforts` | a model a template names has efforts in the catalogue | settings test (3.6) | provider failures |
| P59 | established | `a_retired_producer_is_refused_while_named` | settings apply refuses retiring a producer a template names | settings test (3.6) | provider failures |
| P76 | established | `a_new_party_gets_account_uses` | a new service party gets an account's uses through the exposed account-use tools, with no list kept by hand | jig people world | provider failures |
| P93 | established | `a_start_without_a_budget_leaves_room` | a start with no budget takes a bounded share, and a second request from the same pool is funded | jig core world | provider failures |
| P81 residual | established | `a_json_proposal_carries_hundredths` | the `--json` answer's proposed command carries the amount in the unit ctl reads | client step tests | its own fix |
| P98 | established | `a_full_period_is_named_as_funding` | a party refused for a fully allocated period is told its funding, not its authority | jig core world | provider failures |
| P82 residual | established | `a_small_delegating_task_funds_its_delegates` | a delegating task near the minimum leaves its delegates enough for one run | jig core world | provider failures |
| S5 | symptom | `a_gate_repair_names_its_failed_tests` | the repair brief of a gate red names its failed tests, from the gate's report, when the job log is past its bounds | engine forge world | check record |
| S2 | symptom | `a_repair_reads_an_oversized_pull` | a repair whose pull request's file list is past the forge's bound still gets the files it needs, or is told which it can't | engine forge world | check record |
| P58 residual | established | `a_repair_whose_tests_pass_regates_by_itself` | a repair whose named tests already pass at its head regates once by itself, rather than holding for a person's release | engine domain world | a change's reds |
| P109 | established | `growing_packs_trigger_packing` | a store whose packs grow past the threshold is packed by the worker's own trigger | worker git step tests | worker disk |

**Held as active regressions,** with no exclusion, since main has their fix: P24, P25 (a merge cut at its deadline is reread, as P103's fix does), P27, P31, P42, P44, P46, P49, P50, P51, P52, P54, P55, P57, P58, P60, P61, P62, P65, P66, P67, P73, P83, P84, P86, P96, P97, P99, P100, P102, P103, P104, P105, P108, P111, P114, P116, P118, P125, S9, S10, S12, and P71 (`a_silent_run_is_cut_at_its_derived_bound`, smith host world: a run silent while its request hangs is cut by the request's own idle deadline, retried, and ended before the watchdog acts; its scenario passes as written and is enabled), and S1 (`spent_credits_stop_placement`, system world: `insufficient_quota` on an account marks it, and no run is placed on it afterwards while the other account's goal lands; its scenario passes as written and is enabled). The sweeps exercise their fixes under the new faults; a residual that turns up is a red, and gets its own row.

**Open, and with no row yet:**
- S3 (an oversized comparison) and S4 (no brief slot free) are reported, and the code reads as handling both: a brief's comparison past its bound degrades with a notice, and a full brief slot pauses without spending a try. The check-record work confirms or adds a row.
- Two residuals claimed from reading the code, which their work proves first, with a scenario, before they get a row: P86's other order (a fourth person task's subscription refused when the findings topic was taken first) for *bounded waits*, and P100's failing tests past a verdict's 64 names for *check record*.
- P116's runner setup, which stops a running job with no grace, is the restart drafts'.
- S17 (CPU budgets judged wrongly) is the budget tooling's, outside temper's worlds.
- The problems that need real machines, real providers or a person's action (CPU contention, disks, DNS, a hand landing, a session's own steps), and those with no established cause (P7, P41, P64, P107's mixing, P110, S6, S14, S24), are out (section 10).

## 8. The budget

- **The budgets are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1), held per tree and per layer (`tools/tiers/budgets.toml`), and this draft raises none. Room in one layer doesn't pay for another: moving budget between layers is the user's decision, asked with the figures.
- **The plan comes first:** before any sweep is added, the milestone's first goal reads each affected binary's CPU from the suites' own reports, and gives, per tree and layer:
  - what the new checks add to the tests that exist;
  - what each new sweep costs at its seeds;
  - what retiring duplicates and moving stories down frees, and where;
  - the moves between layers that would pay for it, for the user to decide.
- **Where the room is likely to come from**, by an inventory of every fuzzy test (2026-11-20):
  - temper's real-loop fuzzy stories, single scripted stories on the real clock and real processes, where `case_5_4` and most of the flakes are;
  - temper's one-process stories, scripted stories on the simulator;
  - the eleven engine-domain tests imported into the fuzzy suite unchanged from the focused one, and sweeps that repeat a focused test's helpers over more seeds.

  The inventory records few catches for these, which means none attributed, not that they never caught anything, so a test with unique coverage stays.
- **What moves:** a story's assertion moves to the tier where it belongs, when that tier can show it; a demonstrated duplicate is retired. A load-sensitive test is repaired at its cause (`skein/docs/foundation/testing-strategy.md`, section 10), never given more time.
- **One landing:** the milestone lands into main whole, so what it frees and what it adds arrive together.

## 9. Shared with the restart drafts

- **The restart drafts** (the reconciling start, and worker restarts that keep their work; 2026-11-20) fix the restart failures. Their faults are this harness's (4.8), so that one sweep can mix them with the rest of section 4.
- **Their checks are this harness's:**
  - a restart or a settled stop spends no try (3.3);
  - the engine opens, and every goal goes on, within a bound after any restart (3.4);
  - a restart changes no outcome it shouldn't (3.5);
  - every commit leaves a store a start would open (3.7).
- **Their milestones don't wait for this one:** each carries its focused scenarios in the worlds that exist, and its fuzzy cases join this harness's sweeps once it has landed.
- **Order:** this milestone starts after the reconciling start's first milestone, since that failure is live.

## 10. Out

- **Fixing any open problem:** the work section 7 names, each with its scenario as its acceptance.
- **What only real machines show:** CPU contention, a real disk's pressure, real git's locks, DNS, a provider's real behaviour, a real service manager. Their modelled effects inside temper are in scope.
- **People landing by hand, and the team's watchers** (ingredient I of 4.9).
- **Fixing what restarts break** (section 9).

## 11. Tests

Cheap: focused and fuzzy suites only; walk = false.
- **Each check of section 3** has a scenario where it finds what it should, and one where correct behaviour passes it, in the world that reaches it most cheaply; 3.3 has its two controls.
- **Each fault of section 4** marked [behaviour] has the fake's own step tests, and every sweep shows that its faults fell.
- **Each catalog row's scenario** compiles, and is ignored with its reason; each exclusion names its scenario.
- **The sweeps pass on main.**

Run only the new and affected tests, by name.

## 12. Later

- A live run's twin (`../testing/fakes.md`, 1) replaying a stopped deployment's agents in the composed workload, so a production incident's interleaving can be replayed rather than rebuilt by hand.
- Coverage of each check's finding kinds over the sweeps' seeds, as the strategy's transition coverage (`skein/docs/foundation/testing-strategy.md`, section 6).
