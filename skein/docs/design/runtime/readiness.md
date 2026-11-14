# The readiness backend

This backend is for hosts where io_uring is not available: a container
whose seccomp profile refuses io_uring (Docker's default profile does), a
kernel with `io_uring_disabled` set, or macOS (Apple Silicon, from macOS
15, Darwin 24: 6).

- **It emulates completions.** It makes each record's call without
  blocking, and completes the record; a call that would block waits for
  its descriptor's readiness (epoll on Linux, kqueue on macOS) and is
  made again. Regular files are always ready, so file operations run at
  submit, as the ring's synchronous operations do.
- **One shared part, and each system's parts beside it.** The table, the
  waits on descriptors, retries, cancels and closes are written once, in
  safe Rust, the same over either wait; the calls themselves and the
  wait are each system's (5).
- **It costs nothing on the ring path.** `Kernel` holds one backend in an
  enum, matched once per submit and once per reap (1). Step code never
  knows which backend runs.
- **The kernel boundary keeps it cheap:** records that carry their own
  memory and plain values, single-shot operations, and synchronous
  operations already in the vocabulary. A feature only io_uring has is
  always an optimisation, never something io's behaviour relies on.
- **It meets the conformance suite** as the ring does, each scenario
  once (kernel.md, 8), on our Linux machines through epoll and on macOS
  through kqueue.

## 1. The choice at startup

- **`Kernel::open` takes the host's backend:** on Linux the ring, or the
  readiness backend when the ring cannot be set up because io_uring is
  refused (`ENOSYS` or `EPERM` from its setup: missing, disabled, or a
  seccomp profile's); on macOS always the readiness backend. Any other
  refusal of the ring stops the service, as before (`shell.md`, section 4): a kernel
  below the floor is below it for both backends.
- **The ring's refusal alone decides.** `OpenError::Setup` with `ENOSYS`
  or `EPERM` falls back; every other error of the ring's open (another
  `Setup`, `Missing`, `BelowFloor`, `NoOperations`) is returned as it
  is. The readiness backend then opens as `open_on` would, and its own
  refusal, below the floor among them, is the one returned.
- **`Kernel::open_on` forces one,** `Backend::Ring` or
  `Backend::Readiness`, for tests: the ring's own tests and its
  conformance take `Ring`, so that a machine without io_uring fails them,
  saying so, rather than running them on the other backend; the readiness
  backend's take `Readiness`. `Ring` on macOS refuses
  (`OpenError::Unsupported`).
- **`Config` names no backend,** so every caller of `Kernel::open` is the
  same on both.
- **No setting of its own.** A service takes `Kernel::open`; nothing in a
  binary's command line or environment chooses the backend, and the
  settings of `shell.md`, 6.5 gain none. The tests that need the readiness backend on
  Linux open it themselves. A binary whose own tests need it forced adds
  a setting of its own then, under 6.5.
- **What runs is said:** `Kernel::chosen()` answers the backend and, when
  the ring was refused, its error number; its `Display` reads, for
  example, `readiness (epoll): io_uring refused (EPERM)`. A service says
  it once on standard error at startup when it is not the ring, beside
  the trust roots' counts (`shell.md`, 6.2), so that a slower host is never a
  mystery. The line is the service's `main`'s, as the counts are, never
  the kit's, which prints nothing: `<service>: kernel: <chosen>`. In
  skein, the echo's and the UI server's `main` say it; a service
  downstream adds it when it next takes skein.

## 2. The table

- **One slot per operation,** as the ring's (`shell.md`, section 3): allocated once at
  `Config`'s size, an operation's slot named by its index and generation,
  a map from token to slot that finds a cancel's target and catches a
  token already in flight. No slot holds memory the kernel writes later:
  every call is made by the backend, on its own thread, and returns
  before the record moves again.
- **The ready list** holds the slots whose record has its result, oldest
  first; `reap` hands them up as far as the completion queue has room.
  `room` and `in_flight` answer as the ring's.
- **The waiters.** The contract allows a descriptor at most one
  operation that reads (`Recv`, `Accept`, `PipeRead`, `ReadSignal`, a
  `Wait`) and one that writes (`Send`, `PipeWrite`, `Connect`) in flight
  at once (`skein_io::kernel`, broken invariants), so each descriptor
  has at most two waiters, one for each side. The backend keeps a map
  from descriptor to its waiters.
- **Interest is level-triggered,** and registered only while a
  descriptor has a waiter: added with its first, changed when the sides
  change, removed with its last. A readiness never seen is never lost,
  and a descriptor with no waiter costs the wait nothing.

## 3. A record's path

- **Made at submit:** every operation on files (`Open`, `Read`, `Write`,
  `Append`, `Truncate`, `Sync`, `Stat`, `Rename`, `Remove`,
  `MakeDirectory`, `List`, `Lock`), the sockets' `Socket`, `Bind`,
  `Listen` and `Shutdown`, `Close`, `Spawn`, `Signal`, `Usage` and
  `Window`. Each makes its one call, and its result waits in the ready
  list.
- **Attempted at submit, then waited for:** the operations that read or
  write a stream, accept, connect, read a signal or wait for a child.
  The call is made at once where it cannot block: on a descriptor the
  backend made non-blocking (below), and a child's wait, which asks not
  to hang (`WNOHANG`). If it would block (`EAGAIN`, a
  connect's `EINPROGRESS`), the record waits on its descriptor's side.
  When the wait reports the side ready, the call is made again, and the
  record completes, or waits again if it would still block: a readiness
  that another reader took first is no failure.
- **A descriptor the backend did not make** (an inherited standard
  stream, the signalfd startup opened) keeps its flags: changing them
  would change them for every process that shares it. A socket's
  `Recv` and `Send` on it ask not to block, call by call
  (`MSG_DONTWAIT`), and are attempted at once as on the backend's own.
  Any other record on it waits for readiness first, then makes its one
  call; a write to a pipe or a terminal is capped at `PIPE_BUF` bytes,
  which a writable pipe takes whole, and completes short as the
  contract allows. A terminal's flow control may still hold such a
  write, as a stalled disk holds a file's.
- **A descriptor the wait cannot watch is always ready.** epoll refuses
  a regular file or `/dev/null` (`EPERM`), which an inherited standard
  stream may be redirected from; their reads and writes never wait, so
  a record on one makes its call at once, as an operation on files does,
  and registers no interest.
- **A known limit: an inherited listener.** An `Accept` has no per-call
  flag, so on a blocking listener shared with another process, one that
  the other takes the connection from between the readiness and the
  call would hold the loop until the next. The backend makes non-blocking
  the sockets it makes; a listener the service inherited (`shell.md`, 6.6) keeps the
  flags it came with (`shell.md`, section 9).
- **The backend's own descriptors are non-blocking:** every socket it
  makes (`SOCK_NONBLOCK`), every one it accepts, and the parent's end of
  each pipe a `Spawn` makes, which is a description of its own, so the
  child's end stays blocking. It remembers which descriptors it made so,
  from the call that made one to its `Close`.
- **Each side's readiness** is the one its call needs: readable for a
  receive, an accept, a pipe's read, a signal and a child's exit (a
  pidfd is readable once the child has exited); writable for a send, a
  pipe's write and a connect, whose result is then the socket's pending
  error (`SO_ERROR`). An error or a hang-up the wait reports wakes both
  sides: the call says what it is.
- **A child and a signal, step by step.** A `Wait` makes `waitid` on the
  pidfd with `WNOHANG` at submit (and `WNOWAIT` to observe); no exit yet
  (no child reported) waits for the pidfd to be readable, and the call
  is made again. A `Wait` that reaps follows the one that observed, so it
  completes at submit. A `ReadSignal` reads one record from the
  signalfd startup opened, which is blocking: it waits for readable
  first, and a readable signalfd holds a whole record. A pipe's read or
  write on a `Spawn`'s parent end is attempted at submit, as a socket's
  is; once every holder of the child's end has closed it (the child and
  what it started), the parent's read sees the end of the stream and its
  write `EPIPE`, as with the ring.

## 4. Counts, retries, cancels and closes

- **One call, one count.** A transfer completes with the count of the one
  call that made progress, never continued by the backend: a short send,
  write or read goes up as it is, and io continues it from `from + n`,
  as with the ring (`skein_io::kernel`, the counts).
- **Interrupted transfers are reissued:** an uncancelled `PipeWrite`,
  `Write`, `Append`, `Send`, `PipeRead`, `Read`, `Recv` or `ReadSignal`
  interrupted with `EINTR` is reissued from its slot with identical
  buffers, offsets and token, without a retry bound. The backend absorbs
  the interruption before any outward completion. A requested cancellation
  instead completes `Cancelled`; other errors and short counts retain
  their usual results. An `Open` whose `..` raced a rename or a mount
  (`EAGAIN`) is made again, up to 16 times, then fails with
  `Other(EAGAIN)`. A `Close` is never made again: its descriptor may
  already be another's.
- **Every other interrupted operation completes `Other(EINTR)`,** a
  `Close` among them (above), as the ring's does, so that one error
  number means one thing on both backends: a `Socket`, `Bind`, `Listen`,
  `Accept`, `Connect`, `Shutdown`, `Open`, `Truncate`, `Sync`, `Stat`,
  `Rename`, `Remove`, `MakeDirectory`, `List`, `Lock`, `Wait`, `Spawn`,
  `Signal`, `Usage` or `Window`. Neither the shared part nor the
  system's call (5) makes it again, nor any call of its own: an `Open`
  makes one `openat2` (`openat` on Darwin), a `Wait` one `waitid`, and a
  `Connect` never calls `connect` a second time on its socket, which one
  whose connection is under way would answer `EALREADY`. The calls that
  set up or inspect around an operation's own (a socket's options, a
  bound address, the kind of what an `Open` opened, a `Spawn`'s working
  directory and the report of its exec) keep their retries: none makes
  the operation again. On Linux the ring's synchronous operations share
  these calls (`shell.md`, section 3), so its `List`, `Lock`, `Spawn`, `Signal`,
  `Usage` and `Window` complete so too. The callers outside the kernel
  that share a call make it again: a held listener's `bind` and `listen`
  (`shell.md`, 6.6), and `keeper::poll_child`, the harness's. With the signals the shell
  watches blocked (Linux) or ignored (Darwin), these calls are seldom
  interrupted.
