# The service's crates

Provisional, 2026-11-12, from `../draft/cli.md`, section 6 and
`../draft/deployment-binaries.md`, section 9's second item. The service
of `README.md`, sections 4 to 6, is four crates: what both halves share,
the engine's, the worker's, and the one process that composes them. The
engine's binary links the engine's alone, the worker's the worker's
alone, and the box and the tests the composition (`binaries.md`, 1 and
4). No Cargo feature chooses either half (`../draft/cli.md`, 3.2).
Section 2 is the crates, 3 to 6 each one, 7 the shells that drive them,
8 the tests, 9 the checks.

## 1. In one page

- **Four crates,** each step code under the subset, `no_std` but in its
  tests, as `temper-service` is today: `temper-service-core`,
  `temper-engine-service`, `temper-worker-service` and `temper-service`
  (section 2).
- **The core names neither half:** the vocabulary both halves speak and
  the process's io they share (section 3).
- **Each half is a component and a service alone:** the component, which
  the composition drives on a shared process, and a service of its own
  that owns one process and drives its component alone, with its own
  start, notes, stops and end (sections 4 and 5). Neither half names the
  other's crates, nor the composition.
- **The composition is today's one process** (`README.md`, section 6):
  both components on one process, joined by the in-process name
  `workers`, with the standalone start's paired checks and the
  supervision that starts, drains and stops them together (section 6).
  It keeps the name `temper-service`, so the box, the playground and the
  worlds that run one process keep theirs.
- **Nothing a person or a program sees changes:** the documents read,
  the refusals, the lines printed (`cluster.md`, 3), the exits, the
  store, the channel and the clients' wire are as they are; the crates
  move code, they change no behaviour.
- **The shells follow** (section 7): `temper-service-shell`'s run and host
  drive any of the three services and format no half's notes; each
  binary's shell formats its own; the box's composes the standalone
  start, which `temper-engine-shell` no longer holds.

## 2. The crates

| Crate | Holds | Its manifest names none of |
|---|---|---|
| `temper-service-core` | section 3: the shared vocabulary, the process, the shape | the engine's crates (`temper-engine-*`, `temper-engine-smith`, `temper-box-machine`, `temper-forge-setup`, `temper-protocol-records-forge`, `temper-protocol-store`, `skein-forge`, `skein-forge-setup`, `skein-forge-connection`, `skein-forge-client`, `jig-protocol-engine-clients`, `jig-protocol-engine-clients-http`, `jig-protocol-engine-workers`, `jig-protocol-accounts`, `jig-protocol-store`) and the worker's (`temper-worker-*`, `jig-protocol-worker-link`, `jig-protocol-worker-agents`, `smith-host-protocol`, `smith-host-domain`, `smith-channel`, `smith-protocol-channel`, `smith-protocol-program`) |
| `temper-engine-service` | section 4: the engine's half and the engine alone; the directory `crates/temper-service` renamed, with today's modules but the worker's | the worker's crates, `temper-worker-service`, `temper-service` |
| `temper-worker-service` | section 5: the worker's half and the worker alone | the engine's crates, `temper-engine-service`, `temper-service` |
| `temper-service` | section 6: the composition, at a new `crates/temper-service` | nothing it may not: it depends on the other three |

- **jig's configuration,** `jig-protocol-config`, which both halves and
  the core read, brings jig's own crates beneath it; that cut is jig's
  (section 10), and no row of `binaries.md`, 4 excludes them.
- **A shared item lives in the core** only when both halves need it and
  it names neither half. What one half alone uses stays in that half,
  even where today's module holds both.
- **A value both halves must agree on** and that a half's own crate
  defines, such as the close job's kind (`temper_engine_domain_forge::CLOSE_KIND`)
  or a documentation answer's bound (`temper_engine_smith::DOC_ANSWER`),
  is mirrored in the crate that needs it without the other, and a test
  in a crate that links both holds the mirror equal to its source: a
  domain crate never depends on a service crate.
- **Readers depend on what they use:** a crate or world that uses one
  half's items alone depends on that half's crate; one that runs one
  process depends on `temper-service`, which re-exports what its readers
  use of the other three under the paths they use today.

## 3. `temper-service-core`

- **The process:** one process's io (`skein_io::Io` and its limits), the
  kernel's routes and its queues of completions and submissions, the
  events told, and the termination signals' descriptor, with the
  mechanical steps of a turn that today's `system.rs` runs whatever the
  shape: completions in, io's events up, its requests down, submissions
  published, completions reclaimed. Each of the three services owns
  one, and the composition's two components share its one. What a step
  does with an event is a component's: the core routes nothing by
  meaning.
