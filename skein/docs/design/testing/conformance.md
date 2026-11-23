# The conformance suite

The kernel boundary's contract (../runtime/kernel.md) in executable
form: the scenarios that hold the simulator and every backend on the
real kernel to the same answers, moved out of that document, whose
testing section names it.

## 1. The suite

The conformance suite is the contract's executable form
(../../foundation/testing-strategy.md, 5). It is a crate of its own,
`testing/skein-conformance`, over a small backend interface (open a
process, submit, reap, enter, let time pass), and runs against the
simulator (`tests/conformance/sim`) and against each backend on the real
kernel (`tests/conformance/real`: the ring, and the readiness backend
through epoll on Linux and kqueue on macOS), each of which implements the
interface for its backend.

- **Each scenario is a scripted sequence of records** that returns what it
  saw, and its check names the rule of the contract behind each
  assertion. An answer that waits on the peer's reset or acknowledgement
  is retried until it changes.
- **A driver checks every completion on the way:** valid for its
  operation, one per submission, the record handed back with its buffer
  in the same `Box`, and every descriptor closed at the end.
- **On the simulator,** each scenario runs over calm seeds and seeds of
  every fault loopback can show: a few of chaos in the focused suite, many
  in the fuzzy one (../../foundation/testing-strategy.md, 8). Every outcome a race allows
  must appear over the fuzzy suite's seeds, and a calm world must pair
  each race as the ring does.
- **What is Linux's is said so.** An answer that Linux gives and the
  contract does not promise (the order of two refusals, an error number
  another system answers otherwise) is checked as Linux's, by name, on
  Linux, and every backend is held to what the contract promises; none
  is loosened silently. The readiness backend on macOS meets every
  scenario through kqueue (../runtime/readiness.md, 5).
- **On the ring,** each scenario runs once, and fails, saying so, if
  io_uring is not usable. **On the readiness backend,** forced on Linux
  (../runtime/readiness.md, 1), each runs once too, and a check that only the ring
  can meet says so: a cancel of a file's operation, made at submit, is
  always too late there.
- What no record can observe (close-on-exec, `TCP_NODELAY`, the modes of
  new files) is not checked; `SO_REUSEADDR` and `IPV6_V6ONLY` are, by
  their effects.
- **Files beneath a root of the scenario's own.** Each scenario on files
  lays out its root from a tree in the suite's own vocabulary (files,
  directories, symbolic links, each with its mode): on the ring, a scratch
  directory made per scenario beneath the system's temporary directory,
  with a directory beside it for a link to lead out to, and removed with
  the backend; in the simulator, the minimal fake machine (README.md, 4).
  The scenarios: a file made, written at offsets over itself and past its
  end, synced, read back and stated; a `0o000` file and directory of the
  scenario's own user that `Read` is denied and `PathNoFollow` opens and
  stats (on the ring, the denial is checked only without
  `CAP_DAC_OVERRIDE`, which skips it), and a link it opens as a link; renames over a file, across
  directories, of a file over a directory and the other way, over a full
  and an empty directory, beneath itself, over itself, and the idiom that
  replaces a file whole; removals of files, directories, a link and a file
  open; new directories, and one removed while open; listings whole, one
  entry at a time, and cut short by long names; a root beneath a root;
  forty-four paths that leave their root, stay beneath it, or name a FIFO,
  which opens as no file; locks: a directory locked through one open,
  `Busy` through a second open of it, locked again through the first,
  and taken through the second once the first is closed; and what the
  owner may not do. Each names every error its operations can be made to
  answer on a healthy scratch directory, and, where two could answer, the
  one Linux checks first: a removed directory before a name's length, a
  final `/` before the last name on a create, both directories of a
  `Rename` before its source, its source before its target, a name
  looked up before its directory is written; a no-replace rename, once
  both directories are searched and the source is found, checks for a
  taken target before ancestor, same-inode and write-permission checks.
  The driver checks that every path, name and buffer comes back in its
  `Box`, written only where the count says.