- **Errors map as the ring's do,** per operation, by the same functions
  (5): an error number means what it means to the ring.
- **A cancel** of a record that waits takes it off its side, completes
  it `Cancelled`, then completes itself `Nothing`. Every other target is
  too late (`TooLate`), and completes with its own result: it was made at
  submit, or made again on a readiness, before the cancel came. A cancel
  of an operation on files is therefore always too late (kernel.md, 6).
- **A close** removes its descriptor's interest, if it has any, before
  the call, so that a description another process still holds never
  reports a readiness for a number reused since. A waiter beside a
  `Close` is a broken invariant of the contract, which the backend
  asserts.
- **The wait** blocks as `Wait` says: not at all while the ready list
  holds anything or `Wait::No` asks; until the deadline, read against
  the monotonic clock just before the call and rounded up to the wait's
  resolution, for `Wait::Until`; or for a readiness, for
  `Wait::Forever`, which asserts that something waits. An interrupted
  wait returns to the loop, as the ring's does. Its event buffer is
  allocated once, a slot's worth per operation.
- **Dropping it** with operations in flight drops their records: no
  memory of theirs is the kernel's, so nothing is leaked. Their
  descriptors stay open until the process ends, as the contract says.

## 5. Each system's parts

The calls, the error numbers and the wait are each system's, in a module
of its own, `linux` or `darwin`, which holds that system's `unsafe` and
says why it is sound, case by case (`shell.md`, section 1). On Linux, the ring adapter
and the readiness backend share every call they both make (the
synchronous operations, the error tables, what decodes a `statx` or a
`siginfo`).

