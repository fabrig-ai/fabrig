# The tree helper

## 1. Why it exists

The builder runs as itself; a layer's files belong to its subordinate
range. So the builder cannot:

- **read every file:** `/etc/shadow` is 0640 and `/root` 0700, both the
  namespace root's;
- **unlink** in the namespace root's directories, so cannot empty a
  layer;
- **change owners and modes,** so cannot apply a change set;
- **snapshot a layer or set its read-only flag:** btrfs allows both only
  to the owner of the subvolume's root (`inode_owner_or_capable`;
  verified, `EPERM` for a user on a subvolume its subordinate root
  owns).

Root of a user namespace that maps the range can do all of it. That is
the tree helper: hearth's own code, `hearth-domain-tree` on
`hearth-service-tree`, in the same binary.

## 2. How it starts

- **The builder spawns it,** it unshares a user namespace, and the
  builder maps it with the setuid helpers:

  ```
  newuidmap <pid> 0 <base> 65536  65536 <builder uid> 1
  newgidmap <pid> 0 <base> 65536  65536 <builder gid> 1
  ```

  Then it takes uid and gid 0 in its namespace. mmdebstrap does the
  same dance: a process left outside runs `newuidmap` on the one that
  unshared. In hearth it is skein's, as spawning with an id map
  (hearth/docs/design/README.md, 7.1).
- **The builder's own ids are mapped too, at 65536:** to write the
  store's directories, read the parent's tree file, and give a new
  subvolume's root to the namespace's root. Build steps' sandboxes map
  the range only (hearth/docs/design/domain/build/sandboxes.md, 1).
- **It acts beneath an inherited descriptor of the store's root:**
  paths resolved with `openat2` and `RESOLVE_BENEATH |
  RESOLVE_NO_SYMLINKS`, stats with `statx` and `AT_SYMLINK_NOFOLLOW`.
  It follows no link.
- **Its pipes are its only channel.** If it dies, the request in flight
  fails, the staging layer it touched is discarded, and the builder
  starts another.

## 3. Its records

**The tree helper does seven things:** walk, hash, apply and empty
trees, and snapshot, freeze or thaw, and delete subvolumes. The last
three are the helper's, not the builder's, because btrfs allows them
only to the owner of a subvolume's root, which is the namespace's root
(1). Its messages are `hearth-protocol-helper`'s: sized, each request
with a token that its one answer echoes.

| Request | Answer |
|---|---|
| `snapshot` a layer into a new name, writable | `snapshotted` |
| `freeze` a layer: the read-only flag, then `syncfs` (`README.md`, 4.2); `thaw` one | `frozen`, `thawed` |
| `walk` a layer against its parent's tree file, with the scan's rules and, for a packages layer, the kernels' inputs (`records.md`, 3); write its tree file | `walked`: tree digest, change set digest, the tree file's SHA-256 and size, counts, findings |
| `hash` a layer with no parent: every file read | `hashed`, as `walked` |
| `apply` a change set from `<key>.changes`, contents by digest | `applied`: counts |
| `empty` a layer | `emptied`: entries removed |
| `delete` a layer whole, where `user_subvol_rm_allowed` is set | `deleted` |
| any | `failed`: the path and a typed cause: not found, permission, a bound, a refused entry, no space, an io failure of no other kind, a bad input |

- **One request at a time;** a builder may run more than one helper
  (one by default).
- **On the pipes, a message is a length and a map,** not a frame of a
  kind (`records.md`, 1): a `u32` BE length, then a map (`README.md`, 3.1) of `kind`, the
  request's or answer's name, `token`, and `fields`, the map its kind
  names. A scan rule is `list [ how, path, what, with ]` (hearth/docs/design/domain/build/README.md,
  7.3). A message is bounded at 4 MiB, 256 rules and 64 findings; a
  message past a bound, or that its kind does not name, ends the helper,
  as its death does (2). The fields of each kind are
  `hearth-protocol-helper`'s, from this section's table.
- **Big data stays in files:** tree files written by the helper, owned
  by the builder's ids, named in answers by SHA-256; a change set to
  apply is written by the builder to `layers/<key>.changes`, beside the
  staging layer, as a tree file's items of tag 2 with each put's entry
  (`records.md`, 3), and deleted once applied, or with the staging layer. The pipes
  carry summaries.
