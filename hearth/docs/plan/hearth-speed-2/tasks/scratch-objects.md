+++
id = "scratch-objects"
part = "real-git"
kind = "feature"
needs = ["profile"]
files = ["hearth/testing/hearth-test-repository/src/lib.rs", "hearth/testing/hearth-test-repository/src/objects.rs", "hearth/testing/hearth-test-repository/Cargo.toml", "Cargo.lock", "hearth/tests/builder/tests/real/main.rs"]
effort = "high"
minutes = 60
+++

# A scratch repository is written, and real git reads it

## Goal

`Repository::make` runs `git init`, `git add -A` and `git commit` before a test has checked anything, in about 195 tests of the real-git binary and about 65 repositories of the builder suite. Nothing there tests git's write path. Write the same objects from the kit, so real git runs for the code under test alone.

Measured at this cut, with git 2.47.3 on one machine, in a scratch directory, for ten files of `THIN`'s size: `make` costs 6.25 ms of git's CPU; `next` costs 3.90; `rev-parse` 0.80. The same shape written directly costs git nothing and under 1 ms of its own, in Python with C hashing; unoptimized Rust will cost more. So expect about 1 CPU s from the real-git binary and 0.3 from the builder suite, more on a loaded builder.

Acceptance:

- `Repository::make` writes the repository without spawning git. `next` does too, and `head` reads the ref, each kept if it measures a saving.
- **The ids are git's.** A new test in `hearth/tests/builder/tests/real/main.rs`, `a_written_repository_is_the_one_git_makes`, makes one repository each way (`make_on` and `make`) of `THIN` plus an executable, a symbolic link and names that sort around a directory (`a.b`, `a/c`), compares `git rev-parse HEAD`, runs `git fsck --strict` on the written one, then calls `next` on both with a file changed and the last file of a directory removed, and compares again.
- `make_on` still runs real git, so `a_scratch_repository_without_git_on_the_path_fails_by_name_not_by_skip` stands unchanged.
- Every test of `hearth-builder-world` and `hearth` passes unchanged: `THIN_TEXT`, every golden, `a_tree_git_cannot_read_prints_git_s_tail_and_exits_75`, `end_to_end_render`'s uncommitted edit.
- `hearth-builder-world`'s `real` falls by what `figures.md`'s git split gives (`### Real git`). Say what it fell by.

## Inputs

- `hearth/docs/plan/hearth-speed-2/notes/common.md` and `hearth/docs/plan/hearth-speed-2/notes/repairs.md`
- `hearth/docs/plan/hearth-speed-2/figures.md`: `### Real git` and `## What it found`
- `hearth/docs/design/domain/testing.md`, 2.7 and 9.1: "A scratch repository is written"

## Owns and may change

- **Owns:** `hearth/testing/hearth-test-repository/src/lib.rs`'s `Repository`, its new `src/objects.rs`, and one new test in `hearth/tests/builder/tests/real/main.rs`. `builder-repeats` may change `hex_digest`'s body in `hearth/testing/hearth-test-repository/src/resolution.rs`, so leave that file alone.
- **May add:** `sha1.workspace = true` to the kit's `Cargo.toml`, and the one line that makes in `Cargo.lock`. `sha1` is in the workspace already (`hearth-fake-git` names it). The kit's lints don't change. A `pub use` in `hearth/tests/builder/src/fixtures.rs` if the new test needs a name from the kit.
- **Not `hearth-fake-git`:** its encoders are `pub(crate)`, it holds trees in memory where this walks a work tree, and its `Tree::insert` scans. The kit keeps its own small writer.
- **What stays real git:** `make_on`; `Repository::git`, and `long_line`'s `add -A` and `commit` (`hearth/tests/builder/src/fixtures/reserved.rs:24-25`), checked at this cut to give git's own ids on a repository with no index; `work_tree` and what it commits in `hearth/tests/builder/src/fixtures.rs`, whose point is what git discovers there, and which keeps its template.
- **A test that needs an index, or a real commit:** make it with `make_on` and say which in the report. A narrow `git add <path>` on a written repository commits a tree of that path alone (checked at this cut), and `git status` calls every file deleted. No test does either at this cut (`rg`: the narrow add at `fixtures.rs:293` is on the containing work tree).
- **If `next` or `head` saves nothing measurable,** it keeps real git, and `make` lands alone.
- **If `figures.md` shows git's share of the fixture is under 1% of the family,** see `notes/repairs.md`: no change.

