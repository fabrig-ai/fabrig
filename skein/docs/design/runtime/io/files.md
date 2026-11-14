# Files

Files are a machine of their own beside io's: `FileIo`
(`skein_io::file_layer`), its requests and events in `skein_io::file`. A
service holds both, submits the records of each to the one kernel, and
hands each completion back to the machine whose operation it was.

The operation's token says whose: `file_layer::owns(op)`. `FileIo` mints
its tokens with `file_layer::LANE`, bit 59, set. `Io`'s never have it: a
flight's token holds its slot in its top 32 bits, and `valid` refuses an
operation table past 2^27 slots. The bit sits below a token's top four
(../lib.md, 4), so a service that tags its children's tokens still can. A
service that runs two `Io`s, or two `FileIo`s, tells them apart itself.

`FileIo` takes its bounds as `Io` does (../../../foundation/programming-model.md, 6.3): a
`file_layer::Limits`, which `file_layer::valid` checks and
`FileIo::with_limits` is made from, in the `Env<Limits>` that
`file_layer::step`'s entry points (`down`, `down_until`, `up`, `expire`)
take. A reader keeps its `file_layer::Limits` beside its `FileIo`, since
`FileIo` gives them back to no one. `up` and `down_until` read no clock, and
no entry point reads `wall`: a reader with no clock at those calls gives
`Time::ZERO` and `Wall::EPOCH`.

Each request `FileIo` works on is one lifecycle (../../../foundation/programming-model.md, 5.4). A
replacing store and a private opening are each one enum, whose every state holds
the descriptors open in it and what it still needs, with no field beside the state
for a phase. A store settling a failure holds the failure and the step of cleanup
it has reached.

```rust
// skein_io::file
pub enum Request {
    // whole files, beneath a root's token
    Load  { owner, root: Token, path, max: u32, no_follow },                       // Loaded, TooLarge
    Scan  { owner, root: Token, path, max: u32, max_bytes: u64, no_follow },       // Scanned
    Store { owner, root: Token, path, bytes, expected: Expect, no_follow }, // Stored, Conflict
    MakeDirectory { owner, root: Token, path, mode },                            // Made
    OpenPrivate { owner, root: Token, path },                                    // Opened, Refused
    // open files, at offsets
    Create, CreateNoFollow { owner, root: Token, name, mode },                     // Opened
    OpenRead, OpenReadNoFollow { owner, root: Token, name },                       // Opened
    OpenWrite, OpenWriteNoFollow { owner, root: Token, name },                     // Opened
    OpenDirectory { owner, root: Token, name, no_follow },                         // Opened: a root
    ReadAt  { owner, file: Token, offset: u64, max: u32 },                         // Read
    WriteAt { owner, file: Token, offset: u64, bytes },                            // Written
    Cut { owner, file: Token, length: u64 },                                       // Cut
    Stat { owner, file }, Sync { owner, file }, Close { owner, file },
    // the entries of a directory
    SyncDirectory { owner, root: Token }, List { owner, root: Token },
    ListPage { owner, directory: Token, max: u32 },                                // Listed
    Rename { owner, from_root: Token, from, to_root: Token, to },
    Remove { owner, root: Token, name, directory },
    // a lock
    Lock { owner, root: Token },                                                   // Locked, Busy
    // a tree's metadata, by name in an open directory (../kernel.md, 6.4)
    StatEntry { owner, directory: Token, name },                                   // Stated
    ReadLink { owner, directory: Token, name },                                    // LinkRead { target }
    MakeLink { owner, directory: Token, name, target },                            // Linked
    HardLink { owner, from_root: Token, from, to_root: Token, to },                // Linked, as Rename names its two
    SetMode { owner, directory: Token, name, mode },                               // Changed
    SetOwner { owner, directory: Token, name, user, group },                       // Changed
    ListAttributes { owner, directory: Token, name },                              // AttributesListed { names }
    GetAttribute { owner, directory: Token, name, attribute },                     // AttributeRead { value }
    SetAttribute { owner, directory: Token, name, attribute, value },              // Changed
    RemoveAttribute { owner, directory: Token, name, attribute },                  // Changed
    // the filesystem, and btrfs's subvolumes (../kernel.md, 6.5)
    FilesystemStatus { owner, file: Token },                                       // FilesystemStated { filesystem }
    SyncFilesystem { owner, file: Token },                                         // Synced
    SubvolumeCreate { owner, directory: Token, name },                             // Made
    Snapshot { owner, source: Token, directory: Token, name },                     // Made
    SubvolumeDestroy { owner, directory: Token, name },                            // Removed
    SubvolumeReadOnly { owner, subvolume: Token, read_only },                      // Changed
    SubvolumeInfo { owner, subvolume: Token },                                     // SubvolumeStated { subvolume }
    SubvolumeChildren { owner, subvolume: Token, from: u64 },                      // Children { children }
    SubvolumePath { owner, subvolume: Token, directory: u64, tree: u64 },          // PathRead { path }
}
// each request ends in one event: its own, Failed { error } or Cancelled
```

