# The real loop

`tests/real-loop`, on `skein-world`'s real-world harness
(skein/docs/foundation/testing-strategy.md, 2.8). One test process, one thread and one loop
drive the standalone service, its agents, its clients and its fake peers
over one real io_uring ring. This tier checks what only the real kernel
shows of temper's assembled stack, within its budgets (`checks.md`, 1),
after landing, in the real pairs (`tools/tiers/layers.md`, 9.2). Every
behaviour, path and corner case a lower world can show is that world's,
never here (`../draft/slow-tests-down.md`, 4).

## 1. The assembled service and agent processes

- **What runs:** `temper-service`'s system module, with the engine,
  store and one worker as standalone runs them; smith's agent hosts;
  the tools' root over jig's native client transport; and skein's fake
  LLM peer on loopback, latency zero. skein's fake issuer joins
  stories that sign in to an LLM provider, and skein's forge peer
  stories that need the forge. Configuration, the store and client
  credentials live in a scratch directory on the real filesystem.
  Listeners are on loopback, on a port the kernel assigns, passed to
  their clients once bound. A story never uses a port the kernel did
  not give it, but for a clients' listener at a fixed address, reserved
  of `temper_fixtures::ports` (`process-tier.md`) before the service
  starts, which one story finds already bound (below).
- **Agents share the loop.** The harness answers the worker's spawn
  with a hosted smith agent, joined by real pipes, and drives its
  `iterate` in the same loop. The agent's LLM is skein's fake peer,
  which plays the story's scripted turns, or, on a program's source,
  smith's fake `claude` (`fakes.md`, 1). git is the fake git program
  (`fakes.md`, 3); other external programs, such as sh, run as real child
  processes. Launching the `temper-box internal agent` executable, with its
  own loop and ring, is checked by the process tier (`process-tier.md`).
- **The hosted agent starts as the shipped one.** It reads the
  document the worker wrote as the shipped `temper-agent` reads it
  (`../protocol/binaries.md`, 3), and hands smith's shell every resource
  the shipped start hands it. Where the document configures a program,
  as the worker's always does, those include the program's shell
  resources (smith's `protocol/programs-claude-code.md`, 1.1): the relay
  is the agent executable the worker spawned, since the shipped
  process's relay is its own executable; its 32 bytes of entropy come
  from the world's seed; and the programs directory is checked and
  opened as the shipped start does it: opened when it is private,
  absent when it is missing, and the start refused when it is unsafe.
  The hosted agent is not a process of its own, so it names no parent
  for the program's child mode. A program it started would start
  directly, as where no parent-death signal exists, but no story
  here runs one: a program's run is the one-process world's (below).
  **A step test holds this start at the gate.** The world's library
  starts its hosted agent on the document the shipped worker writes
  for the tier's fixture, with no real file or descriptor, so smith's
  shell refuses it just as it would refuse the real loop's. A resource
  the shell starts to require then reddens the gate, not the real
  pairs after landing. The shell's refusals of a start are smith's to
  test (smith's `protocol/agent.md`, 8).

## 2. Shared stories and lower-world coverage

- **Reuse across worlds.** The host composition, fake state machines,
  scenario scripts and outside referee observations are shared with
  the one-process world wherever they apply. Backend setup supplies
  real descriptors, directories and time here. The real loop replays
  neither the simulator's schedule nor its crashes and power losses.
- **The stories,** one each, what only the real kernel shows: a
  sign-in begun from the terminal, the engine's redirect listener on a
  port the kernel assigned answering the browser's return
  (`suite::sign_in`, below); an agent's shell finding a program only on
  its worker's `PATH`, a real child shell under a worker alone
  (`suite::cluster`, below); a standalone service drained by its
  termination (`suite::lifecycle`); two acts on one DIRECTORY taking
  `temper ctl`'s lock in turn (`suite::lock`; ../client/ctl/testing.md,
  3); the scripted peer listening on a port the kernel assigned
  (`suite::peers`); the sample's shell script passing its checks for the
  greetings and failing for a broken hello (`suite::sample`); an agent
  ignoring its cancellation and its grace, killed and named
  (`suite::teardown`, below); and a service whose clients' listener is
  already bound, exiting before its link, the story failing once its
  caller has settled and its descriptors are released
  (`suite::world::teardown`). The case library's step and the fuzzy
  stories beside it, `suite::fuzzy::landing` among them, are those
  failure-worlds' F7 left here (`../draft/failure-worlds.md`), and its to
  move.
- **Their behaviours are the lower worlds'.** A person's words during
  a chat's turn, answered once in its next call, and a goal's design
  delivered straight to its goal branch are the system world's
  (`../domain/agent/README.md`, 7); `chat show`'s older page at an activity cursor is
  `tests/client`'s (../client/ctl/testing.md, 2); a stop while a
  program's turn runs is the one-process world's (`tiers.md`, 1.5);
  the welcome's pages past its Ready, its project made, To begin
  skipped and the empty Home and Work, are the joint world's
  (../client/testing.md); and the fake issuer's admission beside a
  browser's held sockets, and past its cap, is temper-fake's, on
  skein's simulator (`process-tier.md`, *The fake box keeps two walks*;
  `../protocol/setup/box-backends.md`, 1). Two goals' changes released
  together, forming one landing by `turn` at a `batch` of two and
  landing on one green gate, are the engine's domain world's
  (`../domain/landing.md`, 16).