| Part | Linux | macOS |
|---|---|---|
| The wait | `epoll_wait` | `kevent` |
| Open beneath a root | `openat2`, `RESOLVE_BENEATH` | `openat`, `O_RESOLVE_BENEATH`, `O_NOFOLLOW_ANY` |
| Listing a directory | `getdents64` | `readdir`, its positions kept (below) |
| Rename, never replacing | `renameat2`, `RENAME_NOREPLACE` | `renameatx_np`, `RENAME_EXCL` |
| Stat | `statx` | `fstat` |
| Spawn | `pidfd_spawn` | `posix_spawn`: the same file actions and process group; for a terminal, `fork` and `execve` (below) |
| A child's handle | its pidfd | a kqueue of the child's own, its `EVFILT_PROC` exit registered: a descriptor that is readable once the child has exited |
| A child's exit | `waitid` on the pidfd | `waitid` on the child's pid, `WNOWAIT` to observe |
| Signalling a child | `pidfd_send_signal` | `kill`, `killpg`, by the pid its handle names |
| Signals to the service | `signalfd` | a kqueue of the service's own, its `EVFILT_SIGNAL`s registered, the signals ignored |
| The seed | `getrandom` | `getentropy` |
| Usage's resident size | KiB | bytes |

- **A child stays unreaped until io closes it,** on macOS as with
  pidfds, so its pid is never another process's while io can still
  signal it, and its handle names it alone.
