# The store: digests and keys

## 1. Digests and keys

### 1.1 The canonical encoding

Every value that is hashed is one of seven kinds:

```
value          bytes
none           00
false          01
true           02
uint n         03  n as 8 bytes, big-endian
bytes b        04  the length of b as 8 bytes, big-endian; then b
digest d       05  the 32 bytes of d
list [v…]      06  the count as 8 bytes, big-endian; then each v, encoded
map {k: v…}    07  the count as 8 bytes, big-endian; then, for each entry, k encoded as bytes (04, its length, k), then v
```

- **Map keys are byte strings, strictly increasing bytewise;** a
  duplicate is not a map. A record's field names are map keys. The order
  is that of the keys' own bytes, not of their encodings, whose lengths
  come first.
- **An optional field is left out when absent.** A record's map holds the
  fields it has: `gid?` without a gid, `security` with the pocket off, a
  container's packages part without `initramfs`. A field a later
  milestone adds (`[M5]`, `[M6]`) is optional, and absent where a record
  has none of it, so a record that does not use it keeps its digest
  across the release that adds it.
- **`none` is a value, not an absence:** it is for a field that is always
  present and may hold nothing, written `| none` (a layer's `parent`,
  `records.md`, 2).
- **An enumeration is its variant's name,** as `bytes`, in the words the
  design gives it: `file`, `link` and `directory` in a config part's
  tree; `enable` and `disable`; `core`, `data` and `cache`; `none` and
  `generic` for `microcode`, which are names, never the `none` kind. A
  variant that carries fields is the map of its fields with one more,
  `kind`, its name. Booleans are `false` and `true`. The tree digest's
  entries (1.4) keep their kinds as numbers, fixed there.
- **A set is a list,** without duplicates, in its members' order: text
  bytewise, a pair by its first member and then its second, unless the
  part names another order, as a packages part's components do
  (hearth/docs/design/domain/render/spec.md, 1.2).
- **Text is its UTF-8 bytes as given,** with no normalisation and no
  terminator.
- **No signed integers, no floats.** A value that needs one enters as
  text (hearth/docs/design/domain/render/spec.md, section 1).
- **Kinds stay apart:** a digest is not 32 bytes of `bytes`, a boolean
  is not 0 or 1, none is not an empty list. One value has one encoding,
  and a decoder needs no lookahead.
- **Decoding is bounded:** every length and count is checked against a
  limit before anything is set aside, and nesting uses an explicit stack
  (`skein/docs/foundation/programming-model.md`, section 8).
- **The encoder and `H` live in `hearth-lib`,** a crate in the lib role:
  `no_std`, depending on skein-lib alone. A domain depends only on lib
  and its children (`skein/docs/foundation/programming-model.md`, 4.5), and six of hearth's
  domains encode and hash, so every hearth domain may depend on it.
  Decoding is parsing, and stays a protocol machine's:
  `hearth-protocol-tree` (`tree-helper.md`, 3).

### 1.2 H

```
H(tag, v1, …, vn) = SHA-256( encode( list [ bytes "hearth/1", bytes tag, v1, …, vn ] ) )
```

- **The tag says what is hashed,** so two kinds of digest never collide.
- **`hearth/1` versions the encoding and hearth's own constants.** A
  change to 1.1 or 1.4, or to a fixed choice of how hearth builds (apt's
  and dpkg's fixed options, `force-confold` among them; the cleanup a
  step runs: hearth/docs/design/domain/build/packages.md), changes it, and every key with it: a
  rebuild, never a migration.
- **A file's content digest is plain SHA-256 of its bytes,** without
  `H`, so it matches `sha256sum`, the `SHA256` of Debian's `Packages`
  and hearth/docs/design/protocol/format/README.md's software digests (hearth/docs/design/protocol/format/README.md, section 8).

| Tag | Values | Defined in |
|---|---|---|
| `empty` | none | 1.3 |
| `packages`, `software`, `config` | the key below, then a part's digest | 1.3 |
| `closure` | a resolution's closure | `records.md`, 4 |
| `resolution` | a resolution's inputs | hearth/docs/design/domain/render/spec.md, section 2 |
| `part:<name>`, `spec` | a part's record; a spec's digests | hearth/docs/design/domain/render/spec.md, 1.5 |
| `tree`, `changes` | a tree's entries; two trees and their changes | 1.4, 1.5 |
| `target`, `state` | a name | `records.md`, 6 |
| `artifact`, `inputs`, `command`, `outputs` | an artifact's | hearth/docs/design/domain/build/artifacts.md, section 3 |
| `environment` | a resolution key, then the packages, software and config part digests | hearth/docs/design/domain/build/artifacts.md, 3.1 |

