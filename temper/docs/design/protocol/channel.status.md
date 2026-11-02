# channel: status

2026-11-02. Where temper's code lags or departs from [channel.md](channel.md).

- **Departs:** 4, 6, 7, 8: a remote's `git` base, a git identity's login and author, and the bounds that read them come from each forge's store record, not from configuration's `forges` and `identities`, because forges are store records and the playground is reset (`../draft/playground-reset.md`, 3, 4, 6). 8: a lent credential's bound, keys included, is `accounts.token` alone on the engine and the worker, checked against smith's credential and program-name bodies only (`temper-service/src/worker_settings.rs`, `startup/checks.rs`), because jig's key bound is that token bound capped by the client wire's keyed value (jig's `jig-protocol-config/src/core.rs`) and temper sends no withdrawn value.