- **The vocabulary both halves speak:**
  - `tokens`, its numbers and subranges as they are (the accounts' and
    the worker's share nibble 3, each its own subrange);
  - `long_pass`, the stages (`Stage`, `STAGES`) and the wait check's
    shared parts;
  - the opening refusal, `Refusal` (`Peer`, `Terms`, `Malformed`);
  - the shared limits: the checkout's and the git component's envelopes
    (`Checkout`, `GitComponent`), the tooling's and jobs' bounds, a
    grant's sizing (`SOURCE_NAME_BYTES`, `grant_bytes`,
    `token_credential_bytes`), and the peer envelope, the three scalars
    a worker needs of its engine's limits: the forges' count, the
    installations' and a forge token's bytes;
  - the grammar of a document's `limits` member that both halves read
    (`configuration/limits.md`): `Profile`, the shared parts' values in
    each profile (the checkout's, the git component's and the peer
    envelope's, which the engine's profile takes from here), their
    override names and how each is applied, and the name of every
    override a document may carry, so that a worker alone refuses an
    unknown name as an engine does; a test in `temper-service` holds
    that list equal to the engine's table of names;
  - the shared settings: `Files`, `Program`, `Adapter` and
    `PROGRAM_PATH_BYTES`;
  - the agent's declaration: what the engine's half and the paired
    checks need of a worker's agent, as scalars (its allowances, its
    runs' bounds, its programs' names and its endpoints' names), which
    the worker's `Agent` gives (5) and the engine's half takes, so that
    the engine's half never names smith's host;
  - the document's shape (`cluster.md`, 1): `Shape` (`Standalone`,
    `EngineAlone`, `WorkerAlone`) and `nearest`, read from the
    document's members alone;
  - the startup problems' container and its lines (`Problems`, `lines`),
    the paths' helpers and the problems either half may raise; each half
    adds its own (4, 5), and the order and the bounded count of the
    lines printed stay as they are.

## 4. The engine's service

`temper-engine-service`: today's `temper-service` but its worker's
modules, under the engine's name, its history kept by the move.

