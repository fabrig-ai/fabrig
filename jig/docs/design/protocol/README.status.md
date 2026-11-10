# README: status

2026-11-08. Where jig's code lags or departs from [README.md](README.md).

- **Not yet:** 4, 5: agents in a host's own process: no engine or worker of jig's composes `smith-inline-agent`. 10: TLS's server side between hosts, the workers' and clients' listeners being plaintext on loopback or in-process (`jig/crates/jig-protocol-config/src/hosts.rs`); contained process trees for workers' agents.

Section 13's third built-in sign-in is present: the clients' component
holds browser approvals and makes an ordinary sign-in only at the take.
Its expiry departure is in `client/sign-in.status.md`; approval between
hosts and listing sign-ins remain `client/README.md`, section 12's later work.

Section 6's agent command and the agent modes beside it are present in
the reference binary an application copies: `jig-test` dispatches
`--mcp-relay` and `--program-child` before it reads its configuration
(`service.status.md`, section 7).
