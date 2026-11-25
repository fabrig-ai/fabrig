# The kernel boundary

Provisional, 2026-10-03, revised 2026-10-19. The design of the boundary
below io: the records io submits to a backend, the completions the
backend hands back, and what every backend promises. The records live in
`skein-io`'s `kernel` module, whose module documentation states each rule
of the contract exactly; this document is the design behind it.

## 1. In one page

- **Records, not calls.** An operation goes down, and its completion
  comes up, handing the operation back. This is all the ring backend, the
  simulator and any later backend share, and all they implement.
- **Memory moves with the operation.** A buffer the kernel reads or
  writes is a `Box` inside the record, held by the backend until the
  completion hands it back.
- **Plain values only.** Kernel structures and error numbers stay in the
  backend; io sees addresses, counts and a skein error enum, and needs no
  `libc`.
- **Every operation completes exactly once,** cancelled or not, and a
  cancel is an operation of its own.
- **Sockets behave as Linux's do,** and the simulator matches them; one
  conformance suite holds every backend to the contract.
- **Files stay beneath a root.** A root is an open directory. Only `Open`
  takes a path, which the kernel resolves beneath its root; every other
  operation on a name acts on one entry of an open directory.
- **A tree's metadata, and its filesystem.** Links, modes, owners and
  extended attributes are read and written by name in an open directory,
  never through a link (`kernel-files.md`, section 2); a filesystem's status, its sync and
  btrfs's subvolumes take a descriptor, the subvolumes through a closed
  list of btrfs's ioctls (`kernel-files.md`, section 3).
- **Cgroups, and a helper for contained trees.** A cgroup is a directory
  and its events file, made, stopped and proved empty through records
  that name its descriptor; a spawn that asks for other ids or
  containment starts the service's own executable as a helper, which
  sets the tree up and reports how far it got (`kernel-cgroups.md`, section 1).

## 2. In skein

```
                    io (step code)
           Submit ↓              ↑ Complete
   ┌───────────────┬────────────────┬───────────────────────────────┐
   │ ring          │ sim            │ readiness                     │
   │ io_uring      │ the simulated  │ wait for readiness, make the  │
   │               │ kernel         │ syscall, complete             │
   │ production    │ tests          │ hosts without io_uring, macOS │
   └───────────────┴────────────────┴───────────────────────────────┘
```

io (io/README.md) is the only user of the records. The ring and the readiness
backend live in the shell (shell.md); the simulator is ../testing/simulator.md. The
records are step-code types, in `skein-io`, so that io can name them and
the backends can depend on io.

## 3. The records

