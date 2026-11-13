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
../testing/README.md, 6.)

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
  binaries (section 14; `../../foundation/cli.md`, 6 to 8).

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
- **It reads trust roots as bytes** (6.2). `skein-tls` turns them into its
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
  death before the signal was installed. macOS returns `Unsupported`.
- **Its checks:** the first two broken invariants of the kernel boundary
  (an invalid record, a token already in flight) at submit.

## 4. The kernel floor

- **One floor, no feature checks.** The first floor is 6.12, an LTS
  release that has every ring operation io uses: for sockets, and for
  files `OPENAT2`, `READ`, `WRITE`, `FSYNC`, `STATX`, `RENAMEAT`,
  `UNLINKAT` and `MKDIRAT`, the newest of them from 5.15. An operation
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

## 6. Startup

Before its loop, a service's `main`, with the kit:

1. reads its configuration, the limits of every layer among it;
2. validates the edge limits and refuses formulas that overflow; there
   is no startup memory fit check (../../foundation/programming-model.md, 6.3);
3. blocks the termination signals that io will read, so they arrive as
   `Shutdown` events (io/README.md);
4. opens the first roots for io's files, from configuration, with
   `open_root` (io/files.md; kernel.md, 6.1), which makes nothing; a root
   that may be absent, such as a state directory, with `make_root`,
   which makes each absent directory along its path with the mode given,
   syncing the directory each lies in, before it opens the last, so that
   a root the service relies on survives a crash. A root, a workspace,
   belongs on a filesystem mounted `nodev`: the ring refuses to open a
   device beneath it, but only after the device's own `open` has run. It
   opens the files it appends to beneath them, a trace among them, which
   io adopts as streams once the service is made (io/files.md, 1);
5. resolves the configured peer host names into socket addresses
   (io/README.md, 3), and hands on an in-process name as configured,
   unresolved;
6. reads certificates, keys and root stores for TLS: the machine's trust
   roots, or a DER file (6.2; ../net/tls.md);
7. reads the seed and opens the kernel.

Startup is each service's `main`, step by step, with the kit's
functions: the kit holds no startup function of its own.

A service that starts trees also dispatches its helper before step 1,
and opens its own cgroup before step 7 (6.4).

A service that listens takes the listeners it was given, and opens
the ones it holds for its children, before step 7 (6.6).

What startup reads, its configuration, its secrets and certificates, it
reads with the kit (6.1). Then `main` makes the service and calls `drive`
(section 11).

A panic is fail-stop, and a supervisor restarts the process. Before
step 1, `main` installs `skein_fail_stop`'s hook, which aborts on a
panic in a build that unwinds (../../foundation/programming-model.md, 2). Its report
carries the panic's backtrace, once, whatever the environment says
(`RUST_BACKTRACE` set or not): an abort in a deployment is often seen
once, in a log, and its frames are what says where it was. The report
is its thread's `panicked at` line, with the panic's location, then
its message, then the backtrace, and it closes with that line and
message again: a supervisor keeps the last bytes of what a process
wrote, and a backtrace longer than them would otherwise push the
message out of what it keeps.

### 6.1 Reading files at startup

Startup runs before io, so what it reads it reads with the kit, never
through a `File` (../../foundation/programming-model.md, 2.1). `read_file` reads one whole
file, up to a stated maximum, through the ring adapter's own calls: it
opens without blocking and never as a controlling terminal, checks that
what it opened is a regular file, reads it, and closes it. It answers the
bytes, or why not: absent, not a regular file, past its maximum, or the
kernel's error. Once the loop runs, files are io's (io/files.md).

`read_private_file` reads a secret that configuration names (a key, a
credential, a digest) the same way, within the same maximum, and makes
io's private-file checks (io/files.md, 3) on what it opened before it reads a
byte:

- a regular file, reached through no link: no component of its path is
  one;
- no group or other permission bits;
- one link;
- owned by the process's effective user, which startup reads once
  (`effective_user()`, below).

