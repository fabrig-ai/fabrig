# lib: status

2026-10-29. Where `skein-lib` lags or departs from [lib.md](lib.md).

- **3, the slab:** `Slab::get2_mut` is not built.
- **5, containers:** built: allocation prices are `price(capacity)`;
  `Slab` and `Deadlines` expose table prices, and `RequestTable` exposes
  `price(&limits)`.
- **11, the journal:** built: uncapped decisions use `push_write` and
  `push_held`; `JournalLimits` holds only `release`.