- **Only the shell's own direct children are watched:** macOS has no
  subreaper.
- **The handles are descriptors,** so io, its records and the
  conformance suite are the same on both systems: a `Wait` waits for its
  handle to be readable, a `Signal` finds the pid its handle names, and
  a `Close` of the handle forgets it.

**The crate's layout.** `unsafe` is in `linux`, `darwin` and `posix` alone; every
other module is safe Rust, the same on both systems.

| Module | What it holds | Built for |
|---|---|---|
| `kernel` | `Kernel`, `Backend`, `Chosen`, `Config`, `Wait`, `OpenError`: the public kernel | both |
| `readiness` | the readiness backend's shared part (2 to 4), over `system` | both |
| `calls` | what every backend decodes the same way: each operation's error table, the kind of what an `Open` opened, the paths' checks, `Open`'s retries, the listing buffer's size, a release against a floor (6) | both |
| `linux` | Linux's calls and wait (`linux::epoll`), and the ring adapter (`linux::ring`), the only user of `io-uring` | Linux |
| `darwin` | Darwin's calls and wait (`darwin::kqueue`) | macOS |
| `posix` | what both systems write alike: terminal modes and sizes, a signal to a group and the probes of a group or a process, a socket's shutdown, the wait for a readable descriptor, a startup file's read and a mode's kind, `SIGPIPE`'s block, the error numbers' meaning | both |
| `clock`, `seed`, `startup`, `terminal`, `settings`, `drive` | the public startup and loop, calling `system` | both |
| `keeper` | what a test harness alone needs of the processes it keeps (below) | both, Linux's own parts on Linux |

- **One face, two systems.** `linux` and `darwin` give the same
  crate-private functions, by name and signature, and `lib.rs` names the
  one built as `system`: sockets, files, processes, the signal source,
  the terminal, the wait, the clock, the seed, the release and the
  startup reads. They take and return io's values (`Fd`, `Addr`, `Stat`,
  `Exit`, `Done`) and error numbers, never a system's structure: no
  `statx`, `siginfo`, `stat` or `kevent` reaches the shared part, so each
  system decodes its own. Building for each system checks the face.
  What the two write alike each takes from `posix`, so the face stays
  whole.
