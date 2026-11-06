+++
id = "hearth-spot"
part = "hearth-spot"
kind = "feature"
needs = []
files = ["hearth/crates/hearth-protocol-builder/src/lib.rs", "hearth/crates/hearth-protocol-builder/src/keep.rs", "hearth/crates/hearth-protocol-builder/src/read.rs"]
effort = "medium"
minutes = 15
+++

# hearth's builder protocol: three thin leftovers

## Goal

Finding 5 of the spot review of debt-25, checked against the trunk by the cut. Lines fall and behaviour stays.

Acceptance:

- `Current::keeping` is gone, and `keep.rs`'s `temporary`, which holds `&mut self`, reads `self.current.keeping_mut()`.
- `Current::git` is gone: `finish` in `lib.rs`, its one caller, matches the replaced `Current` itself, every variant listed, with the same message as the `expect`.
- `read.rs`'s `Current as Work` is gone: `current(current: &mut Current)` says `Current`.
- `hearth-protocol-builder`'s lib tests and `hearth-builder-world`'s pass with their counts unchanged.

## Inputs

- `temper/docs/plan/debt-26/inputs/debt24-review.md`, second half, finding 5.
- `temper/docs/plan/debt-26/notes/common.md`.

## Owns and may change

Owns the three files. `hearth-memory-5/machine` is changing `src/client.rs` in this crate, and fabrig/hearth holds `hearth-service-builder/src/lib.rs`: stay off both.

The review counts -6 for `git()`, but the listed match keeps some of them: `git_mut` stays, and so does anything else with a caller. Report what you measured; a smaller fall isn't a question.

## Steps

```rust
// lib.rs, finish
let (child, mode) = match core::mem::replace(&mut protocol.current, Current::Idle) {
    Current::Git { child, mode } => (child, mode),
    Current::Call(_) | Current::Idle | Current::Step(_) | Current::Keeping(_) | Current::Resolving(_) => {
        unreachable!("closed git child")
    }
};
```

Tests: `heavy cargo nextest run -p hearth-protocol-builder --lib`, `heavy cargo nextest run -p hearth-builder-world`. Then fmt, `python3 -I -B scripts/old_forms.py` from `hearth/`, `heavy cargo clippy -p hearth-protocol-builder --all-targets -- -D warnings`. Commit, push.

## Report

As your text says: lines before and after.