- **Paths resolve beneath a root.** A root is an open directory. Every
  request names one, and a path relative to it that the kernel resolves
  beneath it (`openat2` with `RESOLVE_BENEATH`), so neither `..` nor a
  symbolic link leaves it. `no_follow` refuses a symbolic link in any
  part of the path.
- **Where roots come from.** The shell opens the first at startup, from
  configuration (`open_root`), and the service gives each to
  `FileIo::adopt_root` for its token. `OpenDirectory` opens another
  beneath one, and its `Opened` gives that root's token. Every request
  names a root by its token, never by its descriptor, which io alone
  sees.
- **Whole files** are one request each; io runs the opens, the reads or
  writes and the closes beneath it:
  - `Load` reads a file of at most `max` bytes; a larger one is
    `TooLarge`, with its size and none of its bytes;
  - `Scan` lists a directory, keeping its first entries in name order
    within `max` entries and `max_bytes` of names, and counts those left
    out;
  - `Store` replaces a file, conditionally and atomically (below).
- **A file's contents print as their length.** The `Debug` of a request
  or an event shows the contents it carries, `Store`'s and `WriteAt`'s
  bytes, `Loaded`'s and `Read`'s, as their length, never a byte of
  them; every other field prints as derived. So does `FileIo`'s own
  `Debug`, for the contents it holds while a store, a read or a load is
  in flight. A trace of a stored record, a credential's among them,
  shows none of it (3).
- **Open files** stay open across requests, behind tokens, up to the
  number the driver is made with. `ReadAt` and `WriteAt` work at an
  offset, and io continues a short transfer until the request is met or
  a read reaches the end. `Sync` makes a file's writes durable,
  `SyncDirectory` a directory's entries. These are what ../../data/kv.md's log
  segments need (../../data/kv.md, section 8).
- **What a request makes by a path, io makes durable** before it
  answers: `Store` after its directory's `Sync` (2); `MakeDirectory`
  after a `Sync` of the directory it made the new one in; `OpenPrivate`,
  when it made its directory, after the same, before it opens it. io
  holds that directory open, having opened it to make the name (below),
  and the owner, which named a root and a path, has no descriptor for
  it. A failed `Sync` there answers `Failed` with `committed`: the
  directory is made, its durability unknown. What an owner makes
  beneath a root it holds open (`Create`, `Rename`, `Remove`, and the
  entries of a directory io made) is durable once it syncs that root
  (`SyncDirectory`), as kv's segments are.
- **Listing** comes whole or by pages. `List` reads a whole directory,
  and fails with `NoBufferSpace` past the driver's `max_entries`. A
  directory that may hold more is listed by pages: the owner opens it
  with `OpenDirectory`, and each `ListPage` answers `Listed` with at most
  `max` of its entries (at least 1 and at most `max_entries`: 0, or
  more, is `InvalidArgument`, before any record), from where the last
  page of that open directory stopped. io fills each page until `max` or
  the directory's end, so a page shorter than `max` is the last, and an
  empty one says the end was reached already. The position is the open
  directory's own (../kernel.md, 6.1's `List`), so the open directory is
  the cursor: two owners listing one directory each open it, and the
  owner closes it when done. Entries come in the
  directory's order, `.` and `..` left out, each once while the
  directory is unchanged; one added or removed between pages may appear
  or not, as `readdir` allows. A page's entries cost what its `List`
  holds. A page that fails, with a kernel's error or stopped by its
  deadline or a cancel (below), may have taken
  entries it never answered: the position is past them. An owner that
  must see every entry closes the directory and lists it from a new
  `OpenDirectory`.
