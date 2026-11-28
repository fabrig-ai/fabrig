# The kernel boundary: files and filesystems

## 1. Files beneath a root

What the records promise is stated exactly in the kernel module's
documentation, with each operation's errors; the decisions behind it:

- **A root is an open directory:** one the shell opened at startup
  (shell-startup.md, 1), or one an `Open` of a directory made beneath another. A
  root opened beneath a root is a root like any other: nothing it opens
  leaves it, though its parent's other entries lie just above.
- **`Open` resolves beneath its root, and follows the symbolic links that
  stay there.** It is `openat2` with `RESOLVE_BENEATH` and
  `RESOLVE_NO_MAGICLINKS`: `..` above the root, an absolute path, and a
  link leading out of the root fail with `Escape` (`EXDEV`), checked by
  the kernel as it resolves, so no race with a rename gets out. Links that
  stay beneath the root are followed, since a workspace holds them (a
  repository's `../../../CLAUDE.md -> AGENTS.md`); `RESOLVE_NO_SYMLINKS` would
  refuse those too and keep nothing more in. Magic links (`/proc/*/fd/*`)
  are refused by name, with `RESOLVE_NO_MAGICLINKS`, as the man page
  asks: `RESOLVE_BENEATH` refuses them today, but may not always. That
  answers `ELOOP` before the escape is looked at, so a magic link is
  `TooManyLinks`, not `Escape`.
- **A `..` that races is tried again.** `RESOLVE_BENEATH` cannot tell a
  `..` from an escape when a rename or a mount anywhere on the system
  moves under it, and answers `EAGAIN` for the caller to retry. The
  backend does, not io: the ring pushes the same `Open` again from its
  slot, up to 16 times, and only one still racing goes up, as
  `Other(11)`. io_uring's worker meets the race only once a quick attempt
  inline has too, so it is rare, about one `Open` in a thousand under a
  storm of renames; the ring's own tests provoke it with renames in its
  workers beside creates through `..`. The simulator has no race, and
  never answers it.
- **Every other operation on a name acts on one entry of an open
  directory.** `renameat`, `unlinkat`, `mkdirat` and `statx` take no
  `RESOLVE_*` flags, and a path with a `/` in it could leave the root
  through `..` or a link. So `Rename`, `Remove` and `MakeDirectory` take a
  name, never a path (no `/`, not `.` or `..`, which `Op::is_valid`
  holds), and `Stat` takes a descriptor. io opens the directory a name
  lies in first, beneath the root (io/files.md). None of them follows the
  entry it names: removing a link removes the link.
- **The opens io makes, not `open`'s flags.** `OpenHow` is `Read` (an
  existing file or directory), `Directory` (an existing directory: a
  root, or one to list), `Create` (a new file, exclusively, to write) or
  `Write` (an existing regular file, to read and write, never created),
  each with a form that refuses a symbolic link, as `Read`'s does: on
  macOS, every link but the system's own (readiness.md, 5).
  Each is one well-defined case for the simulator and the suite to hold,
  and more come when a user pulls them.
- **Cutting a file** is `Truncate` on a descriptor opened to write
  (`ftruncate`): the file's length becomes the one given, and what lay
  past it is gone. Like a write, it is durable once the file is synced;
  the simulator's crashing disk keeps the old length past a crash until
  then (../testing/simulator.md, 3.1).
- **A path-only open, to stat what cannot be read.** `PathNoFollow` is
  `O_PATH | O_NOFOLLOW`, beneath the root with links refused in every
  parent, as the other no-follow cases. It answers a descriptor for the
  entry itself, a symbolic link included, and needs search permission on
  the directories above it and none on the entry, so it opens another
  user's `0o700` directory or `0o600` file that `Read` cannot (io/files.md,
  11). Only `Stat` and `Close` take it; any other operation on it is an
  invalid record. Its `Stat` may answer a symbolic link or a special
  file (a device, a FIFO, a socket), which otherwise only `List` and
  `StatEntry` show. It does no I/O, so the ring keeps whatever
  it opened, without the `NotAFile` check, and leaves it as it is (a
  path-only descriptor takes no `fcntl` flags). The simulator holds the
  same permission rule: search on the parents, nothing on the entry.
  `StatEntry` (below) states an entry with no open at all.
