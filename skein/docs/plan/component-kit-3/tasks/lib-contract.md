+++
id = "lib-contract"
part = "kit-contract"
kind = "interface"
needs = []
files = ["skein/crates/skein-lib/**"]
effort = "high"
minutes = 60
+++

## Goal

lib gains the primitives skein's components write privately today
(`skein/docs/design/draft/component-kit.md`, 3.1): an earliest-deadline
helper, `Queue::clear` and `Stack::clear`, one component `Lifecycle`, one
growth rule with one floor, slab iteration, and hex and decimal parsing.
Every later lib-sites task moves a crate onto exactly these items, so the
names and signatures below are binding. No call site outside skein-lib
moves in this task.

Acceptance:
- the items below exist, are documented in one or two lines each, and are
  tested in lib's own tests (`crates/skein-lib/src/tests/`);
- lib's five growing containers (`Queue`, `Stack`, `Slab`, `Intake`, `List`)
  use one growth rule and one `GROWTH_FLOOR`, and grow exactly as they do
  today (the same capacities after the same pushes);
- `requests.rs`'s private `earliest` is gone, replaced by the new helper;
- `heavy cargo nextest run -p skein-lib` and clippy pass.

## Inputs

- `skein/docs/plan/component-kit-3/notes/common.md`.
- `skein/docs/design/lib.md`, sections 3 (handles and slabs), 5
  (containers), 6 (bytes) and 8 (time); `skein/docs/foundation/programming-model.md`,
  5.2 (the lifecycle: the `Lifecycle` contract below).
- The copies, to see what callers need (read only, do not change):
  `crates/skein-llm-connection/src/component.rs` and
  `crates/skein-oauth-accounts/src/component.rs` (`enum Lifecycle` and its
  transitions), `crates/skein-llm-connection/src/deadlines.rs` and
  `crates/skein-forge-setup/src/host.rs` (`earlier`), `crates/skein-http/src/client/head.rs`
  (`decimal`), `crates/skein-forge/src/forgejo/id.rs` and
  `crates/skein-oauth-accounts/src/redirect.rs` (`hex`).

## Owns and may change

- Owns `skein/crates/skein-lib/**`. New modules `lifecycle.rs`, `growth.rs`
  and `hex.rs`; changes to `lib.rs`, `time.rs`, `queue.rs`, `stack.rs`,
  `slab.rs`, `intake.rs`, `list.rs`, `decimal.rs`, `requests.rs` and the
  tests.
- No other crate. Adding public items breaks nothing downstream; if a
  name collides in a crate that glob-imports `skein_lib::*`, report it
  rather than editing that crate.
- Pre-approved: the private shape of the growth rule (one `pub(crate)`
  function and one `pub(crate) const GROWTH_FLOOR`), the iterators' private
  fields, and where in `src/tests/` each test goes.

## Steps