- **Renaming** moves one name: `Rename` takes the directory the entry
  leaves and the one it lands in, each a root, the same one for a rename
  in place, and a single name in each. Both lie on one filesystem;
  across two, the kernel's error is `Other(EXDEV)`. A rename between two
  directories survives a crash once the owner has synced the directory it
  lands in (../kernel.md, 6.1).
- **Locking** is `Lock` on a root: an exclusive lock, taken without
  waiting (../kernel.md, 6.1). It answers `Locked` when the lock is taken or
  already held through this root, or `Busy` when another open of the
  directory holds it, in this process or another; `Failed` with the
  kernel's error otherwise. io never waits for a lock and keeps no
  timer for one: an owner that must wait asks again on a timer of its
  own, at the interval it chooses. The lock lasts as long as the root's
  descriptor: until its `Close`, for a root `OpenDirectory` made, and
  until the process ends, for one the shell opened. There is no unlock.
  A spawned child never holds it.
- **A tree's metadata** is read and written by name, in a directory the
  owner holds open: a root, or one `OpenDirectory` made, named by its
  token, as `ListPage` names one. Each request is one record (../kernel.md,
  6.4), so it acts on the entry itself, never a link's target, and io
  opens nothing for it. `StatEntry` answers `Stated` with the entry's
  whole stat; `ReadLink` answers the link's target; `ListAttributes`
  answers each attribute's name, unpacked from the kernel's list;
  `GetAttribute` answers one value. `MakeLink` and `HardLink` answer
  `Linked`, and the requests that set a mode, an owner or an attribute,
  or remove one, answer `Changed`. io checks each request's names, mode,
  target, attribute name and value against the record's bounds, and
  answers one outside them `InvalidArgument`, before any record. What
  they write is durable as ../kernel.md, 6.4 says: a link's name once the
  owner syncs its directory (`SyncDirectory`), the rest once it syncs the
  entry's file or its filesystem (`SyncFilesystem`). A file is made with
  `Create`, a directory with `MakeDirectory`, and cut with `Cut`.
- **The filesystem** of an open file or directory: `FilesystemStatus`
  answers its kind and its space, and `SyncFilesystem` makes everything
  written to it durable, answering `Synced`. **btrfs's subvolumes**
  (../kernel.md, 6.5): `SubvolumeCreate` and `Snapshot` make one at a name
  in an open directory (`Made`), and `SubvolumeDestroy` destroys one
  (`Removed`); `SubvolumeReadOnly`, `SubvolumeInfo`, `SubvolumeChildren`
  and `SubvolumePath` take a subvolume's root, held open as a directory.
  `SubvolumeChildren` answers one page, at most 255 children, from tree
  ID `from` up: a page shorter than 255 is the last, and after a full one
  the owner asks again from the last tree ID plus one. Off btrfs each
  fails `NotSupported`.
- **These requests stall the loop.** Every one but `StatEntry`,
  `MakeLink` and `HardLink` is synchronous (../kernel.md, 6): it runs in the
  loop, at submit, and nothing stops it. A request's deadline cannot cut
  one short, and a stalled filesystem stalls the service. The writing
  subvolume requests and `SyncFilesystem` are long even on a healthy
  disk (../kernel.md, 6.5). A service that must not stall makes them in a
  process of its own.
- **Removing** takes one name. `Remove` removes a file, or, with
  `directory`, an empty directory, never what lies inside one: a
  directory that is not empty fails with the kernel's error, as
  `unlinkat` with `AT_REMOVEDIR` does (../kernel.md, 6.1). io never
  traverses to remove a tree. Its owner removes the names it knows, then
  the directory, so nothing it did not write is removed.
- **Writing a file that exists:** `OpenWrite` opens an existing regular
  file to read and write, never creating it (`Missing` if it is absent),
  so a writer that resumes a file after a restart writes at the offsets
  it chooses; `Cut` sets an open file's length, dropping what lies past
  it, durable once the file is synced. They are what a file of records
  appended at a known committed length needs after a crash: open it, cut
  it to that length, sync it, and write on from there (../../data/kv.md, section 6's
  payload logs; a transcript's file in an application's store).
