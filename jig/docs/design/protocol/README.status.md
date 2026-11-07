# README: status

2026-11-07. Where jig's code lags or departs from [README.md](README.md).

- **Not yet:** 4, 5: agents in a host's own process: no engine or worker of jig's composes `smith-inline-agent`. 10: TLS's server side between hosts, the workers' and clients' listeners being plaintext on loopback or in-process (`jig/crates/jig-protocol-config/src/hosts.rs`); contained process trees for workers' agents.

Section 13's third built-in sign-in is present: the clients' component
holds browser approvals and makes an ordinary sign-in only at the take.
Its expiry departure is in `client.status.md`, section 8; approval between
hosts and listing sign-ins remain `client.md`, section 15's later work.
