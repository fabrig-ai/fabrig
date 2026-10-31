+++
id = "kit-slab"
part = "kit-contract"
kind = "interface"
needs = ["kit-lib", "counts"]
files = ["skein/crates/skein-lib/src/lib.rs", "skein/crates/skein-lib/src/growth.rs", "skein/crates/skein-lib/src/slab.rs", "skein/crates/skein-lib/src/queue.rs", "skein/crates/skein-lib/src/stack.rs", "skein/crates/skein-lib/src/list.rs", "skein/crates/skein-lib/src/intake.rs", "skein/crates/skein-lib/src/tests/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 35
+++
# One growth rule in lib, and slab iteration

## Goal

Draft, 3.1 (lib): lib's five growing containers each write the growth
rule with their own `GROWTH_FLOOR`; they share one. And `Slab` gains
iteration over its live entities, as a cursor, so that oauth-accounts
can drop the linked list it keeps over its slab (`lib-accounts-slab`).

Acceptance:

- One `const GROWTH_FLOOR` in lib, in `src/growth.rs`; `queue.rs`,
  `stack.rs`, `list.rs`, `intake.rs` and `slab.rs` use its function.
  `growth_floor` falls from 5 to 1 (`--write`).
- Capacity behaviour is unchanged: lib's container tests pass as they
  are (`heavy cargo nextest run -p skein-lib`), and `heavy cargo nextest
  run -p skein-lib-tests` (tests/lib) passes, including its memory suite.
- `Slab::first` and `Slab::after` exist with the contracts below; tests
  cover: an empty slab; slot order; a retired entity is skipped before
  `reclaim`; a reused slot after `reclaim` is found under its new handle;
  `after` from a retired entity's handle still finds the next live one;
  a full walk visits each live entity once.
- `heavy cargo clippy -p skein-lib --all-targets -- -D warnings`.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`;
  `notes/decisions.md` (the cursor, and why not a borrowing iterator).
- `skein/docs/design/lib.md`, 3 (handles and slabs) and 5 (containers).
- The five rules: `src/queue.rs:84-91`, `src/stack.rs:~60-66`,
  `src/list.rs:~72-78`, `src/intake.rs:~110-117` (with `max(needed)`),
  `src/slab.rs:178-182` (`growth`, used for the slab's lists).
- The walks the cursor replaces, read only:
  `skein/crates/skein-oauth-accounts/src/component.rs:543-560` and
  `:1001-1040`.

## Owns and may change

- skein-lib's files in the header. `src/lib.rs` gains only `mod
  growth;`. `kit-lib`'s items are merged on your base: leave them.

Pre-approved:

- Intake's rule takes a `needed` floor the others lack: the shared
  function may take the needed length as a parameter (the others pass
  `len + 1`), or intake calls it and takes the larger of its result and
  `needed`. Either, if intake's tests pass unchanged.

## Steps

1. `src/growth.rs`, crate-private:

   ```rust
   //! The growth rule lib's containers share (lib.md, 5): double, at least
   //! the floor, never past the cap.
   pub(crate) const GROWTH_FLOOR: usize = 4;
   /// The length a full container of `len` items grows to under `capacity`.
   pub(crate) fn target(len: usize, capacity: usize) -> usize
   ```

   (Shape the helper as the five call sites need; one function and one
   floor.)
2. The five containers call it; their own floors go.
3. On `Slab<T>`:

   ```rust
   /// The first live entity in slot order, or `None`. Retired entities
   /// are not live, though `get` still reaches them until `reclaim`.
   #[must_use]
   pub fn first(&self) -> Option<Id<T>>
   /// The next live entity after `id`'s slot, in slot order, or `None`.
   /// `id` may name a retired or reclaimed entity: the walk goes on from
   /// its slot.
   #[must_use]
   pub fn after(&self, id: Id<T>) -> Option<Id<T>>
   ```

   A walk is `let mut next = slab.first(); while let Some(id) = next {
   next = slab.after(id); ... }`: it holds no borrow, so the loop body
   may take `&mut` of the owner. Extend the type's doc by one sentence
   naming the walk.
4. Tests in `src/tests/handles.rs`, where slab's tests live.
5. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

The helper's final shape, how intake's floor was kept, the cursor's
tests (names, count), `growth_floor` before and after.
