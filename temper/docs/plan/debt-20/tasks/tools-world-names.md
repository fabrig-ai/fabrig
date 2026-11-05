+++
id = "tools-world-names"
part = "tools-namespace"
kind = "feature"
needs = []
files = ["smith/tests/tools/**", "smith/crates/smith-domain-tools/src/tests.rs", "smith/tests/session/tests/suite/jobs.rs", "smith/tests/session/tests/suite/orders.rs", "smith/tests/session/src/world.rs"]
effort = "high"
minutes = 100
+++

# smith: the tools' tests mount what a run mounts

## Goal

The first half of `smith/docs/development/debt/D007-tools-namespace.md`. The only production constructor mounts one name (`smith-domain/src/translate.rs`), so the tests prove a namespace no run builds: `/work/temper`, a repository mounted inside another, the filesystem's root. Move every test onto one-name mounts while the kit still takes the general shape, so `tools-one-name` only has types left to change. **No production file changes in this task.**

Acceptance:

- `rg -n 'vendor/lib|third_party' smith/tests/tools smith/crates/smith-domain-tools/src` finds nothing.
- Every `Directory` these tests build mounts one name, and every `cwd` is that one name or empty: `fixture.rs` mounts `temper` and `docs`, and a read-only `lib` beside them, on the hosts and roots they have (`work/temper`, `work/docs`, a new `work/lib`). No test mounts `/`, or names `/work/...` as a mount.
- `heavy cargo nextest run -p smith-tools-world -p smith-domain-tools` passes, and `-p smith-session-world` with `jobs` and `orders` moved.
- The one-name shape has stories of its own, in the tools world: a relative path from the working directory answers `Resolved::Working`; a sibling answers `Resolved::Other` by its name; a sibling read-only refuses a write with `ReadOnly`; listing `/` answers the directories' names, sorted; `..` out of a directory reaches a sibling only by its name; a name that only starts like a mount (`docs2`, `temperature`) is `Outside`.
- By hand: make `locate` take a sibling's writability from the first mount; the read-only story goes red. Make `directories` skip the working name; the listing story goes red. Restore each.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `smith/docs/development/debt/D007-tools-namespace.md`
- `smith/docs/design/domain/tools.md`, 2 (the namespace these fixtures follow)

## Owns and may change

- **Owns:** the files in `files`.
- **This task is the common note's exception on assertions.** A test's assertion may change in two ways only, and your report says how many of each:
  1. **Its path's spelling, as the one-name namespace yields it:** a call's `/work/temper/src/lib.rs` is `/temper/src/lib.rs`; an outcome's `Resolved::Working { path: b"temper/src/lib.rs" }` is `b"src/lib.rs"`; `docs` under `work` is `Resolved::Other { directory: docs, .. }`. A size a test derives from a path's bytes follows its spelling, and only that (`tests.rs`' "22 bytes" note).
  2. **A story of a shape that goes:** a mount inside another, a root mount, a link into a nested mount (`third_party`), and the longest-mount choice between them are dropped, or restated on the read-only sibling where they also prove something a run can meet. Your report lists each dropped test by name, with what it proved.
  The outcome's kind stays in every other case (a `ReadOnly` stays `ReadOnly`, a `NotFile` stays `NotFile`). If one turns into another kind with a one-name mount, stop and ask: that's a behaviour you've found.
- **Host aliases and their own `longest`** in `locate` are a run's shape (a host path can sit inside another), so their tests stay.
- **`checkout`'s tree** may gain `work/lib`; `escape`, `absolute` and `loop` links stay.
- **A link from a directory into a sibling** is out of its root, so it refuses as any escape does. That's what such a link does in a run: no story needs it beyond what `escape` proves.
- **Reader files, call sites only:** `smith/tests/session/tests/suite/{jobs,orders}.rs` and `smith/tests/session/src/world.rs` take one-name mounts. `jobs.rs` mounts the root with no working name today: give it a name and its `cwd`. The session's other files mount one name already.
- **`smith/benchmarks/tasks/**` stays:** its grader is a task's fixture, not this kit's reader, and tests read that tree.

## Steps

1. `fixture.rs`, then `smith/tests/tools/**`, a commit once it passes.
2. `smith-domain-tools/src/tests.rs`, a commit.
3. The session world's two tests and its `world.rs`.
4. The new stories in `smith/tests/tools/tests/suite/simulation.rs`, and the breaks by hand.
5. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-tools-world -p smith-domain-tools -p smith-session-world --all-targets -- -D warnings`; the tests above. The citations check. Commit and push.

## Report

The common note's items; each kind of assertion change, counted; each dropped test with what it proved; what you broke and what failed.
