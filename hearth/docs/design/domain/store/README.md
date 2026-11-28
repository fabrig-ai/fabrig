# The store

Provisional, 2026-10-09, revised 2026-10-19. The builder's store on
btrfs: its layout, the canonical encoding every digest uses, layer keys
and tree digests, a layer's life, the tree helper, records, pins and
collection, how each builder reads the store back, the steward's records
beside it, what the machine must provide, and how peers exchange layers.
M2 builds it, but for peers (section 8, M6). M1 builds the encoding and
`H` (`digests.md`, 1.1, 1.2), and a store without layers, which holds render's cache
and resolutions (2.4). How a layer's contents are made is hearth/docs/design/domain/build/README.md.
What is open is in section 10.

The companion pages are [`digests.md`](digests.md) for encoding, hashes and keys,
[`state.md`](state.md) for startup and recovery, [`records.md`](records.md) for
record formats, and [`tree-helper.md`](tree-helper.md) for tree operations.

## 1. In one page

- **The store is a btrfs subvolume of the builder's own, mode 0700.**
  It holds layers with their records and tree files, resolutions, a
  download cache by SHA-256 and the scratch of steps in flight. Every
  name in it is a digest in lowercase hex.
- **One encoding, one hash.** Every digest is SHA-256 over the
  canonical encoding of `digests.md`, 1.1: typed, length-prefixed, maps sorted by key.
  `H(tag, values)` hashes a list that starts with a version and a tag.
- **Layer keys are cumulative:** packages, software and config are each
  `H` of the key below and their own part's digest; the config key
  names the output. The packages key takes the digest of what the
  resolution resolved to, not of what was asked.
- **A tree digest is what a tree is, as the namespace sees it:** paths,
  kinds, permission bits, owners as namespace ids, capabilities, ACLs,
  `user.*` attributes, contents, link targets and hard links. No times,
  no inode numbers.
- **A layer is one read-only subvolume, `/var` included:** a writable
  snapshot of its parent, changed by build steps, walked, frozen,
  recorded and published by a rename. What a crash leaves half-made is
  deleted at startup.
- **What changed is found by a walk.** An entry whose path, inode number
  and change time match the parent's record is unchanged; only the rest
  is read and hashed. A snapshot keeps inode numbers and change times.
- **The tree helper does what the builder cannot.** Root of a user
  namespace mapped onto the builder's subordinate range, it walks,
  hashes, applies and empties layers, and snapshots and freezes them,
  which btrfs allows only to the owner of a subvolume's root.
- **Records sit beside what they describe,** in one framed, versioned
  format, each written to a temporary name and renamed.
- **Pins keep layers and their ancestors; collection deletes the
  rest:** unrecord, clear the read-only flag, empty inside the
  namespace, `rmdir`. Builders and test filesystems may delete whole
  with `user_subvol_rm_allowed`.
- **The filesystem is hearth's only state.** At each builder's start
  three unprivileged btrfs ioctls and the records rebuild what it holds
  in memory. No kv store.
- **Peers exchange change sets [M6]** by layer key over HTTP, applied to
  a snapshot of a parent the receiver holds and checked by tree digest,
  within one plane.

## 2. On disk

### 2.1 The layout

```
<store>/                           a subvolume, 0700, the builder's (@hearth.store on a machine)
├── store.record                   the store's own record (6.6)
├── layers/                        0700
│   ├── <key>                      a published layer: a read-only subvolume
│   ├── <key>.record               its record (6.2), 0644
│   ├── <key>.tree                 its tree file: entries and change set (6.3), 0644
│   ├── <key>.staging              a layer being built: a writable subvolume (4.2)
│   ├── <key>.trash                a layer being deleted (7.3)
│   └── <key>.tree.tmp             a tree file being written (6.8)
├── resolutions/<key>.record       a resolution (6.4)
├── cache/<sha256>                 bytes by content: downloads, kept indices, render's blobs; 0644, never modified (2.2)
├── cache/<sha256>.record          where they came from (6.5)
├── cache/<random>.tmp             a file being written, whose digest is known at its end: renamed to <sha256> (6.8)
├── <record>.skein-<16 hex>        beside any record, in any directory: skein's Store replacing it (6.8)
├── targets/<H("target", name)>.record   a target's last builds (6.6)
├── state/                         the steward's records and rings (8.3)
├── artifacts/                     [M6] artifacts, as layers/ holds layers: <key>, .record, .tree, .staging, .trash
├── sources/, caches/              [M6] artifacts' sources and compile caches, as subvolumes (../build/artifacts.md, 3.2, 6.1)
├── repo/                          the builder's clone of the IaC repository (../render/README.md, 2.2)
├── credentials/                   0700: the builder's own read credential for it (../render/README.md, 2.2)
└── staging/<key>/                 scratch of a layer being built: fetched files, the local repository, inputs;
                                   or of a resolution being made, by its resolution key (../render/spec.md, 2.2)
```