Anything else is refused unread, and the service does not start: the
refusal says what was unsafe, as io's `Unsafe` names it (a link, the
kind, the bits, the link count, the owner). A file another user keeps
private fails its open for want of permission; the read names it as io
does, by a path-only open whose stat decides nothing (io/files.md, 3). The
bytes are a secret's: the caller keeps them out of logs, traces and
`Debug` (lib.md, section 6).

**The process's own identity** is read the same way, once, at startup,
through the ring adapter:
- `effective_user()`, the effective user ID, for io's private files
  (io/files.md, 3);
- `process_id()`, the process ID, for a service that reports it, such
  as smith's `session.started`.

Neither is read again: a process's identity does not change while it
runs. A service never reads them itself, with `std::process` or a
syscall of its own. It takes them from its shell.

**Whether a directory can be written** is asked the same way, at
startup only, for a service that confirms a directory something it runs
will write, such as smith's writable caches. `directory_access(path)`
answers, through the ring adapter, as the kernel answers for the
process's effective user, its groups, the directory's ACLs and a
read-only mount: whether `path` is a directory that user may write and
search (`faccessat2` with `AT_EACCESS`, and `W_OK | X_OK`). It answers
writable, absent, not a directory, or denied with the kernel's error. It
creates nothing, opens nothing for writing and changes nothing. The
answer is the kernel's at that moment: a confirmation for a person,
never a guard, since what later writes there finds out for itself. A
mode bit or an owner read from a stat is never taken in its place: it
cannot see groups, ACLs or a read-only mount.

### 6.2 Trust roots

A TLS client trusts the roots its configuration names, one of two:

- **The machine's.** The bundle that the `SSL_CERT_FILE` environment
  variable names, or else the first that exists of the distributions'
  bundle files (`/etc/ssl/certs/ca-certificates.crt` and its kin, in an
  order the module documentation fixes). A bundle is PEM: the kit takes
  each certificate out of it as DER, and skips what is not a certificate.
  A directory of hashed certificates (`SSL_CERT_DIR`) is not read
  (section 9).
- **A DER file:** one certificate, for an issuer or a provider on a
  private network, or a test's fake.

Each is read with `read_file` within bounds that are configuration: the
bundle's bytes, the number of certificates and each one's bytes. The kit
hands the certificates to `skein-tls`, which builds its configuration
from them (../net/tls.md, 3.4), skips and counts one it cannot parse, and
refuses roots of none. Every connection shares the roots, so the process
holds them once. A machine with neither a bundle nor
`SSL_CERT_FILE` refuses to start when its configuration asks for the
machine's roots.

- **A bundle is read in RFC 7468's lax form.** Text outside blocks is
  ignored. A block runs from its `-----BEGIN <label>-----` line to the
  next `END` line or the next `BEGIN` line, whichever comes first. Its
  base64 may hold whitespace and line breaks anywhere.
- **Blocks that are not certificates** (any label but `CERTIFICATE`:
  `TRUSTED CERTIFICATE`, `X509 CRL`, a key) are skipped without being
  decoded, and counted as others.
- **A malformed certificate block is skipped and counted, never a
  refusal.** A block is malformed when it:
  - does not end with its own `END CERTIFICATE` line;
  - carries encapsulated headers;
  - has invalid base64: a character outside the alphabet, or padding
    anywhere but at the end.

  Reading goes on at the next `BEGIN` line. The rest of the bundle still
  serves, as it does past a certificate that does not parse (../net/tls.md,
  3.4). Skipping takes trust away and never adds it, and a damaged bundle
  should not stop every service on the machine.
- **The bounds refuse.** They are configuration, so a bundle past them is
  refused, naming the bound, never cut short to fit it:
  - the bundle's bytes;
  - the number of certificates taken out as DER;
  - each one's bytes, checked as its block decodes. The refusal names
    the certificate's place among the file's certificate blocks,
    malformed ones included.

  A block decodes until its first defect or its bound, whichever comes
  first, and that decides which it is. A malformed block is not kept, so
  it does not count towards the number of certificates.
- **Startup says nothing of its roots unless a block is malformed.** A
  machine's bundle ordinarily holds a root `skein-tls` can't parse, or a
  block of another kind, and no one acts on a line at every start. With a
  malformed block it says the counts once on standard error: how many
  roots were taken; how many `skein-tls` skipped as unparsable; how many
  blocks were malformed; and how many were others.

