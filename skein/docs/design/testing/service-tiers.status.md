# service-tiers: status

2026-11-23. Where the tree's code lags or departs from [service-tiers.md](service-tiers.md).

- **Not yet:** 1: address-only reuse checks still do not count allocations
  for lib's reused list and UI's reused tree storage.
- **Not yet:** 1: the real loop's kill records are checked by the real
  loop's own stories, not yet without a kernel over its handed-in state.
  [in flight: change 13901]

- **Departs:** 1: `skein/testing/skein-world/src/lib.rs` fences six
  items `cfg(unix)`, though these sections have the crate native, on
  Linux and macOS, so off unix it fails to build however it is fenced; no
  reason is recorded.
