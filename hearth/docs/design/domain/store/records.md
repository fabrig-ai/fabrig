# The store's records

## 1. The frame

```
magic      8 bytes   "hearth\0r"
kind       u16 BE    1 store, 2 layer, 3 tree, 4 resolution, 5 cache file, 6 target, 7 state, 8 ring [M5], 9 artifact [M6]
version    u16 BE    the kind's format, from 1
reserved   u32       zero
length     u64 BE    the body's length, checked against the kind's bound first
body       a map of named fields (3.1); for a tree file, a sequence of items (6.3)
```

`hearth-protocol-tree`'s machines decode records; the domain gets typed
values.

**Bounds.** Each kind's bound on its body, and the limits within it, are
configured (`skein/docs/foundation/programming-model.md`, section 7), with these defaults:

| Kind | Body | Within it |
|---|---|---|
| 1 store | 4 KiB | — |
| 2 layer | 1 MiB | 4,096 digests in `placed`, 256 `tools`, 64 steps in `costs`; a tool's name or version of 256 bytes |
| 3 tree | 1 GiB | an item of 4,718,592 bytes; 8 kernels in `boot`; an attribute's name, a map key in an entry, of 255 bytes (`tree-helper.md`, 6) |
| 4 resolution | 4 MiB | 8,192 packages in a closure, 8 source lines, 16 indices a line; a URI, a path or a filename of 4,096 bytes; a suite, a component, a name, a version or an architecture of 256 |
| 5 cache file | 8 KiB | a URI or a path of 4,096 bytes; a git object id of 64 |
| 6 target | 8 KiB | a name of 256 bytes, 16 builds, a revision of 64 |

A body past its bound, or a field past its limit, is a record of a
version the builder cannot read (8). A resolution whose closure would
pass 8,192 packages is an error of kind resolution (hearth/docs/design/domain/render/errors.md, 2),
never a record; so is one whose record would pass its bound or a
field's limit, which the store declines to write: it depends on the
inputs alone, a configuration error. The other kinds' bounds are set
with the milestone that writes them: M2's are above, and kinds 7 to 9
come with M3, M5 and M6. The tree helper's messages are not records
and have no kind: a length and a map, bounded by `tree-helper.md`, 6 (`tree-helper.md`, 3).

- **The decoders' own bounds,** which no valid record reaches, hold
  for every kind: nesting 8 deep, a field's name of 64 bytes, a list or
  a map of 8,256 items. A tree file's decoder takes map keys of 255
  bytes, since an entry's attributes are a map keyed by their names
  (`README.md`, 3.4).
- **Field shapes,** for every kind: a count is a `u64`, a cost in time a
  `u64` of nanoseconds, a uuid 16 bytes.
- **A record that is not its version's** is one the builder cannot read
  (8): a reserved field that is not zero, a field its kind's version
  does not name, a field it requires missing, a value of another kind.
  A field a later release adds comes with a new version of its kind.

## 2. A layer's record

`layers/<key>.record`, small, read at startup:

```
key          digest                 the layer's key
kind         bytes                  empty | packages | software | config
parent       digest | none          the parent's key
inputs       map                    packages: { part, resolution }; software, config: { part }; empty: { }
tree         digest                 the tree digest (3.4)
changes      digest                 the change set's digest (3.5)
tree_file    list [ digest, uint ]  <key>.tree's SHA-256 and size
subvolume    map                    { parent_uuid: uuid | none, uuid }, as read after the snapshot
counts       map                    { bytes, deletes, entries, files, hashed, puts }
scan         digest                 the scan's rules it passed (../build/README.md, 7.3); absent for the empty layer
placed       list [ digest ]        software only: each download's SHA-256 it placed, and each artifact's tree digest [M6]
tools        list                   list [ name, version ], bytes each, by name: mmdebstrap, apt, dpkg, the target's tools it ran
costs        map                    by step name: { cpu, fetched, hashed, memory, wall, written }
made         map                    { at: wall time in ns, by: built | received [M6], from [M6]; packages only: how: scratch | reconcile, base: key | none, depth: uint, line: wall time in ns }
```

`line` is the `at` of the last scratch build in the layer's line of
reconciles: its own `at` for a scratch build, its base's `line` for a
reconcile. It is recorded, not derived from the base chain, since a
base may be collected while its descendants are kept (hearth/docs/design/domain/build/README.md, 4.2).

`counts`: `entries` and `files` in the tree, `bytes` the files' sizes
summed, `puts` and `deletes` the change set's, `hashed` the bytes the
walk read. `costs`: per step (its name a map key of at most 64 bytes,
as `fetch`, `mmdebstrap`, `walk`), `wall` and `cpu` in ns, `memory` the peak in
bytes, and the bytes `fetched`, `written` and `hashed`. The empty
layer's `tools` and `costs` are empty; `placed` is present, maybe empty,
in a software layer's record alone, and `how`, `base`, `depth` and
`line` in a packages layer's alone.