- **What only the simulator shows:** a disk beyond a healthy one (no
  space, a filesystem gone read-only, an I/O error) is the simulator's
  own tests' (simulator.md, 6), and an `Open` past the descriptor limit a
  scenario on the simulator only: lowering a process's limit on the real
  kernel takes `unsafe` outside the ring adapter, or a child process, and
  neither is allowed. A `Cancel` of a file's `Read` runs on
  both: the ring stops it or comes too late, from one run to the next,
  and requires a pairing the simulator draws; the simulator's `hung`
  fault, a filesystem gone away, adds the `Read` that waits for good,
  which no scratch directory shows, and every pairing must appear over
  the fuzzy seeds. `EMLINK` (`TooManyLinks` on a `Rename`, a
  `MakeDirectory` or a `HardLink`) neither backend provokes. A `Rename` across
  filesystems (`Other(EXDEV)`) is the ring's own test, between the
  temporary directory and `/dev/shm` where they are two mounts. Short
  reads and writes are an outcome the simulator draws and the ring never
  gives on a regular file: the fuzzy suite asserts both counts appear over its
  seeds, and a calm world counts every byte, as the ring does.
- **Appending.** A scenario opens a file to append as the shell does, in
  its own root: `Append`s land at the end in order, after bytes another
  descriptor appended between them, and a file that was not empty keeps
  what it held; a short `Append` is continued from its count; a `Cancel`
  stops one or comes too late, as for a `Write`. The suite's tree gains a
  second name for a file (a hard link), so that `Stat` answers two links
  for it, one for the rest, and the owner every scenario's files have.
- **A tree's metadata,** on both backends, in the scenario's own root,
  whose tree gains FIFOs, modes with setuid, setgid and sticky, and
  `user.` attributes: `StatEntry` of each kind it lays out, a link
  stated as a link, its inode, device and links those `Stat` answers
  through an open, its group the scenario's, and `changed` moving on
  after a `SetMode`, never back; `ReadLink` of a link, whole, and
  `NotALink` of a file; `MakeLink` to a target out of the root, which
  `ReadLink` reads back as written and an `Open` through it refuses with
  `Escape`; `HardLink` of a file and of a link, each counted in its
  links, and `Exists`; `SetMode` with setuid, setgid and sticky, read
  back, and `NotSupported` on a link; `SetOwner` to the scenario's own
  user and group, the only ones a run without `CAP_CHOWN` may give, and
  `Permission` for another's, checked only without it; a `user.`
  attribute set, listed, read, replaced and removed, then `NoAttribute`;
  a value of 65,536 bytes; `Permission` for a `user.` attribute on a
  link. On the ring, the scratch directory's filesystem keeps `user.`
  attributes (tmpfs and ext4 do); a scenario that finds it does not
  fails, saying so.
- **The filesystem,** on both backends: `FilesystemStatus` of the root
  answers a kind, and an available space no larger than its size;
  `SyncFilesystem` answers `Nothing`; and where the kind is not btrfs,
  each subvolume record answers `NotSupported`.
- **btrfs, on the ring alone,** in the directory `SKEIN_BTRFS_SCRATCH`
  names, on a btrfs filesystem mounted `user_subvol_rm_allowed`, where
  the run may write; without it, each such scenario is skipped by name,
  saying so. A subvolume made, a file written in it, a snapshot of it
  whose parent UUID is its UUID, made read-only (a write in it then fails
  `ReadOnly`) and writable again; the scratch subvolume's children, both
  among them, and each one's path; `from` past the first child's tree,
  which leaves it out; both destroyed, and `NotFound` after. The
  simulator has no btrfs: the minimal fake machine answers these records
  `NotSupported` (README.md, 4), and a service's own fake machine that
  models btrfs holds itself to these scenarios (
  `hearth/docs/design/testing/fake-machine.md`, 3).
- **Processes,** with the fixture program of both backends (echo its
  input, exit with a status, never exit): pipes through chosen
  descriptors, an exit status kept, a `Wait` that waits while the child
  lives, a signal, a kill. To these the groups add: the fixture starting
  a child of its own in its group and exiting; a `Wait` that observes the
  exit, then a signal to the group, which ends the grandchild and its
  hold on a pipe; then the reaping `Wait`. Usage, in a process that has
  reaped no child before: the children's part zero, still zero after the
  observing `Wait`, and with a peak above zero after the reap; and the
  process's own CPU and peak never falling between two reads.
