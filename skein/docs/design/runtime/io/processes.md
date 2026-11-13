# Processes

- **Spawn** takes a program, its arguments, its environment, a working
  directory beneath a root, and which pipes to make. It answers with a
  child and its pipes. The backend also returns an opaque `kernel::Sender`,
  kept by io beside the child's token until its `Exited` event. It is
  compared only for equality with signal senders (`README.md`, section 4); no numeric PID
  enters io's events or its child entity.
  - The ring adapter uses glibc's `pidfd_spawn`, so the child is a pidfd from
    the start. Its file actions install requested pipes at chosen child
    descriptors and close everything else on exec. The kernel's own check
    then stops a reused PID from being signalled. `std::process` is not used.
  - The child starts in a process group of its own, so a terminal's
    interrupt reaches the service alone, which decides what its children
    hear (`README.md`, section 4).
  - A given descriptor is an entry of the descriptor table whose way is
    `Given(token)` (3): one io holds, a held listener (`sockets.md`, 2) so far,
    which the child inherits at the entry's descriptor; io keeps its own,
    and `Spawned` names no token for it.
  - A null descriptor is an entry whose way is `Null`: the child's
    descriptor opened on `/dev/null`, for reading and writing, by the
    spawn's file actions. io holds nothing for it and `Spawned` names no
    token for it; a child given null descriptors for 0, 1 and 2 reads
    nothing and says nothing to anyone, and writes to them never fail.
  - An inherited descriptor is an entry whose way is `Inherit`: the
    child's descriptor 0, 1 or 2 is the service's own descriptor of that
    number, as the service has it, so that a front end lends the
    person's terminal to a program of its own (`../shell.md`, 6.3, *The
    terminal lent to a child*). The spawn's file actions leave it open
    across the exec where they would open `/dev/null`. io holds nothing
    for it and `Spawned` names no token for it. An `Inherit` entry at
    any other child descriptor is refused before anything is made, as a
    descriptor named twice is (3). The simulator gives the child the
    parent's own descriptor at that number.
  - A requested pipe has a chosen child descriptor and direction. Its
    parent end is a one-way stream token in the `Spawned` event, in request
    order. Each pipe consumes one entity slot alongside the child.
- **Exit** is a wait on the pidfd, through the ring (`waitid`). It
  observes the exit without reaping the child (`WNOWAIT`): the child
  stays unreaped, holding its PID, until io closes it.
- **Signals** (`Interrupt`, `Terminate`, `Kill`, `Stop`, or `Continue`) go
  to the child alone, or to its process group: `Signal` names which (`Target::Child` or
  `Target::Group`). Both go through the child's pidfd,
  `pidfd_send_signal`, the group's with the kernel's
  process-group scope, which reaches the group the child leads from its
  spawn (../kernel.md, 6.2). The group's ID is the child's PID, which the
  kernel cannot give another process while the child is unreaped. A
  signal to the group therefore reaches the child and whatever it started
  that stayed in its group, and never a stranger, even once the child
  itself has exited.
  - **One signal in flight.** A child has at most one signal in flight
    (`sockets.md`, 4). A `Signal` sent while one is in flight, or once its owner has
    begun to close the child, is dropped, and nothing tells the owner. An
    owner that must be sure its signal went closes the child instead:
    closing kills its group.
- **A child is *closed*** once it has exited and its pipes and pidfd are
  closed. It has one terminal event, which comes after all of those.
  Closing a child sends `Kill` to its group, whether or not the child
  itself has exited, then reaps it: what it started in its group ends
  with it. The exit and pipe closures still settle before its `Closed`
  event.
- **A child spawned alone** (`SpawnAlone`, the same fields and events as
  `Spawn`) is one whose group io never signals: closing it sends `Kill`
  to the child alone while it runs, and none once it has exited, then
  reaps it as any child. What it started stays, in its group, and
  outlives it, as a descendant that left the group does. It is for a
  child that hands work to a program meant to outlive it, as an opener
  starts a browser; its owner can still signal its group by `Signal`.
