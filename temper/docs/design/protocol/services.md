# The service's crates

Provisional, 2026-11-13, from `../draft/cli.md`, section 6 and
`../draft/deployment-binaries.md`, section 9's second item; the
engine's half's pieces from `../draft/crate-graph.md`, 5. The service
of `README.md`, sections 4 to 6, is four crates: what both halves share,
the engine's, the worker's, and the one process that composes them. The
engine's binary links the engine's alone, the worker's the worker's
alone, and the box and the tests the composition (`binaries.md`, 1 and
4). No Cargo feature chooses either half (`../draft/cli.md`, 3.2).
Section 2 is the crates, 3 to 6 each one, with the engine's pieces in
4.1 to 4.4, 7 the shells that drive them, 8 the tests, 9 the checks.

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
- **The engine's half is five crates** (section 4.1): its start, which
  keeps the name `temper-engine-service`, over the component and the
  startup, which compile side by side over a vocabulary both name, with
  the stock beneath the startup.
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
| `temper-engine-service` | section 4: the engine's half and the engine alone; its start, and every item of its pieces under today's paths: `temper-engine-service-component`, `temper-engine-service-startup`, `temper-engine-service-stock` and `temper-engine-service-vocabulary` (4.1), each under this row's rule | the worker's crates, `temper-worker-service`, `temper-service` |
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

### 4.1 Its crates

From `../draft/crate-graph.md`, 5, milestone G4 of its execution file.
The engine's half is five step crates in `temper/crates/`, cut by that
draft's rules (5.2) so that the parts which change apart compile side
by side:

```
temper-engine-service                 the engine's start: a document read and checked by the startup, then built by the component into the engine alone; and every piece's items under today's paths
├── temper-engine-service-component   the component, Engine, and the engine alone, Service: the children, the routing, the seam, the notes, stops and endings
├── temper-engine-service-startup     a document read and checked into the engine's settings, or refused with its problems
├── temper-engine-service-stock       the binary's stock settings: its charters and their words, ways, kinds, models and floors
└── temper-engine-service-vocabulary  what the component and the startup both name, and what readers outside need without the component: settings, limits, the store's sizing and records, the forges' and the tokens' tables, the translations at the clients' and the workers' edges
```

- **Its edges:** the vocabulary and the stock name no other piece; the
  startup names the vocabulary and the stock; the component names the
  vocabulary alone; `temper-engine-service` names all four. So the
  component and the startup compile side by side, and neither waits on
  the other.
- **Why there.** Over the three weeks to main `68382158be`, 279
  landings changed four of the crate's modules or fewer, a module's own
  tests counted with it. The stock changed in 109 of them, mostly alone
  (its charters' words), the startup in 67, the limits in 62 and the
  family in 40; then the setup (36), the engine (28), the clients and the
  system (25 each) and the route (23). The pairs most often together were
  the limits with the stock (30) or with the startup (21), and the startup
  with the stock (18). So:
  - **side by side:** the startup reads a document into the settings and
    the component runs from them; they share only the settings and what
    both name. A change to the startup rebuilds neither the component's
    library nor what is built on it alone, and a change to the component
    rebuilds nothing of the startup;
  - **the stock alone:** it names no module of the engine's half, and the
    startup alone names it; the component takes what it needs of the
    stock through the settings (4.2). A change to its words rebuilds the
    stock, the startup and the start, not the component's library;
  - **stable below:** the vocabulary holds what both sides name. The
    limits among it change often, and every piece rebuilds with them; no
    cut avoids that, since reading a document and running from it both
    take the limits;
  - **the component is not cut further:** `Engine` is the state its
    children's modules take whole (the clients', the setup's, the
    forge's, the route's and the workers' functions take `&mut Engine`,
    and four files add its methods), and those modules form one cycle
    through it: a state machine and what lies around it
    (`../draft/crate-graph.md`, 5.2);
  - **the start keeps the name:** `restoring_with`, `restoring_prepared`
    and `with_setup_given` read a document through the startup and build
    the engine alone through the component, so they sit above both, in
    `temper-engine-service`. It keeps that code of its own, and isn't a
    facade that goes once no caller names it.