- **What the records allow** (../kernel.md, 6.1). Only an `Open` resolves a
  path beneath a root, so io opens the directory a name lies in, beneath
  the root, before it renames or removes that name or makes a directory
  there, and it stats what it has opened: a stat needs the file readable,
  but for the path-only open that names why a private open was denied
  (3). Reads and writes may be short,
  and io continues them; a listing comes an entry count at a time, and
  io's stated count is a limit it checks as the entries come.
- **Replacing a file, in order:** `Stat` the old file, opened to read, for
  its permission bits; `Create` the temporary, beside the target in its
  directory, with those bits; `Write` it all; `Sync` it; `Close` it;
  `Rename` it over the target; `Sync` the directory. A reader sees the old
  file or the new, never part of either, and after the last `Sync` the
  new one survives a crash (../kernel.md, 6.1). `Absent` uses an atomic
  no-replace rename: any entry at the target makes it fail with `Conflict`.
  `Any` skips the conditional recheck and uses a replacing rename.
  `Digest` rechecks the target before a replacing rename (2); a recheck
  is not a lock, so an outside writer can still race with that rename.
  `Stored` carries the new content's digest.
  - The temporary is named `<target>.skein-<16 hex>`: the target's name
    (its first 230 bytes), `.skein-`, and 16 lowercase hex digits drawn
    from io's seeded randomness. The name's shape is part of the
    contract, so that an owner's scan can tell io's temporaries from its
    own names (2). A `Create` that finds the name taken
    (`Exists`) draws another.
  - The old file's permission bits are kept, never its setuid, setgid
    or sticky bits, which `Stat` also answers. A target that does not
    exist, or cannot be opened to read, is made with `0o666` less the
    umask.
  - A symbolic link at the target is replaced by the file, not written
    through: the rename gives the link's own name the new file.
- **One request at a time, each with a deadline:** the driver's timeout
  from when it starts, or the owner's own (`down_until`). At the
  deadline, or when the owner cancels, io cancels the operation in
  flight and holds the request until it settles; then, once every
  descriptor the request opened is closed, it sends the one terminal:
  `Failed` with `TimedOut`, or `Cancelled`, if the stop won, and the
  request's own if the operation finished first. A store gives up only
  before its rename: once the new file is in place, it runs to the commit
  terminal described in section 2. An operation that completes without
  a `Cancel` (../kernel.md, 6.1: a `Stat`, `StatEntry`, `Rename`, `Remove`,
  `MakeDirectory`, `List` or `Lock`, and every operation of its sections
  6.4 and 6.5) always finishes first, so a request
  whose answer it decides answers its own, whatever its deadline: a
  `Lock` past it still says `Locked` or `Busy`, and a `Rename` that
  landed says so. A request with more to do after it, such as a page of
  a listing with entries still to read, stops at that completion, unless
  the completion finished it.
- **Memory:** the open files' table and the driver's own buffers grow as
  they fill; an admitted `Load`, `Scan` or `Store` keeps its edge
  bounds. The buffers an owner hands in, `WriteAt`'s bytes and paths,
  are the owner's. The records of a tree's metadata and of btrfs take
  buffers at the kernel's largest (../kernel.md, 6.4 and 6.5), which io
  makes for the request in flight and hands up cut to their length, so
  one request holds at most 65,536 bytes for an attribute's list or
  value and 4,096 for a link's target or a subvolume's path.
- **Beyond whole files:** a file appended to as a stream (1); the
  durable replace that services keep their records with (2); and
  private files, for secrets (3). A file read as a stream, by demand,
  comes when a user needs to read one too large to hold.

## 1. Append streams

A file written as the service goes, a trace or a log, is a stream like a
write pipe's: lib's stream vocabulary (../lib.md, 7), output only, driven by
the ring like any stream. It replaces a shell thread that writes the
file beside the loop (../../../foundation/programming-model.md, 2.1).

- **Made by adoption.** The shell opens the file at startup, beneath one
  of its roots (../shell.md, 6), to write at its end, creating it with the
  mode configuration gives if it is absent, and io adopts the descriptor
  as a write stream, as it adopts an inherited pipe. A standard output
  that is a regular file, as when a person redirects it, is adopted the
  same way. Opening one beneath a root while the service runs, as a file
  request, comes when a user needs it.
