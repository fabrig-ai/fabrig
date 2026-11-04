+++
id = "kv-admission"
part = "kv-review"
kind = "feature"
needs = ["kv-phase-ready"]
files = ["skein/crates/skein-kv/src/map.rs", "skein/crates/skein-kv/src/store.rs", "skein/crates/skein-kv/src/tests.rs"]
effort = "high"
minutes = 40
+++

# skein-kv: one admission check, run once

## Goal

The review's Part A, finding 5. `Queued::check` is a 45-line near-copy of `KvMap::check`, and
`commit_disk` checks twice: once to choose a refusal when the byte queue is full, once inside `admit`.
`KvMap::check` delegates, `apply` uses `has_later`, and `commit_disk` checks once, tests the bytes,
then reserves what the check projected. Every refusal stays in its order.

Acceptance:

- `rg -n 'ops\.get\(index \+ 1\.\.\)' skein/crates/skein-kv/src/map.rs` reads nothing.
- `commit_disk` makes one call to `check` and none to `admit`; `Queued::admit` is gone from `map.rs`.
- `queued_admission_matches_concatenated_ops_after_refusals` stays, and changes only at its calls.
- A commit past the byte queue that also fails the budget is refused `Full`, and one that fits it `Busy`,
  as today. `rg -n 'queued_bytes' skein/tests/kv` says whether a test holds this; if none does, add one
  to `src/tests.rs`.
- The note's `What a task runs` passes.

## Inputs

- `temper/docs/plan/debt-6/notes/common.md`, `temper/docs/plan/debt-6/notes/skein-kv.md`, `AGENTS.md`,
  `skein/AGENTS.md`.
- `skein/docs/design/kv.md`, 4 and 7.

## Owns and may change

- Owns: `skein-kv/src/map.rs`, `src/store.rs`'s `commit_disk`, `src/tests.rs`.
- `map.rs`'s `#![expect(clippy::arithmetic_side_effects, ..)]` goes if nothing is left under it: the
  compiler says so. Say so in the report.
- `KvMap::check`'s readers (`commit_memory`, `apply_snapshot`, `replay`, `batch_synced`) keep its
  signature, and need no edit.

## Steps

1. The shapes:

   ```rust
   /// What the admitted commits and one more leave: the map's used bytes and entries.
   #[derive(Clone, Copy, Debug)]
   pub(crate) struct Projected { used: u64, count: u32 }

   impl Queued {
       pub(crate) fn check(&self, map: &KvMap, ops: &[Op], limits: &Limits) -> Result<Projected, Refusal>;
       /// Admit `ops`, which `check` projected: they are commit `number`.
       pub(crate) fn reserve(&mut self, ops: &[Op], projected: Projected, number: u64);
   }

   impl KvMap {
       pub(crate) fn check(&self, ops: &[Op], limits: &Limits) -> Result<u64, Refusal> {
           Queued::new().check(self, ops, limits).map(|projected| projected.used)
       }
   }
   ```

2. `commit_disk`, after `encode`: `check`, refusing what it refuses; then the byte test, `Busy` past it;
   then `reserve`.
3. `apply`'s two passes use `has_later`.
4. The seeded test calls a small helper in `src/tests.rs`, `check` then `reserve`, where it calls `admit`.
5. Hand-break: in `Queued::check`, read the map before `charges`. The seeded test must go red: it is
   the referee of the check both paths now share. Then what the note's `What a task runs` names; the
   citations check. Commit and push.

## Report

The common note's table, and the note's line counts. If a refusal's order moved, that is a defect in
this task: say what you found and fixed.