- **Each system's wrappers are its own,** written against that system's
  manual and the `libc` types for it (a `sockaddr_in`'s length byte on
  Darwin, its `stat`'s layout): neither module calls the other, and
  neither emulates the other's calls. A wrapper that reads the same
  against both manuals lives once, in `posix` (`tcgetattr`, `kill`); one
  that differs in a type or a step (a `stat`'s `mode_t`, `fsync`) is written
  in each, so `linux` and `darwin` hold only what differs.
- **Linux's own is built for Linux alone:** the ring adapter and
  `Running`'s ring arm; cgroups (`open_cgroup`, `start_binary_in_cgroup`,
  `wait_cgroup_change`); the subreaper (`make_subreaper`, `subreaper`,
  `set_subreaper`); and the pidfd helpers (`open_pidfd`, `pidfd_exited`).
  `Backend::Ring` itself is on both, so a match on it downstream stays
  exhaustive on both: `open_on(.., Backend::Ring)` on macOS is
  `OpenError::Unsupported`, and `Kernel::open` there never tries the
  ring. Every other public item is on both.
- **The tree keeper's calls,** on both (../examples.md, 6): `child_id`, the
  pid a child's handle names; `signal_group`, a signal to a process group
  by its ID (`kill` of its negative); and `group_live`, whether a process
  group still has a member (`kill` of its negative with no signal, `ESRCH`
  for none). `process_live` probes a positive pid with no signal, counting
  an unreaped zombie as live and refusing 0, 1 and values beyond `pid_t`.
  These calls name processes and groups by number, as no record does,
  and are for the harness alone.
- **What a harness alone needs is fenced in `skein_shell::keeper`,** a
  public module no service has a use for: the tree keeper's calls; a
  kept child's start, handle, signal, status and abandonment
  (`start_binary`, `hosted_pipes`, `open_child_pidfd`, `signal_kept_child`,
  `poll_child`, `abandon_binary`, and the closing of what it kept,
  `close_keeper_fd`); a signal raised by hand (`open_signal_pipe`,
  `write_service_signal`, `signal_current_thread`); a blocking wait for a
  readable descriptor (`wait_readable`); and Linux's own cgroups,
  subreaper and pidfd helpers. The crate's root holds a service's calls
  alone. A service closes a descriptor it opened at startup and never gave
  to io with the root's `close_startup_fd`.

**Darwin's parts,** in `darwin`:
- **Sockets** are made without blocking and closed on exec by `fcntl`
  after `socket` and `accept`, which take no such flags on Darwin; the
  loop's thread makes both calls and every spawn, so no child starts
  between them. Addresses carry their length byte. `SIGPIPE` is blocked
  on the loop's thread, as on Linux.
  A V6ONLY socket's bind to an IPv4-mapped address is `AddressNotAvailable`
  (`bsd/netinet/tcp_usrreq.c:334-354`, `bsd/netinet6/in6_pcb.c:273-281`),
  and its mapped connect is `InvalidArgument` (`tcp_usrreq.c:546-552`'s
  `EAFNOSUPPORT`). Linux and the simulator assert `InvalidArgument` for
  the bind and `Unreachable` for the connect; both systems keep families
  separate.
- **The wait** is a kqueue: `EVFILT_READ` and `EVFILT_WRITE` for a
  descriptor's sides, level-triggered, added and deleted as its waiters
  change (2). `EV_EOF` and `EV_ERROR` wake both sides, as epoll's
  hang-up and error do. A descriptor kqueue cannot watch is always
  ready, as 3's: the error its registration answers for one (`EINVAL`
  for a device it does not support) is the one epoll's `EPERM` is.