### 6.3 Terminal modes

A service with a person at a terminal (an interactive front end) may
want its input a key at a time, without the terminal's echo or line
editing, and the window's size. These are shell effects, made with the
kit, since step code makes no syscall and the shell's rules allow only
the ring adapter's:

- **Raw input at startup.** If standard input is a terminal and the
  configuration asks for it, startup reads the terminal's modes, keeps
  them, and sets the ones asked for: no echo, no line editing, and the
  terminal's own signals kept or not, as the service says. Standard input
  is then adopted as a read stream, as an inherited pipe is (io/sockets.md).
  Standard input that is not a terminal is left as it is, and the service
  hears that it has none.
- **Restored at the end.** `main` restores the kept modes once `drive`
  returns, on every path out of it, before it writes its last line to
  standard error.
- **Restored at a panic.** A panic aborts before `drive` returns, so the
  kept modes make a restoration of their own: `Kept::on_abort(output,
  leave)` gives a function that writes `leave`, the bytes that undo what
  the service wrote to its terminal (for the full-screen shell, its
  `LEAVE`: ../ui/screen.md, section 8), to `output` with blocking writes,
  then sets the kept modes back on the kept descriptor. `main` hands it
  to skein-fail-stop (`before_report`), whose hook runs it once, before
  its report (../../foundation/programming-model.md, section 2), and takes it back
  (`before_report(None)`) before `Kept::restore`. A process killed
  outright cannot restore: the terminal stays as it was set, as with any
  program that sets it.
- **The window's size** is read at startup, and again when `SIGWINCH`
  arrives: startup blocks it with the termination signals, so it reaches
  io through the same signalfd, as an event that the window changed
  (io/README.md, 4), and the service asks for the new size with a synchronous
  record (kernel.md, 6.3).
- **A terminal's closing.** A service that is to stop when its terminal
  closes opens its signals with `Watch::TerminationAndHangup`: startup
  blocks `SIGHUP` with the termination signals, and it arrives as `Shutdown
  { signal: Hangup }` (io/README.md, 4). A `SIGHUP` already ignored when startup
  looks stays ignored and is not watched, as `nohup` asks: a blocked signal
  reaches a signalfd whatever its disposition, so startup must look. A
  service that does not ask keeps the default, and a hangup ends it at once.
- **The terminal's foreground.** A child starts in a process group of its
  own (io/processes.md), a background group on its parent's terminal, where the
  kernel stops a group that reads the terminal or sets its modes. A front
  end a service starts as its child, to stand at the person's terminal in
  its place, takes the terminal itself: `open_terminal` opens the process's
  controlling terminal, `/dev/tty`, for reading and writing, close-on-exec,
  never making a terminal controlling, and the caller owns the descriptor;
  `take_foreground(terminal)` makes the caller's process group the
  terminal's foreground group, `SIGTTOU` blocked for the call so that a
  background group may take it, the signal mask restored after. The child
  takes the foreground before it reads the terminal or sets its modes; the
  service that started it takes it back, the same way, once the child has
  ended and before it writes to the terminal again, so that a second
  interrupt reaches the service. A process with no controlling terminal
  can open none, and a terminal that is not the caller's controlling one
  refuses the foreground: each is the call's error. Linux and macOS alike.
- **The terminal lent to a child.** A front end that hands the person's
  terminal to a program of its own for a while (../ui/screen.md, section
  8, *Lent to a child*) gives the child its own descriptors 0, 1 and 2
  (`Way::Inherit`, io/processes.md, section 1), and the modes it found
  rather than its raw ones: `Kept::set_kept(&self)` sets the kept modes
  back on the kept descriptor without giving them up, and, once the
  child has ended and the front end has taken the foreground back
  (above), `Kept::set_raw(&self)` sets the modes raw again as startup
  set them, with the same `Keys`, which `Kept` keeps beside the modes.
  The child takes the foreground itself (above). Each is the call's
  error where the kept descriptor is no longer a terminal; `on_abort`
  and `restore` are unchanged, so a panic or an end while lent leaves
  the terminal as it was found.