- **A file beside it, path-only.** `PathSiblingFile` opens one name of
  an open directory (no `/`, not `.` or `..`, at most 255 bytes) as
  `PathNoFollow` does, where the entry is a regular file; where it is a
  symbolic link whose target is such a name, naming a regular file in
  the same directory, it opens that file instead: one link, never a
  chain. Any other link (to a path, to another link, to anything but a
  regular file) is `TooManyLinks`, and a missing entry `NotFound`. The
  backend reads the link and states its target without following it
  before the open, which follows no link. Its descriptor takes `Stat`
  and `Close` alone, as `PathNoFollow`'s, so the caller checks the kind
  and mode it opened. It is a repository hook's check, the one rule smith
  and temper's worker share (`temper/docs/design/protocol/hooks.md`,
  2.2), so nothing above the kernel reads a link's target for it.
- **Reads and writes are at an offset,** never at the descriptor's
  position, so concurrent ones need no order. Both may be short: a `Read`
  at the end of the file, and whenever the backend says (POSIX allows it;
  Linux does not cut a regular file's read short but at its end, and the
  simulator draws it); a `Write` likewise, continued by io from where it
  stopped, as a `Send` is. An offset that would pass `i64::MAX` is an
  invalid record: the kernel's offsets are signed, and the ring would read
  `u64::MAX` as the descriptor's position.
- **`Append` is the one write at the descriptor's position.** It is for a
  file written as a stream (io/files.md, 9): a descriptor the shell opened at
  startup to write at its end (`O_WRONLY | O_APPEND`, created with the
  mode its configuration gives), or a standard output that is a regular
  file. The ring writes at the position (offset −1): with `O_APPEND` the
  kernel moves it to the file's end before each write, so each `Append`
  lands at the end, after whatever another process appended meanwhile;
  without it, after the last write. It may be short, and io continues
  from its count, as a `Write`. One is in flight on a descriptor at a
  time, so the pieces of one stream land in order. No `Open` makes such
  a descriptor yet: opening one beneath a root while the service runs
  comes when a user needs it.
- **`Stat` and `StatEntry` answer what an entry is,** from one `statx`:
  its kind; its size; its mode, the permission bits with setuid, setgid
  and sticky (`0o7777` at most); its owner and its group (user and group
  IDs); its number of links; its inode and the device it lies on; the
  device it is, for a device (`special`); and when its metadata and its
  content last changed (`changed`, `modified`), in seconds and
  nanoseconds as the filesystem keeps them, before 1970 included. A value
  its filesystem does not report is zero. `Stat` takes a descriptor.
  `StatEntry` takes a name in an open directory, and states the entry
  itself, never a link's target (`AT_SYMLINK_NOFOLLOW`), nor what an
  automount point would mount (`AT_NO_AUTOMOUNT`): it needs search
  permission on the directory and none on the entry. It is the ring's
  `statx` on the directory and the name. The checks of a private file
  need the owner and the links: one link, and the owner of the directory
  it lies in (io/files.md, 11); a tree's walk needs the rest (2).
- **A kind for each thing a name may be:** a file, a directory, a
  symbolic link, a character or a block device, a FIFO, a socket. `Other`
  is left for a `List` entry whose kind neither its directory nor a stat
  of it told, because it went in between.
- **`Rename` is the atomic step.** With `no_replace: false`, it replaces
  its target whole. With `no_replace: true`, it uses `RENAME_NOREPLACE`
  through the ring's `RenameAt.flags` and, once both directories are
  searched and the source is found, fails with `Exists` if any entry names
  the target, including a dangling link, an unreadable file or a directory.
  The check and move are one atomic operation, including for the same
  name or two names of one inode: those fail with `Exists` too.
  A filesystem rejecting the flag with `EINVAL` answers `InvalidArgument`;
  there is no fallback to a replacing rename. With replacement, the
  idiom that replaces a file is: `Stat` the old file for its permission
  bits, `Create` a temporary in the same directory with them, `Write` it,
  `Sync` it, `Close` it, `Rename` it over the old name, `Sync` the
  directory. A reader sees the old file or the new, and after the last
  `Sync` the new one survives a crash. A `Rename` between two
  directories is the same step: the entry leaves one and lands in the
  other whole, both on one filesystem (`Other(EXDEV)` otherwise), and
  it survives a crash once the directory it lands in is synced; until
  then, a crash keeps it in one or the other, never both or neither.
  Its pieces are io's decisions
  (io/files.md): the temporary's name drawn from the seed, drawn again on
  `Exists`; the old file's bits kept, which is why `Create` takes a mode
  and `Stat` answers one (and why `MakeDirectory` takes one too: a private
  directory is made `0o700`, io/files.md, 11); a symbolic link at the target replaced by the
  file, as `Rename` replaces a link and does not follow it, since writing
  through links is not offered.