- **Moved below** (`../draft/slow-tests-down.md`, 5.4), each former
  story's checks held by the lower tests named, none of them by
  failure-worlds' workloads (`../draft/failure-worlds.md`, 5), which
  may vary them once they land, since each is a pinned path whose
  oracle a random workload would not keep:
  - the shipped agent fitting Standard's page with its derived
    carriers: a step test of this world's library;
  - a second world in the same process numbering its tasks from one:
    the engine's domain world, two stores each numbering their first
    two tasks from one;
  - a run's live text and a chat's committed reply, read on the line
    terminal's pages: the client world (`tiers.md`, 1.1), beside the
    view's step tests and the system world's live text; the terminal
    signed in by the fresh link printed for a spent one, its DIRECTORY
    private to its user: the one-process world (`tiers.md`, 1.5);
  - `chat show` printing a reply only once its turn commits:
    `temper-client-shell-cli`'s step tests, beside the client world's
    activity cursor; a signed-in DIRECTORY private to its user, and two
    end waits open while `me` answers: the one-process world;
  - a chat's document rewritten whole after its person's follow-up: the
    system world, its turns and activity as `ctl` prints them
    `temper-client-tools`' step tests;
  - a repository adopted through `ctl`, and a hooked repository's
    webhook answered `204` by the engine's listener: the one-process
    world;
  - GitHub's stories: the one-process world's (`tiers.md`, 1.5), with
    a landing's halves in the forge's protocol world (one pull opened
    and merged by the App's bot at the checked head), the engine's
    domain world (two party reads of a landed change's one result), the
    system world (a chat's reply after hearing the landing), and
    `temper-client-tools`' and the TUI's tests (an ended change as JSON,
    the committed reply in its chat);
  - a model's request hung by its provider after a shell call: the
    one-process world; the worker's endpoint bounds beside its stock
    retries, a step test of `temper-worker-shell`;
  - the judge's forced overdue, and a busy world failing through it:
    this world's library's step tests, over its checker with scripted
    records and time;
  - the ladder's twins, B0's green light and B1's repair: the system
    world (`tiers.md`, 1.1);
  - a chat on a program's source landing its change and resuming its
    session: the one-process world;
  - a private file's key reaching a chat's model; a call whose answer
    was lost, asked again under its name from another DIRECTORY; a
    welcome's model check on a program's source; a service started
    again on its directory, keeping its chat's reply and its settings:
    the one-process world;
  - two one-slot workers each running a chat to its end: the system
    world; two workers alone taking their figures, linking once and
    draining: the one-process world; the figures line, a step test of
    `temper-worker-shell`;
  - a person's latch answered once, releasing the batch waiting on it:
    the system world; the turn-landing setup's red top: the forge
    connector's world (`tests/engine/forge`), its two reviewed changes
    behind one live latch the engine's domain world's;
  - a chat made after its home's adoption, delivering to its own run
    branch: the system world, ignored until a chat's `deliver` before
    its end is built (`../domain/agent/tools.md`).

## 3. Accounts and sign-in

- **An account first,** with area 5's tools (../client/ctl/areas.md):
  a story whose agents run adds a key the fake LLM peer accepts, from
  the client, before its first task, and waits for its account to be
  usable: a key needs no sign-in to drive (jig's `domain/accounts-sources.md`,
  1; ../client/ctl/testing.md, 3). The real loop and process tier (`process-tier.md`)
  use runtime key adoption. Lower domain worlds first keep temper's real
  first-start rows at receipt one, then preseed restored fixture keys at receipt two; configuration
  names providers only, and no world hands a record in; ChatGPT's
  account comes from a sign-in alone, on the fake issuer below the live
  tier (`process-tier.md`) and the real one in it (`live-tier.md`).
- **An LLM account's sign-in** (../protocol/credentials.md, section 7),
  scripted on the fake peers, never by chance: on skein's fake issuer,
  the engine's redirect listener on a port the kernel assigns, and the
  redirect driven by the test, as a browser would; the account usable,
  then a run whose credential the fake LLM peer observes as that
  account's, by its scheme, never its value; begun from the terminal's
  Accounts (`suite::sign_in`), the issuer's admission beside a
  browser's held sockets being temper-fake's (above). After it, no key,
  token or record's value in the engine's store, a fact, a request, an
  answer, a trace, the client's small store or a printed page
  (../protocol/credentials.md, section 7). A program's source, declared
  and verified and run through smith's program backend, is the
  one-process world's (`tiers.md`, 1.5).
- **The accounts' cases are tested below,** each in the world that owns
  it:
  - **a sign-in refused, and one past its deadline,** each settled with
    no account usable: `tests/client`, its answer scripted
    (../client/ctl/testing.md, 2); the issuer's side in skein's OAuth
    worlds (skein's `oauth/README.md`, section 3, and `skein/docs/design/oauth/accounts.md`, section 8) and jig's protocol
    world (jig's `protocol/llm.md`, section 8);
  - **a turn limited (exhausted with a reset and without), refused or
    unauthorized,** and the account's state following (jig's
    `domain/accounts-sources.md`, 2): jig's protocol world, "failures of each
    class" and "a source limited and resumed" (jig's `protocol/llm.md`,
    section 8), and jig's accounts world (jig's `domain/testing.md`,
    section 3); the next task waiting for its account until the reset
    passes (jig's `domain/authority-checks.md`, 1.3): jig's core world (jig's
    `domain/testing.md`, section 4); in temper, the limited notice, the
    next task waiting with `task show TASK` saying why, then `account
    retry` and the run started: `tests/client` (../client/ctl/testing.md,
    2);
  - **a fenced attempt's notice dropped:** jig's fleet world, "Attempts
    are fenced" (jig's `domain/engine/fleet.md`; jig's
    `domain/testing.md`, section 3);
  - **a program source's usage limit and its lost login** (smith's
    `protocol/programs-claude-code.md`, 1.6): smith's world over its fake `claude`
    (smith's `protocol/programs.md`, section 8), relayed as limited and
    as unauthorized in jig's channel world (jig's `protocol/channel/testing.md`,
    section 1); `account verify` answered as each host reports:
    `tests/client` (../client/ctl/testing.md, 2).

## 4. Iteration checks and shutdown

- **Checks between iterations** are the strategy's for the real loop
  (skein/docs/foundation/testing-strategy.md, 2.8 and section 6), with the lower worlds'
  outside expectations; real deadlines bound waits, and the fake peers
  stay available until their clients have settled. A story's patience is
  20 s on Linux and twice that on macOS, the slower host, through skein's
  `real::host_bound` (`skein/docs/design/testing/service-tiers.md`, 1); the
  judge's early diagnostic stays one second before it, and every poll,
  wait or link deadline a story sets beyond or within the patience is
  derived from it, so both move together. Each check returns
  its finding (`scenarios.md`, 1.2): the judge's `watch` returns a broken
  check, and an overdue deadline seen in its `act` with the early
  diagnostic, the report and the scheduler figures, kept for that
  `watch`; a call answered past its bound, or a CLI timeout before it,
  is a finding with its recorded figures, beside the assertion a story
  calls at its top, and a starved wait still passes as starved; a
  service that exits before its link fails the story through
  `real::World::try_run` after the caller has settled, its receipts and
  descriptors checked on that ordinary return. The judge's own
  decisions, forced and busy, are its library's step tests (above).
- **A story ends what it starts.** Its last call returns only once every
  activation the story started or woke has ended: a chat whose reply it
  read, a change it landed, a delegate it waited on, joined by `task
  show TASK --until ended` where a reply alone leaves a run closing. When
  that call returns, before the service is told to stop, the judge
  checks the story's completion obligations against the engine's and the
  worker's state at that instant: no task has an attempt due,
  preparing, claimed or running, none is closing, no assignment is
  pending, and no agent or git child of the worker's is alive. A task
  parked between runs is no obligation. A story that means to leave work
  in flight declares each leftover, by its task and the states it may be
  left in, with its reason (the bounds story its live delegates); an
  undeclared obligation fails the story, and so does a declared leftover
  whose task has ended.
  Teardown's own check stays beside it: no child is killed but those a
  story asks for, no more strict and no less. A story asks for a kill by
  a `Kill` step, and by a cancel the kill follows (the next item): every
  kill is a `Signal(9)`, and those no cancel accounts for number exactly
  the story's `Kill` steps.

## 5. Cancellation and kill causes

- **A cancel's kill is asked for.** git kills its child when a cancel
  wins the race with the child's end (`../protocol/git.md`, 7), and
  whether the child still runs then is timing; so a kill a story's
  cancel causes is asked for as the cancel is. The world keeps, from the
  service's observations and on the clock the kills' records read:
  - each root's `Cancelling` notes (`../protocol/services.md`, 5), with
    the iteration's `now` each was taken at, whichever root took them:
    the standalone service or a cluster's worker;
  - each task's requester, read from the engine's live tasks while the
    task lives, so that a delegate's ancestry outlives its end;
  - each cancellation the engine decides: its task, the party that
    decided it (jig's `Ending::Cancelled`, its `by`), and the
    iteration's `now` it was first seen at.

  The story cancelled a task when a person decided its cancellation (a
  story's `task cancel` step) or a task whose executor is an agent did
  (a scripted agent's `cancel` tool); the deployment's own cancels, and
  a procedure's or a job's, are not the story's. A kill whose reason is
  `Cancelled` and whose requester is a root stands for the attempts that
  root noted `Cancelling` at the latest instant at or before the kill,
  every one of them where several share that instant (a cancel's
  cascade). The check accepts it when it is a `Signal(9)`, it stands
  for at least one attempt, and each of those attempts' tasks is one the
  story cancelled, or a delegate below one, whose cancellation was first
  seen at or before the kill. Every other kill needs a `Kill` step: a
  `Cancelled` kill with no `Cancelling` note at or before it, or one
  standing for work the story didn't cancel (a root's own cancel), a
  `Deadline` kill, a `Grace` kill (supervision's forced kill of an
  agent), a `Closed` kill, and any kill a hosted child requests. On a
  refusal, the finding's line for each `Cancelled` kill goes on, after
  its time, with `; stands for run R attempt A, cancelled with task C`
  for each attempt it stands for whose cancel the story asked, `; stands
  for run R attempt A, not the story's cancel` for each other, or `; no
  cancel noted`.
- **A kill no story asked for names its cause.** The check's finding
  prints every kill the world recorded, one line each as skein's
  `testing/service-tiers.md`, section 1 writes it, then each one's time from the run's
  start in milliseconds, and then the service's notes from the run's
  last 5 seconds: each note the service root took, kept with the
  iteration's `now` it was taken at, those taken no more than 5 seconds
  before the run's end, the latest 100 at most, oldest first, each as
  its time from the run's start in milliseconds and its `Debug`. A
  story's checks are `check(&Ran) -> Result<(), String>`, the finding
  as a value (`scenarios.md`, 1.2), which `checked` panics on, so a test
  that runs a story to its end without them reads the finding itself.
  A focused story whose agent ignores its cancellation and its grace
  holds it: smith's supervision kills the agent, and the finding names
  the agent's program, the service root as its killer and `forced after
  its grace`.

## 6. Forge, git and CI

- **The forge and git:** skein's forge peer (skein's `testing/fakes/forge.md`,
  section 5), which the one-process world hosts too, serves the fake forge on a
  loopback listener, its git route among its routes, which the fake
  git program reaches (`fakes.md`, 3). Stories that clone, push or land run on the
  fake git program over that route, with neither an owned Forgejo nor
  its runner. Real git against the fake forge is an open question
  (`README.md`, section 6).
- **CI and webhooks.** CI follows the fake forge's content cue (skein's
  `testing/fakes/forge.md`, section 2); CI that runs a workflow's steps in the
  shell subset is an open question (`README.md`, section 6). The forge peer
  delivers a hooked repository's webhooks to the engine's webhooks'
  listener (../protocol/forge/README.md, section 4), and polling stays the
  backstop, at paces the stories' configuration sets short (`poll`,
  `poll_max`, `slow` and `hinted`, the read a delivery re-arms).

## 7. Terminal stories

- **The person's terminal client.** The real loop also hosts temper's
  terminal client, in the loop as the service is: temper's client root
  and view under skein's text shell, over jig's native transport to the
  service's clients' listener, on a loopback port the kernel assigned.
  It is a general part of the tier (`tests/real-loop/src/terminal.rs`),
  not of one story: a story that needs a person at a client in the real
  loop has one.
  - **As `temper-box open` makes it.** The harness hosts the terminal's own
    composition, `temper-client-shell-terminal`'s client and terminal,
    as a program the harness spawns with its pipes for the terminal's
    input and output, as the tools' root is hosted for `temper ctl`.
    Its settings are `temper-box open`'s (../client/line.md, 2) but its
    print interval, which the tier sets short, at 20 ms, as it sets the
    engine's paces short: the shell prints at once after a command, and
    a story waits on what the pages say, never on the interval. The
    shipped interval's pacing is skein's text shell's to test (skein's
    `ui/text.md`, section 8).
  - **Signed in by a link no client has spent,** as `temper-box open DIRECTORY LINK`
    is (../client/line.md, 1), on a DIRECTORY of its own in the
    scratch directory, never one of `temper ctl`'s
    (../client/ctl/README.md, section 12). A story's key comes first
    through `ctl`, as for the tier's other stories (above), and that
    `ctl` spends the printed link. The terminal therefore opens on a
    fresh link that the signed-in `ctl` asks for the same person
    (`link --out FILE`, ../client/ctl/command-line.md, 1.1), a private
    file in the scratch directory, never on the printed one. A terminal
    refused on the spent link, then signed in on the fresh link the
    service prints, is the one-process world's (above).
  - **The typist** stands where a person sits. It types whole commands
    to the shell's input, as the person would (../client/line.md,
    section 5): a control's number from the last printed page, found
    by its label, with the words after it, and `q` to close. It reads
    the pages the shell prints (../client/line.md, sections 3 and
    4): what it knows of the system is what the pages say. It is
    ordinary Rust in the test process, with the process tier's typist
    as its model (`testing/temper-box-kit/src/terminal.rs`) and no
    terminal device; the shell's input and output are the harness's
    pipes in the loop. Its acts are a story's steps: open, type on a
    label, see a page that holds a text and lacks another, and close.
    It waits by the loop's deadlines, never by sleeping; a see reads
    only the pages printed after its last command; and every page it
    read is kept, in order, for the story's checks.
  - **The peers are fakes:** skein's fake LLM, the fake forge and the
    fake git program (`fakes.md`, 1 to 3), as for the tier's other stories.
  - **The terminal is the real loop's test surface.** People use the
    browser, or the TUI; the line client is the
    way a real-loop story reads what a person would see
    (../client/line.md, section 9). The process tier's `temper-box open`
    tests stay smokes (`process-tier.md`), outside the gate. A chat
    read on its pages, from a reply's first held piece to its committed
    turn, is the client world's (above).

## 8. Cluster and reference case stories

- **The cluster's services** (../protocol/cluster.md, 6): an engine
  alone and workers alone, `temper-service` services in the one loop,
  each on three scratch directories of its own (../protocol/cluster.md,
  1), the workers dialling the engine's workers' listener on loopback
  with their fixture secrets (../protocol/cluster.md, 3). Its story here
  is the worker's `PATH`, which a real child shell reads; the case
  library's stories run on it too (below). It shares the standalone
  stories' hosts, fakes and checks. Placement, a lost worker and its
  grace are the engine's domain world's; two one-slot workers each
  running a chat, the system world's, and two workers alone taking
  their figures, linking and draining, the one-process world's
  (above). An engine restarted within the worker's engine-away bound,
  after refused starts and past the bound, and a worker stopped before
  its run's first turn, are the system world's
  (`../domain/worker/README.md`, 9); the link's grace across refused
  hellos is jig's worker link's.
- **The case library's stories** (`reference-case.md`): its act 1, one story for each
  step's slice that temper can run, on the case's cast, scripts and
  oracle. The cluster's runner hosts them too: the case's fixture and
  fake forge under the engine and every worker, and a worker configured
  from the start but started at a step of the story, the person's act
  (../protocol/cluster.md, 2 and 6), and a worker stopped and
  started again at a step, the person's acts: its ending (15.4), then a
  new service on its own three directories in the same host, which links
  at its position. Every worker, a late one and a restarted one too, has
  the cluster story's link deadlines and grace, so no worker is lost in
  a story: a run under a worker that ends fails into a retry, and a lost
  worker and its grace stay the engine's domain world's.

## 9. What belongs in this tier

What only the real kernel shows is this tier's: a real child's `PATH`,
a service drained by its termination, `temper ctl`'s lock between two
acts, ports the kernel assigns and a listener already bound, a real
script's output and an agent's child tree torn down, within `checks.md`,
1's budgets. A failure found here
is rerun lower down as a scenario, where it replays (skein/docs/foundation/testing-strategy.md,
2.8). The process tier keeps executable startup, CLI
parsing, exit statuses, OS process launch and signal handling, the
owned Forgejo and runner, the cluster's processes, and production's
containment (`checks.md`).
