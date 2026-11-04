# debt-6: skein-kv

For the `kv-*` tasks, which run one after another: each owns `skein-kv` for its turn, in the review's
order. `skein/docs/foundation/programming-model.md`, 5.4 and `skein/docs/design/kv.md` are the rules.

## Behaviour that stays

- The on-disk format. `skein/tests/kv/golden` does not move, and `skein/tests/kv`'s crash and golden
  tests pass unchanged. A task adds tests there only where its spec says.
- Every refusal, in its order. A `Close` while recovering waits, and `open_failed` answers it.
- A failed rollover create or directory sync stops the store, its commits unanswered:
  `skein/docs/design/kv.status.md`, 4. No task here answers them.
- A `Close` answered `Failed` on a snapshot's file closes it once more.
- `Store`, `down`, `up`, `fire` and `is_ready` keep their public signatures; `_above` stays. Every
  change in this part is inside the crate.
- `#[expect]`s are scoped, with a reason. Module-wide ones stay out of `store.rs` and its modules.

## What a task runs

`cargo fmt`; `heavy cargo clippy -p skein-kv -p skein-kv-tests --all-targets -- -D warnings`;
`heavy cargo nextest run -p skein-kv -p skein-kv-tests`.

This departs from the implementer's text: `skein-kv`'s public items do not change, nor what they mean,
so a task does not run its dependents; the trunk's end check does.

## The lines

The request aims at `store.rs` and its modules ending about 200 lines down, less what the two fixes
add. Each task's report gives, before and after,
`wc -l skein/crates/skein-kv/src/store.rs skein/crates/skein-kv/src/store/compaction.rs skein/crates/skein-kv/src/map.rs`.
At this plan's base they read 1,302, 368 and 267.
