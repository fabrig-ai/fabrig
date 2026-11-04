# service: status

2026-11-04. Where jig's code lags or departs from [service.md](service.md).

- **Not yet:** 2, 6, 10: agents in a host's own process (`smith-inline-agent`). 10: a client the process holds, over an in-process transport. 6, 7: contained process trees, and TLS between hosts. 7, 8: `ops`'s service and `ops` standalone: `jig/examples/ops` has no service. 8: the live story fails on a real plan.

- **Not yet:** 11.4: `jig_protocol_engine_clients_http::bundle`: temper's bundle still writes its own file kinds, digest-path lookup and reply chunking (`temper/crates/temper-service/src/clients/bundle.rs`; `jig/docs/development/debt/D005-accounts-host-copied.md`).

- **Not yet:** 11, 11.5: `Drivers` is named `Host`, and `Hosted` has no `mark_durable`.

- **Not yet:** 11: all seven parts ship and both services compose them, but the services' shared distinct lines read 356 against the bound of 200, largely each one's own pass and adapters (`temper/docs/development/debt/D001-reference-service-copy.md`).
