+++
id = "kit-slab"
part = "kit-contract"
kind = "interface"
repo = "fabrig"
needs = ["ratchets", "kit-lib"]
files = ["skein/crates/skein-lib/src/lib.rs", "skein/crates/skein-lib/src/growth.rs", "skein/crates/skein-lib/src/intake.rs", "skein/crates/skein-lib/src/list.rs", "skein/crates/skein-lib/src/queue.rs", "skein/crates/skein-lib/src/slab.rs", "skein/crates/skein-lib/src/stack.rs", "skein/crates/skein-lib/src/tests.rs", "skein/crates/skein-lib/src/tests/**", "skein/tests/lib/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 35
+++
# One growth rule, and slab iteration

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.1 (lib): "one growth rule with one floor, used by lib's five
growing containers; slab iteration".

## What to build

- A private `growth.rs` with the one `const GROWTH_FLOOR: usize = 4` and
  `pub(crate) fn grown(len: usize, needed: usize, cap: usize) -> usize`:
  the target capacity, `len` doubled, at least `needed` and the floor,
  never past `cap`. The five copies move onto it and drop their floors:
  `intake.rs:60/115` (which also takes `needed`), `list.rs:19/76`,
  `queue.rs:20/87`, `slab.rs:47/180`, `stack.rs:20/64`.
- `Slab::iter(&self)`: the occupied entities, retired ones included until
  `reclaim` (as `get` sees them), each `(Id<T>, &T)`, in slot order.
  Return a named iterator type exported beside `Slab`, in lib's style
  (`List::iter`, `Map::iter` return named iterators), not `impl Trait`.

## Acceptance

- `growth_floor` falls from 5 to 1 (`--write`).
- Each container's growth tests pass unchanged; add one test of `grown`
  at the floor, at the cap, and with `needed` above double.
- `Slab::iter` tests: empty; after inserts and a retire (still seen);
  after `reclaim` (gone); the ids it yields resolve with `get`.
- `heavy cargo nextest run -p skein-lib -p skein-lib-tests` (the package
  in `skein/tests/lib`; use its name from its `Cargo.toml`).
