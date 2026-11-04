# service: status

2026-11-04. Where jig's code lags or departs from [service.md](service.md).

- **Not yet:** 2, 6, 10: agents in a host's own process (`smith-inline-agent`). 10: a client the process holds, over an in-process transport. 6, 7: contained process trees, and TLS between hosts. 7, 8: `ops`'s service and `ops` standalone: `jig/examples/ops` has no service. 8: the live story fails on a real plan.
- **Not yet:** 11.3: `Workers`, `Component`, `Request` and `Event` derive `Debug`.
- **Not yet:** 11.6: `address_text` stands in `jig_protocol_engine_clients::host`.
