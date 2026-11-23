# What the tiers check

The service's startup world holds call-input composition at every
entrance (`../protocol/configuration/limits.md`): omitted allowances below,
at and above the agent's declared payload, explicit profile-default and
short allowances kept, core/store/worker-wire parity, engine-only
configuration, worker-only runtime and memory agreement, and per-kind
call/reply Terms mismatches. The binary's offline store reader supplies
checked agent metadata; its init metadata regression holds the declared
payload through that reader.

Every world checks what the strategy lists (skein/docs/foundation/testing-strategy.md, section
6). In temper:

- **Contracts** include each operation's deadline and identity, and each
  component's own (`../domain/worker/README.md`, section 9).
- **Engine replay** runs two independent walking, escalation or role
  worlds in lockstep under the same stimuli. After each iteration it
  compares their complete traces exactly, and hashes the frozen root,
  children, store and pending scripts through their `Hash` implementations
  (skein's `runtime/lib-containers.md`, 1.1). The world's test store hashes the four fields
  its `Debug` prints: `applied`, `rows`, `pending` and `held`.
  The comparison uses a test-only 64-bit word hasher: its
  digests are compared within one process and never kept, with no
  cross-run or cross-target promise. A digest mismatch is the
  comparison's finding, returned with both states' text even when that
  text agrees, and the world fails with it. At each seed's end the worlds
  also compare their complete `Debug` text, retaining checks of what
  `Hash` omits, a difference again a returned finding, before both close
  through the cold restart oracle. A test of the comparison reads its
  finding (`scenarios.md`, 1.2).
- **Secrets stay beside the domains:** in every world with the protocol
  layer, no git identity's value or API token appears in a domain's
  state, a fact, a store record or a log; the fake git program checks
  that a credential reaches git only through its environment (`fakes.md`, 3).
- **The process tier** adds that what the binary started ends with it:
  after a drain or a kill, no agent or git process of its remains; and
  that production's containment holds: a stopped run's process tree is
  empty, and a command cannot write a git directory.

## 1. Focused tests and fuzzy tests

Every tier follows skein/docs/foundation/testing-strategy.md, 8.1: CPU budget, wall hard limit,
starved warning and the three-question repair rule.

The two suites are the strategy's (skein/docs/foundation/testing-strategy.md, section 8). In temper:

- **Focused tests** are the strategy's, with facts changing nothing (a
  run is the same whether its facts are kept or dropped): every test
  in each world's `suite` binary outside its `fuzzy::` modules, the real-loop
  world's stories among them (`real-loop.md`). Step scenarios in the client domain and
  view also use `tests/suite`. Accounts, first-repository, dependency,
  change-panel, transcript, identity and rate scenarios run here: their
  behaviour defines their tier. Repeated formatting and codec cases are rows
  of the existing step tests; duplicated scenarios are removed.
  `tests/client/tests/suite/reloads.rs` checks watcher release across more
  loads than the party's watcher bound. The service's provider test holds
  document methods, query routing and authorization refusals without Chromium.
- **Fuzzy tests** are each world's `tests/suite/fuzzy/*.rs` modules. A sweep
  settles each random world under every invariant, and reaches every
  ending among them. A fuzzy module never compiles a focused module a
  second time, nor wraps a focused test with the same arguments: that
  repeats the focused suite's work with no draw of its own, and is
  retired, the focused test kept. A wrapper that is its scenario's only
  run is not such a copy, and stays.
- **A story keeps only what its tier alone shows.** A real-loop or
  one-process story keeps the assertions that only its tier can show
  (skein/docs/foundation/testing-strategy.md, 2.7 and 2.8): real
  processes and a worker's lives, a wall or a timer on the real clock,
  git and the forge reached over loopback, the simulator's disk and
  service. Each of its other assertions is a focused scenario of the
  lowest world that shows it: the engine's domain worlds for the
  engine's policy and its order, the system world for what a query
  carries to the model, and a step test for a check over supplied
  values. A story whose every assertion is held below is retired. No
  story asserts an order between its own work and a product deadline on
  the real clock, nor a count that depends on how fast the machine runs
  (a placement's rechecks, a poll's tries): those are the domain
  worlds', on their own clock, and a story that asserted one was
  load-sensitive at its cause (skein/docs/foundation/testing-strategy.md,
  10, *The same work at every run*). `reference-case.md` names, for each step, what its story
  still shows and which tests hold the rest.