Test vectors:

```
encode(list [bytes "hearth/1", bytes "empty"])
  = 06 0000000000000002 04 0000000000000008 6865617274682f31 04 0000000000000005 656d707479
H("empty")
  = 8f43cea6a4bf9d3aec62c882bdafeab30cc6d71acc9b0b984b7f2fd8f70975d3
H("packages", digest 00…00, digest 01…01)
  = 22d5156ffefadb1f0df4eb2c4d405b2c8937215e044486b06b1725e3506b238d
H("tree", list [ list [bytes "", 1, 0o755, 0, 0, map {}, none] ])        the empty layer's tree
  = 6022d1fbc6ad38d08d2ae02cefc21e7e6af5a35e6be9d115f0ba267fdb0d005a
encode(map { "a": uint 1 })                                               a key as bytes
  = 07 0000000000000001 04 0000000000000001 61 03 0000000000000001
H("test", map { "adm": map { "gid": uint 4 }, "staff": map {} })          staff's gid absent
  = 8291ea66351528fd3979b84aec0397019d25dae6f8d8aa9a0318331d2fd164f5
H("test", map { "kind": bytes "file", "mode": uint 0o644, "owner": bytes "root" })   a variant by name
  = 72653b9931df15f292b4f5f17e091263cd9fb738b0f2c06cce1cd74242b5b211
```

`test` is the vectors' tag alone; no record of hearth's uses it. The
last three pin 1.1's rules for keys, absent fields and enumerations:
with the key's `04` left out, the second would be `3f4edef7…`, and with
`gid: none` for staff, `ab3793df…`.

### 1.3 Keys

```
empty key          = H("empty")
packages key       = H("packages", resolution digest, packages part digest)
software key       = H("software", packages key, software part digest [, heads])         heads [M6], below
config key         = H("config",   software key, config part digest)       the output's name
resolution digest  = H("closure",  closure)                               6.4
```

- **Each key covers everything beneath it.** A change to a part changes
  its layer's key and every key above, nothing below.