```rust
// skein-io's kernel module, a sketch
// io -> kernel
pub struct Submit   { pub op: Token, pub kind: Op }
// kernel -> io: the operation handed back, and what came of it
pub struct Complete { pub op: Token, pub kind: Op, pub result: Result<Done, Error> }

// one success shape per operation
pub enum Done {
    Nothing, Count(u32), Fd(Fd), Accepted { fd: Fd, peer: Addr }, Bound(Addr),
    Stat(Stat),                       // kind, size, mode, owners, links, inode, times (6.1)
    Subvolume(Subvolume), Filesystem(Filesystem),  // btrfs's, and a filesystem's status (6.5)
    Spawned { pidfd: Fd, sender: Sender, report: Option<Fd> },  // report: the helper's (6.6)
    Exit(Exit), Signalled(Signalled),
    Cgroup { dir: Fd, events: Fd }, Populated(bool), Counts(Counts),  // cgroups (6.6)
    Usage(Usage), Window(Window),     // the process's own (6.3)
}

pub enum Op {
    // sockets
    Socket   { family: Family },
    Bind     { fd: Fd, addr: Addr },
    Listen   { fd: Fd, backlog: u32 },
    Accept   { fd: Fd },
    Connect  { fd: Fd, addr: Addr },
    Recv     { fd: Fd, buf: Box<[u8]> },
    Send     { fd: Fd, bytes: Box<[u8]>, from: u32 },
    Shutdown { fd: Fd },
    Close    { fd: Fd },
    // files, beneath a root (section 6.1)
    Open          { root: Fd, path: Box<[u8]>, how: OpenHow },   // Read, Directory, Create
    Read          { fd: Fd, buf: Box<[u8]>, at: u64 },
    Write         { fd: Fd, bytes: Box<[u8]>, from: u32, at: u64 },
    Append        { fd: Fd, bytes: Box<[u8]>, from: u32 },       // at the end (6.1)
    Truncate      { fd: Fd, length: u64 },                       // ftruncate (6.1)
    Sync          { fd: Fd },
    Stat          { fd: Fd },
    StatEntry     { dir: Fd, name: Box<[u8]> },                  // the entry itself, never a link's target
    Rename        { from_dir: Fd, from: Box<[u8]>, to_dir: Fd, to: Box<[u8]>, no_replace: bool },
    Remove        { dir: Fd, name: Box<[u8]>, directory: bool },
    MakeDirectory { dir: Fd, name: Box<[u8]>, mode: u32 },     // less the umask (io/files.md, 11)
    List          { fd: Fd, entries: Box<[Entry]>, names: Box<[u8]> },  // synchronous
    Lock          { fd: Fd },                                    // exclusive, without waiting; synchronous
    // a tree's metadata, by name in an open directory (section 6.4)
    ReadLink        { dir: Fd, name: Box<[u8]>, buf: Box<[u8]> },                          // synchronous
    MakeLink        { dir: Fd, name: Box<[u8]>, target: Box<[u8]> },
    HardLink        { from_dir: Fd, from: Box<[u8]>, to_dir: Fd, to: Box<[u8]> },
    SetMode         { dir: Fd, name: Box<[u8]>, mode: u32 },                               // synchronous
    SetOwner        { dir: Fd, name: Box<[u8]>, user: u32, group: u32 },                   // synchronous
    ListAttributes  { dir: Fd, name: Box<[u8]>, names: Box<[u8]> },                        // synchronous
    GetAttribute    { dir: Fd, name: Box<[u8]>, attribute: Box<[u8]>, value: Box<[u8]> },  // synchronous
    SetAttribute    { dir: Fd, name: Box<[u8]>, attribute: Box<[u8]>, value: Box<[u8]> },  // synchronous
    RemoveAttribute { dir: Fd, name: Box<[u8]>, attribute: Box<[u8]> },                    // synchronous
    // the filesystem a descriptor lies on, and btrfs's subvolumes (section 6.5); all synchronous
    FilesystemStatus  { fd: Fd },
    SyncFilesystem    { fd: Fd },
    SubvolumeCreate   { dir: Fd, name: Box<[u8]> },
    Snapshot          { source: Fd, dir: Fd, name: Box<[u8]> },
    SubvolumeDestroy  { dir: Fd, name: Box<[u8]> },
    SubvolumeReadOnly { fd: Fd, read_only: bool },
    SubvolumeInfo     { fd: Fd },
    SubvolumeChildren { fd: Fd, from: u64, children: Box<[Child]> },
    SubvolumePath     { fd: Fd, directory: u64, tree: u64, path: Box<[u8]> },
    // processes (section 6.2)
    Spawn     { spawn: Box<Spawn> },                         // synchronous
    Wait      { pidfd: Fd, reap: bool },
    Signal    { pidfd: Fd, signal: Signal, to: Target },     // synchronous
    PipeRead  { fd: Fd, buf: Box<[u8]> },
    PipeWrite { fd: Fd, bytes: Box<[u8]>, from: u32 },
    // cgroups (section 6.6); all synchronous but CgroupChanged
    MakeCgroup        { parent: Fd, name: Box<[u8]>, setup: CgroupSetup },  // limits, bounds, owner
    EnableControllers { cgroup: Fd, controllers: Controllers },
    EnterCgroup       { cgroup: Fd },                         // the process itself
    CgroupCounts      { cgroup: Fd },
    SignalCgroup      { cgroup: Fd, signal: Signal },         // each process, through a pidfd
    KillCgroup        { cgroup: Fd },
    Populated         { events: Fd },
    CgroupChanged     { events: Fd },                         // the ring's poll
    RemoveCgroup      { cgroup: Fd, name: Box<[u8]> },        // with those beneath, deepest first
    // the process's own (section 6.3)
    ReadSignal { fd: Fd },
    Usage,                                                   // synchronous
    Window     { fd: Fd },                                   // synchronous
    Cancel     { target: Token },
}

// what the records on files answer (6.1, 6.5)
pub struct Stat {
    pub kind: Kind, pub size: u64,
    pub mode: u32,                   // 0o7777 at most; a Create's or a MakeDirectory's, 0o777
    pub owner: u32, pub group: u32, pub links: u32,
    pub inode: u64, pub device: Device, pub special: Device,   // special: what a device is
    pub changed: FileTime, pub modified: FileTime,
}
pub enum Kind { File, Directory, Symlink, CharDevice, BlockDevice, Fifo, Socket, Other }
pub struct Device   { pub major: u32, pub minor: u32 }
pub struct FileTime { pub seconds: i64, pub nanos: u32 }
pub struct Child    { pub tree: u64, pub directory: u64 }
pub struct Subvolume {
    pub tree: u64, pub uuid: [u8; 16], pub parent_uuid: Option<[u8; 16]>,
    pub read_only: bool, pub generation: u64,
}
pub struct Filesystem { pub kind: FilesystemKind, pub size: u64, pub available: u64 }
pub enum FilesystemKind { Btrfs, Other(u64) }   // Other: the kernel's magic number
```

