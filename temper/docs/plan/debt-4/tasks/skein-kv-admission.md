+++
id = "skein-kv-admission"
part = "skein-kv-states"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-kv/src/map.rs",
  "skein/crates/skein-kv/src/store.rs",
  "skein/crates/skein-kv/src/tests.rs",
]
effort = "high"
minutes = 45
+++

# skein D017, admission: a commit pays for its own ops

## Goal

`down` copies every queued op on each commit to ask the map whether the whole queue fits: the
O(n²) copy `skein/docs/development/debt/D017-kv-store-states.md` says goes. A new `map::Queued`
holds what the queued commits will leave, so a commit is checked against it with its own ops. Every
admission answers what it answers today. This is the first of the item's five tasks; the item
stays open.

Acceptance:

- `all_ops` is gone, and `Commit` copies no queued op.
- A commit's refusals are today's, in today's order. The new test holds `Queued::admit` to
  `KvMap::check` over the concatenated ops.
- The note's `What a task runs` passes, with no existing test changed.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/kv-store.md`, `AGENTS.md`,
  `skein/AGENTS.md`.
- `skein/docs/development/debt/D017-kv-store-states.md`, whole; `skein/docs/design/kv.md`, 2, 4 and 7.
- `skein/docs/foundation/programming-model.md`, 10.

## Owns and may change

- Owns: `skein-kv/src/map.rs`, `skein-kv/src/store.rs`, and one added test in `skein-kv/src/tests.rs`.
- `KvMap::check` stays: `commit_memory`, `replay`, `apply_snapshot` and the `Syncing` arm read it.

## Steps

1. In `map.rs`:

   ```rust
   /// What the commits in flight and queued leave behind: each key they touch, with its charge
   /// once they apply, and the map's used bytes and entries then (skein/docs/design/kv.md, 7).
   #[derive(Debug)]
   pub(crate) struct Queued {
       charges: Map<Box<[u8]>, Option<u64>>,   // None: erased
       used: u64,
       count: u32,
   }

   impl Queued {
       pub(crate) fn new() -> Queued;
       /// Admit `ops` after those already admitted, or refuse them and change nothing.
       pub(crate) fn admit(&mut self, map: &KvMap, ops: &[Op], limits: &Limits) -> Result<(), Refusal>;
       /// Nothing is queued or in flight any more: forget what was projected.
       pub(crate) fn clear(&mut self);
   }
   ```

   `admit` starts from the map's `used` and entry count when `charges` is empty, and from its own
   otherwise; a key's old charge is `charges`' if it has one, else the map's; only each key's last
   op counts, as in `check`; it refuses `Full` by `check`'s same test. Applying a commit does not
   change what `Queued` holds, since it is the projection of them all. `charges` is
   `Map::uncapped()`: its keys are bounded by the commits it projects.

2. In `store.rs`, the note's `admitted: Queued` field. `Commit` calls
   `store.admitted.admit(&store.map, &ops, &env.limits)` where `all_ops` and `check` stand, and a
   refusal returns as it does. `Syncing`'s arm calls `clear` when `active` and `queue` are empty,
   and `fail_commits` always; `recover` needs nothing more. `all_ops` goes.
3. In `tests.rs`, a seeded test over a few hundred random puts and erases on a small key set: each
   commit's `Queued::admit` agrees with `KvMap::check` over all the ops so far, on the answer, and
   a refused commit leaves the next answer unchanged.
4. What the note's `What a task runs` names; the citations check. Commit and push.

## Report

The test's count, and any admission that differed from `check`.
