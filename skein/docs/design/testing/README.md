# Testing skein

Provisional, 2026-10-03, revised 2026-10-30. How ../../foundation/testing-strategy.md
applies to skein itself: which tiers skein has, what they run, the
example services and the minimal fake machine they need, where the tests
live, what skein supplies for a service's own tiers, and where things
stand. How each part is tested in its own tier is in that part's
document. The shared LLM, forge and checkout fakes are indexed in
[`fakes/README.md`](fakes/README.md).

## 1. In one page

- **skein tests the kit.** Its tiers stop at io under a small example
  service, the echo server and its fake client; and at a
  browser under the UI example, a small application on skein-ui. A
  service's domain, protocol layer and fakes are tested in the service's
  own tiers, which stand on skein's and do not retest them.
- **Each part has its tier:** lib in step tests, each machine in machine
  worlds, the machines stacked in protocol worlds, io in io worlds, the
  loop in simulated worlds of the examples, and the ring in the real loop
  and the conformance suite.
- **The examples are the template.** skein's simulated worlds and real
  loop run them, and a service's worlds copy them.
- **skein ships no fake machine.** Its own io and simulated worlds use a
  minimal one, which stays that small.

## 2. skein's tiers

| Tier | In skein | Its design |
|---|---|---|
| step tests | lib's containers and value types; each crate's step functions | ../runtime/lib.md, 10 |
| machine worlds | HTTP, server-sent events, JSON, each side alone; the TLS client, against rustls's server in memory | http.md, 6; ../data/json.md, 6; ../net/tls.md, 5 |
| protocol worlds | an LLM client's HTTP, server-sent events and JSON against a server's; skein-llm's subscription client against independent wire scenarios | http.md, 6; ../llm/README.md, 5 |
| io worlds | io over the simulator, with a scripted owner | ../runtime/io/README.md, 5 |
| simulated worlds | the examples' `iterate`, each a process of the simulator | section 3 |
| real loop | the examples under the shell, on the real kernel | section 3 |
| beside them | the conformance suite, against the simulator and the ring | ../runtime/kernel.md, 8 |
| beside them | the simulator's own tests; the ring's own tests | simulator.md, 6; ../runtime/shell.md, 8 |
| durable store | `skein-kv` step tests; `tests/kv` over the crashing fake disk and the real ring | ../data/kv.md, 9 |
| browser tier | the browser kit against real Chromium; the UI example's bundle in Chromium, through the person's browser face and harness | browser.md, 9; ../ui/web/testing.md, 1 |

TLS is tested on its own (../net/tls.md, 5), and the replaying tiers run in
plaintext.

For a domain world, `skein_world::domain::Stage::uncapped(limits)` owns
its environment, FIFO inbox and uncapped output queue. The world calls
the domain for each event and due alarm without reserving output room;
the stage holds neither back. Incoming work is bounded by the world.

## 3. The examples

A small service in `examples/`: the echo server, with its fake client.
It exists to be tested, and to be copied.

- **In simulated worlds,** it runs as a process of the simulator, with
  every layer real. It is the first tier where the loop runs, so it tests
  what the loop adds: the two passes, room checked at the kernel's queues
  and new work held at the root's watermark (../../foundation/programming-model.md, 7.1),
  deadlines across layers, and requests made in the down pass reaped in
  a later iteration.
- **In the real loop,** it runs under the shell on the real kernel: the
  ring, loopback sockets, a scratch directory as the root, real child
  processes and the real clock. It shows what only the real kernel
  can: the ring adapter's `unsafe`, the probe at startup, signals read
  from a signalfd, and a process tree that ends.
- **As a template,** a service's simulated worlds start from its.

The UI example (../examples.md, section 9) is an application's rather than
a service's: a view on skein-ui, its browser shell and its server. Its
view and its server are tested in step tests, and the three together in
the browser tier (../ui/web/testing.md).

Their design, the steps before code of each, their fakes and the harness
their worlds run on are ../examples.md. The echo has a domain, a protocol
layer, a service and a `main`, its fake client, and its worlds in
`tests/echo`, over the world harness, `skein-world` (../examples.md, 6).

## 4. The minimal fake machine

The simulator plays the kernel, not what a program does: that is the
embedder's fake machine (simulator.md, 3). skein's own io and simulated
worlds need a minimal one:

- a few files beneath a root;
- a program that echoes its input, one that exits with a given status,
  and one that never exits.

It stays that small. A service's fake machine is the service's, built
on it or beside it.

It is `skein-fake-machine`, in `crates/`: a playground ships it in its
world, as it ships the simulated world (world.md, 2). A world lays a
root in it from a scenario's items (files, directories, symbolic links,
each with its mode) and gives its handle to a process as the shell would
a root opened at startup; after each submit, the world takes the
simulator's calls and has the machine answer each (simulator.md, 3.1).
The machine resolves paths beneath a root as `openat2` with
`RESOLVE_BENEATH` does, follows the links that stay beneath it, keeps its
owner's permissions, and refuses what a real filesystem refuses, in the
order Linux checks; the conformance suite holds it, through the
simulator, to the real kernel in a scratch directory. It keeps a step
machine's shape, a call in and an answer out, in a vocabulary of its own
that a face translates to and from the simulator's. Its programs are
the fixture of the conformance suite's processes, and its crash model
keeps what a crash keeps (simulator.md, 3.3). It
keeps each file's lock with the handle that took it (simulator.md, 3.1).
For a tree's metadata (../runtime/kernel.md, 6.4) it keeps each entry's inode,
group, mode up to `0o7777`, times and attributes, and hard links, and
lays FIFOs; it has no devices, no sockets and no btrfs. It answers every
subvolume call `NotSupported`, and its status a filesystem that is not
btrfs, as Linux does off btrfs, so skein's btrfs scenarios run on the
ring alone (../runtime/kernel.md, 8). A service whose fake machine models btrfs,
such as hearth's, holds that machine to those scenarios itself.