- **A head artifact's resolved key enters the software key [M6].** When
  the software part names an artifact by its head form (a recipe and its
  listing's digest), the plan resolves it to an artifact key before the
  software layer (hearth/docs/design/domain/build/artifacts.md, 7.2), and `heads` is a map from each
  such entry's path to that key, appended as a third value. It is absent
  when the part has no head entry, so keys without one are as above.
  It plays the closure digest's part in the packages key: the part holds
  what the revision says, the key what was resolved (hearth/docs/design/domain/render/spec.md, 1.5).
- **A resolution's key and its digest differ.** The key, over its
  inputs (hearth/docs/design/domain/render/spec.md, section 2), names its record. The digest, over the
  closure it gave, enters the packages key, so the packages layer is
  named by what is installed: a resolution made again (its record
  collected, a newer apt) that gives another closure makes another
  layer.
- **Part digests are render's** (hearth/docs/design/domain/render/spec.md, 1.5), made with this `H`.
  The packages part carries what the packages step reads besides the
  closure: sources, the request, the variant, preseeds, the initramfs's
  modules, and `install_recommends`, its one apt option (hearth/docs/design/domain/render/spec.md,
  1.2). hearth's fixed apt and dpkg options are constants of
  `hearth/1` (1.2), not fields of the part.
- **The empty layer** is every packages layer's parent (`README.md`, 4.1).
- **Nothing else enters a key:** not the builder, its subordinate base,
  its store's path, the time, the target's name, the revision, nor the
  builder's tools' versions, which the record keeps (`records.md`, 2). A step is
  given nothing its key does not cover (hearth/docs/design/domain/build/sandboxes.md, 6).
- **A key names what was asked for; the tree digest records what was
  built.** Until builds are reproducible, two builds of one key differ,
  and each records its own (hearth/docs/design/domain/build/reproducible.md).

### 1.4 The tree digest

**Covered,** per entry, as the tree helper reads it in the namespace:

- the path, relative to the layer's root;
- the kind: directory, file, symbolic link, hard link, character or
  block device, fifo, socket;
- the permission bits with setuid, setgid and sticky (`mode & 0o7777`);
- owner and group as namespace ids, 0 to 65535;
- extended attributes, by name and value, of four families:
  `security.capability` as the namespace reads it (version 2: a set
  made in the namespace is stored as version 3 with the base as root id
  and reads back so); `system.posix_acl_access` and
  `system.posix_acl_default`, their ids translated by the kernel;
  `user.*`;
- for a file, its size and content digest; for a link, its target's
  bytes; for a device, major and minor; for a hard link, the path it
  shares an inode with.

**Left out:**

- every time: mmdebstrap clamps none in a directory, and reproducible
  builds are deferred;
- inode numbers, link counts, the device, the generation, holes;
- inode flags: of those a user may set, only no-copy-on-write (`C`,
  which systemd's tmpfiles sets on `/var/log/journal`), a hint;
- btrfs properties (`btrfs.compression`) and the subvolume's identity;
- `trusted.*` and every other `security.*`: setting them needs privilege
  in the initial namespace, so no layer built in the builder's holds
  any, and a change set carrying one is refused.

```
entry    = list [ path: bytes, kind: uint, mode: uint, uid: uint, gid: uint, xattrs: map, payload ]

kind 1   directory      payload none
kind 2   file           payload list [ size: uint, content: digest ]
kind 3   symbolic link  payload bytes: the target, as stored
kind 4   hard link      payload bytes: the path of the first entry with its inode
kind 5   char device    payload list [ major: uint, minor: uint ]
kind 6   block device   payload list [ major: uint, minor: uint ]
kind 7   fifo           payload none
kind 8   socket         payload none

tree digest = H("tree", list [ entry… ])        in tree order
```

- **A path** joins its components with `/`, with no leading or trailing
  `/`; the root's is empty. A component is not empty, `.` or `..`, and
  holds no `/` or NUL. At most 4096 bytes.
- **Tree order** compares paths component by component, each bytewise:
  a directory comes before everything beneath it, and its subtree before
  its next sibling (`a`, `a/b`, `a-b`). It is a depth-first walk with
  sorted names, so walks and tree files stream.
- **Hard links:** an inode's first path in tree order is its file; each
  later path is a kind 4 entry with the same mode, owners and
  attributes. Debian ships some (`perl-base`'s `/usr/bin/perl`).
- **Refused by the walk:** a root that is not a directory, 0755, owners
  0; an owner past 65535, which catches a file the builder wrote (its
  ids show as 65536 to the tree helper, `tree-helper.md`, 2); a nested subvolume (an
  entry on another device); a bound passed (`tree-helper.md`, 6). systemd-container's
  postinst runs `systemd-tmpfiles --create`, whose `Q` lines make
  `/var/lib/machines` and `/var/lib/portables` subvolumes on btrfs, so
  the packages step makes both directories first (hearth/docs/design/domain/build/packages.md, 5).
- **A subtree's digest** is the same formula over its entries, paths
  made relative to it, computed from the tree file without reading the
  layer. When the kernel is unchanged is hearth/docs/design/domain/slots/README.md's to decide (`records.md`, 1),
  from digests the tree file records (`records.md`, 3).
- **Its record format** is the tree file (`records.md`, 3).

### 1.5 The change set

```
change            = list [ op: uint, path: bytes, entry or none ]
  op 1  put       the child's entry at path, whatever the parent had there
  op 2  delete    the parent had path; the child has nothing there
change set digest = H("changes", parent tree digest, child tree digest, list [ change… ])
```

- **In tree order, a path at most once.** A deleted directory lists
  every path beneath it; a put that changes a path's kind replaces what
  was there.
- **Of the two trees alone:** a path whose entry, read again (`README.md`, 4.3),
  equals the parent's is no change, so the change set and its digest
  depend on the parent's tree and the child's, never on which entries a
  walk read.
- **With no parent** (`hash`, `tree-helper.md`, 4), the parent tree digest is `none`
  and every entry is a put, as the end item's `parent_tree` (`records.md`, 3).
- **Never trusted alone:** applied to the parent and walked, it must
  give the child's tree digest (`tree-helper.md`, 5, `README.md`, 8.2).
- **It is what a layer changed,** for peers (`README.md`, 8.1) and for people and
  agents: the files a config change touched, from the tree file.