- **Cgroups,** on both backends, beneath a cgroup the run may write: on
  the ring, the one `SKEIN_CGROUP_SCRATCH` names, or else a delegated
  one above the run's own, found as the end-to-end kit finds it
  (../examples.md, 6); without one, each scenario is skipped by name, saying
  so. A cgroup made, with limits, counted, removed; one made twice
  (`Exists`); a limit without its controller; controllers refused while
  the cgroup holds a process; the run's fixture spawned into one, with a
  grandchild that leaves its group (`setsid`) and stays in the cgroup; a
  terminate that reaches both; `Populated` true, `CgroupChanged` pending,
  then a kill, after which it completes and `Populated` is false, with
  the first child exited but unreaped; a removal refused while populated,
  then deepest first once empty.
- **Contained trees, on the ring alone,** where the run's user has
  subordinate ids and `newuidmap`; else skipped by name, saying so. Ids
  alone, read back by the fixture (run as root, or skipped). A tree in a
  user namespace whose maps name subordinate ids only, whose own user
  and group then read as the overflow ids, and one whose maps name the
  run's own group, where `setgroups` is denied; each stage failed in turn
  and reported as it; a view whose git directory is read-only inside a
  writable bind, the write refused; a bind whose path was swapped for a
  link, which fails the view; `/proc` showing the tree's processes alone;
  loopback alone; the hostname; a double fork contained, and killed with
  the cgroup; a polite signal relayed to the command, and the command's
  exit status and its death by a signal, each relayed to the first
  process, and a signal sent before the relay was ready still relayed;
  a bind beneath a directory only the service's user may search; a
  service run with `no_new_privs` refused the maps that need
  `newuidmap`.
- **A reach** (../runtime/io/processes.md, 6), on the simulator and on each Linux
  backend, ring and readiness: each scenario one spawn of the probe
  (README.md, 4), which answers every question in one run,
  over the calm seeds alone on the simulator, where nothing is drawn.
  On macOS a spawn with a reach fails `Reach(Unsupported)`, and that is
  its only scenario. Each scenario lays its root: `granted/`, holding a
  file, a directory `inner/` with a file in it, a link `out` to
  `../canary`, and a program the probe may execute; and `canary`,
  beside `granted/`. Its reach grants `granted` every right but devices'
  (`Rights::FULL`), and on the ring the machine's paths the probe needs
  to run, beneath a root on `/`: the system's libraries and the
  fixture's own directory, reading and executing, and `/proc`, reading.
  The probe runs in `granted`:
  - **what was granted, and the rest refused:** it reads, writes, lists
    and executes beneath `granted` as granted; reading `../canary` (the
    canary beside the root), listing `..` (the parent directory) and
    reading `out` (a link that leads out) are each refused,
    `Permission`;
  - **no privilege gained:** its `no_new_privs` is set, and a set-id
    copy of the probe it runs keeps the probe's ids. On the ring the copy
    is set-group-id to a group of the run's user other than its own, or,
    as root, set-user-id to another user; where the run has neither, or
    the scratch directory's filesystem is mounted `nosuid`, that part is
    skipped by name, saying so, and `no_new_privs` is still checked;
  - **a signal to its parent** (the service, outside the tree) is
    refused, `Permission`, where the same probe unrestricted would send
    it: on the ring, the probe's report says which signal it sent;
  - **a missing ABI:** a reach written at an ABI above any kernel's fails
    the spawn, `Reach(Abi { found, needs })`, `needs` the reach's, and
    nothing runs; on the simulator, a machine without Landlock fails it
    `Absent`, one where it is disabled `Disabled`, and one at ABI 5
    `Abi { found: 5, needs: 6 }`, which only the simulator shows;
  - **a rule's path missing** fails the spawn, `Rule { index, errno }`,
    `ENOENT`, at that rule's index;
  - **a child narrowing beneath its parent:** the probe starts a probe
    of its own narrowed to reading `inner`, which cannot read
    `granted`'s file that the outer probe still reads, nor write
    beneath `inner`;
  - **descriptors:** the service holds a descriptor on `canary`, not
    close-on-exec on the ring, when it spawns; the probe lists the
    descriptors it holds and finds those the spawn named and no other.
  A disagreement between the simulator and the kernel is a conformance
  bug, fixed in the suite first (`temper/docs/design/draft/confinement.md`, 8.1). The startup check
  (../runtime/shell-startup.md, 1.4) is the ring's own test: `can_restrict` answers `Ok`
  on a machine the suite runs on, which has Landlock at ABI 6 or later,
  and fails, saying so, where it does not.

## 2. Open questions

None yet: what a scenario leaves to the ring alone, or to the simulator
alone, is said in section 1, beside it.