- **A timeout is the owner's** (../../../foundation/programming-model.md, 4). An owner that
  gives a child a deadline signals its group when it passes, then closes
  it. Whatever the child started that holds its pipes then ends too, and
  the pipes with it, so a command that timed out is answered within its
  deadline and io's close deadline, not when the last of its descendants
  chooses to exit.
- **A group is not a tree.** A descendant that leaves the group
  (`setsid`, `setpgid`) escapes its signals and outlives its child, and
  nothing bounds or counts what a group may use: a descendant reparented
  away from the child is in no one's resource usage (1). A child
  spawned into a cgroup of its own is a tree (3): stopped as a whole,
  proved empty, bounded and counted.

## 1. Resource usage

A service may report what it used, at its end: CPU time and peak
resident size, its own and its children's. io reads them from the kernel
for it.

- **One request, one terminal.** `Usage` answers `Usage`, or that the
  kernel would not read it (`Unread`, with its error), with two parts:
  the process's own, and its reaped children's. Each holds user and
  system CPU time, and the peak resident size in bytes. The children's
  CPU is the sum over the children reaped so far, and their peak is the
  largest one of them reached, as the kernel counts them.
- **It holds no entity.** The kernel reads it at submit (../kernel.md, 6.3).
  One may be in flight at a time, in an operation slot of its own; a
  second while one is in flight is the layer above's bug, asserted.
- **A child counts once io has reaped it,** at its close (`processes.md`),
  not when it exits: a service that wants every child in its report asks
  after its last child's `Closed`. What a child started counts only if
  the child itself reaped it: a descendant reparented away, as one the
  group's `Kill` ends after the child exited, is not counted (`processes.md`).
- **When to ask is the service's.** A service that writes a report at its
  end asks once its children have closed and before the stream it writes
  the report to closes (../shell.md, 12).

## 2. Cgroups

A cgroup is an io entity, beside children: a directory of the kernel's
cgroup v2 hierarchy, which holds processes, counts what they use and
bounds it, and is stopped and proved empty as a whole. A service makes
its cgroups beneath its own, and spawns children into them (3).

```rust
// skein-io's cgroups, the protocol side; in io's Request as
// Cgroup { owner: Token, op: cgroup::Op }, answered by cgroup::Event
pub enum Op {
    Make    { parent: Token, name: Box<[u8]>, limits: Limits, delegate: Option<Owner> },
    Enable  { cgroup: Token, controllers: Controllers }, // for its children
    Enter   { cgroup: Token },                           // the service moves itself in
    Counts  { cgroup: Token },
    Signal  { cgroup: Token, signal: Signal },           // every process in it (6.4)
    Kill    { cgroup: Token },                           // every process, at once (6.4)
    Empty   { cgroup: Token },                           // answered once none is left (6.4)
}
pub struct Limits     { pub memory: Option<u64>, pub processes: Option<u32> }
pub struct Owner      { pub user: u32, pub group: u32 }
pub struct Controllers { pub cpu: bool, pub memory: bool, pub processes: bool }
pub struct Counts {
    pub user: Duration, pub system: Duration,         // CPU, of every process it held
    pub peak: Option<u64>, pub oom_kills: Option<u64>, // with the memory controller
}
pub enum Event {
    Made { owner: Token, cgroup: Token }, Entered { owner: Token },
    Enabled { owner: Token }, Counts { owner: Token, counts: Counts },
    Signalled { owner: Token }, Killed { owner: Token }, Empty { owner: Token },
}
// a refusal is io's Failed; a cgroup's close, its Closed
```

- **The service's own cgroup** is opened by the shell at startup
  (../shell.md, 6.4), and io adopts it as it adopts a root. io never kills
  or removes an adopted cgroup: closing it closes its descriptors.
- **`Make`** makes a cgroup by name beneath another io holds, and opens
  it. It sets its limits (`memory.max`, `pids.max`), whose controllers
  must be enabled in the parent, and always `memory.oom.group` where the
  memory controller is: the kernel's out-of-memory kill then takes the
  whole cgroup, not one process of it. With `delegate`, it gives the
  cgroup to another user, as the kernel's delegation does: the directory
  and the files the kernel names delegable (`cgroup.procs`,
  `cgroup.subtree_control` and the rest) change owner, which needs
  privilege, and its depth and number of descendants are bounded
  (`Limits::cgroup_depth`, `Limits::cgroup_descendants`), so that what
  the other user makes beneath it stays within what io removes.
