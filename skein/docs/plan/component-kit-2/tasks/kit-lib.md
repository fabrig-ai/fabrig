+++
id = "kit-lib"
part = "kit-contract"
kind = "interface"
needs = []
files = ["skein/crates/skein-lib/src/lib.rs", "skein/crates/skein-lib/src/time.rs", "skein/crates/skein-lib/src/queue.rs", "skein/crates/skein-lib/src/stack.rs", "skein/crates/skein-lib/src/lifecycle.rs", "skein/crates/skein-lib/src/hex.rs", "skein/crates/skein-lib/src/decimal.rs", "skein/crates/skein-lib/src/requests.rs", "skein/crates/skein-lib/src/tests.rs", "skein/crates/skein-lib/src/tests/**"]
effort = "high"
minutes = 35
+++
# lib's missing small primitives

## Goal

Draft, 3.1 (lib): the earliest-deadline helper, `Queue::clear` and
`Stack::clear`, one `Lifecycle`, and hex and decimal parsing, as lib's
public items with short contracts and lib's own tests. No call site
outside lib moves in this task; lib's own private `earliest`
(`src/requests.rs:718`) does.

Acceptance:

- The items below exist with these signatures, each with a `///`
  contract, and lib's tests cover each (`heavy cargo nextest run -p
  skein-lib` passes; name the new tests and the count).
- `src/requests.rs` uses `crate::earliest`; its private `fn earliest`
  is gone.
- `heavy cargo clippy -p skein-lib --all-targets -- -D warnings` is clean.
- `heavy cargo check -p skein-io -p skein-shell --tests` passes (lib's
  first readers).

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`.
- `skein/docs/design/lib.md`, 5 (containers) and 8 (time);
  `skein/docs/foundation/programming-model.md`, 5.2 (the lifecycle) and 8
  (counts as text).
- The shapes the call sites use, to see what the items replace:
  `skein/crates/skein-llm-connection/src/deadlines.rs:159` (`earlier`),
  `skein/crates/skein-forge-setup/src/host.rs:376` (`earlier`, `&mut`
  shape), `skein/crates/skein-llm-connection/src/component.rs:24` and
  `skein/crates/skein-oauth-accounts/src/component.rs:122` (`Lifecycle`),
  `skein/crates/skein-http/src/client/head.rs:281` (`decimal`),
  `skein/crates/skein-forge/src/forgejo/id.rs:64` (`hex` digit),
  `skein/crates/skein-oauth-accounts/src/redirect.rs:109` (`hex` value).
  Read only; they move in the lib-sites tasks.

## Owns and may change

- skein-lib's files in the header. Nothing outside skein-lib.
- `src/slab.rs`, `src/list.rs`, `src/intake.rs` and lib's growth rule
  are `kit-slab`'s: do not touch them.

## Steps

1. **`earliest`**, in `src/time.rs`, exported from the crate root beside
   `Time`:

   ```rust
   /// The earlier of two deadlines, either of which may be absent: `None`
   /// only when both are.
   #[must_use]
   pub fn earliest(a: Option<Time>, b: Option<Time>) -> Option<Time>
   ```

   `requests.rs`'s `earliest(&mut next, deadline)` callers become `next
   = earliest(next, Some(deadline))` (or `*next = ...`).
2. **`clear`** on `Queue<T>` (`src/queue.rs`) and `Stack<T>`
   (`src/stack.rs`):

   ```rust
   /// Drops every item, keeping the storage it has grown.
   pub fn clear(&mut self)
   ```
3. **`Lifecycle`**, a new `src/lifecycle.rs`, exported from the root:

   ```rust
   /// Where a component is in its lifecycle (programming-model.md, 5.2):
   /// taking requests, closing gracefully, aborting, or closed after its
   /// one `Closed`.
   #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
   pub enum Lifecycle { Live, Closing, Aborting, Closed }
   ```

   A one-line `///` on each variant. No methods.
4. **Hex**, a new `pub mod hex` (`src/hex.rs`):

   ```rust
   /// A nibble's lowercase ASCII hex digit. A nibble above 15 is a bug.
   #[must_use]
   pub fn digit(nibble: u8) -> u8
   /// An ASCII hex digit's value, either case, or `None` for another byte.
   #[must_use]
   pub fn value(byte: u8) -> Option<u8>
   ```

   A module doc of one line citing `skein/docs/design/lib.md, 6`.
5. **Decimal parsing**, on `Decimal` (`src/decimal.rs`), beside `of`:

   ```rust
   /// `1*DIGIT` as a `u64`: `None` for empty input, a byte that is not an
   /// ASCII digit, or a value past `u64::MAX`.
   #[must_use]
   pub fn parse(digits: &[u8]) -> Option<u64>
   ```

   Extend the module doc by one line: it reads counts too.
6. **Tests**, in lib's own test files (`src/tests.rs`, `src/tests/time.rs`,
   `src/tests/containers.rs`, a new `src/tests/text.rs` if hex and decimal
   want one), one behaviour each: `earliest` with each side absent and
   both present (either order, equal); `clear` empties a queue and a
   stack that then take pushes up to their capacity again; `digit` for
   0, 9, 10, 15 and a panic above 15; `value` for both cases and a
   refused byte; `parse` for `0`, `u64::MAX`, one past it, empty, a sign
   and a space.
7. Checks of `common.md` (no count changes in this task); commit; push.

## Report

The items with their final signatures, the tests added (names and
count), the checks run.
