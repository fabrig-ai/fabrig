# golden: status

2026-11-23. Where the tree's code lags or departs from [golden.md](golden.md).

- **Not yet:**
  - 1: the generator's test of a declaration inserted into a schema, its records' module keeping every earlier line [in flight: change 14018]
  - 1: the same test for a translation module, with the declaration's maps line [in flight: change 14019]
  - 1: `bounds_text`; skein's families' bounds files and their drift checks; the memory harness clamping by `at_most`; skein's tests naming no limit to clamp [in flight: change 14033]
  - 1: jig's families' bounds files; jig's memory tests without limit lists or pins; `task_bounds.rs` gone [in flight: change 14040]
  - 1: smith's families' bounds files; smith's memory tests without limit lists or pins [in flight: change 14041]
  - 1: temper's families' bounds files; temper's memory tests without limit lists or pins [in flight: change 14042]
  - 1: the memory harness taking no limit list or pin [in flight: change 14075]
- **Departs:**
  - 1: skein-codegen's construct tests read `every-v3` from its file, because its owner is a private world whose library no other package may name (`fabrig-boundary`'s `skein_worlds` test). Decided by the goal's coordinator, 2026-11-19.
  - 1: an option whose value is an option generates `Option<Option<T>>`, which `clippy::option_option` rejects outside exported items. Only the pin, `every-v3`, nests options, and only as a public field's type, which clippy leaves alone in exported items (its `avoid_breaking_exported_api`); the lean form has no accessor or `Parts` signature for the lint to reach, so its module's declaration carries no expectation. A family that nests options in earnest takes the generator's fix first. Decided by the goal's coordinator, 2026-11-18, to keep the pin off a generator change.