- **The component,** `Engine`: the engine's root, its protocol children
  (the store, the accounts, the clients', the forge's, the setup's and
  the workers' listener) and its routing, driven through a seam of
  plain methods the composition calls in the turn's order: its clocks,
  an event given, a completion given or handed back, its iteration, its
  requests and submissions taken down onto the process, whether it is
  ready for a local worker, draining or ended, its notes, its deadlines
  and its diagnostics. Its fields stay its own.
- **The engine alone,** `Service`: one process and the component, with
  today's constructor of an engine alone (`Service::engine`), its
  `iterate` and `iterate_stage`, the kernel's queues, its deadlines, its
  listeners, handover and DNS, `shutdown`, `notes`, `end`, `is_empty`
  and `operations`. Its start is today's engine's (`startup::engine`,
  `named` and their kin), refusing any other shape as
  `binaries.md`, 2 says.
- **Its vocabulary:** its own `End`, `Stop` and `Note`, holding what the
  engine emits: `Stop`'s journal, commit, start, room, restore, forge
  fit, binding, store, listener and webhooks endings; `Note`'s core,
  forge, setup, runs, clients' and remote workers' notes. A note about a
  worker that the engine's workers' listener raises (`WorkerLost`,
  `WorkerUnsent`, `WorkerOpeningEnded`, `WorkerListenerFailed`) is the
  engine's. With them: `Listening`, the setup's and the handover's
  types, `NotConnected`, `Unsettled`, the restore's and the jobs'
  diagnostics.
- **Its limits and settings** are the engine's aggregate on the core's
  shared parts: the profile, its overrides and the reference documents
  (`limits::reference`, the box's limits among them). Its checks of a
  local agent read the agent's declaration (3).

## 5. The worker's service

`temper-worker-service`: the worker's half, today `worker`,
`worker_git`, `worker_route`, `worker_settings`, `identities`,
`program_sources`, `startup/worker.rs`, the worker's limits and checks
and their tests.

- **The component,** `Worker`, as today: its root, its link, the program
  sources, the git component and the jobs, driven by the composition
  through its seam (an event offered, a completion given, its
  iteration, its requests and submissions, its stop and abort, whether
  its agents ended, its refusal, its notes and deadlines).
- **The worker alone,** `Service`: one process and the component, with
  today's start of a worker alone (`startup::worker_part`,
  `startup::worker_alone`, `NamedWorker`, `WorkerPart`), its `iterate`
  and `iterate_stage`, the kernel's queues, its deadlines, its signals,
  `shutdown`, `notes`, `end`, `is_empty` and `operations`. Its start
  refuses any other shape as `binaries.md`, 5 says.
- **Its vocabulary:** its own `End`, `Stop` and `Note` (`WorkerJob`,
  `Linked`, `Lost`, `Refused`), its loss and refusal reasons, and a job's
  ending of its own, in jig's host's words, where it builds the engine's
  `JobDiagnosticEnding` today; the line printed for it is unchanged.
- **Its settings and limits:** `settings::Worker` and `Agent`, smith's
  host profile, which gives the agent's declaration (3); the worker's
  aggregate on the core's shared parts and the peer envelope, never
  the engine's profile. A worker alone reads them from its document's
  `limits` with the core's grammar (3): the profile's shared parts and
  the overrides of the names it holds; another known name's value it
  takes unread, since a worker alone checks only its side
  (`configuration/limits.md`, *In the cluster's shapes*), and that
  name's checks are an engine's. In the composition, the engine's
  aggregate gives the worker's parts. `job_kinds`, on
  its mirror of the close job's kind (2). A conversion from the core's
  limits to the worker's domain's is a function of this crate.

## 6. The standalone service

`temper-service`, the composition: one process and both components,
joined by the in-process name `workers`, the engine listening and the
worker dialling it through the same process, as `README.md`, section 6
says. Nothing crosses between them but bytes.

- **One turn, today's order:** the process's completions and events,
  the routing, the engine's iteration, the worker's, then the
  submissions, publication and reclaim; a completion goes to the
  component whose request it answers, the store's and the worker's
  files' as today.
- **What only the composition decides:** the worker starts once the
  engine's store, its workers' listener and its clients are open; the
  worker's agents ending is told to the engine's drain; a drained
  engine stops the worker, a second signal aborts it, and a worker's
  start refused ends the service as the worker's failure without
  replacing the engine's end; the worker's notes join the engine's in
  one drain, its order and bound kept.
- **Its start:** the standalone document's and the dispatch of every
  shape with its full checks (`startup`, `startup_with`, `check`,
  `with_setup`, `with_setup_given`, `shape`): the paired checks
  (credentials and grants, the repository and the channel, the agent and
  the endpoints, turns and the checkout, the sockets, and the process's
  io, its policy as today), the worker built and handed to the engine's
  start.
- **Its vocabulary:** `Service`, `iterate`, `iterate_stage`, and `End`,
  `Stop` and `Note`, each joining the two halves' (`Stop`'s and
  `Note`'s variants an engine's or a worker's, wrapped), and the
  diagnostics' joined wait; with the re-exports of section 2.
- **Its mirrors' tests:** that the worker's close kind and the core's
  documentation bound equal their sources (2).

## 7. The shells

- **`temper-service-shell` names no half:** serve's run (the inherited
  listeners, the configuration, the kernel and `drive`) and its host
  drive a service given as a type of the binary's, through a table of
  plain functions the binary's shell fills, or a trait on a type of
  that shell's own: an iteration of a stage, the kernel's queues, the
  deadlines and work, `shutdown`, the drain of its notes into lines,
  and its ending's words and exit. The files a document names,
  `jig_directories`, the local link file, the long pass's lines, the
  wait check, the startup problems' lines and a listener's refusal stay
  in it. It depends on the core, never on a half or the composition;
  shells are ordinary Rust (`skein/docs/foundation/programming-model.md`,
  10), and no step crate gains a trait or a function table.
- **Each half's lines are its binary's shell's:** the engine's notes,
  stops and endings, its forge's DNS, the handover, the restore's and
  the setup's words, `temper-engine-shell`'s; the worker's notes and
  endings, `temper-worker-shell`'s. Every line keeps its words, its
  prefix (`temper-engine serve: …`, `temper-worker serve: …`,
  `temper-box serve: …`) and its order.
- **The standalone start is the box's:** `temper-engine-shell` starts an
  engine alone, and gives the engine's parts of a start (its settings,
  runtimes, bundle and checks) as functions; `temper-worker-shell` gives
  the worker's (the agent's preflight on the worker's naming,
  `NamedWorker`, the root, the program sources' resources and the
  launcher's links), never configuring a service it doesn't own;
  `temper-box-shell` composes them into the standalone start and its
  service, with the box's pipes and first secrets. Today's
  `serve::WorkerHalf` and `WorkerStart` go with it.

## 8. Tests

- **Each test goes with what it exercises:** a half's own tests to its
  crate, the paired and supervision tests (`system`, the paired startup
  checks, link room, the cross-half translations, workspace reuse's
  forwarding) to `temper-service`, the shared arithmetic and shape cases
  to the core. A test that mixes moves in parts.
- **A worker's test that needs an engine's assignment** takes it from a
  fixture in the worker crate's dev-dependencies, an engine test kit or
  the engine's crate itself, or builds the wire bytes itself; the
  worker's normal dependencies stay free of the engine's.
- **What every test checks stays as it is:** no assertion is weakened,
  and the lines' tests in the shells pin each line's words.

## 9. The checks

- **The closures** (`binaries.md`, 4): `temper-engine`'s row excludes the
  worker's crates and `temper-service`; `temper-worker`'s excludes the
  engine's crates, `temper-service`, `temper-forge-setup` and
  `temper-protocol-records-forge`, with no exception left.
- **No feature** (`../draft/cli.md`, 3.8's check 3): none of the four
  declares `[features]`, and no manifest names one of them with
  `features`, `default-features` or `optional`.
- **Sizes:** the change that removes the `worker` feature reports
  `temper-engine`'s, `temper-worker`'s and `temper-box`'s sizes, each
  built alone in the release profile, before and after
  (`../draft/cli.md`, 3.2).

## 10. Later

- **jig's configuration's closure:** `jig-protocol-config` links jig's
  engine core, its clients' parts and its store, so the worker's binary
  links them through it; a configuration crate without them is jig's to
  cut, in jig's round.
