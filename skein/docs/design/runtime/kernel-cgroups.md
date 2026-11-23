# The kernel boundary: cgroups and the helper

## 1. Cgroups, and the helper

- **A cgroup is two descriptors:** its directory, which a spawn into it
  and every record name, and its `cgroup.events`, which `Populated` reads
  and `CgroupChanged` polls. Each file a record touches is opened beneath
  the directory by its fixed name. No record carries a path or a PID.
- **`MakeCgroup`** makes the directory beneath its parent and opens both.
  It writes the limits (`memory.max`, `pids.max`), `memory.oom.group`
  where the memory controller is enabled, and `cgroup.max.depth` and
  `cgroup.max.descendants`, which only the parent's owner may change, so
  what is made beneath it stays within what `RemoveCgroup` walks. To
  delegate, it changes the owner of the directory and of each file the
  kernel lists in `/sys/kernel/cgroup/delegate`, which the backend reads
  once. A make that fails after its directory was made removes it before
  it answers: a failed make leaves nothing.
- **`EnableControllers`** writes them, `+cpu`, `+memory`, `+pids`, to the
  cgroup's `cgroup.subtree_control`; the kernel answers `Busy` while the
  cgroup holds a process. **`EnterCgroup`** writes `0` to its
  `cgroup.procs`, which moves the whole process, every thread.
- **`CgroupCounts`** reads `cpu.stat`'s user and system time, and, where
  the files are, `memory.peak` and `memory.events`'s `oom_kill`.
- **`SignalCgroup`** takes any of `Interrupt`, `Terminate`, `Kill`,
  `Stop`, or `Continue`, and reads the `cgroup.procs` of the cgroup and of each
  cgroup beneath it. For each PID it opens a pidfd, then checks by the
  process's `/proc/<pid>/cgroup`, compared a path segment at a time, that
  it is in the tree, then signals it
  through the pidfd. A process alive when the signal goes was the PID's
  owner throughout the check, which its pidfd held; one gone meanwhile is
  not signalled, and is no failure.
- **`KillCgroup`** writes `1` to `cgroup.kill`: the kernel kills every
  process in the cgroup and beneath it, those forked meanwhile included.
- **`Populated` and `CgroupChanged`** read `cgroup.events`, each read
  whole from its start (`pread` at 0), and poll it
  for the kernel's next change: the ring's poll (`POLLPRI`) on the same
  descriptor, the one cgroup record that is a ring operation, and the
  one that may be cancelled. The kernel marks each read on the
  descriptor, and the poll completes at once if the file changed since
  the last, so a read, a poll, and a read again lose no change. A read
  that does not start at 0 neither reads the state nor marks it read.
  The kernel may delay a change's wake by some milliseconds, and wake
  with no change: each wake reads again. An exited process not yet
  reaped no longer counts: `populated` falls as a process exits, before
  its parent hears of it, and a cgroup that holds only such processes
  may be removed. `pids.current` counts them until they are reaped.
- **`RemoveCgroup`** removes the cgroups beneath it, deepest first, then
  removes it from its parent (`..` of its directory) by the name io keeps.
  A cgroup that still holds a process answers `Busy`, and what was
  removed stays removed.

**The helper.** A `Spawn` that asks for ids, containment (io/processes.md, 3)
or a reach (io/processes.md, 6) starts the helper as the child, and the
helper starts the command. The helper is the service's own executable,
`/proc/self/exe`, with a marker argument, which `main` hands first to
the shell's `skein_shell::helper::dispatch` (shell-startup.md, 1.4): one file to
ship, and no version between the service and its helper to skew. A
backend's `Kernel::set_helper(&mut self, executable: Box<[u8]>)` names
another executable whose `main` does the same, as a test binary, whose
`main` is its harness's, names a fixture of its suite. A spawn that asks
for none of them is `pidfd_spawn` alone, into its cgroup
(`POSIX_SPAWN_SETCGROUP`) if it names one. A reach alone is set up as
*A reach alone* below says; ids and containment as the rest of this
section does.

