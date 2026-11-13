# channel: status

2026-11-13. Where smith's code lags or departs from [channel.md](channel.md).

- **Not yet:** 4, *How a run ended*: a model failure's report carries no status, and the channel is version 7 [in flight: change 31].

- **Not yet:** the workspace's files left in conflict: the schema allows 64 in a directory (`channel-v8.schema`), and the run limits' `conflicts` is fixed at 64 (`smith-domain-run/src/limits.rs`, `derive`).

Sections 3 and 4 have no known implementation gap in the save ask in channel version 6.