- **Opening beneath a root:** `openat` with `O_RESOLVE_BENEATH`, and
  `O_NOFOLLOW_ANY` for the ways that refuse a link (kernel.md, 6.1),
  close-on-exec, without blocking or a controlling terminal, as on Linux.
  `PathNoFollow`, which Linux opens `O_PATH | O_NOFOLLOW`, is
  `O_SYMLINK | O_EVTONLY` with `O_NOFOLLOW_ANY` and `O_RESOLVE_BENEATH`:
  the entry itself, a link as a link, for `Stat` and `Close` alone.
  Linux needs no permission to read the entry; Darwin needs it readable:
  `O_EVTONLY` still authorizes `READ_DATA` (XNU `bsd/vfs/vfs_syscalls.c`,
  lines 4894-4900; `bsd/vfs/vfs_subr.c`, lines 8434-8438). Each system's
  answer is held to conformance. It is opened without blocking too:
  `O_EVTONLY`'s access is `O_RDONLY`'s, so a FIFO without a writer would
  hold `fifo_open` asleep, where `O_PATH` never opens the FIFO (macOS
  runs 40003540284 and 40010966195, the path-only conformance test
  stopped at the suite's budget).
  Darwin answers an escape with `ENOTCAPABLE`, which `darwin` gives up
  as Linux's `EXDEV`, so that one error table serves both (`Escape`).
- **Files:** `pread`, `pwrite`, `write`, `ftruncate`, `fstat` (decoded by
  `darwin`), `renameatx_np` with `RENAME_EXCL` for a rename that never
  replaces and `renameat` otherwise, `unlinkat`, `mkdirat`, `linkat`
  with no flags (a second name, `HardLink`: kernel.md, 6.4), `flock`.
  A no-replace rename to the identical name succeeds without changing
  it on case-insensitive APFS (macOS conformance run 40003540284): XNU
  `bsd/vfs/vfs_syscalls.c:9538-9564` admits the same vnode on those
  volumes, and `9748-9753` returns success for the identical name.
  Linux and the simulator refuse it with `Exists`; conformance keeps
  the distinct-name hard-link refusal separate.
  Component lookup is the volume's (`bsd/vfs/vfs_lookup.c:1518-1524,1552`).
  APFS refuses a 300-byte component with `NameTooLong` before a removed
  directory's missing entry or a creation's final-slash refusal
  (`1766-1767`), as observed in that run. Linux and the simulator
  assert `NotFound` and `IsADirectory`, respectively.
  `Sync` is `fcntl(F_FULLFSYNC)`, which reaches the disk as Linux's
  `fsync` does, where Darwin's `fsync` stops at the drive's cache.
- **Listing a directory** keeps `readdir`'s positions: the backend holds
  a directory stream for each descriptor it has listed, opened on a
  duplicate of it at its first `List` and closed with its `Close`. A
  `List` reads entries until the next does not fit the record, sets the
  stream back before that one (`telldir`, `seekdir`), and the next `List`
  starts there. `.` and `..` are skipped and an entry's kind is its
  `d_type`, or its `fstatat` where that is unknown, as on Linux. The
  listing state is each system's: Linux's one buffer of the table's,
  Darwin's streams.
- **Signal reads** from kqueue have no sender identity. Their
  `Signalled::Termination` carries `sender: None`, so io's `Shutdown` always
  has `from: None` on macOS (io/README.md, 4).
- **Spawn** is `posix_spawn`, with the file actions Linux's spawn has
  (the working directory by `posix_spawn_file_actions_addfchdir_np`), the
  child in a group of its own (`POSIX_SPAWN_SETPGROUP`), its signal mask
  emptied and every signal the service ignores set back to its default
  (`POSIX_SPAWN_SETSIGMASK`, `POSIX_SPAWN_SETSIGDEF`).
- **A child on a terminal** is started by `fork` and `execve`, as
  `posix_spawn` can't do it. On XNU, `posix_spawn` runs its file actions
  before its `POSIX_SPAWN_SETSID`, so the terminal it opens never becomes
  the child's controlling terminal; and its `SETPGROUP` comes before
  `SETSID`, which then fails, the child already leading its group. Every
  string, list and descriptor the child needs is made before the
  `fork`. Between the `fork` and the `execve`, the child makes only
  async-signal-safe calls, and allocates nothing:
  1. `setsid`, which also makes the child a group of its own;
  2. the terminal opened, and made controlling by `TIOCSCTTY`;
  3. the same duplications as the file actions, then the working
     directory;
  4. the descriptors above its streams closed;
  5. the mask emptied and the defaults restored;
  6. `execve`.

  A failure there writes its error number to a close-on-exec pipe and
  `_exit`s. The parent reads that pipe to its end: no bytes, and the exec
  happened; an error number, and the spawn fails with it, once the parent
  has reaped the child.
- **A child's handle** is a kqueue of the child's own, close-on-exec,
  with `EVFILT_PROC` and `NOTE_EXIT` registered for its pid: readable once
  the child has exited. A child that exits before its registration
  (`ESRCH`) is still unreaped and the parent's: its handle is made
  readable by an `EVFILT_USER` event triggered in the same kqueue. The
  module keeps a table from each handle to its pid and whether it is
  reaped, process-wide behind a lock: a spawn adds the handle, the `Wait`
  that reaps marks it, its `Close` removes it. A `Wait` is `waitid` on
  the pid (`P_PID`, `WEXITED | WNOHANG`, `WNOWAIT` to observe). A `Signal`
  is `kill`, or `killpg` for the group the child leads; one after the
  reap answers `ESRCH`, as a pidfd's does, and never signals the number.
  A handle not in the table answers `EBADF`.
  A stop posts `NOTE_SIGNAL`, which this `NOTE_EXIT` registration does
  not request (`bsd/kern/kern_sig.c:2124-2125`,
  `bsd/kern/kern_event.c:1213-1218`). `waitid` skips stopped children
  without `WSTOPPED` (`bsd/kern/kern_exit.c:3270-3276`); a stop must
  leave `Wait` pending. To distinguish run 40003540284's `ECHILD`,
  Darwin reports diagnostic `Other(-10001)` for an already-reaped
  table entry, `Other(-10002)` for `waitid`'s own `ECHILD`, and
  `Other(-10003)` for a non-exit status, whose pid, code and status
  also print to stderr. Run 40010966195 answered `-10003`: `waitid`
  reported a state that is not an exit though only `WEXITED` was asked,
  so a stopped or continued child's state now leaves `Wait` pending, and
  any other non-exit code still fails with the diagnostic.
- **Signals to the service:** startup sets the signals the service
  watches (`SIGINT`, `SIGTERM`, `SIGWINCH` at a terminal, and `SIGHUP`
  when asked and not already ignored, `shell.md`, 6.3) to be
  ignored, which Darwin still reports to a kqueue, and opens a kqueue with
  an `EVFILT_SIGNAL` for each: the descriptor `ReadSignal` names, readable
  while one is pending. A `ReadSignal` takes one event without waiting and
  decodes its signal; deliveries of one signal before a read are one, as
  signalfd's are for a standard signal. Darwin reports only a signal sent
  to the process, never one sent to a thread (XNU `kern_sig.c`,
  `psignal_internal`), so `signal_current_thread` signals the process,
  and every signal kqueue in it hears that signal.
- **The rest:** `getentropy` for the seeds; `clock_gettime` with
  `CLOCK_MONOTONIC` and `CLOCK_REALTIME`; `getrusage`, whose resident
  size Darwin gives in bytes; `TIOCGWINSZ` and the terminal's modes as on
  Linux; `faccessat` with `AT_EACCESS` for a directory's access; the
  startup reads opened with `O_NOFOLLOW_ANY` where Linux's refuse links.
- **Names that differ only in case** are one name on a default APFS
  volume. The shell passes names as they are and answers as the volume
  does (a `Create` of `A` beside `a` is `AlreadyExists` there); it folds
  and checks nothing. The refusal of such names is the layer's that
  names things, where a checkout's name is checked.

## 6. The floor on each system

- **Linux's floor is `shell.md`, section 4's,** 6.12, for both backends. Without a
  ring the opcode probe cannot confirm it, so the readiness backend reads
  the kernel's release (`uname`) and refuses below it
  (`OpenError::BelowFloor`, whose message names the floor, not the
  ring). The release's first two numbers are compared, whatever follows
  them (`6.12.107+deb13-cloud-amd64` is 6.12); a release without them is
  refused (`OpenError::Setup(EINVAL)`).
- **macOS's floor is 15, Darwin 24,** the first with `O_RESOLVE_BENEATH`,
  which an open beneath a root needs: confirmed the same way, by the
  release (`uname`'s, Darwin's own: `24.6.0` is 24), whose first number
  is compared, and refused below it. One comparison serves both, a
  release against a system's floor, in `calls`, and its table of cases
  runs on Linux for both floors. No feature checks, on either.
- **Apple Silicon only:** `aarch64-apple-darwin`. The shell does not
  compile for Intel Macs: a `compile_error!` says so.

## 7. What is weaker without the ring

- **A stalled filesystem holds the loop.** File operations run at submit,
  so an NFS or FUSE mount that stalls stalls the service, and a cancel of
  a file operation always comes too late (kernel.md, 6).
- **Processes are groups, as they are with the ring:** a descendant that
  leaves its group escapes the close's kill. A cgroup or a contained
  tree (kernel.md, 6.6) is refused on macOS as unsupported.
- **A descriptor closed outside io leaves its waiters waiting.** The
  ring holds an in-flight operation's file, so the operation completes
  however its descriptor is closed; a readiness waiter holds only the
  number, and a close removes it from kqueue or epoll without an event.
  io closes a descriptor only once nothing is in flight on it (io/README.md,
  Settling; kernel.md, 7). An abandoned binary observer follows that rule:
  `Binary::abandon` queues cancels for its outstanding wait and stderr read;
  its driver reaps the targets and the cancels before the observer settles
  the tree and queues its closes. The host stays in the loop until empty
  on every backend. Its emergency destructor kills the tree and closes
  only quiescent descriptors; it leaves an outstanding record's descriptor
  open, and its leader unreaped while a wait or signal remains outstanding.
  A caller that needs full resource settlement must drive explicit abandonment.
- **A `PathNoFollow` open needs the entry readable on macOS.** Linux's
  `O_PATH` asks nothing of the entry; Darwin's nearest, `O_EVTONLY`,
  still checks read access (XNU `vfs_syscalls.c`, `vfs_subr.c`), so an
  entry the user cannot read refuses with `Permission`. skein-io meets
  this when it names a refusal: after a refused private open it opens the
  entry `PathNoFollow` to say what refused (`file_layer/private.rs`, the
  refusal on opening and `denied_load`), and on macOS that probe is
  itself refused, so the caller gets a plain `Permission` where Linux
  names the unsafe entry (`Refused` with what was found). A later fix: Darwin's handle holds the parent and the
  name, and `Stat` goes through `fstatat(AT_SYMLINK_NOFOLLOW)`.