1. The public surface, exactly:

   ```rust
   // time.rs, re-exported at the root: `pub use time::{Duration, Time, Wall, earliest};`
   /// The earlier of two optional deadlines; `None` only when both are.
   #[must_use]
   pub fn earliest(left: Option<Time>, right: Option<Time>) -> Option<Time>;

   // queue.rs and stack.rs
   impl<T> Queue<T> {
       /// Drops every item; keeps the cap and the storage.
       pub fn clear(&mut self);
   }
   impl<T> Stack<T> {
       /// Drops every item; keeps the cap and the storage.
       pub fn clear(&mut self);
   }

   // lifecycle.rs, re-exported: `pub use lifecycle::Lifecycle;`
   /// A component's lifecycle as its owner drives it (programming-model.md, 5.2).
   #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
   pub enum Lifecycle { Live, Closing, Aborting, Closed }
   impl Lifecycle {
       /// The owner's close: `Live` becomes `Closing`. True when it changed;
       /// a close while closing or aborting changes nothing.
       pub fn close(&mut self) -> bool;
       /// The owner's abort: `Live` or `Closing` becomes `Aborting`. True when it changed.
       pub fn abort(&mut self) -> bool;
   }
   // Neither changes `Closed` (both return false); the caller asserts what its
   // design asserts. `Closed` is set by assignment once everything below settled.

   // slab.rs, re-exported: `pub use slab::{Slab, SlabIter, SlabIterMut};`
   impl<T> Slab<T> {
       /// Each live entity (inserted, not retired) with its handle, in slot order.
       pub fn iter(&self) -> SlabIter<'_, T>;
       pub fn iter_mut(&mut self) -> SlabIterMut<'_, T>;
   }
   // SlabIter: Iterator<Item = (Id<T>, &'a T)>; SlabIterMut: Iterator<Item = (Id<T>, &'a mut T)>.

   // hex.rs, `pub mod hex;` beside `pub mod base64;`
   /// The lowercase ASCII digit of a value below sixteen; a larger one is a bug (`expect`).
   pub fn digit(nibble: u8) -> u8;
   /// Two lowercase digits, high nibble first.
   pub fn pair(byte: u8) -> [u8; 2];
   /// The value of an ASCII hex digit of either case, or `None`.
   pub fn value(byte: u8) -> Option<u8>;
   /// The value of a lowercase ASCII hex digit, or `None` (uppercase included).
   pub fn lower(byte: u8) -> Option<u8>;

   // decimal.rs
   impl Decimal {
       /// `1*DIGIT` as a `u64`, or `None`: empty, another byte, or past `u64::MAX`.
       /// Leading zeros are digits.
       #[must_use]
       pub fn parse(digits: &[u8]) -> Option<u64>;
   }
   ```

   All of it within the step-crate subset (programming-model.md, 10): no
   indexing, checked arithmetic, every `match` exhaustive, no closures.
2. The growth rule: one `pub(crate)` function in `growth.rs` that each of
   the five containers calls for its next storage, and one
   `pub(crate) const GROWTH_FLOOR: usize = 4;`. `Intake`'s rule also takes
   what it needs now (`.max(needed)` today); the others pass their length
   plus one, or the function takes an `Option`, as you judge clearer.
   Delete the five private constants and the private `growth` functions.
3. `requests.rs`: its private `earliest(&mut Option<Time>, Time)` becomes
   `*next = earliest(*next, Some(deadline))` at each call, and the function
   goes.
4. Tests in `src/tests/` (the existing files: `time.rs`, `containers.rs`,
   `handles.rs`, `bytes.rs`, or a new `hex.rs` and `lifecycle.rs` beside them),
   each a sentence about one behaviour:
   - `earliest`: each of the four `Some`/`None` combinations, and equal times;
   - `clear` keeps the room: after pushes and `clear`, `len` is zero and the
     same number of pushes fit again; a stack likewise;
   - `Lifecycle`: close then abort; abort then close; a second close; neither
     changes `Closed`;
   - slab iteration skips a retired entity before `reclaim`, yields handles
     that `get` resolves, and is empty for an empty slab; `iter_mut` changes
     what `get` then reads;
   - `hex`: `digit` and `value` round-trip every nibble; `value` takes `A`..`F`
     and `lower` refuses them; `pair(0xa5)` is `*b"a5"`; `digit(16)` panics
     (`#[should_panic]`);
   - `Decimal::parse`: `b"0"`, `b"007"`, `u64::MAX` written out, one past it,
     the empty slice and a non-digit;
   - growth: a queue, a stack, a list, a slab and an intake reach the same
     capacities after the same pushes as before (pin today's sequence in the
     test before you change the rule, then keep it green).
5. `cargo fmt`; `heavy cargo clippy -p skein-lib --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-lib`. lib's direct dependents need no
   run: nothing they call changes. Commit and push.

## Report

The surface as it landed (any deviation from step 1, and why), the growth
rule's shape, and the tests added.