- **Budgets.** temper's focused and fuzzy suites have a CPU budget for
  each layer they have, in `tools/tiers/budgets.toml`'s gate tables
  (`tools/tiers/budgets.md`, 2): the CI builder's gate figures of 11-16
  plus a tenth, by the user's word of 11-16
  (`temper/docs/design/draft/test-budgets.md`, 8). Over a layer's budget is
  red at the gate and a warning elsewhere
  (skein/docs/foundation/testing-strategy.md, 8.1). Their ideals are the
  focused suite's 100 CPU s, by layer: step tests 25, domain worlds 20,
  system worlds 25, machine worlds 1, protocol worlds 1, simulated worlds
  15, the real loop 10, end to end 3; and the fuzzy suite's 180 s, reported
  beside the budgets and never enforced (`temper/docs/design/draft/test-budgets.md`, 9).
  Each layer's per-test reference is the strategy's guide for it, and
  temper's own where these guides set one: a focused protocol world test
  1 s, and a focused simulated, real-loop or end-to-end test 2 s
  (`tools/tiers/budgets.md`, 6). All are separate from the profiles' wall
  hard limits. Within them, guides:
  - the engine component's unit tests (`temper-engine-service-component`'s
    lib tests, every one a focused test) each within the strategy's
    40 ms unit-test guide (skein/docs/foundation/testing-strategy.md, 8),
    and their binary's process at most 20 CPU s. A test builds the
    services it checks and no other: no fixture builds a whole service
    only to take its store or to throw it away. A service builds each of
    its fixed tables (a wire's schemas, bounds, fingerprints and bridges)
    once, however many of its parts read it;
  - a focused protocol-world test at most 1 s, and a one-process or
    real-loop story at most 2 s;
  - a GitHub story of the real loop (`real-loop.md`) at most 1 s, and one that runs
    a whole chat (its agents, git and `ctl`) at most 3 s, so that its
    eight stay within the real loop's budget with the rest; a
    whole-chat GitHub story that would pass the budget goes to the user
    with its numbers.
  - a playground story walked natively through the web client's host
    (`../playground.md`, 9) at most 3 s: it runs whole chats, a delegate
    with its agents, git, CI and a landing, as a whole-chat story does.

  A test that misses its guide follows the three questions in 8.1; it
  moves only if what it checks belongs in another tier. It is never
  merged with another or named as an exception: a real-loop story with
  CI and webhooks (`real-loop.md`) is held to the 2 s as any other. If GitHub's
  stories would push the real loop's layer past its budget, the budget
  goes to the user with the measurements, and no real-loop story moves to
  the fuzzy suite to make room for them: cost never decides a test's tier.
  A guide above its layer's reference (a whole-chat or playground story's
  3 s) is a test that passes its reference: when it is new, or grows, it
  takes its line with its reason beside the budgets
  (`tools/tiers/references.md`, 2).
- **The layers' figures are measured as the suites' rules are:** user plus
  system CPU time per process, waited descendants included, summed
  across the layer's selected processes (skein/docs/foundation/testing-strategy.md, 8.1).
  No test's CPU is measured within a shared process: the gate runs a new
  or flagged test alone after the suite and holds it to its reference
  (`tools/tiers/references.md`). For comparing an
  individual test's guide by hand, run it alone through nextest on the idle
  development machine with the tier's profile and `-j 1`, taking the
  median of three runs; these diagnostic measurements keep the same
  budget rule and do not replace a shared-process suite total. So no
  test asserts its own wall time, and no story's
  deadline is its guide: a deadline is the world's patience, there to
  end a hung run. A story that shows a landing needs no slow poll sets
  the polls beyond that patience, so that one needing a poll fails
  (`suite::protocol::a_landing_merges_on_its_own_wakes_before_any_resource_poll`).