- **Writes go at the end,** one in flight, in order, each an `Append`
  (../kernel.md, 6.1), from the output queue under its cap, as for a pipe:
  room, a `Send` within it, `Finish`. A short write continues from its
  offset in the same box. Each write
  lands at the file's end, so another process appending to the same file
  may land between two pieces of one `Send` cut short: a file has one
  appending writer, or its writers accept that.
- **The kernel's copy, not the disk's.** A write is done once the kernel
  took its bytes, which then outlive the process but not a crash of the
  machine. The stream does not sync; what must survive a crash is
  replaced whole (2).
- **A write deadline gives up on a stalled filesystem.** A regular file
  has no peer to stop reading, but its filesystem may stall (../kernel.md,
  6.1), and a write that never completes would keep facts waiting in
  their queue for good. So the owner states, when it adopts the stream,
  the most one write may stay in flight. io arms that deadline with each
  write and, at it, cancels the write, keeps the stream settling until
  the write's completion arrives, drops what is queued, and tells the
  owner the stream failed (`Failed`), once. The owner decides what an
  abandoned stream means: a trace's sink records it where it still can.
  As for a whole-file operation (`files.md`), the deadline is the owner's
  policy and io's mechanism.
- **An inherited write pipe takes the same deadline when its owner
  states one.** A standard output that is a pipe or a terminal, adopted
  to carry such a stream, as an event stream's, stalls when its reader
  stops reading, as a file stalls on its filesystem. Its owner states
  the write deadline when it adopts the pipe, and io arms, cancels and
  tells it the same way. Without one, a write pipe waits on its reader,
  as a child's pipe does.
- **Close flushes.** `Close` writes what is queued, then closes the
  descriptor: there is no half-close, and nothing to drain. While the
  stream closes, its close deadline (`sockets.md`) replaces the write
  deadline, as a function of its state, so a filesystem that stalls at
  the end costs the stream its last bytes, never the process its end
  (../../../foundation/programming-model.md, 5.2). `Abort` drops what is queued and cancels
  the write in flight.
- **Backpressure is the owner's choice.** An owner that must lose
  nothing asks for room before it encodes (../../../foundation/programming-model.md, 7), so a
  slow disk lets facts wait in their uncapped queue until the stream
  takes more. One that may lose drops and counts when the room does not
  come (../../../foundation/programming-model.md, section 3).

## 2. Durable replace

Replacing a file whole, in the order above, is how a service keeps a
record it must neither lose nor tear: a store's file, a configuration it
writes, a token record. It is the idiom of ../kernel.md, 6.1 (`Create`,
`Write`, `Sync`, `Rename`, then `Sync` of the directory) as one request,
run by io.

- **One request, one terminal,** answered `Stored` only after the
  directory's `Sync`: the new content then survives a crash. A reader
  sees the old file or the new, never part of either.
- **The rename is the commit.**
  - Before it, a failure, a deadline, a cancel or a `Conflict` found at
    the recheck removes the temporary before the one terminal, and the
    old file is as it was. So does a rename that does not commit: an
    `Absent` one that finds the target taken, or one whose flag the
    filesystem rejects. If the temporary cannot be removed, as on a file
    system that has turned read-only, the terminal says so and names it,
    and nothing waits on it. The old file is still as it was, the next
    replace of the target does not depend on the temporary, and a scan
    may remove it.
  - After it, the new file is in place. A failure of the directory's
    `Sync`, or its deadline, answers `Failed` saying that the replace
    committed: the new content is what a reader sees, and its durability
    is unknown. Nothing is rolled back, and the same replace done again
    makes it durable.
  - A cancel after the rename is not honoured: the request runs to its
    terminal.
  - Rolling back would need a second copy of the old file, and would
    still leave a window a crash could fall in, so none is kept.
- **What a crash leaves:** the old file or the new, whole, and at most a
  temporary beside the target, named from it. The next replace of the
  target does not depend on it, and a scan may remove it.