## Steps

The sketch binds the names; `make`, `next` and `head` keep their signatures:

```rust
// hearth/testing/hearth-test-repository/src/objects.rs
//! A scratch repository's objects as `git add -A` and `git commit` would write them
//! (hearth/docs/design/domain/testing.md, 9.1).
pub(crate) fn lay_out(work_tree: &Path);  // .git/HEAD, .git/config, .git/objects, .git/refs/heads
pub(crate) fn commit(work_tree: &Path, message: &str) -> String;  // walks the work tree, moves refs/heads/main, gives the id
fn tree(git: &Path, directory: &Path) -> Option<[u8; 20]>;        // None for a directory with nothing below it
fn object(git: &Path, kind: &str, body: &[u8]) -> [u8; 20];       // sha1, then one loose file
fn stored(raw: &[u8]) -> Vec<u8>;                                 // zlib's stored blocks

// hearth/testing/hearth-test-repository/src/lib.rs
pub fn make(label, files) -> Self;  // empty + lay_out + write + commit("thin")
pub fn next(&self, files, removed) -> String;  // write + removals + commit("next")
pub fn head(&self) -> String;  // reads .git/refs/heads/main
```

What the writer must match, each checked against real git at this cut with a throwaway writer of the same shape, with identical commit ids and a clean `fsck --strict`:

- **`.git/config`:** `[core]` with `repositoryformatversion = 0`, `filemode = true`, `bare = false`, `logallrefupdates = true`. **`.git/HEAD`:** `ref: refs/heads/main\n`.
- **An object:** `<kind> <length>\0<body>`, its SHA-1 the id, at `.git/objects/xx/yyyy...`, loose always, because `remove_object` removes a loose tree.
- **zlib, stored:** `78 01`; blocks of at most 65,535 bytes, each a final-block byte, `LEN` and `NLEN` little-endian; then the Adler-32 big-endian.
- **A tree:** entries `<mode> <name>\0<20 bytes>`, sorted by name with a directory's name taken as `name/`. Modes: `120000` for a link, whose blob is its target; `100755` if the owner's execute bit is set, else `100644`; `40000` for a directory. Read with `symlink_metadata`, so a link isn't followed.
- **The walk** skips the root's `.git` alone, and **a directory with nothing below it enters no tree**, as with `next` removing a directory's last file. An empty root is the empty tree, as `--allow-empty` gives.
- **A commit:** `tree <hex>\n`, then `parent <hex>\n` if `refs/heads/main` exists, then `author hearth <hearth@hearth.invalid> 1700000000 +0000\n`, `committer` the same, then `\n<message>\n`. Share those strings with `git_output`'s environment by constants.

1. The start check. Then `Ir` of `hearth_render_prints_the_thin_repositorys_spec_through_real_git`, and `hearth-builder-world`'s `real` figure, twice.
2. `objects.rs`, `make` and the new test. Break it by hand twice: sort directories without their `/`, then let an empty directory enter its tree. See the test fail for each, and restore. Commit with the saving.
3. `next` and `head`. The same figures. Commit with the saving, or leave them out.
4. **The readers' tests:** `heavy cargo nextest run -p hearth-test-repository -p hearth-builder-world -p hearth`.
5. The end of `notes/common.md`. Push.

## Report

`make`'s saving and `next`'s and `head`'s, the figures on both sides, the machine, any test moved to `make_on` and why, and `## Left`.