- **Sizes** (`tools/crates/design.md`, 2.3), about: the component 14,500
  lines and 45,000 with its own tests, the startup 5,200, the vocabulary
  9,000, the stock 1,200. Each is under the library bound, so the size
  check excepts none of them.

### 4.2 What each holds

- **The vocabulary:** an item goes down when the component and the
  startup both name it, or when a reader outside the engine's crates
  needs it without the component (section 2's readers: the worlds and
  the testing kits), with every item that its definition, its inherent
  methods and its trait impls reach. None of it names an item of another
  piece. Today that is:
  - whole modules: `limits` (with `names` and `reference`), `settings`,
    `values`, `translate`, `assignment`, `tools`, `family`, and
    `clients::translation`, with the `clients::moved_bytes` that the
    limits read;
  - the forges' table, `forges::Forges` and its iterator; the tokens'
    table, `forge::tokens::Tokens`, and `forge::tokens::account`; the
    forge's configuration, which the startup and the limits name:
    `forge::ForgeRuntime`, `config`, `prepare` and `small`;
  - the system's configuration helpers that the startup names:
    `system::GITHUB_REMOTE`, `machine_config` and `playground_config`;
  - the store's sizing, constants and records: `store` but `Store`
    itself and its methods (`store::limits`, `WIRES`, `PATCH_SETS`,
    `route`, `ranges`, `unreadable`);
  - the startup's items that the component names: the application's
    settings types (`startup::application::Clients`, `Origin` with its
    methods and `Scheme`, `startup::application::sign_in::Member` and
    `ForgeSignIn`), `application::forge_origin`, `sign_in::visible`,
    `startup::sign_in_url`, `startup::admission_words` and
    `startup::checks::membership`.
- **The stock:** `stock`, whole, its words' files among it.
- **The startup:** `startup` but the vocabulary's items and the start:
  the document's reading, its members, checks, problems and references,
  the checkout's capacity, the application's parsing, `engine`, `named`,
  `shape`, `check`, `checked_start_with`, `prepare_forges`,
  `store_limits` and their kin.
