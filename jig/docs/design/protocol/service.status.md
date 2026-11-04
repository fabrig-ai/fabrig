# service: status

2026-11-04. Where jig's code lags or departs from [service.md](service.md).

- **Not yet:** 2, 6, 10: agents in a host's own process (`smith-inline-agent`). 10: a client the process holds, over an in-process transport. 6, 7: contained process trees, and TLS between hosts. 7, 8: `ops`'s service and `ops` standalone: `jig/examples/ops` has no service. 8: the live story fails on a real plan. 11: the user's request of 2026-11-04 has jig's D005 and temper's D001 both paid and gone. D005 went with `bundle` (11.4), but D001 stays open: all seven parts ship and both services compose them, yet the services' shared distinct lines read 355 against its bound of 200, largely each one's own pass and adapters (`temper/docs/development/debt/D001-reference-service-copy.md`), and what would take them below it awaits the user (section 9).
- **Departs:** 11, 11.3: `Workers`' `Debug` counts, but its public `io`, formatted alone, prints a queued write's bytes, a grant's among them, because `Queue` and `io::Request` derive their `Debug`s in skein, which is no part. No service formats it.