The completion hands back the operation it answers, buffers and all,
beside its result: a `Done` of the one shape that operation succeeds
with, or an `Error`. Every record comes back up, whatever happened. An
error means the operation did nothing usable: a failed `Socket` or
`Accept` made no descriptor.

## 4. Memory and values

- **Memory moves with the operation.** io moves a buffer down inside the
  record, and the backend holds the record until the operation's
  completion, which moves the buffer back up: filled, or with the count
  sent. A `Recv` comes back with its buffer filled up to the count, a
  `Send` with its bytes untouched.
  - While the kernel holds the buffer, no other code can name it: the
    compiler's half of the rule (../../foundation/programming-model.md, 6.2).
  - The backend's `unsafe` is the other half. It takes addresses only
    from records it holds, and gives a record back only after the
    operation's completion. It never drops, copies or replaces a `Box`.
  - **The backend allocates nothing that goes up.** A `Spawn` comes back
    with the parent's end of each requested pipe written into its own
    pipe table, and its completion carries only the child's descriptor.
    What a process owns is then always allocated within its own calls
    (../testing/simulator.md, 5).
- **Kernel structures belong to the backend.** A socket address, a
  `statx` buffer, a `siginfo`, an `rusage` and an `open_how` live in the
  backend's in-flight table, beside the record. They are decoded into
  plain values (`Addr`, `Stat`, `Exit`, `Usage`) before they go up. A
  backend for another kernel translates.
- **A path or a name is bytes,** a `Box<[u8]>` in the record with no NUL
  in it. The NUL-terminated string the kernel reads is the backend's own
  copy, held beside the record until its completion.
- **Errors cross as a skein enum:** the errors io handles by name, plus an
  `Other` code. Each backend maps its kernel's error numbers onto it, per
  operation: the same number can mean different things on a cancel and
  on a receive.
- **Socket options are backend defaults, not records,** until a service
  pulls one: every descriptor is close-on-exec, a socket that binds gets
  `SO_REUSEADDR`, an IPv6 socket gets `IPV6_V6ONLY` (families never mix:
  it neither binds nor reaches an IPv4-mapped address), connected and
  accepted sockets get `TCP_NODELAY`, and a send never raises `SIGPIPE`.

## 5. Completion and cancelling

- **Every operation completes exactly once,** cancelled or not, with its
  token. Completions arrive in any order, even on one descriptor.
- **Single-shot operations only, to start:** one submission, one
  completion, and every cancel takes an operation slot of its own. The
  completion queue can then be sized from io's operation slab, so it
  cannot overflow, and every receive and accept is a choice io made while
  it had room.
- **A cancelled operation still completes:** as cancelled, or with what it
  did before the cancel landed. Until then, io keeps the entity
  *settling* (../../foundation/programming-model.md, 5.3).
- **A cancel completes too,** before or after its target:
  - it stopped the target, which then completes cancelled. A stopped
    receive, send or accept took nothing: what had arrived waits for the
    next one. A stopped connect may still have reached its peer, which
    sees the connection end when io closes it;
  - too late: the target had completed or could not be stopped, and
    completes with its own result, or cancelled if the kernel interrupted
    it;
  - not submitted at all, when the backend failed, the target running on.
- **A process with several owners of records** renames them at its
  hand-off: each owner numbers its own operations, and the kernel sees one
  token space. `kernel::Routes`, in `skein-io`, gives each record a
  process-wide token as it leaves its owner's queue, gives the owner its
  own token back with the completion, and carries a cancel to its target's
  process-wide token. A target that has completed while its cancel waits
  unpublished keeps its route until the cancel goes. It holds routes for
  a stated number of operations; past that, or with no room in the shared
  queue, a record waits in its owner's queue.

## 6. What every backend does

