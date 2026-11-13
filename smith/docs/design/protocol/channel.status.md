# channel: status

2026-11-13. Where smith's code lags or departs from [channel.md](channel.md).


- **Not yet:** 2: `schema/` holds `channel-v6.schema` and `channel-v7.schema` beside the current text, and `src/generated/v7.rs` stays beside `v8.rs` [in flight: change 2447].
- **Not yet:** the workspace's files left in conflict: the schema allows 64 in a directory (`channel-v8.schema`), and the run limits' `conflicts` is fixed at 64 (`smith-domain-run/src/limits.rs`, `derive`).

Sections 3 and 4 have no known implementation gap in the save ask in channel version 6.
