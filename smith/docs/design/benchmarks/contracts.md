# Benchmarks and smith's slow tests: contracts

2026-11-23. The shapes that the changes of one goal share and no other
area reads (`temper/docs/design/draft/design-economy.md`, 4.3): the goal
that moves smith's benchmark, store and setup checks to the gate
(`temper/docs/design/draft/slow-tests-down.md`, 5.2 and 5.3). They bind
its changes as any delivered design section does. Its last change deletes
this file and its status; the lasting documents don't cite it.

## 1. The fake machine's read-back

`skein-fake-machine`'s `Machine` gains two read-only observers, for a
test reading back what it laid or what the code under test left there,
without the simulator (`skein/docs/design/testing/README.md`, 4):

- `Machine::node(&self, handle: Opened) -> Node`: the identity of the
  node a handle names. `Node` is a `Copy + Eq + Ord + Hash + Debug`
  value, exported beside `Facts`: equal for two handles on one node,
  whatever name or hard link opened it and across a rename; different
  for a node made anew, even with the same name, bytes and mode; the
  same across `crash` for a node the crash keeps. It promises no inode
  number.
- `Machine::link_target(&self, handle: Opened) -> Option<Vec<u8>>`: for
  a handle opened on a symbolic link without following it
  (`How::PathNoFollow`), the link's target bytes as laid or made;
  `None` for any other node.

Neither changes the machine's state, its handles, or any answer its face
gives the simulator; nothing in the simulator's vocabulary changes.

## 2. The harness's files face

`smith_bench::files` (`smith/benchmarks/src/files.rs`) is the one face
through which the harness's file reads and writes reach a filesystem
wherever a step test drives them (`README.md`, 13).

- **`pub trait Files`**, object-safe and used as `&dyn Files`. Each
  operation is named for its `std::fs` counterpart and answers as it
  does, with `std::io::Error` as its error, so that the native path keeps
  its diagnostics' words and kinds:
  - reads: an entry's facts without following (`symlink_metadata`) and
    following (`metadata`); a canonical path (`canonicalize`); a
    directory's children as paths (`read_dir`; native's order, which
    callers that need one sort, as they do today); a file's bytes whole
    (`read`) and at most N bytes from its start (a bounded read, so a
    caller sees N + 1 to know a file is over N); a regular file's SHA-256
    over its bytes, streamed natively in 8 KiB chunks as
    `arm::binary_digest` streams today;
  - writes: a directory made with a mode, alone (`DirBuilder::create`) or
    with its parents (`recursive(true)`); a file made new with a mode,
    written and synced (`OpenOptions::create_new`, `write_all`,
    `sync_all`); a file's bytes replaced (`fs::write`); a mode set
    (`set_permissions`); a file removed (`remove_file`); a tree removed
    (`remove_dir_all`); a rename (`rename`); a file copied (`copy`).
- **`pub struct Facts { pub kind: Kind, pub mode: u32, pub size: u64, pub owner: u32 }`**,
  `mode` the permission bits (`& 0o7777`), `owner` the user id; and
  **`pub enum Kind { Directory, File, Link, Other }`**, `Other` a FIFO,
  device or socket.
- **`pub struct Native`**, `impl Files` with the `std::fs` calls the
  harness makes today, in today's order.
- **A function a moved test drives takes the face** as its first
  parameter, `files: &dyn Files`, in a form named with the crate's
  `_with` suffix (`isolate::working_directory_with`,
  `Guard::write_path_with`); where a `_with` form exists, it takes the
  face in place of its closures or traits. Today's public form keeps its
  signature and passes `&Native`, so the harness's commands and
  `smith-bench::real` are unchanged.
- **The guard resolves through the face:** `Guard` gains `new_with` and
  `write_path_with`, each taking `files: &dyn Files`, so that a
  destination's existing ancestor and the protected paths are resolved
  in the tree a test laid.
- **One face:** `check::{SeedEntry, SeedFiles, SeedKind}` retire into it;
  `seed_digest_with` takes `&dyn Files`. `batch::DispatchFiles` stays as
  it is: it is settlement's face, not a filesystem.
- **A change needing an operation the face lacks** adds it to `Files`,
  `Native` and the tree (3) together, named for its `std::fs`
  counterpart, and says so in its report.

## 3. The test tree

`smith_bench_fixtures::files::Tree`
(`smith/testing/smith-bench-fixtures/src/files.rs`) implements `Files`
over a `skein_fake_machine::Machine`:

- **Laid from `skein_fake_machine::Item`s** at absolute paths, the
  machine's root standing for `/`: a test lays `/home/u/.codex/sessions`
  as `Item::directory(b"home/u/.codex/sessions")`, with its mode and
  owner as `Item` gives them.
- **Paths answer as the host's do:** a canonical path is resolved
  component by component, each link followed through
  `Machine::link_target` (an absolute target from `/`, a relative one
  from the link's directory), `..` taken after the component before it
  resolves, at most 40 links before an error of the kind native gives a
  loop. Errors carry the `io::ErrorKind` native gives for the same tree
  (`NotFound`, `NotADirectory`, `IsADirectory`, `PermissionDenied`,
  `AlreadyExists`, `DirectoryNotEmpty`).
- **What a test reads and scripts:** its machine, to read back kinds,
  modes, bytes, nodes and link targets; every write the face made, in
  order, with its path; and a refusal it lays for one operation at one
  path, answered as an error of a given kind, so that a test shows an
  unreadable file or a failed write without a real one.
- **Reads it never wants:** a test marks a path not to be read (a login,
  a credential source); each read of it is recorded, and the test
  asserts there was none.
