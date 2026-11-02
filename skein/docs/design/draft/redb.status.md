# redb: status

2026-11-02. Where the design lags or departs from [redb.md](redb.md).

- **Design departs:** the whole: skein's store is `../kv.md`'s own `no_std` machine, its map in memory with a commit log and snapshots, not redb on the loop, because a dataset that fits in memory needs only a log and snapshots, whose recovery is tested by crashing at every operation, where redb's crash-safety would rest on its internal call patterns (kv.md, 1; decided with kv.md, 2026-10-05).
- **Open:** whether to take this way past memory, after retention and values on disk (kv.md, 12).
