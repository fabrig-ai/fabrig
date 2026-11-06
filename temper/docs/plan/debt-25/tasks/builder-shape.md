+++
id = "builder-shape"
part = "spot-review"
kind = "feature"
needs = []
files = ["hearth/crates/hearth-protocol-builder/src/**"]
effort = "high"
minutes = 55
+++

# hearth-protocol-builder: one resource type, accessors in place of "nothing" lists

## Goal

Finding 2's `client.rs` and `read.rs` parts and finding 3's hearth part in `temper/docs/plan/debt-25/inputs/debt23-review.md`, checked against the code. `client.rs`'s `Output` is `held::Held` under other names. `Stream::Ending.printing` is copied and never read. `Read::take_answer`'s `Caching` arm looks dead. `resume`, `store.rs` and `keep.rs` write out five-variant lists to say "nothing here".

Acceptance:

- `client.rs`'s `Output` goes: `output` is the crate's resource type (`Unbound` is `Awaited`, `Bound` is `Open`). Its `io::Event::Closed` rule stays a match in `client.rs`.
- `Stream::Ending` holds only `outcome`.
- `take_answer`'s `Current::Caching` arm joins its `unreachable!` set, if the reading below holds and every test still passes.
- `Current` (in `lib.rs`) gets `resolving()`, `keeping()` and `git()`, each with a `_mut` form only where a site needs one, and the lists in `store.rs`'s `store_request` and `resume_store`, `keep.rs`'s `temporary` and `lib.rs`'s `resume` read through them.
- `resume`'s `#[expect(clippy::too_many_lines)]` goes: its git tail, from the `Current::Git` split on, becomes a function of its own, in the same order. The other arms' priority doesn't change. Don't raise clippy's threshold.
- Renames: `read.rs`'s `Current` is `Holding`; the crate's `Held` is `Resource`, and `held.rs` is `resource.rs`.
- No test's assertion changes. Production falls by about 40 lines, renames aside.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, findings 2 and 3
- `hearth/docs/design/domain/render.md`, 2.2
- `skein/docs/foundation/programming-model.md`, 5.2 and 5.4

## Owns and may change

- **Owns:** `hearth/crates/hearth-protocol-builder/src/**`, this crate's tests included, in their spelling only.
- **Stay off** `git.rs`'s `Stage` and `Intake`, beyond the rename: `builder-stage` moves them next. `hearth/crates/hearth-service-builder/src/lib.rs` is not yours.
- **The `Caching` arm.** `take_answer` has two callers. `resume`'s last branch can't see a settled, finished cache, because the `Caching` block returns first. `lib.rs`'s `finish` calls `take_terminal` first, and that holds the last blob's event. If a test reaches the arm after you remove it: restore it, say which test and why, and carry on. `render::vocabulary::Event::Cached` stays either way: `hearth-domain-builder` produces it too.
- **An accessor that saves no line at a site:** leave that list and say so.

## Steps

1. `client.rs`, then `read.rs`'s arm. Run `heavy cargo nextest run -p hearth-protocol-builder --lib`.
2. The accessors and `resume`'s split, then the renames last, as their own commit.
3. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-protocol-builder --all-targets -- -D warnings`; `heavy cargo nextest run -p hearth-protocol-builder --lib`, then `-p hearth-render-world`, `-p hearth-builder-world`, `-p hearth-service-builder` and `-p hearth-test-render`. Commit and push.

## Report

The common note's items; the `Caching` arm's outcome; any list left. These are moves and renames, but for the arm: removing it is its own check, so no rule to break by hand.
