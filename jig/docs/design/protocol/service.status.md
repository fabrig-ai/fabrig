# service: status

2026-11-04. Where jig's code lags or departs from [service.md](service.md).

- **Not yet:** 2, 6, 10: agents in a host's own process (`smith-inline-agent`). 10: a client the process holds, over an in-process transport. 6, 7: contained process trees, and TLS between hosts. 7, 8: `ops`'s service and `ops` standalone: `jig/examples/ops` has no service. 8: the live story fails on a real plan.

- **Not yet:** 11.2: `route::Invalid` and its two copies stand where `jig_domain_host::Undecoded` belongs.

- **Not yet:** 11.4: `jig_protocol_engine_clients_http::bundle`: each service's bundle holds its own copy.

- **Not yet:** 11, 11.5: `Drivers` is named `Host`, and `Hosted` has no `mark_durable`.
