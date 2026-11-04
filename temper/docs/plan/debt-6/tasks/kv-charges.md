+++
id = "kv-charges"
part = "kv-review"
kind = "feature"
needs = []
files = ["skein/crates/skein-kv/src/map.rs", "skein/crates/skein-kv/src/store.rs", "skein/crates/skein-kv/src/tests.rs"]
effort = "high"
minutes = 60
+++

# skein-kv: `Queued::charges` holds only what is in flight or queued

## Goal

The review's Part A, finding 1. `Queued::clear` runs only when the queue is empty at `batch_synced`.
Under sustained load every batch has a successor, so it never runs: erased keys stay as `None`
charges, each holding a copy of its key, no budget counts them, and put-and-erase churn on distinct
keys grows the heap until traffic pauses. The cut read this in `map.rs` and `store.rs`'s
`batch_synced` and agrees. Each charge is tagged with the last commit that touched it and goes once
that commit is in the map; a miss then falls back to the map, which is correct by then.

Acceptance:

- A test that drives a `Store` through put-and-erase churn on distinct keys, with a commit queued at
  every sync, asserts the projected keys stay bounded, and **fails on today's code**.
- `rg -c 'admitted\.clear\(\)' skein/crates/skein-kv/src/store.rs` reads 1: `commits_failed`'s.
- `queued_admission_matches_concatenated_ops_after_refusals` stays, and also runs the fallback.
- The note's `What a task runs` passes, and `skein/tests/kv` is unchanged.

## Inputs

- `temper/docs/plan/debt-6/notes/common.md`, `temper/docs/plan/debt-6/notes/kv.md`, `AGENTS.md`,
  `skein/AGENTS.md`.
- `skein/docs/design/kv.md`, 4 and 7.

## Owns and may change

- Owns: `skein-kv/src/map.rs`, `src/store.rs`'s `commit_disk` and `batch_synced`, `src/tests.rs`.
- The seeded test's call sites of `admit` take the new argument; beyond that it changes only as Step 4
  says.
- If the accessors in Step 1 cannot be `#[cfg(test)]`-only for a reason you find, make them
  `pub(crate)` and say why.

## Steps

1. Write the churn test first, in `src/tests.rs`, and see it fail on today's code. Fold what
   `commit_is_invisible_until_sync` does by hand into a small helper that opens a store. Raise
   `limits.segment` past what the churn writes, so no rollover stops the batches and fills the queue;
   then only `WriteAt` and `Sync` need answers, and no `fire` means no snapshot.
   `limits()` holds `queued: 8` and `ops: 8`, so at most 128 keys are ever in flight and queued:

   ```rust
   #[test]
   fn put_and_erase_churn_on_distinct_keys_keeps_the_projection_bounded() {
       // Each round queues a put and an erase of a key no round used, then answers the one
       // outstanding request, so every sync finds a commit queued behind it.
       // assert the store's projected keys <= queued * ops after each round, over 1,024 rounds.
   }
   ```

   The test reads the count through two accessors, each `#[cfg(test)] pub(crate)`:
   `Queued::keys(&self) -> u32` and `Store::projected_keys(&self) -> u32`. Assert the churn ran under
   load: the queue is never empty at a sync, which `Committed` numbers and the next `WriteAt` show.
2. In `map.rs`, tag each charge:

   ```rust
   /// A key's charge once the commits touching it apply, and the last of those commits.
   struct Charge {
       bytes: Option<u64>,
       number: u64,
   }

   pub(crate) struct Queued {
       charges: Map<Box<[u8]>, Charge>,
       used: u64,
       count: u32,
   }

   impl Queued {
       pub(crate) fn admit(&mut self, map: &KvMap, ops: &[Op], limits: &Limits, number: u64)
           -> Result<(), Refusal>;
       /// Commit `number` is in the map: forget the keys no later commit touches.
       pub(crate) fn applied(&mut self, ops: &[Op], number: u64);
   }
   ```

   `applied` walks `ops`, so it costs what the commit costs. `used` and `count` stay what every admitted
   commit leaves, so nothing else changes; `check`'s `charges.is_empty()` still means the map is at
   the projection, because a key goes only once its last commit has applied, and commits apply in order.
3. In `store.rs`: `commit_disk` passes `number`. `batch_synced` calls
   `store.admitted.applied(&pending.ops, pending.number)` after `store.map.apply`, and its
   `if store.queue.is_empty()` goes. `clear` stays for `commits_failed`, its comment saying so.
   `Map::uncapped()` stays: `admit`'s assert message, that projected keys are bounded by admitted
   commits, is true now.
4. Two tests for the tag, in `src/tests.rs`:
   - In the seeded test, the `APPLY_EVERY` branch calls `admitted.applied(&accepted, number)` after it
     applies them, so admission falls back to the map under the same referee. Use `step` as `number`.
   - A new one: two admitted commits touch one key with different lengths; applying the first keeps
     the second's charge. Assert `check` of a third commit reads as the concatenated `map.check` does.
     Hand-break for it: `applied` drops a key whatever its tag.
5. What the note's `What a task runs` names; the citations check. Commit and push.

## Report

The common note's table: today's code red on the churn test (the count it reached), and the tag's
hand-break. The note's line counts.