Raw input and window sizing are asked for only at a terminal; a line
pipe needs neither. A service may request hangup handling with or without
a terminal, so that the same stop rule holds on a terminal and on pipes.

### 6.4 Trees at startup

A service that starts trees (io/processes.md, 2 to 4) does two more things
at startup, with the kit:

- **The helper, first of all.** The first line of `main` hands its
  arguments to `skein-contain`'s dispatch, before it reads any
  configuration. It returns at once unless the first argument is the
  helper's marker. With the marker, the process is a helper the service
  started (kernel.md, 6.6): it sets up the tree and executes the command,
  or reports its stage and exits, and never returns into `main`. A
  service that starts no tree may leave the line out: without it, a
  helper it never starts is never dispatched.
- **Its own cgroup,** opened before step 7: the `0::` line of
  `/proc/self/cgroup`, beneath `/sys/fs/cgroup`, which must be a cgroup
  v2 mount (`statfs`) mounted with `nsdelegate` (`/proc/self/mountinfo`),
  so that a tree in a cgroup namespace cannot leave its cgroup (io/processes.md,
  4). Startup opens its directory and its `cgroup.events` through the
  ring adapter, and io adopts the two as the service's own cgroup
  (io/processes.md, 2). A machine without either, or a process in a cgroup v1
  hierarchy alone, refuses to start a service that asks for trees, saying
  which it lacks. So does a service whose trees' maps need `newuidmap`
  and that runs with `no_new_privs` (`PR_GET_NO_NEW_PRIVS`), as a unit
  its supervisor hardens may: the helper could not gain `newuidmap`'s
  privilege (kernel.md, 6.6).

A service with neither reads none of this. Under the simulator, the
world gives the service its own cgroup (../testing/simulator.md, 3.4).

### 6.5 A command's settings

Every binary of every repository built on skein reads what it is given
alike, before startup's step 1:

- **Each setting is a flag,** `--name VALUE`, with its set (public,
  internal or test: `../../foundation/cli.md`, 4), its environment twin
  where the table declares one, and a default, documented with the
  flag. The twin is declared by name, never derived from the flag's,
  and its name says the setting's set (`TEMPER_CONFIG` for temper's
  public `--config`; `../../foundation/cli.md`, 5). A setting with no
  sensible default has none, and its absence refuses the start, naming
  the flag and its twin.
- **A table of settings is a group** with one line of its own, which
  help prints once for the group's flags (14.2): temper's three
  directories are one group.
- **The flag wins over its twin, and the twin over the default.** An
  empty variable is unset.
- **Directories default to XDG's base directories:**
  `$XDG_CONFIG_HOME`, `$XDG_DATA_HOME`, `$XDG_STATE_HOME` and
  `$XDG_CACHE_HOME`, each with the specification's fallback under
  `$HOME` when unset, empty or not absolute, then the tool's name.
- **The kit resolves them:** a function of `skein-shell` takes the
  settings' table (name, set, twin, default) with the arguments and the
  environment, and answers each setting's value and where it came
  from, or the setting it refused, so that a binary's `--help` and its
  refusals say the same. Positional arguments stay the binary's own.
- **A test never relies on a default under `$HOME`:** it gives every
  directory as a flag.

### 6.6 Listeners, inherited and held

A service may be given its listening sockets by its parent, so that
the address it reports is certainly its own from before it starts, and
a parent may hold its children's from its own start (io/sockets.md, 2). The
kit's functions, each before startup's step 7, through the kernel as
`open_root` is:

- **`inherited_listeners() -> Result<Box<[(Fd, SocketAddr)]>, ListenersRefusal>`**
  reads systemd's socket activation convention: `LISTEN_FDS`, a decimal
  count *n*, and the descriptors 3 to 2 + *n*. `LISTEN_PID`, which a
  parent cannot know before its spawn, is not read, as Forgejo does not
  read it. Each descriptor must be a socket (`fstat`) of a stream type,
  listening (`SO_ACCEPTCONN`), IPv4 or IPv6 (`getsockname`, which gives
  its address); each is made close-on-exec, so that a child inherits it
  only when given it (io/processes.md, 3). `main` hands each to
  `Io::adopt_listener` once io is made, and the component whose
  configured address it is claims it by its `Listen`.
  - **Absent or empty,** none, and descriptors from 3 are left alone.
  - **Refused,** naming the descriptor and what it is: a count that is
    not a decimal number, a count past 1024 (`MAX_INHERITED_LISTENERS`,
    a bound on startup's work before the service's limits exist; io's
    adoption then holds each to the service's sockets), a descriptor not
    open, or one that is not a listening TCP socket.
  - **A call that fails names itself:** `ListenersRefusal::Check` holds
    the descriptor, the call (`ListenerCheck`: `fstat`, `SO_TYPE`,
    `SO_ACCEPTCONN` or `getsockname`) and its errno. `Kernel` holds the
    close-on-exec update's.
  - **On macOS no socket says it is listening.** XNU's `getsockopt` has
    no case for `SO_ACCEPTCONN` (`bsd/kern/uipc_socket.c`,
    `sogetoptlock`), so it answers `ENOPROTOOPT` for every socket, and
    there that answer passes: a stream socket that never listened is
    taken, and its first accept fails.
- **`listener_at(fd) -> Result<SocketAddr, ListenersRefusal>`,** the same check
  of one descriptor, which it leaves as it is: for a command that hands
  its listener on across its exec, as temper's `forge run` hands
  Forgejo's, and for a parent that checks what it holds. The refusal
  says nothing of where the descriptor came from: `LISTEN_FDS` is the
  caller's word, for a descriptor it was given.
- **`open_listener(addr, backlog) -> Result<(Fd, SocketAddr), Error>`**
  makes, binds and listens, as io's `Listen` does, and answers the
  address bound, without accepting: a parent holds it, adopted, to give
  to its children (io/sockets.md, 2). A bind refused is the caller's to retry
  at port 0, where it chooses a port.
- **The environment is the service's to pass on.** The kit cannot
  remove `LISTEN_FDS` from its own environment; a service that gives
  its children its environment drops it, and sets each child's own.

### 6.7 A durable file at startup

Startup writes what it must keep, a key a person gave it or a generated
configuration, with the kit, as it reads what it needs (6.1).
`store_file(kernel, clock, store)` runs io's whole-file store (io/files.md,
2) to its end, on the kernel it is given, before the service is made:
the sync, the rename and the directory's sync are io's, and no caller
drives them by hand.

- The caller opens the root with `open_root`. The store takes it, and
  closes it on every path.
- The file goes in that directory, or in a private directory named under
  it, which the store opens through io's private-file checks (io/files.md, 3)
  as the effective user the caller read.
- It answers `Kept`, or `Conflict` when the expectation failed. Otherwise
  it answers why not: the unsafe entry io's checks found, the kernel's
  error from the store or from a close after it, the first failure of
  either, or bytes past a `u32`.

The caller states the timeout and gives the randomness seed. A caller
that keeps its kernel for the service hands that one in; one that opens a
kernel for this write alone opens it with `STORE_OPERATIONS`. The store
holds nothing once it returns.

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
  root opened at startup, the invariants it asserts, and dropping it with
  operations in flight.
- **The conformance suite** runs against each backend the host has,
  each scenario once (`tests/conformance/real`; kernel.md, 8): one test
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

- **A child dying with its parent on macOS** (3, 13): whether to provide
  an alternative to Linux's parent-death signal. Default:
  `exec_dying_with_parent` returns `Unsupported` there; a supervisor
  ends the crashed service's whole tree before restarting it.
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
  start, as the shell does (6.2).
- **An inherited listener under the readiness backend** (`readiness.md`, 3): whether
  the kit makes a listener it was given non-blocking, which the shared
  description makes non-blocking for every process that holds it.
- **A terminal suspended and resumed** (`SIGTSTP`, `SIGCONT`): whether
  the shell restores the modes before the process stops and sets them
  again when it continues, which takes a signal the shell handles while
  the loop runs (6.3).
- **Listening, on macOS** (6.6): whether anything reads it another way,
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
  - `state_digest()`, a digest of what it holds, for replay (lib.md,
    5.1): `None` unless the service gives one. Only a world reads it
    (../testing/README.md, 5); `drive` never calls it.
  `Host` is portable: it builds for `wasm32-unknown-unknown`, where a
  world hosts a service with everything handed to it (section 13).