- **`List` hands back entries as plain values,** `getdents64`'s structures
  staying in the backend: each entry's kind and its name, packed into the
  record's `names`, at most `entries.len()` of them, `.` and `..` left
  out, from where the last `List` of the descriptor stopped. It is
  synchronous: `getdents64` is not a ring operation. A `names` of at
  least 255 bytes always takes the next entry, so a `List` stops short but
  never makes no progress while entries are left; `Count(0)` is the end.
  A filesystem whose names may be longer than 255 bytes (one storing them
  in another encoding) fails a `List` with `NameTooLong` when its next
  name fits in none of `names`, rather than answer the end; entries a
  `List` took are handed back even if the directory's position could not
  be set back after them.
- **`Lock` takes an exclusive lock without waiting.** It is `flock` with
  `LOCK_EX | LOCK_NB` on a descriptor `Open` made of a file or a
  directory (a root among them), never a path-only one. It completes
  with `Nothing` when the lock is taken, or already held through the
  same open; or fails with the error `Busy` (`EWOULDBLOCK`) when another
  open of the same file holds it, in this process or another. It never waits, so the
  loop never blocks on it: an owner that must wait asks again on a timer
  of its own (io/files.md). The lock belongs to the open, not the process:
  it goes with the descriptor's close, or with the process's end, and
  never passes to a child, every descriptor being close-on-exec. There
  is no unlock and no shared lock until a user pulls one. `flock` is not
  a ring operation, so `Lock` runs at submit and completes at the next
  reap, as `List` does. A filesystem with no `flock` (some network
  filesystems) fails it with its error, as `Other`.
- **An `Open`, `Read`, `Write`, `Append` or `Sync` may be cancelled.**
  None waits on a peer, but on a filesystem that can stall (NFS whose
  server went away, a FUSE daemon that hangs) one may wait for good, and
  io must be able to give up on it. So a `Cancel` may target one, with a socket's
  outcomes (`kernel.md`, section 5): it stops one still queued for io_uring's worker,
  or interrupts one running, or comes too late, the target answering its
  own result. Even on tmpfs a `Read` goes to the worker, and the ring's
  conformance run sees its `Cancel` stop it in some runs and come too late
  in others. The ring reads `ECANCELED` and `EINTR` as `Cancelled` on an
  operation a `Cancel` was submitted for. A `Stat`, `StatEntry`,
  `Rename`, `Remove`, `MakeDirectory`, `List` or `Lock` completes without
  one, as does every operation of `kernel-files.md`, sections 2 and 3: a `Cancel` of one
  is a broken invariant, until a filesystem that stalls them pulls it.
- **Only files and directories open.** A FIFO beneath a root opened to
  read opens at once, and its first `Read` then waits for a writer's
  bytes for good, which nothing above can stop. So every `Open` goes down
  with `O_NONBLOCK | O_NOCTTY`, and the ring stats what it opened, at once,
  in its own call, as a `List` makes its own: anything but a regular file
  or a directory (a FIFO, a device) is closed and answered `NotAFile`,
  and a regular file is made blocking again, so that io_uring sends a read
  it cannot do at once to its worker rather than answer `EAGAIN`. A
  socket, or a device with no driver, the kernel itself refuses (`ENXIO`,
  `NotAFile` too). A device's own `open` may still act (a tape rewinds),
  so roots belong on filesystems mounted `nodev`, where the kernel refuses
  every device with `Permission`; the shell's startup says so
  (shell-startup.md, 1).
- **Errors are named per operation,** on a table the module documentation
  keeps and `Complete::is_valid` checks: an operation on files answers its
  own errors, never a socket's or `Cancelled`, and an operation on sockets
  never answers a file's. The same number means different things per
  operation: `EXDEV` is `Escape` on an `Open` and `Other` on a `Rename`
  (two filesystems); `ENOENT` is `NotFound` on a file and `TooLate` on a
  cancel; `EEXIST` is `Exists` on a `Create` or a no-replace `Rename`,
  and `NotEmpty` on a replacing `Rename` over a directory; `EINVAL` is
  `NotALink` on a `ReadLink` and `InvalidArgument` elsewhere; `ENOTTY` is
  `NotSupported` on a btrfs record, off btrfs.
- **Modes are backend defaults,** as socket options are, but for the one
  a `Create` asks for: a new file is `0o666` unless asked otherwise, and a
  new directory `0o777`, less the process's umask, and every descriptor
  is close-on-exec. The fake machine keeps a umask of `0o022`; the suite
  compares a created file's owner bits, which no usual umask takes.

## 2. A tree's metadata

