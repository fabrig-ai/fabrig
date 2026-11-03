# README: status

2026-11-03. Where jig's code lags or departs from [README.md](README.md).

- **Not yet:** 4, 5: agents in a host's own process: no engine or worker of jig's composes `smith-inline-agent`. 10: TLS's server side between hosts, the workers' and clients' listeners being plaintext on loopback or in-process (`jig/crates/jig-protocol-config/src/hosts.rs`); contained process trees for workers' agents. 11: `jig-protocol-transcript` is not in the gate's wasm32 clippy (fab's gate list, `.config/gate/wasm32.txt`).