- **`Enable`** enables controllers for a cgroup's children. The kernel
  refuses it while the cgroup itself holds a process, so a service enables
  controllers on its own cgroup only after `Enter`ing one beneath it.
  The usual start: make `self` beneath the service's cgroup, enter it,
  enable the controllers, then make the trees' cgroups beside `self`.
- **`Enter`** moves the service's own process into a cgroup. The kernel
  allows it where the service may write both that cgroup's `cgroup.procs`
  and their common ancestor's.
- **`Counts`** reads the CPU time of every process the cgroup and those
  beneath it held (`cpu.stat`), and, with the memory controller, the
  peak of their memory together (`memory.peak`) and how many the kernel
  killed for want of it (`memory.events`). Read once the cgroup is empty,
  they are its final counts. A cgroup's processes are counted here, not
  in the service's `Usage` (1), whatever reparents them.
- **Closing a cgroup io made** kills what it holds, if anything, waits
  until it is empty (4), removes the cgroups beneath it, deepest first,
  then removes it. Its `Closed` comes after all of that, and is the proof
  that the cgroup and everything in it are gone. A process the kernel
  cannot kill (one in uninterruptible sleep on a filesystem that stalls)
  holds the close open, and io waits with it (`README.md`, section 7). `Abort` of a
  cgroup is its close.
- **Bounds.** A cgroup takes one entity slot and holds two descriptors,
  its directory and its `cgroup.events`, and the name it was made with.
  The operations on it are synchronous but `Empty` (../kernel.md, 6.6); one
  `Empty` may be in flight on a cgroup, and a second is the layer above's
  bug, asserted.

## 3. Spawning a tree

A *tree* is a child spawned into a cgroup of its own, and everything it
starts. `Spawn` gains what trees need; a spawn that asks for none of it
is a child as in `processes.md`.

```rust
pub struct Spawn {
    pub program: Box<[u8]>, pub args: Box<[Box<[u8]>]>, pub env: Box<[Box<[u8]>]>,
    pub dir: Dir,                       // beneath an io root, or a path in the view
    pub pipes: Box<[Pipe]>,             // the child's descriptor table: pipes and given
    pub cgroup: Option<Token>,          // a cgroup io holds; the service's own if none
    pub ids: Option<Ids>,               // the user and groups the command runs as
    pub contain: Option<Box<Contain>>,  // namespaces, and a view of the file system
}
pub enum Dir    { Beneath { root: Token, path: Box<[u8]> }, InView(Box<[u8]>) }
pub struct Ids  { pub user: u32, pub group: u32, pub groups: Box<[u32]> }
pub enum Way   { In, Out, Given(Token), Null, Inherit }  // Given: a held listener (3.2) or an io root; Null: /dev/null; Inherit: the service's own 0, 1 or 2 (1)
pub struct Contain {
    pub users: Box<[Map]>, pub groups: Box<[Map]>,  // the user namespace's maps
    pub pid: bool,                                  // a pid namespace of its own
    pub network: Network,
    pub hostname: Option<Box<[u8]>>,                // a uts namespace with this name
    pub view: Option<View>,
    pub rlimits: Box<[Rlimit]>,                     // lowered, never raised
}
pub struct Map { pub inside: u32, pub outside: u32, pub count: u32 }
pub enum Network { Loopback, Host }
pub struct View {
    pub base: Base,                     // the view's root
    pub binds: Box<[Bind]>,             // io roots, where the tree sees them
    pub read_only: Box<[Box<[u8]>]>,    // paths within writable binds
    pub scratch: Box<[Scratch]>,        // empty, writable, in memory
}
pub enum Base    { Empty, Root { root: Token, path: Box<[u8]>, writable: bool } }
pub struct Bind  { pub root: Token, pub path: Box<[u8]>, pub at: Box<[u8]>, pub writable: bool }
pub struct Scratch { pub at: Box<[u8]>, pub size: u64 }
pub struct Rlimit { pub resource: Resource, pub soft: u64, pub hard: u64 }
```

