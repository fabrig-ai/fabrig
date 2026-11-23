# service-tiers: status

2026-11-23. Where the tree's code lags or departs from [service-tiers.md](service-tiers.md).

- **Not yet:** 1: address-only reuse checks still do not count allocations
  for lib's reused list and UI's reused tree storage.
- **Not yet:** 1: `Deadline::expired_for` is not built; only the real
  loop's referee classifies against another wait's account, crate-private.
  [in flight: change 13987]

- **Departs:** 1: `skein/testing/skein-world/src/lib.rs` fences six
  items `cfg(unix)`, though these sections have the crate native, on
  Linux and macOS, so off unix it fails to build however it is fenced; no
  reason is recorded.