The key recomputes from `kind`, `parent` and `inputs`, and the startup
scan checks it. `placed` is what collection keeps in the cache for a
kept software layer (`README.md`, 5.5), since its part digest names no download. The
activator may read records to check what it places (hearth/docs/design/domain/activator/README.md,
section 5).

## 3. A layer's tree file

`layers/<key>.tree`, large, read to build a child, serve a peer or
check a layer. After the header, items, each `tag: u8`, `length: u32
BE`, then a value:

```
tag 1  entry    list [ entry (3.4), local: list [ inode, ctime s, ctime ns, mtime s, mtime ns ] ]
tag 2  change   list [ op, path ]       a put's entry is the entry item of its path
tag 3  end      map { key, parent_tree, tree, changes, counts, boot }

boot             list, per kernel the layer holds; empty for a container:
  version        the kernel's version, as in usr/lib/modules/<version>
  kernel         the content digest of boot/vmlinuz-<version>
  modules        the subtree digest of usr/lib/modules/<version> (3.4)
  initramfs      the digest of the initramfs's inputs, not of its bytes (../build/packages.md, 8)
```

- **In tree order;** a change item follows its path's entry item, or
  stands alone for a delete. Both digests are computed from the items,
  and the end item must agree.
- **`boot` is the packages layer's,** computed by its walk (hearth/docs/design/domain/build/packages.md,
  8): the `walk` request carries, per kernel package of the
  resolution's closure, its name, its version and the kernel version it
  installs, with tiny-initramfs-core's version and the microcode
  packages; the helper adds the digests the walk computed and writes the
  end item. A kernel found in the layer that the request does not name
  is a finding. The layers above copy it unchanged from their parent's
  end item, which may not touch
  the kernel, its modules or the initramfs's configuration (hearth/docs/design/domain/build/README.md,
  7.3). hearth/docs/design/domain/slots/README.md (1) reads it to decide when the kernel is unchanged.
- **About 150 bytes an entry,** 15 MB for 100,000 entries (an
  estimate). A packages layer's change set is every entry; a config
  layer's, a few.

## 4. A resolution's record

`resolutions/<key>.record`. hearth/docs/design/domain/render/spec.md (section 2) makes resolutions;
the store keeps them. What the packages layer needs from one:

```
key        digest   H("resolution", sources, request, variant, apt), over its inputs (../render/spec.md, 2.1)
inputs     map      { sources, request, variant, apt }: the key's values, as the packages part holds them
                      (../render/spec.md, 1.2); the architecture is the sources'
sources    list     per source line: list [ uri, suite, components,
                      indices: list [ path, size, sha256 ] ]      kept in cache/: its InRelease first, as
                                                                  [ "InRelease", size, sha256 ], then each Packages
                                                                  file, decompressed, at the path and with the size
                                                                  and SHA-256 of its InRelease entry; each path
                                                                  relative to the suite's directory, dists/<suite>/
closure    list     per package, by name then architecture:
                      list [ name, architecture, version, source: uint, filename, size, sha256 ]
digest     digest   H("closure", list [ list [ name, architecture, version, sha256 ] … ])
made       map      { at, apt, mmdebstrap, costs: { fetched, took } }: `at`, wall time in ns when
                      recorded; apt's and mmdebstrap's versions, empty in M1 (../render/spec.md, 2.4);
                      `fetched`, the bytes apt reports fetching; `took`, the run's duration in ns
```

- **The key recomputes from `inputs`,** as a layer's from its record
  (2), **and the digest from `closure`;** the startup scan checks
  both, since the scan reads each resolution's record whole anyway, and
  a packages key built on a digest its closure does not give would name
  bytes it does not install. A record whose key or digest does not
  recompute is deleted.
- **Every byte the packages step installs is named by SHA-256 here:**
  each `.deb` by its `Packages` entry, each `Packages` file by its
  `InRelease` entry, each `InRelease` by the digest it had when apt
  checked its signature.
- **The indices are kept, not fetched again.** Debian publishes no
  uncompressed `Packages` file, so the digest an `InRelease` gives for
  one names nothing a mirror serves. Resolution moves each `InRelease`
  and each decompressed `Packages` file, as apt verified them, from its
  scratch directory, `staging/<resolution key>/`, into `cache/<sha256>`
  before the directory is removed: hashed as it is read for its stanzas,
  then renamed, with no copy (hearth/docs/design/domain/render/spec.md, 2.3). The fetch then gets only `.deb`s
  (hearth/docs/design/domain/build/packages.md, 1), and the local repository serves the kept indices
  (hearth/docs/design/domain/build/packages.md, 3).
