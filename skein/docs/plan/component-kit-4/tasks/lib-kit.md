+++
id = "lib-kit"
part = "kit-contract"
kind = "interface"
needs = []
files = ["skein/crates/skein-lib/**"]
effort = "high"
minutes = 75
+++

## Goal

lib gains the primitives each component writes again for itself
(`skein/docs/design/draft/component-kit.md, 3.1`): an earliest-deadline
helper, `Queue::clear` and `Stack::clear`, one `Lifecycle`, one growth rule,
slab iteration, and hex and decimal digits. No crate outside lib moves onto
them here; later tasks do.

Acceptance:

- The public items below exist with these names and signatures, each with
  a one- or two-line doc that states its contract, and unit tests in
  lib's `src/tests/` that pin each contract, edges included.
- lib's five growing containers (`intake.rs`, `list.rs`, `queue.rs`,
  `slab.rs`, `stack.rs`) use one growth rule with one floor: one `const
  GROWTH_FLOOR` left in lib, no change in what any container allocates.
- `requests.rs`'s private `earliest` is gone, using the new one.
- lib's tests and clippy pass; every skein package that depends on lib
  still builds (`heavy cargo check --workspace --tests` is the one wide
  build allowed here, since every crate reads lib; nothing outside lib may
  need a change: if one does, the item is wrong).

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md`.
- `skein/docs/design/draft/component-kit.md, 3.1`.
- `skein/docs/design/lib.md, 3, 5 and 8`; `programming-model.md, 5.2 and
  10.2` (lib's own allowances: generics, cursors with lifetimes).
- The private copies these replace, to read the contracts they need (do not
  change them): `crates/skein-llm-connection/src/component.rs:24` and
  `crates/skein-oauth-accounts/src/component.rs:122` (`Lifecycle`);
  `crates/skein-forge-setup/src/host.rs:376`,
  `crates/skein-llm-connection/src/deadlines.rs:159`,
  `crates/skein-oauth-accounts/src/component.rs:1239` (`earlier`);
  `crates/skein-http/src/client/head.rs:281`,
  `crates/skein-codec/src/patch/text.rs:327` (decimal parsing);
  `crates/skein-forge/src/marker.rs:67`, `crates/skein-json/src/writer.rs:593`
  (a hex digit); `crates/skein-oauth-accounts/src/redirect.rs:109`,
  `crates/skein-codec/src/patch/plan.rs:1058` (a hex value).

## Owns and may change

- Owns `skein/crates/skein-lib/` (its `src/`, its tests, its `lib.rs`
  exports).
- Nothing else.
- Pre-approved: if making `slab` a public module would clash with an
  existing public name, export the iterators as `skein_lib::SlabIter` and
  `skein_lib::SlabIterMut` instead and say so.

## Steps

1. The items (signatures bind; bodies and private helpers are yours):

   ```rust
   // time.rs, exported at the root
   /// The earlier of two optional deadlines: `None` only when both are.
   #[must_use]
   pub const fn earliest(left: Option<Time>, right: Option<Time>) -> Option<Time>;

   // queue.rs, stack.rs
   impl<T> Queue<T> {
       /// Empties the queue, keeping its grown storage (as `List::clear`).
       pub fn clear(&mut self);
   }
   impl<T> Stack<T> {
       /// Empties the stack, keeping its grown storage.
       pub fn clear(&mut self);
   }

   // lifecycle.rs, exported at the root
   /// A component's own lifecycle (programming-model.md, 5.2): it takes
   /// requests while `Live`; `Closing` after a close, `Aborting` after an
   /// abort, `Closed` once it has told its `Closed`.
   #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
   pub enum Lifecycle { Live, Closing, Aborting, Closed }

   // slab.rs, the module made public (`pub mod slab`, `Slab` still at the root)
   impl<T> Slab<T> {
       /// Every live entity (inserted, not retired), in slot order, with its handle.
       pub fn iter(&self) -> Iter<'_, T>;      // Item = (Id<T>, &T)
       /// The same, mutably.
       pub fn iter_mut(&mut self) -> IterMut<'_, T>;  // Item = (Id<T>, &mut T)
   }
   pub struct Iter<'a, T> { .. }     // impl Iterator
   pub struct IterMut<'a, T> { .. }  // impl Iterator

   // hex.rs, a public module
   /// The lowercase hex digit of `nibble`'s low four bits.
   pub const fn digit(nibble: u8) -> u8;
   /// The value of an ASCII hex digit of either case, or `None`.
   pub const fn value(byte: u8) -> Option<u8>;
   /// The value of a lowercase ASCII hex digit, or `None` (uppercase too).
   pub const fn lower_value(byte: u8) -> Option<u8>;

   // decimal.rs
   impl Decimal {
       /// The number ASCII decimal `digits` spell: `None` when empty, on any
       /// other byte, or past a `u64`. Leading zeros are digits.
       #[must_use]
       pub fn parse(digits: &[u8]) -> Option<u64>;
   }
   ```

   `Lifecycle` holds no data and no methods: it is the shared vocabulary
   of 5.2, nothing more. The slab iterators are concrete types, as `Map`'s
   `btree_map::Iter` is; no `impl Trait` in a signature.
2. The growth rule: one private module (`grow.rs` or similar) with the one
   `GROWTH_FLOOR` and one function every container calls. `intake.rs`
   grows by what it needs as well as by doubling: give the function that
   input, so all five use it, and keep each container's allocation figures
   exactly as they are (its existing tests show it; add one for intake if
   none pins its growth).
3. Replace `requests.rs`'s private `earliest` with the new one.
4. Tests in `src/tests/` beside the existing container tests: `earliest`
   on all four `Some`/`None` pairs and on equal times; `clear` keeps
   capacity (a refill to the old length allocates nothing, as lib's tests
   of `List::clear` check, if any; otherwise by `price` or len/room) and
   leaves the container usable; `Slab::iter` skips retired and reclaimed
   entities and vacant slots, yields handles `get` accepts, and visits in
   slot order after reuse; `hex::digit` for 0..=15 and a byte with high
   bits set; `value` and `lower_value` over every byte class (digits,
   `a-f`, `A-F`, `g`, `G`, `/`, `:`, `@`, `` ` ``); `Decimal::parse` on
   empty, `0`, `007`, `u64::MAX`, `u64::MAX + 1`, a non-digit first, last
   and between.
5. `cargo fmt`; `heavy cargo clippy -p skein-lib --all-targets -- -D
   warnings`; `heavy cargo nextest run -p skein-lib`; `heavy cargo check
   --workspace --tests`.
6. `old_forms.py --write` (common.md); commit and push.

## Report

As common.md says, plus: the growth rule's function and where intake's
input differs; the slab iterators' shape (and any borrow problem a mutable
walk would meet that you foresee for oauth-accounts' walks).
