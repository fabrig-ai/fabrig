+++
id = "skein-kv-subset"
part = "skein-kv-states"
kind = "feature"
needs = ["skein-kv-recovery"]
files = [
  "skein/crates/skein-kv/src/store.rs",
  "skein/crates/skein-kv/src/store/**",
  "skein/docs/development/debt/D017-kv-store-states.md",
  "skein/docs/development/debt/README.md",
  "skein/docs/design/kv.status.md",
]
effort = "high"
minutes = 50
+++

# skein D017, the subset: `store.rs` holds no module-wide exception, and the item goes

## Goal

The last task on `skein/docs/development/debt/D017-kv-store-states.md`: `Writing` and `Syncing`
hold their batch, the store's `while`s and closures go, and the module-wide `#![expect]`s with
them. It deletes the item and closes `skein/docs/design/kv.status.md`'s entry for 2, 3 and 4.

Acceptance:

- The item's **Measure**, `grep -c '^#!\[expect' skein/crates/skein-kv/src/store.rs`, prints 0, and
  so does `rg -c '^#!\[expect' skein/crates/skein-kv/src/store/` if that directory exists.
- `rg -nw 'loop|while|matches' skein/crates/skein-kv/src/store*` reads nothing, comments included.
- `Dir` holds no `active`; the note's `Phase` holds.
- `store.rs` and any `store/` module total at most 1,250 lines; say in your report if not, and why.
- The item's file and its README row are gone; `python3 -I -B tools/citations/citations.py --all`
  reads 0, and `skein/docs/development/debt/D025-request-table-in-lib.md` says what step 5 gives it.
- The note's `What a task runs` passes, with no test changed, and so does
  `heavy cargo nextest run -p jig-store-world -p temper-protocol-store`, the store's readers: every
  count above zero. `heavy cargo check -p temper-service -p jig-protocol-store --tests` passes.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/kv-store.md`, `AGENTS.md`,
  `skein/AGENTS.md`.
- `skein/docs/foundation/programming-model.md`, 10; `skein/clippy.toml`, whose bans `store.rs`
  meets from here on; `skein/docs/design/kv.status.md`.

## Owns and may change

- Owns: `skein-kv/src/store.rs`, and `src/store/*.rs` if split; the item, its row; the status
  entry.
- May change: `skein/docs/development/debt/D025-request-table-in-lib.md`, its one sentence.
- Pre-approved: a `Vec` or a closure with no shorter form stays under a scoped
  `#[expect(clippy::disallowed_types, reason = "...")]` or `disallowed_methods` on that item, its
  bound in the reason, and a `use alloc::vec::Vec` under its own. `segments` and `ReadingLog`'s
  `bytes` are likely ones.

## Steps

1. `batch: Queue<Pending>` on `Writing` and `Syncing`, `Queue::with_capacity(limits.queued)`,
   filled from `queue` with a `for` over its `len()`; the write's bytes are one `Box<[u8]>`
   allocated at the batch's total, not a `Vec` grown per frame. `Failed` answers the batch's
   commits as it answers `active`'s, then the queue's, in that order.
2. Each `while` becomes a `for` over a stated bound (`segment_name`'s over its digits, `replay`'s
   over `bytes.len()`, `start_batch`'s over `queue.len()`).
3. `Commit`'s and `commit_memory`'s two limit scans become one `too_large(ops, limits) -> bool`,
   a `for` over the ops. Each `is_none_or`, `is_some_and`, `any`, `position` and `find` becomes a
   `match` or a `for`. `rows: Vec<Row>` in `fire` becomes a `List` or a bounded `Box<[Row]>`, by what
   `snapshot::chunk` takes; change the private `snapshot.rs` call only if it must, and say so.
4. Remove the `#![expect]`s. Clippy says what is left; each goes by a reshape, or under the
   scoped `#[expect]` above.
5. Remove `skein/docs/design/kv.status.md`'s entry for 2, 3, 4, with the date line today's. The
   entries for 3, 7, 9 and for 4's rollover stay; if `ReadingLog`'s name moved, keep them true.
   Delete the item and its row. In D025, "flags are the shape D017 to D024 remove" becomes
   "D018 to D024".
6. What the note's `What a task runs` names and the readers above. One commit for the code, one
   for the debt and status text. Push.

## Report

`store.rs`'s lines before and after this task and at the plan's base, each scoped `#[expect]` left
with its reason, the item's Measure, and what the readers ran.