- **The digest covers what is installed;** where it came from does not
  enter.

## 5. A cache file's record

`cache/<sha256>.record`, kind 5, version 1:

```
sha256     digest   the file's SHA-256: its name
size       uint     its length in bytes
origin     map      where it first came from, one of:
                      { kind: "fetch", uri }                  fetched (../build/packages.md, 1)
                      { kind: "index", resolution: digest }   an index a resolution kept (6.4), by its key
                      { kind: "blob", oid: bytes }            a static file render streamed from git, by its object
                                                              id as git prints it, lowercase hex
                      { kind: "rendered", path: bytes }       a file render produced, by its template's path in the revision
at         uint     when it entered the cache: wall time in ns
took       uint     how long it took to enter, in ns: the fetch, the stream or the write
```

It is provenance and cost; whether a file is kept depends on who names
it (`README.md`, 5.5). A file that enters again, from another origin, keeps its first
record.

## 6. The store, targets and state

- **`store.record`,** kind 1, version 1: `layout`, the layout's version,
  1; `uid`, the builder's; `range`, `{ base, count }`, the subordinate
  range its layers are owned by; `made`, wall time in ns. A store without
  layers (`README.md`, 2.4) has no `range`; the store records it, by rename, at the
  start of its first layer [M2], before the empty layer's subvolume is
  made. Once it is recorded, a builder configured with
  another base refuses to start: every layer on disk is the recorded
  range's. So does a builder of another uid, which would write files the
  recorded one cannot replace (`sudo hearth render --store` on a
  developer's store): the store is not opened, and the client's startup
  error (`README.md`, 6.1) names both uids.
- **`targets/<H("target", name)>.record`:** the name and the last N
  builds (`README.md`, 5.1), newest first, each `[revision, output key, at]`,
  rewritten when a build for the target completes. Target records are
  never collected in M2 (`README.md`, section 10).
- **`state/`:** the steward's records and rings (`README.md`, 6.3).

## 7. An artifact's record [M6]

`artifacts/<key>.record`, beside the artifact's subvolume and its tree
file, in the same frame (kind 9): its tree digest, environment, sources,
inputs, command, outputs, cache, how it was made and its costs, as
hearth/docs/design/domain/build/artifacts.md (`README.md`, 4.3) lists them. Its key is hearth/docs/design/domain/build/artifacts.md's (`README.md`, 3.1).

## 8. Written by rename; versioned

- **Atomic:** a record is replaced whole by skein's `Store` (
  `skein/docs/design/runtime/io/files.md`): its temporary, `<name>.skein-<16 hex>`,
  written beside it, synced, renamed over `<name>`, the directory
  synced. A reader sees the old record or the new. A tree file [M2] is
  written the same way by the builder itself, as `<key>.tree.tmp`.
- **Ordered:** the layer's files synced (`SyncFilesystem`, at the end
  of the tree helper's freeze) and its tree file in place before its
  record, a record before the publishing rename
  (`README.md`, 4.2), an unrecording before the subvolume is touched (`README.md`, 5.3). The
  startup scan repairs any prefix (`README.md`, 4.4).
- **A cache file is durable before its record.** It is written to
  `cache/<random>.tmp` (`README.md`, 2.1), hashed as it is written, synced, and
  renamed to `cache/<sha256>`, in its own directory, so a reader never
  sees half a file under a digest; then its record is written. Several
  files may be made durable with one `SyncFilesystem` before their
  records [M2], made by the tree helper, never on the builder's loop.
  A file whose name is already present is not written again: its
  temporary is removed. An index a resolution keeps is synced where apt
  left it, renamed from its scratch directory into `cache/<sha256>`
  (`skein/docs/design/runtime/io/files.md`: a rename between two directories), and `cache/`
  synced, before its record (hearth/docs/design/domain/render/spec.md, 2.3). A cut anywhere leaves a
  temporary, or a file with no record, which the startup scan deletes
  (`README.md`, 4.4).
- **Versioned per kind, across releases** (hearth/docs/design/domain/activator/README.md, 6.2's rule): a
  release reads its own version of each kind and the previous one, and
  writes only versions the previous release can read. After a fall back,
  the blessed slot's older builder and steward read what the newer ones
  wrote.
- **An unknown version never stops the builder.** A layer, tree file,
  resolution, cache file or target record of a version it cannot read is
  treated as absent: the layer deleted (`README.md`, 5.3), the rest made again, which
  costs time and breaks no slot, since slots are copies. A
  `store.record` it cannot read is written again, and every layer it
  holds is treated as unrecorded; so is one whose `layout` is not 1.
  A resolution it cannot read stays in place, unindexed, until a
  resolution of its key is made again over it. `state/` records follow
  `README.md`, 6.3's table.