- **A host declares no heap ceiling or worst case.** The simulated
  and real worlds meter each host independently of this interface.
  A scenario may check its measured peak under a named ceiling with
  headroom; the outcomes report the measured peak alone
  (../../foundation/testing-strategy.md, 6; ../testing/README.md, 5).
- **`drive(kernel, clock, host)`** is the loop: reap, read the clock,
  iterate, drain; then return the host's exit once it holds nothing
  (section 12), or else submit, waiting not at all while work is pending,
  until the earliest deadline, or until a completion. A service's `main`
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
  (../testing/README.md, 5).

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
  kit, such as a terminal's modes (6.3), and writes nothing to the
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
  (section 6), runs none of this end: the children it started stay in
  their own groups (io/processes.md) and outlive it, since io kills a child's
  group only when it closes the child. On Linux, a child that execs its
  program through `exec_dying_with_parent` (`shell.md`, section 3) is the exception: the
  kernel sends it its signal when the thread that started it ends, so a
  service whose crash must not leave a child running starts it that way.
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

- **Portable:** `Host` (section 11), a command's settings (6.5):
  `Settings`, `Setting`, `Fallback`, `Base`, `Value`, `Resolved`,
  `Origin`, `Refusal`, `Reason`, `resolve` and `help`, and a binary's
  table and its rendering (section 14), which take the arguments and
  the environment as bytes and read nothing. They build
  for `wasm32-unknown-unknown`, the portable target, and on it they are
  the whole crate.
- **Native, under `cfg(unix)`:** everything that reads the system or
  waits on it: `drive`; the kernel and its backends (`shell.md`, section 3 and `readiness.md`);
  the clock and the seed (`shell.md`, section 5); startup's reads, roots and outputs
  (`shell.md`, section 6); the process, group and signal calls; the terminal's modes
  and input (6.3); and settings' `arguments` and `environment`, which
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

What `../../foundation/cli.md`, 6 to 8 asks of skein: one table per binary,
whose types and rendering every tree's binaries share. A tree's rows are
its own; the binary's crate holds them as constants. It is portable
(`shell.md`, section 13): it takes bytes and returns text, and reads nothing.

### 14.1 Entries and sets

- **`Set`:** `Public`, `Internal` or `Test` (`../../foundation/cli.md`, 4).
- **`Table`:** the binary's name; its one line, what the binary is; its
  version and commit, as its tree's build identity words them; its
  commands; the settings every command takes, as groups (6.5); the
  variables it reads or sets beside its settings' twins; and its last
  line, where to read more.
- **`Command`:** how it's reached, its set, its forms (each one's
  arguments as usage shows them after the name), its one line, its
  positional arguments, its settings' groups, its switches, its
  variables, and its notes. It's reached by its words after the
  binary's name, none for the bare binary's form, or by the name the
  binary runs as, for a delegated grammar
  (`../../foundation/cli.md`, 5).
- **`Argument`:** a positional argument as usage shows it, and its line.
- **`Switch`:** a flag with no value, `--name`: its name, set, line and
  notes. A setting (6.5) is a flag with a value.
- **`Variable`:** its name, set, form, line, what holds when it's
  absent, notes, and whether the binary reads it or sets it for the
  programs it runs.
- **The host's variables** that the kit itself reads are its constants,
  public: `HOME` and the four XDG base directories, read by a setting's
  `Fallback::Base` (6.5). A table that holds such a setting reads them,
  and its documents say so without the tree declaring them again.
- **`TestTable`:** a tree's test set: its variables, all `Test`, held in
  its test-support crate (`../../foundation/cli.md`, 8).
- **A lookup names a declared entry:** `Variable::value` answers a
  variable's bytes from the captured environment, the pairs
  `environment` returns, and an empty value is unset. No lookup takes a
  string.

### 14.2 Help

Every page is lines of at most 80 columns. A two-column line puts its
first column at column 2 and its second at column 37; a first column
of more than 33 characters takes the line's own row, with the second
column on the next. A second column, or a one-column line, wraps at
spaces to 80 columns, a two-column line's continuation at column 37 and
a one-column line's at its own indent.

