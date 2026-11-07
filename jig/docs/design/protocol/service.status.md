# service: status

2026-11-07. Where jig's code lags or departs from [service.md](service.md).

- **Not yet:** 2, 6, 10: agents in a host's own process (`smith-inline-agent`). 10: a client the process holds, over an in-process transport. 6, 7: contained process trees, and TLS between hosts. 7, 8: `ops`'s service and `ops` standalone: `jig/examples/ops` has no service. 8: the live story fails on a real plan.
- **Not yet:** 3: the startup's loopback check for the clients' `approval`, as for `link`.
- **Not yet:** 10: the test service's workers cancelled before they are closed: `jig/testing/jig-test-service/src/lib.rs` aborts each worker once the engine has ended, killing its agent processes' groups with no cancel first. This remains outside the temper-service repair: the reference agents use skein's fake LLM, with no program of their own to leave running. Temper's service cancels and waits for its agents before closing its worker.
- **Departs:** 11, 11.3: `Workers`' `Debug` counts, but its public `io`, formatted alone, prints a queued write's bytes, a grant's among them, because `Queue` and `io::Request` derive their `Debug`s in skein, which is no part. No service formats it.