- **Its descriptors.** The backend spawns the helper as any child, in
  its group and its cgroup, the pipes and given roots at their numbers,
  and two of its own above them, named in its arguments:
  - **the spec,** a pipe the backend wrote the whole spawn into before
    the spawn, at most 64 KiB, a pipe's capacity, so the write never
    waits;
  - **the report,** a pipe whose read end is `Done::Spawned`'s `report`.
    It closes when the command executes (close-on-exec), and its end of
    file tells io the command started. A process of the helper that fails
    writes one record (its stage, the kernel's error) and exits. A spawn
    the kernel refuses before the helper runs fails as any spawn does.

**The setup, in order.** H is the child io holds; M maps; I is the pid
namespace's first process; C is the command.

1. **H** reads the spec whole, and blocks every signal it will relay, so
   that one sent before it relays is queued, not lost. Asked for ids
   alone, it sets the supplementary groups, then the group, then the
   user (`setgroups`, `setresgid`, `setresuid`), and executes the
   command at `dir`.
2. Contained, **H forks M** first. M closes every descriptor it
   inherited, sets `PR_SET_PDEATHSIG`, exits if its parent is no longer
   H (whose PID it was given before the fork), and waits to be told.
3. **H unshares,** in one call, a user namespace and the mount, ipc and
   cgroup namespaces, with the pid, network and uts ones as asked: the
   user namespace owns them all, H holds every capability in them, and
   the cgroup namespace's root is the tree's cgroup, which H was spawned
   into.
4. **M writes H's maps,** from the service's namespaces, against H's
   PID, which cannot be reused while H lives: itself, to `uid_map`,
   `setgroups` and `gid_map`, where it may (one range of its own id, or
   with `CAP_SETUID` and `CAP_SETGID`), and otherwise by running
   `newuidmap`, then `newgidmap`, each a child it waits for, with H's PID
   and the maps. Both check that the caller owns H, by the owner of
   `/proc/<H>`, which a process freshly executed from a readable file
   keeps. `newgidmap` denies `setgroups` in the namespace when the map is
   one id, the caller's own primary group, which `/etc/subgid` does not
   give it. M exits with what came of it, and H, which reaps it, fails at
   `Namespaces` unless both maps were written. A setuid `newuidmap` gains
   no privilege for a service that runs with `no_new_privs`: the shell
   checks at startup (shell-startup.md, 1.4).
5. **H takes what the view is made of,** still the service's ids on the
   host, so with its rights over each path: for the base and each bind,
   it opens the host path in the new mount namespace (`openat2`,
   `O_PATH`, `RESOLVE_NO_SYMLINKS`), checks its device and inode against
   the io root's descriptor, and clones the mount there
   (`open_tree`, `OPEN_TREE_CLONE`, `AT_RECURSIVE`). What was checked is
   what is mounted; a path that is not the io root's directory fails the
   view. Each host path is the one io was given for the root: one with a
   link on the way fails.
6. **H takes the namespace's root:** supplementary groups dropped where
   `setgroups` is allowed, then `setresgid` and `setresuid` 0. Without a
   pid namespace, H goes on as I does, from step 7, and executes the
   command itself. With one, **H forks I**, then relays: it reads the
   signals it blocked from a signalfd and passes each to I, waits for
   I, and exits with the status I sent it over a pipe, or with I's own
   if none came. A command that ended by a signal, H ends by the same:
   its default action restored, unblocked, dumping off
   (`PR_SET_DUMPABLE` 0). I sets `PR_SET_PDEATHSIG` (`SIGKILL`) and
   then checks, by a pidfd of H opened before the fork, that H still
   lives; I's ids never change, which would clear it.
7. **I composes the view** (io/processes.md, 3), if one is asked: every mount
   private, recursively; the new root, an empty `tmpfs` or the base's
   clone, attached on `/tmp` of the namespace's copy of the host's tree,
   and made the working directory; `/proc`, new (`nosuid`, `nodev`,
   `noexec`), mounted while the host's `/proc` is still in the
   namespace, which the kernel requires of a user namespace's; each
   bind's clone attached (`move_mount`) on its target, which is opened
   beneath the new root following no link (`RESOLVE_IN_ROOT`,
   `RESOLVE_NO_SYMLINKS`), made if missing where the root may be
   written; read-only binds, and `read_only` paths bound on themselves,
   made read-only recursively (`mount_setattr`, `MOUNT_ATTR_RDONLY`,
   `AT_RECURSIVE`), which keeps each mount's other flags, the locked
   ones among them; the scratches and `/tmp`, `tmpfs`; `/sys`, new
   with a network namespace of its own, else the host's, read-only;
   `/dev`, a `tmpfs` with the host's devices bound in, its own `devpts`
   (`newinstance`, `ptmxmode=0666`, `mode=0620`, no `gid`) and links.
   Then `pivot_root(".", ".")`, the old root detached, `/` the working
   directory, and the root read-only, not recursively, unless given
   writable.
8. **I finishes:** the hostname; loopback up, in a network namespace of
   its own; the rlimits, lowered. It **forks C**, relays to it as H does
   to I, reaps whatever ends in the namespace, and once C has ended sends
   H C's status and exits 0: the first process of a pid namespace cannot
   end by a signal it raises on itself. Its end ends the pid namespace:
   the kernel kills what is left in it.
9. **C** sets the ids inside, its supplementary groups cleared, or set
   to `ids.groups`, where `setgroups` is allowed; then `no_new_privs`;
   its working directory; its signals' mask and handlers reset; every
   descriptor closed but the pipes, the given roots and the report; and
   executes the command.

Each process closes what only its child needs once it has forked it, so
that the report's end of file and the pipes' ends are the command's. A
step that fails writes its stage and exits: `Ids` (1, 6, 9's ids),
`Namespaces` (3, 4), `View` (5, 7), `Command` (9). io refuses, at its
entrance, a view without a pid namespace of its own, for its `/proc`,
and a view whose maps leave the namespace's 0 unmapped, which could
create nothing in its `tmpfs`.

The report's end of file with no record is also what a helper killed
during its setup leaves, by a deadline or for want of memory: io then
answers `Spawned`, and the child's exit is by that signal (io/processes.md, 3).

The command runs with the capabilities its ids have in the namespace,
with no seccomp filter (io/README.md, 7), and with no terminal.

**A reach alone.** A spawn whose only setup is its reach
(io/processes.md, 6), with neither ids nor containment, starts the
helper built for Landlock and `no_new_privs` alone. The backend builds
the domain in the service, where a failure is its own to answer; the
helper only takes it on, and the backend answers the spawn once the
helper has executed the command or said why not. Linux's ring and its
readiness backend share it, as they share the rest of a spawn.

1. **The ABI.** The backend reads the kernel's Landlock ABI once, at its
   first reach (`landlock_create_ruleset` with
   `LANDLOCK_CREATE_RULESET_VERSION`): `ENOSYS` is `Absent`,
   `EOPNOTSUPP` is `Disabled`, and an ABI below the reach's `abi` fails
   the spawn, `Abi { found, needs }`.
2. **The domain.** It creates a ruleset that handles every file-system
   right of ABI 6 and scopes signals and abstract Unix sockets
   (`LANDLOCK_SCOPE_SIGNAL`, `LANDLOCK_SCOPE_ABSTRACT_UNIX_SOCKET`), and
   no network right. For each rule in order, it opens the rule's path
   beneath its root (`openat2`, `O_PATH | O_CLOEXEC`, `RESOLVE_BENEATH
   | RESOLVE_NO_MAGICLINKS`, the root itself for an empty path), adds it
   (`LANDLOCK_RULE_PATH_BENEATH`) with its rights, or with those of them
   a file can have where it opened no directory, skipping it where none
   remain, and closes it. A rule that fails, opened or added, fails the
   spawn, `Rule { index, errno }`, and nothing is spawned.
3. **The spec,** written whole into a pipe before the spawn: the
   command's program, its arguments, its environment, and the child
   descriptors its table names, each entry its length (four bytes) and
   its bytes. io's bound on the command (io/processes.md, 6) keeps it
   within a pipe's 64 KiB, so the write never waits.
4. **The spawn.** `pidfd_spawn` of the helper, in its group, at the
   spawn's directory, with the pipes and given descriptors at their
   numbers as any spawn has them, and above them three of its own: the
   spec's read end, the report's write end and the ruleset. Its
   arguments are the marker and those three numbers; its environment is
   empty, so nothing of the command's environment acts on the helper
   before it is restricted, and none of it reaches a command line, which
   any user may read.
5. **H,** the helper, reads the spec whole and closes it; sets
   `no_new_privs` (`PR_SET_NO_NEW_PRIVS`); restricts itself
   (`landlock_restrict_self` with the ruleset) and closes the ruleset;
   closes every descriptor but 0, 1, 2, those the spec names and the
   report (`close_range` over each gap); writes `Restricted` on the
   report; marks the report close-on-exec; and executes the command
   (`execve`, the spec's program as a spawn without a reach names it,
   with the spec's environment). A step that fails writes one record,
   its stage (`Restrict` for the first four, `Command` for the
   `execve`) and the error number, and exits 127.
6. **The backend reads the report to its end,** and answers:
   `Restricted` then end of file is `Spawned`, with no `report`, which
   the backend has read; `Restricted` then a `Command` record is the
   error a spawn without a reach answers for that `execve`, the helper
   reaped; a `Restrict` record is `Reach(Restrict { errno })`, the
   helper reaped; end of file with no record is `Reach(Helper)`: the
   executable never ran the helper's dispatch, which a `main` that does
   not hand it the marker first leaves. The backend waits for the
   report as glibc's `pidfd_spawn` waits for the helper's `exec`: the
   helper's start, a few system calls and an `execve`, so a reached
   spawn holds the loop that long.

The command so starts in the domain from its first instruction, with
nothing open but what the spawn named, and every process it starts
inherits the domain and `no_new_privs`. A restricted service that spawns
with a reach of its own stacks a second domain beneath the one it is in,
which Landlock allows a process with `no_new_privs`.

**The startup check** (shell-startup.md, 1.4) is the same spawn with no command:
a reach at ABI 6 with no rule, and a spec whose program is empty, which
io never admits for a spawn (`Restricted` then exit 0 is the helper's
whole answer). `Kernel::can_restrict(&mut self) -> Result<(),
Unenforced>` builds the domain, spawns the helper, reads its report,
waits for its exit and answers: `Ok` where the helper restricted itself
and exited 0, the `Unenforced` met otherwise. macOS answers
`Unsupported` at once.