- **What a tree's walk and its making need,** for a store of trees
  (hearth's): to read each entry without following it (`StatEntry`,
  `ReadLink`, the attributes), and to make each entry as a tree records it
  (`MakeDirectory`, `Create`, `MakeLink`, `HardLink`, `SetMode`,
  `SetOwner`, `SetAttribute`, `Truncate`). Each takes a name in an open
  directory, never a path, as `Rename` does (1), and acts on the entry
  itself, never a link's target: a link is read, owned and given
  attributes as a link. Setting an entry's times comes with its first
  user, hearth's shift of a slot (its M3).
- **`ReadLink`** reads a symbolic link's target, as bytes, into `buf`,
  and answers its length: `readlinkat`, which has no ring operation.
  `buf` holds at least 4,096 bytes, more than the longest target Linux
  keeps, so a target is never cut; a shorter `buf` is an invalid record.
  An entry that is not a link fails `NotALink`.
- **`MakeLink`** makes a symbolic link at a name, to the target given:
  the ring's `symlinkat`. The target is 1 to 4,095 bytes with no NUL,
  else an invalid record. It is kept as given and never resolved, so it
  may lead anywhere, out of the root included: what holds a root in is
  `Open`, which follows no link out of it (`Escape`, 1). A name taken
  fails `Exists`.
- **`HardLink`** gives an existing entry a second name: the ring's
  `linkat`, with no flags, so it links the entry itself, a symbolic link
  included, never a link's target. Both names lie in open directories on
  one filesystem (`Other(EXDEV)` otherwise). A name taken fails `Exists`;
  an entry with as many links as its filesystem allows, `TooManyLinks`
  (`EMLINK`); a directory, or a file the caller neither owns nor may read
  and write (Linux's `protected_hardlinks`), `Permission`.
- **`SetMode`** sets the mode, setuid, setgid and sticky included
  (`0o7777` at most, else an invalid record): `fchmodat2` with
  `AT_SYMLINK_NOFOLLOW`, which the kernel floor has. A symbolic link has
  no mode of its own and fails `NotSupported`. The kernel drops setgid
  from a file whose group the caller is not in, unless it may keep it
  (`CAP_FSETID`); a caller that must know states the entry again.
- **`SetOwner`** sets the user and the group: `fchownat` with
  `AT_SYMLINK_NOFOLLOW`, a link's own for a link. The kernel then clears
  setuid and setgid from a file that may run, and removes its capability
  attribute (`security.capability`), so a caller that makes an entry
  whole sets its owner first, then its mode, then its attributes. An ID
  the caller's user namespace does not map fails `InvalidArgument`; an
  owner the caller may not give, `Permission`.
- **Extended attributes,** by name: `ListAttributes` packs the entry's
  attribute names into `names`, each followed by a NUL, as the kernel
  gives them, and answers their length; `GetAttribute` reads one into
  `value` and answers its length; `SetAttribute` creates or replaces one;
  `RemoveAttribute` removes one. An attribute's name is 1 to 255 bytes
  with no NUL; `names` and `value` hold at least 65,536 bytes, Linux's
  largest list and value, so neither is ever cut; a value to set holds at
  most 65,536; each else an invalid record. An attribute the entry lacks
  fails `NoAttribute` (`ENODATA`); one its filesystem does not keep,
  `NotSupported`; one the caller may not touch, `Permission` (a `user.`
  attribute on a link, a FIFO or a device; `security.capability` without
  `CAP_SETFCAP`; a `trusted.` one without `CAP_SYS_ADMIN`). A list longer
  than Linux hands back fails `Other(E2BIG)`.
- **The attributes of the entry, never of a link's target.** At the
  kernel floor no attribute call takes a directory and a name, and the
  `l*xattr` calls take a path, which could leave the root. So the backend
  opens the entry path-only, without following it (`O_PATH |
  O_NOFOLLOW`, the name in the directory), calls `*xattr` on
  `/proc/self/fd/<n>`, a magic link to the entry itself, a link
  included, and closes it, all in the one synchronous call. This needs
  procfs at `/proc`: the ring checks at its start that `/proc/self/fd`
  names its own descriptors, and answers every attribute record
  `NotSupported` when it does not. Once the kernel floor has
  `getxattrat` and its siblings (Linux 6.13), the backend calls those,
  and the records stay as they are.
- **Truncating and making directories** are `Truncate` and
  `MakeDirectory` (1), and a file is made with `Create`: they need no
  record of their own.
- **Durability.** A new link and a new name for a file (`MakeLink`,
  `HardLink`) are entries of their directory: durable once it is synced,
  as a `Create`'s name is. A mode, an owner and attributes are the
  entry's: durable once its file or its filesystem is synced (`Sync`,
  `SyncFilesystem`); a link's own, which cannot be opened to sync, only
  with its filesystem.
