# limits-worlds: the shapes that cross tasks

These are this run's binding calls. `plan.md` says why.

## A sweep's fork (`cuts-fork`)

`temper`'s native stories are straight-line functions with about forty nested drains, so a snapshot that has to resume them would mean rewriting each into phases, as jig's `Continuation` is. A forked process holds the story's position with no rewrite, and goes through the same `crash()` as a fresh cut.

**`skein/testing/skein-fork`,** a new kit on the `ring` lint profile, its one module allowed `unsafe` saying why (`tools/lints/design.md`, 2.4):

```rust
/// How a forked child ended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Exit { Code(i32), Signal(i32) }

/// Each side of one fork.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Split {
    /// This process is the child. It goes on from here, and ends at `passed` or at its first panic.
    Child,
    /// This process is the parent, and its child has ended.
    Parent(Exit),
}

/// Fork the process, and in the parent wait for the child. A child that splits again is refused.
#[must_use]
pub fn split() -> Split;

/// In a child, end it with exit code 0. In a process that isn't a child, return.
pub fn passed();
```

- **Before forking,** flush stdout and stderr, so that the child repeats nothing buffered.
- **In the child,** a panic prints as usual through the hook already set, then ends the process with `libc::_exit(101)`. It never unwinds into libtest, whose main thread doesn't exist in the child. `passed` also ends by `_exit`.
- **A test's process stays its own measure:** the parent waits for each child, so its CPU counts as a descendant's (`tools/tiers/test_cpu.py`).

**`jig/testing/jig-conformance/src/harness.rs`:**

```rust
pub enum Cut {
    None,
    AfterCommit(u64),
    Random { seed: u64 },
    /// Each commit from this number on: fork there; the child crashes as `AfterCommit` does,
    /// and the parent waits for it and goes on uncut (jig/docs/design/domain/testing.md, 5).
    EachCommitFrom(u64),
}

pub struct Outcome {
    pub commits: u64,
    pub iterations: u64,
    pub stopped: bool,
    pub crashes: Vec<u64>,
    /// The cuts whose forked children passed, in order. A child's own list starts empty.
    pub forked: Vec<u64>,
}

/// A forked cut's story has ended well: end its child. In the parent, this returns.
pub fn cut_passed();
```

`cut_passed` calls `skein_fork::passed`, and `lib.rs` exports it, so that a world names no fork. At an `EachCommitFrom` commit:

- **Child:** set `self.cut = Cut::None`, clear `self.forked`, then take the same `self.crash()?; return Ok(true)` that `AfterCommit` does.
- **Parent, `Exit::Code(0)`:** push the commit to `forked`, and go on as if uncut.
- **Parent, anything else:** return a `Violation` whose words name the commit and the exit. The child's own stderr, just above it, says why.

**A sweep in a world** then reads:

```rust
let first_live = Temper::store(&config).applied + 1;
let outcome = stories::run(story, Cut::EachCommitFrom(first_live), 101)
    .unwrap_or_else(|error| panic!("{story:?}: {error:?}"));
jig_conformance::cut_passed();
assert_eq!(outcome.forked, (first_live..=outcome.commits).collect::<Vec<_>>(), "{story:?}: every live commit, once");
```

A child runs to `cut_passed` and ends there, and only the parent reaches the assertion. `Cut::AfterCommit` stays: the focused cuts and the jig worlds keep it, so a fresh cut stays exercised.
