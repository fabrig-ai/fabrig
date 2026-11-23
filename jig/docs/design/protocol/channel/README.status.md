# channel: status

2026-11-21. Where jig's code lags or departs from [README.md](README.md).



- **Not yet:** 10: the application's parts' ceilings: a run's saved work is at most 65,536 bytes (`worker-wire-v9.schema`, `Work`), not 16 MiB.


- **Not yet:** 1, 4.1, 7: TLS between hosts: the workers' listener and the worker's dial are plaintext on loopback or in-process (`jig/crates/jig-protocol-config/src/hosts.rs`, `address.rs`). 3, 7: a worker whose root composes `smith-inline-agent`: none does, so no worker gives a tool begun with its call.