- **Errors** are named per operation, on the module's table (1): each
  of these answers `NotFound` for an entry or a directory gone,
  `Permission`, `ReadOnly` and `NoSpace` for those that write, and
  `NameTooLong`, beside its own above.

## 3. The filesystem: its status, its sync, btrfs's subvolumes

- **`FilesystemStatus`** reads what the filesystem a descriptor lies on
  is, and its space: `fstatfs`, answering `Filesystem { kind, size,
  available }`. `kind` is `Btrfs`, or `Other` with the kernel's magic
  number; `size` and `available` are bytes, the filesystem's and what a
  user without privilege may still write (`f_bavail`).
- **`SyncFilesystem`** makes everything written to the filesystem a
  descriptor lies on durable: `syncfs`, answering `Nothing`. It is the
  one sync of a tree of many entries, whose links and modes have no
  descriptor to sync. A write the kernel failed to put on disk since the
  descriptor was opened fails it, with the kernel's error, as `Other`.
- **btrfs's subvolumes, through a closed list of its ioctls.** A store of
  trees on btrfs makes each tree a subvolume, snapshots it, makes it
  read-only, finds them all again at its start, and destroys them. Each
  of these records is one btrfs ioctl, and no other ioctl is offered:
  the record names what it does, and the backend holds the ioctl's
  structure. A record names a subvolume by a descriptor of its root
  directory (inode 256), one `Open` made with `Directory`; another
  directory may fail `InvalidArgument`.
  - `SubvolumeCreate` makes an empty subvolume at a name in an open
    directory (`BTRFS_IOC_SUBVOL_CREATE`); it needs only write
    permission there.
  - `Snapshot` makes a writable snapshot of the subvolume `source`, at a
    name in an open directory on the same filesystem
    (`BTRFS_IOC_SNAP_CREATE_V2`); Linux allows it to the source's owner.
  - `SubvolumeDestroy` destroys the subvolume at a name in an open
    directory, whatever it holds (`BTRFS_IOC_SNAP_DESTROY_V2`). Without
    privilege, the filesystem must be mounted `user_subvol_rm_allowed`,
    and the caller may write the subvolume. One that holds another
    subvolume fails `NotEmpty`.
  - `SubvolumeReadOnly` makes the subvolume `fd` read-only, or writable
    again (`BTRFS_IOC_SUBVOL_SETFLAGS`), for its owner.
  - `SubvolumeInfo` reads the subvolume `fd` (`BTRFS_IOC_GET_SUBVOL_INFO`),
    answering `Subvolume { tree, uuid, parent_uuid, read_only,
    generation }`: its tree ID, its UUID, the UUID of the subvolume it is
    a snapshot of (`None` for one that is not), whether it is read-only,
    and the generation of its last change. Anyone may read it.
  - `SubvolumeChildren` lists the subvolumes directly beneath the
    subvolume `fd`, from tree ID `from` up
    (`BTRFS_IOC_GET_SUBVOL_ROOTREF`): each child's tree ID and the inode
    of the directory it lies in (`Child { tree, directory }`), into
    `children`, which holds at least 255, Linux's page, else an invalid
    record; it answers how many. A page shorter than 255 is the last;
    after a full one, the caller asks again from the last tree ID plus
    one, which may answer none. The kernel's `EOVERFLOW`, which it gives
    when more children follow a full page, is that full page, not a
    failure. Anyone may list.
  - `SubvolumePath` says where a child lies
    (`BTRFS_IOC_INO_LOOKUP_USER`): from the child's tree ID and its
    directory's inode, as `SubvolumeChildren` gave them, it writes the
    child's path beneath the subvolume `fd`, the directory's path and the
    child's name joined by `/`, into `path`, of at least 4,096 bytes,
    else an invalid record, and answers its length. It needs read and
    search permission on each directory of the path (`Permission`).
  - Off btrfs, each fails `NotSupported` (`ENOTTY`).
- **The worst case.** The writing ones (create, snapshot, destroy,
  read-only) commit a btrfs transaction, milliseconds on an idle
  filesystem and seconds on a busy one; a destroy leaves the space to a
  cleaner the kernel runs later. They and `SyncFilesystem` are the long
  synchronous calls of `kernel.md`, section 6, which a service that must not stall
  makes in a process of its own.
- **Durability.** A subvolume made, snapshotted, made read-only or
  destroyed is durable once its filesystem is synced; btrfs may make it
  so sooner.
