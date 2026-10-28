# lib: status

2026-10-28. Where `skein-lib` lags or departs from [lib.md](lib.md).

- **11, the journal:** `Decision::write` and `hold` remain beside `push_write` and `push_held`, always `Ok`, for temper's engine root until it moves to the push forms; `JournalLimits` remains (the journal reads only its `release`) and `Journal::worst_case` with it, for jig's and temper's last readers. All three go with dynamic memory's last set.
- **3, `Slab`:** makes every slot at construction; growing as it fills is in progress.
