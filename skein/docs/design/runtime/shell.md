# The shell kit

Provisional, 2026-10-03, revised 2026-11-03. The design of
`skein-shell`: what a service's `main` runs its loop on. The kernel (the
io_uring backend of the kernel boundary, kernel.md, and the readiness
backend for hosts without it, macOS among them), the clock and the
seed; the loop itself, over a service's `Host`, and how it ends; what
startup reads, trust roots among it, and a terminal's modes. It is
ordinary Rust, and the only `unsafe` in skein that a service runs lives
here, in each system's module, `linux` (the ring adapter among it) or
`darwin`, and in `posix`, what the two write alike (`readiness.md`, 5). (The counting allocator, test-only, has the other:
../testing/layout.md, 1.)

## 1. In one page

- **One loop, written once.** skein provides the kernel (open, submit,
  reap), the clock, the seed, and `drive`: the loop of
  ../../foundation/programming-model.md, section 2, over a service's `Host` (section 11).
  The service brings its `iterate`, its startup and a per-iteration hook.
  skein's world harness hosts the same `Host`, so a world runs the shell
  as it ships.
- **The end is settlement.** The loop exits once the service holds
  nothing: everything it owns has reported closed and io is empty. Only
  io's deadlines bound that; no timer, grace or thread decides it
  (section 12).
- **The ring adapter makes no decisions.** It maps each record onto one
  submission entry, keeps the record in its in-flight table, and when the
  entry completes decodes the result and hands the record back.
- **One thread, one kernel floor.** The ring is set up for a single
  issuer, and assumes a kernel at or above one floor, with no checking for
  individual features; a probe at startup confirms the floor.
- **Two backends, one chosen at startup.** Linux takes the ring, and
  the readiness backend when io_uring is refused; macOS takes the
  readiness backend, which waits for readiness (epoll, kqueue) and makes
  each call without blocking. Step code never knows which runs
  (`readiness.md`).
- **Startup refuses rather than degrades:** invalid edge limits or a
  kernel below the floor stop the service before its loop runs. The one
  fallback, to the readiness backend, is said on standard error (`readiness.md`, 1).
- **Optimisations stay behind the records.** None changes code above io.
- **A binary's table is the shell's,** with its settings: its commands,
  flags and variables, each in its set, and the help, version line and
  reference documents rendered from it, in one format for every tree's
  binaries (`shell-table.md`, section 1; `../../foundation/cli.md`, 6 to 8).

## 2. In skein

`skein-shell` depends on lib, io (for the records), and the only crates
from outside: `io-uring`, on Linux alone, and `libc`, on Unix alone. A
service's `main` depends on it and on the service's own `iterate`. It
builds for Linux and for `aarch64-apple-darwin`, and for
`wasm32-unknown-unknown` as its portable face, `Host` and a command's
settings with nothing that reads the system (section 13); any other
target is a compile error naming the three (`readiness.md`, 5, 6).

- **`Host` and `drive` are here** (section 11), and `skein-world`
  re-exports both. A service's shell then links no testing crate,
  and its worlds host the shell's own `Host`.
- **It reads trust roots as bytes** (`shell-startup.md`, 1.2). `skein-tls` turns them into its
  configuration, so the shell needs no TLS crate.

## 3. The kernel

- **Open, submit, reap.** `Kernel::open` takes a size, the most operations
  in flight at once (cancels included, as io's operation slab counts
  them); both rings are sized from it, so the completion queue cannot
  overflow, and submit takes no record past it. Submit pushes a queue of
  records and enters the ring once; reap moves completions into a queue,
  up to its room.
- **Waiting is the loop's call.** Submit takes how to wait: not at all
  (work is pending above), until a monotonic deadline (the earliest over
  every layer), or until a completion arrives (no deadline armed, which
  asserts that something is in flight). Timers are not operations: this
  one timeout covers them all.
- **Set up for one thread:** a single issuer, with deferred task running,
  so completions are processed only when the loop enters the ring.