- **Every name beneath the top is a digest,** 64 lowercase hex digits,
  then a suffix for its kind; nothing is named by a target, a path or a
  time. The one exception is a cache file being written, whose digest is
  known only at its end: it is named by 64 random hex digits, drawn from
  the builder's generator, and `.tmp`. The other is a record being
  replaced: skein's `Store` writes it as `<record's name>.skein-<16 hex>`
  beside it (`records.md`, 8). The top's names are the fixed ones above, and
  `store.record`'s temporary. A name of no known
  shape is not hearth's: the startup scan reports it and leaves it alone
  (`state.md`, 1.2).
- **Directories are 0700; records, tree files and cache files 0644,**
  all the builder's: user `hearth` on a machine, the developer on a
  developer's machine. The builder runs with the umask 022, and skein's
  `Store` makes a file `0o666` less it, with no mode of its own: the
  store's directories keep everyone else out (2.2).
- **Two writers, apart:** on a machine the steward writes only
  `state/`, and the builder writes the rest and never `state/`
  (hearth/docs/design/domain/steward.md, 2.2). Both run as `hearth`.
- **Artifacts [M6]** are read-only subvolumes named by key, with records
  and tree files beside them, built, published and deleted as layers are
  (hearth/docs/design/domain/build/artifacts.md, section 4); they live in `artifacts/` so that their
  renames, too, stay within one directory (2.3).

### 2.2 Owners and modes

- **A layer's files are owned by the builder's subordinate range.** Its
  root is a directory, 0755, owner and group 0 in the namespace (the
  range's base on disk), so the builder, as "other", can always open it
  to ask btrfs about it (`state.md`, 1.2).
- **Cache files are 0644 in a 0700 directory.** A sandbox reads them
  through a bind mount (hearth/docs/design/domain/build/packages.md, 3), where the builder's own uid is
  unmapped and only the "other" bits apply; the directory keeps everyone
  else out.
- **The cache and staging share the store's subvolume,** so a packages
  step's local repository is hard links to cache files: `link(2)` across
  subvolumes fails with `EXDEV`.

### 2.3 Why staging and trash live in `layers/`

- **A rename into another directory needs write permission on the
  directory moved,** to rewrite its `..` (`vfs_rename`, the kernel's
  `fs/namei.c`). A layer's root is the namespace root's, not the
  builder's, and once frozen btrfs refuses write permission on it
  (`EROFS`).
- **A rename within one directory needs only that directory's
  permissions,** which are the builder's. So a layer is built as
  `<key>.staging`, published as `<key>` and retired as `<key>.trash`,
  all within `layers/`.

### 2.4 A store without layers

A store whose filesystem is not btrfs, or whose builder has built no
layer, holds what render keeps: its cache and its resolutions. M1's
`render` and `diff` use one (hearth/docs/design/client/README.md, 3.5), and so may a builder that
only renders.

- **Any filesystem,** in a directory of the user's own, 0700: btrfs and
  the subordinate range are prerequisites of layers alone (section 7).
  `build` refuses a store that is not a btrfs subvolume [M2], naming what
  is missing (7.1).
- **What it holds:** `store.record` (`records.md`, 6), `cache/` with its records
  (`records.md`, 5), `resolutions/` (`records.md`, 4) and `staging/`, whose only scratch is a
  resolution's (hearth/docs/design/domain/render/spec.md, 2.2). The layout and the records are those of
  a store with layers, so a builder of M2 reads it as its own.
- **The lock and the scan hold:** a builder takes `state.md`, 1.2's lock on the
  store's root, and its startup scan runs steps 1 and 2, step 6 for its
  resolutions and its cache alone, and step 7, settling 4.4's rows for
  temporaries, staging, resolutions and cache files. Steps 3 to 5, the
  subvolumes', belong to a builder that builds [M2], which runs them on
  btrfs and skips them elsewhere, telling the two by `statfs`'s
  filesystem type (hearth/docs/design/README.md, 7.1). M1's builder builds no layer and
  runs none of them: a store it makes has no `layers/`.
- **Everything in it can be made again:** a resolution is resolved
  again, a cache file fetched or rendered again. A store without layers
  may be deleted whole, and made again as a btrfs subvolume when its user
  starts to build.

## 4. Layers

### 4.1 What a layer is

- **One read-only subvolume, `/var` included.** dpkg's database lives in
  `/var` and travels with the layer; a slot's `/var` is split from it at
  prepare (hearth/docs/design/domain/slots/prepare.md, 3).
- **Of four kinds:** `empty`, `packages`, `software`, `config`, each the
  parent of the next. Artifacts are layers of their own kind on the
  same machinery [M6] (hearth/docs/design/domain/build/artifacts.md, 4.2).
- **The empty layer** holds only its root, a directory, 0755, owners 0.
  The builder creates the subvolume (`BTRFS_IOC_SUBVOL_CREATE` needs no
  privilege); the tree helper applies the root's entry, since a new
  root is the builder's (`tree-helper.md`, 5). It is made with the store and never
  collected.
- **No subvolume nests in a layer:** a snapshot does not descend into
  one, and it blocks the parent's deletion.
- **Never changed once published.** The read-only flag is cleared only
  to delete (5.3).

### 4.2 A layer's life

```
1  snapshot   tree helper   writable snapshot of the parent → layers/<key>.staging
2  steps      sandboxes     domain/build.md, each in a sandbox over the staging subvolume
3  walk       tree helper   against the parent's tree file  → layers/<key>.tree.tmp
4  freeze     tree helper   the read-only flag set, then syncfs(2) on the store
5  record     builder       <key>.tree.tmp → <key>.tree; <key>.record written by rename
6  publish    builder       rename layers/<key>.staging → layers/<key>
```

1. **Snapshot:** `BTRFS_IOC_SNAP_CREATE_V2`, writable, by the tree
   helper, since the parent's root is the namespace root's (`tree-helper.md`, 1). The
   builder reads the new uuid and parent uuid
   (`BTRFS_IOC_GET_SUBVOL_INFO`).
2. **Steps** run in sandboxes (hearth/docs/design/domain/build/sandboxes.md). The last one's
   process tree is proved empty before the walk, and nothing else holds
   the staging subvolume, so nothing changes between walk and freeze.
3. **Walk:** the tree helper writes the tree file and answers the tree
   digest, the change set's digest, counts, and the scan's findings
   (hearth/docs/design/domain/build/README.md, section 7), any of which fails the layer.
4. **Freeze and sync:** `BTRFS_IOC_SUBVOL_SETFLAGS` with
   `BTRFS_SUBVOL_RDONLY`, then `syncfs(2)` on the store's filesystem,
   both by the tree helper, which answers `freeze` only after the sync.
   Records are synced as they are written (`records.md`, 8), but the files the steps
   wrote are not; without the sync, a power cut could leave torn files
   in a published layer under a record that claims its tree digest.
   `syncfs` is one call for the whole layer. It is synchronous and can
   take seconds, so it runs in the helper, whose loop has nothing else
   to do meanwhile, never on the builder's (skein/docs/design/runtime/kernel-files.md, 3).
5. **Record:** the tree file renamed into place, then the record, which
   names the tree file's SHA-256 and the uuids (`records.md`, 8).
6. **Publish:** a rename within `layers/` (2.3). A layer is published
   when `<key>` is a read-only subvolume whose uuid `<key>.record`
   names.

A failed step, a finding, a limit or a stop discards the staging
subvolume (5.3) and publishes nothing.

### 4.3 Unchanged: path, inode and ctime

- **The rule:** an entry whose path, inode number and change time (to
  the nanosecond) all equal the parent's record is unchanged, and keeps
  the parent's covered fields unread. Every other entry is read in full,
  and a file is hashed.
- **It holds** because a btrfs snapshot keeps every inode number and
  change time (verified: same `st_ino`, ctime and mtime after `btrfs
  subvolume snapshot`; btrfs-subvolume(8)), and because every change to
  a covered field moves the change time: a write, `chmod`, `chown`,
  `setxattr`, a link, a rename.
- **Caveats:** compare inode numbers, never `st_dev`, which differs per
  subvolume. On trixie's 6.12 the change time is coarse (btrfs has
  multigrain timestamps from 6.13), which is harmless: the parent was
  frozen before the child's steps began. A freed inode number may come
  back, with a new change time. A link added to an unchanged file moves
  its change time, so both paths are read again.
- **Nothing cheaper exists unprivileged:** `btrfs subvolume find-new`
  needs `BTRFS_IOC_TREE_SEARCH`, which needs root.
- **The local index** (inode, change and modification times) sits in
  the tree file beside each entry, outside the digest (`records.md`, 3). It is this
  machine's: a received layer gets its own from its first walk (8.2).

### 4.4 Crash recovery

The startup scan (`state.md`, 1.2) settles what a crash left:

| Found | What happened | The scan |
|---|---|---|
| `<key>.staging` | a build stopped | deletes it (5.3) |
| `<key>.trash` | a deletion stopped | finishes it |
| `<key>` with no record | a deletion began (5.3, step 1) | deletes it |
| a record with no `<key>` | a publish stopped before its rename | deletes the record and tree file |
| a record whose uuids are not the subvolume's | the subvolume is not the one recorded | deletes both |
| a record whose key does not recompute from its fields, or that cannot be read | a damaged record, or a version the builder cannot read (`records.md`, 8) | deletes it and its layer, reported once |
| `<key>.tree` with neither a layer nor a record | a deletion stopped after its rmdir (5.3, step 6) | deletes it |
| `cache/<64 hex>.tmp`, `layers/<key>.tree.tmp` and `layers/<key>.changes` (with or without a finding); a record's name, then `.skein-` and 16 lowercase hex digits, skein's `Store`'s temporary (`records.md`, 8), in `layers/`, `resolutions/`, `cache/` or `targets/`; anything in `staging/` | a write or a step in flight | deletes it |
| `cache/<sha256>` with no record | a cut between a cache file's rename and its record, or before the file was durable (`records.md`, 8) | deletes it |
| `cache/<sha256>.record` with no file | a deletion stopped (5.5) | deletes the record |

An interrupted build is built again when next asked for, and a cache
file deleted so is fetched or rendered again. A temporary is only one of
those shapes, in its own directory: any other name, `cache/notes.tmp`
among them, is a stray, reported and left alone (2.1).

- **A layer that no longer matches its record** is found by a check, not
  by the scan, which reads no file: the activator's verification of what
  it places (hearth/docs/design/domain/activator/README.md, section 5), a scrub, a peer's walk (8.2).
  It is never a lasting refusal of the key. The builder hashes the layer
  again (`tree-helper.md`, 4), in a `verify` job (hearth/docs/design/domain/steward.md, 3.3); if the tree differs
  from its record, it deletes the layer and every layer built on it
  (5.3) and builds them again; if it does not, the mismatch is the
  activator's to report.

### 4.5 Lineage through `parent_uuid`

- **A snapshot's `parent_uuid` is its source's uuid,** and
  `BTRFS_IOC_GET_SUBVOL_INFO` reads both without privilege.
- **A layer's record keeps both** as read after its snapshot. The
  startup scan checks them against the live subvolume, and the parent's
  record, while it exists, must name the same uuid as the child's
  parent uuid. So the chain of records is the chain of snapshots, and a
  subvolume swapped or made by hand is never taken for a layer.
- **A slot inherits it:** its parent uuid names the output it was
  snapshotted from (hearth/docs/design/domain/slots/prepare.md, 1).

### 4.6 One build per key

- **A key is built once at a time.** A request for a key in progress
  waits for it; one for a published key finds it.
- **Different keys build at once** within the builder's limits
  (hearth/docs/design/domain/build/sandboxes.md, 7); a chain is built bottom-up. On a machine the steward
  gives its builder the pass's jobs one at a time, so one of its own
  outputs builds there at a time, with others' work beside it from M6
  (hearth/docs/design/domain/steward.md, 3.4).

## 5. Pins and collection

### 5.1 What pins a layer

| Pin | From | Lasts |
|---|---|---|
| the empty layer | the store | always |
| the outputs of the machine's slots [M3], its containers' included [M5] | the activator's facts over the link (hearth/docs/design/domain/activator/README.md, 4.3), passed on by the steward in the jobs' `Welcome` and `Pins` (hearth/docs/design/domain/steward.md, 3.2) | while a slot holds it, or is prepared from it |
| the desired revision's outputs [M3]: the machine's target's, and its containers' [M5] | the steward's `hearth-domain-converge` (hearth/docs/design/protocol/standalone.md), in the same records | while it is desired |
| builds in flight | the plan (hearth/docs/design/domain/build/README.md, section 4) | until the build ends |
| a target's last N outputs | `targets/` (`records.md`, 6); N is 3 by default | until N newer builds |
| a peer's request [M6] | a lease per transfer (8.3) | until it ends or its deadline |
| an artifact in use [M6] | hearth/docs/design/domain/build/artifacts.md, 4.4 | as it says |

- **A pin holds a layer's ancestors too.** A child does not need its
  parent's subvolume (snapshots share extents, not existence), but the
  target's next build starts from it and a peer's change set is against
  it. An ancestor costs only what its descendants changed or deleted.
  A reconciled packages layer's base (hearth/docs/design/domain/build/README.md, 4.2) is an ancestor
  too: its line's next reconcile starts from it.
- **In M2 there is no steward:** the builder's root gives its store
  empty pins at its start, so the empty layer, builds in flight and the
  targets' last N outputs alone pin layers.
- **A builder collects nothing before it has the steward's pins,** in
  its `Welcome` (hearth/docs/design/domain/steward.md, 3.2): the activator's slots and the desired
  revision's outputs. The steward spawns no builder that may collect
  before its activator's `Welcome` has given it the slots' outputs.

### 5.2 Collection

- **What no pin holds is collected:** after each publish, at a
  builder's start once the pins are known, and when the store's
  filesystem falls below its free-space floor (`statfs`), which the
  steward watches between builders and answers with a `collect` job
  (hearth/docs/design/domain/steward.md, 3.3).
- **Oldest first, bounded per run,** 16 layers by default. What a
  deletion frees is not known
  in advance: without quotas, which need root, btrfs does not say what a
  subvolume holds alone.
- **Each deletion is reported** with its reason (hearth/docs/design/protocol/wire.md, section 5).

### 5.3 Deleting a layer

```
1  unrecord    builder       unlink layers/<key>.record; sync the directory
2  retire      builder       rename layers/<key> → layers/<key>.trash
3  thaw        tree helper   clear the read-only flag
4  empty       tree helper   remove every entry, deepest first
5  rmdir       builder       rmdir layers/<key>.trash: an empty subvolume, since Linux 4.18
6  tree file   builder       unlink layers/<key>.tree
```

- **Unrecorded first,** so a crash at any later step leaves an
  unrecorded subvolume or a `.trash`, which startup deletes.
- **Thawed first:** nothing in a read-only subvolume can be unlinked.
- **Emptied in the namespace:** the builder cannot unlink in its range's
  directories. `rmdir` of an empty subvolume checks only the parent
  directory, the builder's. A staging layer is deleted from step 3.

### 5.4 `user_subvol_rm_allowed`

- **With it, steps 4 and 5 are one `delete`** by the tree helper
  (`BTRFS_IOC_SNAP_DESTROY_V2`): constant time for the caller, the
  extents freed by btrfs's cleaner. The flag must be cleared first: a
  read-only subvolume refuses the write permission deletion checks
  (`EROFS`).
- **Only on builders and test filesystems.** The builder sees the option
  in its mount and uses it where set.
- **The caveat.** It is per filesystem (the first mount's options apply
  to every subvolume), and lets any user with write and search
  permission on a subvolume's root delete it whole, with others' files
  and immutable or append-only ones inside: the kernel's comment says
  it checks neither emptiness nor what could otherwise be deleted. On a
  machine whose slots and data share the filesystem with local users,
  that widens what a user can destroy. Incus sets it on its pools by
  default; hearth does not ask for it where slots run.

### 5.5 Resolutions and the cache

- **A resolution is kept** while a kept packages layer names its digest,
  and the newest per target for a configured time.
- **A cache file is kept** while a kept resolution names it (its
  `.deb`s and its kept indices), while a kept software layer's record
  lists it in `placed` (`records.md`, 2), while a build in flight uses it (render's
  blobs among them), and for a configured time after its last use, so a
  rebuild soon after a collection fetches nothing.
- **Collected by the store's collection,** after its layers, in the same
  run and by the same pins: what no kept layer, build in flight or
  configured time holds.
- **Deleting one** unlinks its name and record; a staging link keeps the
  inode until its step ends.
- **Nothing is collected in M1.** A store without layers grows with
  each resolution and each rendered or streamed file, and is full at a
  limit of `state.md`, 1.1; its user deletes it whole (2.4).

## 7. Prerequisites

### 7.1 btrfs and the mount

- **The store is a btrfs subvolume of its own:** `@hearth.store` on a
  machine, mounted at `/var/lib/hearth/store` in every slot (hearth/docs/design/domain/machine/README.md,
  3.2 and 3.3), or one a developer makes with `btrfs subvolume create`,
  which needs no privilege, and names with `--store` (hearth/docs/design/client/README.md, 3.5). The scan lists the store's child subvolumes, so the
  store must be their parent. Only layers need it: a store that only
  renders may be a plain directory (2.4).
- **Linux 4.18 or later,** for unprivileged `rmdir` of an empty
  subvolume (`/sys/fs/btrfs/features/rmdir_subvol`) and `state.md`, 1.2's ioctls.
  Trixie's 6.12 has them.
- **Mount options:** `nodev,nosuid,noatime` on the store's own mount
  (hearth/docs/design/domain/machine/README.md, 3.2), all per mount: no setuid file in a layer works from
  the store, and reading a layer writes nothing; `user_subvol_rm_allowed` (per filesystem) on
  builders and test filesystems only (5.4).

### 7.2 The builder's user and its subordinate range

- **User `hearth`,** a system user, the steward's and the builder's,
  with one subordinate uid range and one gid range of 65536 ids, at a
  base aligned to 64 Ki: the activator's shift is then a mask
  (hearth/docs/design/domain/slots/prepare.md, 2), as systemd's shifter assumes.
- **Made explicitly.** `useradd -r` gives a system user no range unless
  `-F` (`--add-subids-for-system`), and then from `SUB_UID_MIN`, 100000,
  which is not aligned. So: `usermod --add-subuids <base>-<base+65535>
  --add-subgids <base>-<base+65535> hearth`, which checks nothing; or
  `useradd -r -F -K SUB_UID_MIN=<base>` (to verify).
- **Base 262144 (0x40000) by default:** aligned, and below
  systemd-nspawn's automatic range (524288 to 1879048191, systemd's
  UID and GID documentation), which hearth's containers may use [M5] only with
  idmapped mounts: a shift into a container's own range needs its base
  at prepare, before nspawn would pick one (hearth/docs/design/domain/slots/containers.md, 1 and 4).
  hearth/docs/design/domain/machine/bootstrap.md) sets it at bootstrap.
- **The range is the builder's alone.** Whoever runs as its base owns
  every layer, so no other user, container or nspawn range may overlap
  it. hearth refuses a range missing, short, overlapping or in more
  than one entry, and a store made with another (`records.md`, 6); a host's
  activator refuses a container whose range overlaps it [M5].
- **Aligned where there is an activator.** Only the shift needs the
  alignment, so a machine with an activator refuses an unaligned range,
  while a builder alone (a developer's machine, a lab builder, the real
  tier of hearth/docs/design/testing/real-builds.md) accepts the range `useradd` gave it.

### 7.3 Namespaces and tools

- **Unprivileged user namespaces:** Debian's default
  (`kernel.unprivileged_userns_clone=1`). Ubuntu since 23.10 needs an
  AppArmor profile with `userns,` or
  `kernel.apparmor_restrict_unprivileged_userns=0`. An Incus container
  needs `security.nesting=true` (to mount `/proc` and `/sys`) and
  `security.idmap.size` above 65536.
- **`newuidmap` and `newgidmap`** (Debian's `uidmap`), setuid root.
- **A cgroup v2 subtree the builder can write,** for its sandboxes'
  limits: on a machine, `steward/builder/`, which the steward makes in
  the subtree the activator gives it, `steward/`, under its own
  delegated cgroup (`Delegate=yes`; hearth/docs/design/domain/steward.md, section 7); on a
  developer's machine, the client's own scope, run with `systemd-run
  --user --scope -p Delegate=yes`. What build does with it is hearth/docs/design/domain/build/sandboxes.md,
  7.
- **The build's tools** (hearth/docs/design/domain/build/README.md, section 8): mmdebstrap, apt, dpkg.
  On a machine, hearth's own task installs them in every target that
  builds.

### 7.4 Where the store lives

- **Reachable by `hearth` alone,** and of its processes only the
  builder and its helpers open a layer (hearth/docs/design/domain/steward.md, 2.2). A layer holds
  setuid files owned by the range: `su` is 4755, owned by the base.
  Whoever could reach and run one would run as the base, which owns
  every layer, and could rewrite an output before it becomes a slot.
  Guix met this (CVE-2024-52867: setuid outputs reachable before they
  were fixed up), and systemd warns of its foreign range the same way.
- **So it is mounted in one place only,** `/var/lib/hearth/store` in
  every slot, from the fstab the activator writes (hearth/docs/design/domain/machine/README.md, 3.3),
  with `nodev,nosuid,noatime`; its root is 0700 and owned by `hearth`,
  so no other user of the machine reaches anything beneath it.
- **No task places anything beneath it:** `/var/lib/hearth/` is
  reserved (hearth/docs/design/protocol/format/tasks-reserved.md, 1). It is never bound into a container, nor into a
  sandbox but for the one layer a step builds.
- **Read-only is no barrier to the builder,** which can clear the flag;
  the activator snapshots an output before verifying it (hearth/docs/design/domain/activator/README.md,
  section 5).

## 8. Peers [M6]

### 8.1 What moves

- **A layer moves as its change set against its parent,** by layer key:
  `GET /v1/change-set/<layer key>` (hearth/docs/design/protocol/wire.md, section 7). Not btrfs send
  and receive: both need root.

  ```
  header    the layer's record (6.2): key, parent, tree digest
  items     in tree order:
              delete  path
              put     entry; for a file whose content digest is not the parent's at that path,
                      its contents, streamed; its modification time, advisory
  end       the change set's digest
  ```

- **Contents travel only where the parent lacks them;** the receiver's
  snapshot has the rest.
- **Modification times travel outside the digest,** so a received tree
  keeps them: Python's byte-compiled files record their sources' times.
- **Downloads move by SHA-256,** `GET /v1/blob/sha256/<digest>`
  (hearth/docs/design/protocol/wire.md, section 7), checked as they arrive.

### 8.2 Receiving

1. **The parent first:** a receiver that lacks it fetches its change set
   the same way, down to a layer it holds; every chain ends at the empty
   layer, which every store has.
2. **Snapshot** the parent into `<key>.staging` (4.2).
3. **Apply** the stream with the tree helper as it arrives (`tree-helper.md`, 5).
4. **Walk** against the parent's tree file, hashing every file written.
5. **Check** the tree digest against the one the receiver's own engine
   gave for the key (hearth/docs/design/protocol/wire.md, section 7). A mismatch deletes the
   staging layer and is reported.
6. **Freeze, record** (`made.by` received, with the peer) **and
   publish.**

### 8.3 Serving

- **A machine serves the layers it holds,** and its cache by digest:
  the steward holds the HTTP listener, and the bytes come from a
  builder's job (hearth/docs/design/domain/steward.md, 2.1 and 3.3). The tree helper streams the
  change set from the tree file and the layer, since only it can read
  every file.
- **A request leases the layer and its parent** until it ends or its
  deadline passes (5.1), within a bound on transfers at once and per
  peer.

### 8.4 Trust per plane

- **A machine fetches only from its own plane's builders and caches,**
  and takes tree digests only from its own engine, over its
  authenticated wire: a lab output never reaches production.
- **A receiver cannot check a key against a tree** until builds are
  reproducible; it trusts its plane's builders for that, and checks the
  bytes against the digest (hearth/docs/design/domain/build/reproducible.md, section 7).

## 9. Its world

- **`hearth-domain-store`'s world** runs on the fake machine (hearth/docs/design/testing/fake-machine.md
  ): subvolumes and snapshots that keep inode numbers and
  change times, read-only flags, ownership checks that refuse the
  builder's snapshots and flags and allow the namespace root's, `rmdir`
  of an empty subvolume, the three ioctls, the mount option. The tree
  helper is a fake with the real one's records, acting on the fake
  machine's files as the namespace sees them.
- **Faults from the seed:** a power cut at every step of 4.2 and 5.3; a
  full disk mid-walk or mid-apply; the helper killed mid-request; a
  subvolume swapped behind its record; the steward's pins late; a
  second builder over the same store.
- **Invariants** (hearth/docs/design/testing/worlds.md):
  - every published layer's key recomputes from its record, its uuids
    match its subvolume, and its tree digest is the one the world gets
    walking the fake tree on its own;
  - after any crash and a builder's start, no staging, trash or
    temporary file remains, and every record has its layer;
  - no two builders hold the store's lock at once;
  - collection never deletes a pinned layer or an ancestor of one, nor
    runs before the pins are known;
  - no layer holds a nested subvolume, an owner outside the range, or
    the builder's ids;
  - a change set applied to its parent gives the child's tree digest.
- **Step tests** hold the codec to `digests.md`, 1.2's vectors and tree order to
  hand-made cases.
- **Real builds** (hearth/docs/design/testing/real-builds.md), as a regular user on btrfs:
  snapshots keep inodes and change times, the walk reads what the
  builder cannot, layers are deleted with and without the option.

## 10. Open questions

- **Whether `useradd -r -F -K SUB_UID_MIN=262144` makes the default
  range,** or bootstrap keeps `usermod --add-subuids`; depends on
  hearth/docs/design/domain/machine/README.md's bootstrap (`../machine/bootstrap.md`).
- **`FICLONE` in apply,** one more ioctl for skein's ring adapter;
  depends on measuring copies of large files.
- **Collecting target records:** a target no longer built keeps its
  record, and so its last N outputs, for ever; depends on how targets
  are retired (hearth/docs/design/domain/steward.md).
- **Hashing speed:** skein's SHA-256, or a tool, for whole packages
  layers; depends on hearth/docs/design/README.md 7.1's first ask and a measurement.
- **Several `statx` in flight for the walk;** depends on measuring the
  walk on the real tier against skein's file machine, which runs one
  request at a time.
- **Finishing an interrupted publish** (a frozen staging layer whose
  record is written) rather than deleting it; depends on how often power
  cuts land there.
- **The pins' and collection's defaults:** N per target, the free-space
  floor, how long unused resolutions and cache files stay; depends on
  the real tier's and the lab's costs.
- **Inode flags in a change set:** whether no-copy-on-write should
  travel; depends on whether a received journal directory ever matters.
- **A per-directory digest** (a Merkle tree) for cheap comparisons of
  subtrees between machines; depends on peers [M6].
- **Rules of the scan that change:** a layer recorded with older rules
  is scanned again from its tree file, with the helper for content
  predicates, before it is used as an output; depends on hearth/docs/design/domain/build/README.md, 7.3.
- **Trusting a received key [M6]:** whether a receiver ever checks a
  layer by building it; depends on reproducible builds.
- **A reconciled layer's base:** `records.md`, 2 lets it be collected while its
  descendants are kept, while 5.1 holds it as an ancestor under a
  descendant's pin; one of the two is to change.
- **A scan's and a chain's lifecycles** (4, `state.md`, 1.2): `Scan` and `Working` keep
  theirs in flags and `Option`s beside `Pass` and `Next`
  (`skein/docs/foundation/programming-model.md`, 5.4). Default: as they stand,
  until a debt item names them.