For trees (../runtime/io/processes.md, 2 to 4; simulator.md, 3.4) its programs add: one
that starts a descendant which leaves its group (`setsid`) and outlives
it; one that forks a child each time it is signalled; and, on the ring,
one that writes what it finds of itself, its ids and groups, its
hostname, its network's interfaces, the processes in its `/proc`, its
`no_new_privs`, its rlimits, and whether chosen paths can be read and
written, so that a conformance scenario checks a contained tree from
inside it (../runtime/kernel.md, 8).

A separate dependency-free kit, `skein-fake-checkout`, supplies generic
byte-path files, scripted commands and local git mechanics for domain
worlds (fakes/checkout.md). It has no simulated-kernel adapter, inode or
permission model. Services retain their policy, remote implementations,
clock, cancellation and delivery ledgers; the kit answers synchronous
mechanical operations only. It lives in `crates/`, as temper's `temper
fake git` ships it, and its focused leaf tests and small replay sweep are
the crate `tests/fake-checkout`, by the rule below.

A code forge has fakes of its own (fakes/forge.md). `skein-fake-forge-domain`,
a step crate on skein-lib alone, plays a forge's state, CI, merges and git,
and `skein-fake-forge-forgejo` serves Forgejo's API over it on skein's HTTP
server. The forge peer in `skein-fake-peers` hosts that layer on loopback
io. Its GitHub layer (`skein-fake-forge-github`) shares the domain and
hosts API/browser routes on one listener and signed job logs on a second
blob listener, with one shared connection limit. `skein-throwaway-forgejo`
runs the pinned Forgejo in a scratch directory, for the tools and process tiers that need a real one, and a
conformance tool outside the suites holds the dialect to it. A service
keeps its fake of its own git component, its worlds and its cost ceilings.

## 5. What skein supplies for a service's tiers

| Service tier | Real | What skein supplies |
|---|---|---|
| step tests | one of its step functions | lib |
| domain worlds | one domain or child domain | lib, the counting allocator; for an application on skein-ui, the fake person (../ui/README.md, 14.1) |
| system worlds | several of its domains, or several services' domains | lib, the counting allocator |
| protocol worlds | its protocol layer, over skein's machines | the machines, tested in skein's tiers; the counting allocator |
| simulated worlds | every layer, `iterate` per process | the simulator, the world harness, the counting allocator, the examples as a template |
| real loop | the service as it ships | the shell kit, the world harness's real loop; for a service with a web, the browser kit (browser.md); for an application on skein-ui, the person's browser face and harness (../ui/web/testing.md, 2) |
| end to end | the service's binaries as they ship | the world harness's end-to-end kit: binaries started on pipes or a pseudo-terminal, and each binary's process tree settled and counted, its CPU and peak memory (../examples.md, 6) |

The service supplies its fakes, the fake machine that plugs into the
simulator among them, and the scenarios its worlds run. A failure in a
service's world that comes from io, a machine or the simulator is
reproduced in skein's own tier, and fixed there.

A service that works on a forge takes the forge's fakes from skein: the
domain and the protocol layer in its protocol worlds, the forge peer in
its real loop, and the throwaway Forgejo in its tools and process tiers
(fakes/forge.md, sections 2 to 6). The peer serves plaintext on loopback
over a forge the service may share with its own fake of git. Its
transcripts of git stay its own.

**The counting allocator** is `skein-heap`, in `testing/` (section 6): a
global allocator that counts each thread's heap, and a meter that checks
a step at the most it held of its own, less what it handed out in its
requests, under its scenario's named ceiling with headroom
(../../foundation/programming-model.md, 6.3). Every world that checks memory uses it,
whatever its tier (../../foundation/testing-strategy.md, 6), in a test binary of its own
that declares it. It measures; it runs no world. In a simulated world,
the check at every iteration is the world harness's: each hosted
service's peak against the ceiling its scenario names
(../../foundation/testing-strategy.md, 6), measured with the allocator's span, the heap's
growth around one call, signed. Whatever the harness does to a process's
own containers runs inside that process's ledger span: delivering
completions, draining submissions, and the real loop's `ring.deliver`
included. A ring that grows on a harness push is charged to its process.
The host declares no ceiling or worst case (../runtime/shell.md, 11).
`skein_world::Memory::Checked` checks ownership and release and reports
peaks without a ceiling; `Within(ceiling)` also checks that ceiling at
every iteration; `Unchecked` uses the system allocator. `Outcome::peaks()`
and `real::Outcome::peaks()` report each process's peak alone, and
`Killed::peak()` reports the ended process's peak. An unchecked run
reports no measurement, never a ceiling or a sentinel.

**Reuse is counted, never read from an address.** A span also counts
the allocations its thread made since its start, a reallocation as one
(`Span::made`). A step test that shows a table kept rather than grown
or replaced at its size counts none around the call that reuses it, in
a test binary that declares the counting allocator. It never compares
the table's address: an allocator may grow a block in place or hand a
freed one back, as what ran before in the process left its heap
(../../foundation/testing-strategy.md, 8).

**Who counts a payload handed to a step.** A step's measured heap covers
what it keeps and what it allocates; what it hands out in its requests
is the next step's input, and the meter takes it off the step's own. An
input moved into a step is counted by whoever made it, with one
exception: a delivery from the side below is made to the reading
machine's own demand, within its largest demand, so the reader's
measured heap covers the one it reads, for the step that reads it. A
request whose size the receiving step's limits do not bound, such as an
HTTP call's target and fields or a piece of a request body, is counted
with the sender's payloads (the protocol layer's), and the memory test
accounts for that input's size when checking the receiving step's peak
under its scenario's ceiling.

**Exact checks are contracts:** zero after release, the ledger's account,
a byte price equal to what the allocator saw, or an edge formula equal
to the size it bounds. Other memory checks hold peaks under named
ceilings with headroom, moved by hand when a real change passes them,
never by a type's size; no write mode updates them (../../foundation/testing-strategy.md,
6).

**The world harness** is `skein-world`, in `testing/` (../examples.md, 6),
built over the simulated world, `skein-sim-world` in `crates/`, whose
steps it runs and whose items it re-exports (world.md): one loop over
the processes' `iterate`, generic over a scenario's processes, with the referee's expectations, the trace, the heap at every
iteration, and the settled world's invariants; and the same loop over the
shell's shared ring, on the real clock. The examples are its first user and its
second, which is why it is skein's (../../foundation/testing-strategy.md, 7).

- **It hosts the shell's `Host`.** `Host` and `drive` are `skein-shell`'s
  (../runtime/shell.md, 11), and `skein-world` re-exports both, so a service's worlds
  host the same `Host` its `main` drives, and a shipped binary links no
  testing crate.
- **It builds on Linux and macOS alike,** and its simulated half,
  `skein-sim-world`, for `wasm32-unknown-unknown` too (section 10).
  The simulated worlds need no real machine; the real loop opens the host's backend on either, and
  refuses one a test forces that the host lacks (the ring on macOS,
  ../runtime/readiness.md, 1). Only the tree keeper's cgroup and walk are Linux's
  (../examples.md, 6); its process group runs on both.
- **It calls each process's hook** where `drive` does: once per
  iteration, after the process's `iterate` and before its submit. The
  drains a world runs are then the ones that ship.
- **It checks the teardown invariant** (../../foundation/testing-strategy.md, 6) in every
  world, at every tier it runs, for every service: after the scenario's
  last word, by default the referee's last expectation met, it fails the
  world when time must reach a process's `next_policy_deadline()` before
  the world settles, naming the process and the deadline. In the
  replaying tiers that is a jump of the clock to it; in the real loop, a
  wake for it. No service writes the check, and none opts out.
- **Its checks return their findings** (../../foundation/programming-model.md, 10.4).
  A referee's, a judge's or the harness's own check that fails gives its
  finding as a value, the text it fails with now: the seed, the time,
  what broke and, in the simulated world, the trace's tail. Only the
  loop or the test that drives it panics on it, once, at its top, so a
  check's own tests read its finding and catch nothing.
  - `Referee::watch(&mut self, now: Time, procs: &[P]) -> Result<(), String>`
    watches every process after one iterated, and returns a broken safety
    expectation's finding. The simulated world's turns and the real loop
    call it after each process iterated, and once in a pass where none
    did, so at least once after each `act`, and fail with its `Err` at
    once, observing nothing after it. Every referee implements it: it has
    no default, and the trait has no `observe`. A referee panics in none
    of its methods. What its `act`, `idle` or `passed` finds, it keeps:
    `passed` answers `false` while it keeps one, and its next `watch`
    returns it. A referee that has nothing to watch returns `Ok(())`.
  - `Expectations::check(&mut self, now: Time, procs: &[P]) -> Result<(), String>`
    sets each met expectation aside and returns the first broken one's
    finding, `seed <seed>: at <ns> ns, <expectation> broke: <why>`.
    `Expectations::observe` panics on it.
  - `World::try_run`, `World::try_run_with_faults` and
    `World::try_run_with_inputs`, each `-> Result<Outcome<P, M>, String>`,
    run as `run`, `run_with_faults` and `run_with_inputs` do, and return
    the run's verdicts as a finding: a referee's, as it wrote it, naming
    the seed; and the world's own, an overdue expectation, a world idle
    and unsettled with nothing due, one past its steps, and the teardown
    policy check, each with the seed and the trace's tail as `run`
    prints them now. `run*` call them and panic on the `Err`. A
    restartable startup that can't open its roots or appends closes
    what it opened and returns why,
    `World::try_spawn_restartable_startup(roots, appends, make) -> Result<usize, String>`,
    which `spawn_restartable_startup` panics on.
  - The real loop fails with a finding only after its forced cleanup,
    as above. `real::World::try_run(self, clock: &Clock, patience: Duration)
    -> Result<Outcome<P>, String>` and `real::try_run(procs, referee,
    clock, patience)`, its unhosted form, return that finding: a
    referee's `watch`, an overdue expectation, the patience verdict with
    its figures, the teardown policy check and a binary's kept finding,
    each in the text `run` fails with now. `real::run` and the real
    `World::run` call them and panic on the `Err`. A starved run is no
    finding: it passes as `Starved::pass` does, below. A binary keeps an expected end's finding, a process group
    or tree process still running at its exit (../examples.md, 6), as a
    value the loop reads, and the tree keeper's report of what it left
    running and its cgroup's removal error are values too, which the
    kit's teardown fails with once, on an ordinary return. Conformance's
    real backend gives a wait that passes unanswered, its listener's
    among them, as a finding with its figures.
  - The domain world's referee, `skein_world::domain::Referee`, holds its
    verdict as a value (`verdict()`). `check_holding(seed)` and
    `check_passed(seed)`, each `-> Result<(), String>`, give the text
    `assert_holding` and `assert_passed` panic with. `Ledger::check_settled()
    -> Result<(), String>` and `domain::check_replays(seed, other, run)
    -> Result<R, String>` are the returning forms of `Ledger::assert_settled`
    and `domain::assert_replays`, which panic on them.
  - A harness that adds context to a failure writes it before it runs
    what may fail: conformance's simulated driver names each seed and
    configuration on stderr before running it, and catches nothing.
- **It tells a starved run from a slow one** in the real loop
  by `Deadline`, the same real-clock deadline every harness uses
  (../../foundation/testing-strategy.md, 2.10). `skein_world::{Deadline, Expired, Starved,
  Figures}` carry the wait and its scheduler accounting.
  `Deadline::arm(what, patience)` starts the calling thread's account;
  `watch(&Binary)` and `watch_process(pid)` add owned process trees,
  preserving earlier watches. Descendants count; a thread first seen
  after arming counts from zero only when its process provably began
  after arming. Unreadable or vanished threads contribute nothing.
  `passed`, `remaining` and `until` let the loop wait; `expired` classifies
  an unanswered wait once it passes. The largest single runnable wait
  decides, never the sum of waits from separate sources. At least half
  the patience spent runnable is starved; absent accounting cannot prove
  starvation. `Figures` records the patience, elapsed time, CPU time and
  runnable waits.
- **The real loop classifies its patience and the referee's overdue
  liveness deadlines.** A liveness deadline's patience is its time less
  the run's start. A referee can return its own starved verdict through
  `Referee::starved() -> Option<Starved>`. The loop drains the forced
  cleanup before passing that verdict on. The same cleanup precedes a
  failure of the loop's patience, the referee's expectations or the
  teardown policy check: it ends the owned trees, cancels and drains the
  shared kernel, then reaps retained leaders and closes the descriptors
  their outstanding records kept open. The diagnostic and scheduler
  figures describe the failure before cleanup. This order holds when
  panics abort; cleanup never depends on unwinding after the report.
  `Starved::pass()` writes the
  `skein-world: starved:` line to stderr, naming the wait with `, by <what>`,
  and ends the test as a starved pass: in a one-process run
  (`TIERS_ONE_PROCESS` set, by `tools/tiers/design.md`'s one-process
  mode) and under nextest it ends its process with exit 0 after the line,
  so that nothing follows it and nothing unwinds, and the one-process
  runner starts a new process for the names its process had left
  (`tools/tiers/design.md`, 11.5 and 11.6); elsewhere it fails with the
  figures. Nothing catches a panic to end a test
  (`../../foundation/programming-model.md`, 10.4).
  `Starved::exit()` ends a helper process, one a test starts and whose
  stderr it reads, with the line and exit 0 wherever a starved wait passes.
  `starved_passes()` says whether it passes here (`TIERS_ONE_PROCESS` or
  `NEXTEST` set), and `pass_starved(line)` ends the running test with a line
  a helper wrote, as `pass` does. Every place that keys on `NEXTEST` for
  starvation keys on `starved_passes()`, and a test that runs a child to see
  a deadline fail removes both variables from it. Tree starvation also names
  the tree's wait and accounting source. Conformance's `Backend` gives
  `wait_begins`, `wait_unanswered` and `wait_figures`; the simulator uses
  their defaults.
- **A binary's counts mark tree settlement.** `Binary::counts()` is
  present once the binary and its descendants settle, whatever stderr's
  state. Stderr EOF and descriptor closes still drain before its observer
  is empty. A cgroup is removed only after `populated 0`; after
  `cgroup.kill` the keeper waits within a 2 s deadline. A group retaining
  a leader is left for process exit. Removal errors while unwinding are
  reported without raising another failure.
- **No fixture outlives its test.** A fixture that would run until
  signalled ends with the test that started it however the test ends,
  a runner's kill that runs no cleanup among them: on Linux its leader
  dies with the thread that started it (a parent-death signal,
  `exec_dying_with_parent`, ../runtime/shell.md, 3), each descendant
  it starts that leaves its group or session ends its own loop once
  that owner process is gone. None of that waits on unwinding: a test
  that unwinds may have the kit end the rest sooner (../examples.md, 6),
  never alone, and no test exercises that path, which a test of the
  process ending covers. skein's
  process fixture (`tests/conformance/real`, built for the worlds as
  `world_process_fixture`) ties its `never`, `fork-live` and
  `supervise-*` modes so; a descendant that only sleeps its bounded
  minute ends by itself.
- **The highest tiers check paths** through the real loop and shipped
  binaries. The world harness tests ring routing, cleanup, observation
  and settlement without a kernel in its unit tests and simulated
  worlds. `skein_shell::drive` requires a real `Kernel` and `Clock`;
  its three scheduling rows stay in the real tier as paths.
- **It carries state digests** (../runtime/lib.md, 5.1). `Host` (../runtime/shell.md, 11) and
  `Machine`, the seam a world's fake machine plugs into, each give
  `state_digest() -> Option<[u8; 32]>`, `None` unless the process or the
  fake gives one. What a host hashes is its own choice, its domains'
  state at least; a fake machine hashes its own state.
  - **At the end of a run:** a world reads them from its `Outcome`'s
    processes and machine, and returns them beside its trace, so
    `assert_replays` compares them with the rest of what must come out
    the same. Its signature is unchanged.
  - **After each iteration, on request:** with the world's `digests`
    option, the harness writes each process's and the machine's digest
    into the trace after each iteration, so two runs that part are found
    at the first iteration whose digests differ, before anything crosses
    a boundary. A process's is taken where its hook runs, after its
    `iterate` and its drain and before its submit; the machine's once
    every process has had its turn in the iteration. Each is an entry of
    the trace that simulator.md, section 3 names. Without it, the trace is
    as before.

Both in-loop worlds refuse `Stop` and `Continue` to hosted children with
`InvalidArgument` (EINVAL). Holding one needs its host's readiness, steps
and deadlines set aside by the scheduler, which no story needs yet.
Supervised binaries accept both; `TreeStatus.stopped_pids` and
`continued_pids` retain identities observed alive and successfully sent
STOP and CONT by the keeper. Neither signal counts as cleanup.

## 6. Layout

```
crates/*/src/tests.rs           step tests; lib's, io's, JSON's, HTTP's and TLS's in a module per area, under src/tests/
crates/skein-ui                 the UI kit (ui.md): its step tests, and its golden scenes beside them
crates/skein-ui-text            the text shell (ui-text.md): its step tests, and its golden text beside them in src/tests/golden/
crates/skein-ui-screen          the full-screen shell's renderer (ui-screen.md): its step tests, and its golden frames beside them in src/tests/golden/
crates/skein-ui-screen-terminal the full-screen shell's keys, drawing and terminal (ui-screen.md): their step tests
crates/skein-ui-web             the browser parts (ui-web.md): each part's pure half tested on the host in its step tests
crates/skein-ui-bundle          the bundler, a tool built optimized (../ui/web/bundle.md, 2)
crates/skein-fake-llm-*         the scripted LLM peer: its domain and its byte peer (fake-llm.md), a product the worlds and the real loop use
crates/skein-fake-forge-domain  the fake forge's domain (fakes/forge.md, 2 and 3)
crates/skein-fake-forge-http    what its two layers share (fakes/github.md, 3)
crates/skein-fake-forge-forgejo its Forgejo protocol layer (fakes/forge.md, 4)
crates/skein-fake-oauth         an independent rotating OAuth issuer (../oauth/README.md, 5)
crates/skein-fake-peers         the LLM, OAuth and forge peers hosted on loopback io, in a world or as an application's own process (fakes/forge.md, 5); links skein-fake-forge-github for GitHub's API/browser and blob faces
crates/skein-fake-checkout      files, scripted commands and local git for a service's domain worlds, and for a service's fake git as a program (fake-checkout.md)
crates/skein-fake-forge-github  the fake forge's GitHub API/browser and signed-log blob layer (fakes/github.md, 1), which the peers link
crates/skein-fake-machine       the minimal fake machine: files beneath a root, its programs, and its face behind the simulator (section 4)
crates/skein-sim-world          the simulated world: processes hosted over the simulator and stepped, which the harness and a playground run (world.md)
testing/skein-conformance       the conformance suite: the backend interface, the scenarios, the driver, the checks
testing/skein-heap              the counting allocator, the meter that checks a step under its scenario's ceiling, and the span a world meters its processes with
testing/skein-scratch           a scratch directory beneath the system's temporary one, removed when dropped, for tests of files on the real kernel
testing/skein-fork              a run forked at a chosen moment of a test and waited for, so that a sweep of cuts holds its prefix once
testing/skein-rows              one table test's rows: each named as it runs, and the first failure ends the table (../../foundation/testing-strategy.md, 8)
testing/skein-world             the world harness: the simulated world's run with the referee, the trace and the heap, and the real ring's loop
testing/skein-echo-client       the fake echo client, a step machine
testing/skein-throwaway-forgejo the pinned Forgejo run on loopback in a scratch directory (fakes/forge.md, 6)
testing/skein-forge-conformance the conformance records the forge tools share, and their secret scans (fakes/forge.md, 7)
testing/skein-fake-channel      a scripted framed peer for a service's protocol worlds (channel.md)
testing/skein-protocol-kit      what the protocol worlds share: a demand's reads and the draws below them, the sweep's tally, the transcript loader
testing/skein-json-kit          what a second crate takes of the JSON world: the reference reader, the generator and their outcome types
testing/skein-http-kit          the HTTP world's reference reader
testing/skein-tls-kit           the TLS world's certificates, in-memory server, byte driver and step meter, over tests/tls/fixtures
testing/skein-llm-kit           the LLM world's wire fixtures and its faces of skein's fake LLM
testing/skein-forge-kit         the forge machine world's machine, peers, pipes and joint world, and its GitHub face
testing/skein-forge-connection-fixtures the forge connection's fixtures for the machine world's hand, a copy of the crate's step tests' own (../forge/connection.md, 10; ../../foundation/cli.md, 3)
testing/skein-codegen           the codec generator, run by a consumer's tests (codec.md)
testing/skein-browser           the browser kit, a step machine: headless Chromium over the DevTools protocol on a pipe (browser.md)
testing/skein-fake-person       the person: targets by role, name and text, steps, the tree face, the random person (../ui/README.md, 14.1)
testing/skein-fake-person-browser the person's face over the browser kit, and the harness: an application's server binary, Chromium and the peers a test lends in one loop; skein's stories for a browser shell (../ui/web/testing.md, 2)
tests/heap                      the counting allocator's own tests, skein-heap-tests
tests/echo                      the echo's simulated worlds and its real loop, skein-echo-world
tests/io                        io worlds: io over the simulator, a scripted owner, a referee, skein-io-world
tests/lib                       lib's comparisons with naive functions, run long, and its allocation prices against the counting allocator, skein-lib-tests
tests/sim                       the simulator's own tests, skein-sim-tests
tests/world                     the world harness's own tests, over scripted processes of raw records, skein-world-tests
tests/ring                      the ring adapter's own tests, skein-ring-tests
tests/readiness                 the readiness backend's own tests, on each system, skein-readiness-tests
tests/conformance/sim           the suite against the simulator, skein-conformance-sim
tests/conformance/real          the suite against the real kernel's backends, the ring and the readiness backend, skein-conformance-real
tests/json                      the JSON tokenizer's machine worlds, the writer against it, and their peaks under scenario ceilings with the counting allocator, skein-json-world
tests/json/transcripts          its transcripts, each with what it must decode to
tests/http                      the HTTP client's and server's and the event stream reader's and writer's machine worlds, each read by a reference reader of `skein-http-kit`, the client and the reader stacked with JSON, and their peaks under scenario ceilings with the counting allocator, skein-http-world
tests/http/transcripts          its transcripts, responses and, in requests/, requests, each with what it must decode to
tests/protocol                  the protocol worlds: an LLM client's stack against a server's, joined by bytes cut at random, skein-protocol-world
tests/llm                       skein-llm's subscription protocol worlds, fragmentation and fault sweeps, and bounds against the counting allocator, skein-llm-world
tests/llm-connection            the LLM connection component's worlds, skein-llm-connection-world
tests/oauth                     the OAuth client against the independent issuer, skein-oauth-world
tests/oauth-accounts            the OAuth driver's worlds, its owner scripted, skein-oauth-accounts-world (oauth.md)
tests/oauth-issuer              the client against the OAuth issuer, skein-oauth-issuer-world (oauth-issuer.md)
tests/forge/machine             the forge component's machine world and the listener's, against a scripted HTTP peer and the fake's GitHub layer on its clock, and their peaks under scenario ceilings with the counting allocator, every holder filled (../forge/connection.md, 10)
tests/forge/protocol            the protocol world: the client and the component against the fake forge's protocol layer and domain, joined by streams cut at random, under a scripted durable owner that restarts; its fuzzy runs take seeds (../forge/connection.md, 10)
tests/forge/client              the client world: the client alone under a scripted parent, over the fake forge's domain (../forge/client.md, 10)
tests/forge/setup               the setup world: the host component over a store of items and the fake forge's setup routes and GitHub layer, through HTTP bytes, and its peak under a scenario ceiling with the counting allocator (../forge/setup/README.md, 5)
tests/forge/dialect             both dialects' documents, plans and bounds driven from outside, with their fuzzy mutations, skein-forge-dialect-tests
tests/forge/conformance         the conformance tool, outside the suites: a throwaway Forgejo driven, each fact the dialect relies on recorded (fakes/forge.md, 7)
tests/forge/github              GitHub's conformance tool, outside the suites: the hosted fake and owner fixtures (fakes/github.md, 2)
tests/fake-forge                the fake forge's domain, its Forgejo layer and its GitHub service against the counting allocator, skein-fake-forge-tests
tests/fail-stop                 a panic on any thread aborts the process after its report, through a fixture binary, skein-fail-stop-tests
tests/fake-peers                the LLM, OAuth and forge peers on loopback io against their clients, including GitHub's job-log redirect to its blob listener, skein-fake-peers-tests
tests/fake-checkout             the fake checkout's leaf tests and its replay sweep against a small private remote, skein-fake-checkout-tests
tests/channel                   the framed channel's machine and protocol worlds, skein-channel-world
tests/codec                     a generated sample codec and its worlds, skein-codec-tests
tests/codec-patches             the patch command's owner crate and its replayed history, skein-codec-patches (../data/codec/command.md, 9)
tests/kv                        the store over file io, the simulated kernel and a crashing fake disk, skein-kv-tests
tests/browser                   the browser kit's machine world against a fake browser, its tests against real Chromium, and its peaks under scenario ceilings with the counting allocator, skein-browser-world
tests/tls                       the TLS client's machine worlds against a rustls server in memory, the HTTP client stacked on it, and its peaks under scenario ceilings with the counting allocator, skein-tls-world
tests/tls/fixtures              its certificates and key, and the script that makes them
tests/link                      a link's close rule against a rustls server in memory, skein-link-world (../net/link.md, 4)
tests/ui                        skein-ui's peaks under scenario ceilings with the counting allocator, and its comparisons with naive functions, skein-ui-tests (../ui/README.md, 14)
tests/ui-text                   the text shell's terminal on the simulated machine, skein-ui-text-tests (../ui/text.md, 8)
tests/ui-screen                 the full-screen shell's terminal on the simulated machine, skein-ui-screen-tests (../ui/screen.md, 9)
tests/ui-web                    the browser parts' host tests that need more than a crate's step tests, and the UI example's browser tier, skein-ui-web-tests (../ui/web/testing.md)
tests/**/tests/suite/*.rs             a crate's focused tests
tests/**/tests/suite/fuzzy/*.rs       its fuzzy tests: sweeps over many seeds
tests/clippy.toml               what the crates under tests/ may not use
examples/echo/*                 the echo: its domain, protocol layer, service and shell, a crate each (../examples.md, 2)
examples/ui/*                   the UI example: its view, its browser shell and its server, a crate each (../examples.md, 9)
```

**A world's library is its own.** No package, whichever tree it's in, depends on a package under `skein/tests/`: what a second crate needs from a world lives in a kit under `testing/` (../../foundation/testing-strategy.md, 7), and the world's own suite reads it through its library, which re-exports the kit. No `#[path]` under `skein/` reaches outside its package. `tests/boundary/tests/skein_worlds.rs` holds both, for `skein/` alone: it doesn't read the worlds under smith's and temper's `tests/`, and packages still depend on those.

No crate under `crates/` has a `tests/` directory: its step tests are
in `src/`, and whatever drives it from outside is a crate under `tests/`,
named by its path. The fakes in `crates/` follow it too: the forge domain's memory test,
the peers' tests and the checkout's are crates under `tests/`, and the
protocol layer's fixtures are in its `src/tests/`. Such a crate is ordinary Rust (../../foundation/programming-model.md,
10.2): what its test binaries share is its library, in `src/` (the
simulator's harness and its scripted exchange, a backend of the suite,
the naive functions lib is compared with, a machine's world and,
through the kit it re-exports, the reference parser it is checked against),
and its tests share the `suite` binary: focused modules under
`tests/suite/`, seeded sweeps under `tests/suite/fuzzy/`.
`tests/clippy.toml` bans what would
make a run unrepeatable: hash maps with a random seed, the system clocks,
threads. Each backend of the conformance suite is implemented beside the
tests that run the suite against it. The global allocator a binary
declares covers the whole binary; memory modules are `tests/suite/memory.rs`, and `tests/suite/fuzzy/memory.rs` for a sweep
under scenario ceilings (../../foundation/testing-strategy.md, 8). A suite that checks
memory declares the counting allocator once. `Memory` selects whether a
world meters its processes; a simulated world checking memory meters
every iteration.

What a crate under `testing/` holds is shared by the tests of more than
one crate, and is ordinary Rust on a lint profile, each with a
`clippy.toml` of its own, or the step crates' (`tools/lints/design.md`, 3).
`skein-heap` needs `unsafe` for its global allocator, an `unsafe impl`.
../../foundation/programming-model.md (1, 2.1, 10.2) confines `unsafe` to the ring adapter
in code a service runs, and in tests to this allocator and the fork kit's
process boundary below. Both are test-only crates, never linked into a
service. The allocator allows its `unsafe` in
place, with a scoped
`#[expect(unsafe_code, reason = "…")]` and a `SAFETY` comment on each
block.

**A sweep's fork** is `skein-fork`, on the `ring` lint profile
(`tools/lints/design.md`, 2.4). One documented module holds its scoped
`unsafe` for `fork`, `waitpid` and `_exit`. A single-threaded test world
calls `split` at a cut, after stdout and stderr are flushed. The child
continues with its owned state; its first panic prints through the panic
hook and exits 101 without unwinding into libtest. `passed` exits a child
with code 0 and returns in the parent. The parent reaps its child and
receives `Exit::Code` or `Exit::Signal`; a child cannot fork again. Each
parent waits for each child, so the suite's CPU includes its descendants
(../../foundation/testing-strategy.md, 8.1). The kit supplies the process boundary alone;
the consuming harness takes the crash and checks every cut.

**A table test's rows** are `skein-rows`, on the `harness` lint profile.
`skein_rows::check(rows, body)` takes rows of `(&'static str, R)` and a
`body: Fn(R)`, and runs the body on each row in order, naming the row
before it runs: a line `skein-rows: row <name>` on stderr, through the
test harness's capture, which the runner prints for a failed test.
Nothing catches a panic (../../foundation/programming-model.md, 10.4),
so the first failing row, an `assert_eq!`'s panic as any other, ends the
table and its test, its name the last line before its message, and the
rows after it don't run. Rows carry data (../../foundation/testing-strategy.md, 10): the kit
hands a body the row's data and never its name. Fewer than two rows, or
two rows of one name, panics before any row runs: the caller's bug, an
assertion no test triggers. A test
that reads what an earlier row of its process would disturb, such as a
heap ceiling, stays a test of its own. The kit shares nothing between
rows; a fixture a body builds is built per row. The kit has no helper
that expects a panic, as no kit has: a row that would panic tests an
assertion, which goes, or a refusal, which its body reads as a value.

A fake that runs as a step machine, as `skein-echo-client` does,
is held to the subset instead (../../foundation/testing-strategy.md, 4): it has no
`clippy.toml` of its own, so the step crates' applies.

A machine's worlds are a crate of their own, `tests/<machine>`, with its
transcripts in `transcripts/` beside its tests. The minimal machine joins
the io worlds with files and processes; the protocol worlds are a crate
of their own, `tests/protocol`. The gate runs formatting, the lints as errors,
clippy's lints as errors for `wasm32-unknown-unknown` over the portable
crates (section 10) and the UI example's view and browser shell, since the
parts' platform halves compile only there (../ui/web/testing.md); then
the focused suite and the fuzzy suite, with their profiles (`README.md`, section 7), and
nothing slower. The browser tier, outside the gate, is
`tools/tiers/run skein browser`: the UI example's
bundle, made with `skein-ui-bundle` from a module built with the dev
profile (../ui/web/bundle.md, 1), and the browser suite, which runs the browser
kit's tests against Chromium and the UI example's tier on that bundle
(../ui/web/testing.md, 1). The builds for that target need only the
target installed: they run nothing, since the portable crates' tests run
on the host, and skein-ui-web's platform half runs in the browser tier.

## 7. Where things stand

How each part is tested, and what its worlds cover, is in the part's own
document; the echo's worlds are ../examples.md, 7.

Every tier follows ../../foundation/testing-strategy.md, 8.1: CPU budget, wall hard limit,
starved warning and the three-question repair rule.

The two gate suites use the root `.config/nextest.toml`'s `default` and
`fuzzy` profiles, with CPU budgets of 40 s and 120 s in
`tools/tiers/budgets.toml`, enforced at the merge gate under
../../foundation/testing-strategy.md, 8.1. The browser kit against real
Chromium and the UI example's browser tier use `browser-skein`
(`tools/tiers/run skein browser`; browser.md, section 9; ../ui/web/testing.md,
1); their four-minute global timeout is a wall
backstop, separate from their CPU budget. skein-llm against real endpoints
uses `live-skein`, with a key or a plan's token from the environment.
These two slow tiers stay out of the gate, each with the user's CPU
budget and wall hard limit under 8.1, in `tools/tiers/budgets.toml`. Schema, value, fingerprint, diff, patch, redirect
admission and fake-browser checks run in focused step tests and worlds;
seeded decoder comparisons, sign-in faults including restart, and the
forge machine world's every-byte TLS sweep run in fuzzy.
Measurements use the tier's profile and keep the same budget rule.

The exhaustive wall-day round trips and Gregorian phrase comparisons,
and HTTP response and request prefixes compared with a reference, belong
in fuzzy (../../foundation/testing-strategy.md, 8). Focused keeps named calendar edges and
incomplete-event behaviours. UTF-8's longer sequences use their range
boundaries in focused; random interiors go through the public writer in
fuzzy. The private Huffman builder's comparison keeps four seeds, two per
rarity regime, with its completeness, canonical decoding, width and
reference-weight checks. Scenario counts follow what their assertions
need: a ring still wraps, a store remains larger than its ceiling, and
every asserted close-state wait still occurs. Every retained turn of a
memory test stays metered.

Replay: a seed replays to the same trace of submissions and completions,
and a JSON world, an HTTP world, an io world and an echo world to the
same run; a TLS world does not, as rustls draws from the kernel. The
state digests replay with the trace (../runtime/lib.md, 5.1; section 5).

Memory: each part's peak is measured with the counting allocator in its
own tier, a call of an entry point at a time, under its scenario's named
ceiling with headroom; the echo's simulated worlds check each host at
every iteration (../examples.md, 6). A focused test drives the echo to its
edge limits and shows it by what it holds: every connection's intake,
receive and output full at once. Reaching the limits is checked by those
contents, never by nearly filling a formula (../../foundation/testing-strategy.md, 6).

The forge's worlds (section 6) keep within a budget of their own: a step
test under 40 ms, a focused world test under one second, and a fuzzy
run within 10 seconds, its seeds included, so that `tests/forge` stays within half of each suite's.

## 8. Not built yet

What of this document is not built is in [`README.status.md`](README.status.md).

## 9. Open questions

- **One-process adoption and suite costs** (5, 6, 7). Switching a
  caller's focused and fuzzy steps is that caller's separate change,
  tree by tree (`tools/tiers/design.md`, 8). Measure suite CPU per
  process after that switch; per-test CPU in a shared process is not
  measured. A run that checks outcomes establishes no CPU margin.
- **Further scenario and product costs** (2, 7). KV's CRC-32C and row
  copying, LLM replay conversion, real TLS and RSA, HTTP byte and chunk
  decoding, protocol generation, conformance pattern generation, GitHub
  transcript and secret checks, and fail-stop symbolization retain work
  that the scenarios require. Further changes must preserve the same
  bytes, refusals, reports, waits and ceilings. Request-table admission
  scans, LRU operations, KV duplicate-key scans and file observation,
  Huffman's reference sorting and the remaining fixture work also have
  identified costs; the measured candidates below the repair's roughly
  0.05 CPU s stop rule stay. The per-row instruction pairs, causes and
  requirements for a further fix are in
  `skein/docs/development/skein-limits-heavy.md`. Shared helper and JSON
  costs recorded there overlap separate repairs; their original counts
  are not measurements of what remains after those repairs.
- **Complete process-world instruction measurements** (2, 5, 7).
  Profiling the process operations needs a profiler that supports them;
  Callgrind's unsupported operations leave incomplete counts. A native
  CPU observation is no instruction comparison. Containment, observation,
  deadlines and the fixture starts each behaviour requires stay.
- **Process startup cost** (6, 7). Isolating the cost before a test's
  body needs separate measurements. Further table tests must still be
  cases of one behaviour under ../../foundation/testing-strategy.md, 8, keeping every
  assertion; a test past its guide cannot be merged away to make room.
- **Further folds and row diagnostics** (6, 7). Different behaviours and
  process-local heap measurements stay separate, with fixtures built
  per row wherever sharing could change what is shown. Signing-heavy
  App JWT routes stay separate when their combined table exceeds the
  unit-test guide. A table reports its first failing row and stops
  there (6): collecting every row's failure would catch a panic
  (../../foundation/programming-model.md, 10.4). Default: no
  fold weakens these rules to reach a suite-margin target; further cost
  changes need a profile of their cause first.
- **The counting allocator's remaining cost** (5). One thread-local
  access, its cells, layout construction and preconditions, allocation
  numbers, size conversions and System allocation remain necessary.
  Whether to add a `[profile.dev.package.skein-heap]` override for the
  remaining unoptimized work is the user's decision under
  ../../foundation/testing-strategy.md, 8. The instruction counts and each cause's share
  are in `skein/docs/development/world-costs.md`.
- **Deployment-world allocation and fork costs** (5, 6). Required checked
  allocation-header layout and the kernel's fork, wait and copy-on-write
  work remain. User-space instruction counts do not measure that kernel
  floor. `temper/docs/development/limits-worlds.md` records the shares
  and native system CPU; its different profiling arms do not establish
  an isolated remaining saving or a whole-suite total. A build override
  remains the user's decision under ../../foundation/testing-strategy.md, 8.
- **Costs beyond the shared world helpers** (5, 6). Product work, a
  world's own fixtures and process startup need separate cost work;
  `skein/docs/development/world-costs.md` gives their measured shares.
  Further JSON string work belongs with ../data/json.md, 8. Required scheduling,
  trace observations and Unicode matching remain; the account identifies
  no further removable helper gain of about 1% of a measured test.
- **Heap handed between services in one thread.** A world checks each
  service under its scenario's ceiling by metering its own calls and
  harness operations on its containers (simulator.md, 5), which holds
  while nothing one service allocates is freed by another: true over the
  kernel boundary, whose backend hands every buffer back. A world that
  joins two services' layers in-process, passing a box from one to the
  other, would need the hand-off counted.
- **The real loop in CI and sanitizers on the ring adapter** are the
  shell's (../runtime/shell.md, 9).
- **A fixture a crate's unit tests and a world both read** (6). No `#[path]`
  crosses a package, and a kit would give a second copy of the crate's types,
  so the dialect world holds its own
  (`skein/tests/forge/dialect/tests/common/admitted.rs`) beside the crate's. No
  feature can join them (../../foundation/cli.md, 3): a fixtures crate,
  `skein-forge-fixtures`, would hold the world's copy, as
  `skein-forge-connection-fixtures` holds the machine world's
  (../forge/connection.md, 10), for 55 lines of test data each side keeps as
  its own numbers. Default: the copy stays.
- **The echo's client host** (6): `ClientHost` in
  `skein/examples/echo/shell/tests/end_to_end.rs` repeats `Proc`'s `Host` arms.
  The test sits in the shell's package so that Cargo supplies its binary,
  `skein-echo-client` is held to the step subset and can't implement `Host`, and
  a kit's manifest alone is longer than the copy. Default: the copy stays.
- **Helpers beside the forge kit's** (6):
  `skein/tests/fake-peers/tests/forge/fixtures.rs` and
  `temper/tests/engine/protocol-forge/src/fixtures.rs` repeat some of
  `skein_forge_kit::fixtures`. Default: they stay, until a debt item names them.

## 10. The portable target

A playground runs a service's simulated world in a browser
(../playground.md), so what that world links builds for
`wasm32-unknown-unknown` as well as natively.

- **What builds there:** lib, io, the simulator, the shell's portable
  face (../runtime/shell.md, 13), the simulated world and the fake machine, the
  fakes a world hosts (the LLM's and the forge's domains and protocol
  layers, the peers, GitHub's layer), the key-value store, HTTP, JSON,
  the channel, the codec, TLS, and the playground's kit (../playground.md).
  The counting allocator, the harness's referee and real loop, the
  browser kit, the conformance suite, the fork kit and the scratch directory
  do not: they are tests', and native.
- **Fenced by target:** what reads the system is under `cfg(unix)` where
  it sits, never behind a feature (../runtime/shell.md, 13).
- **No TLS there.** A world draws every choice from its seed and runs
  in plaintext; on this target `skein-tls` refuses every configuration,
  session and signature (../net/tls.md, 4), so no path of a world reaches
  ring's randomness, and one that tries fails at once. The peers serve
  plaintext alone there.
- **Checked in the gate** with clippy's lints as errors on this target,
  over every crate of the list above: a native item reached from the
  portable face fails the build, and a fence that leaves code dead warns.
  Nothing runs there in the gate: the simulated world's behaviour is the
  same code natively, which its tests run; a playground's browser tier
  runs it in Chromium (../playground.md, 8).