- **The in-flight table** has one slot per operation, allocated once at
  its final capacity and never moved while anything is in flight. A slot
  holds the record and the kernel structures its operation needs, which
  never leave the module. An operation's `user_data` is its slot's index
  and generation, so a completion or an async cancel can never name the
  slot's next operation.
- **Paths are the slot's own.** A record's path or name has no NUL; the
  slot holds the NUL-terminated copy the kernel reads, made at submit and
  freed with the slot, beside the `open_how` an `Open` hands over and the
  `statx` buffer a `Stat` reads back.
- **Synchronous operations run at submit.** A record that is not a ring
  operation (listing a directory: `getdents64` has none; locking a file:
  `flock` has none, and `Lock` never waits; spawning; signalling;
  reading the process's resource usage) is performed when it
  is submitted, and its completion waits in the table's ready list for
  the next reap, so io cannot tell it from a ring operation (kernel.md,
  6). A `List` reads with `getdents64` into one buffer of the table's,
  copies out what fits the record, and sets the directory's position back
  to just past the last entry it took. A `Usage` reads `getrusage` twice,
  for the process and for its reaped children (kernel.md, 6.3).
- **The `unsafe`, and why it is sound,** is stated in the ring adapter's
  module documentation, case by case: the pointers handed to the kernel,
  entering the ring, the socket address casts, the synchronous calls, and
  dropping a kernel with operations in flight, which leaks the table so
  that the kernel never writes into freed memory.
- **Signal senders** cross the kernel boundary as an opaque `kernel::Sender`,
  compared only for equality. Linux's `read_signal` wraps signalfd's
  `ssi_pid` when it is nonzero in `Signalled::Termination { signal, sender }`.
  A successful spawn returns the child's identity in `Done::Spawned` beside
  its pidfd: Linux obtains it from the unreaped pidfd with `pidfd_getpid`,
  and Darwin wraps the pid returned by `posix_spawn` or `fork`.
  io keeps the identity only until the child's `Exited` event (io/processes.md and io/README.md, 4).
- **A child-side exec,** `exec_dying_with_parent(command, parent, signal)`,
  installs Linux's parent-death signal, re-checks the parent PID, then
  execs, returning only on failure. The signal survives exec of a
  non-setuid binary and is cleared by a setuid/setgid exec. It fires when
  the parent's **thread that forked the child** exits, rather than when
  the parent process exits; with io, that thread runs the shell's kernel
  and lives as long as the loop. `parent` must be the PID the caller's
  parent gave it (for example as an argument), so the re-check detects
  death before the signal was installed. macOS, which has no such
  signal, forks a watcher before the exec that sends it when the parent
  **process** exits (`readiness.md`, 5); the re-check is the same.
- **A parent-side spawn,** `spawn_dying_with_parent(command: &mut
  std::process::Command, signal) -> std::io::Result<std::process::Child>`,
  for a test kit with no binary of its own to exec through
  (`skein/docs/design/testing/service-tiers.md`, 1's "No fixture outlives its
  test"): the same parent-death signal and parent re-check, installed in
  the forked child before its exec, the parent being the spawning
  process. A re-check that fails refuses the spawn as the child-side
  exec does. On Linux the signal fires when the spawning thread exits,
  as above, so a kit spawns on the thread that keeps the process; on
  macOS the watcher, forked in the child's pre-exec hook, sends it when
  the spawning process exits.
- **Its checks:** the first two broken invariants of the kernel boundary
  (an invalid record, a token already in flight) at submit.

## 4. The kernel floor

- **One floor, no feature checks.** The first floor is 6.12, an LTS
  release that has every ring operation io uses: for sockets, and for
  files `OPENAT2`, `READ`, `WRITE`, `FSYNC`, `STATX`, `RENAMEAT`,
  `UNLINKAT`, `MKDIRAT` and `LINKAT`, the newest of them from 5.15. An operation
  newer than the floor stays synchronous until the floor moves: making a
  pipe, for example, has been a ring operation only since 6.16; listing a
  directory has none at all. Lowering the floor for a deployment that
  needs it means making the operations newer than the new floor
  synchronous, and io cannot see that change.
- **The probe at startup** checks the ring for every operation it uses. It
  confirms the floor; it does not choose between features. If an
  operation is missing, or the ring cannot be set up at all (io_uring
  missing, disabled, or refused by a seccomp profile), the shell refuses
  to start, unless io_uring was refused outright: then it takes the
  readiness backend (`readiness.md`, 1), which confirms the floor from the kernel's
  release (`readiness.md`, 6).

## 5. The clock and the seed

- **The clock gives two times,** read together once per iteration:
  `CLOCK_MONOTONIC`, for every deadline (the kernel's wait deadline is on
  this clock), and `CLOCK_REALTIME`, for things about the world. The loop
  hands both to the steps in `Env`; `Now::with(limits)` makes the `Env` of
  one reading for a step with those limits.
- **The seed** comes from `getrandom`, once, at startup; everything random
  after it is drawn from the layers' state. It fails, and the service
  does not start, when the kernel refuses `getrandom`.
- **The cryptographic seed,** for a service whose layers hold a
  `CryptoRng` (lib.md, section 8): 32 bytes, drawn from `getrandom` once
  at startup, beside the seed and never derived from it, and handed to
  the service's startup as a `Secret`. The service keys each `CryptoRng`
  from it; a world draws it from its own seed instead.
- **A fixed clock,** for a process a tool walks on a fixed start
  (lib.md, section 8, `FixedStart`): `Clock::fixed(start: &FixedStart)
  -> Clock` reads `FixedStart::NOW` and the start's wall, then moves only
  to the deadlines it is asked for, through a `FixedClock` over
  `CLOCK_MONOTONIC`'s readings. `Clock::until(&self, deadline:
  Option<Time>) -> Option<Time>` gives the monotonic instant the kernel
  waits until for a step's deadline: the deadline itself on the host's
  clock, and on a fixed one the instant the host's time reaches it
  (`FixedClock::until`). `Clock::new` is the host's clock, as before, and
  a wait of the shell's own, `posix::wait_readable`'s, keeps it. A process
  on a fixed start takes its seed and its cryptographic seed from the
  start (`FixedStart::draw`), not from `getrandom`: the program's startup
  chooses both, and the kit reads no setting of its own for them.

## 6. Startup

Moved to [`shell-startup.md`](shell-startup.md), section 1.

## 7. Deferred optimisations

Each sits behind the same records, keeps the contract of the kernel
boundary (explicit backpressure, one completion per operation, a
completion queue that cannot overflow), is measured before it goes in,
and changes no code above io:

- multishot accept and receive with a provided-buffer ring, so idle
  sockets hold no buffer;
- registered buffers; fixed files (kernel.md, 9); zero-copy send; linked
  operations;
- exact-size receives for a fill demand: io allocates the `Box`, the
  kernel reads straight into it, and the copy out of the intake
  disappears;
- shared immutable buffers (`Arc<[u8]>`, confined to io) for large values
  sent to many slow readers, or zero-copy relaying;
- batched submits: submitting every few rounds, or when about to wait,
  which saves syscalls under pipelining at the cost of delaying each
  submission by up to that many rounds. The loop reaps and submits
  every round: going round costs a completion-queue read, a clock read,
  and a ring syscall only when there is something to submit or the kernel
  has completions to flush, and reaping every round keeps one peer's
  backlog from holding up other connections' completions and timers;
- kernel TLS (../net/tls.md);
- in the readiness backend, a descriptor's interest kept for its life,
  edge-triggered (`EPOLLET`, `EV_CLEAR`), rather than registered and
  removed with each waiter (`readiness.md`, 2): it saves two calls a wait, and asks
  every record to be attempted before it waits.

## 8. Testing

- **The ring's own tests** (`tests/ring`) run on the real kernel, on
  loopback and in a scratch directory: its slots and waits, a large
  transfer through short sends, a large file through its reads and
  writes, what it completes itself (a `List` among them), a listing's
  position handed from one `List` to the next across its own buffer, a
  root opened at startup, and dropping it with operations in flight. The
  invariants it and the readiness backend assert, their guards'
  included, are untested (../../foundation/programming-model.md, 10.4).
- **The conformance suite** runs against each backend the host has,
  each scenario once (`tests/conformance/real`; ../testing/conformance.md, 1): one test
  binary, a module for each backend over one list of cases, `ring`
  (Linux only, opened with `Backend::Ring`) and `readiness` (opened with
  `Backend::Readiness`, so epoll on Linux and kqueue on macOS).
- **The readiness backend's own tests** (`tests/readiness`), on each
  system, on loopback, pipes and a scratch directory: a record made at
  submit and one that waits, then completes on its readiness; a
  readiness another reader took first, which waits again; a large
  transfer through short sends; a blocking pipe it did not make, read
  once readable and written in `PIPE_BUF` bytes at most; a descriptor
  shared with a child, closed, and its number reused, with no readiness
  of the old one reported; a descriptor the wait cannot watch, its call
  made at once; a signal read through a blocking signalfd; a cancel of a
  waiter and one too late; a deadline met without waking early again
  and again; and dropping it with records waiting.
- **The choice at startup,** in the shell's own tests, below the kernel:
  each refusal of the ring's open, as a table, falling back or returned
  (`readiness.md`, 1); each release, as a table, at the floor, above it, below it and
  unreadable (`readiness.md`, 6); and `chosen`'s line for each. On a host with
  io_uring, `Kernel::open` takes the ring and says so. A host that
  refuses io_uring is not made here: a seccomp profile or
  `io_uring_disabled` takes privileges, or `unsafe` outside the shell.
  macOS takes the readiness backend through `Kernel::open` itself.
- **The real loop** exercises the rest: the probe at startup, the signal
  path, and the ring adapter's `unsafe` under a sanitizer
  (../../foundation/testing-strategy.md, 2.8). The real world opens the host's backend,
  or the one a test forces (`skein_world::real`), and the echo's
  real-loop story runs on each: its smoke on the readiness backend,
  forced, beside the ring's.
- **`drive`'s own tests,** in the real loop, over scripted hosts: one that
  ends by itself returns its exit at once; one that waits on a keep time
  after its last word fails the teardown check (../../foundation/testing-strategy.md, 6);
  a termination signal while it closes aborts what is still closing; the
  hook runs once per iteration, after `iterate`.
- **Startup reads** (`tests/ring`): a whole file within its maximum, one
  past it, a FIFO and a directory refused; a PEM bundle with certificates,
  other blocks, a certificate that does not parse, and malformed blocks
  of each kind (invalid base64, unterminated, ended by another label,
  with headers), each skipped and counted while the rest are taken; a
  bundle past each bound refused; a DER file. `directory_access`: a
  writable directory, an absent path, a file, a directory without write
  permission for the user, each answered as such, and nothing created
  or changed by any of them.
- **A command's settings** (the crate's own tests, no kernel): flag and
  variable precedence, empty variables, literal and XDG defaults, refusals,
  argument order and `--`, byte values, and help agreeing with refusals.
- **Terminal modes,** end to end under a pseudo-terminal
  (../../foundation/testing-strategy.md, 2.9): input read a key at a time, the modes found
  again once the binary exits, and a size change heard.
- **The Darwin check.** On our Linux machines, clippy runs over every
  target of the workspace for `aarch64-apple-darwin` with warnings
  denied, as the gate's clippy does; clippy's pass is `cargo check`'s
  too. Nothing links, and no file of Apple's SDK is on our machines. The
  members that reach rustls (whose `ring` crate compiles C against
  Apple's headers) are excluded there by name; they are checked on the
  macOS runner. Darwin's code is built
  here and run there.
- **Linux's own tests, by name.** A test that only Linux can run is
  built for Linux alone, `cfg(target_os = "linux")` on its whole binary
  or module, with its reason beside it, and listed here; none is dropped
  silently. Every other test runs on both systems.

| Test | Its reason |
|---|---|
| `tests/ring` (`skein-ring-tests`): `ring` and `files` but its `startup` module | the ring adapter's own |
| `tests/conformance/real`: the `ring` module | the conformance on the ring |
| `tests/io/tests/suite/ring.rs` | io's real-kernel files on the ring |
| `tests/echo/tests/suite/real.rs`: the ring's cases | the echo's real loop on the ring |
| `crates/skein-shell`: `linux`'s and `linux::ring`'s unit tests | Linux's calls: epoll, signalfd, pidfds, the ring |
| `crates/skein-shell`: `kernel`'s `the_host_chooses_the_ring` and `forcing_a_backend_reports_it_and_refuses_zero_capacity` | each opens the ring |
| `crates/skein-shell`: `readiness`'s `a_registration_error_completes_without_calling_or_adding_a_waiter` | its error is epoll's second add (`EEXIST`); kqueue's `EV_ADD` modifies instead |
| `tests/world/tests/suite/end_to_end.rs`: the cgroup's and the walk's tree tests, an escape from the group among them | a cgroup, the subreaper and `/proc` are Linux's (../examples.md, 6) |
| `tests/world/tests/suite/end_to_end.rs`: `supervised_stderr_overflow_is_observed_before_the_referees_term_grace_and_whole_tree_kill` | its descendant leaves its session and group through `setsid`; a cgroup or the subreaper's walk follows it, Group does not (../examples.md, 6) |
| `testing/skein-throwaway-forgejo/tests/suite/throwaway.rs` (`skein-throwaway-forgejo`) | starts the pinned Forgejo binary; no owned Forgejo on macOS, its releases are Linux's |

  The list grows with each test that a Darwin build shows to be
  Linux's, in the commit that marks it.

## 9. Open questions

- **A child dying with its parent on macOS** (3, 13): the watcher
  (`readiness.md`, 5) is written without a macOS machine to run it, so
  whether a consumer that would start every program through it takes it
  before a macOS run has shown it. Default: no; smith's agent still
  starts its programs directly on macOS
  (`smith/docs/design/protocol/programs.md`, 5.1), and jig-test's
  program-child test (`jig/testing/jig-test/tests/process.rs`) is the
  watcher's first proof there.
- **The remaining identical helpers** (`readiness.md`, 5): `linux` and `darwin` each
  keep `count`, `process::fd` and `files::truncate`, whose only differences
  are a panic's words or a path. Moving them to `posix` waits for a round
  that names them; the shared-call rule still applies to them.
- **The real loop in CI:** a kernel at the floor, with io_uring allowed by
  the CI's container profile.
- **Sanitizers on the ring adapter:** Miri cannot run the ring, so which of
  the address and leak sanitizers the real loop runs under.
- **A hashed certificate directory** (`SSL_CERT_DIR`) as the machine's
  trust roots, on a machine with no bundle file: read it, or refuse to
  start, as the shell does (`shell-startup.md`, 1.2).
- **An inherited listener under the readiness backend** (`readiness.md`, 3): whether
  the kit makes a listener it was given non-blocking, which the shared
  description makes non-blocking for every process that holds it.
- **A terminal suspended and resumed** (`SIGTSTP`, `SIGCONT`): whether
  the shell restores the modes before the process stops and sets them
  again when it continues, which takes a signal the shell handles while
  the loop runs (`shell-startup.md`, 1.3).
- **Listening, on macOS** (`shell-startup.md`, 1.6): whether anything reads it another way,
  such as `TCP_CONNECTION_INFO`'s state. Default: no; only a parent's
  mistake reaches it, and that shows at the first accept.
- **A service that names `keeper`** (`readiness.md`, 5): the fence is a public module's name,
  and no check refuses a service that names it; `temper`'s `fake` module, a test
  peer in a shipping crate, does name it. Whether a gate step holds the fence.
  Default: no; a service has no use for the module.
- **A table's dispatch** (15): whether the shell matches a binary's
  arguments to its table's commands, a `Reach::RunAs` row by the name the
  binary runs as, or each binary's crate does. Default: each binary's
  crate, until temper's first tables
  (`temper/docs/design/draft/cli.execution.md`, C2) show what they share.

## 10. Not built yet

What of this document is not built is in [`shell.status.md`](shell.status.md).

## 11. The loop: `Host` and `drive`

The loop of ../../foundation/programming-model.md, section 2 is written once, here, not in
every service's `main`.

- **`Host`** is what the loop needs of a service, and what skein's world
  harness needs of a process it hosts: one trait, so a world runs a
  service's shell as it ships. A service's shell implements it over its
  `iterate`:
  - `iterate(now, wall)`: one turn, both passes and the reclaim point;
  - its completion and submission queues;
  - `work_pending(now)`, and `next_deadline()`, its earliest over every
    layer;
  - `next_policy_deadline()`, its earliest deadline other than io's own,
    the mechanics' (io/README.md, 2): its close and retry deadlines, and the
    stall deadlines owners state for files. It is the one the teardown
    invariant watches (../../foundation/testing-strategy.md, 6);
  - `is_empty()`, when it holds nothing, and `exit()`, its exit status
    once it does;
  - `operations()`, the size of its ring;
  - `drain()`, the hook below, which does nothing unless the service
    gives it something to do;
  - `state_digest()`, a digest of what it holds, for replay (lib-containers.md,
    1.1): `None` unless the service gives one. Only a world reads it
    (../testing/service-tiers.md, 1); `drive` never calls it.
  `Host` is portable: it builds for `wasm32-unknown-unknown`, where a
  world hosts a service with everything handed to it (`shell.md`, section 13).
- **A host declares no heap ceiling or worst case.** The simulated
  and real worlds meter each host independently of this interface.
  A scenario may check its measured peak under a named ceiling with
  headroom; the outcomes report the measured peak alone
  (../../foundation/testing-strategy.md, 6; ../testing/service-tiers.md, 1).
- **`drive(kernel, clock, host)`** is the loop: reap, read the clock,
  iterate, drain; then return the host's exit once it holds nothing
  (section 12), or else ask the clock's `until` for the earliest deadline
  (section 5), at every iteration, and submit, waiting not at all while
  work is pending, until the instant `until` gave, or until a completion. A service's `main`
  is its startup, then `drive`. It is native, as the kernel and the
  clock it takes are (section 13).
- **The hook** runs once per iteration, after `iterate` and before the
  submit, while the service is a frozen snapshot. It is for drains that
  belong to the shell: reading what the service holds for its operator
  and saying it once, such as the address it listens at. It is shell code
  under the shell's rules (../../foundation/programming-model.md, 2.1): a short line to
  standard error is the most it writes, and it writes no file and starts
  no thread. Output that is large, or must be complete, such as a trace,
  is an io stream instead (io/files.md, 1). The world harness calls the hook
  at the same point, so the drains a world runs are the ones that ship
  (../testing/service-tiers.md, 1).

## 12. The end

- **The loop exits when the service holds nothing:** every slab empty,
  nothing in flight, every queue empty, and io holding no entity. `drive`
  then returns the service's exit, and `main` returns it. It does not
  exit at a signal, at an answer or at a timer: after its last word, a
  service closes everything it owns (../../foundation/programming-model.md, 5.2), and the
  loop ends when that has settled.
- **Only io's deadlines bound it.** A service's teardown bound is io's
  close timeout, the write deadline of a stream it still writes after its
  last word (io/files.md, 1), and the moment its cancels take to settle;
  nothing else in it waits on time once its last word is out. A
  supervisor derives its grace from that bound, and its grace exceeds
  it.
- **A signal while closing aborts.** A termination signal arrives as io's
  `Shutdown` at any time (io/README.md, 4). Before the last word, the service
  decides what it means; after it, the remaining graceful closes become
  aborts, and the loop ends as soon as those settle. A second signal
  means the same as the first.
- **Nothing outlives the loop.** There is no thread to join, no buffer to
  flush and no grace timer: what the service wrote went through io, which
  has settled it, and the kernel is dropped with nothing in flight. A
  service never ends with `std::process::exit` while it holds anything.
  After `drive` returns, `main` only undoes what startup set with the
  kit, such as a terminal's modes (`shell-startup.md`, 1.3), and writes nothing to the
  service's files.
- **What the process used,** for a service that reports it at its end:
  the kernel's count of CPU time and peak resident size, for the process
  and for the children it has reaped. The service reads it through io
  (io/processes.md, 1), a `Usage` the kernel answers at submit, after its last
  child has closed, since io reaps a child only at its close, and before
  the stream it reports on closes. After `drive` returns no stream is
  open to report on.
- **A supervisor's kill is a backstop** for a process that does not end:
  a bug, or an operation the kernel will not interrupt (../../foundation/notes.md). It is
  never how a healthy process ends.
- **A crash closes nothing.** A process killed, or aborted by a panic
  (`shell-startup.md`, section 1), runs none of this end: the children it started stay in
  their own groups (io/processes.md) and outlive it, since io kills a child's
  group only when it closes the child. A child that execs its
  program through `exec_dying_with_parent` (`shell.md`, section 3) is the exception: it
  is sent its signal when the thread that started it ends, on Linux, or
  its process, on macOS, so a service whose crash must not leave a child
  running starts it that way.
  A supervisor that restarts a crashed service first ends its whole tree,
  every descendant at once, as a systemd unit's control group is by
  default, and starts it again on the same state only once nothing of
  the old tree runs. A
  descendant still running at the restart is the supervisor's failure,
  not the service's.
- **Checked** in every world, by the teardown invariant
  (../../foundation/testing-strategy.md, 6), and in the real loop by `drive`'s own tests
  (section 8).

## 13. The portable face

A service's layers take everything from their host as input: time and
wall, the seed, descriptors, configuration's bytes, roots. Only the shell
reads the system to hand them over. So the shell has two faces, chosen
by the target, and a world built for a browser hosts the same `Host`
its native worlds do (../testing/world.md; ../playground.md).

- **Portable:** `Host` (section 11), a command's settings (`shell-startup.md`, 1.5):
  `Settings`, `Setting`, `Fallback`, `Base`, `Value`, `Resolved`,
  `Origin`, `Refusal`, `Reason`, `resolve` and `help`, and a binary's
  table and its rendering (`shell-table.md`, section 1), which take the arguments and
  the environment as bytes and read nothing. They build
  for `wasm32-unknown-unknown`, the portable target, and on it they are
  the whole crate.
- **Native, under `cfg(unix)`:** everything that reads the system or
  waits on it: `drive`; the kernel and its backends (`shell.md`, section 3 and `readiness.md`);
  the clock and the seed (`shell.md`, section 5); startup's reads, roots and outputs
  (`shell-startup.md`, section 1); the process, group and signal calls; the terminal's modes
  and input (`shell-startup.md`, 1.3); and settings' `arguments` and `environment`, which
  read the process's.
- **Fenced by target, not by feature.** The target is the fact: a
  feature would unify across the graph, and every native build would
  have to opt in. A native item is fenced at the coarsest grain that
  holds it: a module in its `mod` line, an export in its `pub use`, an
  item in a portable module (`drive`, `arguments`, `environment`) on its
  own. `libc` is a `cfg(unix)` dependency.
- **No third system module.** The portable target has no system calls to
  select, so `system` stays Linux's and Darwin's (`readiness.md`, 5); Unix other than
  Linux and macOS, and every target but these three, stay a compile
  error.
- **No consumer changes.** A world's closure names only `Host` of the
  shell (the fakes, the simulated world, a service's composition that a
  playground runs); a service's `main` names the rest, and builds only
  natively.
- **Checked** in skein's gate: clippy over the shell and the simulated
  world for `wasm32-unknown-unknown`, warnings denied, so a native
  item reached from the portable face fails the build, and a fence that
  leaves dead code warns (../testing/README.md, 10).

## 14. A binary's table

Moved to [`shell-table.md`](shell-table.md), section 1.
