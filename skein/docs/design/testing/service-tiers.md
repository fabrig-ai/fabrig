# Testing skein: what skein supplies for a service's tiers

## 1. What skein supplies for a service's tiers

| Service tier | Real | What skein supplies |
|---|---|---|
| step tests | one of its step functions | lib |
| domain worlds | one domain or child domain | lib, the counting allocator; for an application on skein-ui, the fake person (../ui/testing.md, 1.1) |
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
(fakes/forge.md, sections 2 to 3, and fakes/forgejo.md, section 1, and fakes/forge.md, section 5, and fakes/throwaway.md, section 1). The peer serves plaintext on loopback
over a forge the service may share with its own fake of git. Its
transcripts of git stay its own.

**The counting allocator** is `skein-heap`, in `testing/` (`layout.md`, section 1): a
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
  `skein-sim-world`, for `wasm32-unknown-unknown` too (`README.md`, section 10).
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
  - A run's finding scope is its thread's, entered as the run starts
    and left as it returns. Inside a run, a binary's teardown keeps its
    finding for the run to return after its forced cleanup; outside
    every run, the teardown fails with it at once. A real run started
    while another is running on the same thread, as a story's scripted
    step that runs a command's own world to its end before it answers,
    is nested: the outer run waits while the inner one runs, the inner
    run keeps and returns its own findings, and a finding the outer run
    kept before the inner one started is still the outer run's after it
    ends. Leaving a scope drops what its run kept but didn't return, so
    no cleanup's later finding reaches another run, nested or next.
  - The domain world's referee, `skein_world::domain::Referee`, holds its
    verdict as a value (`verdict()`). `check_holding(seed)` and
    `check_passed(seed)`, each `-> Result<(), String>`, give the text
    `assert_holding` and `assert_passed` panic with. `Ledger::check_settled()
    -> Result<(), String>` and `domain::check_replays(seed, other, run)
    -> Result<R, String>` are the returning forms of `Ledger::assert_settled`
    and `domain::assert_replays`, which panic on them.
    `Ledger::try_open(key, value) -> Result<(), String>` and
    `Ledger::try_end(key) -> Result<V, String>` return a second open of a
    key in flight and an end of one not in flight as findings, in the
    words `open` and `end` fail with now; `open` and `end` keep their
    signatures and panic on them. A referee's contracts built on a ledger
    return its findings to the referee's `watch`, as oauth accounts' do.
  - The heap's checks are the run's findings. A process past its
    scenario's ceiling, at an iteration, a restart or its construction,
    and a process ended in the run that frees other than it held of its
    own, are findings that `World::try_run*` and `real::World::try_run`
    return, in the text they fail with now; a construction's is kept from
    `spawn` and returned as the run starts. A settled world's hosted child
    whose output was not read to its end is one too
    (`Sim::check_hosted_settled`, simulator.md, 5). A settled process's
    leak is found as its outcome drops it: `Outcome::try_release(self)
    -> Result<(), String>` drops each process and returns the first
    leak's finding, and an outcome dropped without it fails with that
    finding, as it does now. A binary's output past its scenario's limit,
    where its expectation measures or ends with it, is kept as the
    binary's finding, as an expected end's is, and the run returns it.
  - `skein_heap::Meter::try_check(&self, step: Measured, bound: u64, what:
    &dyn Debug) -> Result<u64, String>` returns the most the step held of
    its own, or, past `bound`, the finding `check` fails with now, `{what:?}:
    {own} bytes held at the peak of a step, more than the worst case of
    {bound}`. `Meter::check` keeps its signature and panics on it. A step
    checked out of turn stays an assertion of both, the caller's bug.
  - A kit's own checks of the code it plays against return their findings
    the same way: the fake echo client's check of its peer (../examples.md,
    5), the minimal fake machine's check that a handle is one it issued and
    still holds (`README.md`, section 4), and the LLM connection's close judge
    (../llm/connection.md, 8). Their callers panic on them; their tests
    read them.
  - A harness that adds context to a failure writes it before it runs
    what may fail: conformance's simulated driver names each seed and
    configuration on stderr before running it, and catches nothing.
    What it writes names the case, never its input: a seeded sweep's
    line gives the seed, the case (its kind, number or mutation) and the
    input's length, and the assertion that fails names the input in its
    message, which the runner prints. A sweep runs thousands of cases,
    and their inputs written whole would outweigh the cases themselves.