- **The component:** `engine` with its diagnostics, `runtime`, `opened`,
  `system` (`Service` and its kin), `clients` but `translation`, `setup`,
  the rest of `forge` (its layer, `gone`, `origins`, `records` and the
  tokens' events), the forges' functions that take the engine, `route`,
  `workers`, `accounts`, `stop`, `upgrade`, `core_wait`, `start_refusal`,
  `diagnostics`, `store::Store`, and the root's `End`, `Stop`, `Note`,
  `Listening`, the setup's and the handover's types.
- **`temper-engine-service`:** the start (4.1) and the re-exports.
- **The settings carry what the component took from the stock:** the
  core's kinds, `stock::kinds` of the start's limits and its first
  connector, and the period's span, `stock::PERIOD_SPAN`. The startup
  fills them as it fills `kinds_stock` and `ways_stock` today, and the
  component reads them where it called the stock.

Every piece holds to these:

- **One module tree.** An item keeps its module path in each piece that
  holds or re-exports it: `temper_engine_service_vocabulary::limits::Profile`
  is `temper_engine_service::limits::Profile`. A module split between
  pieces (`startup`, `store`, `forge`, `forges`, `system`, `clients`) is
  a module in each, and the upper one re-exports the lower ones' whole.
  `temper-engine-service`'s root re-exports each piece's root. So a
  caller builds unchanged through `temper_engine_service`'s paths, and a
  moved file's `crate::` paths resolve in its new crate through its
  root's re-exports.
- **Interfaces:** an item a piece reaches across the cut is public in
  the piece that holds it, documented as the interface it is. A
  `cfg(test)` hook that another piece's tests read becomes plain code, or
  those tests go to the hook's piece.
- **Step crates** in the strict subset (`AGENTS.md`, 2), `no_std` but in
  their tests, each with `temper-engine-service`'s lint profile
  (`tools/lints/design.md`), none with features
  (`../draft/cli.md`, 3.2), and each manifest under section 2's row for
  the engine's half: none names a worker's crate, `temper-worker-service`
  or `temper-service`.

### 4.3 Its tests

- **Each test goes with what it exercises,** to the lowest piece that
  holds everything it uses, through that piece's dependencies and
  dev-dependencies: the store's routes to the vocabulary, the stock's
  own checks to the stock where they build there, the startup's reading
  to the startup, the component's to the component. A test of the start
  (4.1) goes to `temper-engine-service`'s tests, or builds its engine
  from the startup's checked start and `Service::prepared` itself.
- **No piece's tests use a crate that depends on that piece,** which
  would give them two copies of its types; so only
  `temper-engine-service`'s own tests use it.
- **The component's tests may use the startup and the stock,** to read a
  document and take the stock's settings as today's fixtures do (section
  10).
- **Fixtures:** each piece's tests keep a private copy of the fixtures
  they use, trimmed to them, as the worker service's do;
  `.config/fixture-readers.toml` names each copy and its readers. A test
  file that exercises two pieces in parts splits, each assertion kept.
- **The tiers' filters follow** the tests they select: the fuzzy tier's
  `tests::sign_in::fuzzy_` cases go with `sign_in`'s tests to their piece
  (`.config/nextest.toml`).
- **What each test checks stays** (section 8).

### 4.4 Landing the split

As `../draft/crate-graph.md`, 5.3 has it:

- **Preparation first,** each change behaviour unchanged and landing
  before the move, so that every file goes whole to one piece: the
  component's files that hold vocabulary items split from them (the
  forge's configuration, the forges' and the tokens' functions that take
  the engine, the system's helpers, the store's `Store`, `moved_bytes`,
  `opened`'s methods on `Store`, and the vocabulary's `cfg(test)` hooks);
  and the startup's (its items that the component names, its start, and
  the stock's kinds and span carried by the settings).
- **Then one change of moves:** each file moved whole into its piece,
  and not edited but where the build forces it (a `mod` line that names
  a file now in another piece becoming its re-export, a visibility, a
  path that `include_str!` or `include_bytes!` reads), in a commit after
  the moves, so that git's rename detection carries branches in flight
  across it. New files: the pieces' manifests and roots, the module
  tree's re-exports, the fixtures' copies; `temper-engine-service`'s root
  and manifest; the workspace's entries; the size check's exception
  removed; the tiers' and the fixture readers' entries.
- **Then the readers move:** a library whose own code needs only the
  vocabulary and the stock, and none of whose other normal dependencies
  reach `temper-engine-service`, depends on them instead
  (`temper-engine-domain-world`, `temper-client-world`,
  `temper-system-world`, `temper-github-app` among them), and its tests
  too where they need no more than those and the startup. Every other
  crate names `temper-engine-service`. So a change to the component
  rebuilds none of them.

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
  to the core. A test that mixes moves in parts. Within the engine's
  half, each test goes to its piece (4.3).
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
- **The component's tests off the startup and the stock** (4.3): its
  fixtures read documents through the startup and take the stock's
  settings, so a change to either rebuilds the component's test binary,
  though not its library. Fixtures that build the settings without them
  would rebuild the component's tests only with the component and the
  vocabulary.
- **The start with its callers:** were `restoring_with`,
  `restoring_prepared` and `with_setup_given` the shells' and the
  composition's (section 7), `temper-engine-service` would hold
  re-exports alone, and could go once its callers name the pieces.
- **The family a crate of its own,** beside the startup, if its history
  keeps it apart from the limits: a change to it then rebuilds neither
  the startup nor the rest of the vocabulary.
- **A check that holds the startup and the component side by side,**
  neither naming the other, beside `binaries.md`, 4's closures.
