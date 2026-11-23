# The shell kit: startup

## 1. Startup

Before its loop, a service's `main`, with the kit:

1. reads its configuration, the limits of every layer among it;
2. validates the edge limits and refuses formulas that overflow; there
   is no startup memory fit check (../../foundation/programming-model.md, 6.3);
3. blocks the termination signals that io will read, so they arrive as
   `Shutdown` events (io/README.md);
4. opens the first roots for io's files, from configuration, with
   `open_root` (io/files.md; kernel-files.md, 1), which makes nothing; a root
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
   roots, or a DER file (1.2; ../net/tls.md);
7. reads the seed and opens the kernel.

Startup is each service's `main`, step by step, with the kit's
functions: the kit holds no startup function of its own.

A service that starts trees also dispatches its helper before step 1,
and opens its own cgroup before step 7 (1.4); one whose spawns carry a
reach dispatches its helper before step 1 too, and checks that it can
restrict a tree once its kernel is open (1.4).

A service that listens takes the listeners it was given, and opens
the ones it holds for its children, before step 7 (1.6).

What startup reads, its configuration, its secrets and certificates, it
reads with the kit (1.1). Then `main` makes the service and calls `drive`
(`shell.md`, section 11).

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

### 1.1 Reading files at startup

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
  one, but for macOS's own among its parents (`readiness.md`, 5);
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
`Debug` (lib-bytes.md, section 1).

**The system's links, for a caller's own check.** A caller that checks
a path's components for links itself first takes
`resolve_system_links(path: &Path) -> PathBuf`, the path with the
system's own links among its parents passed by the walk the no-follow
opens make (`readiness.md`, 5), always, not only on a refusal; on
Linux, the path as given. What it answers is checked as before, so the
caller refuses what the shell's opens refuse, and no other link.

**The process's own identity** is read the same way, once, at startup,
through the ring adapter:
- `effective_user()`, the effective user ID, for io's private files
  (io/files.md, 3);
- `process_id()`, the process ID, for a service that reports it, such
  as smith's `session.started`.

Neither is read again: a process's identity does not change while it
runs. A service never reads them itself, with `std::process` or a
syscall of its own. It takes them from its shell.

**The host's name** is the kernel's, asked at startup only, for a
service whose defaults name its machine, such as temper's worker
(`temper/docs/design/protocol/configuration/file.md`, 5).
`host_name() -> Result<Box<[u8]>, i32>` answers it through `uname`,
on Linux and macOS alike, one call in `posix` (readiness.md, 5): the
node name's bytes before its NUL, or the call's error number. On Linux
that is the UTS namespace's host name, the bytes
`/proc/sys/kernel/hostname` gives before its line feed; on macOS,
which has no `/proc`, it is `kern.hostname`, the name `hostname`
prints. It is the kernel's answer at that moment, neither checked nor
decoded: the caller says what a name must be, and a later change of
the name is not seen. A service never reads `/proc` or calls `uname`
for it itself.

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

### 1.2 Trust roots

A TLS client trusts the roots its configuration names, one of two:

- **The machine's.** The bundle that the `SSL_CERT_FILE` environment
  variable names, or else the first that exists of the distributions'
  bundle files (`/etc/ssl/certs/ca-certificates.crt` and its kin, in an
  order the module documentation fixes). A bundle is PEM: the kit takes
  each certificate out of it as DER, and skips what is not a certificate.
  A directory of hashed certificates (`SSL_CERT_DIR`) is not read
  (`shell.md`, section 9).
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

### 1.3 Terminal modes

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

### 1.4 Trees at startup

A service that starts trees (io/processes.md, 2 to 4), or whose spawns
carry a reach (io/processes.md, 6), does more at startup, with the kit:

- **The helper, first of all.** The first line of `main` hands its
  arguments to the helper's dispatch, `skein_shell::helper::dispatch()`,
  before it reads any configuration. It returns at once unless the first
  argument is the helper's marker. With the marker, the process is a
  helper the service started (kernel-cgroups.md, 1): it sets up the tree and
  executes the command, or reports its stage and exits, and never
  returns into `main`. A service that starts no tree and spawns with no
  reach may leave the line out: without it, a helper it never starts is
  never dispatched. One that needs it and leaves it out has every reached
  spawn fail `Reach(Helper)`, never run unrestricted.
- **Whether it can restrict a tree.** A service whose spawns carry a
  reach (io/processes.md, 6) calls `Kernel::can_restrict` once its kernel
  is open, before it serves: it spawns a restricted child (kernel-cgroups.md,
  1) and answers `Ok`, or the `Unenforced` a reach would meet, whose
  words say what an operator can do. What follows is the service's: a
  worker refuses the runs that rely on the fence, or runs them
  unconfined only by its configuration's word
  (`temper/docs/design/draft/confinement.md`, 6.5). Inside a container,
  whether its seccomp profile admits Landlock's calls is this check's to
  find. Under the simulator, the world gives the service the answer it
  chooses, as it gives its own cgroup.
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
  privilege (kernel-cgroups.md, 1).

A service with neither reads none of this. Under the simulator, the
world gives the service its own cgroup (../testing/simulator.md, 3.4).

### 1.5 A command's settings

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
  help prints once for the group's flags (`shell-table.md`, 1.2): temper's three
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

### 1.6 Listeners, inherited and held

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

### 1.7 A durable file at startup

Startup writes what it must keep, a key a person gave it or a generated
configuration, with the kit, as it reads what it needs (1.1).
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