- **It names each kill in the real loop.** A hosted child the real
  world kills, at a `Kill` some host's `Op::Signal` sent it, is
  recorded once its operations and descriptors settle, in the order
  they settle, with who it was, who killed it, when and why:

  ```rust
  pub struct Killed {
      /// Its admission index.
      pub host: usize,
      pub exit: Exit,
      /// When the kill was submitted: the iteration's `now`, on the clock
      /// `Outcome::start` and `end` read.
      pub at: Time,
      /// Its program and its arguments' first word, as admitted (lossy UTF-8).
      pub program: String,
      pub argument: Option<String>,
      /// The host whose signal killed it, and that host's program as
      /// admitted; `None` for a root.
      pub requester: usize,
      pub requester_program: Option<String>,
      /// The signal's reason (../runtime/io/processes.md, 5).
      pub reason: Reason,
      peak: Option<u64>,
  }
  ```

  Its `Display` is one line, with the reason in words:
  `host 158 (/s/agent/temper-agent run) killed at 7134000000 ns by host
  1 (a root): forced after its grace; exit Signal(9)`. The argument is
  left out when there is none, and a hosted requester shows its program
  where a root shows `a root`. The reasons' words are `no reason given`
  (`Unknown`), `asked for` (`Asked`), `closed while running`
  (`Closed`), `forced after its grace` (`Grace`), `at its deadline`
  (`Deadline`) and `cancelled` (`Cancelled`). A test that fails on a
  kill it didn't ask for prints its records with it. The simulated
  world's `Killed` stays as it is. The records, their fields and their
  words, are checked without a kernel, on the real loop's own state with
  the admissions, the kill's clock reading, and the cancelled records
  and cleanup completions handed in: none is made before they settle,
  and each is stamped with its kill's submission, not its cleanup.
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
- **A bound only a slower host misses doubles on macOS,** as nextest's
  slow timeouts do (`temper/docs/design/testing/macos.md`, 7):
  `skein_world::real::host_bound(linux: Duration) -> Duration`, a
  `const fn`, is `host_bound_on(linux, cfg!(target_os = "macos"))`, the
  pure reading Linux's tests pin. A world opts in where it sets its
  patience; `Deadline::arm` and `real::World::try_run` take the bound
  given, and no budget or limit is scaled.
- **macOS has no schedstat:** the loop thread's CPU there is
  `skein_shell::keeper::thread_cpu() -> Option<Duration>`
  (`CLOCK_THREAD_CPUTIME_ID`, on both systems), its runnable wait
  unknown, so no wait there is starved. The settle line reads `thread CPU
  elapsed (thread clock): N ns; thread runnable wait elapsed: UNAVAILABLE
  on macOS`; Linux's lines are unchanged.
- **A binary's counts mark tree settlement.** `Binary::counts()` is
  present once the binary and its descendants settle, whatever stderr's
  state. Stderr EOF and descriptor closes still drain before its observer
  is empty. A cgroup is removed only after `populated 0`; after
  `cgroup.kill` the keeper waits within a 2 s deadline. A group retaining
  a leader is left for process exit. Removal errors while unwinding are
  reported without raising another failure.
- **No fixture outlives its test.** A fixture that would run until
  signalled ends with the test that started it however the test ends,
  a runner's kill that runs no cleanup among them: its leader dies with
  the thread that started it on Linux, and with its process on macOS (a
  parent-death signal or its watcher, `exec_dying_with_parent`,
  ../runtime/shell.md, 3), each descendant
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
- **It carries state digests** (../runtime/lib-containers.md, 1.1). `Host` (../runtime/shell.md, 11) and
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