- **What the request expects of the target,** one of three, always
  stated:
  - `Any`: replace whatever is there, or nothing. No read of the old file
    is needed, so a target io cannot read is replaced all the same;
  - `Absent`: there must be no target at the final rename. Any entry
    there, including an unreadable file, a dangling link or a directory,
    is answered `Conflict` without replacing it;
  - `Digest`: the target must hold exactly the content with this digest.

  `Any` skips the conditional recheck; opening the old file for its mode
  still applies. For `Absent` and `Digest`, io checks once more before
  the rename. A failed expectation, or a target io cannot read for
  `Digest`, is answered with nothing replaced: `Conflict` or the read's
  fault. `Absent` also commits with `RENAME_NOREPLACE`: even when two
  creators both rechecked a missing target, only one can create it; the
  other answers `Conflict` and cleans its temporary before that terminal.
  A filesystem that rejects the flag with `EINVAL` answers `Failed` with
  `InvalidArgument`, uncommitted, after temporary cleanup; io never
  retries with a replacing rename. `Digest` uses a replacing rename after
  its optimistic recheck: a recheck is not a lock, and an outside writer
  can change the target between that check and the rename.
- **The mode** is the old file's, or `0o600` for a private file (3).
- **Its deadline** is the owner's, as for every whole-file operation. A
  replace that fails or times out before its rename removes its
  temporary before its one terminal, and leaves the old file as it was.
  After its rename, it answers as the commit says.
- **Not here:** updating part of a file, or a set of records larger than
  one write. Those are skein-kv's (../../data/kv.md).

## 3. Private files

A secret a service keeps between runs, such as a token record or a key,
lives in a private directory, readable by the service's user alone. io's
part is the modes and the checks; what the files hold, and their names,
are the owner's (../../oauth/accounts.md).

- **A private root.** The directory is made `0o700` if it is absent
  (`MakeDirectory` takes a mode, as `Create` does; ../kernel.md, 6.1), the
  directory it lies in synced (`files.md`), and opened beneath its root
  as a root of its own, without following a link at its name. io stats
  what it opened and refuses it unless it is a directory with no group
  or other permission bits and the expected owner, saying what it found
  unsafe (a link, a non-directory, group or other bits, another owner),
  so an owner tells a planted file from a failed read.
- **Private files are made `0o600`,** less the umask, which only takes
  bits away, and are replaced whole with that mode (2), never with the
  old file's: a file found with group or other bits is not copied
  forward.
- **Reading one checks what was opened:** a regular file, reached
  without following a link, with no group or other permission bits, one
  link, and the directory's owner. Anything else is refused unread. It is
  read whole within a stated maximum (`files.md`). If a replace by rename
  between opening and stat leaves the opened file with no links, a
  private load closes it and reopens its path once. It checks the new
  file's metadata by the same rules, then reads that version. A second
  such replace is refused; every other private-file refusal is unchanged.
- **What io does not do:** encrypt, lock, or keep a secret out of memory.
  A secret's bytes are its owner's, which keeps them out of logs, traces
  and `Debug` (../../oauth/README.md, 4).

The checks of owner and links need the kernel's `Stat` to answer both
(../kernel.md, 6.1). The owner they expect is the process's own effective
user: the shell reads it once at startup (../shell.md, 6) and gives it to
io's file configuration, never inferred from a path or its parents. A
private root or file owned by anyone else is refused, saying so.

- **A denied open is still named.** A private root or file of another
  user is usually unreadable to this one (`0o700`, `0o600`), so its open
  fails with `Permission` before there is anything to stat. io then opens
  the same path again, beneath the same root, for its path only
  (`OpenHow::PathNoFollow`, ../kernel.md, 6.1), which needs no permission on
  the entry itself, stats that, closes it, and answers the refusal it
  found: another owner, a link, a non-directory, group or other bits, in
  the order the checks above take. Only when that stat finds nothing
  unsafe (a `0o000` directory of this user, say), or the path-only open
  fails too, is the answer the first open's `Failed`, unchanged.
- **The path-only descriptor decides nothing.** It is never read and
  never makes a file acceptable: every check that admits a root or a
  file is made on the descriptor that will be read. It only names a
  refusal, so a swap between the two opens can change which refusal is
  told, never let an unsafe file be read. This is why a user who once ran
  the service as root hears that the directory is root's, not that
  permission was denied.
