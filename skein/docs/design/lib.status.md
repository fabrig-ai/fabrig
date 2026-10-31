# lib: status

2026-10-31. Where `skein-lib` lags or departs from [lib.md](lib.md).

- **Built:** 3, 3.1, 5, 6.3, 11, 12: slab growth and its walk, the lifecycle, one growth rule and `clear` for queues, lists and stacks, container and table prices, uncapped collections and decisions, `Decimal::read`, hex digits, `Time::earliest`; journal limits hold only `release`.
- **Not yet:** 3: `Slab::get2_mut` (see `lib.status.md`).

- **3, the slab:** `Slab::get2_mut` is not built.
- **5, containers:** built: allocation prices are `price(capacity)`;
  `Slab` and `Deadlines` expose table prices, and `RequestTable` exposes
  `price(&limits)`.
- **11, the journal:** built: uncapped decisions use `push_write` and
  `push_held`; `JournalLimits` holds only `release`.