- **Formats are machines, each its own lib-only crate**
  (`skein/docs/foundation/programming-model.md`, section 4), so the helper's domain parses
  nothing:
  - `hearth-protocol-tree`: the decoder of `README.md`, 3.1's encoding and the
    store's record and tree files, which share the frame (`records.md`, 1); it depends on skein-lib and `hearth-lib` only;
  - `hearth-protocol-accounts`: the lines of `/etc/passwd`, `/etc/group`,
    `/etc/shadow` and `/etc/gshadow`, read from a layer, as typed
    entries: names to ids for apply's owners (5), and whether each
    password field is locked, for the scan (hearth/docs/design/domain/build/README.md, 7.3). An empty
    password field is not locked; a group's password fields are read as
    a user's; an id past 65,535 is refused, since a layer holds one range
    of 65,536 ids (`README.md`, 7.2);
  - `hearth-protocol-xattr`: `security.capability` and POSIX ACL values,
    checked as the walk reads them and apply writes them;
  - `hearth-protocol-helper`: the helper's protocol layer, which routes
    bytes between those machines, `hearth-domain-tree` and io: the
    pipes' messages, the files the helper reads and writes, and the
    attribute and account files it decodes.

## 4. The walk

- **Depth-first with an explicit stack, each directory's names
  sorted,** so entries come out in tree order, merged with the parent's
  tree file read as a stream: a path in both is compared (`README.md`, 4.3), one
  only in the walk is a put, one only in the parent is a delete.
- **A changed entry is read whole:** skein's `StatEntry`,
  `ListAttributes` and `GetAttribute` for the covered families,
  `ReadLink`, and a file's contents, hashed as they stream (
  skein/docs/design/kernel.md, 6.4). Files with more than one link are grouped by inode
  number.
- **The walk checks what a stream cannot defer:** a path with a hard
  link to an entry seen before must agree with that first entry's mode,
  owners and attributes; and a directory's entry, its attributes read,
  is written before any entry beneath it, so a child never waits on its
  parent. A mismatch is a refusal of the layer, never a repair.
- **The digests come from the finished tree file.** The tree's entries
  and `README.md`, 3.5's change set each name their count first, so the helper writes
  the tree file as it walks, then reads it twice more: once for the tree
  digest, and once for the change set's, which hashes the child's tree
  digest before its changes. Then it writes the end item (`records.md`, 3). Each
  read is of a file the page cache holds. The walk itself is
  `hearth-domain-tree`, which parses nothing: it hands typed items down,
  and the helper's protocol layer encodes them, reads the file back for
  the digests with the counts the walk gives, and returns the file's
  SHA-256 and size once the end item and the header are written.
- **The scan's rules come with the request** as data (hearth/docs/design/domain/build/README.md, 7.3).
  The helper checks them on what changed, since only it can read every
  file, and answers findings, never contents. A rule on a file's lines
  (a password field locked) is judged on the typed entries
  `hearth-protocol-accounts` gives the helper's domain (3).
- **`hash` is a walk with no parent.** It verifies a layer against its
  record (a scrub, a damaged tree file) and makes a tree file where
  none exists.
- **Later, the walk clamps modification times.** The reproducible
  retrofit has it set every mtime newer than the layer's time to that
  time as it visits changed inodes, and record the change time read
  after the clamp, so `README.md`, 4.3's rule stays sound (hearth/docs/design/domain/build/reproducible.md, 5.1). No
  digest changes: times are left out (`README.md`, 3.4).

## 5. Apply, empty and the subvolume operations

- **Apply** takes a change set in tree order: deletes first, deepest
  first; then puts, parents first. A file is written to a temporary
  name in its directory and renamed over its path, its contents from
  `cache/<sha256>` (copied; a clone with `FICLONE` is a later item,
  `README.md`, section 10) or from a peer's stream (`README.md`, 8.2). For each put: owners first (a `chown`
  clears setuid, setgid and capabilities), then the mode, then
  `security.capability`, then ACLs and `user.*`; covered attributes the
  entry does not list are removed. It refuses a non-canonical path, an
  owner past 65535, a device (no `mknod` in a user namespace), a socket,
  an uncovered attribute. It does not hash: the walk after it does, and
  the tree digest is the check.
- **Uses in M2:** the empty layer's root; the config layer's file tree,
  owners resolved by name from the layer's own `/etc/passwd` and
  `/etc/group` as `hearth-protocol-accounts` decodes them (hearth/docs/design/domain/build/README.md, 6.3),
  and hearth's own sysusers, tmpfiles and preset files (hearth/docs/design/domain/build/README.md, 6.2 to 6.5
  ); single-file downloads (hearth/docs/design/domain/build/README.md, 5.2).
- **Refused in M2 as well:** a FIFO, which no M2 layer's change set
  makes; it comes with a need. Entries' times are not set, since no
  digest holds them (`README.md`, 3.4): setting them comes with the slot's shift in
  M3 (hearth/docs/design/domain/slots/prepare.md, 2).
- **Records:** skein's `MakeDirectory`, `Create`, `Truncate`,
  `MakeLink`, `HardLink`, `SetOwner`, `SetMode`, `SetAttribute` and
  `RemoveAttribute`, beneath the store's root (skein/docs/design/kernel.md, 6.4).
- **Empty** removes everything beneath a layer's root, deepest first, so
  the builder can `rmdir` it (`README.md`, 5.3). Stopped half-way, it is done again.