- **Sockets behave as Linux's do,** and the simulator matches them:
  - a bind answers with the address bound, its port chosen when it asked
    for port 0; two sockets may bind one address, and the second listen
    fails;
  - a full accept queue delays a connect, never refuses it (refused means
    nothing listened there), until it times out; a failed accept takes no
    waiting connection, and a connection closed or reset while it waits
    is still accepted;
  - a send's count is what the kernel accepted, and a half-close goes out
    behind every completed send; receiving still works after it;
  - a receive of zero bytes means the stream ended, which is not proof of
    a graceful close;
  - a reset fails one operation: a send at once, a receive after the
    bytes already received; after it, receives give zero bytes and sends
    and half-closes fail. An end that already received the peer's end of
    stream hears of no reset, and a send after the peer closed may
    succeed, its bytes lost, until the peer's reset arrives, then fails
    without a reset, the connection closed;
  - a descriptor is closed only by a close. Closing with unread data
    resets the peer; closing a listener resets the connections waiting on
    it.
- **Some operations are synchronous:** spawning, signalling, every
  operation on a cgroup but the wait for its change (`kernel-cgroups.md`, 1), listing a
  directory, locking a file without waiting, reading the process's
  resource usage and its terminal's size, reading a link, setting a mode
  or an owner, the extended attributes, and every operation on the
  filesystem (`kernel-files.md`, 3). None of these is a ring
  operation at the kernel floor (shell.md), and some are not ring
  operations at all. The backend performs them when
  they are submitted and completes them at the next reap, in the same
  records, so io cannot tell the difference.
- **A synchronous operation blocks the loop for its call,** which no
  `Cancel` stops: microseconds on a local filesystem, without bound on
  one that stalls. Some are long even on a healthy disk: btrfs's writing
  ioctls commit a transaction, and `SyncFilesystem` lasts until the
  filesystem's dirty data is written (`kernel-files.md`, 3). A service that must not
  stall makes them in a process of its own, and talks to it as to any
  child. On the readiness backend every operation on files is
  synchronous so (readiness.md, 7).
- **Timers are not operations.** The shell waits for completions with one
  timeout: the earliest deadline over every layer.

### 6.1 Files beneath a root

Moved to [`kernel-files.md`](kernel-files.md), section 1.

### 6.2 Processes

- **A child is a pidfd from its spawn,** and the leader of a process
  group of its own (io/processes.md). On macOS its handle is a descriptor of
  the shell's own that is readable once the child has exited, and
  `posix_spawn` makes it (readiness.md, 5): the records are the same. `Spawn` is synchronous: glibc's
  `pidfd_spawn`, with the group set in its attributes, and the cgroup if
  the spawn names one. `Spawn` carries io/processes.md 3's fields, each io token
  its descriptor; one that asks for ids, containment or a reach
  (io/processes.md, 6) starts the helper (`kernel-cgroups.md`, 1). A spawn whose reach the
  backend cannot enforce fails `Reach(Unenforced)` before anything is
  spawned, or once the helper has said why, and never runs with less:
  macOS fails every one, `Unsupported`. A `Null` entry is an open of `/dev/null`, read and write, at
  its child descriptor in the file actions (`posix_spawn_file_actions_addopen`),
  on Linux and macOS; the simulator gives the child a descriptor that
  reads its end and takes every write. Its Debug prints each env entry's
  name, never its value.
- **`Wait` observes an exit, or reaps it.** Both are the ring's `waitid`
  on the pidfd. A `Wait` that does not reap (`WEXITED | WNOWAIT`)
  completes with the child's exit and leaves it a zombie, which keeps its
  PID, and so its group's ID, from being given to another process. A
  `Wait` that reaps (`WEXITED`) follows one that observed the exit, at
  io's close of the child, and completes at once. Until it is reaped, a
  child's resource usage is not among the process's children's (6.3).
- **`Signal` goes to the child, or to its group,** both through the
  child's pidfd: `pidfd_send_signal`, with no flag for the child, and
  with `PIDFD_SIGNAL_PROCESS_GROUP` for the group the child leads, which
  the kernel floor has. No record carries a PID: the pidfd names the
  child, and through it the group the child leads, so no signal reaches a
  stranger. `Interrupt` sends `SIGINT`, `Terminate` sends `SIGTERM`,
  `Kill` sends `SIGKILL`, `Stop` suspends with `SIGSTOP`, and `Continue`
  resumes with `SIGCONT` (a no-op unless stopped); an interrupt, like a
  termination, may be handled by its recipient. The unreaped leader is still a member of its group, so a
  signal to a group whose other members are gone succeeds and changes
  nothing.
