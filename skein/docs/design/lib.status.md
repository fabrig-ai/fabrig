# lib: status

2026-10-31. Where `skein-lib` lags or departs from [lib.md](lib.md).

- **Built:** 3, 5, 11, 12: slab growth, container and table prices, uncapped collections and decisions; journal limits hold only `release`.
- **Not yet:** 3: `Slab::get2_mut` (see `lib.status.md`).

- **3, 5, 6, 8, lib primitives:** built: `Lifecycle`, shared buffer growth,
  Queue and Stack clear with retained storage, live slab iteration,
  ASCII hex digits, `Decimal::parse` and `Time::earliest`.
- **3, the slab:** `Slab::get2_mut` is not built.
- **5, containers:** built: allocation prices are `price(capacity)`;
  `Slab` and `Deadlines` expose table prices, and `RequestTable` exposes
  `price(&limits)`.
- **11, the journal:** built: uncapped decisions use `push_write` and
  `push_held`; `JournalLimits` holds only `release`.