- **Into a cgroup.** A child spawned with `cgroup` is in it from its
  first instruction (the kernel's `CLONE_INTO_CGROUP`), so nothing it
  starts is ever outside it, a double fork included. One cgroup may hold
  several children: smith's run, whose commands share a cgroup beneath
  its host's, is one cgroup, or one with a cgroup per command beneath it
  (`README.md`, section 7's trees in one process are this).
- **As other ids.** `ids` runs the command as that user, with that group
  and those supplementary groups, which replace the service's; an
  activator running as root starts a service as its user so. Outside a
  user namespace this needs privilege; inside one, `ids` are the
  namespace's, root of the namespace if absent.
- **Given descriptors.** A descriptor table entry whose way is
  `Given(token)` names a descriptor io holds, a held listener (`sockets.md`, 2) or
  an io root, an open directory, at the entry's child descriptor; with
  the pipes, they are all the child inherits. It is a way of the table's
  entries, not a field of its own, so that it is added without changing
  a `Spawn` anyone writes. Built first for held listeners, on every
  spawn, a tree's or not: each backend's `posix_spawn` file actions,
  Linux's and macOS's, `dup2` io's descriptor to the child's number,
  which leaves it open across the exec, and io keeps its own. The
  caller leaves its `parent` empty, as a pipe's; io fills it with its own
  descriptor for the backend, and `Spawned` names tokens for the pipes
  alone, in request order. A spawn whose table names one child descriptor
  twice, or a token which is neither, is refused before anything is
  made. The simulator puts the same listener in the child's socket
  table, so that the child's claim (`sockets.md`, 2) accepts what the parent's
  backlog held. A helper that works beneath a store the
  service opened takes it so (hearth/docs/design/domain/store/tree-helper.md, 2).
- **Contained.** A contained child has a user namespace of its own,
  mapped as `users` and `groups` say, and with it mount, ipc and cgroup
  namespaces of its own; a pid namespace if `pid`; a network namespace
  with only loopback, up, unless `Host`; a uts namespace, with its
  hostname, if one is given. It runs with `no_new_privs`, with
  `rlimits` lowered, and without the service's supplementary groups.
  - **The maps** may name the service's subordinate ids
    (`/etc/subuid`, `/etc/subgid`), which `newuidmap` and `newgidmap`
    map, and the service's own ids. The service's own ids need not be
    mapped: unmapped, they are the overflow id inside, and the tree
    reaches the service's files only as any other user would. A map that
    names only the service's own group leaves the command the service's
    supplementary groups, which the namespace cannot change then
    (../kernel.md, 6.6). A tree with a view must map the namespace's 0.
  - **No fallback.** A machine without user namespaces, or without
    `newuidmap` where the maps need it, fails a contained spawn at its
    namespaces, and so does a service that runs with `no_new_privs`
    where the maps need `newuidmap`, which the shell checks at startup
    (../shell.md, 6.4). A service whose own `/proc` or `/sys` has parts
    mounted over, as a unit hardened by its supervisor may, fails a view
    for its new ones: the kernel lets a user namespace mount them only
    where the host's are whole. A service checks at startup, by spawning
    a tree with a view, when it must not start without them.
- **The view** is the file system a contained tree sees, composed in its
  mount namespace before the command starts:
  - **its root:** an empty one in memory (`Empty`), or a directory io
    holds, writable or not, given as the io root and its path on the
    host;
  - **binds:** directories io holds, each where the tree sees it,
    writable or not; within writable ones, `read_only` paths, such as a
    repository's git directory; and `scratch`, writable directories in
    memory, each bounded;
  - **always:** a new `/proc`; `/sys`, new and read-only, or with `Host`
    the host's, read-only; a minimal `/dev` (`null`, `zero`, `full`,
    `random`, `urandom`, `tty`, its own `pts` and `ptmx`, `shm`, and the
    `fd`, `stdin`, `stdout` and `stderr` links); a private `/tmp`.
  - **Paths stay beneath the view.** Each place a mount goes is resolved
    beneath the view's root, following no link; one missing is made only
    where the root is writable or `Empty`. A bind is found by its path
    on the host, as the service's own user, and checked to be the
    directory io holds (its device and inode); what was checked is what
    is mounted, so a path changed under it fails the view instead of
    binding something else. Once composed, the root is read-only unless it was
    given writable, and the command starts at `dir` in the view.
- **Spawned, or failed.** io answers `Spawned` once the command is
  executing, after the whole of the setup, or `Failed` naming the stage
  that failed (`Spawn(Stage)`: the cgroup, the ids, the namespaces and
  their maps, the view, or the command itself) and the kernel's error,
  then `Closed` once the child is reaped. A helper killed during the
  setup, by its owner's deadline or for want of memory in its cgroup,
  reports nothing: io answers `Spawned`, and the child's exit then says
  the signal.
- **The child io holds** is the tree's first process. When the spawn
  asks for ids or containment, it is a helper (../kernel.md, 6.6) that
  relays the command: what it is signalled it passes on, and it exits
  as the command did, with its status, or by its signal. In a pid
  namespace of its own, the command's end ends the tree: the kernel kills
  what the command left in the namespace.
- **Bounds.** A spawn's maps, binds, read-only paths, scratches, rlimits
  and the bytes of their paths are bounded (`Limits::contain`), so that
  the whole of it fits the helper's 64 KiB (../kernel.md, 6.6); past them, the
  spawn is refused at io's entrance. A given root takes no slot of its
  own. The helper's memory is its own process's, outside the service's
  measured heap.

## 4. Stopping a tree, and its end

| Step | The request | What reaches the tree |
|---|---|---|
| polite | `Signal { child, Terminate, Child }` | the first process, which passes it to the command |
| terminate | cgroup `Signal { cgroup, Terminate }` | every process in the cgroup and beneath it |
| kill | cgroup `Kill` | every process in it and beneath it, at once (`cgroup.kill`) |

- **Each step, and its deadline, is the owner's,** as everywhere in io
  (../../../foundation/programming-model.md, 4): a tree's owner signals politely, gives the
  command its time, terminates, kills.
- **Terminate is one pass,** over the processes the cgroup and those
  beneath it list. Each is opened as a pidfd and checked to be in the
  tree still before it is signalled, so a PID reused meanwhile is never
  signalled. A process forked during the pass may be missed: kill is the
  step that leaves none. The command may hear the signal twice, from the
  pass and relayed by the first process.
- **Empty** answers once the cgroup and those beneath it hold no live
  process (`cgroup.events`, `populated 0`). A process that has exited
  but is not reaped is not live, so a tree's first process, which io
  keeps unreaped until its close, does not hold the tree open. io reads
  the state, and while it is populated waits for the kernel's next
  change and reads it again, so no end is missed. `Empty` may come
  before the first process's exit is observed. A cgroup that is empty
  is filled again only by a spawn into it, or by a process moved in.
- **Escape.** A process leaves its cgroup only by moving itself, which
  needs the right to write the `cgroup.procs` of where it goes and of the
  common ancestor. A tree that runs as the service's own user, in no
  user namespace, can move into the service's other cgroups on purpose.
  A tree run as another user cannot, and nor can a contained one: it has
  a cgroup namespace of its own, and the shell checks that cgroup2 is
  mounted with `nsdelegate` (../shell.md, 6.4), which forbids a move out of
  it. A service that runs a command it does not trust contains it.
- **Nesting.** A tree may make cgroups beneath its own where its ids may
  write there: when the service runs it as its own user, or when the
  cgroup was delegated to its user. Closing the cgroup removes them, and
  stops what is in them.
- **The end.** A tree has ended when its cgroup is empty: its pid
  namespace, if it had one, has ended, and the mounts of its view are
  gone with the last process that held them. Its owner then
  closes the child (its exit observed, its pipes settled, reaped) and the
  cgroup (removed), in either order, and hears each one's `Closed`. A
  cgroup closed while it holds processes is killed first.