- **Pipes** are the parent's ends of what `Spawn` made, read and written
  through the ring, as sockets are, with a pipe's errors.

### 6.3 The process's own

- **`ReadSignal`** returns `Signalled::Termination { signal, sender }`, where
  `sender: Option<Sender>` is an opaque process identity compared only for
  equality with the `Sender` returned by `Spawn`. io maps a live spawned
  sender to its child token (io/README.md, 4). Linux reads signalfd's `ssi_pid`;
  Darwin's kqueue supplies no sender.
  It reads one blocked signal from the signalfd the shell
  opened at startup: `SIGINT`, `SIGTERM`, `SIGWINCH` for a service at
  a terminal, and `SIGHUP` for one that asks (io/README.md, 4; shell-startup.md, 1.3).
- **`Usage` reads what the process has used,** in one synchronous call:
  `getrusage` for the process itself (`RUSAGE_SELF`) and for its reaped
  children (`RUSAGE_CHILDREN`). Each part answers user and system CPU
  time, as `Duration`s, and the peak resident size, in bytes (the kernel
  counts it in KiB on Linux, which the backend multiplies, checked, and
  in bytes on macOS). The
  children's CPU is the sum over every child reaped so far, and over what
  each of them reaped in turn; their peak is the largest any one of them
  reached. A child that has exited but is not yet reaped is in neither
  part.
- **`Window` reads a terminal's size,** rows and columns, from the
  descriptor it names (`TIOCGWINSZ`). The shell found at startup that the
  descriptor is a terminal (shell-startup.md, 1.3); one that has since hung up
  fails with the kernel's error, as `Other`.

### 6.4 A tree's metadata

Moved to [`kernel-files.md`](kernel-files.md), section 2.

### 6.5 The filesystem: its status, its sync, btrfs's subvolumes

Moved to [`kernel-files.md`](kernel-files.md), section 3.

### 6.6 Cgroups, and the helper

Moved to [`kernel-cgroups.md`](kernel-cgroups.md), section 1.

## 7. Broken invariants

Some mistakes io never makes, so a backend may assume they never happen:

- an invalid record (an empty receive buffer, a send with nothing left,
  a name with a `/` in it, a buffer below its operation's least);
- a token already in flight, or a cancel of a cancel;
- an address of the wrong family;
- more than one receive, send or accept in flight on a socket, or
  anything beside a connect;
- any operation but close after a failed connect, a connect on a socket
  that is not fresh, a listen on an unbound socket;
- a half-close on a socket that is not a connection, or during a send;
- an operation on files on a socket's descriptor, or one on sockets on a
  file's; a read on a descriptor not opened to read, a write on one not
  opened to create, a list on one opened to create;
- a cancel of a stat, a rename, a removal, a new directory, a listing, a
  lock, or any operation of `kernel-files.md`, sections 2 and 3;
- a lock, or an operation of `kernel-files.md`, sections 2 and 3, on a socket, a pipe
  or a path-only descriptor;
- a close with anything else in flight on its descriptor, an operation on
  files being on every descriptor it names;
- an `Append` beside another in flight on its descriptor, or on one not
  open to write;
- a `Wait` beside another on the same pidfd, or one that reaps before a
  `Wait` has reported the exit; any operation on a pidfd after its reap
  but its close;
- a cancel of a spawn, a signal, a usage or a window, or of a cgroup
  record but `CgroupChanged`;
- a `CgroupChanged` beside another on the same descriptor; a given
  descriptor or a pipe at a number another has, or at one the helper's
  own would take; a view without a pid namespace.

The simulator fails the world on each; the ring checks the first two at
submit.

## 8. Testing

The contract's executable form is the conformance suite, which holds
every backend and the simulator to this document, scenario by scenario:
[`../testing/conformance.md`](../testing/conformance.md), a document of
its own since it outgrew this one's size.

## 9. Open questions

- **Fixed files** change what an `Fd` names: a slot in the ring's table
  instead of a descriptor. Decide when it has been measured.
- **Root's links on Linux** (`kernel-files.md`, 1): whether Linux's no-follow opens and
  startup reads pass the system's own links as macOS's do
  (readiness.md, 5), such as `/var/run -> /run`. Default: no; Linux's
  refuse every link (`RESOLVE_NO_SYMLINKS`), and its temporary and home
  directories lie through none.

## 10. Not built yet

What of this document is not built is in [`kernel.status.md`](kernel.status.md).
