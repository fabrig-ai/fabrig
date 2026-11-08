# skein-walk-reuse: the patcher's reuse checks hold in any order in one process

Since 11-08 the gate runs each test binary's tests in one process, in sequence (`skein/docs/foundation/testing-strategy.md`, 8: "A binary is a process"). One of skein-codec's focused tests now fails depending on which tests ran before it in that process:

- **The test:** `skein/crates/skein-codec/src/patch/run/tests.rs`, `nested_walks_grow_both_stacks_and_reuse_them`.
- **The failure:**

  ```
  assertion `left == right` failed: nested walk reuses reserved stacks
    left:  Allocations { entries: [(0, 0x4), (8, 0x…2570)], out: (13, 0x…0db0), walk: (5, 0x…1690), opened: (5, 0x4) }
    right: Allocations { entries: [(0, 0x4), (8, 0x…2570)], out: (13, 0x…0db0), walk: (5, 0x…2260), opened: (5, 0x4) }
  ```

- **What it says:** the walk stack's capacity is the same, 5, but its buffer moved. Either the walk stack is freed and allocated again during the walk, and a fresh process's allocator happened to hand back the same address, or something else replaces the vector.

## What's wanted

1. **Find which it is, and fix the cause.**
   - If the patcher drops or replaces the walk stack's buffer where the design says it reuses it, the patcher keeps its buffer. That is the inefficiency the test exists to catch.
   - If the buffer legitimately moves, the reuse check stops depending on addresses. It counts allocations instead, for example with `skein-heap`'s counting allocator if the crate's tests can use it, or compares capacities.
2. **Every test in the file that compares `allocations(...)` by address** gets the same treatment, so that none depends on what ran before it in the process.
3. **The binary passes in one process,** in its listed order and in reverse: `tools/tiers/one-process` over skein-codec's focused tests, or the test binary with `--test-threads=1`.

## Limits

- No test moves to a process of its own while a fix at the cause exists.
- No budget, tier or filter changes.
- `walk = false`. Effort low. One task.

## Sources

- `skein/crates/skein-codec/src/patch/run/tests.rs`

Read by section: `skein/docs/foundation/testing-strategy.md`, 8.

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands.** Never `tools/tiers/run`, nor a browser, process or live tier. No walk, render check or throwaway driver.
- **Decide, don't ask.** A small edit just outside a task's file list is made, and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
