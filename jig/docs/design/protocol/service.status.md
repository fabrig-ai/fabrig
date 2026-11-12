# service: status

2026-11-12. Where jig's code lags or departs from [service.md](service.md).


Sections 5 and 10 have no known gap in settling in-process agent claims
before the engine reports drained. The one-process drain story checks
one interrupted terminal at the stop, no claim left to settle at restart
and no lost try. This claim settlement is separate from cancelling the
worker's agents before closing it; the reference-service gap below
remains. A worker stopping in another process retains section 9's
deferred failure accounting.

- **Not yet:** 2, 6, 10: agents in a host's own process (`smith-inline-agent`). 10: a client the process holds, over an in-process transport. 6, 7: contained process trees, and TLS between hosts. 7, 8: `ops`'s service and `ops` standalone: `jig/examples/ops` has no service. 8: the live story fails on a real plan.
- **Not yet:** 10: the test service's workers cancelled before they are closed: `jig/testing/jig-test-service/src/lib.rs` aborts each worker once the engine has ended, killing its agent processes' groups with no cancel first. This remains outside the temper-service repair: the reference agents use skein's fake LLM, with no program of their own to leave running. Temper's service cancels and waits for its agents before closing its worker.
- **Departs:** 11, 11.3: `Workers`' `Debug` counts, but its public `io`, formatted alone, prints a queued write's bytes, a grant's among them, because `Queue` and `io::Request` derive their `Debug`s in skein, which is no part. No service formats it.

Section 7's agent modes are present: `jig-test`'s `main` gives
`--mcp-relay` and `--program-child` to smith's agent shell before it
reads its configuration (`jig/testing/jig-test/src/main.rs`). Section
8's table runs the built binary in each, the program child over a
scripted Claude Code's `auth status`, read `Usable` and not signed in
by smith's status check (`jig/testing/jig-test/tests/process.rs`,
`jig_test_serves_smiths_agent_modes`).

Section 3's reference startup refuses approval off loopback, naming
`engine.clients.approval`, independently of whether the printed link is
enabled. The startup table includes that refusal.

Section 7's reference service prints the printed link alone and drops
the secret `LocalLink` gives (`jig/testing/jig-test-service/src/clients.rs`);
no known gap.
