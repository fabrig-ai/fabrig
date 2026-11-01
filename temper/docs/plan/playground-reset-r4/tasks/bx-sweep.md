+++
id = "bx-sweep"
part = "box-groups"
kind = "feature"
needs = ["box-pipe"]
files = ["temper/crates/temper-box/**"]
effort = "high"
minutes = 60
+++

## Goal

`remove N` also removes the forge's directory (contracts, section 3: "the
entry's `directory`; keep the cache"), so a removed owned Forgejo or
playground forge leaves nothing beneath the data directory but what other
forges own, and a removal cut by the box's death ends whole when `remove N`
is sent again.

Acceptance:

- At `remove N`, after the group's steps are closed and before
  `box/started/N` and `box/groups/N` are removed, the box takes the
  directory of forge N: the `setup.directory` member of `box/groups/N` when
  the file exists, else the one `Settings.directories` gives for N, else
  none. That directory is moved at once into `box/trash/N` (made 0700 as
  `box/started` is), and `box/trash` is swept empty before `removed N`.
- A directory already absent is removed. A directory outside the data
  directory, or one whose path `..` or a link takes outside it, is not
  touched: named on stderr (`box: forge N: directory outside data: PATH`),
  and the removal goes on to `removed N`.
- At the box's start, a `box/trash` left by a cut removal is swept, beside
  the steps (the steps do not wait for it); a `remove` arriving while that
  sweep runs waits for it (the ask in flight answers once both are done).
- Nothing outside the entry's directory and the box's own two files is
  removed: the cache directory is never touched (the box's root is the data
  directory, so the cache is outside its reach), nor `box/logs`, nor another
  forge's directory.
- Every temper-box test passes; the new ones below pass.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 3).
- `temper/docs/design/draft/playground-reset.md`, section 4 ("Its box group
  stopped and removed") and 6.4 (the owned Forgejo set up afresh, its
  binaries from the cache).
- `temper/docs/design/protocol/setup.md`, 7.1 (the record's directories and
  modes).
- `skein/crates/skein-io/src/kernel.rs`, the module documentation on files
  beneath a root and the `Rename`, `Remove`, `List` operations.
- The model for a sweep with no stack of open directories:
  `temper/crates/temper-worker-protocol-git/src/files.rs`, `Sweep` (a trash
  directory, its first entry removed, or its child entries moved up into
  the trash until it is empty).
- box-pipe's report (`## Interfaces`) and its code: `temper/crates/temper-box/src/groups.rs`
  (the group file's read and the removal's file steps), `host.rs`.

## Owns and may change

- Owns `temper/crates/temper-box/**`.
- Neighbour, `temper/crates/temper/src/box_command.rs`: the `Settings`
  literal's new field, filled from the checked setup:
  `directories` = each `checked.setup.entries` entry of mode owned or fake
  with its `directory` (absolute, as the setup check resolves it), and
  nothing else.
- Options:
  - Absolute directories: `Settings.data` is the box's root's absolute
    path; a directory that starts with it and a `/` is the rest beneath the
    root (default). If the setup entries' directories arrive relative, take
    them as beneath the root.
  - The sweep's bound: one operation in flight, as `records.rs` keeps;
    `List` pages sized as the worker's (`LONGEST_NAME`); no limit on the
    entries' count. If a limit is needed for the names' buffer, add it to
    `temper_box::limits::Limits` with a default and say so.
  - If moving into `box/trash/N` fails with `Exists` (a cut removal of N
    left it), sweep the trash first, then move again (default); never
    `no_replace: false` over a directory.

## Steps

1. `host::Settings` gains

   ```rust
   /// Each document group's forge and its directory, absolute, removed at
   /// its `remove` when its group file names none (contracts, 3).
   pub directories: Box<[(u16, Box<[u8]>)]>,
   ```

2. The group file's `setup.directory` is read beside `playground` by the
   reader box-pipe wrote (one more member; the rest stays the steps'). A
   relative directory is beneath the data directory, as the setup reader
   resolves it (`temper-forge-setup`'s `Setup::read`).
3. A new `src/sweep.rs`: the move into `box/trash/N` (open the directory's
   parent beneath the root, `Rename` its last name into the opened trash),
   then the sweep of `box/trash` as the worker's `Sweep` does it: open the
   trash, list one page; remove a file entry; for a directory entry, try
   `Remove { directory: true }`, and on `NotEmpty` open it, move its entries
   up into the trash under fresh names (`N-<serial>`, the serial the box's
   own), and remove it once empty; until the trash lists nothing. Its
   failures are named on stderr as `records.rs`'s diagnostics are
   (`box: sweep box/trash/NAME: <why>`), and leave the removal answering
   `failed N files not removed`, so a `remove` sent again resumes it.
4. The removal's order in the host: steps closed (box-pipe), file read,
   directory moved, `box/started/N` removed, `box/groups/N` removed, trash
   swept, `removed N`. Each step after a cut finds its part done or absent.
5. Tests, `src/tests/host.rs` on the simulated kernel, whose root holds the
   files:
   - `remove_moves_the_forges_directory_to_the_trash_and_sweeps_it_whole`:
     a group file for 7 naming `forge-7`, `forge-7/forgejo/repositories/a/b/c`
     files and a `forge-8/` beside it; after `removed 7`, no `forge-7`, no
     `box/trash` entry, `forge-8` and `box/logs` intact.
   - `a_document_groups_directory_comes_from_the_settings`: no file, a
     `Settings.directories` entry for 3, its directory gone at `removed 3`.
   - `a_directory_outside_data_is_named_and_left`.
   - `a_trash_left_by_a_cut_is_swept_at_the_next_start_and_remove_again_answers`:
     the root starts with `box/trash/7/...` and `box/groups/7`; the box
     starts its steps, sweeps, and `remove 7` answers `removed 7`.
   - `a_sweep_error_fails_the_remove_and_a_second_remove_finishes_it`.
6. `cargo fmt`; `heavy cargo clippy -p temper-box --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-box`; `heavy cargo check
   -p temper --tests`. Commit, push.

## Report

Fab's sections; under `## Interfaces`: `host::Settings::directories`, any
`Limits` field, the stderr words, and the group file members the box now
reads (`playground`, `setup.directory`).