- **Snapshot, freeze, thaw and delete** are skein's `Snapshot`,
  `SubvolumeReadOnly` and `SubvolumeDestroy` (`BTRFS_IOC_SNAP_CREATE_V2`,
  `BTRFS_IOC_SUBVOL_SETFLAGS`, `BTRFS_IOC_SNAP_DESTROY_V2`), the
  helper's because a layer's root is the namespace root's (1). Freeze
  ends with `SyncFilesystem` (`README.md`, 4.2).

## 6. Bounds

| Bound | Default |
|---|---|
| entries in a layer | 2,000,000 |
| names in one directory | 262,144 |
| depth of the walk | 256 |
| names held across the walk's stack, in all | 1,048,576 |
| inodes with more than one link, held by the walk; their first paths, in all | 65,536; 16 MiB |
| a path; a link target | 4096 bytes each |
| attributes per entry; a name; one value | 64; 255 bytes (`XATTR_NAME_MAX`); 65,536 bytes (`XATTR_SIZE_MAX`) |
| a tree file; an item in it | 1 GiB; 4,718,592 bytes |
| kernels in a layer's `boot` | 8 |
| a message on the pipes | 4 MiB; 256 rules, 64 findings |
| requests in flight per helper | 1 |

Memory is the sorted names of each directory on the stack, bounded in
all and not only per directory and per level, each inode with more than
one link (its first path, and a digest of the mode, owners and
attributes its later paths must agree with), and the frames in flight:
tree files stream in and out. A bound passed fails the request, naming
the path.

Contents never pass through a domain whole: the helper hashes files as
they stream, and render's static blobs reach `cache/<sha256>` streamed
by its protocol layer (hearth/docs/design/domain/render/README.md, 2.3), whence apply reads them.

## 7. Costs

- **An unchanged entry costs one `statx`;** a changed file, a read and a
  hash. A config layer over 100,000 entries is one pass of `statx` and a
  few files hashed: under a second warm (an estimate, to measure).
- **A packages layer is hashed whole,** at the helper's SHA-256 rate:
  6 seconds for 1.5 GB at 250 MB/s. skein's SHA-256 is byte at a time
  and unmeasured (hearth/docs/design/README.md, 7.1 asks for a fast streaming one).
- **Emptying costs about 15 µs a path** (measured: 3,942 paths in
  0.062 s, unprivileged). A snapshot and a freeze cost constant time.
- **Each request's time, entries, bytes read and hashed** are costs
  (hearth/docs/design/domain/build/README.md, section 10).

## 8. The ioctls, and who makes them

| Call | skein's record | Made by | For |
|---|---|---|---|
| `BTRFS_IOC_SUBVOL_CREATE` | `SubvolumeCreate` | the builder | the empty layer's subvolume (`README.md`, 4.1) |
| `BTRFS_IOC_GET_SUBVOL_INFO` | `SubvolumeInfo` | the builder | a subvolume's uuids and flags (`README.md`, 4.2, 6.2) |
| `BTRFS_IOC_GET_SUBVOL_ROOTREF` | `SubvolumeChildren` | the builder | the subvolumes under the store, and nested ones (`README.md`, 6.2) |
| `BTRFS_IOC_INO_LOOKUP_USER` | `SubvolumePath` | the builder | each subvolume's path (`README.md`, 6.2) |
| `BTRFS_IOC_SNAP_CREATE_V2` | `Snapshot` | the tree helper | a writable snapshot of a parent (`README.md`, 4.2) |
| `BTRFS_IOC_SUBVOL_SETFLAGS` | `SubvolumeReadOnly` | the tree helper | freeze and thaw (`README.md`, 4.2, 5.3) |
| `BTRFS_IOC_SNAP_DESTROY_V2` | `SubvolumeDestroy` | the tree helper | delete whole, where `user_subvol_rm_allowed` is set (`README.md`, 5.4) |
| `syncfs(2)` | `SyncFilesystem` | the tree helper | the layer durable, at the end of freeze (`README.md`, 4.2) |
| `fstatfs(2)` | `FilesystemStatus` | the builder | the store on btrfs, and its space (`README.md`, 6.1, 6.2) |

All are unprivileged: the ioctls the builder makes for any user, the
helper's for the owner of a subvolume's root. They are synchronous
records of skein's ring adapter, a closed list (skein/docs/design/kernel.md, 6.5;
hearth/docs/design/README.md, 7.1), made in two processes: the builder and its tree
helper. The long ones (a snapshot, a flag, a destroy, a sync) are the
helper's alone, so they never stall the builder's loop; the builder's
own are made once at the store's start (`SubvolumeCreate`, for the
empty layer) or are reads. `FICLONE`, to clone a file from the cache
(5), is not among them in M2: apply copies, and the clone comes with
a measured need (`README.md`, section 10).