- **A layer past its budget** is red at the gate: its slowest tests are
  profiled, then made cheaper at the cause the profile shows, or moved
  only where their checks belong under 8.1, by named steps; otherwise its
  measurements go to the user (the repository's `AGENTS.md`, section 5),
  and until the user moves it, the budget stands. The worlds' former sums
  (8 s for the worlds below the domain but the real loop, 15 s for the
  real loop and 80 s of the fuzzy suite) gave way to the layers' budgets
  and ideals (`temper/docs/design/draft/test-budgets.md`, 9).
- **Outside both suites:** the process tier's binary `process` and the
  live tier's binary `live`, which both suites' filters leave out, run
  under the nextest profiles `process-temper` and `live-temper`, the
  live one after every landing and by `scripts/proof-live.sh` (`live-tier.md`). The
  gate never runs them.

A fuzzy test lives in its world's crate, beside the focused ones, and
uses that world: a world's settings, random ones included, are the
world's, not either suite's. The nextest filters select `fuzzy::` modules
in the `suite` binary. A binary's tests share its process, one at a time
(skein/docs/foundation/testing-strategy.md, 8, *A binary is a process*), so a memory scenario
meters only what grows within its own spans (skein's `heap`), and reads no
counter a sweep before it moved.

## 2. The worker's runtime world

`tests/worker/domain/src/runtime.rs` drives the real worker root and its
children against scripted engine and process records and deterministic
git terminals. Its independent boundary referee checks complete turn
replay, exact names and cumulative spend, read credit, identical answers,
separate push title and body, merge paths and expected heads, and release
at iteration reclaim points. Focused stories cover busy backoff,
reconnects, crossed acknowledgements, ordinary saved work and
snapshot-free merge parking. Replay compares digests of the full boundary
records and facts; dropping facts leaves every boundary record unchanged.
Referee tests inject a changed duplicate body and an unseen turn prefix
and require rejection. Counted memory in the root and its children
exercises saturated turn windows, transcripts and conflict paths; a small
deterministic sweep varies body lengths and one-to-three count and byte
windows.

## 3. The client and the engine, joined

A client's world on a scripted engine agrees with the client by
construction: only a world that joins the client to its engine shows
where either side has drifted from their contract. So `tests/client`
joins temper's client to temper's engine root, and there is no client
world of temper's on a scripted engine (../client/testing.md).

- **The translation** between the client's vocabulary and the engine's
  is exhaustive both ways, so a variant added on either side without
  the other is a build error.
- **The joint world is the client's acceptance.** Each client feature
  has its stories there, or extends one.
- **jig's scripted engine** is jig's; the forge's client family has no
  scripted engine of temper's, so its stories run against temper's
  engine root alone.
- **The agents' tools** run there too: their root, as a scripted agent
  beside the fake person (../client/ctl/testing.md, 2).
- **The TUI's host** runs there too (../client/tui/host.md, 1): its
  shipped host with its root linked, as a native shell holds it, whose
  exchanges and stream the world's front carries to the engine's
  clients' component as the transport would, signed in by the printed
  link the component makes; its stories read the host's frames as text.
- **A run's live text** has its story there, with a scripted host in
  place of an agent: the host sends a run's text in pieces, held after
  the first (jig's `protocol/channel/owners.md`, 1.4), and the fake person
  reads the chat's words being written, that piece before its "▍", then the
  rest of the held text, and then the committed turn, which replaces it.
  The joint world's front plays the run's worker: at the chat run's
  assignment it sends the reply's first piece up the worker link, as
  the text kind at offset 0, and holds; at each of the story's releases
  it sends the next of its parts, the rest of the text as a second piece
  at its offset, then the committed turn with the whole text and the
  run's answer.
  Retries, gaps and cuts are jig's views' to test (jig's
  `domain/engine/views.md`); this story checks temper's wiring
  from the engine's root to the chat's page. The same story over real
  agents runs in the real loop (`real-loop.md`).
- **A change waiting on a review on the forge** extends the story of a change
  watched through CI, its gate and landing: with a landing rule asking for the
  person's approval, the change waits at its merge, the person's *Needs you*
  draws its card (`../client/forge/README.md`, 4.1) with "Review on Forgejo ↗" at its
  files page's address, a review approving it at its head is posted on the
  forge, and the card goes as the merge is admitted, before the change lands.
  This is where the card and its link are checked whole, rather than in the
  browser tier: no browser story there reaches a change waiting on a forge
  with an address, the box's demo forge refusing such a rule
  (`../domain/forge/change/gates.md`, 3), and the tier is at its cap (`../client/web/testing.md`,
  3).

## 4. Settings in the store

How settings in the store are tested (`../protocol/configuration/settings.md`
; jig's `domain/engine/settings.md`, and its
`domain/testing.md`, sections 4 and 9, for jig's own). By the pyramid,
each behaviour in the lowest world that owns it, and one smoke above:

| Where | What | Priced |
|---|---|---|
| step tests, in their crates | a landing rule's record: its built-value round trips, the largest at its bounds within the kind's bound, one past refused; the forge's judge of each item, each refusal and an accepted rule, by a table; the two standing verdicts over a table of rules, deployment's and project's, enforcing or not, met, wait and refuse, a branch no rule covers; the stock defaults passing jig's one check for `temper-box init`'s document, for one with a single model and for one with the demo; each of the six members refused, naming its path; `temper-box init`'s document without them, and no `charters/` | milliseconds, outside the worlds' guides |
| the client's step and view tests (`../client/testing.md`) | How work runs' rows from the `settings` document and each setting, in each tag; the card for an owner and for a member; the review's split of a proposal's items into enforced and followed, and its effect; stale and accepted; each act as the operations it sends; the gallery's worlds and the terminal's goldens gaining the two pages | milliseconds each, in their suites |
| the forge's world (`tests/engine/forge`) | ../domain/forge/README.md 10's settings stories: a rule changed between two merges; a gate a rule adds asked for on a queued change, which leaves the queue; one a rule drops no longer waited on; a project's rule adding an approval; a rule refused; a change kept on its charter, and one held when it is no longer offered; a restart handing the values back before the records | about 0.3 s, seven stories at the world's usual cost |
| the engine's domain world (`tests/engine/domain`) | through the root and the core on jig's store: a change naming a landing rule and a charter committed in one decision, and refused whole when the forge refuses its rule; a project's rule changed by its owner, refused as not theirs for another; a new forge's charter reaching the next change's delegates while a live change keeps its own; a project made after the first start covered by the deployment's rule; the two requirements removed from the rules, landing left to the procedure's waits; the resume fixture restarting with changed values. A stored settings policy and a stored project policy, each admitted under a larger policy allowance and committed through the real store, refuse a cold start under the smaller allowance naming the setting or project; a policy at the bound is admitted in its own story. Root and store use the same test envelope, including sufficient journal and page bytes. | about 0.6 s |
| the engine's domain world, configuring by Ask temper (`../domain/agent/agents.md`, 1 and `../domain/agent/ask-temper.md`) | scripted runs of charter 11 from template 5: an owner's chat proposing and its acceptance heard; a member's, waiting for an owner; one withdrawn as stale, read again and proposed afresh; one rejected with a reason and revised; one accepted with running work moved; `see` reads of each kind, a held charter, a revision and the history, each part within its bound; `propose_settings` refused for a stale revision and for an item out of shape; a run with no `settings` family refused by its authority. The settings family's own tools, which no stock charter is declared (`../domain/agent/tools.md`), keep their stories on a charter of the world's own holding that family: the `settings` read's offsets and pages, a proposal from a note, the note's refusals and its replay, `more` refused to a charter that is not asking, and a rule refused at acceptance | about 0.5 s, its stories at the world's usual cost |
| the engine's domain world, Ask temper's configuring reads (`../domain/agent/agents.md`, 1, and `../domain/agent/ask-temper.md`, 1) | scripted runs of charter 11: a project's ways and questions read at their stock with none stored, and as stored once a change sets them; a read that finds nothing, answered not read; the inventory's people, its own project's alone, the person and temper's writer marked; a member's chat reads what an owner's does and a run without the settings family is refused; an approval's refusal naming no command | its stories at the world's usual cost, not measured apart |
| the engine's domain world, a box's approval identities (`../domain/agent/agents.md`, 1; `../domain/forge/adoption-person.md`, 1) | on a box whose owned Forgejo's `Reconcile` answered the person's id, and on one whose manifest-made App's account is the person's: the inventory names the person's forge account by the link; their owner-role rule with `people: 1` is accepted, and `people: 2` refused naming the forge; a link cleared when the deployment has a second owner, and the rule then refused. On a deployment of several members, and on a pasted App, the requester without a forge identity is refused as before, and accepted once a human collaborator is adopted as admin. The script checks context and outcomes, not a real model's judgement | within the domain world's focused budget |
| the engine's domain world, a box's approval waiting (`../domain/forge/change/questions.md`, 2) | on a box whose forge links its person and whose rule of theirs is accepted, as in the box's approval identities, on each forge it links: a change made and stepped to its merge waits on the person's approval; the person's `approvals` document holds its row, theirs to give, while a second party of another project reads none; the linked user's review approving it at the head is posted on the fake forge; the row goes as the merge is admitted, and the change lands. The read's rows by state, head and role are the root's and the forge's step tables | within the domain world's focused budget, steps of an existing story |
| the engine's domain world, a charter's documents (`../protocol/configuration/settings.md`, 6) | through the core's plan, the root's binding and the forge's gather: a run of a charter naming a document gets its section at the branch's head; after a change lands that edits the file, the next run's section has the new text; a heading's part alone; a missing file, heading or repository, a section saying so and the run going on. The gather's own cases (a heading's bounds, a cut at a line end, each section saying why) are the forge crate's step tests | about 0.1 s, four stories |
| fuzzy, in an existing sweep | one operation in the forge world's `suite::fuzzy::simulation`: random changes of landing rules among the changes stepping and merging; its referee holds that no merge lands without each requirement of the rules held at its decision | within the forge world's layer's fuzzy budget (1), measured with it; no new binary |
| the client and the engine, joined (3) | Ask temper's page, and `chat start --template 5` through ctl, starting Ask temper, whose run of charter 11 proposes; "How work runs" read at its defaults and after a change, with its revisions in use; a proposal's review page split into temper enforces and your agents follow, with its effect; one accepted, making a revision; one withdrawn as stale; a member's, waiting for an owner | about 0.4 s, within the system worlds' focused budget (1) |
| the one-process world (`tiers.md`, 1.5) | its fixture's document without the six members; its restart story with one change of settings before it | no new story |
| the real loop (`real-loop.md`) | one smoke: a gate added to a project's landing rule from the client, the next change asking for its review and landing after it | at most 2 s, its guide, within the real loop's focused budget (1) |
| the system world (`tiers.md`, 1.1) | running work keeping its revision, and `revision advance` moving it, one story (`../client/ctl/testing.md`, 4) | at most 2 s, a world story's aim (1) |
| the process tier | Owned Forgejo gates, real signals and process-tree supervision only (`process-tier.md`). Settings reads, proposals, owner decisions and held revisions use the CLI machine, core and system worlds. | process-temper; one focused wiring smoke |

- **The tests that move:** every test document and typed builder that
  writes the six members (25 files of documents, 20 of builders, a JSON
  fixture and the cluster documents) takes the stock defaults, or, where
  its stories need other settings, defaults of its own passed to the
  service in the stock's place (`../protocol/configuration/stock.md`),
  so that no story pays for a change of settings it is not about; one
  story per world makes its change through ctl after sign-in. A story on
  a cheap fake model gives its own run floor and cap: the stock sizes
  them from C, the default model's worst completion, so a fake's run
  would be capped at a few units (the fuzzy repair and case 4 keep 1
  unit, cases `fakes.md`, 1 and 3 a 10-unit cap). A world that copies init's
  seed onto a cheaper model prices the seed for its own model, and the
  cluster fixture keeps five charter slots for the stock's five. Their
  times do not change.
- **Its checks:** settings change the facts of the forge's world and the
  engine's domain world (how a merge is judged), so their focused and
  fuzzy scenarios run with them.
- **Measured** by 1's rule. Should the smoke miss its 2 s, or the real
  loop pass its layer's budget, the numbers go to the user; no story is
  cut or moved to fit.

## 5. Stage 1: the golden, its check and its judge

Moved to [`checks-stage-one.md`](checks-stage-one.md), section 1.

## 6. Checks every world holds

Written from `../draft/failure-worlds.md`, sections 3, 5 and 6. Each
check is a referee expectation, which observes only what the fakes saw,
the facts the engine and the worker emit and the durable records
(`scenarios.md`, 1.2); a harness invariant, which reads a component's frozen
state through its inspection adapter (skein's `testing/inspection.md`);
or a settings test, outside every world. Each returns its finding, naming
the object, the moment and the seed. The settings tests are
`settings-tests.md`'s, and the check of every commit's store against a
start is `store-check.md`'s.

### 6.1 The inspection adapters

Each is a kit of its component's tree, of skein's form (skein's
`testing/inspection.md`, 2): one function from a borrow of the component
to the records of section 3 there, over the component's `pub` reads, each
private table gaining a `pub` read documented as its adapter's. Every
adapter names a waiter `task N`, by its task's number, so that a
landing's or a change's wait and a task's resolve across inspections.

- **The engine's,** `temper/testing/temper-engine-inspect`, in full in
  `engine-checks.md`, 2:
  `temper_engine_inspect::inspect(root: &temper_engine_domain::Domain,
  into: &mut Inspection)`, component `engine`, and
  `temper_engine_inspect::forge(forge: &temper_engine_domain_forge::Domain,
  into: &mut Inspection)` for the forge world, which drives the forge
  component alone.
  - **Waits:** each live landing in its named wait
    (`../domain/landing-waits.md`, 2), and each change in its
    procedure's wait, each kind in its design's words, with the bound
    its design gives.
  - **Tables,** as the draft's 3.2 names them: forge write entries;
    forge read admissions; branch holds and landing rows with their asks.
    Build places are the worker's.
  - **The tasks the root holds** are jig's adapter's
    (`jig/docs/design/domain/testing.md`, 7), over `Domain::tasks()`:
    an engine world runs both, and checks the two inspections together,
    once jig's is built (`engine-checks.md`, 7).
  - **Called** through the engine's world kit (`engine-checks.md`, 3):
    at the direct driver's fence once it has drained its immediate work
    (`temper/tests/engine/domain/src/direct.rs`), at the quiet points of
    the walking, escalation, roles and forge-route worlds, of the forge
    world and of the composed system world.
- **The worker's,** `temper/testing/temper-worker-inspect`:
  `temper_worker_inspect::inspect(domain: &temper_worker_domain::Domain,
  limits: &temper_worker_domain::Limits, disk:
  &skein_fake_checkout::Checkout, into: &mut Inspection)`, component
  `worker`: the domain's workspaces and the world's fake disk, the limits
  the world steps it with giving the tables' capacities,
  with a function for each part a world holds alone, the checkout child,
  the fake disk and the git component's trash (`worker.md`, 2).
  - **Waits:** a prepare queued for a permit or waiting for another's
    store, and an operation in flight, a workspace's fetch among them. A
    check waiting for a build place is the domain world's fake builder's,
    inspected beside the worker, since build places are the deployment's
    (`../domain/worker/agent-processes.md`, 1).
  - **Tables:** workspaces and per-run scratch, released by the run's end
    or the workspace's release; prepare permits; the Git stores' packs,
    released by the packing their own counts trigger; git's locks; run
    slots; and trash, released by the sweep. Build places are the fake
    builder's table.
  - **Called** where the checkout world checks its bounds each
    iteration, before its domain reclaims
    (`temper/tests/worker/checkout/src/world.rs`), at the worker
    domain world's quiet point, and in the git protocol world after each
    operation (`worker.md`, 4).
- **smith's and jig's** are their trees' (`smith/docs/design/testing.md`,
  6; `jig/docs/design/domain/testing.md`, 7); the system world calls
  each beside the engine's and the worker's.

### 6.2 Faults, streams and sweeps

- **The fault ledger** (skein's `testing/sweeps.md`, 3): a world that
  injects faults keeps one per run. The world records its own (a party's
  action, a restart, a worker drained or killed, a store's fault) as it
  injects them, and copies the fake forge's and fake LLM's from each
  fake's own report of what fell. Its identities: the fake LLM's account,
  the fake forge's runner and gate run, the candidate's and base's
  commits, and the request.
- **The failure table** of the draft's 3.3, each failure class's
  expected transition on each counter, is held by temper's 3.3 check,
  beside the gates' (`blame.md`) and the runs' blame. For runs it has two halves:
  smith's worlds hold the retries and the run's end
  (`smith/docs/design/faults.md`, 4), since smith names no temper and
  its worlds run the runs; the system world holds the task tries and
  the holds, and calls smith's half beside them (6.3).
- **Streams** (skein's `testing/sweeps.md`, 2): a world that sweeps
  seeds its workload (goals, items, trees and scripts) from `Workload`;
  each injector from its own part of `Faults` (`forge`, `llm`, `worker`,
  `store`, `restarts`); each party from its own part of `Parties`; and
  its simulated world, latencies and delivery order from `Scheduling`.
- **Sweeps and exclusions** (skein's `testing/sweeps.md`, 4 and 5): each
  sweep configures the faults it draws, and its only exclusions name the
  ignored scenarios of the draft's section 7. A condition an exclusion
  names is read through the adapters of 6.1, or from the world's own
  state, at the moment the fault would be injected.

### 6.3 Blame for runs, and an account's faults, in the system world

Written from `../draft/failure-worlds.md`, 3.3 for runs, 3.4, 5 and
the catalog's P26, S1 and P126 rows. The faults are skein's fake LLM's account strikes
(skein's `testing/fakes/llm.md`, 2.7); smith's worlds hold the rest of
the providers' faults (`smith/docs/design/faults.md`).

- **Two accounts:** the composed system world (`tiers.md`, 1.1) can
  give its fake LLM two accounts, each with a key source, each of its own
  party, so that each party's goal runs on its own account. The queries
  its agents make carry their account's credential index (skein's
  `testing/fakes/llm.md`, 2.7), which the agent world's kit fills from
  the run's account. Its one account with two sources
  (`World::two_source_chat`) stays for the stories that use it.
- **The account case,** `an_account_fault_holds_only_its_goals`, a fuzzy
  test (`tests/suite/fuzzy/accounts.rs`). Each seed draws from `Faults`,
  part `llm`, one of spent credits, an expired sign-in and a rate window,
  the account it strikes, the step of the struck goal's run at which it
  falls, and whether it is cleared at a later step; each party runs a
  goal; the world records each fault in its ledger, and holds:
  - **3.3 for runs:** no task try is spent for the fault:
    `Counter::TaskTries`, counted from jig's failed attempts
    (`jig/docs/design/domain/tasks/lifecycle.md`, 6) of the struck
    account's tasks within the fault's interval, stays at zero; spent
    credits or an expired sign-in that is never cleared holds the struck
    goal's tasks with words naming the account
    (`jig/docs/design/domain/accounts.md`, 3.5); and smith's
    half, by `smith_faults::Runs::judge` over the runs' attempts and ends
    (`smith/docs/design/faults.md`, 4).
  - **3.4:** the other account's goal lands as if the fault weren't
    there, within its bound; the struck goal ends within its bound past
    the clear, or, spent credits or a sign-in never cleared, is held
    naming the account. A rate window, cleared or not, lets each window's
    first prompt through, and it is shorter than the first backoff, so a
    retry meets a fresh window: the struck goal ends within the same
    bound, whether or not the window is ever cleared.
  - **Each fault it configures fell** (`Sweep::check_fell`), over the
    seeds its budget allows.

  Its exclusions name only the ignored scenarios below, by their narrow
  conditions, where the case meets their combinations. A rate window is
  drawn shorter than the session's shortest first backoff: a longer one
  is P127's combination, whose scenario, `a_rate_window_is_waited_out`,
  is smith's, and an exclusion names a scenario of its own tree
  (skein's `testing/sweeps.md`, 5), so smith's sweep excludes it there.
- **The scenarios,** in `tests/suite/accounts.rs`; the open ones are
  `#[ignore = "provider failures: ..."]`:
  - `a_limited_account_frees_its_tasks` (P26): two accounts of one
    holder; the one its tasks use is limited with no reset; its tasks
    choose again on the other, and their goal lands;
  - `spent_credits_stop_placement` (S1): `insufficient_quota` on an
    account marks it, and no run is placed on it afterwards, while the
    other account's go on; it passes, and is enabled, an active
    regression;
  - `an_unserved_model_moves_the_task` (P126): a model refused for an
    account narrows that account's models, and the task chooses again
    and runs.

  The draft's rows name smith's session world for S1 and P126: their
  assertions are jig's placement and an account's models, which only
  this world runs with smith's agents, so their scenarios are here. A
  scenario that passes today shows no open problem: it stays enabled, as
  an active regression, and the draft's row moves to its held list, as
  S1's has.
