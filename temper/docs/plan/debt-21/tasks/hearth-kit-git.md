+++
id = "hearth-kit-git"
part = "hearth-kits"
kind = "migration"
needs = ["hearth-kits-lay"]
files = ["hearth/testing/hearth-test-git/**", "hearth/tests/git/src/expect.rs", "hearth/tests/git/src/replay.rs", "hearth/tests/git/src/hunks.rs", "hearth/tests/git/src/world.rs", "hearth/tests/git/src/lib.rs", "hearth/tests/git/Cargo.toml", "hearth/tests/conformance/src/fixtures.rs", "hearth/tests/conformance/tests/conformance_git.rs", "hearth/tests/conformance/Cargo.toml", "Cargo.toml", "Cargo.lock"]
effort = "medium"
minutes = 60
+++

# hearth: the conformance world stops depending on the git world

## Goal

The conformance world uses the git world's expected answers and replay. These move to `hearth-test-git`, about 625 lines, and the edge goes.

Acceptance:

- `hearth/testing/hearth-test-git` holds what the note's table gives it, and depends on no world.
- `hearth/tests/git/src/lib.rs` and `world.rs` keep `pub use` at the old paths, so that no file under `hearth/tests/git/tests/` changes.
- `hearth/tests/conformance/src/fixtures.rs` (lines 6-9 and 250) and `tests/conformance_git.rs` (line 2) name the kit. `hearth/tests/conformance/Cargo.toml` has the kit and no `hearth-git-world`.
- The root `Cargo.toml`'s `hearth-git-world` workspace-dependency line goes, if no manifest still uses it by `.workspace`.
- The item's **Measure** falls by one.
- `heavy cargo nextest run -p hearth-git-world`, the same under `--profile fuzzy`, and `heavy cargo check -p hearth-conformance-world --tests` pass, with the same tests and no assertion changed. Conformance itself is outside the gate; if the machine you're on can run `conformance_git` through its `conformance-hearth` profile, run it and report; if not, say so.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/hearth-kits.md`, binding
- `hearth/docs/development/debt/D009-worlds-as-kits.md`
- `hearth/docs/design/domain/testing.md`, 2.9; `skein/docs/design/testing.md`, 6
- The shape to copy: `git show c721602b44` created `hearth-test-fixtures`

## Owns and may change

- **Owns:** the files in `files`.
- **`world.rs` was just changed** by hearth's memory-model work (`git log -3 -- hearth/tests/git/src/world.rs`), which may go on: move what's there at your base, don't restate it, and leave every queue form as it stands.
- **`.config/fixture-readers.toml`** names `hearth-git-world` as a reader of `hearth/tests/git/src/fixtures` and of recordings, which stay. If a moved file was named as read by a package that no longer reads it, take that reader off and say so.

## Steps

1. The moves, `pub use`s and kit manifest.
2. The conformance world's lines and manifest; the root line.
3. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-test-git -p hearth-git-world -p hearth-conformance-world --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the **Measure**; the test counts before and after.