- **The page,** for `-h`, `--help` and `help` alike, its parts
  separated by one blank line:
  1. `BINARY: LINE`;
  2. `usage:`, then `  BINARY FORM` for each form of a public bare
     command, and `  BINARY COMMAND [ARGUMENT...]` when a public command
     has words;
  3. `commands:`, then each public command with words: `WORDS FORM` and
     its line, the line beside the first form only;
  4. `flags:`, then each of the table's groups of settings that has a
     public one: its public flags, `--name VALUE` joined by `, `, and
     the group's line; then `-h, --help` with `Print this.` and
     `--version` with `Print the name, version and commit.`;
  5. `environment:`, when there is any: for each group of the table's
     or of a public command's, the twins of its public settings, joined
     by `, `, with `As --a, --b and --c.`; then each public variable
     that the table or a public command reads, once, with its line;
  6. the table's last line.
- **A command's page,** for `COMMAND -h` or `--help`: an internal
  command's first line is `internal: not part of BINARY's public
  command line; it may change without notice.` Then `usage:` with its
  forms; its line; `arguments:`; `flags:`, each setting with its line,
  `Default: DEFAULT.` or `Required.`, and `Environment: TWIN.` where it
  has one, then each switch with its line; `environment:` with the
  variables it reads; and its notes. A public command's page lists its
  public entries; an internal command's lists all of its own. A part
  with nothing in it is left out.
- **The version line:** `BINARY VERSION (COMMIT)`, as temper's
  `release.md`, section 3 words it for every binary of temper's.

### 14.3 The reference documents

Rendered for `../../foundation/cli.md`, 8, each a Markdown document whose
first line is its title and whose second paragraph says it's generated:

- **`<binary>-args.md`:** `# BINARY: arguments`, then `## Public` and
  `## Internal`, each holding the commands then the flags of that set.
  A command's heading is `` ### `BINARY WORDS` `` (`` `BINARY` `` for
  the bare form, `` run as `NAME` `` for a delegated grammar), followed
  by a list: `**Set:**`, `**Forms:**`, its line, `**Arguments:**`,
  `**Flags:**` and `**Notes:**`, each item only where it has one. A
  flag's heading is `` ### `--name VALUE` `` or `` ### `--name` ``,
  with `**Set:**`, `**Default:**` (or `**Required.**`), `**Twin:**`,
  its line and `**Notes:**`.
- **`<binary>-env.md`:** `# BINARY: environment`, then `## Public` and
  `## Internal`, holding the variables the binary alone reads or sets,
  its settings' twins among them. A variable's entry is its heading,
  `` ### `NAME` ``, then a list whose first item is `- **Set:** public`,
  `internal` or `test`, then `**Reads**` or `**Sets**`, `**Form:**`,
  `**Absent:**`, `**Twin of:**` for a twin, its line and `**Notes:**`.
- **`<tree>/docs/design/cli/common-env.md`:** `# TREE: common environment`, the variables two
  or more of the tree's tables read, the host's among them, as
  `<binary>-env.md` holds them, with `**Read by:**` naming the binaries.
- **`<tree>/docs/design/cli/test-env.md`:** `# TREE: test environment`, the tree's test table,
  as `<binary>-env.md` holds a variable.

Entries keep their table's order; a variable read by several commands
appears once. The checks read these headings and `**Set:**` items
(`../../foundation/cli.md`, 9, check 5), so their form is the contract.

### 14.4 Reaching a command

`Table::reach` answers what a binary's arguments reach, so that every
binary's dispatch, help and refusals agree
(`../../foundation/cli.md`, 5 and 7). It takes the name the binary was
run by, argv's first word, and the words after it, as the binary's
capture holds them, and reads nothing else:

```rust
pub enum Reached<'a> {
    /// `-h`, `--help` or `help`, alone: the page (14.2).
    Help,
    /// `--version`, alone: the version line (14.2).
    Version,
    /// A command's words, then `-h` or `--help` alone: its page.
    CommandHelp(&'static Command),
    /// A command and the arguments after its words.
    Command(&'static Command, &'a [Box<[u8]>]),
    /// No command: the refusal's line, newline included.
    Unknown(String),
}

impl Table {
    pub fn reach<'a>(&self, run_as: &[u8], arguments: &'a [Box<[u8]>]) -> Reached<'a>;
    pub fn unknown(&self, arguments: &[Box<[u8]>]) -> String;
}
```

In this order:

1. **Run as another program:** a command reached by `Reach::RunAs(NAME)`
   whose NAME is `run_as`'s last part, after its last `/`, is reached
   with every argument, its grammar delegated: `--help` and `--version`
   are its own.
2. **The binary's page and version:** exactly `-h`, `--help` or `help`
   is `Help`; exactly `--version` is `Version`.
3. **By its words:** the command reached by `Reach::Words` whose words
   are the longest prefix of the arguments, the bare form's none being
   a prefix of any. When the arguments after its words are exactly
   `-h` or `--help`, `CommandHelp`; otherwise `Command`, with those
   arguments.
4. **None:** `Unknown`, the line `unknown` gives.

`unknown` is `BINARY: no command WORDS; BINARY --help lists them` and a
newline, WORDS the arguments up to and including the first one no
command's words continue with, joined by spaces, and
`BINARY: no command; BINARY --help lists them` for no arguments. A
binary prints it on stderr and exits 2 (`../../foundation/cli.md`, 5).
A bare form that takes no positional argument answers a word it
doesn't take with the same line, so a mistyped command reads the same
whether the binary has a bare form or not.

### 14.5 The table's own words

What every binary answers from its table before any command of its own
runs: its page, its version line, a command's page and the refusal of a
command it doesn't have (`../../foundation/cli.md`, 5 and 7). One code
answers them for every binary, so their words and exit codes are the
same everywhere. Like the rest of the table it is portable: it takes the
captured arguments and returns text, and the binary writes it.

- **`Table::front(&self, arguments: &[Box<[u8]>]) -> Front<'_>`** reads
  the arguments after the binary's name, as `arguments` returns them
  (6.5), and answers the first of these that holds:
  1. the first argument is `-h` or `--help`: `Print` of the page
     (14.2), whatever follows;
  2. the first argument is `--version`: `Print` of the version line,
     whatever follows;
  3. the only argument is `help` and no command is reached by the words
     `help`: `Print` of the page;
  4. the words of some command reached by one or more words are a
     prefix of the arguments: the command with the longest such words.
     When the argument right after its words is `-h` or `--help`,
     `Print` of its page, whatever follows; otherwise `Run` of it with
     the arguments after its words;
  5. the table has a bare command, reached by no words: `Run` of it with
     every argument, or with none when there is none. For a bare
     command, `-h` and `--help` are the page's (1), so its own page is
     printed only within a refusal (below);
  6. otherwise `Refuse`, of `BINARY: no command WORD; BINARY --help says
     more` and a newline, `WORD` the first argument as UTF-8 with each
     invalid sequence replaced, or of `BINARY: no command; BINARY --help
     says more` and a newline when there is no argument.
- **`Front<'a>`:**
  - `Print(String)`: the binary writes it to stdout and exits 0;
  - `Run { command: &'a Command, rest: &'a [Box<[u8]>] }`: the
    binary runs the command, one of its table's, reading `rest` as that
    command's own arguments, with `resolve` (6.5) for its settings;
  - `Refuse(String)`: the binary writes it to stderr and exits 2.
- **`Table::refusal(&self, command: &Command, what: &str) -> String`:**
  the words a binary refuses one of its commands' arguments with, before
  it exits 2: `BINARY WORDS: WHAT` (`BINARY: WHAT` for the bare
  command) and a newline, then the `usage:` part of the command's page,
  as 14.2 renders it. For example, a bare command whose form is `OUT`,
  given none:

  ```
  demo: OUT is not given
  usage:
    demo OUT
  ```

- **What it leaves to the binary:** a delegated grammar's rows
  (`Reach::RunAs`), which the binary matches by the name it runs as
  before it reads its table, and every word of a command's own arguments,
  a `--` among them.
